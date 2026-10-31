use crate::jobs::JobId;
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::ai::AiBackend;
use gicio_core::review::Snapshot;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> Vec<crate::jobs::JobInfo> {
    state.jobs.list()
}

#[tauri::command]
pub fn cancel_job(app: AppHandle, state: State<'_, AppState>, id: JobId) {
    state.jobs.cancel(&app, &id);
}

/// Switches engine, model or reasoning effort for the following AI calls.
#[tauri::command]
pub fn set_backend(app: AppHandle, state: State<'_, AppState>, backend: AiBackend) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.state.backend = backend;
    s.touch();
    Ok(publish(&app, &s))
}

/// Which AI command-line tools can be found (the app's PATH plus the usual install locations).
#[tauri::command]
pub fn ai_tools() -> std::collections::HashMap<&'static str, bool> {
    ["claude", "codex", "copilot"].into_iter().map(|t| (t, gicio_core::ai::resolve_program(t).is_some())).collect()
}
