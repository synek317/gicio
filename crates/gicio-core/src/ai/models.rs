//! Which models an AI CLI accepts: Codex stores the list for the account, the others name some in
//! what their `--help` says about `--model`.

use super::codex_models;
use super::{login_path, resolve_program};
use crate::process::async_command;
use std::time::Duration;

/// Arguments that print the help describing `--model`.
fn help_args(program: &str) -> &'static [&'static str] {
    match program {
        "codex" => &["exec", "--help"],
        _ => &["--help"],
    }
}

/// Models the installed CLI knows about; empty when it is missing or names none.
pub async fn detect_models(program: &str) -> Vec<String> {
    if program == "codex"
        && let Some(models) = codex_models::cached_models()
    {
        return models;
    }
    let Some(bin) = resolve_program(program) else { return Vec::new() };
    let help = async_command(bin).args(help_args(program)).env("PATH", login_path()).output();
    match tokio::time::timeout(Duration::from_secs(5), help).await {
        Ok(Ok(out)) => models_in_help(&String::from_utf8_lossy(&out.stdout)),
        _ => Vec::new(),
    }
}

/// Quoted names in the description of the `--model` option (up to the next option).
pub fn models_in_help(help: &str) -> Vec<String> {
    let mut lines = help.lines().skip_while(|l| !l.contains("--model"));
    let Some(first) = lines.next() else { return Vec::new() };
    let text: Vec<char> = std::iter::once(first).chain(lines.take_while(|l| !l.trim_start().starts_with('-'))).collect::<Vec<_>>().join(" ").chars().collect();
    let mut found: Vec<String> = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let q = text[i];
        // An opening quote, not an apostrophe inside a word (`model's`).
        let opens = matches!(q, '\'' | '"' | '`') && (i == 0 || !text[i - 1].is_alphanumeric());
        let close = opens.then(|| (i + 1..text.len()).find(|&j| text[j] == q && text.get(j + 1).is_none_or(|c| !c.is_alphanumeric()))).flatten();
        match close {
            Some(j) => {
                let name: String = text[i + 1..j].iter().collect();
                if is_model_name(&name) && !found.contains(&name) {
                    found.push(name);
                }
                i = j + 1;
            }
            None => i += 1,
        }
    }
    found
}

pub(super) fn is_model_name(s: &str) -> bool {
    s.len() < 60 && s.chars().any(|c| c.is_ascii_alphabetic()) && s.chars().all(|c| c.is_ascii_alphanumeric() || "-._:/".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_aliases_from_claude_help() {
        let help = "  --mcp-config <configs...>  Load MCP servers\n  --model <model>   Model for the current session. Provide\n                    an alias for the latest model (e.g.\n                    'fable', 'opus', or 'sonnet') or a\n                    model's full name (e.g. 'claude-sonnet-4-5').\n  --permission-mode <mode>  'plan' or 'auto'\n";
        assert_eq!(models_in_help(help), ["fable", "opus", "sonnet", "claude-sonnet-4-5"]);
    }

    #[test]
    fn reads_choices_and_ignores_help_without_names() {
        assert_eq!(models_in_help("--model <model>  Set the AI model (choices: \"gpt-5\", \"claude-sonnet-4.5\")"), ["gpt-5", "claude-sonnet-4.5"]);
        assert!(models_in_help("-m, --model <MODEL>  Model the agent should use\n").is_empty());
        assert!(models_in_help("no such option").is_empty());
    }
}
