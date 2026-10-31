//! Command line: `gicio [PATH] [--since REF] [--start]`.

use gicio_core::domain::{GitRef, RepoRoot};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

const USAGE: &str = "\
Review huge changes with AI assist.

Usage: gicio [PATH] [--since REF] [--start]

  PATH          repository to review (default: the current directory)
  --since REF   review everything since REF (a branch or commit) instead of uncommitted changes
  --start       start the review right away, skipping the start screen
  -h, --help    show this help";

/// What the start screen should be prefilled with.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArgs {
    pub repo: Option<RepoRoot>,
    pub since: Option<GitRef>,
    pub start: bool,
    pub warning: Option<String>,
}

impl LaunchArgs {
    /// Parses the command line; prints help and exits for `--help`.
    pub fn from_env() -> Self {
        let Some(args) = Self::parse(std::env::args().skip(1), std::env::current_dir().ok()) else {
            println!("{USAGE}");
            std::process::exit(0);
        };
        if let Some(w) = &args.warning {
            eprintln!("gicio: {w}");
        }
        args
    }

    /// `None` means help was requested.
    fn parse(args: impl IntoIterator<Item = String>, cwd: Option<PathBuf>) -> Option<Self> {
        let mut path: Option<PathBuf> = None;
        let mut since = None;
        let mut start = false;
        let mut warnings = Vec::new();
        let mut args = args.into_iter();
        while let Some(a) = args.next() {
            match a.as_str() {
                "-h" | "--help" => return None,
                "--start" => start = true,
                "--since" => match args.next().map(|r| GitRef::new(&r)) {
                    Some(Ok(r)) => since = Some(r),
                    _ => warnings.push("--since needs a branch or commit".to_string()),
                },
                // macOS adds a process serial number when launched from Finder.
                s if s.starts_with("-psn_") => {}
                s if s.starts_with('-') => warnings.push(format!("unknown option {s} (see gicio --help)")),
                s => path = Some(PathBuf::from(s)),
            }
        }
        let explicit = path.is_some();
        let dir = path.or(cwd);
        let repo = dir.as_ref().and_then(|d| gicio_core::git::toplevel(d).ok());
        if repo.is_none() && explicit {
            warnings.push(format!("not a git repository: {}", dir.map(|d| d.display().to_string()).unwrap_or_default()));
        }
        Some(Self { start: start && repo.is_some(), repo, since, warning: (!warnings.is_empty()).then(|| warnings.join("; ")) })
    }
}

/// Launch arguments, handed to the UI once (returning to the start screen must not auto-start again).
pub struct Launch(pub Mutex<Option<LaunchArgs>>);

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str], cwd: &str) -> Option<LaunchArgs> {
        LaunchArgs::parse(args.iter().map(|s| s.to_string()), Some(PathBuf::from(cwd)))
    }

    fn this_repo() -> String {
        format!("{}/src", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn defaults_to_the_repo_of_the_current_directory() {
        let a = parse(&[], &this_repo()).unwrap();
        assert!(a.repo.unwrap().path().join("src-tauri").is_dir());
        assert!(a.since.is_none() && !a.start && a.warning.is_none());
    }

    #[test]
    fn explicit_path_since_and_start() {
        let a = parse(&[&this_repo(), "--since", "main", "--start"], "/").unwrap();
        assert!(a.repo.is_some());
        assert_eq!(a.since.unwrap().as_str(), "main");
        assert!(a.start);
    }

    #[test]
    fn outside_a_repository() {
        let a = parse(&[], "/").unwrap();
        assert!(a.repo.is_none() && a.warning.is_none());
        let a = parse(&["/", "--start"], "/").unwrap();
        assert!(a.repo.is_none() && !a.start && a.warning.unwrap().contains("not a git repository"));
    }

    #[test]
    fn help_and_unknown_options() {
        assert!(parse(&["--help"], "/").is_none());
        assert!(parse(&["--bogus"], "/").unwrap().warning.unwrap().contains("--bogus"));
    }
}
