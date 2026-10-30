//! Persists review state under `.git/gicio/`, keyed by the base commit.

use crate::domain::{CommitId, RepoRoot};
use crate::git;
use crate::review::ReviewState;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn path_for(root: &RepoRoot, base: &CommitId) -> Result<PathBuf> {
    Ok(git::git_dir(root)?.join("gicio").join(format!("review-{}.json", base.short())))
}

pub fn load(path: &Path) -> Result<Option<ReviewState>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&std::fs::read(path)?)?))
}

pub fn save(path: &Path, state: &ReviewState) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}
