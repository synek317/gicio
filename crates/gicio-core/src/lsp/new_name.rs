use anyhow::{bail, Result};

/// The new name for a symbol rename: an identifier, nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewName(String);

impl NewName {
    pub fn new(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        let ok = raw.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') && raw.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !ok {
            bail!("not a valid name: {raw:?}");
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
