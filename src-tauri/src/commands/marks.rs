use crate::publish::publish;
use crate::state::AppState;
use gicio_core::review::{LineKey, MarkStatus, Snapshot};
use tauri::{AppHandle, State};

/// `status: null` resets lines back to pending.
#[tauri::command]
pub fn set_marks(app: AppHandle, state: State<'_, AppState>, keys: Vec<LineKey>, status: Option<MarkStatus>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.set_marks(keys, status);
    Ok(publish(&app, &s))
}

#[tauri::command]
pub fn undo(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.undo();
    Ok(publish(&app, &s))
}
