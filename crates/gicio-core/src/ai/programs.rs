//! Finding CLI binaries. A GUI app started from Finder gets a minimal PATH, so we borrow
//! the user's login-shell PATH and look in the usual install locations.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

pub fn login_path() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        let current = std::env::var("PATH").unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let login = Command::new(shell)
            .args(["-lc", "printf %s \"$PATH\""])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();
        let home = std::env::var("HOME").unwrap_or_default();
        let extra = [".local/bin", ".claude/local", ".npm-global/bin", ".cargo/bin", ".bun/bin", ".volta/bin"]
            .iter()
            .map(|d| format!("{home}/{d}"))
            .chain(["/opt/homebrew/bin".into(), "/usr/local/bin".into()]);
        let mut parts: Vec<String> = Vec::new();
        for p in login.split(':').map(str::to_string).chain(current.split(':').map(str::to_string)).chain(extra) {
            if !p.is_empty() && !parts.contains(&p) {
                parts.push(p);
            }
        }
        parts.join(":")
    })
}

/// `GICIO_<NAME>_BIN` overrides; otherwise the first match on the extended PATH.
pub fn resolve_program(name: &str) -> Option<PathBuf> {
    let var = format!("GICIO_{}_BIN", name.to_uppercase().replace('-', "_"));
    if let Ok(p) = std::env::var(var) {
        return Some(PathBuf::from(p));
    }
    login_path().split(':').map(|d| PathBuf::from(d).join(name)).find(|p| p.is_file())
}
