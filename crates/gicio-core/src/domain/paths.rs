use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Absolute path to the top level of a git working tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepoRoot(PathBuf);

impl RepoRoot {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.is_absolute() {
            bail!("repository path must be absolute: {}", path.display());
        }
        if !path.join(".git").exists() {
            bail!("not the top level of a git repository: {}", path.display());
        }
        Ok(Self(path))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn join(&self, file: &FilePath) -> PathBuf {
        self.0.join(file.as_str())
    }

    /// Converts an absolute path inside the repo into a repo-relative one.
    pub fn relative(&self, abs: &Path) -> Option<FilePath> {
        let rel = abs.strip_prefix(&self.0).ok()?;
        FilePath::new(rel.to_str()?).ok()
    }
}

/// Repo-relative path of a file, always with forward slashes and no `..`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FilePath(String);

impl FilePath {
    pub fn new(raw: &str) -> Result<Self> {
        if raw.is_empty() || raw.starts_with('/') || raw.split('/').any(|p| p == "..") {
            bail!("invalid repo-relative path: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.0).extension().and_then(|e| e.to_str())
    }
}

impl TryFrom<String> for FilePath {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<FilePath> for String {
    fn from(p: FilePath) -> String {
        p.0
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
