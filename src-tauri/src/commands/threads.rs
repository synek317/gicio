use crate::jobs::spawn_thread_job;
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::diff::Side;
use gicio_core::domain::{ChunkId, FilePath, ThreadId};
use gicio_core::review::{Anchor, Author, Message, MessageKind, Session, Snapshot, ThreadStatus};
use serde::Deserialize;
use tauri::{AppHandle, State};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorInput {
    path: FilePath,
    side: Side,
    start: u32,
    end: u32,
}

/// What happens with a reviewer's message.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SendMode {
    /// The AI answers now (and may edit code if allowed).
    Ask,
    /// Left for later: the "resolve later" step hands all of these to the AI at once.
    Later,
}

fn excerpt(s: &Session, a: &AnchorInput) -> String {
    s.excerpt_of(&a.path, a.side, a.start, a.end)
}

#[tauri::command]
pub fn create_thread(app: AppHandle, state: State<'_, AppState>, chunk: Option<ChunkId>, anchor: Option<AnchorInput>, allow_edits: bool) -> Result<ThreadId, String> {
    let mut s = state.session()?;
    let anchor = anchor.map(|a| Anchor { excerpt: excerpt(&s, &a), path: a.path, side: a.side, start: a.start, end: a.end, outdated: false });
    let id = s.create_thread(chunk, anchor, allow_edits);
    publish(&app, &s);
    Ok(id)
}

#[tauri::command]
pub fn set_thread_status(app: AppHandle, state: State<'_, AppState>, thread: ThreadId, status: ThreadStatus) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.thread_mut(&thread).map_err(err)?.status = status;
    Ok(publish(&app, &s))
}

/// Adds the reviewer's message and either starts an AI job for it or leaves the thread for later.
#[tauri::command]
pub fn send_message(app: AppHandle, state: State<'_, AppState>, thread: ThreadId, body: String, mode: SendMode, allow_edits: Option<bool>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.ensure_not_busy(&thread).map_err(err)?;
    s.push_message(&thread, Message::new(Author::Human, MessageKind::Text, body.trim())).map_err(err)?;
    let t = s.thread_mut(&thread).map_err(err)?;
    t.status = if mode == SendMode::Later { ThreadStatus::Deferred } else { ThreadStatus::Open };
    if let Some(a) = allow_edits {
        t.allow_edits = a;
    }
    let snap = publish(&app, &s);
    drop(s);
    if mode == SendMode::Ask {
        spawn_thread_job(app, thread, body.trim().to_string());
    }
    Ok(snap)
}
