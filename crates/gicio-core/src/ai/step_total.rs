use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// The exact number of review steps the reviewer asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct StepTotal(u32);

impl StepTotal {
    pub const MAX: u32 = 999;

    pub fn new(raw: u32) -> Result<Self> {
        if raw == 0 || raw > Self::MAX {
            bail!("number of steps must be between 1 and {}: {raw}", Self::MAX);
        }
        Ok(Self(raw))
    }

    pub fn get(self) -> u32 {
        self.0
    }

    /// What is left of the total once `taken` steps exist. Always at least one, because the
    /// planner has to place the hunks it is given somewhere.
    pub fn minus(self, taken: usize) -> Self {
        let taken = u32::try_from(taken).unwrap_or(u32::MAX);
        Self(self.0.saturating_sub(taken).max(1))
    }
}

impl TryFrom<u32> for StepTotal {
    type Error = anyhow::Error;
    fn try_from(raw: u32) -> Result<Self> {
        Self::new(raw)
    }
}

impl From<StepTotal> for u32 {
    fn from(t: StepTotal) -> u32 {
        t.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_values_outside_the_range() {
        assert!(StepTotal::new(0).is_err());
        assert!(StepTotal::new(StepTotal::MAX + 1).is_err());
        assert_eq!(StepTotal::new(StepTotal::MAX).unwrap().get(), StepTotal::MAX);
    }

    #[test]
    fn deserializing_goes_through_validation() {
        assert!(serde_json::from_str::<StepTotal>("0").is_err());
        assert_eq!(serde_json::from_str::<StepTotal>("7").unwrap().get(), 7);
    }

    #[test]
    fn minus_never_goes_below_one() {
        let ten = StepTotal::new(10).unwrap();
        assert_eq!(ten.minus(3).get(), 7);
        assert_eq!(ten.minus(10).get(), 1);
        assert_eq!(ten.minus(usize::MAX).get(), 1);
    }
}
