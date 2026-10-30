use crate::commands::lsp::start_lsp;
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::AiBackend;
use gicio_core::diff::{self, FileTexts};
use gicio_core::domain::{ChunkId, FilePath, GitRef};
use gicio_core::review::{Session, Snapshot};
use gicio_core::stage::StageMode;
use std::path::PathBuf;
use tauri::{AppHandle, State};

const MAX_PEEK_BYTES: u64 = 4 * 1024 * 1024;

#[tauri::command]
pub async fn open_session(app: AppHandle, state: State<'_, AppState>, repo: PathBuf, base: GitRef, backend: AiBackend, stage_mode: StageMode) -> Result<Snapshot, String> {
    let session = tauri::async_runtime::spawn_blocking(move || Session::open(&repo, base, backend, stage_mode)).await.map_err(err)?.map_err(err)?;
    let root = session.state.repo.clone();
    let snap = publish(&app, &session);
    state.replace_session(session);
    start_lsp(app.clone(), root);
    Ok(snap)
}

#[tauri::command]
pub fn get_session(state: State<'_, AppState>) -> Result<Snapshot, String> {
    Ok(state.session()?.snapshot())
}

#[tauri::command]
pub fn refresh(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.refresh(None).map_err(err)?;
    Ok(publish(&app, &s))
}

#[tauri::command]
pub fn list_refs(repo: PathBuf) -> Result<Vec<String>, String> {
    let root = gicio_core::git::toplevel(&repo).map_err(err)?;
    gicio_core::git::list_refs(&root).map_err(err)
}

#[tauri::command]
pub fn file_texts(state: State<'_, AppState>, path: FilePath) -> Result<FileTexts, String> {
    let s = state.session()?;
    let file = s.file(&path).ok_or("file is not part of the change")?;
    diff::file_texts(&s.state.repo, &s.state.base_commit, &file.path, file.old_path.as_ref(), file.status).map_err(err)
}

/// Read-only peek at any file (definition targets may live in std or dependencies).
#[tauri::command]
pub fn read_file(abs_path: PathBuf) -> Result<String, String> {
    let meta = std::fs::metadata(&abs_path).map_err(err)?;
    if meta.len() > MAX_PEEK_BYTES {
        return Err("file too large to preview".into());
    }
    std::fs::read_to_string(&abs_path).map_err(err)
}

/// Discards all review progress for this base and starts with a file-by-file queue.
#[tauri::command]
pub fn reset_review(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.reset();
    Ok(publish(&app, &s))
}

/// Saves the reviewer's own edit of a working-tree file.
#[tauri::command]
pub fn save_file(app: AppHandle, state: State<'_, AppState>, path: FilePath, text: String, after: Option<ChunkId>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.save_user_edit(&path, &text, after).map_err(err)?;
    Ok(publish(&app, &s))
}

#[tauri::command]
pub fn set_diff_options(app: AppHandle, state: State<'_, AppState>, options: gicio_core::diff::DiffOptions) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.set_diff_options(options).map_err(err)?;
    Ok(publish(&app, &s))
}

/// Writes an exported file the user picked in a save dialog.
#[tauri::command]
pub fn write_text_file(path: PathBuf, text: String) -> Result<(), String> {
    std::fs::write(path, text).map_err(err)
}
