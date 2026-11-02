//! "Resolve later": every issue the reviewer left for later goes to the AI as one task.

use super::json_extract::extract_object_with_key;
use super::overview::change_overview;
use super::{instructions_prompt, ExtraInstructions};
use crate::domain::ThreadId;
use crate::review::{Author, Session};
use serde::Deserialize;

pub fn build_review_prompt(s: &Session, threads: &[ThreadId], extra: &ExtraInstructions) -> String {
    let mut out = String::from("A human reviewing a code change left the comments below for later. Address all of them in one go.\n");
    out.push_str(&change_overview(s));
    out.push_str("\nComments:\n");
    for id in threads {
        let Ok(t) = s.thread(id) else { continue };
        out.push_str(&format!("\n### Comment {id}\n"));
        if let Some(a) = &t.anchor {
            out.push_str(&format!("On {} lines {}-{} ({}):\n```\n{}\n```\n", a.path, a.start, a.end, if a.side == crate::diff::Side::Old { "removed lines" } else { "current code" }, a.excerpt));
        } else if let Some(c) = t.chunk.as_ref().and_then(|c| s.chunk(c)) {
            out.push_str(&format!("About review step \"{}\"\n", c.title));
        } else {
            out.push_str("About the whole change\n");
        }
        if !t.allow_edits {
            out.push_str("(Answer only: do not edit any file for this comment.)\n");
        }
        for m in &t.messages {
            let who = match m.author {
                Author::Human => "Reviewer",
                Author::Ai { .. } => "You (earlier)",
                Author::System => "[system]",
            };
            out.push_str(&format!("{who}: {}\n", m.body));
        }
    }
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nFor each comment: if it asks for a change or points at a real problem, fix it by editing files in the working \
         tree (focused edits, nothing unrelated; never commit or change git state). If it is a question, answer it. \
         Then reply with ONLY a JSON object, one short reply per comment, in the reviewer's language:\n\
         {\"replies\":[{\"thread\":\"t3\",\"reply\":\"Fixed: …\"}]}\n",
    );
    out
}

#[derive(Deserialize)]
pub struct Reply {
    pub thread: String,
    pub reply: String,
}

#[derive(Deserialize)]
struct Replies {
    replies: Vec<Reply>,
}

pub fn parse_review_replies(answer: &str) -> Option<Vec<Reply>> {
    serde_json::from_value::<Replies>(extract_object_with_key(answer, "replies")?).ok().map(|r| r.replies)
}
