use super::{JobId, JobKind, JobStatus, Jobs};
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::domain::{ThreadId, TreeId};
use gicio_core::review::{Author, Message, MessageKind};
use tauri::{AppHandle, Manager};

/// Runs the AI on a thread's latest message in the background. It answers right away and puts
/// code changes right after the discussed step.
pub fn spawn_thread_job(app: AppHandle, thread: ThreadId, title: String) {
    let job = app.state::<AppState>().jobs.add(&app, JobKind::Ask, title.clone(), Some(thread.to_string()), JobStatus::Running);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let prepared = (|| -> Result<_, String> {
            let mut s = state.session()?;
            let prompt = ai::build_ask_prompt(&s, &thread, &state.settings.instructions()).map_err(err)?;
            let t = s.thread_mut(&thread).map_err(err)?;
            t.busy = true;
            let mode = if t.allow_edits { AccessMode::Edit } else { AccessMode::ReadOnly };
            publish(&app, &s);
            Ok((s.state.backend.clone(), s.state.repo.clone(), prompt, mode))
        })();
        let (backend, repo, prompt, mode) = match prepared {
            Ok(p) => p,
            Err(e) => return finish(&app, &thread, &job, Err(e), &title, None),
        };
        // What the working tree looked like, to show (and undo) what this answer changed.
        let before = if mode == AccessMode::Edit { gicio_core::git::worktree_tree(&repo).ok() } else { None };
        let cancel = state.jobs.start(&app, &job);
        let answer = ai::run_streaming(&backend, &repo, &prompt, mode, Jobs::sink(&app, &job), Some(cancel)).await;
        finish(&app, &thread, &job, answer.map_err(err).map(|a| (a, backend.label())), &title, before);
    });
}

fn finish(app: &AppHandle, thread: &ThreadId, job: &JobId, answer: Result<(String, String), String>, title: &str, before: Option<TreeId>) {
    let state = app.state::<AppState>();
    let status = match &answer {
        Ok(_) => JobStatus::Done,
        Err(e) if e.contains("stopped") => JobStatus::Cancelled,
        Err(_) => JobStatus::Failed,
    };
    if let Ok(mut s) = state.session() {
        let msg = match answer {
            Ok((text, label)) => Message::new(Author::Ai { backend: label }, MessageKind::Text, text.trim()),
            Err(e) => Message::new(Author::System, MessageKind::Error, e),
        };
        let _ = s.push_message(thread, msg);
        let after = s.thread(thread).ok().and_then(|t| t.chunk.clone());
        let short: String = title.chars().take(60).collect();
        if let Err(e) = s.refresh(Some((after, format!("Follow-up: {short}"), Some(thread.clone())))) {
            let _ = s.push_message(thread, Message::new(Author::System, MessageKind::Error, format!("refresh failed: {e:#}")));
        }
        let by = s.state.backend.label();
        if let Some(change) = s.record_change(before, title, &by, Some(thread.clone())) {
            let body = format!("Code changed in {} (shown right after this step).", s.change_files(&change));
            let _ = s.push_message(thread, Message::new(Author::System, MessageKind::CodeChanged, body).with_change(Some(change)));
        }
        if let Ok(t) = s.thread_mut(thread) {
            t.busy = false;
        }
        publish(app, &s);
    }
    state.jobs.set_status(app, job, status);
}
