use super::model::{DiffLine, Hunk, LineKind};
use crate::domain::{FilePath, HunkId};
use similar::{Algorithm, ChangeTag, TextDiff};
use std::collections::HashMap;

const CONTEXT: usize = 3;

pub fn compute_hunks(path: &FilePath, old: &str, new: &str) -> Vec<Hunk> {
    let diff = TextDiff::configure().algorithm(Algorithm::Patience).diff_lines(old, new);
    let mut seen: HashMap<HunkId, usize> = HashMap::new();
    let mut hunks = Vec::new();
    for group in diff.grouped_ops(CONTEXT) {
        let mut lines = Vec::new();
        for op in &group {
            for change in diff.iter_changes(op) {
                let kind = match change.tag() {
                    ChangeTag::Equal => LineKind::Context,
                    ChangeTag::Insert => LineKind::Added,
                    ChangeTag::Delete => LineKind::Removed,
                };
                lines.push(DiffLine {
                    kind,
                    old_no: change.old_index().map(|i| i as u32 + 1),
                    new_no: change.new_index().map(|i| i as u32 + 1),
                    text: change.value().trim_end_matches(['\n', '\r']).to_string(),
                });
            }
        }
        let (first, last) = (group.first().unwrap(), group.last().unwrap());
        let old_range = first.old_range().start..last.old_range().end;
        let new_range = first.new_range().start..last.new_range().end;
        let body: String = lines.iter().filter(|l| l.kind != LineKind::Context).map(|l| format!("{:?}{}\n", l.kind, l.text)).collect();
        let base_id = HunkId::from_content(path.as_str(), &body, 0);
        let n = seen.entry(base_id).or_insert(0);
        let id = HunkId::from_content(path.as_str(), &body, *n);
        *n += 1;
        hunks.push(Hunk {
            id,
            path: path.clone(),
            old_start: old_range.start as u32 + 1,
            old_len: old_range.len() as u32,
            new_start: new_range.start as u32 + 1,
            new_len: new_range.len() as u32,
            lines,
        });
    }
    hunks
}
