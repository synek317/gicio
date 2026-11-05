use crate::domain::FilePath;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequirementId(String);

impl RequirementId {
    pub fn new(n: u64) -> Self {
        Self(format!("q{n}"))
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How far the change meets a requirement of the task, according to the AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Done,
    Partial,
    Missing,
    /// Cannot be told from the code.
    Unclear,
}

impl Verdict {
    /// An answer the AI was not asked for proves nothing, so it counts as unclear.
    pub fn from_answer(answer: &str) -> Self {
        match answer.trim().to_ascii_lowercase().as_str() {
            "done" => Self::Done,
            "partial" => Self::Partial,
            "missing" => Self::Missing,
            _ => Self::Unclear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequirementStatus {
    Open,
    /// An AI job is working on it.
    Implementing,
    /// An AI job finished working on it; only a new check can say whether it is met now.
    Implemented,
    /// The reviewer decided it does not matter.
    Dismissed,
}

/// A place in the code the AI points at. Not necessarily part of the change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeRef {
    pub path: FilePath,
    pub line: u32,
}

/// One thing the task asks for and how far the change meets it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirement {
    pub id: RequirementId,
    pub text: String,
    pub verdict: Verdict,
    pub note: String,
    pub evidence: Vec<CodeRef>,
    pub status: RequirementStatus,
}

/// The result of checking the change against the task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentCheck {
    pub requirements: Vec<Requirement>,
    /// Fingerprint of the change that was checked, to tell when the result is out of date.
    pub signature: Vec<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unexpected_answers_are_unclear() {
        assert_eq!(Verdict::from_answer(" DONE "), Verdict::Done);
        assert_eq!(Verdict::from_answer("missing"), Verdict::Missing);
        assert_eq!(Verdict::from_answer("probably fine"), Verdict::Unclear);
    }
}
