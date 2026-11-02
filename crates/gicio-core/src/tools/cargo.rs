use super::tail::tail;
use super::ToolDiagnostic;
use crate::ai::{login_path, resolve_program};
use crate::domain::{FilePath, RepoRoot};
use crate::process::async_command;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;

async fn cargo(root: &RepoRoot, args: &[&str]) -> Result<std::process::Output> {
    if !root.path().join("Cargo.toml").exists() {
        bail!("not a Cargo project (no Cargo.toml at the repository root)");
    }
    let cargo = resolve_program("cargo").context("cargo not found: install Rust (https://rustup.rs)")?;
    async_command(cargo).args(args).current_dir(root.path()).env("PATH", login_path()).output().await.context("failed to run cargo")
}

pub async fn cargo_fmt(root: &RepoRoot) -> Result<()> {
    let out = cargo(root, &["fmt", "--all"]).await?;
    if !out.status.success() {
        bail!("cargo fmt failed: {}", tail(&out.stderr));
    }
    Ok(())
}

pub async fn cargo_clippy_fix(root: &RepoRoot) -> Result<()> {
    let out = cargo(root, &["clippy", "--fix", "--allow-dirty", "--allow-staged", "--workspace", "--all-targets"]).await?;
    if !out.status.success() {
        bail!("cargo clippy --fix failed: {}", tail(&out.stderr));
    }
    Ok(())
}

/// Clippy warnings and errors located in the repository.
pub async fn cargo_clippy(root: &RepoRoot) -> Result<Vec<ToolDiagnostic>> {
    let out = cargo(root, &["clippy", "--workspace", "--all-targets", "--message-format=json"]).await?;
    let mut found = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        if v.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let msg = &v["message"];
        let level = msg["level"].as_str().unwrap_or("warning").to_string();
        if !matches!(level.as_str(), "warning" | "error") {
            continue;
        }
        let Some(span) = msg["spans"].as_array().and_then(|s| s.iter().find(|s| s["is_primary"].as_bool() == Some(true))) else { continue };
        let file = span["file_name"].as_str().unwrap_or_default();
        let rel = Path::new(file).strip_prefix(root.path()).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| file.to_string());
        let Ok(path) = FilePath::new(&rel) else { continue };
        found.push(ToolDiagnostic {
            path,
            line: span["line_start"].as_u64().unwrap_or(1) as u32,
            end_line: span["line_end"].as_u64().unwrap_or(1) as u32,
            level,
            title: msg["message"].as_str().unwrap_or_default().to_string(),
            detail: msg["rendered"].as_str().unwrap_or_default().to_string(),
        });
    }
    if found.is_empty() && !out.status.success() {
        bail!("cargo clippy failed: {}", tail(&out.stderr));
    }
    Ok(found)
}
