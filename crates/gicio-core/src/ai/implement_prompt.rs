use super::intent_section::intent_section;
use super::{instructions_prompt, ExtraInstructions};
use crate::review::{Requirement, Session};

/// Prompt for finishing requirements of the task that the change does not fully meet.
pub fn build_implement_prompt(s: &Session, requirements: &[Requirement], extra: &ExtraInstructions) -> String {
    let mut out = format!(
        "A code change does not fully do what it was supposed to. Finish the requirements below by editing files in the working tree.\nRepository: {}\n",
        s.state.repo.path().display()
    );
    out.push_str(&intent_section(s.state.intent.as_ref()));
    out.push_str("\nRequirements to finish, with what a check of the code found:\n");
    for r in requirements {
        out.push_str(&format!("- {} ({:?}): {}\n", r.text, r.verdict, r.note));
    }
    out.push_str(&instructions_prompt(&s.state.repo, &s.state.backend.engine, extra));
    out.push_str(
        "\nDo only what these requirements need and change nothing unrelated; never commit or change git state. If one is \
         already met or cannot be done, leave the code alone and say so. Finish with a short list: one line per requirement, what you did.\n",
    );
    out
}
