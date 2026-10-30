use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A user-provided git revision expression (branch, tag, `HEAD~3`, ...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GitRef(String);

impl GitRef {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('-') || raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
            bail!("invalid git revision: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GitRef {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<GitRef> for String {
    fn from(r: GitRef) -> String {
        r.0
    }
}

impl fmt::Display for GitRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Full hex SHA of a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CommitId(String);

impl CommitId {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.len() < 40 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("invalid commit id: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0[..10]
    }
}

impl TryFrom<String> for CommitId {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<CommitId> for String {
    fn from(c: CommitId) -> String {
        c.0
    }
}
