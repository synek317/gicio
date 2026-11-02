use crate::diff::Side;
use crate::domain::{ChangeId, ChunkId, FilePath, ThreadId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThreadStatus {
    Open,
    Resolved,
    /// Left for later: the AI takes all of these in one go in the "resolve later" step.
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub path: FilePath,
    pub side: Side,
    pub start: u32,
    pub end: u32,
    /// The anchored lines' text when the thread was created.
    pub excerpt: String,
    /// That code is gone (changed or removed).
    #[serde(default)]
    pub outdated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Author {
    Human,
    Ai { backend: String },
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageKind {
    Text,
    Error,
    CodeChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub author: Author,
    pub kind: MessageKind,
    pub body: String,
    pub at: u64,
    /// For a "code changed" note: the AI change it reports, to look at, keep or undo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<ChangeId>,
}

impl Message {
    pub fn new(author: Author, kind: MessageKind, body: impl Into<String>) -> Self {
        let at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        Self { author, kind, body: body.into(), at, change: None }
    }

    pub fn with_change(self, change: Option<ChangeId>) -> Self {
        Self { change, ..self }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: ThreadId,
    pub chunk: Option<ChunkId>,
    pub anchor: Option<Anchor>,
    pub status: ThreadStatus,
    pub allow_edits: bool,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub busy: bool,
}
