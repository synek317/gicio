use super::{Chunk, Mark, Message, Thread};
use crate::ai::AiBackend;
use crate::diff::FileDiff;
use crate::domain::{CommitId, GitRef, RepoRoot};
use serde::Serialize;

/// What the UI gets to see.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub version: u64,
    pub repo: RepoRoot,
    pub base_ref: GitRef,
    pub base_commit: CommitId,
    pub backend: AiBackend,
    pub files: Vec<FileDiff>,
    pub chunks: Vec<Chunk>,
    pub marks: Vec<Mark>,
    pub threads: Vec<Thread>,
    pub chat: Vec<Message>,
    pub findings: Vec<super::Finding>,
    pub changes: Vec<super::AgentChange>,
    pub summary: Option<Message>,
    pub chat_busy: bool,
    pub summary_busy: bool,
    pub planning: bool,
    pub planned_by_ai: bool,
    pub step_count: crate::ai::StepCount,
    pub can_undo: bool,
    /// One-off message about how the change differs from the last session.
    pub notice: Option<String>,
    pub last_chunk: Option<crate::domain::ChunkId>,
    pub diff_options: crate::diff::DiffOptions,
    pub stage_mode: crate::stage::StageMode,
    pub instructions: Vec<crate::ai::InstructionFile>,
    pub intent: Option<super::Intent>,
    pub requirements: Vec<super::Requirement>,
    /// The change was checked against the task (an empty list then means nothing could be checked).
    pub intent_checked: bool,
    /// The code changed since that check.
    pub intent_stale: bool,
    pub checking_intent: bool,
    pub intent_error: Option<String>,
}
