use super::{PlannedChunk, PlannedItem};
use crate::diff::FileDiff;

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/") || path.starts_with("tests/") || path.contains("_test.") || path.contains("test_")
}

/// One chunk per file, sources before tests. Used before (or instead of) AI planning.
pub fn fallback_plan(files: &[FileDiff]) -> Vec<PlannedChunk> {
    let mut files: Vec<&FileDiff> = files.iter().filter(|f| !f.hunks.is_empty()).collect();
    files.sort_by_key(|f| (is_test_path(f.path.as_str()), f.path.as_str().to_string()));
    files
        .into_iter()
        .map(|f| PlannedChunk {
            title: f.path.to_string(),
            summary: format!("{:?} file, {} hunk(s).", f.status, f.hunks.len()),
            items: f.hunks.iter().map(|h| PlannedItem { hunk: h.id.clone(), part: None }).collect(),
            notes: Vec::new(),
            group: None,
        })
        .collect()
}
