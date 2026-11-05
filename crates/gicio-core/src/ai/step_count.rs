use super::StepWish;
use serde::{Deserialize, Serialize};

/// How many review steps the planner should aim for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", try_from = "StoredStepCount")]
pub enum StepCount {
    Few,
    #[default]
    Balanced,
    Many,
    Custom {
        wish: StepWish,
    },
}

/// What may be stored: reviews saved by older versions can hold an exact number of steps.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum StoredStepCount {
    Few,
    Balanced,
    Many,
    Custom { wish: StepWish },
    Exact { steps: u32 },
}

impl TryFrom<StoredStepCount> for StepCount {
    type Error = anyhow::Error;

    fn try_from(stored: StoredStepCount) -> anyhow::Result<Self> {
        Ok(match stored {
            StoredStepCount::Few => StepCount::Few,
            StoredStepCount::Balanced => StepCount::Balanced,
            StoredStepCount::Many => StepCount::Many,
            StoredStepCount::Custom { wish } => StepCount::Custom { wish },
            StoredStepCount::Exact { steps } => {
                let noun = if steps == 1 { "step" } else { "steps" };
                StepCount::Custom { wish: StepWish::new(&format!("{steps} {noun}"))? }
            }
        })
    }
}

impl StepCount {
    /// The planning rule that says how large a chunk is. `existing` steps stay outside this plan
    /// (kept while re-planning, or for skipped files) and count towards a number the reviewer named.
    pub fn prompt_rule(&self, existing: usize) -> String {
        match self {
            StepCount::Few => "A chunk is one broad area of the change a careful reviewer can digest in about 15-20 minutes (typically 150-600 changed lines). Aim for as few chunks as the features allow: a feature is one chunk unless it is very large; keep related files and concerns together rather than splitting them.".into(),
            StepCount::Balanced => "A chunk is one concern a careful reviewer can digest in about 5 minutes (typically 20-150 changed lines). A small feature is one chunk; a large one is split into several chunks. Split unrelated changes in one file into different chunks when it helps; group related hunks across files.".into(),
            StepCount::Many => "A chunk is one narrow concern a careful reviewer can digest in 1-2 minutes (typically 5-40 changed lines), such as a single function, type or test. Aim for many small chunks: split features, files and large hunks wherever a natural boundary allows, even when the parts are closely related.".into(),
            StepCount::Custom { wish } => {
                let others = if existing > 0 { format!(" The review already has {existing} other step(s) outside this plan: they count towards a number the reviewer named, but always plan at least one chunk.") } else { String::new() };
                format!("The reviewer asked for this number or size of chunks: \"{wish}\". Following it overrides every other rule about chunk size and feature boundaries: merge related concerns or features, or split features and large hunks at natural boundaries, keeping chunks roughly even in size. A number means the total, counted across all features: count your chunks before answering. Only a change with fewer changed rows than that may have fewer chunks.{others}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom(wish: &str) -> StepCount {
        StepCount::Custom { wish: StepWish::new(wish).unwrap() }
    }

    #[test]
    fn serializes_as_the_ui_expects() {
        assert_eq!(serde_json::to_string(&StepCount::Few).unwrap(), r#"{"kind":"few"}"#);
        assert_eq!(serde_json::to_string(&custom("10 steps")).unwrap(), r#"{"kind":"custom","wish":"10 steps"}"#);
        assert_eq!(serde_json::from_str::<StepCount>(r#"{"kind":"custom","wish":"10 steps"}"#).unwrap(), custom("10 steps"));
        assert_eq!(serde_json::from_str::<StepCount>(r#"{"kind":"balanced"}"#).unwrap(), StepCount::Balanced);
    }

    #[test]
    fn an_exact_number_from_an_older_review_becomes_a_wish() {
        assert_eq!(serde_json::from_str::<StepCount>(r#"{"kind":"exact","steps":12}"#).unwrap(), custom("12 steps"));
        assert_eq!(serde_json::from_str::<StepCount>(r#"{"kind":"exact","steps":1}"#).unwrap(), custom("1 step"));
    }

    #[test]
    fn an_empty_wish_does_not_deserialize() {
        assert!(serde_json::from_str::<StepCount>(r#"{"kind":"custom","wish":" "}"#).is_err());
    }

    #[test]
    fn custom_rule_quotes_the_wish_and_counts_existing_steps() {
        let rule = custom("as few as possible").prompt_rule(0);
        assert!(rule.contains("\"as few as possible\"") && !rule.contains("already has"));
        assert!(custom("10 steps").prompt_rule(3).contains("already has 3 other step(s)"));
        assert_eq!(StepCount::Many.prompt_rule(3), StepCount::Many.prompt_rule(0));
    }
}
