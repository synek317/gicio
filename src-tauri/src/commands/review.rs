use crate::jobs::{run_ai_job, JobKind};
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::diff::Side;
use gicio_core::domain::{ChunkId, FilePath, ThreadId};
use gicio_core::review::{export_markdown, Author, FindingId, FindingStatus, HunkRef, Message, MessageKind, Snapshot, Thread, ThreadStatus};
use serde::Deserialize;
use std::collections::HashSet;
use tauri::{AppHandle, Manager, State};

/// Hands every issue left for later to the AI as one task (like submitting a GitHub review).
#[tauri::command]
pub fn resolve_later(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    let later: Vec<&Thread> = s.state.threads.iter().filter(|t| t.status == ThreadStatus::Deferred && !t.busy).collect();
    if later.is_empty() {
        return Err("nothing left for later".into());
    }
    let mode = if later.iter().any(|t| t.allow_edits) { AccessMode::Edit } else { AccessMode::ReadOnly };
    let threads: Vec<ThreadId> = later.iter().map(|t| t.id.clone()).collect();
    let prompt = ai::build_review_prompt(&s, &threads, &state.settings.instructions());
    for t in s.state.threads.iter_mut().filter(|t| threads.contains(&t.id)) {
        t.busy = true;
        t.status = ThreadStatus::Open;
    }
    s.touch();
    let snap = publish(&app, &s);
    let (backend, repo) = (s.state.backend.clone(), s.state.repo.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = state.jobs.edit_lock.lock().await;
        let title = format!("Resolve {} issue(s) left for later", threads.len());
        let before = if mode == AccessMode::Edit { gicio_core::git::worktree_tree(&repo).ok() } else { None };
        let res = run_ai_job(&app, JobKind::Review, &title, &backend, &repo, &prompt, mode).await;
        let Ok(mut s) = state.session() else { return };
        let author = Author::Ai { backend: backend.label() };
        match &res {
            Ok(answer) => match ai::parse_review_replies(answer) {
                Some(replies) => {
                    for r in replies {
                        let _ = s.push_message(&ThreadId::from_raw(r.thread), Message::new(author.clone(), MessageKind::Text, r.reply.trim()));
                    }
                }
                None => {
                    let _ = s.push_message(&threads[0], Message::new(author.clone(), MessageKind::Text, answer.trim()));
                }
            },
            Err(e) => {
                let _ = s.push_message(&threads[0], Message::new(Author::System, MessageKind::Error, e.clone()));
            }
        }
        let _ = s.refresh(Some((None, title.clone(), Some(threads[0].clone()))));
        if let Some(change) = s.record_change(before, &title, &backend.label(), Some(threads[0].clone())) {
            let body = format!("Code changed in {}; added as a step at the end.", s.change_files(&change));
            let _ = s.push_message(&threads[0], Message::new(Author::System, MessageKind::CodeChanged, body).with_change(Some(change)));
        }
        for t in s.state.threads.iter_mut().filter(|t| threads.contains(&t.id)) {
            t.busy = false;
        }
        s.touch();
        publish(&app, &s);
    });
    Ok(snap)
}

