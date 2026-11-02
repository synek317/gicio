use super::{instructions_prompt, Engine, ExtraInstructions, Language};
use crate::diff::Hunk;
use crate::domain::{CommitId, GitRef, RepoRoot};

const BUDGET_CHARS: usize = 350_000;

pub struct PlanPromptInput<'a> {
    pub repo: &'a RepoRoot,
    pub base_ref: &'a GitRef,
    pub base_commit: &'a CommitId,
    pub language: &'a Language,
    pub engine: &'a Engine,
    pub extra: &'a ExtraInstructions,
    pub hunks: &'a [Hunk],
    pub kept_titles: &'a [String],
}

fn render_hunks(hunks: &[Hunk], max_lines: usize) -> String {
    let mut out = String::new();
    for (i, h) in hunks.iter().enumerate() {
        out.push_str(&format!("### H{} {} {}\n```\n{}```\n\n", i + 1, h.path, h.header(), h.as_numbered(max_lines)));
    }
    out
}

/// Hunks are labelled H1..Hn in the prompt; the caller maps labels back to ids.
pub fn build_plan_prompt(input: &PlanPromptInput) -> String {
    let body = [200, 60, 15, 3]
        .iter()
        .map(|max| render_hunks(input.hunks, *max))
        .find(|b| b.len() <= BUDGET_CHARS)
        .unwrap_or_else(|| render_hunks(input.hunks, 0));
    let kept = if input.kept_titles.is_empty() {
        String::new()
    } else {
        format!(
            "\nThese chunks are already in the reviewer's queue (for context only, do not repeat them):\n{}\n",
            input.kept_titles.iter().map(|t| format!("- {t}\n")).collect::<String>()
        )
    };
    format!(
        r#"You are helping a human review a large code change, quite possibly written by an AI agent.
Repository: {repo}
The change under review = the working tree compared to base commit {sha} ({base_ref}).

Split the hunks listed below into features, each feature into review chunks, and order them.

Rules:
- A feature is one piece of functionality (a new capability, a bug fix, a refactoring, a dependency bump…). If the change contains several features, give each its own entry so the reviewer goes through one feature at a time; never mix two features in one chunk. A change that is really one feature is a single entry.
- A chunk is one concern a careful reviewer can digest in about 5 minutes (typically 20-150 changed lines). A small feature is one chunk; a large one is split into several chunks. Split unrelated changes in one file into different chunks when it helps; group related hunks across files.
- Order features so that ones others build on come first. Within a feature, order chunks so understanding builds up: data types and interfaces first, then core logic, then call sites and wiring, then tests, docs, config. Generated or mechanical changes (renames, formatting) go last, as their own feature.
- Every hunk must be covered. A large hunk that mixes separate concerns (e.g. a new file with several unrelated items) may be split across chunks with row ranges like "H3:1-40" and "H3:41-90" (rows are numbered at the left of each hunk). Split only at natural boundaries such as between functions or items, never inside one.
- Feature and chunk "title": at most ~8 words. "summary": 1-4 sentences saying what the chunk does and what the reviewer should check (risks, invariants, edge cases).
- Write titles and summaries in {language}.
- Optionally add "notes": short explanations for spots where a reviewer would likely wonder "why?" (a non-obvious invariant, a subtle fix, a surprising API choice). Attach each to a row: {{"at":"H3:12","text":"..."}}. Most chunks need none; never restate what the code plainly says.
- You may read repository files for context, but do not modify anything.
{instructions}{kept}
Respond with ONLY a JSON object, no prose:
{{"features":[{{"title":"...","chunks":[{{"title":"...","summary":"...","hunks":["H1","H2:1-30"],"notes":[{{"at":"H2:12","text":"..."}}]}}]}}]}}

Hunks:

{body}"#,
        repo = input.repo.path().display(),
        sha = input.base_commit.as_str(),
        base_ref = input.base_ref,
        language = input.language.as_str(),
        instructions = instructions_prompt(input.repo, input.engine, input.extra),
    )
}
