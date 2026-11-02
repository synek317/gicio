use super::{JobKind, JobStatus, Jobs};
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::review::{Author, Message, MessageKind};
use tauri::{AppHandle, Manager};

/// What the Chat tab asked for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChatRequest {
    Message { allow_edits: bool },
    Summary,
}

/// Runs a chat reply or a change summary in the background.
pub fn spawn_chat_job(app: AppHandle, req: ChatRequest, title: String) {
    let kind = if req == ChatRequest::Summary { JobKind::Summary } else { JobKind::Chat };
    let state = app.state::<AppState>();
    let job = state.jobs.add(&app, kind, title.clone(), None, JobStatus::Running);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let extra = state.settings.instructions();
        let prepared = (|| -> Result<_, String> {
            let mut s = state.session()?;
            let (prompt, mode) = match req {
                ChatRequest::Summary => {
                    s.summary_busy = true;
                    (ai::build_summary_prompt(&s, &extra, &state.settings.skip_filter()), AccessMode::ReadOnly)
                }
                ChatRequest::Message { allow_edits } => {
                    s.chat_busy = true;
                    (ai::build_chat_prompt(&s, &extra, allow_edits), if allow_edits { AccessMode::Edit } else { AccessMode::ReadOnly })
                }
            };
            s.touch();
            publish(&app, &s);
            Ok((s.state.backend.clone(), s.state.repo.clone(), prompt, mode))
        })();
        let (backend, repo, prompt, mode) = match prepared {
            Ok(p) => p,
            Err(e) => {
                state.jobs.log(&app, &job, e);
                return state.jobs.set_status(&app, &job, JobStatus::Failed);
            }
        };
        // What the working tree looked like, to show (and undo) what this answer changed.
        let before = if mode == AccessMode::Edit { gicio_core::git::worktree_tree(&repo).ok() } else { None };
        let cancel = state.jobs.start(&app, &job);
        let answer = ai::run_streaming(&backend, &repo, &prompt, mode, Jobs::sink(&app, &job), Some(cancel)).await;
        let status = match &answer {
            Ok(_) => JobStatus::Done,
            Err(e) if e.to_string().contains("stopped") => JobStatus::Cancelled,
            Err(_) => JobStatus::Failed,
        };
        if let Ok(mut s) = state.session() {
            let msg = match answer {
                Ok(text) => Message::new(Author::Ai { backend: backend.label() }, MessageKind::Text, text.trim()),
                Err(e) => Message::new(Author::System, MessageKind::Error, err(e)),
            };
            match req {
                ChatRequest::Summary => {
                    s.summary_busy = false;
                    if msg.kind == MessageKind::Text {
                        s.state.summary = Some(msg);
                    } else {
                        s.state.chat.push(msg);
                    }
                }
                ChatRequest::Message { allow_edits } => {
                    s.chat_busy = false;
                    s.state.chat.push(msg);
                    if allow_edits {
                        let short: String = title.chars().take(60).collect();
                        let _ = s.refresh(Some((None, format!("From chat: {short}"), None)));
                        if let Some(change) = s.record_change(before, &title, &backend.label(), None) {
                            let body = format!("Code changed in {}; added as a step at the end.", s.change_files(&change));
                            s.state.chat.push(Message::new(Author::System, MessageKind::CodeChanged, body).with_change(Some(change)));
                        }
                    }
                }
            }
            s.touch();
            publish(&app, &s);
        }
        state.jobs.set_status(&app, &job, status);
    });
}
