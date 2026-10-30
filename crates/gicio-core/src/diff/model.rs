use crate::domain::{FilePath, HunkId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Old,
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: LineKind,
    pub old_no: Option<u32>,
    pub new_no: Option<u32>,
    pub text: String,
}

impl DiffLine {
    /// Side and line number that identify a reviewable (changed) line.
    pub fn review_pos(&self) -> Option<(Side, u32)> {
        match self.kind {
            LineKind::Added => Some((Side::New, self.new_no?)),
            LineKind::Removed => Some((Side::Old, self.old_no?)),
            LineKind::Context => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub id: HunkId,
    pub path: FilePath,
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
    pub lines: Vec<DiffLine>,
}

impl Hunk {
    pub fn header(&self) -> String {
        format!("@@ -{},{} +{},{} @@", self.old_start, self.old_len, self.new_start, self.new_len)
    }

    pub fn as_unified(&self, max_lines: usize) -> String {
        let mut out = String::new();
        for line in self.lines.iter().take(max_lines) {
            let sign = match line.kind {
                LineKind::Context => ' ',
                LineKind::Added => '+',
                LineKind::Removed => '-',
            };
            out.push(sign);
            out.push_str(&line.text);
            out.push('\n');
        }
        if self.lines.len() > max_lines {
            out.push_str(&format!("... ({} more lines)\n", self.lines.len() - max_lines));
        }
        out
    }

    /// Lines prefixed with 1-based row numbers, so a planner can refer to row ranges.
    pub fn as_numbered(&self, max_lines: usize) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate().take(max_lines) {
            let sign = match line.kind {
                LineKind::Context => ' ',
                LineKind::Added => '+',
                LineKind::Removed => '-',
            };
            out.push_str(&format!("{:>4} {sign}{}\n", i + 1, line.text));
        }
        if self.lines.len() > max_lines {
            out.push_str(&format!("... ({} more lines)\n", self.lines.len() - max_lines));
        }
        out
    }

    pub fn changed_lines(&self) -> usize {
        self.lines.iter().filter(|l| l.kind != LineKind::Context).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: FilePath,
    pub old_path: Option<FilePath>,
    pub status: FileStatus,
    pub binary: bool,
    pub hunks: Vec<Hunk>,
}
