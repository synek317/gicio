use super::{Chunk, Marks, Message, Thread};
use crate::ai::AiBackend;
use crate::domain::{ChunkId, CommitId, GitRef, RepoRoot, ThreadId};
use serde::{Deserialize, Serialize};

/// Everything about a review that is persisted between runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewState {
    pub repo: RepoRoot,
    pub base_ref: GitRef,
    pub base_commit: CommitId,
    #[serde(deserialize_with = "crate::ai::lenient_backend")]
    pub backend: AiBackend,
    pub chunks: Vec<Chunk>,
    pub marks: Marks,
    pub threads: Vec<Thread>,
    pub planned_by_ai: bool,
    /// How many steps the AI was last asked to plan.
    #[serde(default)]
    pub step_count: crate::ai::StepCount,
    #[serde(default)]
    pub diff_options: crate::diff::DiffOptions,
    #[serde(default)]
    pub stage_mode: crate::stage::StageMode,
    /// Free conversation about the whole change (the Chat tab).
    #[serde(default)]
    pub chat: Vec<Message>,
    #[serde(default)]
    pub summary: Option<Message>,
    #[serde(default)]
    pub findings: Vec<super::Finding>,
    /// What AI jobs changed, to keep or undo.
    #[serde(default)]
    pub changes: Vec<super::AgentChange>,
    /// Fingerprint of the change at the last save, to recognise it next time.
    #[serde(default)]
    pub line_signature: Vec<u64>,
    /// The step the reviewer was on.
    #[serde(default)]
    pub last_chunk: Option<ChunkId>,
    /// What the change was supposed to do.
    #[serde(default)]
    pub intent: Option<super::Intent>,
    /// The last check of the change against `intent`.
    #[serde(default)]
    pub intent_check: Option<super::IntentCheck>,
    next_id: u64,
}

impl ReviewState {
    pub fn new(repo: RepoRoot, base_ref: GitRef, base_commit: CommitId, backend: AiBackend) -> Self {
        Self { repo, base_ref, base_commit, backend, chunks: Vec::new(), marks: Marks::default(), threads: Vec::new(), planned_by_ai: false, step_count: Default::default(), diff_options: Default::default(), stage_mode: Default::default(), chat: Vec::new(), summary: None, findings: Vec::new(), changes: Vec::new(), line_signature: Vec::new(), last_chunk: None, intent: None, intent_check: None, next_id: 1 }
    }

    fn next(&mut self) -> u64 {
        let n = self.next_id;
        self.next_id += 1;
        n
    }

    pub fn next_chunk_id(&mut self) -> ChunkId {
        ChunkId::new(self.next())
    }

    pub fn next_thread_id(&mut self) -> ThreadId {
        ThreadId::new(self.next())
    }

    pub fn next_change_id(&mut self) -> crate::domain::ChangeId {
        crate::domain::ChangeId::new(self.next())
    }

    pub fn next_finding_id(&mut self) -> super::FindingId {
        super::FindingId::new(self.next())
    }

    pub fn next_requirement_id(&mut self) -> super::RequirementId {
        super::RequirementId::new(self.next())
    }
}
