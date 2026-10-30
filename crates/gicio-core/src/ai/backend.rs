use super::{Effort, Language, ModelName};
use anyhow::{bail, Result};
use serde::{Deserialize, Deserializer, Serialize};

/// A shell command line for the custom backend; it gets the prompt on stdin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ShellCommand(String);

impl ShellCommand {
    pub fn new(raw: &str) -> Result<Self> {
        if raw.trim().is_empty() {
            bail!("custom command must not be empty");
        }
        Ok(Self(raw.trim().to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ShellCommand {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self> {
        Self::new(&s)
    }
}

impl From<ShellCommand> for String {
    fn from(c: ShellCommand) -> String {
        c.0
    }
}

/// Which CLI runs the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Engine {
    Claude,
    Codex,
    Copilot,
    Custom { command: ShellCommand },
}

impl Engine {
    pub fn name(&self) -> &'static str {
        match self {
            Engine::Claude => "claude",
            Engine::Codex => "codex",
            Engine::Copilot => "copilot",
            Engine::Custom { .. } => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiBackend {
    pub engine: Engine,
    pub model: Option<ModelName>,
    pub effort: Option<Effort>,
    #[serde(default)]
    pub language: Language,
}

impl Default for AiBackend {
    fn default() -> Self {
        Self { engine: Engine::Claude, model: None, effort: None, language: Language::default() }
    }
}

impl AiBackend {
    pub fn label(&self) -> String {
        let extra: Vec<&str> = [self.model.as_ref().map(|m| m.as_str()), self.effort.map(|e| e.claude())].into_iter().flatten().collect();
        if extra.is_empty() {
            self.engine.name().to_string()
        } else {
            format!("{} ({})", self.engine.name(), extra.join(", "))
        }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }
}

/// Accepts state files written by older versions: an unreadable backend becomes the default.
pub fn lenient_backend<'de, D: Deserializer<'de>>(d: D) -> Result<AiBackend, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    Ok(serde_json::from_value(v).unwrap_or_default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    ReadOnly,
    Edit,
}
