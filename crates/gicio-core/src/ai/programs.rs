//! Finding CLI binaries. A GUI app started from Finder gets a minimal PATH, so we borrow
//! the user's login-shell PATH and look in the usual install locations.

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

fn home() -> PathBuf {
    std::env::home_dir().unwrap_or_default()
}

/// `~/…` → absolute.
fn expand_home(p: &str) -> PathBuf {
    match p.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None => PathBuf::from(p),
    }
}

/// PATH as the user's login shell sets it (empty where there is no such thing, i.e. Windows).
fn login_shell_path() -> Vec<PathBuf> {
    if cfg!(windows) {
        return Vec::new();
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    crate::process::sync_command(shell)
        .args(["-lc", "printf %s \"$PATH\""])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| std::env::split_paths(&OsString::from(String::from_utf8_lossy(&o.stdout).as_ref())).collect())
        .unwrap_or_default()
}

/// Where CLIs usually get installed, in case PATH misses them.
fn usual_dirs() -> Vec<PathBuf> {
    let home = home();
    let mut dirs: Vec<PathBuf> = [".local/bin", ".claude/local", ".npm-global/bin", ".cargo/bin", ".bun/bin", ".volta/bin"].iter().map(|d| home.join(d)).collect();
    if cfg!(windows) {
        dirs.extend(std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("npm")));
    } else {
        dirs.extend(["/opt/homebrew/bin", "/home/linuxbrew/.linuxbrew/bin", "/usr/local/bin"].map(PathBuf::from));
    }
    dirs
}

pub fn login_path() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        let current: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
        let mut parts: Vec<PathBuf> = Vec::new();
        for p in login_shell_path().into_iter().chain(current).chain(usual_dirs()) {
            if !p.as_os_str().is_empty() && !parts.contains(&p) {
                parts.push(p);
            }
        }
        std::env::join_paths(parts).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
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
    let mut found: Vec<(Vec<u64>, PathBuf)> = std::fs::read_dir(expand_home(dir))
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

/// File names a program can have: on Windows `claude.exe`, or `claude.cmd` for npm shims.
fn file_names(name: &str) -> Vec<String> {
    if cfg!(windows) { ["exe", "cmd", "bat"].iter().map(|ext| format!("{name}.{ext}")).collect() } else { vec![name.to_string()] }
}

/// `GICIO_<NAME>_BIN` overrides; then the extended PATH; then CLIs bundled in desktop apps.
pub fn resolve_program(name: &str) -> Option<PathBuf> {
    let var = format!("GICIO_{}_BIN", name.to_uppercase().replace('-', "_"));
    if let Ok(p) = std::env::var(var) {
        return Some(PathBuf::from(p));
    }
    let names = file_names(name);
    std::env::split_paths(login_path())
        .flat_map(|d| names.iter().map(move |n| d.join(n)))
        .chain(BUNDLED.iter().filter(|(n, _)| *n == name).map(|(_, p)| expand_home(p)))
        .find(|p| p.is_file())
        .or_else(|| if name == "copilot" { newest_versioned("~/Library/Caches/github-copilot-sdk/cli", "copilot") } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_path_keeps_the_current_path() {
        let path: Vec<PathBuf> = std::env::split_paths(login_path()).collect();
        for p in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).filter(|p| !p.as_os_str().is_empty()) {
            assert!(path.contains(&p), "{} missing", p.display());
        }
    }

    #[test]
    fn finds_git() {
        assert!(resolve_program("git").is_some());
    }
}
