//! A compact description of the whole change for chat and summary prompts.

use crate::diff::{FileStatus, LineKind};
use crate::review::{LineKey, Session, ThreadStatus};
use crate::skip::SkipFilter;

const DIFF_BUDGET: usize = 250_000;

pub fn change_overview(s: &Session) -> String {
    let mut out = format!(
        "Repository: {}\nThe change = working tree vs base commit {} ({}).\n\nChanged files:\n",
        s.state.repo.path().display(),
        s.state.base_commit.as_str(),
        s.state.base_ref
    );
    for f in &s.files {
        let (add, del) = f.hunks.iter().flat_map(|h| h.lines.iter()).fold((0, 0), |(a, d), l| match l.kind {
            LineKind::Added => (a + 1, d),
            LineKind::Removed => (a, d + 1),
            LineKind::Context => (a, d),
        });
        let status = match f.status {
            FileStatus::Added => "added",
            FileStatus::Modified => "modified",
            FileStatus::Deleted => "deleted",
            FileStatus::Renamed => "renamed",
        };
        out.push_str(&format!("- {} ({status}, +{add} -{del})\n", f.path));
    }
    out.push_str("\nReview steps and the reviewer's progress (accepted / changed lines):\n");
    for (i, c) in s.state.chunks.iter().enumerate() {
        let (mut total, mut accepted) = (0, 0);
        for r in &c.hunks {
            let Some(h) = s.hunk(&r.id) else { continue };
            for (side, line) in r.lines(h).iter().filter_map(|l| l.review_pos()) {
                total += 1;
                if s.state.marks.status(&LineKey { path: h.path.clone(), side, line }).is_some() {
                    accepted += 1;
                }
            }
        }
        out.push_str(&format!("{}. {} ({accepted}/{total})\n", i + 1, c.title));
    }
    let open: Vec<String> = s
        .state
        .threads
        .iter()
        .filter(|t| t.status == ThreadStatus::Open)
        .filter_map(|t| t.messages.first().map(|m| format!("- {}", m.body.chars().take(160).collect::<String>())))
        .collect();
    if !open.is_empty() {
        out.push_str(&format!("\nOpen review discussions:\n{}\n", open.join("\n")));
    }
    out
}

/// The diff itself, shortened per hunk until it fits the budget. Skipped files are only named.
pub fn change_diff(s: &Session, skip: &SkipFilter) -> String {
    let skipped: Vec<String> = s.files.iter().filter(|f| skip.skips(&f.path)).map(|f| format!("--- {} (lock/generated file, content omitted)\n", f.path)).collect();
    let render = |max: usize| {
        s.files.iter().filter(|f| !skip.skips(&f.path)).flat_map(|f| f.hunks.iter()).map(|h| format!("--- {} {}\n{}", h.path, h.header(), h.as_unified(max))).chain(skipped.iter().cloned()).collect::<String>()
    };
    [400, 120, 30, 5].iter().map(|m| render(*m)).find(|d| d.len() <= DIFF_BUDGET).unwrap_or_else(|| render(0))
}
