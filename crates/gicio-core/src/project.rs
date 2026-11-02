//! .NET solutions and projects in a repository, for csharp-ls and the dotnet tools.

use crate::domain::RepoRoot;
use std::path::{Path, PathBuf};

/// How deep below the repository root solutions and projects are looked for.
const MAX_DEPTH: usize = 3;
/// Directories that never hold the solution: build output, dependencies, tooling.
const SKIPPED_DIRS: &[&str] = &["bin", "obj", "node_modules", "target", "packages"];

/// A `.sln`, `.slnx` or `.csproj` file inside the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotnetProject(PathBuf);

impl DotnetProject {
    /// The solution to work on: the shallowest solution, else the shallowest C# project.
    pub fn find(root: &RepoRoot) -> Option<Self> {
        let mut found = Vec::new();
        collect(root.path(), 0, &mut found);
        found.into_iter().min().map(|(_, _, p)| Self(p))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// A solution (as opposed to a single project).
    pub fn is_solution(&self) -> bool {
        rank(&self.0) == Some(0)
    }
}

/// 0 for solutions, 1 for C# projects, `None` for anything else.
fn rank(path: &Path) -> Option<u8> {
    match path.extension()?.to_str()? {
        "sln" | "slnx" => Some(0),
        "csproj" => Some(1),
        _ => None,
    }
}

fn collect(dir: &Path, depth: usize, found: &mut Vec<(u8, usize, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if depth < MAX_DEPTH && !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_ref()) {
                collect(&path, depth + 1, found);
            }
        } else if let Some(r) = rank(&path) {
            found.push((r, depth, path));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(name: &str, files: &[&str]) -> RepoRoot {
        let dir = std::env::temp_dir().join(format!("gicio-test-project-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        for f in files {
            let p = dir.join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "").unwrap();
        }
        RepoRoot::new(dir).unwrap()
    }

    fn found(root: &RepoRoot) -> Option<String> {
        DotnetProject::find(root).map(|p| p.path().strip_prefix(root.path()).unwrap().to_string_lossy().replace('\\', "/"))
    }

    #[test]
    fn prefers_a_solution_over_projects() {
        let root = repo("sln", &["src/App/App.csproj", "src/App.sln", "tests/Tests.csproj"]);
        assert_eq!(found(&root).as_deref(), Some("src/App.sln"));
        assert!(DotnetProject::find(&root).unwrap().is_solution());
    }

    #[test]
    fn takes_the_shallowest_project_without_a_solution() {
        let root = repo("csproj", &["src/App/App.csproj", "App.csproj"]);
        assert_eq!(found(&root).as_deref(), Some("App.csproj"));
        assert!(!DotnetProject::find(&root).unwrap().is_solution());
    }

    #[test]
    fn ignores_build_output_and_other_repos() {
        let root = repo("none", &["bin/Debug/x.csproj", ".git/y.sln", "Cargo.toml", "a/b/c/d/Deep.csproj"]);
        assert_eq!(found(&root), None);
    }
}
