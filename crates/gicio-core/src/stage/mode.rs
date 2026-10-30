use serde::{Deserialize, Serialize};

/// Where accepted changes live.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StageMode {
    /// Only in gicio's review state.
    #[default]
    Memory,
    /// Also in the git index: accepted lines are staged, everything else is not.
    Index,
}
