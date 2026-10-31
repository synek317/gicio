use gicio_core::lsp::RustAnalyzer;
use gicio_core::review::Session;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Default)]
pub struct AppState {
    session: Mutex<Option<Session>>,
    pub lsp: Mutex<LspSlot>,
    pub jobs: crate::jobs::Jobs,
    pub settings: crate::settings::SettingsStore,
    /// Bumped per opened review, so only the newest working-tree watcher keeps running.
    pub watch_generation: std::sync::atomic::AtomicU64,
}

#[derive(Default)]
pub enum LspSlot {
    #[default]
    Off,
    Starting,
    Running(Arc<RustAnalyzer>),
    Failed(String),
}

pub struct SessionGuard<'a>(MutexGuard<'a, Option<Session>>);

impl std::ops::Deref for SessionGuard<'_> {
    type Target = Session;
    fn deref(&self) -> &Session {
        self.0.as_ref().expect("checked in AppState::session")
    }
}

impl std::ops::DerefMut for SessionGuard<'_> {
    fn deref_mut(&mut self) -> &mut Session {
        self.0.as_mut().expect("checked in AppState::session")
    }
}

impl AppState {
    pub fn session(&self) -> Result<SessionGuard<'_>, String> {
        let guard = self.session.lock().unwrap();
        if guard.is_none() {
            return Err("no review is open".into());
        }
        Ok(SessionGuard(guard))
    }

    pub fn replace_session(&self, s: Session) {
        *self.session.lock().unwrap() = Some(s);
    }

    pub fn lsp(&self) -> Option<Arc<RustAnalyzer>> {
        match &*self.lsp.lock().unwrap() {
            LspSlot::Running(ra) => Some(ra.clone()),
            _ => None,
        }
    }
}
