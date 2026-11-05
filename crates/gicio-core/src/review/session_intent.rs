//! The task the change was supposed to do, and the result of checking the change against it.

use super::{line_signature, CodeRef, FindingSource, FindingStatus, Intent, IntentCheck, IntentSource, Requirement, RequirementId, RequirementStatus, Session, Verdict};
use crate::ai::RawIntentCheck;
use crate::domain::{ChangeIntent, FilePath};
use crate::git;
use anyhow::Result;
use std::collections::HashMap;

/// What a check of the change against the task found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentCheckCounts {
    pub requirements: usize,
    pub out_of_scope: usize,
}

impl Session {
    /// The reviewer's description of the task; `None` goes back to the commit messages, if any.
    pub fn set_intent(&mut self, text: Option<ChangeIntent>) -> Result<()> {
        let intent = match text {
            Some(text) => Some(Intent::from_reviewer(text)),
            None => self.intent_from_commits(),
        };
        self.install_intent(intent);
        self.save()
    }

    /// Commit messages may have changed since the last session; the reviewer's own text stays.
    pub(super) fn sync_commit_intent(&mut self) {
        if self.state.intent.as_ref().is_none_or(|i| i.source == IntentSource::Commits) {
            let intent = self.intent_from_commits();
            self.install_intent(intent);
        }
    }

    fn intent_from_commits(&self) -> Option<Intent> {
        let messages = git::commit_messages(&self.state.repo, &self.state.base_commit).unwrap_or_default();
        Intent::from_commits(&messages)
    }

    /// Whatever was found by comparing the change with the old text means nothing for a new one.
    fn install_intent(&mut self, intent: Option<Intent>) {
        let same_text = self.state.intent.as_ref().map(|i| &i.text) == intent.as_ref().map(|i| &i.text);
        if !same_text {
            self.state.intent_check = None;
            self.state.findings.retain(|f| f.source != FindingSource::Intent || f.status == FindingStatus::Fixing);
            self.intent_error = None;
        }
        self.state.intent = intent;
        self.touch();
    }

    /// Whether the code changed since the last check.
    pub fn intent_stale(&self) -> bool {
        self.state.intent_check.as_ref().is_some_and(|c| c.signature != line_signature(&self.files))
    }

    /// Stores the result of a check of the change with fingerprint `signature`. Requirements with
    /// the same text are the same ones: they keep their id and, when dismissed, stay dismissed.
    pub fn apply_intent_check(&mut self, raw: RawIntentCheck, signature: Vec<u64>) -> IntentCheckCounts {
        let mut previous: HashMap<String, Requirement> = self.state.intent_check.take().into_iter().flat_map(|c| c.requirements).map(|r| (r.text.clone(), r)).collect();
        let mut requirements = Vec::new();
        for r in raw.requirements {
            let text = r.text.trim().to_string();
            if text.is_empty() || requirements.iter().any(|q: &Requirement| q.text == text) {
                continue;
            }
            let (id, status) = match previous.remove(&text) {
                Some(p) => {
                    let status = if matches!(p.status, RequirementStatus::Dismissed | RequirementStatus::Implementing) { p.status } else { RequirementStatus::Open };
                    (p.id, status)
                }
                None => (self.state.next_requirement_id(), RequirementStatus::Open),
            };
            let evidence = r.evidence.into_iter().filter_map(|e| Some(CodeRef { path: FilePath::new(e.path.trim_start_matches("./")).ok()?, line: e.line.max(1) })).collect();
            requirements.push(Requirement { id, text, verdict: Verdict::from_answer(&r.verdict), note: r.note.trim().to_string(), evidence, status });
        }
        let counts = requirements.len();
        self.state.intent_check = Some(IntentCheck { requirements, signature });
        self.state.findings.retain(|f| f.source != FindingSource::Intent || f.status != FindingStatus::Open);
        let out_of_scope = raw.out_of_scope.into_iter().filter(|f| self.add_raw_finding(FindingSource::Intent, f)).count();
        self.touch();
        IntentCheckCounts { requirements: counts, out_of_scope }
    }

    pub fn requirements(&self) -> &[Requirement] {
        self.state.intent_check.as_ref().map_or(&[], |c| &c.requirements)
    }

    pub fn requirements_by_id(&self, ids: &[RequirementId]) -> Vec<Requirement> {
        self.requirements().iter().filter(|r| ids.contains(&r.id)).cloned().collect()
    }

    pub fn set_requirement_status(&mut self, ids: &[RequirementId], status: RequirementStatus) {
        if let Some(check) = self.state.intent_check.as_mut() {
            for r in check.requirements.iter_mut().filter(|r| ids.contains(&r.id)) {
                r.status = status;
            }
        }
        self.touch();
    }

