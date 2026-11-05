use crate::diff::{Hunk, LineKind};
use crate::review::{HunkRef, Session};

/// A hunk with old and new line numbers in front of every line, so the AI can point at lines.
pub fn numbered_hunk(h: &Hunk, lines: &[crate::diff::DiffLine], max: usize) -> String {
    let mut out = format!("--- {} {}\n", h.path, h.header());
    for l in lines.iter().take(max) {
        let n = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_default();
        let sign = match l.kind {
            LineKind::Context => ' ',
            LineKind::Added => '+',
            LineKind::Removed => '-',
        };
        out.push_str(&format!("{:>5} {:>5} {sign}{}\n", n(l.old_no), n(l.new_no), l.text));
    }
    if lines.len() > max {
        out.push_str(&format!("... ({} more lines)\n", lines.len() - max));
    }
    out
}

/// The hunks of `scope` with line numbers, each shortened as far as needed to fit `budget` characters.
pub fn numbered_diff(s: &Session, scope: &[HunkRef], budget: usize) -> String {
    let render = |max: usize| scope.iter().filter_map(|r| s.hunk(&r.id).map(|h| numbered_hunk(h, r.lines(h), max))).collect::<String>();
    [400, 120, 30].iter().map(|m| render(*m)).find(|d| d.len() <= budget).unwrap_or_else(|| render(10))
}
