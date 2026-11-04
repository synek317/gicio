use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::domain::CommitMessage;
use gicio_core::git;
use gicio_core::review::Snapshot;
use serde::Deserialize;
use tauri::{AppHandle, State};

/// What to do with the accepted changes once the review is finished.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinishAction {
    Nothing,
    Stage,
    Commit,
    Amend,
}

/// Stages exactly the accepted lines, whatever the stage mode (e.g. before suggesting a message).
#[tauri::command]
pub fn stage_accepted(state: State<'_, AppState>) -> Result<(), String> {
    state.session()?.stage_accepted().map_err(err)
}

/// Message of the last commit, to start an amend from.
#[tauri::command]
pub fn last_commit_message(state: State<'_, AppState>) -> Result<String, String> {
    git::head_message(&state.session()?.state.repo).map_err(err)
}

/// Applies `action` to the accepted changes; returns what happened, for the reviewer.
#[tauri::command]
pub fn finish_review(app: AppHandle, state: State<'_, AppState>, action: FinishAction, message: Option<CommitMessage>) -> Result<(String, Snapshot), String> {
    let mut s = state.session()?;
    let needs_message = || message.as_ref().ok_or("a commit message is required");
    let done = match action {
        FinishAction::Nothing => "Review finished".to_string(),
        FinishAction::Stage => {
            s.stage_accepted().map_err(err)?;
            "Accepted changes staged".to_string()
        }
        FinishAction::Commit => {
            let message = needs_message()?;
            s.stage_accepted().map_err(err)?;
            format!("Committed {}", git::commit(&s.state.repo, message).map_err(err)?)
        }
        FinishAction::Amend => {
            let message = needs_message()?;
            s.stage_accepted().map_err(err)?;
            format!("Amended the last commit: {}", git::amend(&s.state.repo, message).map_err(err)?)
        }
    };
    s.touch();
    Ok((done, publish(&app, &s)))
}

/// Closes gicio.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
