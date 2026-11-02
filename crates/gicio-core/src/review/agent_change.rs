use crate::domain::{ChangeId, FilePath, ThreadId, TreeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeStatus {
    /// Waiting for the reviewer to keep or undo it.
    Pending,
    /// Kept; the lines it wrote count as reviewed.
    Accepted,
    /// Undone; its files were put back as they were before.
    Rejected,
}

/// What one AI job (or tool) changed in the working tree: snapshots of the whole tree before and after.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentChange {
    pub id: ChangeId,
    pub title: String,
    /// Who made it: the AI backend or the tool.
    pub by: String,
    /// The conversation it answered, if any.
    pub thread: Option<ThreadId>,
    pub before: TreeId,
    pub after: TreeId,
    pub files: Vec<FilePath>,
    pub status: ChangeStatus,
}
