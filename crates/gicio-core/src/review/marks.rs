use crate::diff::{FileDiff, Side};
use crate::domain::FilePath;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkStatus {
    Accepted,
    Deferred,
}

/// Identifies one changed line: removed lines live on the old side, added on the new one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineKey {
    pub path: FilePath,
    pub side: Side,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    #[serde(flatten)]
    pub key: LineKey,
    /// Line content at the time of marking, used to follow the line when the diff shifts.
    pub text: String,
    pub status: MarkStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(from = "Vec<Mark>", into = "Vec<Mark>")]
pub struct Marks(BTreeMap<LineKey, Mark>);

impl From<Vec<Mark>> for Marks {
    fn from(v: Vec<Mark>) -> Self {
        Self(v.into_iter().map(|m| (m.key.clone(), m)).collect())
    }
}

impl From<Marks> for Vec<Mark> {
    fn from(m: Marks) -> Self {
        m.0.into_values().collect()
    }
}

impl Marks {
    /// Sets (or clears, with `None`) a mark and returns the previous one.
    pub fn set(&mut self, key: LineKey, text: String, status: Option<MarkStatus>) -> Option<Mark> {
        match status {
            Some(status) => self.0.insert(key.clone(), Mark { key, text, status }),
            None => self.0.remove(&key),
        }
    }

    pub fn restore(&mut self, key: LineKey, previous: Option<Mark>) {
        match previous {
            Some(m) => self.0.insert(key, m),
            None => self.0.remove(&key),
        };
    }

    pub fn status(&self, key: &LineKey) -> Option<MarkStatus> {
        self.0.get(key).map(|m| m.status)
    }

    /// Re-attaches marks to the new diff: a mark survives if a changed line with the same
    /// path, side and text still exists; the closest such line wins.
    pub fn remap(&self, files: &[FileDiff]) -> Marks {
        let mut candidates: HashMap<(&FilePath, Side, &str), Vec<u32>> = HashMap::new();
        for file in files {
            for hunk in &file.hunks {
                for line in &hunk.lines {
                    if let Some((side, no)) = line.review_pos() {
                        candidates.entry((&file.path, side, line.text.as_str())).or_default().push(no);
                    }
                }
            }
        }
        let mut used: HashSet<LineKey> = HashSet::new();
        let mut out = BTreeMap::new();
        for mark in self.0.values() {
            let Some(lines) = candidates.get(&(&mark.key.path, mark.key.side, mark.text.as_str())) else { continue };
            let best = lines
                .iter()
                .map(|no| LineKey { path: mark.key.path.clone(), side: mark.key.side, line: *no })
                .filter(|k| !used.contains(k))
                .min_by_key(|k| k.line.abs_diff(mark.key.line));
            if let Some(key) = best {
                used.insert(key.clone());
                out.insert(key.clone(), Mark { key, text: mark.text.clone(), status: mark.status });
            }
        }
        Marks(out)
    }
}
