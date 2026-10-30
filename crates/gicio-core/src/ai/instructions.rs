//! Project instruction and skill files of the common agent tools. Each CLI reads its own
//! files natively; the rest are passed in the prompt so every backend follows the same rules.

use super::{Engine, ExtraInstructions};
use crate::domain::RepoRoot;
use serde::Serialize;
use std::path::Path;

const MAX_FILE: usize = 8_000;
const MAX_TOTAL: usize = 24_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstructionKind {
    Claude,
    Agents,
    Copilot,
    Cursor,
    Gemini,
    ClaudeSkill,
    Skill,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionFile {
    /// Repo-relative path.
    pub path: String,
    pub kind: InstructionKind,
    /// Skill name and description from the SKILL.md front matter.
    pub skill: Option<(String, String)>,
    /// Whether the current backend loads this file by itself.
    pub native: bool,
}

fn native(engine: &Engine, kind: InstructionKind) -> bool {
    use InstructionKind::*;
    match engine {
        Engine::Claude => matches!(kind, Claude | ClaudeSkill),
        Engine::Codex => matches!(kind, Agents),
        Engine::Copilot => matches!(kind, Copilot | Agents),
        Engine::Custom { .. } => false,
    }
}

fn front_matter(text: &str) -> Option<(String, String)> {
    let body = text.strip_prefix("---")?.split("\n---").next()?;
    let field = |k: &str| body.lines().find_map(|l| l.strip_prefix(k)).map(|v| v.trim().trim_matches('"').to_string());
    Some((field("name:")?, field("description:").unwrap_or_default()))
}

fn list_dir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
    v.sort();
    v
}

pub fn discover_instructions(repo: &RepoRoot, engine: &Engine) -> Vec<InstructionFile> {
    use InstructionKind::*;
    let root = repo.path();
    let mut found: Vec<(String, InstructionKind)> = Vec::new();
    for (p, k) in [
        ("CLAUDE.md", Claude),
        ("CLAUDE.local.md", Claude),
        (".claude/CLAUDE.md", Claude),
        ("AGENTS.md", Agents),
        (".github/copilot-instructions.md", Copilot),
        (".cursorrules", Cursor),
        ("GEMINI.md", Gemini),
    ] {
        if root.join(p).is_file() {
            found.push((p.to_string(), k));
        }
    }
    for (dir, kind) in [(".github/instructions", Copilot), (".cursor/rules", Cursor)] {
        for f in list_dir(&root.join(dir)) {
            if f.extension().is_some_and(|e| e == "md" || e == "mdc") {
                found.push((format!("{dir}/{}", f.file_name().unwrap_or_default().to_string_lossy()), kind));
            }
        }
    }
    for (dir, kind) in [(".claude/skills", ClaudeSkill), (".agents/skills", Skill), (".codex/skills", Skill), (".github/skills", Skill)] {
        for d in list_dir(&root.join(dir)) {
            if d.join("SKILL.md").is_file() {
                found.push((format!("{dir}/{}/SKILL.md", d.file_name().unwrap_or_default().to_string_lossy()), kind));
            }
        }
    }
    found
        .into_iter()
        .map(|(path, kind)| {
            let skill = matches!(kind, ClaudeSkill | Skill).then(|| std::fs::read_to_string(root.join(&path)).ok().and_then(|t| front_matter(&t))).flatten();
            InstructionFile { native: native(engine, kind), path, kind, skill }
        })
        .collect()
}

/// Prompt section with the instructions the backend does not read by itself.
pub fn instructions_prompt(repo: &RepoRoot, engine: &Engine, extra: &ExtraInstructions) -> String {
    let files = discover_instructions(repo, engine);
    let mut out = String::new();
    let mut budget = MAX_TOTAL;
    for f in files.iter().filter(|f| !f.native && f.skill.is_none()) {
        let Ok(text) = std::fs::read_to_string(repo.path().join(&f.path)) else { continue };
        let text: String = text.chars().take(MAX_FILE.min(budget)).collect();
        if text.is_empty() {
            break;
        }
        budget -= text.len().min(budget);
        out.push_str(&format!("\n--- Project instructions from {} ---\n{}\n", f.path, text.trim()));
    }
    let skills: Vec<_> = files.iter().filter(|f| !f.native).filter_map(|f| f.skill.as_ref().map(|(n, d)| format!("- {n}: {d} (read {} when relevant)", f.path))).collect();
    if !skills.is_empty() {
        out.push_str(&format!("\n--- Project skills ---\n{}\n", skills.join("\n")));
    }
    if !extra.is_empty() {
        out.push_str(&format!("\n--- The reviewer's own instructions (highest priority) ---\n{}\n", extra.as_str()));
    }
    if out.is_empty() { out } else { format!("\nFollow these instructions:{out}\n") }
}
