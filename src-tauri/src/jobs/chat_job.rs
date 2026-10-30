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
                    (ai::build_summary_prompt(&s, &extra), AccessMode::ReadOnly)
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
                        let after = s.state.chunks.last().map(|c| c.id.clone());
                        let short: String = title.chars().take(60).collect();
                        if let Ok(r) = s.refresh(Some((after, format!("From chat: {short}"), None)))
                            && !r.touched_files.is_empty()
                        {
                            let files = r.touched_files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ");
                            s.state.chat.push(Message::new(Author::System, MessageKind::CodeChanged, format!("Code changed in {files}; added as a step at the end of the queue.")));
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
