//! The AI as a reviewer: it reads (part of) the change and reports findings anchored to lines.

use super::json_extract::extract_object_with_key;
use super::numbered::numbered_hunk;
use super::overview::change_overview;
use super::{instructions_prompt, ExtraInstructions};
use crate::review::{HunkRef, Session};
use serde::Deserialize;

const BUDGET: usize = 250_000;

pub fn build_ai_review_prompt(s: &Session, scope: &[HunkRef], focus: Option<&str>, extra: &ExtraInstructions) -> String {
    let mut out = String::from("You are a strict senior code reviewer. Review the code change below, which may have been written by an AI agent.\n");
    out.push_str(&change_overview(s));
    let render = |max: usize| {
        scope.iter().filter_map(|r| s.hunk(&r.id).map(|h| numbered_hunk(h, r.lines(h), max))).collect::<String>()
    };
    let diff = [400, 120, 30].iter().map(|m| render(*m)).find(|d| d.len() <= BUDGET).unwrap_or_else(|| render(10));
    out.push_str(&format!(
        "\nThe part to review (columns: old line, new line, then the diff line):\n```\n{diff}```\n\
         You may read any file in the repository for context. Do not modify anything.\n"
    ));
    if let Some(f) = focus {
        out.push_str(&format!("\nThe reviewer asks you to focus on: {f}\n"));
    }
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nReport only real problems, most important first: bugs, unhandled edge cases, swallowed or wrong error handling, \
         race conditions, security issues, dead or duplicated code, APIs that don't exist or are misused, needless \
         complexity, changes unrelated to the purpose, tests that don't test anything. No style nitpicks a formatter \
         would fix, no praise. If there is nothing worth reporting, return an empty list.\n\
         Respond with ONLY a JSON object (titles and details in the reviewer's language, details at most 3 sentences):\n\
         {\"findings\":[{\"path\":\"src/a.rs\",\"line\":12,\"endLine\":14,\"side\":\"new\",\"severity\":\"error|warning|info\",\
         \"title\":\"…\",\"detail\":\"…\"}]}\n\
         `line`/`endLine` are new-file line numbers (old-file numbers with \"side\":\"old\" for removed code).\n",
    );
    out
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawFinding {
    pub path: String,
    pub line: u32,
    pub end_line: Option<u32>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    pub title: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Deserialize)]
struct Findings {
    findings: Vec<RawFinding>,
}

pub fn parse_findings(answer: &str) -> Option<Vec<RawFinding>> {
    serde_json::from_value::<Findings>(extract_object_with_key(answer, "findings")?).ok().map(|f| f.findings)
}

/// Prompt for fixing a set of findings in the working tree.
pub fn build_fix_prompt(s: &Session, findings: &[crate::review::Finding], extra: &ExtraInstructions) -> String {
    let mut out = String::from("Fix the following problems found during code review, by editing files in the working tree.\n");
    out.push_str(&format!("Repository: {}\n\n", s.state.repo.path().display()));
    for f in findings {
        out.push_str(&format!("- {}:{}-{} [{:?}] {}: {}\n", f.path, f.line, f.end_line, f.severity, f.title, f.detail));
    }
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nKeep each fix minimal and focused; change nothing unrelated; never commit or change git state. If a finding is \
         wrong, leave the code alone and say so. Finish with a short list: one line per finding, what you did.\n",
    );
    out
}
