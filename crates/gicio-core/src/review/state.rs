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
    /// Free conversation about the whole change (the Chat tab).
    #[serde(default)]
    pub chat: Vec<Message>,
    #[serde(default)]
    pub summary: Option<Message>,
    next_id: u64,
}

impl ReviewState {
    pub fn new(repo: RepoRoot, base_ref: GitRef, base_commit: CommitId, backend: AiBackend) -> Self {
        Self { repo, base_ref, base_commit, backend, chunks: Vec::new(), marks: Marks::default(), threads: Vec::new(), planned_by_ai: false, chat: Vec::new(), summary: None, next_id: 1 }
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
}
