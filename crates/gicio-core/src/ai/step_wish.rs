use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The number or size of review steps in the reviewer's own words ("10 steps", "as few as
/// possible"): one line, never empty and short enough to quote in a prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct StepWish(String);

impl StepWish {
    pub const MAX_CHARS: usize = 200;

    pub fn new(raw: &str) -> Result<Self> {
        let wish = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if wish.is_empty() {
            bail!("say how many steps you want, e.g. \"10 steps\"");
        }
        if wish.chars().count() > Self::MAX_CHARS {
            bail!("the wish for the number of steps is too long (at most {} characters)", Self::MAX_CHARS);
        }
        Ok(Self(wish))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StepWish {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for StepWish {
    type Error = anyhow::Error;

    fn try_from(raw: String) -> Result<Self> {
        Self::new(&raw)
    }
}

impl From<StepWish> for String {
    fn from(w: StepWish) -> String {
        w.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_one_tidy_line() {
        assert_eq!(StepWish::new("  as few\n as   possible ").unwrap().as_str(), "as few as possible");
    }

    #[test]
    fn rejects_empty_and_too_long() {
        assert!(StepWish::new(" \n ").is_err());
        assert!(StepWish::new(&"x".repeat(StepWish::MAX_CHARS + 1)).is_err());
        assert!(StepWish::new(&"x".repeat(StepWish::MAX_CHARS)).is_ok());
        assert!(serde_json::from_str::<StepWish>(r#""  ""#).is_err());
    }
}
