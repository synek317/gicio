//! Thin wrappers around the `git` CLI.

use crate::domain::{CommitId, FilePath, GitRef, RepoRoot};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("failed to run git")?;
    if !out.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(out.stdout)
}

fn git_with_input(dir: &Path, args: &[&str], input: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut child = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to run git")?;
    child.stdin.take().context("no stdin")?.write_all(input)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(out.stdout)
}

fn git_str(dir: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git(dir, args)?)?.trim().to_string())
}

pub fn toplevel(path: &Path) -> Result<RepoRoot> {
    RepoRoot::new(PathBuf::from(git_str(path, &["rev-parse", "--show-toplevel"])?))
}

pub fn git_dir(root: &RepoRoot) -> Result<PathBuf> {
    Ok(PathBuf::from(git_str(root.path(), &["rev-parse", "--absolute-git-dir"])?))
}

pub fn resolve_commit(root: &RepoRoot, rev: &GitRef) -> Result<CommitId> {
    CommitId::new(&git_str(root.path(), &["rev-parse", "--verify", &format!("{}^{{commit}}", rev.as_str())])?)
}

pub fn list_refs(root: &RepoRoot) -> Result<Vec<String>> {
    let out = git_str(
        root.path(),
        &["for-each-ref", "--sort=-committerdate", "--count=40", "--format=%(refname:short)", "refs/heads", "refs/remotes"],
    )?;
    Ok(out.lines().map(str::to_string).collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub kind: ChangeKind,
    pub path: FilePath,
    pub old_path: Option<FilePath>,
}

/// Files that differ between `base` and the working tree, including untracked files.
pub fn changed_files(root: &RepoRoot, base: &CommitId) -> Result<Vec<ChangedFile>> {
    let raw = git(root.path(), &["diff", "--name-status", "-z", "-M", base.as_str()])?;
    let mut fields = raw.split(|b| *b == 0).filter(|f| !f.is_empty()).map(|f| String::from_utf8_lossy(f).to_string());
    let mut files = Vec::new();
    while let Some(status) = fields.next() {
        let first = fields.next().context("truncated git diff output")?;
        let file = match status.chars().next() {
            Some('A') => ChangedFile { kind: ChangeKind::Added, path: FilePath::new(&first)?, old_path: None },
            Some('D') => ChangedFile { kind: ChangeKind::Deleted, path: FilePath::new(&first)?, old_path: None },
            Some('R') | Some('C') => {
                let new = fields.next().context("truncated rename entry")?;
                ChangedFile { kind: ChangeKind::Renamed, path: FilePath::new(&new)?, old_path: Some(FilePath::new(&first)?) }
            }
            _ => ChangedFile { kind: ChangeKind::Modified, path: FilePath::new(&first)?, old_path: None },
        };
        files.push(file);
    }
    let untracked = git(root.path(), &["ls-files", "--others", "--exclude-standard", "-z"])?;
    for f in untracked.split(|b| *b == 0).filter(|f| !f.is_empty()) {
        files.push(ChangedFile { kind: ChangeKind::Added, path: FilePath::new(&String::from_utf8_lossy(f))?, old_path: None });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

pub fn show_file(root: &RepoRoot, commit: &CommitId, path: &FilePath) -> Result<Vec<u8>> {
    git(root.path(), &["show", &format!("{}:{}", commit.as_str(), path.as_str())])
}

/// Unstages everything (index back to HEAD), keeping the working tree.
pub fn unstage_all(root: &RepoRoot) -> Result<()> {
    git(root.path(), &["reset", "-q"]).map(|_| ())
}

/// Index entry of `path` back to HEAD (removed from the index if HEAD doesn't have it).
pub fn reset_path(root: &RepoRoot, path: &FilePath) -> Result<()> {
    git(root.path(), &["reset", "-q", "--", path.as_str()]).map(|_| ())
}

pub fn remove_from_index(root: &RepoRoot, path: &FilePath) -> Result<()> {
    git(root.path(), &["rm", "--cached", "-q", "--ignore-unmatch", "--", path.as_str()]).map(|_| ())
}

/// Puts `content` into the index as `path` without touching the working tree.
pub fn stage_content(root: &RepoRoot, path: &FilePath, mode: &str, content: &[u8]) -> Result<()> {
    let sha = String::from_utf8(git_with_input(root.path(), &["hash-object", "-w", "--stdin"], content)?)?;
    git(root.path(), &["update-index", "--add", "--cacheinfo", &format!("{mode},{},{}", sha.trim(), path.as_str())]).map(|_| ())
}

/// Short `git diff --cached --stat` of what is staged right now.
pub fn staged_stat(root: &RepoRoot) -> Result<String> {
    git_str(root.path(), &["diff", "--cached", "--stat"])
}

pub fn commit(root: &RepoRoot, message: &str) -> Result<String> {
    git_with_input(root.path(), &["commit", "-q", "-F", "-"], message.as_bytes())?;
    git_str(root.path(), &["rev-parse", "--short", "HEAD"])
}
