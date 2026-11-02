use super::msbuild::MsbuildMessage;
use super::tail::tail;
use super::ToolDiagnostic;
use crate::ai::{login_path, resolve_program};
use crate::domain::RepoRoot;
use crate::process::async_command;
use crate::project::DotnetProject;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::path::Path;

async fn dotnet(root: &RepoRoot, args: &[&str]) -> Result<(DotnetProject, std::process::Output)> {
    let Some(project) = DotnetProject::find(root) else {
        bail!("not a .NET project (no .sln, .slnx or .csproj in the repository)");
    };
    let dotnet = resolve_program("dotnet").context("dotnet not found: install the .NET SDK (https://dot.net)")?;
    let out = async_command(dotnet)
        .args(args)
        .arg(project.path())
        .current_dir(root.path())
        .env("PATH", login_path())
        // English, single-line messages whatever the system language and terminal.
        .env("DOTNET_CLI_UI_LANGUAGE", "en")
        .env("MSBUILDTERMINALLOGGER", "off")
        .env("DOTNET_NOLOGO", "1")
        .output()
        .await
        .context("failed to run dotnet")?;
    Ok((project, out))
}

pub async fn dotnet_format(root: &RepoRoot) -> Result<()> {
    let (_, out) = dotnet(root, &["format"]).await?;
    if !out.status.success() {
        bail!("dotnet format failed: {}", tail(if out.stderr.is_empty() { &out.stdout } else { &out.stderr }));
    }
    Ok(())
}

/// Compiler and analyzer warnings and errors located in the repository. A full rebuild, as an
/// incremental one stays silent about projects that are up to date.
pub async fn dotnet_build(root: &RepoRoot) -> Result<Vec<ToolDiagnostic>> {
    let (project, out) = dotnet(root, &["build", "--no-incremental", "-consoleLoggerParameters:NoSummary"]).await?;
    let project_dir = project.path().parent().unwrap_or(root.path());
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Some(m) = MsbuildMessage::parse(line) else { continue };
        // Several target frameworks report the same problem once each.
        if !seen.insert((m.file.clone(), m.line, m.code.clone(), m.text.clone())) {
            continue;
        }
        let Some(path) = root.relative(&project_dir.join(Path::new(&m.file))) else { continue };
        found.push(ToolDiagnostic { path, line: m.line, end_line: m.end_line, detail: format!("{} {}: {}", m.level, m.code, m.text), level: m.level, title: m.text });
    }
    if found.is_empty() && !out.status.success() {
        bail!("dotnet build failed: {}", tail(&out.stdout));
    }
    Ok(found)
}
