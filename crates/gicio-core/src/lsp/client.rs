use super::content_modified::{self, ContentModified};
use super::readiness::Readiness;
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
    readiness: Readiness,
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
        let readiness = Readiness::new();
        tokio::spawn(reader_loop(BufReader::new(stdout), stdin.clone(), pending.clone(), status.clone(), readiness.clone()));
        Ok(Self { stdin, pending, next_id: AtomicI64::new(1), status, readiness, _child: child })
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

    /// Waits until the server has loaded the whole workspace.
    pub async fn wait_until_loaded(&self, timeout: Duration) -> Result<()> {
        self.readiness.wait(timeout).await
    }

    /// Like `request`, but asks again when the server cancels it because the content changed.
    pub async fn request_retrying(&self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        const ATTEMPTS: u32 = 5;
        let mut attempt = 1;
        loop {
            match self.request(method, params.clone(), timeout).await {
                Err(e) if e.is::<ContentModified>() && attempt < ATTEMPTS => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                res => return res,
            }
        }
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        write_message(&mut *self.stdin.lock().await, &json!({"jsonrpc": "2.0", "method": method, "params": params})).await
    }
}

async fn reader_loop(mut reader: BufReader<tokio::process::ChildStdout>, stdin: Arc<Mutex<ChildStdin>>, pending: Pending, status: Arc<StdMutex<LspStatus>>, readiness: Readiness) {
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
                        Some(e) if e.get("code").and_then(Value::as_i64) == Some(content_modified::CODE) => Err(ContentModified.into()),
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
                update_status(&status, &active_progress, &readiness);
            }
            (None, Some("experimental/serverStatus")) => {
                readiness.set(msg.pointer("/params/quiescent").and_then(Value::as_bool).unwrap_or(false));
                update_status(&status, &active_progress, &readiness);
            }
            _ => {}
        }
    }
}

/// Indexing while the workspace is loading or any progress is running, Ready otherwise.
fn update_status(status: &StdMutex<LspStatus>, active_progress: &HashMap<String, String>, readiness: &Readiness) {
    let mut s = status.lock().unwrap();
    if matches!(*s, LspStatus::Failed { .. } | LspStatus::Starting) {
        return;
    }
    *s = match active_progress.values().next() {
        Some(t) => LspStatus::Indexing { detail: t.clone() },
        None if !readiness.is_loaded() => LspStatus::Indexing { detail: "loading workspace".into() },
        None => LspStatus::Ready,
    };
}
