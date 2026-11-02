use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

const MAX_LOG: usize = 400;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct JobId(String);

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobKind {
    Ask,
    Plan,
    Chat,
    Summary,
    Review,
    AiReview,
    Fix,
    Commit,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub title: String,
    pub thread: Option<String>,
    pub log: Vec<String>,
}

#[derive(Default)]
struct Inner {
    list: Vec<JobInfo>,
    cancels: HashMap<JobId, oneshot::Sender<()>>,
    next: u64,
}

/// Every AI call the app makes, with its live activity log; the UI shows them and can stop them.
#[derive(Default)]
pub struct Jobs {
    inner: Mutex<Inner>,
    /// Background AI edits (resolving later issues, fixes) share the working tree, so they run one at a time.
    pub edit_lock: tokio::sync::Mutex<()>,
}

#[derive(Clone, Serialize)]
struct LogEvent<'a> {
    id: &'a JobId,
    line: &'a str,
}

impl Jobs {
    pub fn add(&self, app: &AppHandle, kind: JobKind, title: String, thread: Option<String>, status: JobStatus) -> JobId {
        let id = {
            let mut i = self.inner.lock().unwrap();
            i.next += 1;
            let id = JobId(format!("j{}", i.next));
            i.list.push(JobInfo { id: id.clone(), kind, status, title, thread, log: Vec::new() });
            id
        };
        self.emit(app);
        id
    }

    pub fn set_status(&self, app: &AppHandle, id: &JobId, status: JobStatus) {
        {
            let mut i = self.inner.lock().unwrap();
            if let Some(j) = i.list.iter_mut().find(|j| &j.id == id) {
                j.status = status;
            }
            if !matches!(status, JobStatus::Queued | JobStatus::Running) {
                i.cancels.remove(id);
            }
        }
        self.emit(app);
    }

    /// Marks the job running and returns the receiver that fires when the user stops it.
    pub fn start(&self, app: &AppHandle, id: &JobId) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.inner.lock().unwrap().cancels.insert(id.clone(), tx);
        self.set_status(app, id, JobStatus::Running);
        rx
    }

    pub fn log(&self, app: &AppHandle, id: &JobId, line: String) {
        {
            let mut i = self.inner.lock().unwrap();
            let Some(j) = i.list.iter_mut().find(|j| &j.id == id) else { return };
            j.log.push(line.clone());
            if j.log.len() > MAX_LOG {
                j.log.drain(..j.log.len() - MAX_LOG);
            }
        }
        let _ = app.emit("job-log", LogEvent { id, line: &line });
    }

    pub fn cancel(&self, app: &AppHandle, id: &JobId) {
        let queued = {
            let mut i = self.inner.lock().unwrap();
            if let Some(tx) = i.cancels.remove(id) {
                let _ = tx.send(());
            }
            i.list.iter().any(|j| &j.id == id && j.status == JobStatus::Queued)
        };
        if queued {
            self.set_status(app, id, JobStatus::Cancelled);
        }
    }

    /// Whether a job that may edit files (or refreshes the diff itself) is queued or running.
    pub fn edits_running(&self) -> bool {
        self.inner.lock().unwrap().list.iter().any(|j| {
            matches!(j.status, JobStatus::Queued | JobStatus::Running) && !matches!(j.kind, JobKind::Plan | JobKind::Summary | JobKind::AiReview | JobKind::Commit)
        })
    }

    pub fn list(&self) -> Vec<JobInfo> {
        self.inner.lock().unwrap().list.clone()
    }

    pub fn emit(&self, app: &AppHandle) {
        let _ = app.emit("jobs", self.list());
    }

    /// Sink for `run_streaming` that appends to this job's log.
    pub fn sink(app: &AppHandle, id: &JobId) -> gicio_core::ai::LogSink {
        let (app, id) = (app.clone(), id.clone());
        Box::new(move |line| {
            use tauri::Manager;
            app.state::<crate::state::AppState>().jobs.log(&app, &id, line)
        })
    }
}
