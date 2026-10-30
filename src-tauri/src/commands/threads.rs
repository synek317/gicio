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
    /// Just a note, no AI.
    Note,
    /// The AI answers now (and may edit code if allowed).
    Ask,
    /// Queued background task for the AI; its changes go to the end of the queue.
    Todo,
    /// Kept as a pending review comment until "Submit review".
    Review,
}

fn excerpt(s: &Session, a: &AnchorInput) -> String {
    let Some(file) = s.file(&a.path) else { return String::new() };
    let mut lines: Vec<(u32, &str)> = file
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .filter_map(|l| {
            let no = match a.side {
                Side::Old => l.old_no,
                Side::New => l.new_no,
            }?;
            (a.start..=a.end).contains(&no).then_some((no, l.text.as_str()))
        })
        .collect();
    lines.sort_by_key(|(n, _)| *n);
    lines.dedup_by_key(|(n, _)| *n);
    lines.into_iter().map(|(_, t)| t).collect::<Vec<_>>().join("\n")
}

#[tauri::command]
pub fn create_thread(app: AppHandle, state: State<'_, AppState>, chunk: Option<ChunkId>, anchor: Option<AnchorInput>, allow_edits: bool, todo: bool) -> Result<ThreadId, String> {
    let mut s = state.session()?;
    let anchor = anchor.map(|a| Anchor { excerpt: excerpt(&s, &a), path: a.path, side: a.side, start: a.start, end: a.end });
    let id = s.create_thread(chunk, anchor, allow_edits, todo);
    publish(&app, &s);
    Ok(id)
}

#[tauri::command]
pub fn set_thread_status(app: AppHandle, state: State<'_, AppState>, thread: ThreadId, status: ThreadStatus) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.thread_mut(&thread).map_err(err)?.status = status;
    Ok(publish(&app, &s))
}

/// Adds the reviewer's message and, unless it is a note, starts an AI job for it.
#[tauri::command]
pub fn send_message(app: AppHandle, state: State<'_, AppState>, thread: ThreadId, body: String, mode: SendMode, allow_edits: Option<bool>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.ensure_not_busy(&thread).map_err(err)?;
    let mut msg = Message::new(Author::Human, MessageKind::Text, body.trim());
    msg.pending = mode == SendMode::Review;
    s.push_message(&thread, msg).map_err(err)?;
    let t = s.thread_mut(&thread).map_err(err)?;
    t.status = ThreadStatus::Open;
    if let Some(a) = allow_edits {
        t.allow_edits = a;
    }
    if mode == SendMode::Todo {
        t.todo = true;
        t.busy = true;
    }
    let snap = publish(&app, &s);
    drop(s);
    if matches!(mode, SendMode::Ask | SendMode::Todo) {
        spawn_thread_job(app, thread, body.trim().to_string(), mode == SendMode::Todo);
    }
    Ok(snap)
}
