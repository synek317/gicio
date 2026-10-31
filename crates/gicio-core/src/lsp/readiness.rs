use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// Whether the server has loaded the whole workspace (rust-analyzer's `quiescent` flag).
/// Until then it knows only the files we opened, so workspace-wide answers such as
/// references or renames would be incomplete.
#[derive(Clone)]
pub struct Readiness(Arc<watch::Sender<bool>>);

impl Readiness {
    pub fn new() -> Self {
        Self(Arc::new(watch::Sender::new(false)))
    }

    pub fn set(&self, loaded: bool) {
        self.0.send_replace(loaded);
    }

    pub fn is_loaded(&self) -> bool {
        *self.0.borrow()
    }

    pub async fn wait(&self, timeout: Duration) -> Result<()> {
        let mut rx = self.0.subscribe();
        match tokio::time::timeout(timeout, rx.wait_for(|loaded| *loaded)).await {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("rust-analyzer is still loading the project, try again in a moment")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wait_returns_once_loaded() {
        let r = Readiness::new();
        assert!(!r.is_loaded());
        let setter = r.clone();
        tokio::spawn(async move { setter.set(true) });
        r.wait(Duration::from_secs(1)).await.unwrap();
        assert!(r.is_loaded());
    }

    #[tokio::test]
    async fn wait_times_out_while_loading() {
        assert!(Readiness::new().wait(Duration::from_millis(10)).await.is_err());
    }
}
