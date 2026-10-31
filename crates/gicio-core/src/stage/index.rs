use super::partial_content;
use crate::diff::{file_texts, FileDiff, FileStatus};
use crate::domain::{CommitId, FilePath, RepoRoot};
use crate::git;
use crate::review::{LineKey, Marks};
use anyhow::Result;

/// Rewrites the index entries of `files` so the index holds exactly the accepted changes.
pub fn sync_index(root: &RepoRoot, base: &CommitId, files: &[&FileDiff], marks: &Marks, opts: crate::diff::DiffOptions) -> Result<()> {
    for f in files {
        let texts = file_texts(root, base, &f.path, f.old_path.as_ref(), f.status)?;
        if texts.binary {
            continue;
        }
        let accepted = |side, line| marks.status(&LineKey { path: f.path.clone(), side, line }) == Some(crate::review::MarkStatus::Accepted);
        let content = partial_content(&texts.old, &texts.new, opts, accepted);
        let nothing = content == texts.old;
        match (f.status, nothing) {
            (FileStatus::Renamed, true) => {
                git::reset_path(root, &f.path)?;
                if let Some(old) = &f.old_path {
                    git::reset_path(root, old)?;
                }
            }
            (_, true) => git::reset_path(root, &f.path)?,
            (FileStatus::Deleted, false) if content.is_empty() => git::remove_from_index(root, &f.path)?,
            (status, false) => {
                let mode = file_mode(root, &f.path);
                git::stage_content(root, &f.path, mode, content.as_bytes())?;
                if status == FileStatus::Renamed
                    && let Some(old) = &f.old_path
                {
                    git::remove_from_index(root, old)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn file_mode(root: &RepoRoot, path: &FilePath) -> &'static str {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(root.join(path)) {
        Ok(m) if m.permissions().mode() & 0o111 != 0 => "100755",
        _ => "100644",
    }
}

/// No executable bit on disk: keep what git already records.
#[cfg(not(unix))]
fn file_mode(root: &RepoRoot, path: &FilePath) -> &'static str {
    if git::executable_in_index(root, path) { "100755" } else { "100644" }
}
