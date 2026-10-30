use super::{JobId, JobKind, JobStatus, Jobs};
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::domain::ThreadId;
use gicio_core::review::{Author, Message, MessageKind};
use tauri::{AppHandle, Manager};

/// Runs the AI on a thread's latest message in the background. A to-do waits for earlier
/// to-dos and lands its code changes at the end of the queue; an ask answers right away and
/// puts changes right after the discussed chunk.
pub fn spawn_thread_job(app: AppHandle, thread: ThreadId, title: String, todo: bool) {
    let kind = if todo { JobKind::Todo } else { JobKind::Ask };
    let status = if todo { JobStatus::Queued } else { JobStatus::Running };
    let job = app.state::<AppState>().jobs.add(&app, kind, title.clone(), Some(thread.to_string()), status);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = if todo { Some(state.jobs.todo_lock.lock().await) } else { None };
        if state.jobs.status(&job) == Some(JobStatus::Cancelled) {
            finish(&app, &thread, &job, Err("stopped before it started".into()), todo, &title);
            return;
        }
        let prepared = (|| -> Result<_, String> {
            let mut s = state.session()?;
            let prompt = ai::build_ask_prompt(&s, &thread, todo, &state.settings.instructions()).map_err(err)?;
            let t = s.thread_mut(&thread).map_err(err)?;
            t.busy = true;
            let mode = if todo || t.allow_edits { AccessMode::Edit } else { AccessMode::ReadOnly };
            publish(&app, &s);
            Ok((s.state.backend.clone(), s.state.repo.clone(), prompt, mode))
        })();
        let (backend, repo, prompt, mode) = match prepared {
            Ok(p) => p,
            Err(e) => return finish(&app, &thread, &job, Err(e), todo, &title),
        };
        let cancel = state.jobs.start(&app, &job);
        let answer = ai::run_streaming(&backend, &repo, &prompt, mode, Jobs::sink(&app, &job), Some(cancel)).await;
        finish(&app, &thread, &job, answer.map_err(err).map(|a| (a, backend.label())), todo, &title);
    });
}

fn finish(app: &AppHandle, thread: &ThreadId, job: &JobId, answer: Result<(String, String), String>, todo: bool, title: &str) {
    let state = app.state::<AppState>();
    let status = match &answer {
        Ok(_) => JobStatus::Done,
        Err(e) if e.contains("stopped") => JobStatus::Cancelled,
        Err(_) => JobStatus::Failed,
    };
    if let Ok(mut s) = state.session() {
        let (msg, label) = match answer {
            Ok((text, label)) => (Message::new(Author::Ai { backend: label.clone() }, MessageKind::Text, text.trim()), label),
            Err(e) => (Message::new(Author::System, MessageKind::Error, e), String::new()),
        };
        let _ = label;
        let _ = s.push_message(thread, msg);
        let chunk = s.thread(thread).ok().and_then(|t| t.chunk.clone());
        let after = if todo { s.state.chunks.last().map(|c| c.id.clone()) } else { chunk };
        let prefix = if todo { "AI to-do" } else { "Follow-up" };
        let short: String = title.chars().take(60).collect();
        match s.refresh(Some((after, format!("{prefix}: {short}"), Some(thread.clone())))) {
            Ok(report) if !report.touched_files.is_empty() => {
                let files = report.touched_files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ");
                let where_ = if todo { "added at the end of the queue" } else { "shown right after this step" };
                let body = format!("Code changed in {files} ({} new or changed hunk(s), {where_}).", report.new_hunks.len());
                let _ = s.push_message(thread, Message::new(Author::System, MessageKind::CodeChanged, body));
            }
            Ok(_) => {}
            Err(e) => {
                let _ = s.push_message(thread, Message::new(Author::System, MessageKind::Error, format!("refresh failed: {e:#}")));
            }
        }
        if let Ok(t) = s.thread_mut(thread) {
            t.busy = false;
        }
        publish(app, &s);
    }
    state.jobs.set_status(app, job, status);
}
