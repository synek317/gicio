use super::transport::{read_message, write_message};
use crate::process::async_command;
use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin};
use tokio::sync::{oneshot, Mutex};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum LspStatus {
    Off,
    Starting,
    Indexing { detail: String },
    Ready,
    Failed { detail: String },
}

type Pending = Arc<StdMutex<HashMap<i64, oneshot::Sender<Result<Value>>>>>;

/// Minimal JSON-RPC client for a language server over stdio.
pub struct LspClient {
    stdin: Arc<Mutex<ChildStdin>>,
    pending: Pending,
    next_id: AtomicI64,
    status: Arc<StdMutex<LspStatus>>,
    _child: Child,
}

impl LspClient {
    pub fn spawn(program: &Path, path_env: &str, cwd: &Path) -> Result<Self> {
        let mut child = async_command(program)
            .env("PATH", path_env)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("failed to start {}", program.display()))?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().context("no stdin")?));
        let stdout = child.stdout.take().context("no stdout")?;
        let pending: Pending = Arc::default();
        let status = Arc::new(StdMutex::new(LspStatus::Starting));
        tokio::spawn(reader_loop(BufReader::new(stdout), stdin.clone(), pending.clone(), status.clone()));
        Ok(Self { stdin, pending, next_id: AtomicI64::new(1), status, _child: child })
    }

    pub fn status(&self) -> LspStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_status(&self, s: LspStatus) {
        *self.status.lock().unwrap() = s;
    }

    pub async fn request(&self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        write_message(&mut *self.stdin.lock().await, &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})).await?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(r) => r.map_err(|_| anyhow!("language server went away"))?,
            Err(_) => {
                self.pending.lock().unwrap().remove(&id);
                Err(anyhow!("{method} timed out"))
            }
        }
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        write_message(&mut *self.stdin.lock().await, &json!({"jsonrpc": "2.0", "method": method, "params": params})).await
    }
}

async fn reader_loop(mut reader: BufReader<tokio::process::ChildStdout>, stdin: Arc<Mutex<ChildStdin>>, pending: Pending, status: Arc<StdMutex<LspStatus>>) {
    let mut active_progress: HashMap<String, String> = HashMap::new();
    loop {
        let msg = match read_message(&mut reader).await {
            Ok(m) => m,
            Err(e) => {
                *status.lock().unwrap() = LspStatus::Failed { detail: e.to_string() };
                for (_, tx) in pending.lock().unwrap().drain() {
                    let _ = tx.send(Err(anyhow!("language server exited")));
                }
                return;
            }
        };
        let method = msg.get("method").and_then(Value::as_str);
        match (msg.get("id"), method) {
            (Some(id), None) => {
                let Some(id) = id.as_i64() else { continue };
                if let Some(tx) = pending.lock().unwrap().remove(&id) {
                    let res = match msg.get("error") {
                        Some(e) => Err(anyhow!("LSP error: {}", e.get("message").and_then(Value::as_str).unwrap_or("?"))),
                        None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                    };
                    let _ = tx.send(res);
                }
            }
            (Some(id), Some(m)) => {
                // Server-to-client request: answer with something harmless.
                let result = match m {
                    "workspace/configuration" => {
                        let n = msg.pointer("/params/items").and_then(Value::as_array).map_or(0, Vec::len);
                        Value::Array(vec![Value::Null; n])
                    }
                    _ => Value::Null,
                };
                let reply = json!({"jsonrpc": "2.0", "id": id, "result": result});
                let _ = write_message(&mut *stdin.lock().await, &reply).await;
            }
            (None, Some("$/progress")) => {
                let token = msg.pointer("/params/token").map(|t| t.to_string()).unwrap_or_default();
                let value = msg.pointer("/params/value").cloned().unwrap_or(Value::Null);
                match value.get("kind").and_then(Value::as_str) {
                    Some("begin") | Some("report") => {
                        let title = value.get("title").and_then(Value::as_str).map(str::to_string).or_else(|| active_progress.get(&token).cloned()).unwrap_or_default();
                        active_progress.insert(token, title);
                    }
                    Some("end") => {
                        active_progress.remove(&token);
                    }
                    _ => {}
                }
                let mut s = status.lock().unwrap();
                if !matches!(*s, LspStatus::Failed { .. } | LspStatus::Starting) {
                    *s = match active_progress.values().next() {
                        Some(t) => LspStatus::Indexing { detail: t.clone() },
                        None => LspStatus::Ready,
                    };
                }
            }
            _ => {}
        }
    }
}
