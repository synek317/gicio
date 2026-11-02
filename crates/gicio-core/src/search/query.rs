use anyhow::{bail, Result};
use serde::Deserialize;

/// What to look for in the project, as the search panel sends it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub pattern: SearchPattern,
    pub case_sensitive: bool,
    pub whole_word: bool,
    /// `pattern` is a regular expression rather than plain text.
    pub regex: bool,
    /// Comma-separated globs of the files to search; empty searches every file.
    #[serde(default)]
    pub include: String,
    /// Comma-separated globs of the files to leave out.
    #[serde(default)]
    pub exclude: String,
}

/// The text or regular expression searched for; never empty.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct SearchPattern(String);

impl SearchPattern {
    pub fn new(raw: &str) -> Result<Self> {
        if raw.is_empty() {
            bail!("the search text is empty");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SearchPattern {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_pattern() {
        assert!(SearchPattern::new("").is_err());
        assert_eq!(SearchPattern::new(" a").unwrap().as_str(), " a");
    }

    #[test]
    fn deserializes_from_the_ui() {
        let q: SearchQuery = serde_json::from_str(r#"{"pattern":"fn","caseSensitive":true,"wholeWord":false,"regex":false,"include":"*.rs"}"#).unwrap();
        assert_eq!(q.pattern.as_str(), "fn");
        assert_eq!(q.include, "*.rs");
        assert_eq!(q.exclude, "");
        assert!(serde_json::from_str::<SearchQuery>(r#"{"pattern":"","caseSensitive":true,"wholeWord":false,"regex":false}"#).is_err());
    }
}
