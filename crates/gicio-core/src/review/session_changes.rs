use super::{AgentChange, ChangeStatus, LineKey, MarkStatus, Session};
use crate::diff::{compute_hunks, FileDiff, FileStatus, Side};
use crate::domain::{ChangeId, FilePath, ThreadId, TreeId};
use crate::git;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;

fn text(bytes: Option<Vec<u8>>) -> String {
    bytes.map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
}

impl Session {
    /// The working tree right before an AI job that may edit it; `None` if it cannot be recorded.
    pub fn snapshot_tree(&self) -> Option<TreeId> {
        git::worktree_tree(&self.state.repo).map_err(|e| eprintln!("snapshot failed: {e:#}")).ok()
    }

    /// Records what changed since `before` as an AI change to keep or undo. `None` when nothing did.
    pub fn record_change(&mut self, before: Option<TreeId>, title: &str, by: &str, thread: Option<ThreadId>) -> Option<ChangeId> {
        let before = before?;
        let after = self.snapshot_tree()?;
        if after == before {
            return None;
        }
        let files = git::changed_between(&self.state.repo, &before, &after).ok().filter(|f| !f.is_empty())?;
        let id = self.state.next_change_id();
        self.state.changes.push(AgentChange { id: id.clone(), title: title.chars().take(80).collect(), by: by.to_string(), thread, before, after, files, status: ChangeStatus::Pending });
        self.touch();
        let _ = self.save();
        Some(id)
    }

