//! Keeps the review in sync with the disk: when files change outside gicio (an editor, another
//! agent, git), the diff is re-read and the state saved.

use crate::publish::publish;
use crate::state::AppState;
use gicio_core::domain::{CommitId, RepoRoot};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone, Serialize)]
struct Changed {
    files: Vec<String>,
}

/// Starts watching for the review just opened; an older watcher stops by itself.
pub fn watch_worktree(app: AppHandle, repo: RepoRoot, base: CommitId) {
    let generation = app.state::<AppState>().watch_generation.fetch_add(1, Ordering::SeqCst) + 1;
    tauri::async_runtime::spawn(async move {
        let mut last: Option<u64> = None;
        loop {
            tokio::time::sleep(INTERVAL).await;
            let state = app.state::<AppState>();
            if state.watch_generation.load(Ordering::SeqCst) != generation {
                return;
            }
            // AI edits and tools refresh the diff themselves when they finish, and put new code
            // in the right place in the queue; don't race them.
            if state.jobs.edits_running() {
                last = None;
                continue;
            }
            let (r, b) = (repo.clone(), base.clone());
            let Ok(Ok(fp)) = tauri::async_runtime::spawn_blocking(move || gicio_core::git::worktree_fingerprint(&r, &b)).await else { continue };
            if last.is_none() || last == Some(fp) {
                last = Some(fp);
                continue;
            }
            last = Some(fp);
            let Ok(mut s) = state.session() else { return };
            if s.state.repo != repo {
                return;
            }
            if let Ok(report) = s.refresh(None)
                && !report.touched_files.is_empty()
            {
                publish(&app, &s);
                let _ = app.emit("worktree-changed", Changed { files: report.touched_files.iter().map(|f| f.to_string()).collect() });
            }
        }
    });
}
