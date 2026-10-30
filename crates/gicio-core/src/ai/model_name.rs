use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Model identifier passed to a CLI's `--model` flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModelName(String);

impl ModelName {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('-') || raw.chars().any(char::is_whitespace) {
            bail!("invalid model name: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ModelName {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<ModelName> for String {
    fn from(m: ModelName) -> String {
        m.0
    }
}
