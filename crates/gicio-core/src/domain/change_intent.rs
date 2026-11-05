use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// What a change was supposed to do (a task, a prompt, an issue, commit messages): trimmed, never
/// empty and small enough to fit in every prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ChangeIntent(String);

impl ChangeIntent {
    pub const MAX_CHARS: usize = 100_000;

    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            bail!("the description of the task is empty");
        }
        if raw.chars().count() > Self::MAX_CHARS {
            bail!("the description of the task is too long (at most {} characters)", Self::MAX_CHARS);
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ChangeIntent {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<ChangeIntent> for String {
    fn from(i: ChangeIntent) -> String {
        i.0
    }
}

impl fmt::Display for ChangeIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_rejects_empty_or_too_long() {
        assert_eq!(ChangeIntent::new("  Add login\n").unwrap().as_str(), "Add login");
        assert!(ChangeIntent::new(" \n\t").is_err());
        assert!(ChangeIntent::new(&"x".repeat(ChangeIntent::MAX_CHARS)).is_ok());
        assert!(ChangeIntent::new(&"x".repeat(ChangeIntent::MAX_CHARS + 1)).is_err());
    }

    #[test]
    fn invalid_values_cannot_be_deserialized() {
        assert!(serde_json::from_str::<ChangeIntent>("\"  \"").is_err());
        assert_eq!(serde_json::from_str::<ChangeIntent>("\" x \"").unwrap().as_str(), "x");
    }
}
