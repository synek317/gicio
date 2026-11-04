use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A commit message: trimmed and never empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CommitMessage(String);

impl CommitMessage {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            bail!("empty commit message");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CommitMessage {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<CommitMessage> for String {
    fn from(m: CommitMessage) -> String {
        m.0
    }
}

impl fmt::Display for CommitMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_rejects_empty() {
        assert_eq!(CommitMessage::new("  Fix it\n").unwrap().as_str(), "Fix it");
        assert!(CommitMessage::new(" \n\t").is_err());
    }
}
