use super::StepTotal;
use serde::{Deserialize, Serialize};

/// How many review steps the planner should aim for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StepCount {
    Few,
    #[default]
    Balanced,
    Many,
    Exact {
        steps: StepTotal,
    },
}

impl StepCount {
    /// The target for a plan that comes on top of `taken` existing steps, so that an exact number
    /// keeps describing the whole review. The other settings describe step size and do not change.
    pub fn remaining_after(self, taken: usize) -> Self {
        match self {
            StepCount::Exact { steps } => StepCount::Exact { steps: steps.minus(taken) },
            other => other,
        }
    }

    /// The planning rule that says how large a chunk is.
    pub fn prompt_rule(self) -> String {
        match self {
            StepCount::Few => "A chunk is one broad area of the change a careful reviewer can digest in about 15-20 minutes (typically 150-600 changed lines). Aim for as few chunks as the features allow: a feature is one chunk unless it is very large; keep related files and concerns together rather than splitting them.".into(),
            StepCount::Balanced => "A chunk is one concern a careful reviewer can digest in about 5 minutes (typically 20-150 changed lines). A small feature is one chunk; a large one is split into several chunks. Split unrelated changes in one file into different chunks when it helps; group related hunks across files.".into(),
            StepCount::Many => "A chunk is one narrow concern a careful reviewer can digest in 1-2 minutes (typically 5-40 changed lines), such as a single function, type or test. Aim for many small chunks: split features, files and large hunks wherever a natural boundary allows, even when the parts are closely related.".into(),
            StepCount::Exact { steps } => {
                let n = steps.get();
                let noun = if n == 1 { "chunk" } else { "chunks" };
                format!("The JSON must contain exactly {n} {noun} in total, counted across all features. Reaching this number overrides every other rule about chunk size and feature boundaries: merge related concerns or features, or split features and large hunks at natural boundaries, keeping chunks roughly even in size. Count your chunks before answering. Only a change with fewer changed rows than that may have fewer chunks.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(n: u32) -> StepCount {
        StepCount::Exact { steps: StepTotal::new(n).unwrap() }
    }

    #[test]
    fn serializes_as_the_ui_expects() {
        assert_eq!(serde_json::to_string(&StepCount::Few).unwrap(), r#"{"kind":"few"}"#);
        assert_eq!(serde_json::to_string(&exact(12)).unwrap(), r#"{"kind":"exact","steps":12}"#);
        assert_eq!(serde_json::from_str::<StepCount>(r#"{"kind":"exact","steps":12}"#).unwrap(), exact(12));
    }

    #[test]
    fn invalid_exact_number_does_not_deserialize() {
        assert!(serde_json::from_str::<StepCount>(r#"{"kind":"exact","steps":0}"#).is_err());
    }

    #[test]
    fn only_an_exact_number_shrinks_for_existing_steps() {
        assert_eq!(exact(10).remaining_after(4), exact(6));
        assert_eq!(exact(3).remaining_after(5), exact(1));
        assert_eq!(StepCount::Many.remaining_after(5), StepCount::Many);
    }

    #[test]
    fn exact_rule_names_the_number() {
        assert!(exact(7).prompt_rule().contains("exactly 7 chunks"));
        assert!(exact(1).prompt_rule().contains("exactly 1 chunk in total"));
    }
}
