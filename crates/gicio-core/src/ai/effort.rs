use serde::{Deserialize, Serialize};

/// How hard the model should think. Mapped per CLI; ignored where unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl Effort {
    pub fn claude(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Xhigh => "xhigh",
            Effort::Max => "max",
        }
    }

    /// Copilot CLI accepts the same names as Claude.
    pub fn copilot(self) -> &'static str {
        self.claude()
    }

    pub fn codex(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High | Effort::Xhigh | Effort::Max => "high",
        }
    }
}
