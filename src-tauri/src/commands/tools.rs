use crate::jobs::run_tool_job;
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::review::FindingSource;
use gicio_core::tools::{self, ToolDiagnostic};
use serde::Deserialize;
use std::future::Future;
use tauri::{AppHandle, State};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    Fmt,
    Clippy,
    ClippyFix,
    DotnetFormat,
    DotnetBuild,
}

/// Runs a project tool, then re-reads the diff (and, for linters, records their findings).
#[tauri::command]
pub async fn run_tool(app: AppHandle, state: State<'_, AppState>, tool: Tool) -> Result<String, String> {
    let repo = state.session()?.state.repo.clone();
    match tool {
        Tool::Clippy => lint(&app, &state, "cargo clippy", FindingSource::Clippy, tools::cargo_clippy(&repo)).await,
        Tool::DotnetBuild => lint(&app, &state, "dotnet build", FindingSource::Dotnet, tools::dotnet_build(&repo)).await,
        Tool::Fmt => fix(&app, &state, "cargo fmt", "cargo fmt done", tools::cargo_fmt(&repo)).await,
        Tool::ClippyFix => fix(&app, &state, "cargo clippy --fix", "clippy fixes applied", tools::cargo_clippy_fix(&repo)).await,
        Tool::DotnetFormat => fix(&app, &state, "dotnet format", "dotnet format done", tools::dotnet_format(&repo)).await,
    }
}

/// A linter: its warnings in changed files become findings.
async fn lint(app: &AppHandle, state: &State<'_, AppState>, title: &str, source: FindingSource, run: impl Future<Output = anyhow::Result<Vec<ToolDiagnostic>>>) -> Result<String, String> {
    let diags = run_tool_job(app, title, run).await?;
    let mut s = state.session()?;
    let n = s.set_tool_findings(source, diags);
    publish(app, &s);
    Ok(format!("{title}: {n} finding(s) in changed files"))
}

/// A tool that edits code: records what it changes, to look at and keep or undo.
async fn fix(app: &AppHandle, state: &State<'_, AppState>, title: &str, report: &str, run: impl Future<Output = anyhow::Result<()>>) -> Result<String, String> {
    let repo = state.session()?.state.repo.clone();
    let before = gicio_core::git::worktree_tree(&repo).ok();
    run_tool_job(app, title, run).await?;
    let mut s = state.session()?;
    s.refresh(None).map_err(|e| e.to_string())?;
    let changed = s.record_change(before, title, title, None).is_some();
    publish(app, &s);
    Ok(if changed { format!("{report}; see its changes under “AI changes” in the steps list") } else { format!("{report}; nothing changed") })
}
