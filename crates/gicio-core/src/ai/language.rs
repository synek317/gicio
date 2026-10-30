use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Natural language the AI should write chunk titles and summaries in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Language(String);

impl Language {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() || raw.len() > 40 {
            bail!("invalid language: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Language {
    fn default() -> Self {
        Self("English".into())
    }
}

impl TryFrom<String> for Language {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<Language> for String {
    fn from(l: Language) -> String {
        l.0
    }
}
