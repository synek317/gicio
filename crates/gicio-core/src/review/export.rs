use super::{Author, FindingStatus, MarkStatus, Session, ThreadStatus};
use crate::diff::Side;

/// The review as markdown: progress, summary, discussions and open findings.
pub fn export_markdown(s: &Session) -> String {
    let st = &s.state;
    let (mut total, mut accepted, mut later) = (0, 0, 0);
    for f in &s.files {
        for (side, line) in f.hunks.iter().flat_map(|h| h.lines.iter()).filter_map(|l| l.review_pos()) {
            total += 1;
            match st.marks.status(&super::LineKey { path: f.path.clone(), side, line }) {
                Some(MarkStatus::Accepted) => accepted += 1,
                Some(MarkStatus::Deferred) => later += 1,
                None => {}
            }
        }
    }
    let mut out = format!(
        "# Review of `{}` against `{}` ({})\n\n{accepted}/{total} changed lines accepted, {later} put off for later.\n",
        st.repo.path().file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        st.base_ref,
        st.base_commit.short()
    );
    if let Some(sum) = &st.summary {
        out.push_str(&format!("\n## Summary\n\n{}\n", sum.body));
    }
    let threads: Vec<_> = st.threads.iter().filter(|t| !t.messages.is_empty()).collect();
    if !threads.is_empty() {
        out.push_str("\n## Discussions\n");
        for t in threads {
            let place = match &t.anchor {
                Some(a) => format!("`{}` {}{}{}", a.path, if a.side == Side::Old { "−" } else { "+" }, a.start, if a.end > a.start { format!("–{}", a.end) } else { String::new() }),
                None => t.chunk.as_ref().and_then(|c| s.chunk(c)).map(|c| format!("step “{}”", c.title)).unwrap_or_else(|| "general".into()),
            };
            let status = match t.status {
                ThreadStatus::Open => "open",
                ThreadStatus::Resolved => "resolved",
                ThreadStatus::Deferred => "later",
            };
            out.push_str(&format!("\n### {place} ({status})\n"));
            if let Some(a) = &t.anchor
                && !a.excerpt.is_empty()
            {
                out.push_str(&format!("\n```\n{}\n```\n", a.excerpt));
            }
            for m in &t.messages {
                let who = match &m.author {
                    Author::Human => "**Reviewer**".to_string(),
                    Author::Ai { backend } => format!("**AI ({backend})**"),
                    Author::System => "_gicio_".to_string(),
                };
                out.push_str(&format!("\n{who}: {}\n", m.body));
            }
        }
    }
    let open: Vec<_> = st.findings.iter().filter(|f| f.status == FindingStatus::Open).collect();
    if !open.is_empty() {
        out.push_str("\n## Open findings\n\n");
        for f in open {
            out.push_str(&format!("- `{}:{}` **{}** — {}\n", f.path, f.line, f.title, f.detail.replace('\n', " ")));
        }
    }
    out
}
