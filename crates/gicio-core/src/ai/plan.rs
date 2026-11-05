use super::json_extract::extract_object_with_key;
use super::plan_prompt::{build_plan_prompt, PlanPromptInput};
use super::{run_streaming, AccessMode, AiBackend, ExtraInstructions, LogSink, StepCount};
use tokio::sync::oneshot;
use crate::diff::Hunk;
use crate::domain::{CommitId, GitRef, RepoRoot};
use super::plan_labels::parse_label;
use crate::diff::{LineKind, Side};
use crate::review::{HunkPart, Intent, Note, PlannedChunk, PlannedItem};
use anyhow::{bail, Context, Result};
use serde::Deserialize;

pub struct PlanRequest {
    pub backend: AiBackend,
    pub repo: RepoRoot,
    pub base_ref: GitRef,
    pub base_commit: CommitId,
    pub hunks: Vec<Hunk>,
    pub kept_titles: Vec<String>,
    pub extra: ExtraInstructions,
    pub steps: StepCount,
    /// Steps that stay outside this plan (kept while re-planning, or for skipped files).
    pub existing_steps: usize,
    pub intent: Option<Intent>,
}

#[derive(Deserialize)]
struct RawPlan {
    features: Vec<RawFeature>,
}

#[derive(Deserialize)]
struct RawFeature {
    title: String,
    chunks: Vec<RawChunk>,
}

#[derive(Deserialize)]
struct RawChunk {
    title: String,
    #[serde(default)]
    summary: String,
    hunks: Vec<String>,
    #[serde(default)]
    notes: Vec<RawNote>,
}

#[derive(Deserialize)]
struct RawNote {
    at: String,
    text: String,
}

pub async fn plan(req: PlanRequest, log: LogSink, cancel: Option<oneshot::Receiver<()>>) -> Result<Vec<PlannedChunk>> {
    if req.hunks.is_empty() {
        return Ok(Vec::new());
    }
    let prompt = build_plan_prompt(&PlanPromptInput {
        repo: &req.repo,
        base_ref: &req.base_ref,
        base_commit: &req.base_commit,
        language: req.backend.language(),
        engine: &req.backend.engine,
        extra: &req.extra,
        hunks: &req.hunks,
        kept_titles: &req.kept_titles,
        steps: &req.steps,
        existing_steps: req.existing_steps,
        intent: req.intent.as_ref(),
    });
    let answer = run_streaming(&req.backend, &req.repo, &prompt, AccessMode::ReadOnly, log, cancel).await?;
    let value = extract_object_with_key(&answer, "features").with_context(|| format!("planner returned no JSON plan: {}", answer.chars().take(400).collect::<String>()))?;
    let raw: RawPlan = serde_json::from_value(value)?;
    let item = |l: &str| {
        let label = parse_label(l)?;
        let h = req.hunks.get(label.hunk)?;
        let part = label.rows.and_then(|(a, b)| HunkPart::new(a as u32, b.min(h.lines.len()) as u32).ok());
        Some(PlannedItem { hunk: h.id.clone(), part })
    };
    let note = |n: &RawNote| {
        let label = parse_label(&n.at)?;
        let h = req.hunks.get(label.hunk)?;
        let line = h.lines.get(label.rows.map_or(0, |r| r.0))?;
        let (side, no) = match line.kind {
            LineKind::Removed => (Side::Old, line.old_no?),
            _ => (Side::New, line.new_no?),
        };
        Some(Note { path: h.path.clone(), side, line: no, text: n.text.clone(), code: line.text.clone(), outdated: false })
    };
    let chunks: Vec<PlannedChunk> = raw
        .features
        .iter()
        .flat_map(|f| f.chunks.iter().map(move |c| (f, c)))
        .map(|(f, c)| PlannedChunk {
            title: c.title.clone(),
            summary: c.summary.clone(),
            items: c.hunks.iter().filter_map(|l| item(l)).collect(),
            notes: c.notes.iter().filter_map(note).collect(),
            group: Some(f.title.clone()),
        })
        .filter(|c| !c.items.is_empty())
        .collect();
    if chunks.is_empty() {
        bail!("planner returned an empty plan");
    }
    Ok(chunks)
}
