use crate::jobs::{run_ai_job, JobKind};
use crate::publish::publish;
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::diff::Side;
use gicio_core::domain::{ChunkId, FilePath, ThreadId};
use gicio_core::review::{export_markdown, Author, FindingId, FindingStatus, HunkRef, Message, MessageKind, Snapshot};
use serde::Deserialize;
use std::collections::HashSet;
use tauri::{AppHandle, Manager, State};

/// Sends every pending review comment to the AI as one task (like submitting a GitHub review).
#[tauri::command]
pub fn submit_review(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    let threads: Vec<ThreadId> = s.state.threads.iter().filter(|t| t.messages.iter().any(|m| m.pending)).map(|t| t.id.clone()).collect();
    if threads.is_empty() {
        return Err("no pending review comments".into());
    }
    let prompt = ai::build_review_prompt(&s, &threads, &state.settings.instructions());
    for t in s.state.threads.iter_mut().filter(|t| threads.contains(&t.id)) {
        t.busy = true;
        t.messages.iter_mut().for_each(|m| m.pending = false);
    }
    s.touch();
    let snap = publish(&app, &s);
    let (backend, repo) = (s.state.backend.clone(), s.state.repo.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = state.jobs.todo_lock.lock().await;
        let title = format!("Review with {} comment(s)", threads.len());
        let res = run_ai_job(&app, JobKind::Review, &title, &backend, &repo, &prompt, AccessMode::Edit).await;
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
        let after = s.state.chunks.last().map(|c| c.id.clone());
        if let Ok(r) = s.refresh(Some((after, title.clone(), Some(threads[0].clone()))))
            && !r.touched_files.is_empty()
        {
            let files = r.touched_files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ");
            let _ = s.push_message(&threads[0], Message::new(Author::System, MessageKind::CodeChanged, format!("Code changed in {files}; added as a step at the end of the queue.")));
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
        let _turn = state.jobs.todo_lock.lock().await;
        let title = if findings.len() == 1 { format!("Fix: {}", findings[0].title) } else { format!("Fix {} findings", findings.len()) };
        let res = run_ai_job(&app, JobKind::Fix, &title, &backend, &repo, &prompt, AccessMode::Edit).await;
        let Ok(mut s) = state.session() else { return };
        s.set_finding_status(&ids, if res.is_ok() { FindingStatus::Fixed } else { FindingStatus::Open });
        let body = match &res {
            Ok(a) => format!("**{title}**\n\n{}", a.trim()),
            Err(e) => format!("{title} failed: {e}"),
        };
        s.state.chat.push(Message::new(Author::Ai { backend: backend.label() }, if res.is_ok() { MessageKind::Text } else { MessageKind::Error }, body));
        let after = s.state.chunks.last().map(|c| c.id.clone());
        let _ = s.refresh(Some((after, title, None)));
        publish(&app, &s);
    });
    Ok(snap)
}
