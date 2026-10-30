//! Files not worth a line-by-line AI review: lock files, generated and minified code.

use crate::domain::FilePath;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

pub const DEFAULT_SKIP_PATTERNS: &[&str] = &[
    "Cargo.lock", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "bun.lockb", "poetry.lock", "uv.lock", "Pipfile.lock",
    "Gemfile.lock", "composer.lock", "go.sum", "*.min.js", "*.min.css", "*.map", "*.snap", "*.pb.go", "*_generated.*",
    "*.generated.*", "dist/**", "vendor/**", "node_modules/**",
];

/// Matches paths to leave out of AI planning, reviews and summaries.
#[derive(Default)]
pub struct SkipFilter(Option<GlobSet>);

impl SkipFilter {
    /// Patterns without a `/` match the file name anywhere in the tree.
    pub fn new(enabled: bool, patterns: &[String]) -> Self {
        if !enabled {
            return Self(None);
        }
        let mut b = GlobSetBuilder::new();
        for p in patterns.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
            let p = if p.contains('/') { p.to_string() } else { format!("**/{p}") };
            if let Ok(g) = GlobBuilder::new(&p).literal_separator(true).build() {
                b.add(g);
            }
        }
        Self(b.build().ok())
    }

    pub fn skips(&self, path: &FilePath) -> bool {
        self.0.as_ref().is_some_and(|s| s.is_match(path.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> SkipFilter {
        SkipFilter::new(true, &DEFAULT_SKIP_PATTERNS.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn matches_lock_files_anywhere() {
        let f = filter();
        assert!(f.skips(&FilePath::new("Cargo.lock").unwrap()));
        assert!(f.skips(&FilePath::new("crates/a/Cargo.lock").unwrap()));
        assert!(f.skips(&FilePath::new("web/dist/app.js").unwrap()) == false);
        assert!(f.skips(&FilePath::new("dist/app.js").unwrap()));
        assert!(!f.skips(&FilePath::new("src/lock.rs").unwrap()));
    }

    #[test]
    fn disabled_skips_nothing() {
        assert!(!SkipFilter::new(false, &["Cargo.lock".into()]).skips(&FilePath::new("Cargo.lock").unwrap()));
    }
}
