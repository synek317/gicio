use crate::publish::err;
use crate::state::{AppState, LspSlot};
use gicio_core::domain::{FilePath, RepoRoot};
use gicio_core::lsp::{LspStatus, RustAnalyzer, SourceLocation};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// Starts rust-analyzer for Cargo projects; replaces any previous instance.
pub fn start_lsp(app: AppHandle, root: RepoRoot) {
    let state = app.state::<AppState>();
    if !root.path().join("Cargo.toml").exists() {
        *state.lsp.lock().unwrap() = LspSlot::Off;
        return;
    }
    *state.lsp.lock().unwrap() = LspSlot::Starting;
    tauri::async_runtime::spawn(async move {
        let slot = match RustAnalyzer::start(root).await {
            Ok(ra) => LspSlot::Running(Arc::new(ra)),
            Err(e) => LspSlot::Failed(err(e)),
        };
        *app.state::<AppState>().lsp.lock().unwrap() = slot;
    });
}

#[tauri::command]
pub fn lsp_status(state: State<'_, AppState>) -> LspStatus {
    match &*state.lsp.lock().unwrap() {
        LspSlot::Off => LspStatus::Off,
        LspSlot::Starting => LspStatus::Starting,
        LspSlot::Running(ra) => ra.status(),
        LspSlot::Failed(e) => LspStatus::Failed { detail: e.clone() },
    }
}

#[tauri::command]
pub async fn lsp_hover(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Option<String>, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    ra.hover(&path, line, character).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_definition(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    ra.definition(&path, line, character).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_references(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    ra.references(&path, line, character).await.map_err(err)
}
