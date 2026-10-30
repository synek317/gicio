use super::programs::{login_path, resolve_program};
use super::{AccessMode, AiBackend, Engine};
use crate::domain::RepoRoot;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::process::Command;

const CLAUDE_READ_TOOLS: &[&str] = &["Read", "Grep", "Glob", "LS", "Skill"];
const CLAUDE_EDIT_TOOLS: &[&str] = &[
    "Read", "Grep", "Glob", "LS", "Skill", "Edit", "MultiEdit", "Write",
    "Bash(cargo check:*)", "Bash(cargo build:*)", "Bash(cargo test:*)", "Bash(cargo clippy:*)", "Bash(git diff:*)", "Bash(git status:*)",
];

/// How a CLI reports its result.
pub enum Output {
    /// Claude `stream-json` events on stdout.
    ClaudeStream,
    /// The final message lands in a file (codex `--output-last-message`).
    LastMessageFile(PathBuf),
    /// All of stdout is the answer.
    Stdout,
}

pub struct Prepared {
    pub cmd: Command,
    pub stdin: Option<String>,
    pub output: Output,
}

fn program(name: &str) -> Result<Command> {
    let path = resolve_program(name).ok_or_else(|| anyhow!("`{name}` not found on PATH (set GICIO_{}_BIN to override)", name.to_uppercase()))?;
    let mut cmd = Command::new(path);
    cmd.env("PATH", login_path());
    Ok(cmd)
}

fn temp_file(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    std::env::temp_dir().join(format!("gicio-{tag}-{}-{nanos}.txt", std::process::id()))
}

/// Builds the CLI invocation for one non-interactive AI turn.
pub fn prepare(backend: &AiBackend, repo: &RepoRoot, prompt: &str, mode: AccessMode) -> Result<Prepared> {
    let model = backend.model.as_ref().map(|m| m.as_str());
    let edit = mode == AccessMode::Edit;
    let mut prepared = match &backend.engine {
        Engine::Claude => {
            let mut c = program("claude")?;
            c.args(["-p", "--output-format", "stream-json", "--verbose"]);
            if let Some(m) = model {
                c.args(["--model", m]);
            }
            if let Some(e) = backend.effort {
                c.args(["--effort", e.claude()]);
            }
            if edit {
                c.args(["--permission-mode", "acceptEdits"]);
                c.arg("--allowedTools").args(CLAUDE_EDIT_TOOLS);
            } else {
                c.arg("--allowedTools").args(CLAUDE_READ_TOOLS);
                c.arg("--disallowedTools").args(["Edit", "MultiEdit", "Write", "NotebookEdit", "Bash"]);
            }
            Prepared { cmd: c, stdin: Some(prompt.to_string()), output: Output::ClaudeStream }
        }
        Engine::Codex => {
            let mut c = program("codex")?;
            let out = temp_file("codex");
            c.args(["exec", "--color", "never", "--skip-git-repo-check"]);
            c.args(["--sandbox", if edit { "workspace-write" } else { "read-only" }]);
            c.arg("--output-last-message").arg(&out);
            if let Some(m) = model {
                c.args(["--model", m]);
            }
            if let Some(e) = backend.effort {
                c.args(["-c", &format!("model_reasoning_effort=\"{}\"", e.codex())]);
            }
            c.arg("-");
            Prepared { cmd: c, stdin: Some(prompt.to_string()), output: Output::LastMessageFile(out) }
        }
        Engine::Copilot => {
            let mut c = program("copilot")?;
            c.arg("-p").arg(prompt);
            if let Some(m) = model {
                c.args(["--model", m]);
            }
            c.arg("--allow-all-tools");
            if !edit {
                c.args(["--deny-tool", "write", "--deny-tool", "shell"]);
            }
            Prepared { cmd: c, stdin: None, output: Output::Stdout }
        }
        Engine::Custom { command } => {
            let mut c = Command::new("/bin/sh");
            c.env("PATH", login_path());
            c.arg("-c").arg(command.as_str());
            c.env("GICIO_MODE", if edit { "edit" } else { "read-only" });
            if let Some(m) = model {
                c.env("GICIO_MODEL", m);
            }
            Prepared { cmd: c, stdin: Some(prompt.to_string()), output: Output::Stdout }
        }
    };
    prepared.cmd.current_dir(repo.path());
    Ok(prepared)
}
