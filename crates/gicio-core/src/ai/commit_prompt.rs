use super::{instructions_prompt, ExtraInstructions};
use crate::review::Session;

/// Commit message for what is staged right now (`staged` = `git diff --cached`).
pub fn build_commit_prompt(s: &Session, staged: &str, extra: &ExtraInstructions) -> String {
    let mut out = String::from("Write a git commit message for the staged changes below.\n");
    if let Some(sum) = &s.state.summary {
        out.push_str(&format!("\nSummary of the whole change, for context:\n{}\n", sum.body));
    }
    let staged: String = staged.chars().take(200_000).collect();
    out.push_str(&format!("\nStaged diff:\n```diff\n{staged}\n```\n"));
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nFollow the repository's existing commit style if you can see it (git log). Otherwise: a subject line in the \
         imperative mood, at most 72 characters, a blank line, then a short body saying what and why. Respond with ONLY \
         the commit message, no code fences, no commentary. Do not modify anything.\n",
    );
    out
}
