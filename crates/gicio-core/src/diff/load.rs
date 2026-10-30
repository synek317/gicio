use super::compute::compute_hunks;
use super::model::{FileDiff, FileStatus};
use super::ops::DiffOptions;
use crate::domain::{CommitId, FilePath, RepoRoot};
use crate::git::{self, ChangeKind};
use anyhow::Result;
use serde::Serialize;

const MAX_FILE_BYTES: usize = 3 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTexts {
    pub old: String,
    pub new: String,
    pub binary: bool,
}

fn decode(bytes: Vec<u8>) -> Option<String> {
    if bytes.len() > MAX_FILE_BYTES || bytes.iter().take(8000).any(|b| *b == 0) {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn file_texts(root: &RepoRoot, base: &CommitId, path: &FilePath, old_path: Option<&FilePath>, status: FileStatus) -> Result<FileTexts> {
    let old_bytes = match status {
        FileStatus::Added => Vec::new(),
        _ => git::show_file(root, base, old_path.unwrap_or(path)).unwrap_or_default(),
    };
    let new_bytes = match status {
        FileStatus::Deleted => Vec::new(),
        _ => std::fs::read(root.join(path)).unwrap_or_default(),
    };
    match (decode(old_bytes), decode(new_bytes)) {
        (Some(old), Some(new)) => Ok(FileTexts { old, new, binary: false }),
        _ => Ok(FileTexts { old: String::new(), new: String::new(), binary: true }),
    }
}

pub fn load_diff(root: &RepoRoot, base: &CommitId, opts: DiffOptions) -> Result<Vec<FileDiff>> {
    let mut out = Vec::new();
    for changed in git::changed_files(root, base)? {
        let status = match changed.kind {
            ChangeKind::Added => FileStatus::Added,
            ChangeKind::Modified => FileStatus::Modified,
            ChangeKind::Deleted => FileStatus::Deleted,
            ChangeKind::Renamed => FileStatus::Renamed,
        };
        let texts = file_texts(root, base, &changed.path, changed.old_path.as_ref(), status)?;
        let hunks = if texts.binary { Vec::new() } else { compute_hunks(&changed.path, &texts.old, &texts.new, opts) };
        if hunks.is_empty() && !texts.binary && status == FileStatus::Modified {
            continue; // e.g. mode-only change
        }
        out.push(FileDiff { path: changed.path, old_path: changed.old_path, status, binary: texts.binary, hunks });
    }
    Ok(out)
}