    /// No job survives the app: whatever was being implemented when it closed is open again.
    pub(super) fn reopen_interrupted_requirements(&mut self) {
        if let Some(check) = self.state.intent_check.as_mut() {
            for r in check.requirements.iter_mut().filter(|r| r.status == RequirementStatus::Implementing) {
                r.status = RequirementStatus::Open;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{parse_intent_check, AiBackend};
    use crate::domain::GitRef;
    use crate::stage::StageMode;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git").current_dir(dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}");
    }

    /// A repository with a base commit, tagged `base`, and one commit on top of it.
    fn repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gicio-test-intent-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t"]);
        git(&dir, &["config", "user.name", "t"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "base"]);
        git(&dir, &["tag", "base"]);
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn login() {}\nfn extra() {}\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "Add login"]);
        dir
    }

    fn open(dir: &Path, base: &str) -> Session {
        Session::open(dir, GitRef::new(base).unwrap(), AiBackend::default(), StageMode::Memory).unwrap()
    }

    fn intent(text: &str) -> Option<ChangeIntent> {
        Some(ChangeIntent::new(text).unwrap())
    }

    const ANSWER: &str = r#"{"requirements":[{"text":"Add login","verdict":"done","note":"ok","evidence":[{"path":"./a.rs","line":2},{"path":"../outside","line":1}]},{"text":"Add logout","verdict":"missing","note":"no"}],"outOfScope":[{"path":"a.rs","line":3,"title":"Unrelated fn","detail":"d"},{"path":"not-changed.rs","line":1,"title":"Ignored"}]}"#;

    fn apply(s: &mut Session, answer: &str) -> IntentCheckCounts {
        let signature = line_signature(&s.files);
        s.apply_intent_check(parse_intent_check(answer).unwrap(), signature)
    }

    #[test]
    fn commit_messages_are_the_description_until_the_reviewer_writes_one() {
        let dir = repo("source");
        let mut s = open(&dir, "base");
        let i = s.state.intent.clone().unwrap();
        assert_eq!((i.text.as_str(), i.source), ("Add login", IntentSource::Commits));

        s.set_intent(intent("My task")).unwrap();
        let s2 = open(&dir, "base");
        assert_eq!(s2.state.intent.as_ref().unwrap().text.as_str(), "My task", "survives closing the app");

        let mut s3 = s2;
        s3.set_intent(None).unwrap();
        assert_eq!(s3.state.intent.unwrap().source, IntentSource::Commits, "clearing it goes back to the commits");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn new_commits_update_a_description_taken_from_commits_but_not_the_reviewers() {
        let dir = repo("commits");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn login() {}\nfn logout() {}\n").unwrap();
        git(&dir, &["commit", "-qam", "Add logout"]);
        let mut s = open(&dir, "base");
        assert_eq!(s.state.intent.as_ref().unwrap().text.as_str(), "Add login\n\nAdd logout");
        assert!(s.state.intent_check.is_none(), "the check was for the old description");

        s.set_intent(intent("My task")).unwrap();
        git(&dir, &["commit", "-q", "--allow-empty", "-m", "More"]);
        assert_eq!(open(&dir, "base").state.intent.unwrap().text.as_str(), "My task");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn uncommitted_review_has_no_description_by_default() {
        let dir = repo("uncommitted");
        std::fs::write(dir.join("a.rs"), "changed\n").unwrap();
        assert!(open(&dir, "HEAD").state.intent.is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_check_stores_requirements_and_findings_for_changed_files_only() {
        let dir = repo("apply");
        let mut s = open(&dir, "base");
        assert_eq!(apply(&mut s, ANSWER), IntentCheckCounts { requirements: 2, out_of_scope: 1 });
        let r = s.requirements();
        assert_eq!((r[0].verdict, r[1].verdict), (Verdict::Done, Verdict::Missing));
        assert_eq!(r[0].evidence, vec![CodeRef { path: FilePath::new("a.rs").unwrap(), line: 2 }], "paths outside the repository are dropped");
        let f = &s.state.findings[0];
        assert_eq!((f.source, f.line, f.code.as_str()), (FindingSource::Intent, 3, "fn extra() {}"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_new_check_keeps_dismissals_and_replaces_open_findings() {
        let dir = repo("recheck");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        let (kept, logout) = (s.requirements()[0].id.clone(), s.requirements()[1].id.clone());
        s.set_requirement_status(std::slice::from_ref(&logout), RequirementStatus::Dismissed);
        s.set_requirement_status(std::slice::from_ref(&kept), RequirementStatus::Implemented);
        apply(&mut s, ANSWER);
        let r = s.requirements();
        assert_eq!((&r[0].id, r[0].status), (&kept, RequirementStatus::Open), "a new verdict replaces the job's claim");
        assert_eq!((&r[1].id, r[1].status), (&logout, RequirementStatus::Dismissed));
        assert_eq!(s.state.findings.len(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn changing_the_description_forgets_what_was_found_with_the_old_one() {
        let dir = repo("forget");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        s.set_intent(intent("Something else")).unwrap();
        assert!(s.state.intent_check.is_none());
        assert!(s.state.findings.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn writing_the_same_text_keeps_the_result() {
        let dir = repo("same");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        s.set_intent(intent("Add login")).unwrap();
        assert!(s.state.intent_check.is_some());
        assert_eq!(s.state.intent.as_ref().unwrap().source, IntentSource::Reviewer);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_result_is_stale_once_the_code_changes() {
        let dir = repo("stale");
        let mut s = open(&dir, "base");
        assert!(!s.intent_stale(), "nothing checked yet");
        apply(&mut s, ANSWER);
        assert!(!s.intent_stale());
        std::fs::write(dir.join("a.rs"), "fn a() {}\nfn login() { todo!() }\n").unwrap();
        s.refresh(None).unwrap();
        assert!(s.intent_stale());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn starting_over_keeps_the_description_but_not_the_result() {
        let dir = repo("reset");
        let mut s = open(&dir, "base");
        s.set_intent(intent("My task")).unwrap();
        apply(&mut s, ANSWER);
        s.reset();
        assert_eq!(s.state.intent.as_ref().unwrap().text.as_str(), "My task");
        assert!(s.state.intent_check.is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prompts_about_the_change_say_what_it_was_supposed_to_do() {
        let dir = repo("prompts");
        let mut s = open(&dir, "base");
        s.set_intent(intent("Add a login form")).unwrap();
        let extra = Default::default();
        let once = |prompt: String| assert_eq!(prompt.matches("Add a login form").count(), 1, "{prompt}");
        once(crate::ai::build_chat_prompt(&s, &extra, false));
        once(crate::ai::build_summary_prompt(&s, &extra, &Default::default()));
        once(crate::ai::build_intent_check_prompt(&s, &Default::default(), &extra));
        let refs: Vec<_> = s.files.iter().flat_map(|f| f.hunks.iter().map(super::super::HunkRef::of)).collect();
        once(crate::ai::build_ai_review_prompt(&s, &refs, None, &extra));
        let chunk = s.state.chunks[0].id.clone();
        for t in [s.create_thread(None, None, false), s.create_thread(Some(chunk), None, false)] {
            once(crate::ai::build_ask_prompt(&s, &t, &extra).unwrap());
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_new_check_is_told_what_the_last_one_found_and_the_reviewer_accepted() {
        let dir = repo("earlier");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        let ids: Vec<_> = s.state.findings.iter().map(|f| f.id.clone()).collect();
        s.set_finding_status(&ids, FindingStatus::Dismissed);
        let prompt = crate::ai::build_intent_check_prompt(&s, &Default::default(), &Default::default());
        assert!(prompt.contains("word for word:\n- Add login\n- Add logout\n"));
        assert!(prompt.contains("do not report them again:\n- a.rs:3 Unrelated fn\n"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn fixing_an_out_of_scope_finding_means_undoing_it() {
        let dir = repo("fix");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        let prompt = crate::ai::build_fix_prompt(&s, &s.state.findings, &Default::default());
        assert!(prompt.contains("not part of the task"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_export_shows_the_task_and_the_verdicts() {
        let dir = repo("export");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        let md = super::super::export_markdown(&s);
        assert!(md.contains("## Task\n\nAdd login\n"));
        assert!(md.contains("- **missing**: Add logout — no\n"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_review_saved_before_this_feature_still_loads() {
        let dir = repo("old-state");
        let s = open(&dir, "base");
        let mut saved = serde_json::to_value(&s.state).unwrap();
        let fields = saved.as_object_mut().unwrap();
        fields.remove("intent");
        fields.remove("intentCheck");
        let state: super::super::ReviewState = serde_json::from_value(saved).unwrap();
        assert!(state.intent.is_none() && state.intent_check.is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reopening_reopens_what_a_closed_app_was_implementing() {
        let dir = repo("interrupted");
        let mut s = open(&dir, "base");
        apply(&mut s, ANSWER);
        let id = s.requirements()[1].id.clone();
        s.set_requirement_status(std::slice::from_ref(&id), RequirementStatus::Implementing);
        s.save().unwrap();
        assert_eq!(open(&dir, "base").requirements()[1].status, RequirementStatus::Open);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
