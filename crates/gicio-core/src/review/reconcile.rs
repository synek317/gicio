use super::{Chunk, ChunkOrigin, HunkRef};
use crate::diff::{FileDiff, Hunk};
use crate::domain::{ChunkId, HunkId, ThreadId};
use std::collections::{HashMap, HashSet};

/// Where to put hunks that appeared because of a discussion.
pub struct FollowUp {
    pub id: ChunkId,
    /// The step the new changes follow; `None` puts them at the end of the queue.
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
        let after = after.and_then(|a| chunks.iter().position(|c| c.id == a));
        let at = after.map(|p| p + 1).unwrap_or(chunks.len());
        // A follow-up of a step stays inside that step's feature.
        let group = after.and_then(|p| chunks[p].group.clone());
        let summary = "Changes that appeared after the review started.".to_string();
        chunks.insert(at, Chunk { id, title, summary, hunks: leftovers, origin: ChunkOrigin::FollowUp { thread }, notes: Vec::new(), group });
    }

    let order: HashMap<&HunkId, usize> = hunks.iter().enumerate().map(|(i, h)| (&h.id, i)).collect();
    for chunk in chunks.iter_mut() {
        chunk.hunks.sort_by_key(|r| (order.get(&r.id).copied().unwrap_or(usize::MAX), r.part.map_or(0, |p| p.range().start)));
    }
    chunks.retain(|c| !c.hunks.is_empty());

    hunks.iter().map(|h| h.id.clone()).filter(|id| !previous.contains(id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::{DiffLine, FileStatus, LineKind};
    use crate::domain::FilePath;

    fn hunk(id: &str, old_start: u32) -> Hunk {
        let line = DiffLine { kind: LineKind::Added, old_no: None, new_no: Some(old_start), text: id.into() };
        Hunk { id: HunkId::from_content("a.rs", id, 0), path: FilePath::new("a.rs").unwrap(), old_start, old_len: 0, new_start: old_start, new_len: 1, lines: vec![line] }
    }

    fn chunk(id: &str, h: &Hunk, group: Option<&str>) -> Chunk {
        Chunk { id: ChunkId::from_raw(id), title: id.into(), summary: String::new(), hunks: vec![HunkRef::of(h)], origin: ChunkOrigin::Ai, notes: Vec::new(), group: group.map(Into::into) }
    }

    fn files(hunks: Vec<Hunk>) -> Vec<FileDiff> {
        vec![FileDiff { path: FilePath::new("a.rs").unwrap(), old_path: None, status: FileStatus::Modified, binary: false, hunks }]
    }

    fn follow_up(after: Option<&str>) -> FollowUp {
        FollowUp { id: ChunkId::from_raw("new"), after: after.map(ChunkId::from_raw), title: "Follow-up".into(), thread: None }
    }

    #[test]
    fn follow_up_of_a_step_stays_in_its_feature() {
        let (h1, h2, h3) = (hunk("h1", 10), hunk("h2", 50), hunk("h3", 90));
        let mut chunks = vec![chunk("c1", &h1, Some("Feature")), chunk("c2", &h2, Some("Feature"))];
        reconcile(&mut chunks, &files(vec![h1, h2, h3]), Some(follow_up(Some("c1"))));
        let ids: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["c1", "new", "c2"]);
        assert_eq!(chunks[1].group.as_deref(), Some("Feature"));
    }

    #[test]
    fn follow_up_at_the_end_belongs_to_no_feature() {
        let (h1, h2) = (hunk("h1", 10), hunk("h2", 90));
        let mut chunks = vec![chunk("c1", &h1, Some("Feature"))];
        reconcile(&mut chunks, &files(vec![h1, h2]), Some(follow_up(None)));
        assert_eq!(chunks.last().map(|c| (c.id.as_str(), c.group.clone())), Some(("new", None)));
    }
}
