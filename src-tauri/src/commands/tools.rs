use crate::jobs::run_tool_job;
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::tools;
use serde::Deserialize;
use tauri::{AppHandle, State};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    Fmt,
    Clippy,
    ClippyFix,
}

/// Runs a project tool, then re-reads the diff (and, for clippy, records its findings).
#[tauri::command]
pub async fn run_tool(app: AppHandle, state: State<'_, AppState>, tool: Tool) -> Result<String, String> {
    let repo = state.session()?.state.repo.clone();
    // fmt and clippy --fix edit code: record what they change, to look at and keep or undo.
    let before = if matches!(tool, Tool::Clippy) { None } else { gicio_core::git::worktree_tree(&repo).ok() };
    let report = match tool {
        Tool::Fmt => run_tool_job(&app, "cargo fmt", tools::cargo_fmt(&repo)).await.map(|_| "cargo fmt done".to_string())?,
        Tool::ClippyFix => run_tool_job(&app, "cargo clippy --fix", tools::cargo_clippy_fix(&repo)).await.map(|_| "clippy fixes applied".to_string())?,
        Tool::Clippy => {
            let diags = run_tool_job(&app, "cargo clippy", tools::cargo_clippy(&repo)).await?;
            let mut s = state.session()?;
            let n = s.set_clippy_findings(diags);
            publish(&app, &s);
            return Ok(format!("clippy: {n} finding(s) in changed files"));
        }
    };
    let mut s = state.session()?;
    s.refresh(None).map_err(|e| e.to_string())?;
    let title = if matches!(tool, Tool::Fmt) { "cargo fmt" } else { "cargo clippy --fix" };
    let changed = s.record_change(before, title, title, None).is_some();
    publish(&app, &s);
    Ok(if changed { format!("{report}; see its changes under “AI changes” in the steps list") } else { format!("{report}; nothing changed") })
}

