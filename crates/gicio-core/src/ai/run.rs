use super::command::{prepare, Output};
use super::events::{parse_claude, ClaudeEvent};
use super::{AccessMode, AiBackend};
use crate::domain::RepoRoot;
use anyhow::{anyhow, bail, Context, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

const TIMEOUT: Duration = Duration::from_secs(45 * 60);

/// Receives live activity lines from a running AI call.
pub type LogSink = Box<dyn Fn(String) + Send + Sync>;

/// Runs one non-interactive AI turn in the repository and returns its final message.
pub async fn run(backend: &AiBackend, repo: &RepoRoot, prompt: &str, mode: AccessMode) -> Result<String> {
    run_streaming(backend, repo, prompt, mode, Box::new(|_| {}), None).await
}

/// Like [`run`], reporting activity to `log` and stopping early when `cancel` fires.
pub async fn run_streaming(backend: &AiBackend, repo: &RepoRoot, prompt: &str, mode: AccessMode, log: LogSink, cancel: Option<oneshot::Receiver<()>>) -> Result<String> {
    let prepared = prepare(backend, repo, prompt, mode)?;
    let mut cmd = prepared.cmd;
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    let mut child = cmd.spawn().with_context(|| format!("failed to start {}", backend.label()))?;
    let mut stdin = child.stdin.take().context("no stdin")?;
    if let Some(p) = prepared.stdin {
        stdin.write_all(p.as_bytes()).await?;
    }
    drop(stdin);

    let (tx, mut rx) = mpsc::unbounded_channel::<(bool, String)>();
    let out = BufReader::new(child.stdout.take().context("no stdout")?);
    let err = BufReader::new(child.stderr.take().context("no stderr")?);
    for (is_err, reader) in [(false, Box::new(out) as Box<dyn tokio::io::AsyncBufRead + Unpin + Send>), (true, Box::new(err))] {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                let _ = tx.send((is_err, l));
            }
        });
    }
    drop(tx);

    let claude = matches!(prepared.output, Output::ClaudeStream);
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut result: Option<Result<String, String>> = None;
    let mut cancel = cancel;
    let deadline = tokio::time::sleep(TIMEOUT);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            line = rx.recv() => {
                let Some((is_err, line)) = line else { break };
                if is_err {
                    stderr.push_str(&line);
                    stderr.push('\n');
                    if !claude && !line.trim().is_empty() { log(line); }
                } else if claude {
                    match parse_claude(&line) {
                        ClaudeEvent::Log(l) => log(l),
                        ClaudeEvent::Result(r) => result = Some(r),
                        ClaudeEvent::Nothing => {}
                    }
                } else {
                    stdout.push_str(&line);
                    stdout.push('\n');
                    if !line.trim().is_empty() { log(line); }
                }
            }
            _ = async { match cancel.as_mut() { Some(c) => { let _ = c.await; } None => std::future::pending::<()>().await } } => {
                let _ = child.kill().await;
                bail!("stopped by you");
            }
            _ = &mut deadline => {
                let _ = child.kill().await;
                bail!("AI call timed out");
            }
        }
    }
    let status = child.wait().await?;

    match prepared.output {
        Output::ClaudeStream => match result {
            Some(Ok(text)) => Ok(text),
            Some(Err(e)) => Err(anyhow!("claude error: {e}")),
            None => bail!("{} failed ({status}): {}", backend.label(), tail(&stderr)),
        },
        Output::LastMessageFile(file) => {
            let msg = std::fs::read_to_string(&file).unwrap_or_default();
            let _ = std::fs::remove_file(&file);
            if msg.trim().is_empty() {
                if !status.success() {
                    bail!("{} failed ({status}): {}", backend.label(), tail(&stderr));
                }
                return Ok(stdout);
            }
            Ok(msg)
        }
        Output::Stdout => {
            if !status.success() {
                bail!("{} failed ({status}): {}", backend.label(), tail(if stderr.trim().is_empty() { &stdout } else { &stderr }));
            }
            Ok(stdout)
        }
    }
}

fn tail(s: &str) -> String {
    let s = s.trim();
    s.chars().rev().take(1500).collect::<Vec<_>>().into_iter().rev().collect()
}
