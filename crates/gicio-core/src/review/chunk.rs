use crate::diff::{DiffLine, Hunk, Side};
use crate::domain::{ChunkId, FilePath, HunkId, ThreadId};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// A contiguous slice `from..to` of a hunk's lines, for chunks that cover only part of a hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawPart")]
pub struct HunkPart {
    from: u32,
    to: u32,
}

#[derive(Deserialize)]
struct RawPart {
    from: u32,
    to: u32,
}

impl TryFrom<RawPart> for HunkPart {
    type Error = anyhow::Error;
    fn try_from(r: RawPart) -> Result<Self> {
        Self::new(r.from, r.to)
    }
}

impl HunkPart {
    pub fn new(from: u32, to: u32) -> Result<Self> {
        if from >= to {
            bail!("empty hunk part {from}..{to}");
        }
        Ok(Self { from, to })
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.from as usize..self.to as usize
    }
}

/// Reference to a hunk (or part of it) plus its position on the (immutable) base side, which
/// lets us recognise the "same place" after the working tree changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HunkRef {
    pub id: HunkId,
    pub path: FilePath,
    pub old_start: u32,
    pub old_len: u32,
    #[serde(default)]
    pub part: Option<HunkPart>,
}

impl HunkRef {
    pub fn of(h: &Hunk) -> Self {
        Self { id: h.id.clone(), path: h.path.clone(), old_start: h.old_start, old_len: h.old_len, part: None }
    }

    pub fn of_part(h: &Hunk, part: HunkPart) -> Self {
        let whole = part.from == 0 && part.to as usize >= h.lines.len();
        Self { part: (!whole).then_some(part), ..Self::of(h) }
    }

    pub fn overlaps(&self, h: &Hunk) -> bool {
        let (a0, a1) = (self.old_start, self.old_start + self.old_len.max(1));
        let (b0, b1) = (h.old_start, h.old_start + h.old_len.max(1));
        self.path == h.path && a0 <= b1 && b0 <= a1
    }

    /// The lines of `h` this reference covers.
    pub fn lines<'h>(&self, h: &'h Hunk) -> &'h [DiffLine] {
        match self.part {
            Some(p) => &h.lines[p.range().start.min(h.lines.len())..p.range().end.min(h.lines.len())],
            None => &h.lines,
        }
    }
}

/// A short AI explanation attached to one line of the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub path: FilePath,
    pub side: Side,
    pub line: u32,
    pub text: String,
    /// The line's code when the note was made, to follow it.
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub outdated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChunkOrigin {
    Fallback,
    Ai,
    FollowUp { thread: Option<ThreadId> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub id: ChunkId,
    pub title: String,
    pub summary: String,
    pub hunks: Vec<HunkRef>,
    pub origin: ChunkOrigin,
    #[serde(default)]
    pub notes: Vec<Note>,
    /// The feature this step belongs to; consecutive steps of one feature are shown nested.
    #[serde(default)]
    pub group: Option<String>,
}

/// One entry of a planned chunk: a whole hunk or a line range of it.
#[derive(Debug, Clone)]
pub struct PlannedItem {
    pub hunk: HunkId,
    pub part: Option<HunkPart>,
}

/// A chunk as proposed by a planner, before it gets an id.
#[derive(Debug, Clone)]
pub struct PlannedChunk {
    pub title: String,
    pub summary: String,
    pub items: Vec<PlannedItem>,
    pub notes: Vec<Note>,
    pub group: Option<String>,
}
