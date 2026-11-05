use crate::domain::ChangeIntent;
use serde::{Deserialize, Serialize};

/// Where the description of the task came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntentSource {
    /// Written or pasted by the reviewer.
    Reviewer,
    /// Taken from the commit messages of the change; follows them until the reviewer writes their own.
    Commits,
}

/// What the change under review was supposed to do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Intent {
    pub text: ChangeIntent,
    pub source: IntentSource,
}

impl Intent {
    pub fn from_reviewer(text: ChangeIntent) -> Self {
        Self { text, source: IntentSource::Reviewer }
    }

    /// The commit messages as a description: as many as fit, oldest first. `None` without any.
    pub fn from_commits(messages: &[String]) -> Option<Self> {
        const SEPARATOR: &str = "\n\n";
        let mut left = ChangeIntent::MAX_CHARS;
        let fitting: Vec<&str> = messages
            .iter()
            .map(String::as_str)
            .take_while(|m| {
                let Some(rest) = left.checked_sub(m.chars().count() + SEPARATOR.len()) else { return false };
                left = rest;
                true
            })
            .collect();
        let text = ChangeIntent::new(&fitting.join(SEPARATOR)).ok()?;
        Some(Self { text, source: IntentSource::Commits })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_messages_become_one_description_in_order() {
        let i = Intent::from_commits(&["Add login".into(), "Fix typo".into()]).unwrap();
        assert_eq!(i.text.as_str(), "Add login\n\nFix typo");
        assert_eq!(i.source, IntentSource::Commits);
    }

    #[test]
    fn no_commits_mean_no_description() {
        assert!(Intent::from_commits(&[]).is_none());
    }

    #[test]
    fn messages_that_do_not_fit_are_left_out() {
        let big = "x".repeat(ChangeIntent::MAX_CHARS - 10);
        let i = Intent::from_commits(&["first".into(), big.clone(), "last".into()]).unwrap();
        assert_eq!(i.text.as_str(), format!("first\n\n{big}").as_str());
        assert!(Intent::from_commits(&["x".repeat(ChangeIntent::MAX_CHARS + 1)]).is_none());
    }
}
