use crate::jobs::{run_ai_job, JobKind};
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::git;
use gicio_core::review::Snapshot;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn staged_stat(state: State<'_, AppState>) -> Result<String, String> {
    git::staged_stat(&state.session()?.state.repo).map_err(err)
}

#[tauri::command]
pub async fn suggest_commit_message(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let (prompt, backend, repo) = {
        let s = state.session()?;
        let staged = git::staged_diff(&s.state.repo).map_err(err)?;
        if staged.trim().is_empty() {
            return Err("nothing is staged".into());
        }
        (ai::build_commit_prompt(&s, &staged, &state.settings.instructions()), s.state.backend.clone(), s.state.repo.clone())
    };
    let msg = run_ai_job(&app, JobKind::Commit, "Commit message", &backend, &repo, &prompt, AccessMode::ReadOnly).await?;
    Ok(msg.trim().trim_matches('`').trim().to_string())
}

/// Commits what is staged (in index mode: exactly the accepted lines).
#[tauri::command]
pub fn commit(app: AppHandle, state: State<'_, AppState>, message: String) -> Result<(String, Snapshot), String> {
    let mut s = state.session()?;
    if message.trim().is_empty() {
        return Err("empty commit message".into());
    }
    let sha = git::commit(&s.state.repo, message.trim()).map_err(err)?;
    s.touch();
    Ok((sha, publish(&app, &s)))
}
