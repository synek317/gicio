use gicio_core::review::{Session, Snapshot};
use tauri::{AppHandle, Emitter};

/// Persists the session, pushes the snapshot to the UI and returns it.
pub fn publish(app: &AppHandle, session: &Session) -> Snapshot {
    if let Err(e) = session.save() {
        eprintln!("failed to save review state: {e:#}");
    }
    let snap = session.snapshot();
    let _ = app.emit("session", &snap);
    snap
}

pub fn err(e: impl std::fmt::Display) -> String {
    format!("{e:#}")
}