#[tauri::command]
pub fn export_review(state: State<'_, AppState>) -> Result<String, String> {
    Ok(export_markdown(&*state.session()?))
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReviewScope {
    All,
    Chunk { id: ChunkId },
    Lines { path: FilePath, side: Side, start: u32, end: u32 },
}

/// The AI reviews the whole change, one step or selected lines and reports findings.
#[tauri::command]
pub fn ai_review(app: AppHandle, state: State<'_, AppState>, scope: ReviewScope, focus: Option<String>) -> Result<(), String> {
    let s = state.session()?;
    let (refs, title, focus): (Vec<HunkRef>, String, Option<String>) = match scope {
        ReviewScope::All => (s.files.iter().flat_map(|f| f.hunks.iter().map(HunkRef::of)).collect(), "AI review of the whole change".into(), focus),
        ReviewScope::Chunk { id } => {
            let c = s.chunk(&id).ok_or("no such step")?;
            (c.hunks.clone(), format!("AI review: {}", c.title), focus)
        }
        ReviewScope::Lines { path, side, start, end } => {
            let file = s.file(&path).ok_or("file not in the change")?;
            let refs = file
                .hunks
                .iter()
                .filter(|h| h.lines.iter().any(|l| matches!(if side == Side::Old { l.old_no } else { l.new_no }, Some(n) if n >= start && n <= end)))
                .map(HunkRef::of)
                .collect();
            let lines = format!("{path} lines {start}-{end}");
            (refs, format!("AI review: {lines}"), Some(focus.map(|f| format!("{f} ({lines})")).unwrap_or(format!("only {lines}"))))
        }
    };
    let skip = state.settings.skip_filter();
    let refs: Vec<HunkRef> = refs.into_iter().filter(|r| !skip.skips(&r.path)).collect();
    if refs.is_empty() {
        return Err("nothing to review (only lock or generated files, which are skipped in Settings)".into());
    }
    let paths: HashSet<FilePath> = refs.iter().map(|r| r.path.clone()).collect();
    let prompt = ai::build_ai_review_prompt(&s, &refs, focus.as_deref(), &state.settings.instructions());
    let (backend, repo) = (s.state.backend.clone(), s.state.repo.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let res = run_ai_job(&app, JobKind::AiReview, &title, &backend, &repo, &prompt, AccessMode::ReadOnly).await;
        let state = app.state::<AppState>();
        let Ok(mut s) = state.session() else { return };
        let note = match res.as_deref().map(ai::parse_findings) {
            Ok(Some(raw)) => format!("{title}: {} finding(s).", s.add_ai_findings(raw, &paths)),
            Ok(None) => format!("{title}: the AI did not return findings in the expected format."),
            Err(e) => format!("{title} failed: {e}"),
        };
        s.state.chat.push(Message::new(Author::System, MessageKind::Text, note));
        publish(&app, &s);
    });
    Ok(())
}

#[tauri::command]
pub fn set_findings_status(app: AppHandle, state: State<'_, AppState>, ids: Vec<FindingId>, status: FindingStatus) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.set_finding_status(&ids, status);
    Ok(publish(&app, &s))
}

/// The AI fixes the given findings in the background; changes land at the end of the queue.
#[tauri::command]
pub fn fix_findings(app: AppHandle, state: State<'_, AppState>, ids: Vec<FindingId>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    let findings = s.findings_by_id(&ids);
    if findings.is_empty() {
        return Err("no findings selected".into());
    }
    let prompt = ai::build_fix_prompt(&s, &findings, &state.settings.instructions());
    s.set_finding_status(&ids, FindingStatus::Fixing);
    let snap = publish(&app, &s);
    let (backend, repo) = (s.state.backend.clone(), s.state.repo.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = state.jobs.edit_lock.lock().await;
        let title = if findings.len() == 1 { format!("Fix: {}", findings[0].title) } else { format!("Fix {} findings", findings.len()) };
        let before = gicio_core::git::worktree_tree(&repo).ok();
        let res = run_ai_job(&app, JobKind::Fix, &title, &backend, &repo, &prompt, AccessMode::Edit).await;
        let Ok(mut s) = state.session() else { return };
        s.set_finding_status(&ids, if res.is_ok() { FindingStatus::Fixed } else { FindingStatus::Open });
        let body = match &res {
            Ok(a) => format!("**{title}**\n\n{}", a.trim()),
            Err(e) => format!("{title} failed: {e}"),
        };
        let _ = s.refresh(Some((None, title.clone(), None)));
        let change = s.record_change(before, &title, &backend.label(), None);
        s.state.chat.push(Message::new(Author::Ai { backend: backend.label() }, if res.is_ok() { MessageKind::Text } else { MessageKind::Error }, body).with_change(change));
        publish(&app, &s);
    });
    Ok(snap)
}
