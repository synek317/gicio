use super::model::{DiffLine, Hunk, LineKind};
use super::ops::{diff_ops, split_lines, DiffOptions};
use crate::domain::{FilePath, HunkId};
use similar::{group_diff_ops, DiffTag};
use std::collections::HashMap;

const CONTEXT: usize = 3;

fn text(l: &str) -> String {
    l.trim_end_matches(['\n', '\r']).to_string()
}

pub fn compute_hunks(path: &FilePath, old: &str, new: &str, opts: DiffOptions) -> Vec<Hunk> {
    let (ol, nl) = (split_lines(old), split_lines(new));
    let mut seen: HashMap<HunkId, usize> = HashMap::new();
    let mut hunks = Vec::new();
    for group in group_diff_ops(diff_ops(&ol, &nl, opts), CONTEXT) {
        let mut lines = Vec::new();
        for op in &group {
            let (o, n) = (op.old_range(), op.new_range());
            match op.tag() {
                DiffTag::Equal => lines.extend(o.zip(n).map(|(i, j)| DiffLine { kind: LineKind::Context, old_no: Some(i as u32 + 1), new_no: Some(j as u32 + 1), text: text(nl[j]) })),
                _ => {
                    lines.extend(o.map(|i| DiffLine { kind: LineKind::Removed, old_no: Some(i as u32 + 1), new_no: None, text: text(ol[i]) }));
                    lines.extend(n.map(|j| DiffLine { kind: LineKind::Added, old_no: None, new_no: Some(j as u32 + 1), text: text(nl[j]) }));
                }
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
