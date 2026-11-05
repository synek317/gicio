use crate::jobs::{run_ai_job, JobKind};
use crate::publish::{err, publish};
use crate::state::AppState;
use gicio_core::ai::{self, AccessMode};
use gicio_core::domain::ChangeIntent;
use gicio_core::review::{Author, Message, MessageKind, RequirementId, RequirementStatus, Snapshot};
use tauri::{AppHandle, Manager, State};

/// The reviewer's description of the task; `None` goes back to the commit messages, if any.
#[tauri::command]
pub fn set_intent(app: AppHandle, state: State<'_, AppState>, intent: Option<ChangeIntent>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.set_intent(intent).map_err(err)?;
    Ok(publish(&app, &s))
}

/// The AI compares the change with the task in the background. The result stays out if the review
/// or its task is no longer the one that was checked by then.
#[tauri::command]
pub fn check_intent(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    if s.checking_intent {
        return Err("the change is already being checked".into());
    }
    let intent = s.state.intent.clone().ok_or("describe what the change was supposed to do first")?;
    if s.files.is_empty() {
        return Err("there is no change to check".into());
    }
    let prompt = ai::build_intent_check_prompt(&s, &state.settings.skip_filter(), &state.settings.instructions());
    let signature = gicio_core::review::line_signature(&s.files);
    s.checking_intent = true;
    s.intent_error = None;
    s.touch();
    let snap = publish(&app, &s);
    let (backend, repo, base) = (s.state.backend.clone(), s.state.repo.clone(), s.state.base_ref.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let title = "Check the change against the task";
        let res = run_ai_job(&app, JobKind::IntentCheck, title, &backend, &repo, &prompt, AccessMode::ReadOnly).await;
        let state = app.state::<AppState>();
        let Ok(mut s) = state.session() else { return };
        if s.state.repo != repo || s.state.base_ref != base {
            return;
        }
        s.checking_intent = false;
        s.touch();
        match res.as_deref().map(ai::parse_intent_check) {
            Ok(Some(_)) if s.state.intent.as_ref() != Some(&intent) => s.intent_error = Some("The task changed during the check, so its result was dropped. Check again.".into()),
            Ok(Some(raw)) => {
                let found = s.apply_intent_check(raw, signature);
                let note = format!("{title}: {} requirement(s), {} change(s) beyond the task.", found.requirements, found.out_of_scope);
                s.state.chat.push(Message::new(Author::System, MessageKind::Text, note));
            }
            Ok(None) => s.intent_error = Some("The AI did not answer in the expected format. See its log in the activity panel.".into()),
            Err(e) => s.intent_error = Some(format!("The check failed: {e}")),
        }
        publish(&app, &s);
    });
    Ok(snap)
}

#[tauri::command]
pub fn set_requirement_status(app: AppHandle, state: State<'_, AppState>, ids: Vec<RequirementId>, status: RequirementStatus) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    s.set_requirement_status(&ids, status);
    Ok(publish(&app, &s))
}

/// The AI finishes the given requirements in the background; its changes land at the end of the queue.
#[tauri::command]
pub fn implement_requirements(app: AppHandle, state: State<'_, AppState>, ids: Vec<RequirementId>) -> Result<Snapshot, String> {
    let mut s = state.session()?;
    let requirements = s.requirements_by_id(&ids);
    if requirements.is_empty() {
        return Err("no requirements selected".into());
    }
    let prompt = ai::build_implement_prompt(&s, &requirements, &state.settings.instructions());
    s.set_requirement_status(&ids, RequirementStatus::Implementing);
    let snap = publish(&app, &s);
    let (backend, repo) = (s.state.backend.clone(), s.state.repo.clone());
    drop(s);
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = state.jobs.edit_lock.lock().await;
        let title = if requirements.len() == 1 { format!("Implement: {}", requirements[0].text.chars().take(60).collect::<String>()) } else { format!("Implement {} requirements", requirements.len()) };
        let before = gicio_core::git::worktree_tree(&repo).ok();
        let res = run_ai_job(&app, JobKind::Fix, &title, &backend, &repo, &prompt, AccessMode::Edit).await;
        let Ok(mut s) = state.session() else { return };
        if s.state.repo != repo {
            return;
        }
        s.set_requirement_status(&ids, if res.is_ok() { RequirementStatus::Implemented } else { RequirementStatus::Open });
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
