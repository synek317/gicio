use crate::diff::{Hunk, LineKind};

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
