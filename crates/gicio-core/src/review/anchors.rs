//! Things attached to lines (discussions, findings, AI notes) follow their code: after the diff
//! changes they are moved to where that code is now, or marked outdated when it is gone.

use super::Session;
use crate::diff::Side;
use crate::domain::FilePath;
use std::collections::HashMap;

/// Start line (1-based) of `excerpt` in `lines`, closest to `near`; `None` if it isn't there.
pub fn locate(lines: &[String], excerpt: &str, near: u32) -> Option<u32> {
    let ex: Vec<&str> = excerpt.lines().map(str::trim).collect();
    if ex.is_empty() || ex.iter().all(|l| l.is_empty()) || ex.len() > lines.len() {
        return None;
    }
    (0..=lines.len() - ex.len())
        .filter(|&i| ex.iter().enumerate().all(|(k, e)| lines[i + k].trim() == *e))
        .map(|i| i as u32 + 1)
        .min_by_key(|start| start.abs_diff(near))
}

/// Where an attached item sits now.
enum Place {
    /// Same code, maybe on other lines.
    At(u32),
    Outdated,
    /// Nothing to compare (no remembered code).
    Unknown,
}

impl Session {
    /// The file's lines on one side: the base version (old) or the working tree (new).
    pub fn read_lines(&self, path: &FilePath, side: Side) -> Option<Vec<String>> {
        let text = match side {
            Side::New => std::fs::read_to_string(self.state.repo.join(path)).ok()?,
            Side::Old => String::from_utf8(crate::git::show_file(&self.state.repo, &self.state.base_commit, path).ok()?).ok()?,
        };
        Some(text.lines().map(str::to_string).collect())
    }

    /// The code on lines `start..=end`, remembered so attached items can follow it.
    pub fn excerpt_of(&self, path: &FilePath, side: Side, start: u32, end: u32) -> String {
        let Some(lines) = self.read_lines(path, side) else { return String::new() };
        lines.iter().skip(start.saturating_sub(1) as usize).take((end.max(start) - start + 1) as usize).cloned().collect::<Vec<_>>().join("\n")
    }

    /// Moves discussions, findings and notes to where their code is now, or marks them outdated.
    pub fn relocate_anchors(&mut self) {
        let mut cache: HashMap<(FilePath, Side), Option<Vec<String>>> = HashMap::new();
        let mut place = |s: &Session, path: &FilePath, side: Side, code: &str, near: u32| {
            if code.trim().is_empty() {
                return Place::Unknown;
            }
            let lines = cache.entry((path.clone(), side)).or_insert_with(|| s.read_lines(path, side));
            match lines.as_deref().and_then(|l| locate(l, code, near)) {
                Some(start) => Place::At(start),
                None => Place::Outdated,
            }
        };
        let mut threads = std::mem::take(&mut self.state.threads);
        for t in threads.iter_mut() {
            let Some(a) = t.anchor.as_mut() else { continue };
            match place(self, &a.path, a.side, &a.excerpt, a.start) {
                Place::At(start) => {
                    a.end = start + (a.end - a.start);
                    a.start = start;
                    a.outdated = false;
                }
                Place::Outdated => a.outdated = true,
                Place::Unknown => {}
            }
        }
        self.state.threads = threads;
        let mut findings = std::mem::take(&mut self.state.findings);
        for f in findings.iter_mut() {
            match place(self, &f.path, f.side, &f.code, f.line) {
                Place::At(start) => {
                    f.end_line = start + (f.end_line.max(f.line) - f.line);
                    f.line = start;
                    f.outdated = false;
                }
                Place::Outdated => f.outdated = true,
                Place::Unknown => {}
            }
        }
        self.state.findings = findings;
        let mut chunks = std::mem::take(&mut self.state.chunks);
        for n in chunks.iter_mut().flat_map(|c| c.notes.iter_mut()) {
            match place(self, &n.path, n.side, &n.code, n.line) {
                Place::At(line) => {
                    n.line = line;
                    n.outdated = false;
                }
                Place::Outdated => n.outdated = true,
                Place::Unknown => {}
            }
        }
        self.state.chunks = chunks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(str::to_string).collect()
    }

    #[test]
    fn finds_moved_code_closest_to_where_it_was() {
        let file = lines("a\nfn f() {\n  x\n}\nb\nfn f() {\n  x\n}\n");
        assert_eq!(locate(&file, "fn f() {\n  x", 6), Some(6));
        assert_eq!(locate(&file, "fn f() {\n  x", 1), Some(2));
    }

    #[test]
    fn indentation_changes_still_match_but_edits_do_not() {
        let file = lines("    fn f() {\n        x\n    }\n");
        assert_eq!(locate(&file, "fn f() {\n  x", 9), Some(1));
        assert_eq!(locate(&file, "fn g() {", 1), None);
        assert_eq!(locate(&file, "   ", 1), None);
    }
}
