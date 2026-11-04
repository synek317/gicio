//! Thin wrappers around the `git` CLI.

use crate::domain::{CommitId, CommitMessage, FilePath, GitRef, RepoRoot, TreeId};
use crate::process::sync_command;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = sync_command("git")
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
    let mut child = sync_command("git")
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

/// `git` with its own index file, so the real index (and what is staged) stays untouched.
fn git_with_index(dir: &Path, index: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = sync_command("git").arg("-C").arg(dir).args(args).env("GIT_INDEX_FILE", index).output().context("failed to run git")?;
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

/// Files in the working tree that git knows about or would add: tracked plus untracked, not ignored.
pub fn listed_files(root: &RepoRoot) -> Result<Vec<FilePath>> {
    let raw = git(root.path(), &["ls-files", "--cached", "--others", "--exclude-standard", "--deduplicate", "-z"])?;
    let mut files: Vec<FilePath> = raw.split(|b| *b == 0).filter_map(|f| FilePath::new(&String::from_utf8_lossy(f)).ok()).collect();
    files.sort();
    Ok(files)
}

/// `path` at `commit` as it would look checked out (line endings converted like the working tree,
/// e.g. CRLF with `core.autocrlf` on Windows), so it compares cleanly with the file on disk.
pub fn show_file(root: &RepoRoot, commit: &CommitId, path: &FilePath) -> Result<Vec<u8>> {
    git(root.path(), &["cat-file", "--filters", &format!("{}:{}", commit.as_str(), path.as_str())])
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

/// Puts `content` (working-tree form) into the index as `path` without touching the working tree.
/// `--path` applies the same conversions as `git add` (e.g. CRLF back to LF).
pub fn stage_content(root: &RepoRoot, path: &FilePath, mode: &str, content: &[u8]) -> Result<()> {
    let sha = String::from_utf8(git_with_input(root.path(), &["hash-object", "-w", "--stdin", "--path", path.as_str()], content)?)?;
    git(root.path(), &["update-index", "--add", "--cacheinfo", &format!("{mode},{},{}", sha.trim(), path.as_str())]).map(|_| ())
}

/// Whether the index (or HEAD) records `path` as executable. For systems without an executable bit.
pub fn executable_in_index(root: &RepoRoot, path: &FilePath) -> bool {
    git_str(root.path(), &["ls-files", "--stage", "--", path.as_str()]).is_ok_and(|s| s.starts_with("100755"))
}

/// `git diff --cached` of what is staged right now.
pub fn staged_diff(root: &RepoRoot) -> Result<String> {
    Ok(String::from_utf8_lossy(&git(root.path(), &["diff", "--cached"])?).into_owned())
}

/// Short `git diff --cached --stat` of what is staged right now.
pub fn staged_stat(root: &RepoRoot) -> Result<String> {
    git_str(root.path(), &["diff", "--cached", "--stat"])
}

/// Commits what is staged; returns the short SHA of the new commit.
pub fn commit(root: &RepoRoot, message: &CommitMessage) -> Result<String> {
    git_with_input(root.path(), &["commit", "-q", "-F", "-"], message.as_str().as_bytes())?;
    git_str(root.path(), &["rev-parse", "--short", "HEAD"])
}

/// Replaces the last commit with one that also has what is staged, under `message`.
pub fn amend(root: &RepoRoot, message: &CommitMessage) -> Result<String> {
    git_with_input(root.path(), &["commit", "-q", "--amend", "-F", "-"], message.as_str().as_bytes())?;
    git_str(root.path(), &["rev-parse", "--short", "HEAD"])
}

/// Full message of the last commit.
pub fn head_message(root: &RepoRoot) -> Result<String> {
    git_str(root.path(), &["log", "-1", "--format=%B", "HEAD"])
}

/// Cheap fingerprint of the changed part of the working tree (names, sizes, modification times of
/// files differing from `base`, plus untracked files). Staging does not affect it.
pub fn worktree_fingerprint(root: &RepoRoot, base: &CommitId) -> Result<u64> {
    let mut names = git(root.path(), &["diff", "--name-only", "-z", base.as_str()])?;
    names.extend(git(root.path(), &["ls-files", "--others", "--exclude-standard", "-z"])?);
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    };
    for name in names.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        mix(name);
        let meta = std::fs::metadata(root.path().join(String::from_utf8_lossy(name).as_ref()));
        match meta {
            Ok(m) => {
                mix(&m.len().to_le_bytes());
                let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
                mix(&t.to_le_bytes());
            }
            Err(_) => mix(b"-"),
        }
    }
    Ok(h)
}

/// Records the whole working tree (tracked and untracked, minus ignored files) as a tree object,
/// without touching the index. Starts from a copy of the index, so unchanged files are not re-read.
pub fn worktree_tree(root: &RepoRoot) -> Result<TreeId> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = git_dir(root)?;
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let index = dir.join(format!("gicio-snapshot-{}-{n}.index", std::process::id()));
    let real = dir.join("index");
    if real.exists() {
        std::fs::copy(&real, &index).context("cannot copy the git index")?;
    }
    let tree = git_with_index(root.path(), &index, &["add", "-A", "--", "."]).and_then(|_| git_with_index(root.path(), &index, &["write-tree"]));
    let _ = std::fs::remove_file(&index);
    TreeId::new(&String::from_utf8(tree?)?)
}

/// Files that differ between two snapshots.
pub fn changed_between(root: &RepoRoot, a: &TreeId, b: &TreeId) -> Result<Vec<FilePath>> {
    let out = git(root.path(), &["diff-tree", "-r", "--no-renames", "--name-only", "-z", a.as_str(), b.as_str()])?;
    Ok(out.split(|b| *b == 0).filter(|n| !n.is_empty()).filter_map(|n| FilePath::new(&String::from_utf8_lossy(n)).ok()).collect())
}

/// `path` in a snapshot as it would look on disk; `None` when the snapshot has no such file.
pub fn tree_file(root: &RepoRoot, tree: &TreeId, path: &FilePath) -> Option<Vec<u8>> {
    git(root.path(), &["cat-file", "--filters", &format!("{}:{}", tree.as_str(), path.as_str())]).ok()
}

/// `path` in a snapshot exactly as stored (line endings as `git add` normalised them).
pub fn tree_file_stored(root: &RepoRoot, tree: &TreeId, path: &FilePath) -> Option<Vec<u8>> {
    git(root.path(), &["cat-file", "blob", &format!("{}:{}", tree.as_str(), path.as_str())]).ok()
}

/// Whether `path` on disk has the content a snapshot recorded for it (both missing counts too).
/// Compared as `git add` would store it, so line endings alone (CRLF with `core.autocrlf`) are no edit.
pub fn worktree_matches(root: &RepoRoot, tree: &TreeId, path: &FilePath) -> bool {
    let recorded = git_str(root.path(), &["rev-parse", "--verify", "-q", &format!("{}:{}", tree.as_str(), path.as_str())]).ok();
    if !root.join(path).exists() {
        return recorded.is_none();
    }
    git_str(root.path(), &["hash-object", "--", path.as_str()]).is_ok_and(|on_disk| recorded.as_deref() == Some(on_disk.as_str()))
}
