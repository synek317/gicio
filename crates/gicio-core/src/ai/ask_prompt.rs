use super::{instructions_prompt, ExtraInstructions};
use crate::domain::ThreadId;
use crate::review::{Author, MessageKind, Session};
use anyhow::Result;

const MAX_CHUNK_DIFF_LINES: usize = 400;

/// Prompt for one reviewer message in a thread: chunk diff, anchor, history, question.
pub fn build_ask_prompt(session: &Session, thread_id: &ThreadId, extra: &ExtraInstructions) -> Result<String> {
    let thread = session.thread(thread_id)?;
    let st = &session.state;
    let mut out = format!(
        "You are assisting a human who is reviewing a code change (possibly written by an AI agent).\n\
         Repository: {}\nThe change under review = the working tree compared to base commit {} ({}).\n\n",
        st.repo.path().display(),
        st.base_commit.as_str(),
        st.base_ref
    );

    if let Some(chunk) = thread.chunk.as_ref().and_then(|c| session.chunk(c)) {
        out.push_str(&format!("The reviewer is looking at review chunk \"{}\": {}\n\nDiff of this chunk:\n", chunk.title, chunk.summary));
        let mut budget = MAX_CHUNK_DIFF_LINES;
        for r in &chunk.hunks {
            let Some(h) = session.hunk(&r.id) else { continue };
            out.push_str(&format!("--- {} {}\n```diff\n{}```\n", h.path, h.header(), h.as_unified(budget)));
            budget = budget.saturating_sub(h.lines.len());
            if budget == 0 {
                out.push_str("(diff truncated; read the files if you need more)\n");
                break;
            }
        }
        out.push('\n');
    } else if thread.anchor.is_none() {
        // A review-wide request: give the whole picture; the AI reads files as needed.
        out.push_str("The reviewer's request concerns the whole change, not one place.\n");
        out.push_str(&super::overview::change_overview(session));
        out.push_str("\nRead whatever files you need.\n\n");
    }

    if let Some(a) = &thread.anchor {
        let side = match a.side {
            crate::diff::Side::Old => "removed lines, numbered in the base version",
            crate::diff::Side::New => "lines in the current working tree",
        };
        out.push_str(&format!("The comment is attached to {} lines {}-{} ({side}):\n```\n{}\n```\n\n", a.path, a.start, a.end, a.excerpt));
    }

    out.push_str("Conversation (oldest first):\n");
    for m in &thread.messages {
        let who = match (&m.author, m.kind) {
            (_, MessageKind::CodeChanged) => "[system]",
            (Author::Human, _) => "Reviewer",
            (Author::Ai { .. }, _) => "You",
            (Author::System, _) => "[system]",
        };
        out.push_str(&format!("{who}: {}\n\n", m.body));
    }

    out.push_str(&instructions_prompt(&st.repo, &st.backend.engine, extra));
    out.push_str(if thread.allow_edits {
        "\nReply to the reviewer's latest message. If they ask for a change, or you find a real problem while answering, \
         fix it by editing files in the working tree: minimal, focused edits, nothing unrelated. Never commit, stage, \
         stash, checkout or otherwise change git state.\n"
    } else {
        "\nReply to the reviewer's latest message. You CANNOT modify files in this conversation (editing is turned off). If the reviewer asks for a change, do not say you will do it: say in one sentence that editing is off here and they can tick \"AI may edit code\"; you may describe the change briefly.\n"
    });
    out.push_str(
        "Your final message is shown verbatim to the reviewer, in the reviewer's language. Be brief and strictly on topic: \
         answer exactly what was asked, no preamble, no restating the question, no recap of the code, no generic advice. \
         A few sentences or a short list; include code only when it is the answer. If you changed code, end with one line \
         saying what you changed.\n",
    );
    Ok(out)
}
