use crate::diff::Side;
use crate::domain::FilePath;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FindingId(String);

impl FindingId {
    pub fn new(n: u64) -> Self {
        Self(format!("f{n}"))
    }
}

impl fmt::Display for FindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FindingSource {
    Ai,
    Clippy,
    Dotnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FindingStatus {
    Open,
    Fixing,
    Fixed,
    Dismissed,
}

/// A problem reported by the AI reviewer or by a tool, attached to lines of the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: FindingId,
    pub source: FindingSource,
    pub severity: Severity,
    pub path: FilePath,
    pub side: Side,
    pub line: u32,
    pub end_line: u32,
    pub title: String,
    pub detail: String,
    pub status: FindingStatus,
    /// The code the finding is about, to follow it when lines move.
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub outdated: bool,
}
