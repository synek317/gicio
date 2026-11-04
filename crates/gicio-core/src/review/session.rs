use super::{compare, line_signature, similarity, fallback_plan, reconcile, Chunk, ChunkOrigin, Coverage, FollowUp, HunkRef, LineKey, Mark, MarkStatus, PlannedChunk, ReviewState, Snapshot};
use super::{Anchor, Message, Thread, ThreadStatus};
use crate::ai::AiBackend;
use crate::diff::{load_diff, FileDiff, Hunk, Side};
use crate::domain::{ChunkId, FilePath, GitRef, HunkId, ThreadId};
use crate::stage::StageMode;
use crate::store::StoreKey;
use crate::{git, store};

/// Below this share of common changed lines, a stored review is for a different change.
const FRESH_BELOW: f64 = 0.2;
use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct Session {
    pub state: ReviewState,
    pub files: Vec<FileDiff>,
    pub planning: bool,
    pub chat_busy: bool,
    pub summary_busy: bool,
    pub notice: Option<String>,
    version: u64,
    undo: Vec<Vec<(LineKey, Option<Mark>)>>,
    store_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub new_hunks: Vec<HunkId>,
    pub touched_files: Vec<FilePath>,
}

impl Session {
    pub fn open(path: &Path, base: GitRef, backend: AiBackend, stage_mode: StageMode) -> Result<Self> {
        let root = git::toplevel(path)?;
        let commit = git::resolve_commit(&root, &base)?;
        let store_path = store::path_for(&root, &StoreKey::for_base(&base))?;
        let mut restored = store::load(&store_path).unwrap_or(None);
        if restored.is_none() {
            restored = store::load(&store::legacy_path(&root, &commit)?).unwrap_or(None);
        }
        let files = load_diff(&root, &commit, restored.as_ref().map(|s| s.diff_options).unwrap_or_default())?;

        // Same change with some edits, or something else entirely?
        let signature = line_signature(&files);
        let mut notice = None;
        if let Some(prev) = &restored
            && !prev.line_signature.is_empty()
        {
            let (common, gone, added) = compare(&prev.line_signature, &signature);
            if similarity(&prev.line_signature, &signature) < FRESH_BELOW && !signature.is_empty() {
                let archived = store::archive(&store_path)?;
                notice = Some(format!(
                    "The changes are very different from your last review here (only {common} of {} changed lines in common), so this is a fresh review.{}",
                    common + gone + added,
                    archived.map(|p| format!(" The previous one is kept in {}.", p.display())).unwrap_or_default()
                ));
                restored = None;
            } else if gone + added > 0 {
                notice = Some(format!(
                    "Since your last session {added} changed line(s) are new and {gone} are gone. The plan, your marks on unchanged lines and the discussions are kept; new and changed lines are pending again."
                ));
            }
        }

        let resumed = restored.is_some();
        let mut state = restored.unwrap_or_else(|| ReviewState::new(root, base.clone(), commit.clone(), backend.clone()));
        let was_staging = resumed && state.stage_mode == StageMode::Index;
        state.backend = backend;
        state.base_ref = base;
        state.base_commit = commit;
        state.stage_mode = stage_mode;
        state.threads.iter_mut().for_each(|t| t.busy = false);
        let mut session = Session { state, files: Vec::new(), planning: false, chat_busy: false, summary_busy: false, notice, version: 0, undo: Vec::new(), store_path };
        if resumed {
            session.install_files(files, None);
        } else {
            session.files = files;
            session.apply_plan(Vec::new(), fallback_plan(&session.files), false);
            session.state.line_signature = signature;
        }
        if stage_mode == StageMode::Index {
            if !was_staging {
                git::unstage_all(&session.state.repo)?;
            }
            session.sync_index(None)?;
        }
        session.save()?;
        Ok(session)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            version: self.version,
            repo: self.state.repo.clone(),
            base_ref: self.state.base_ref.clone(),
            base_commit: self.state.base_commit.clone(),
            backend: self.state.backend.clone(),
            files: self.files.clone(),
            chunks: self.state.chunks.clone(),
            marks: self.state.marks.clone().into(),
            threads: self.state.threads.clone(),
            chat: self.state.chat.clone(),
            findings: self.state.findings.clone(),
            changes: self.state.changes.clone(),
            summary: self.state.summary.clone(),
            chat_busy: self.chat_busy,
            summary_busy: self.summary_busy,
            planning: self.planning,
            planned_by_ai: self.state.planned_by_ai,
            step_count: self.state.step_count,
            can_undo: !self.undo.is_empty(),
            notice: self.notice.clone(),
            last_chunk: self.state.last_chunk.clone(),
            diff_options: self.state.diff_options,
            stage_mode: self.state.stage_mode,
            instructions: crate::ai::discover_instructions(&self.state.repo, &self.state.backend.engine),
        }
    }

    pub fn touch(&mut self) {
        self.version += 1;
    }

    pub fn save(&self) -> Result<()> {
        store::save(&self.store_path, &self.state)
    }

    pub fn hunk(&self, id: &HunkId) -> Option<&Hunk> {
        self.files.iter().flat_map(|f| f.hunks.iter()).find(|h| &h.id == id)
    }

    pub fn file(&self, path: &FilePath) -> Option<&FileDiff> {
        self.files.iter().find(|f| &f.path == path)
    }

    pub fn chunk(&self, id: &ChunkId) -> Option<&Chunk> {
        self.state.chunks.iter().find(|c| &c.id == id)
    }

    fn install_files(&mut self, files: Vec<FileDiff>, follow_up: Option<FollowUp>) -> RefreshReport {
        let before: HashSet<HunkId> = self.files.iter().flat_map(|f| f.hunks.iter().map(|h| h.id.clone())).collect();
        self.state.marks = self.state.marks.remap(&files);
        reconcile(&mut self.state.chunks, &files, follow_up);
        let new_hunks: Vec<HunkId> = files.iter().flat_map(|f| f.hunks.iter()).map(|h| h.id.clone()).filter(|id| !before.contains(id)).collect();
        let after: HashSet<&HunkId> = files.iter().flat_map(|f| f.hunks.iter().map(|h| &h.id)).collect();
        let mut touched: Vec<FilePath> = self
            .files
            .iter()
            .flat_map(|f| f.hunks.iter())
            .filter(|h| !after.contains(&h.id))
            .map(|h| h.path.clone())
            .chain(files.iter().flat_map(|f| f.hunks.iter()).filter(|h| new_hunks.contains(&h.id)).map(|h| h.path.clone()))
            .collect();
        touched.sort();
        touched.dedup();
        self.state.line_signature = line_signature(&files);
        self.files = files;
        self.relocate_anchors();
        if !touched.is_empty() {
            self.undo.clear();
        }
        self.touch();
        RefreshReport { new_hunks, touched_files: touched }
    }

    /// Re-reads the working tree. `follow_up` decides where brand-new hunks land.
    pub fn refresh(&mut self, follow_up: Option<(Option<ChunkId>, String, Option<ThreadId>)>) -> Result<RefreshReport> {
        let files = load_diff(&self.state.repo, &self.state.base_commit, self.state.diff_options)?;
        if self.state.stage_mode == StageMode::Index {
            // A file that is back to its base version must not stay staged.
            for gone in self.files.iter().filter(|f| !files.iter().any(|n| n.path == f.path)) {
                git::reset_path(&self.state.repo, &gone.path)?;
            }
        }
        let follow_up = follow_up.map(|(after, title, thread)| FollowUp { id: self.state.next_chunk_id(), after, title, thread });
        let report = self.install_files(files, follow_up);
        self.sync_index(None)?;
        self.save()?;
        Ok(report)
    }

    /// Replaces the queue: `keep` chunks stay first, `planned` follow; anything left out by the
    /// planner is collected in a final chunk so no changed line is ever lost.
    pub fn apply_plan(&mut self, keep: Vec<Chunk>, planned: Vec<PlannedChunk>, by_ai: bool) {
        let files = std::mem::take(&mut self.files);
        let mut coverage = Coverage::new(files.iter().flat_map(|f| f.hunks.iter()));
        for c in &keep {
            for r in &c.hunks {
                coverage.claim(&r.id, r.part);
            }
        }
        let mut chunks = keep;
        for p in planned {
            let refs: Vec<HunkRef> = p.items.iter().flat_map(|i| coverage.claim(&i.hunk, i.part)).collect();
            if refs.is_empty() {
                continue;
            }
            let origin = if by_ai { ChunkOrigin::Ai } else { ChunkOrigin::Fallback };
            chunks.push(Chunk { id: self.state.next_chunk_id(), title: p.title, summary: p.summary, hunks: refs, origin, notes: p.notes, group: p.group });
        }
        let leftovers = coverage.leftovers();
        if !leftovers.is_empty() {
            chunks.push(Chunk {
                id: self.state.next_chunk_id(),
                title: "Remaining changes".into(),
                summary: "Lines the planner did not place anywhere.".into(),
                hunks: leftovers,
                origin: ChunkOrigin::Fallback,
                notes: Vec::new(),
                group: None,
            });
        }
        self.files = files;
        self.state.chunks = chunks;
        self.state.planned_by_ai |= by_ai;
        self.touch();
    }

    /// Throws away all marks, chunks and discussions and starts with a file-by-file queue.
    pub fn reset(&mut self) {
        let st = &self.state;
        let (stage_mode, diff_options, step_count) = (st.stage_mode, st.diff_options, st.step_count);
        self.state = ReviewState::new(st.repo.clone(), st.base_ref.clone(), st.base_commit.clone(), st.backend.clone());
        self.state.stage_mode = stage_mode;
        self.state.step_count = step_count;
        self.state.diff_options = diff_options;
        self.undo.clear();
        let _ = self.sync_index(None);
        let plan = fallback_plan(&self.files);
        self.apply_plan(Vec::new(), plan, false);
    }

    /// Splits the queue for re-planning: chunks with any progress or discussion are kept,
    /// the rest is returned as hunks to plan again.
    pub fn replan_split(&self) -> (Vec<Chunk>, Vec<HunkId>) {
        let discussed: HashSet<&ChunkId> = self.state.threads.iter().filter_map(|t| t.chunk.as_ref()).collect();
        let mut keep = Vec::new();
        let mut pending = Vec::new();
        for chunk in &self.state.chunks {
            let started = discussed.contains(&chunk.id)
                || chunk.hunks.iter().any(|r| {
                    let Some(h) = self.hunk(&r.id) else { return false };
                    r.lines(h).iter().filter_map(|l| l.review_pos()).any(|(side, line)| self.state.marks.status(&LineKey { path: h.path.clone(), side, line }).is_some())
                });
            if started {
                keep.push(chunk.clone());
            } else {
                pending.extend(chunk.hunks.iter().map(|r| r.id.clone()));
            }
        }
        (keep, pending)
    }

    /// Writes the reviewer's own edit to disk. Lines the edit introduces count as accepted:
    /// nobody needs to review what they just typed themselves.
    pub fn save_user_edit(&mut self, path: &FilePath, text: &str, after: Option<ChunkId>) -> Result<RefreshReport> {
        let abs = self.state.repo.join(path);
        self.record_user_change(std::slice::from_ref(path), after, || {
            if let Some(dir) = abs.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&abs, text).with_context(|| format!("cannot write {path}"))
        })
    }

    /// Runs a change the reviewer made themselves (an edit, a rename) to `paths`, re-reads the
    /// diff and accepts the lines the change introduced: nobody needs to review what they did.
    pub fn record_user_change(&mut self, paths: &[FilePath], after: Option<ChunkId>, change: impl FnOnce() -> Result<()>) -> Result<RefreshReport> {
        let wanted: HashSet<&FilePath> = paths.iter().collect();
        let lines = |files: &[FileDiff]| -> HashSet<(FilePath, Side, String)> {
            files.iter().filter(|f| wanted.contains(&f.path)).flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().map(move |l| (f, l))))
                .filter_map(|(f, l)| l.review_pos().map(|(side, _)| (f.path.clone(), side, l.text.clone()))).collect()
        };
        let before = lines(&self.files);
        change()?;
        let report = self.refresh(Some((after, "Your edits".into(), None)))?;
        let mine: Vec<LineKey> = self.files.iter().filter(|f| wanted.contains(&f.path)).flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().map(move |l| (f, l))))
            .filter_map(|(f, l)| l.review_pos().filter(|(side, _)| !before.contains(&(f.path.clone(), *side, l.text.clone()))).map(|(side, line)| LineKey { path: f.path.clone(), side, line }))
            .collect();
        self.set_marks(mine, Some(MarkStatus::Accepted));
        self.save()?;
        Ok(report)
    }

    /// A symbol rename by the reviewer. Lines it adds to the change are the reviewer's own edit and
    /// count as accepted. Lines that were already part of the change keep their status: accepted
    /// stays accepted with `keep_accepted`, otherwise it needs a look again; pending stays pending.
    /// `moved` lists files the rename moved (old, new).
    pub fn record_rename(&mut self, paths: &[FilePath], moved: &[(FilePath, FilePath)], keep_accepted: bool, after: Option<ChunkId>, change: impl FnOnce() -> Result<()>) -> Result<RefreshReport> {
        let wanted: HashSet<&FilePath> = paths.iter().collect();
        let before: HashMap<(FilePath, u32), (Option<MarkStatus>, String)> = self
            .files
            .iter()
            .filter(|f| wanted.contains(&f.path))
            .flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().map(move |l| (f, l))))
            .filter(|(_, l)| l.kind == crate::diff::LineKind::Added)
            .filter_map(|(f, l)| {
                let line = l.new_no?;
                let status = self.state.marks.status(&LineKey { path: f.path.clone(), side: Side::New, line });
                Some(((f.path.clone(), line), (status, l.text.clone())))
            })
            .collect();
        change()?;
        let report = self.refresh(Some((after, "Your edits".into(), None)))?;
        let previous_path = |p: &FilePath| moved.iter().find(|(_, to)| to == p).map(|(from, _)| from.clone()).unwrap_or_else(|| p.clone());
        let (mut accept, mut defer, mut reset) = (Vec::new(), Vec::new(), Vec::new());
        for f in self.files.iter().filter(|f| wanted.contains(&f.path)) {
            for l in f.hunks.iter().flat_map(|h| h.lines.iter()).filter(|l| l.kind == crate::diff::LineKind::Added) {
                let Some(line) = l.new_no else { continue };
                let key = LineKey { path: f.path.clone(), side: Side::New, line };
                match before.get(&(previous_path(&f.path), line)) {
                    Some((_, text)) if *text == l.text => {}
                    Some((Some(MarkStatus::Accepted), _)) if keep_accepted => accept.push(key),
                    Some((Some(MarkStatus::Deferred), _)) => defer.push(key),
                    Some(_) => reset.push(key),
                    None => accept.push(key),
                }
            }
        }
        self.set_marks(accept, Some(MarkStatus::Accepted));
        self.set_marks(defer, Some(MarkStatus::Deferred));
        self.set_marks(reset, None);
        self.save()?;
        Ok(report)
    }

    /// In index mode, re-stages the given files (all changed files with `None`).
    pub fn sync_index(&self, paths: Option<&HashSet<FilePath>>) -> Result<()> {
        if self.state.stage_mode != StageMode::Index {
            return Ok(());
        }
        let files: Vec<&FileDiff> = self.files.iter().filter(|f| paths.is_none_or(|p| p.contains(&f.path))).collect();
        crate::stage::sync_index(&self.state.repo, &self.state.base_commit, &files, &self.state.marks, self.state.diff_options)
    }

    /// Stages exactly the accepted lines of every changed file, whatever the stage mode.
    pub fn stage_accepted(&self) -> Result<()> {
        let files: Vec<&FileDiff> = self.files.iter().collect();
        crate::stage::sync_index(&self.state.repo, &self.state.base_commit, &files, &self.state.marks, self.state.diff_options)
    }

    /// Switches between keeping accepted lines in gicio and staging them. Turning staging on
    /// unstages everything first, so the index holds exactly what is accepted.
    pub fn set_stage_mode(&mut self, mode: StageMode) -> Result<()> {
        if mode == self.state.stage_mode {
            return Ok(());
        }
        self.state.stage_mode = mode;
        if mode == StageMode::Index {
            git::unstage_all(&self.state.repo)?;
            self.sync_index(None)?;
        }
        self.touch();
        self.save()
    }

    /// Replaces the queue with one step per file; marks are kept (they belong to lines).
    pub fn plan_by_files(&mut self) {
        let plan = fallback_plan(&self.files);
        self.apply_plan(Vec::new(), plan, false);
        self.state.planned_by_ai = false;
    }

    pub fn set_last_chunk(&mut self, chunk: Option<ChunkId>) {
        if self.state.last_chunk != chunk {
            self.state.last_chunk = chunk;
            let _ = self.save();
        }
    }

    /// Recomputes the diff with other options (e.g. ignoring whitespace).
    pub fn set_diff_options(&mut self, opts: crate::diff::DiffOptions) -> Result<()> {
        self.state.diff_options = opts;
        self.refresh(None)?;
        Ok(())
    }

    pub fn set_marks(&mut self, keys: Vec<LineKey>, status: Option<MarkStatus>) {
        let wanted: HashSet<&LineKey> = keys.iter().collect();
        let mut texts: HashMap<LineKey, String> = HashMap::new();
        for file in &self.files {
            for hunk in &file.hunks {
                for line in &hunk.lines {
                    if let Some((side, no)) = line.review_pos() {
                        let key = LineKey { path: file.path.clone(), side, line: no };
                        if wanted.contains(&key) {
                            texts.insert(key, line.text.clone());
                        }
                    }
                }
            }
        }
        let mut entry = Vec::new();
        for key in keys {
            let Some(text) = texts.remove(&key) else { continue };
            if self.state.marks.status(&key) == status {
                continue;
            }
            let prev = self.state.marks.set(key.clone(), text, status);
            entry.push((key, prev));
        }
        if !entry.is_empty() {
            let paths: HashSet<FilePath> = entry.iter().map(|(k, _)| k.path.clone()).collect();
            self.undo.push(entry);
            self.touch();
            if let Err(e) = self.sync_index(Some(&paths)) {
                eprintln!("staging failed: {e:#}");
            }
        }
    }

    pub fn undo(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else { return false };
        let paths: HashSet<FilePath> = entry.iter().map(|(k, _)| k.path.clone()).collect();
        for (key, prev) in entry.into_iter().rev() {
            self.state.marks.restore(key, prev);
        }
        self.touch();
        let _ = self.sync_index(Some(&paths));
        true
    }

    pub fn create_thread(&mut self, chunk: Option<ChunkId>, anchor: Option<Anchor>, allow_edits: bool) -> ThreadId {
        let id = self.state.next_thread_id();
        self.state.threads.push(Thread { id: id.clone(), chunk, anchor, status: ThreadStatus::Open, allow_edits, messages: Vec::new(), busy: false });
        self.touch();
        id
    }

    pub fn thread_mut(&mut self, id: &ThreadId) -> Result<&mut Thread> {
        self.version += 1;
        self.state.threads.iter_mut().find(|t| &t.id == id).context("no such thread")
    }

    pub fn thread(&self, id: &ThreadId) -> Result<&Thread> {
        self.state.threads.iter().find(|t| &t.id == id).context("no such thread")
    }

    pub fn push_message(&mut self, id: &ThreadId, msg: Message) -> Result<()> {
        self.thread_mut(id)?.messages.push(msg);
        Ok(())
    }

    pub fn ensure_not_busy(&self, id: &ThreadId) -> Result<()> {
        if self.thread(id)?.busy {
            bail!("AI is still working on this thread");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CommitMessage;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git").current_dir(dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}");
        String::from_utf8(out.stdout).unwrap()
    }

    #[test]
    fn stage_accepted_then_amend_adds_only_accepted_lines_to_the_last_commit() {
        let dir = std::env::temp_dir().join(format!("gicio-test-finish-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t"]);
        git(&dir, &["config", "user.name", "t"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "base"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn kept() {}\nfn left_out() {}\n").unwrap();

        let mut s = Session::open(&dir, GitRef::new("HEAD").unwrap(), AiBackend::default(), StageMode::Memory).unwrap();
        s.set_marks(vec![LineKey { path: FilePath::new("a.rs").unwrap(), side: Side::New, line: 2 }], Some(MarkStatus::Accepted));
        assert_eq!(git(&dir, &["diff", "--cached", "--name-only"]), "", "memory mode stages nothing by itself");

        s.stage_accepted().unwrap();
        git::amend(&s.state.repo, &CommitMessage::new("base, reviewed").unwrap()).unwrap();
        assert_eq!(git::head_message(&s.state.repo).unwrap(), "base, reviewed");
        assert_eq!(git(&dir, &["rev-list", "--count", "HEAD"]).trim(), "1");
        assert_eq!(git(&dir, &["show", "HEAD:a.rs"]), "fn a() {}\nfn kept() {}\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
