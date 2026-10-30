use super::overview::{change_diff, change_overview};
use super::{instructions_prompt, ExtraInstructions};
use crate::review::{Author, MessageKind, Session};

const STYLE: &str = "Write in the reviewer's language. Be brief and strictly on topic: no preamble, no restating the question, \
                     no generic advice. Use short markdown.";

/// Free conversation about the whole change, outside any particular review step.
pub fn build_chat_prompt(s: &Session, extra: &ExtraInstructions, allow_edits: bool) -> String {
    let mut out = String::from("You are assisting a human who is reviewing a large code change (possibly written by an AI agent).\n");
    out.push_str(&change_overview(s));
    out.push_str("\nYou can read any file in the repository for details.\n");
    if let Some(sum) = &s.state.summary {
        out.push_str(&format!("\nEarlier summary of the change:\n{}\n", sum.body));
    }
    out.push_str("\nConversation (oldest first):\n");
    for m in &s.state.chat {
        let who = match (&m.author, m.kind) {
            (_, MessageKind::CodeChanged) | (Author::System, _) => "[system]",
            (Author::Human, _) => "Reviewer",
            (Author::Ai { .. }, _) => "You",
        };
        out.push_str(&format!("{who}: {}\n\n", m.body));
    }
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(if allow_edits {
        "\nReply to the reviewer's latest message. If they ask for a change, edit files in the working tree (focused, \
         nothing unrelated; never commit or change git state) and end with one line saying what you changed.\n"
    } else {
        "\nReply to the reviewer's latest message. Do NOT modify any files.\n"
    });
    out.push_str(STYLE);
    out
}

/// One-shot summary of the whole change for the reviewer.
pub fn build_summary_prompt(s: &Session, extra: &ExtraInstructions, skip: &crate::skip::SkipFilter) -> String {
    let mut out = String::from("A human is about to review (or is reviewing) the code change below. Summarize it for them.\n");
    out.push_str(&change_overview(s));
    out.push_str(&format!("\nThe diff:\n```diff\n{}```\n", change_diff(s, skip)));
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nWrite the summary as markdown with these sections, each a few bullets at most:\n\
         **What it does** (the purpose, in plain words), **How** (the main design decisions and where they live), \
         **Risks** (what could break, surprising behaviour, missing tests), **Look closely at** (specific files/functions).\n\
         Do not modify any files. ",
    );
    out.push_str(STYLE);
    out
}
