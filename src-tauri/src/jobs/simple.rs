use super::{JobKind, JobStatus, Jobs};
use crate::publish::err;
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode, AiBackend};
use gicio_core::domain::RepoRoot;
use tauri::{AppHandle, Manager};

/// One AI call tracked in the activity panel (live log, Stop). Returns the final message.
pub async fn run_ai_job(app: &AppHandle, kind: JobKind, title: &str, backend: &AiBackend, repo: &RepoRoot, prompt: &str, mode: AccessMode) -> Result<String, String> {
    let jobs = &app.state::<AppState>().jobs;
    let job = jobs.add(app, kind, title.to_string(), None, JobStatus::Running);
    let cancel = jobs.start(app, &job);
    let res = ai::run_streaming(backend, repo, prompt, mode, Jobs::sink(app, &job), Some(cancel)).await.map_err(err);
    jobs.set_status(app, &job, status_of(&res));
    res
}

/// A non-AI job (cargo fmt, clippy…) shown in the activity panel.
pub async fn run_tool_job<T>(app: &AppHandle, title: &str, fut: impl std::future::Future<Output = anyhow::Result<T>>) -> Result<T, String> {
    let jobs = &app.state::<AppState>().jobs;
    let job = jobs.add(app, JobKind::Tool, title.to_string(), None, JobStatus::Running);
    let res = fut.await.map_err(err);
    if let Err(e) = &res {
        jobs.log(app, &job, e.clone());
    }
    jobs.set_status(app, &job, status_of(&res));
    res
}

pub fn status_of<T>(res: &Result<T, String>) -> JobStatus {
    match res {
        Ok(_) => JobStatus::Done,
        Err(e) if e.contains("stopped") => JobStatus::Cancelled,
        Err(_) => JobStatus::Failed,
    }
}
