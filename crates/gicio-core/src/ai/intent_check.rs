//! The AI compares the change with the task: what is met, what is missing, what nobody asked for.

use super::json_extract::extract_object_with_key;
use super::numbered::numbered_diff;
use super::overview::change_overview;
use super::{instructions_prompt, ExtraInstructions, RawFinding};
use crate::review::{FindingSource, FindingStatus, HunkRef, Session};
use crate::skip::SkipFilter;
use serde::Deserialize;

const BUDGET: usize = 250_000;

pub fn build_intent_check_prompt(s: &Session, skip: &SkipFilter, extra: &ExtraInstructions) -> String {
    let scope: Vec<HunkRef> = s.files.iter().filter(|f| !skip.skips(&f.path)).flat_map(|f| f.hunks.iter().map(HunkRef::of)).collect();
    let mut out = String::from(
        "You check whether a code change does what it was supposed to do. It may have been written by an AI agent, so \
         do not take comments, commit messages or test names for proof: verify everything in the code.\n",
    );
    out.push_str(&change_overview(s));
    out.push_str(&format!(
        "\nThe change (columns: old line, new line, then the diff line; long hunks are shortened):\n```\n{}```\n\
         You may read any file in the repository. Do not modify anything.\n",
        numbered_diff(s, &scope, BUDGET)
    ));
    out.push_str(&earlier_results(s));
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(&format!(
        "\nDo this:\n\
         1. Split the description into separate, checkable requirements, in the order it states them. Take only what it \
         actually asks for: invent nothing and leave out background and motivation. If it is too vague to check, return \
         few requirements or none.\n\
         2. Give each one a verdict. \"done\": real code fully does it; a stub, a TODO or a test alone does not count. \
         \"partial\": some of it is done; the note says what is not. \"missing\": nothing does it. \"unclear\": the code \
         cannot tell (for example a performance goal); the note says what would settle it. Before saying \"missing\", \
         search the repository: it may be done outside the changed lines. Point at the code that does it in \"evidence\".\n\
         3. List in \"outOfScope\" the changes the description neither asks for nor needs: extra features, unrelated \
         refactorings, drive-by edits. What a requirement needs (supporting types, its tests, its docs) is in scope. With \
         a vague description report only what is clearly unrelated. \"detail\" says what was changed and why it is unrelated.\n\n\
         Respond with ONLY a JSON object, with notes, titles and details in {language}, each at most 2 sentences:\n\
         {{\"requirements\":[{{\"text\":\"…\",\"verdict\":\"done|partial|missing|unclear\",\"note\":\"…\",\"evidence\":[{{\"path\":\"src/a.rs\",\"line\":12}}]}}],\
         \"outOfScope\":[{{\"path\":\"src/a.rs\",\"line\":12,\"endLine\":14,\"side\":\"new\",\"severity\":\"warning\",\"title\":\"…\",\"detail\":\"…\"}}]}}\n\
         `line`/`endLine` are new-file line numbers (old-file numbers with \"side\":\"old\" for removed code).\n",
        language = s.state.backend.language().as_str(),
    ));
    out
}

/// What the last check found and the reviewer decided, so a new check does not start from nothing.
fn earlier_results(s: &Session) -> String {
    let mut out = String::new();
    if !s.requirements().is_empty() {
        out.push_str("\nRequirements from the previous check. Copy the text of those that still apply word for word:\n");
        for r in s.requirements() {
            out.push_str(&format!("- {}\n", r.text));
        }
    }
    let accepted: Vec<String> = s.state.findings.iter().filter(|f| f.source == FindingSource::Intent && f.status == FindingStatus::Dismissed).map(|f| format!("- {}:{} {}\n", f.path, f.line, f.title)).collect();
    if !accepted.is_empty() {
        out.push_str(&format!("\nThe reviewer already accepted these changes beyond the task; do not report them again:\n{}", accepted.concat()));
    }
    out
}

#[derive(Deserialize)]
pub struct RawEvidence {
    pub path: String,
    pub line: u32,
}

#[derive(Deserialize)]
pub struct RawRequirement {
    pub text: String,
    #[serde(default)]
    pub verdict: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub evidence: Vec<RawEvidence>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawIntentCheck {
    pub requirements: Vec<RawRequirement>,
    #[serde(default)]
    pub out_of_scope: Vec<RawFinding>,
}

pub fn parse_intent_check(answer: &str) -> Option<RawIntentCheck> {
    serde_json::from_value(extract_object_with_key(answer, "requirements")?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_result_in_free_form_output() {
        let raw = parse_intent_check("Here you go:\n```json\n{\"requirements\":[{\"text\":\"A\",\"verdict\":\"done\"}],\"outOfScope\":[{\"path\":\"a.rs\",\"line\":1,\"title\":\"T\"}]}\n```").unwrap();
        assert_eq!(raw.requirements[0].text, "A");
        assert_eq!(raw.out_of_scope[0].title, "T");
    }

    #[test]
    fn optional_parts_may_be_left_out() {
        let raw = parse_intent_check("{\"requirements\":[{\"text\":\"A\"}]}").unwrap();
        assert!(raw.out_of_scope.is_empty());
        assert!(raw.requirements[0].evidence.is_empty());
    }

    #[test]
    fn anything_else_is_not_a_result() {
        assert!(parse_intent_check("I could not do it").is_none());
        assert!(parse_intent_check("{\"requirements\":\"none\"}").is_none());
    }
}
