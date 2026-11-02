use super::line_hit::LineHit;
use crate::domain::FilePath;
use serde::Serialize;
use std::path::PathBuf;

/// The lines of one file that match.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHits {
    pub path: FilePath,
    pub abs_path: PathBuf,
    pub lines: Vec<LineHit>,
}

/// Everything a project search found.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub files: Vec<FileHits>,
    pub matches: u32,
    /// The search stopped at the limit; there are more matches.
    pub truncated: bool,
}
