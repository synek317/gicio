use crate::domain::FilePath;
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// Which files a project search looks at, from "files to include / exclude" like in VS Code:
/// comma-separated globs; one without a `/` matches anywhere in the tree (`*.rs`, `tests`), and a
/// folder matches everything inside it.
#[derive(Debug)]
pub struct FileFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

fn glob_set(list: &str) -> Result<Option<GlobSet>> {
    let patterns: Vec<&str> = list.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        let p = p.trim_start_matches("./").trim_end_matches('/');
        let p = if p.contains('/') { p.trim_start_matches('/').to_string() } else { format!("**/{p}") };
        for g in [p.clone(), format!("{p}/**")] {
            b.add(GlobBuilder::new(&g).literal_separator(true).build().with_context(|| format!("invalid file pattern {p:?}"))?);
        }
    }
    Ok(Some(b.build()?))
}

impl FileFilter {
    pub fn new(include: &str, exclude: &str) -> Result<Self> {
        Ok(Self { include: glob_set(include)?, exclude: glob_set(exclude)? })
    }

    pub fn allows(&self, path: &FilePath) -> bool {
        let p = path.as_str();
        self.include.as_ref().is_none_or(|s| s.is_match(p)) && !self.exclude.as_ref().is_some_and(|s| s.is_match(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allows(include: &str, exclude: &str, path: &str) -> bool {
        FileFilter::new(include, exclude).unwrap().allows(&FilePath::new(path).unwrap())
    }

    #[test]
    fn empty_allows_everything() {
        assert!(allows("", "", "src/a.rs"));
        assert!(allows(" , ", "", "a"));
    }

    #[test]
    fn extensions_match_anywhere() {
        assert!(allows("*.rs", "", "crates/x/src/a.rs"));
        assert!(allows("*.rs, *.ts", "", "src/lib/a.ts"));
        assert!(!allows("*.rs", "", "src/a.ts"));
    }

    #[test]
    fn folders_match_their_contents() {
        assert!(allows("src", "", "src/lib/a.ts"));
        assert!(allows("lib", "", "src/lib/a.ts"));
        assert!(allows("src/lib/", "", "src/lib/a.ts"));
        assert!(!allows("src/lib", "", "lib/a.ts"));
        assert!(allows("./src-tauri/**/*.rs", "", "src-tauri/src/a.rs"));
    }

    #[test]
    fn exclude_wins() {
        assert!(!allows("*.rs", "tests", "crates/tests/a.rs"));
        assert!(!allows("", "*.lock", "Cargo.lock"));
        assert!(allows("", "*.lock", "Cargo.toml"));
    }

    #[test]
    fn invalid_glob_is_an_error() {
        assert!(FileFilter::new("a[", "").is_err());
    }
}
