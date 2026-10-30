use super::{HunkPart, HunkRef};
use crate::diff::{Hunk, LineKind};
use crate::domain::HunkId;
use std::collections::HashMap;

/// Tracks which lines of each hunk already belong to some chunk, so a plan can split hunks
/// without losing or duplicating lines.
pub struct Coverage<'a> {
    order: Vec<&'a Hunk>,
    covered: HashMap<&'a HunkId, Vec<bool>>,
}

impl<'a> Coverage<'a> {
    pub fn new(hunks: impl Iterator<Item = &'a Hunk>) -> Self {
        let order: Vec<&Hunk> = hunks.collect();
        let covered = order.iter().map(|h| (&h.id, vec![false; h.lines.len()])).collect();
        Self { order, covered }
    }

    /// Claims the not yet covered lines of `part` (whole hunk if `None`) and returns them as
    /// refs, one per contiguous run that contains a changed line.
    pub fn claim(&mut self, id: &HunkId, part: Option<HunkPart>) -> Vec<HunkRef> {
        let Some(h) = self.order.iter().find(|h| &h.id == id).copied() else { return Vec::new() };
        let covered = self.covered.get_mut(&h.id).expect("same hunk set");
        let range = part.map(|p| p.range()).unwrap_or(0..h.lines.len());
        let range = range.start.min(h.lines.len())..range.end.min(h.lines.len());
        let mut runs = Vec::new();
        let mut start = None;
        for i in range.clone() {
            match (covered[i], start) {
                (false, None) => start = Some(i),
                (true, Some(s)) => {
                    runs.push(s..i);
                    start = None;
                }
                _ => {}
            }
            covered[i] = true;
        }
        if let Some(s) = start {
            runs.push(s..range.end);
        }
        runs.into_iter()
            .filter(|r| h.lines[r.clone()].iter().any(|l| l.kind != LineKind::Context))
            .filter_map(|r| HunkPart::new(r.start as u32, r.end as u32).ok())
            .map(|p| HunkRef::of_part(h, p))
            .collect()
    }

    /// Everything nobody claimed, in diff order.
    pub fn leftovers(&mut self) -> Vec<HunkRef> {
        let ids: Vec<HunkId> = self.order.iter().map(|h| h.id.clone()).collect();
        ids.iter().flat_map(|id| self.claim(id, None)).collect()
    }
}
