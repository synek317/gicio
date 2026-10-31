use std::path::Path;
use std::process::Command;

fn npm(root: &Path, args: &[&str]) {
    let program = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .unwrap_or_else(|e| panic!("cannot run npm ({e}); install Node.js to build the gicio frontend"));
    assert!(status.success(), "`npm {}` failed", args.join(" "));
}

/// `cargo run` and `cargo install` embed the frontend in the binary, so build it first.
/// `tauri dev` (default features off) serves it live and `tauri build` builds it itself.
fn build_frontend() {
    println!("cargo:rerun-if-env-changed=TAURI_ENV_PLATFORM");
    if std::env::var_os("CARGO_FEATURE_CUSTOM_PROTOCOL").is_none() || std::env::var_os("TAURI_ENV_PLATFORM").is_some() {
        return;
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo");
    let root = Path::new(&manifest).parent().expect("src-tauri lives in the repository root");
    for p in ["src", "static", "package.json", "package-lock.json", "svelte.config.js", "vite.config.js", "tsconfig.json"] {
        println!("cargo:rerun-if-changed={}", root.join(p).display());
    }
    if !root.join("node_modules").exists() {
        npm(root, &["ci"]);
    }
    npm(root, &["run", "build"]);
}

fn main() {
    build_frontend();
    tauri_build::build()
}
