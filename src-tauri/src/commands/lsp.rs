use crate::jobs::run_tool_job;
use crate::publish::err;
use crate::state::{AppState, LspSlot};
use gicio_core::domain::{FilePath, RepoRoot};
use gicio_core::lsp::{self, LanguageServer, LspStatus, ServerKind, SourceLocation, Symbol};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// Starts the language server for the project (rust-analyzer for Cargo, csharp-ls for .NET);
/// replaces any previous instance. A server that is not installed is reported as missing.
pub fn start_lsp(app: AppHandle, root: RepoRoot) {
    let state = app.state::<AppState>();
    let Some(kind) = ServerKind::detect(&root) else {
        *state.lsp.lock().unwrap() = LspSlot::Off;
        return;
    };
    *state.lsp.lock().unwrap() = LspSlot::Starting(kind);
    tauri::async_runtime::spawn(async move {
        let slot = if !lsp::is_installed(kind).await {
            LspSlot::Missing(kind)
        } else {
            match LanguageServer::start(kind, root).await {
                Ok(ls) => LspSlot::Running(Arc::new(ls)),
                Err(e) => LspSlot::Failed(kind, err(e)),
            }
        };
        *app.state::<AppState>().lsp.lock().unwrap() = slot;
    });
}

/// How a missing server gets installed.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    command: String,
    /// The installer (rustup, dotnet) is there, so gicio can run the command itself.
    available: bool,
    /// What to install first otherwise.
    prerequisite: &'static str,
}

/// The server's state, with which server it is, the files it understands and, when it is
/// missing, how to install it.
#[derive(Serialize)]
pub struct LspInfo {
    #[serde(flatten)]
    status: LspStatus,
    server: Option<&'static str>,
    extensions: &'static [&'static str],
    install: Option<InstallInfo>,
}

impl LspInfo {
    fn new(status: LspStatus, kind: Option<ServerKind>) -> Self {
        let install = kind.filter(|_| matches!(status, LspStatus::Missing)).map(|k| InstallInfo { command: k.install_command(), available: lsp::can_install(k), prerequisite: k.prerequisite() });
        Self { status, server: kind.map(ServerKind::name), extensions: kind.map_or(&[], ServerKind::extensions), install }
    }
}

#[tauri::command]
pub fn lsp_status(state: State<'_, AppState>) -> LspInfo {
    match &*state.lsp.lock().unwrap() {
        LspSlot::Off => LspInfo::new(LspStatus::Off, None),
        LspSlot::Missing(kind) => LspInfo::new(LspStatus::Missing, Some(*kind)),
        LspSlot::Installing(kind) => LspInfo::new(LspStatus::Installing, Some(*kind)),
        LspSlot::Starting(kind) => LspInfo::new(LspStatus::Starting, Some(*kind)),
        LspSlot::Running(ls) => LspInfo::new(ls.status(), Some(ls.kind())),
        LspSlot::Failed(kind, e) => LspInfo::new(LspStatus::Failed { detail: e.clone() }, Some(*kind)),
    }
}

/// Installs the project's missing server (as a job with its own log), then starts it.
#[tauri::command]
pub async fn lsp_install(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let root = state.session()?.state.repo.clone();
    let kind = match &*state.lsp.lock().unwrap() {
        LspSlot::Missing(kind) => *kind,
        _ => return Err("the language server is not missing".into()),
    };
    *state.lsp.lock().unwrap() = LspSlot::Installing(kind);
    if let Err(e) = run_tool_job(&app, &format!("Install {}", kind.name()), lsp::install(kind)).await {
        *state.lsp.lock().unwrap() = LspSlot::Missing(kind);
        return Err(e);
    }
    start_lsp(app.clone(), root);
    Ok(())
}

/// Starts the server again: after it failed, or after it was installed by hand.
#[tauri::command]
pub fn lsp_restart(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let root = state.session()?.state.repo.clone();
    start_lsp(app, root);
    Ok(())
}

/// The running server, if it understands `path`.
fn server_for(state: &AppState, path: &FilePath) -> Result<Arc<LanguageServer>, String> {
    let ls = state.lsp().ok_or("no language server is running")?;
    if !ls.kind().handles(path) {
        return Err(format!("{} does not handle {path}", ls.kind().name()));
    }
    Ok(ls)
}

#[tauri::command]
pub async fn lsp_hover(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Option<String>, String> {
    let ls = server_for(&state, &path)?;
    ls.hover(&path, line, character).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_definition(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>, String> {
    let ls = server_for(&state, &path)?;
    ls.definition(&path, line, character).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_references(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>, String> {
    let ls = server_for(&state, &path)?;
    ls.references(&path, line, character).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_symbols(state: State<'_, AppState>, path: FilePath) -> Result<Vec<Symbol>, String> {
    let ls = server_for(&state, &path)?;
    ls.symbols(&path).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_prepare_rename(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Option<String>, String> {
    let ls = server_for(&state, &path)?;
    ls.prepare_rename(&path, line, character).await.map_err(err)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameResult {
    files: usize,
    edits: usize,
}

/// Renames the symbol everywhere (language server), writes the files and re-reads the diff.
#[tauri::command]
pub async fn lsp_rename(app: tauri::AppHandle, state: State<'_, AppState>, path: FilePath, line: u32, character: u32, new_name: String, keep_accepted: bool, after: Option<gicio_core::domain::ChunkId>) -> Result<RenameResult, String> {
    let ls = server_for(&state, &path)?;
    let name = gicio_core::lsp::NewName::new(&new_name).map_err(err)?;
    let ops = ls.rename(&path, line, character, &name).await.map_err(err)?;
    if ops.is_empty() {
        return Err("nothing to rename here".into());
    }
    let (paths, moved, edits) = {
        let s = state.session()?;
        let rel = |p: &std::path::Path| s.state.repo.relative(p).ok_or_else(|| format!("the rename would change {} outside the repository", p.display()));
        let mut paths = Vec::new();
        let mut moved = Vec::new();
        for op in &ops {
            for p in op.paths() {
                paths.push(rel(p)?);
            }
            if let gicio_core::lsp::FileOp::Rename { from, to } = op {
                moved.push((rel(from)?, rel(to)?));
            }
        }
        paths.sort();
        paths.dedup();
        (paths, moved, ops.iter().map(|o| o.edit_count()).sum())
    };
    {
        let mut s = state.session()?;
        s.record_rename(&paths, &moved, keep_accepted, after, || gicio_core::lsp::apply_file_ops(&ops)).map_err(err)?;
        crate::publish::publish(&app, &s);
    }
    ls.files_changed(&paths).await;
    Ok(RenameResult { files: paths.len(), edits })
}
