use crate::publish::err;
use crate::state::{AppState, LspSlot};
use gicio_core::domain::{FilePath, RepoRoot};
use gicio_core::lsp::{LspStatus, RustAnalyzer, SourceLocation, Symbol};
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

#[tauri::command]
pub async fn lsp_symbols(state: State<'_, AppState>, path: FilePath) -> Result<Vec<Symbol>, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    ra.symbols(&path).await.map_err(err)
}

#[tauri::command]
pub async fn lsp_prepare_rename(state: State<'_, AppState>, path: FilePath, line: u32, character: u32) -> Result<Option<String>, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    ra.prepare_rename(&path, line, character).await.map_err(err)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameResult {
    files: usize,
    edits: usize,
}

/// Renames the symbol everywhere (rust-analyzer), writes the files and re-reads the diff.
#[tauri::command]
pub async fn lsp_rename(app: tauri::AppHandle, state: State<'_, AppState>, path: FilePath, line: u32, character: u32, new_name: String, keep_accepted: bool, after: Option<gicio_core::domain::ChunkId>) -> Result<RenameResult, String> {
    let ra = state.lsp().ok_or("rust-analyzer is not running")?;
    let name = gicio_core::lsp::NewName::new(&new_name).map_err(err)?;
    let ops = ra.rename(&path, line, character, &name).await.map_err(err)?;
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
    ra.files_changed(&paths).await;
    Ok(RenameResult { files: paths.len(), edits })
}
