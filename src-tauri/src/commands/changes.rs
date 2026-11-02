use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::diff::FileDiff;
use gicio_core::domain::{ChangeId, FilePath};
use gicio_core::review::{Author, Message, MessageKind, Session, Snapshot};
use tauri::{AppHandle, State};

/// What one AI job changed: each file before it against right after it.
#[tauri::command]
pub fn change_diff(state: State<'_, AppState>, id: ChangeId) -> Result<Vec<FileDiff>, String> {
    state.session()?.change_diff(&id).map_err(err)
}

/// A note about the reviewer's decision in the conversation the change came from.
fn note(s: &mut Session, id: &ChangeId, body: String) {
    let msg = Message::new(Author::System, MessageKind::Text, body);
    match s.state.changes.iter().find(|c| &c.id == id).and_then(|c| c.thread.clone()) {
        Some(t) => {
            let _ = s.push_message(&t, msg);
        }
        None => s.state.chat.push(msg),
    }
}

/// Keeps an AI change: the lines it wrote count as reviewed.
#[tauri::command]
pub fn accept_change(app: AppHandle, state: State<'_, AppState>, id: ChangeId) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.accept_change(&id).map_err(err)?;
    let files = s.change_files(&id);
    note(&mut s, &id, format!("You kept the changes in {files}."));
    Ok(publish(&app, &s))
}

/// Undoes an AI change; returns the files left alone because they were edited again since.
#[tauri::command]
pub fn reject_change(app: AppHandle, state: State<'_, AppState>, id: ChangeId) -> Result<(Snapshot, Vec<FilePath>), String> {
    let mut s = state.session()?;
    let skipped = s.reject_change(&id).map_err(err)?;
    let files = s.change_files(&id);
    note(&mut s, &id, format!("You undid the changes in {files}."));
    Ok((publish(&app, &s), skipped))
}
