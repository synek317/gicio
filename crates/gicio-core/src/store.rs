//! Persists review state under `.git/gicio/`, one file per review scope: uncommitted changes,
//! or everything since a given ref. The base commit may move (commits, rebases) and the review
//! carries over.

use crate::domain::{CommitId, GitRef, RepoRoot};
use crate::git;
use crate::review::ReviewState;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// File-name-safe identity of what is reviewed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreKey(String);

impl StoreKey {
    pub fn for_base(base: &GitRef) -> Self {
        if base.as_str() == "HEAD" {
            return Self("uncommitted".into());
        }
        let safe: String = base.as_str().chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '_' }).collect();
        Self(format!("since-{safe}"))
    }
}

pub fn path_for(root: &RepoRoot, key: &StoreKey) -> Result<PathBuf> {
    Ok(git::git_dir(root)?.join("gicio").join(format!("review-{}.json", key.0)))
}

/// Where versions before per-scope keys kept the review of `base`.
pub fn legacy_path(root: &RepoRoot, base: &CommitId) -> Result<PathBuf> {
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

/// Moves a review aside (kept, not deleted) and returns where it went.
pub fn archive(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let to = path.with_extension(format!("{stamp}.json"));
    std::fs::rename(path, &to)?;
    Ok(Some(to))
}
