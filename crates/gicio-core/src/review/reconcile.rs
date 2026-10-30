use super::{Chunk, ChunkOrigin, HunkRef};
use crate::diff::{FileDiff, Hunk};
use crate::domain::{ChunkId, HunkId, ThreadId};
use std::collections::{HashMap, HashSet};

/// Where to put hunks that appeared because of a discussion.
pub struct FollowUp {
    pub id: ChunkId,
    pub after: Option<ChunkId>,
    pub title: String,
    pub thread: Option<ThreadId>,
}

/// Brings chunk assignments in line with a freshly computed diff and returns ids of hunks
/// that did not exist before.
pub fn reconcile(chunks: &mut Vec<Chunk>, files: &[FileDiff], follow_up: Option<FollowUp>) -> Vec<HunkId> {
    let hunks: Vec<&Hunk> = files.iter().flat_map(|f| f.hunks.iter()).collect();
    let live: HashSet<&HunkId> = hunks.iter().map(|h| &h.id).collect();
    let previous: HashSet<HunkId> = chunks.iter().flat_map(|c| c.hunks.iter().map(|r| r.id.clone())).collect();
    let old_refs: Vec<(usize, HunkRef)> = chunks.iter().enumerate().flat_map(|(i, c)| c.hunks.iter().map(move |r| (i, r.clone()))).collect();

    for chunk in chunks.iter_mut() {
        chunk.hunks.retain(|r| live.contains(&r.id));
    }
    let mut assigned: HashSet<HunkId> = chunks.iter().flat_map(|c| c.hunks.iter().map(|r| r.id.clone())).collect();

    // A changed hunk usually covers the same base lines as the one it replaced: keep it in that chunk.
    let mut leftovers = Vec::new();
    for hunk in &hunks {
        if assigned.contains(&hunk.id) {
            continue;
        }
        match old_refs.iter().find(|(_, r)| r.overlaps(hunk)) {
            Some((i, _)) => chunks[*i].hunks.push(HunkRef::of(hunk)),
            None => leftovers.push(HunkRef::of(hunk)),
        }
        assigned.insert(hunk.id.clone());
    }

    if !leftovers.is_empty() {
        let (id, after, title, thread) = match follow_up {
            Some(f) => (f.id, f.after, f.title, f.thread),
            None => (ChunkId::from_raw(format!("c-new-{}", leftovers[0].id)), None, "New changes".to_string(), None),
        };
        let at = after.and_then(|a| chunks.iter().position(|c| c.id == a)).map(|p| p + 1).unwrap_or(chunks.len());
        let summary = "Changes that appeared after the review started.".to_string();
        chunks.insert(at, Chunk { id, title, summary, hunks: leftovers, origin: ChunkOrigin::FollowUp { thread }, notes: Vec::new() });
    }

    let order: HashMap<&HunkId, usize> = hunks.iter().enumerate().map(|(i, h)| (&h.id, i)).collect();
    for chunk in chunks.iter_mut() {
        chunk.hunks.sort_by_key(|r| (order.get(&r.id).copied().unwrap_or(usize::MAX), r.part.map_or(0, |p| p.range().start)));
    }
    chunks.retain(|c| !c.hunks.is_empty());

    hunks.iter().map(|h| h.id.clone()).filter(|id| !previous.contains(id)).collect()
}
