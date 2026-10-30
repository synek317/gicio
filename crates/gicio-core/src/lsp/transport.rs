//! LSP base protocol framing: `Content-Length` header + JSON body.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::ChildStdout;

pub async fn read_message(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut len = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            bail!("language server closed its output");
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            len = Some(v.trim().parse::<usize>()?);
        }
    }
    let mut body = vec![0; len.context("missing Content-Length")?];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body)?)
}

pub async fn write_message(writer: &mut (impl AsyncWrite + Unpin), msg: &Value) -> Result<()> {
    let body = serde_json::to_vec(msg)?;
    writer.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}
