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

/// CLIs that desktop apps ship inside their bundle (the ChatGPT app carries Codex, signed in
/// with the same account).
const BUNDLED: &[(&str, &str)] = &[
    ("codex", "/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex"),
    ("codex", "~/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex"),
];

/// Newest `<dir>/<version>/<name>` (the GitHub Copilot app downloads its CLI into a version cache).
fn newest_versioned(dir: &str, name: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut found: Vec<(Vec<u64>, PathBuf)> = std::fs::read_dir(dir.replacen('~', &home, 1))
        .ok()?
        .flatten()
        .map(|e| e.path().join(name))
        .filter(|p| p.is_file())
        .map(|p| {
            let version = p.parent().and_then(|d| d.file_name()).map(|v| v.to_string_lossy().split(['.', '-']).filter_map(|n| n.parse().ok()).collect()).unwrap_or_default();
            (version, p)
        })
        .collect();
    found.sort();
    found.pop().map(|(_, p)| p)
}

/// `GICIO_<NAME>_BIN` overrides; then the extended PATH; then CLIs bundled in desktop apps.
pub fn resolve_program(name: &str) -> Option<PathBuf> {
    let var = format!("GICIO_{}_BIN", name.to_uppercase().replace('-', "_"));
    if let Ok(p) = std::env::var(var) {
        return Some(PathBuf::from(p));
    }
    let home = std::env::var("HOME").unwrap_or_default();
    login_path()
        .split(':')
        .map(|d| PathBuf::from(d).join(name))
        .chain(BUNDLED.iter().filter(|(n, _)| *n == name).map(|(_, p)| PathBuf::from(p.replacen('~', &home, 1))))
        .find(|p| p.is_file())
        .or_else(|| if name == "copilot" { newest_versioned("~/Library/Caches/github-copilot-sdk/cli", "copilot") } else { None })
}
