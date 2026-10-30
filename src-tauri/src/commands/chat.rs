use crate::jobs::{spawn_chat_job, ChatRequest};
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::review::{Author, Message, MessageKind, Snapshot};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn chat_send(app: AppHandle, state: State<'_, AppState>, body: String, allow_edits: bool) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    if s.chat_busy {
        return Err("the AI is still answering".into());
    }
    s.state.chat.push(Message::new(Author::Human, MessageKind::Text, body.trim()));
    s.touch();
    let snap = publish(&app, &s);
    drop(s);
    spawn_chat_job(app, ChatRequest::Message { allow_edits }, body.trim().to_string());
    Ok(snap)
}

#[tauri::command]
pub fn summarize(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.session()?.summary_busy {
        return Err("a summary is already being written".into());
    }
    spawn_chat_job(app, ChatRequest::Summary, "Summary of the change".into());
    Ok(())
}

#[tauri::command]
pub fn clear_chat(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.state.chat.clear();
    s.touch();
    Ok(publish(&app, &s))
}
