use crate::publish::{err, publish};
use crate::jobs::{JobKind, JobStatus, Jobs};
use crate::state::AppState;
use gicio_core::ai::{self, PlanRequest, StepCount};
use gicio_core::domain::HunkId;
use gicio_core::review::{Chunk, PlannedChunk, PlannedItem, Snapshot};
use std::collections::HashSet;
use tauri::{AppHandle, State};

fn begin(app: &AppHandle, state: &AppState, replan: bool, steps: StepCount) -> Result<(PlanRequest, Vec<Chunk>, Vec<HunkId>), String> {
    let mut s = state.session()?;
    if s.planning {
        return Err("planning is already running".into());
    }
    let (keep, wanted): (Vec<Chunk>, Option<HashSet<_>>) = if replan {
        let (keep, pending) = s.replan_split();
        (keep, Some(pending.into_iter().collect()))
    } else {
        (Vec::new(), None)
    };
    let skip = state.settings.skip_filter();
    let (skipped, hunks): (Vec<_>, Vec<_>) = s.files.iter().flat_map(|f| f.hunks.iter()).filter(|h| wanted.as_ref().is_none_or(|w| w.contains(&h.id))).cloned().partition(|h| skip.skips(&h.path));
    let skipped: Vec<HunkId> = skipped.into_iter().map(|h| h.id).collect();
    // Kept steps and the step for skipped files count towards an exact number of steps.
    let taken = keep.len() + usize::from(!skipped.is_empty());
    let req = PlanRequest {
        backend: s.state.backend.clone(),
        repo: s.state.repo.clone(),
        base_ref: s.state.base_ref.clone(),
        base_commit: s.state.base_commit.clone(),
        hunks,
        kept_titles: keep.iter().map(|c| c.title.clone()).collect(),
        extra: state.settings.instructions(),
        steps: steps.remaining_after(taken),
    };
    s.state.step_count = steps;
    s.planning = true;
    s.touch();
    publish(app, &s);
    Ok((req, keep, skipped))
}

async fn plan_and_apply(app: AppHandle, state: State<'_, AppState>, replan: bool, steps: StepCount) -> Result<Snapshot, String> {
    let (req, keep, skipped) = begin(&app, &state, replan, steps)?;
    let title = if replan { "Re-plan remaining steps" } else { "Plan review steps" };
    let job = state.jobs.add(&app, JobKind::Plan, title.into(), None, JobStatus::Running);
    let cancel = state.jobs.start(&app, &job);
    let result = ai::plan(req, Jobs::sink(&app, &job), Some(cancel)).await;
    state.jobs.set_status(&app, &job, match &result {
        Ok(_) => JobStatus::Done,
        Err(e) if e.to_string().contains("stopped") => JobStatus::Cancelled,
        Err(_) => JobStatus::Failed,
    });
    let mut s = state.session()?;
    s.planning = false;
    s.touch();
    match result {
        Ok(mut planned) => {
            if !skipped.is_empty() {
                planned.push(PlannedChunk {
                    title: "Lock files and generated code".into(),
                    summary: "Not sent to the AI (see Settings → AI → skip). Usually not worth a line-by-line review; skim and accept.".into(),
                    items: skipped.iter().map(|id| PlannedItem { hunk: id.clone(), part: None }).collect(),
                    notes: Vec::new(),
                    group: None,
                });
            }
            // Chunks kept during a replan may have changed meanwhile; take their current version.
            let keep = keep.into_iter().filter_map(|k| s.chunk(&k.id).cloned()).collect();
            s.apply_plan(keep, planned, true);
            Ok(publish(&app, &s))
        }
        Err(e) => {
            publish(&app, &s);
            Err(err(e))
        }
    }
}

#[tauri::command]
pub async fn plan_ai(app: AppHandle, state: State<'_, AppState>, steps: StepCount) -> Result<Snapshot, String> {
    plan_and_apply(app, state, false, steps).await
}

/// Re-plans only chunks the reviewer has not touched yet.
#[tauri::command]
pub async fn replan_ai(app: AppHandle, state: State<'_, AppState>, steps: StepCount) -> Result<Snapshot, String> {
    plan_and_apply(app, state, true, steps).await
}
