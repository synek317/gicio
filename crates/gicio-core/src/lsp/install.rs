use super::ServerKind;
use crate::ai::{login_path, resolve_program};
use crate::process::async_command;
use crate::tools::tail;
use anyhow::{bail, Context, Result};
use std::process::Stdio;

/// Whether the server is there and runs. Being on PATH is not enough: rustup puts a
/// `rust-analyzer` proxy there that fails until the component is added.
pub async fn is_installed(kind: ServerKind) -> bool {
    let Some(program) = resolve_program(kind.name()) else { return false };
    async_command(program).arg("--version").env("PATH", login_path()).stdin(Stdio::null()).output().await.is_ok_and(|o| o.status.success())
}

/// Whether gicio can install the server itself: its installer (rustup, dotnet) is available.
pub fn can_install(kind: ServerKind) -> bool {
    resolve_program(kind.installer()).is_some_and(|p| p.is_file())
}

/// Installs the server with its installer and checks that it runs afterwards.
pub async fn install(kind: ServerKind) -> Result<()> {
    let installer = resolve_program(kind.installer()).with_context(|| format!("{} not found: install {} first", kind.installer(), kind.prerequisite()))?;
    let out = async_command(installer)
        .args(kind.install_args())
        .env("PATH", login_path())
        .env("DOTNET_CLI_UI_LANGUAGE", "en")
        .env("DOTNET_NOLOGO", "1")
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("failed to run {}", kind.install_command()))?;
    if !out.status.success() {
        bail!("{} failed: {}", kind.install_command(), tail(if out.stderr.is_empty() { &out.stdout } else { &out.stderr }));
    }
    if !is_installed(kind).await {
        bail!("{} finished, but {} still does not run", kind.install_command(), kind.name());
    }
    Ok(())
}