    /// "`a.rs`, `b.rs`": the files of a change, for the note about it in a conversation.
    pub fn change_files(&self, id: &ChangeId) -> String {
        self.change(id).map(|c| c.files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")).unwrap_or_default()
    }

    fn change(&self, id: &ChangeId) -> Result<&AgentChange> {
        self.state.changes.iter().find(|c| &c.id == id).context("no such AI change")
    }

    /// The change itself: each file as it was before the AI job against how it was right after.
    pub fn change_diff(&self, id: &ChangeId) -> Result<Vec<FileDiff>> {
        let c = self.change(id)?;
        let repo = &self.state.repo;
        Ok(c.files
            .iter()
            .map(|path| {
                let (old, new) = (git::tree_file(repo, &c.before, path), git::tree_file(repo, &c.after, path));
                let status = match (&old, &new) {
                    (None, _) => FileStatus::Added,
                    (_, None) => FileStatus::Deleted,
                    _ => FileStatus::Modified,
                };
                let hunks = compute_hunks(path, &text(old), &text(new), self.state.diff_options);
                FileDiff { path: path.clone(), old_path: None, status, binary: false, hunks }
            })
            .collect())
    }

    /// Keeps the change: the review lines it brought in count as reviewed.
    pub fn accept_change(&mut self, id: &ChangeId) -> Result<()> {
        let c = self.change(id)?.clone();
        if c.status != ChangeStatus::Pending {
            bail!("this AI change was already {}", if c.status == ChangeStatus::Accepted { "accepted" } else { "rejected" });
        }
        let mut mine = Vec::new();
        for path in &c.files {
            let Some(file) = self.file(path) else { continue };
            // Review lines of this file as they were before the AI job; anything else is the AI's.
            let base = text(git::show_file(&self.state.repo, &self.state.base_commit, path).ok());
            let before: HashSet<(Side, String)> = compute_hunks(path, &base, &text(git::tree_file(&self.state.repo, &c.before, path)), self.state.diff_options)
                .iter()
                .flat_map(|h| h.lines.iter())
                .filter_map(|l| l.review_pos().map(|(side, _)| (side, l.text.clone())))
                .collect();
            mine.extend(file.hunks.iter().flat_map(|h| h.lines.iter()).filter_map(|l| {
                let (side, line) = l.review_pos()?;
                (!before.contains(&(side, l.text.clone()))).then(|| LineKey { path: path.clone(), side, line })
            }));
        }
        self.set_marks(mine, Some(MarkStatus::Accepted));
        self.set_change_status(id, ChangeStatus::Accepted);
        self.save()
    }

    /// Undoes the change: its files go back to how they were before the AI job. A file edited again
    /// since then is left alone (undoing would lose those edits); those are returned.
    pub fn reject_change(&mut self, id: &ChangeId) -> Result<Vec<FilePath>> {
        let c = self.change(id)?.clone();
        if c.status != ChangeStatus::Pending {
            bail!("this AI change was already {}", if c.status == ChangeStatus::Accepted { "accepted" } else { "rejected" });
        }
        let mut skipped = Vec::new();
        for path in &c.files {
            let abs = self.state.repo.join(path);
            if std::fs::read(&abs).ok() != git::tree_file(&self.state.repo, &c.after, path) {
                skipped.push(path.clone());
                continue;
            }
            match git::tree_file(&self.state.repo, &c.before, path) {
                Some(bytes) => std::fs::write(&abs, bytes).with_context(|| format!("cannot restore {path}"))?,
                None => std::fs::remove_file(&abs).with_context(|| format!("cannot remove {path}"))?,
            }
        }
        self.set_change_status(id, ChangeStatus::Rejected);
        self.refresh(None)?;
        Ok(skipped)
    }

    fn set_change_status(&mut self, id: &ChangeId, status: ChangeStatus) {
        if let Some(c) = self.state.changes.iter_mut().find(|c| &c.id == id) {
            c.status = status;
        }
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiBackend;
    use crate::domain::GitRef;
    use crate::stage::StageMode;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// A throwaway repository with `a.rs` committed and one line of the reviewed change on top.
    fn repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gicio-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| assert!(Command::new("git").current_dir(&dir).args(args).output().unwrap().status.success());
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "base"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn reviewed() {}\n").unwrap();
        dir
    }

    fn open(dir: &Path) -> Session {
        Session::open(dir, GitRef::new("HEAD").unwrap(), AiBackend::default(), StageMode::Memory).unwrap()
    }

    /// What an AI job does: edits `a.rs` and adds `b.rs`, recorded as a change.
    fn ai_edit(s: &mut Session, dir: &Path) -> ChangeId {
        let before = s.snapshot_tree();
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn reviewed() {}\nfn by_ai() {}\n").unwrap();
        std::fs::write(dir.join("b.rs"), "fn new_file() {}\n").unwrap();
        s.refresh(None).unwrap();
        s.record_change(before, "edit", "ai", None).unwrap()
    }

    fn status(s: &Session, path: &str, line: u32) -> Option<MarkStatus> {
        s.state.marks.status(&LineKey { path: FilePath::new(path).unwrap(), side: Side::New, line })
    }

    #[test]
    fn rejecting_puts_files_back_as_they_were_before_the_ai() {
        let dir = repo("reject");
        let mut s = open(&dir);
        let id = ai_edit(&mut s, &dir);
        assert_eq!(s.change(&id).unwrap().files.len(), 2);
        assert!(s.reject_change(&id).unwrap().is_empty());
        assert_eq!(std::fs::read_to_string(dir.join("a.rs")).unwrap(), "fn a() {}\nfn reviewed() {}\n");
        assert!(!dir.join("b.rs").exists());
        assert!(s.reject_change(&id).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn accepting_marks_only_what_the_ai_wrote() {
        let dir = repo("accept");
        let mut s = open(&dir);
        let id = ai_edit(&mut s, &dir);
        s.accept_change(&id).unwrap();
        assert_eq!(status(&s, "a.rs", 3), Some(MarkStatus::Accepted));
        assert_eq!(status(&s, "b.rs", 1), Some(MarkStatus::Accepted));
        assert_eq!(status(&s, "a.rs", 2), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_edited_again_after_the_ai_is_not_undone() {
        let dir = repo("skip");
        let mut s = open(&dir);
        let id = ai_edit(&mut s, &dir);
        std::fs::write(dir.join("b.rs"), "fn edited_by_me() {}\n").unwrap();
        assert_eq!(s.reject_change(&id).unwrap(), [FilePath::new("b.rs").unwrap()]);
        assert!(dir.join("b.rs").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
