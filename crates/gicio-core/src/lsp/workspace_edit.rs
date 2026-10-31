//! Applying an LSP `WorkspaceEdit` (what a rename returns) to files on disk.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::PathBuf;

use super::uri::from_uri;

#[derive(Debug, Clone)]
pub struct TextEdit {
    /// (line, UTF-16 column), 0-based.
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub new_text: String,
}

#[derive(Debug, Clone)]
pub enum FileOp {
    Edit { path: PathBuf, edits: Vec<TextEdit> },
    Rename { from: PathBuf, to: PathBuf },
}

impl FileOp {
    /// Files this operation touches (both names for a rename).
    pub fn paths(&self) -> Vec<&PathBuf> {
        match self {
            FileOp::Edit { path, .. } => vec![path],
            FileOp::Rename { from, to } => vec![from, to],
        }
    }

    pub fn edit_count(&self) -> usize {
        match self {
            FileOp::Edit { edits, .. } => edits.len(),
            FileOp::Rename { .. } => 1,
        }
    }
}

fn pos(v: &Value) -> Option<(u32, u32)> {
    Some((v.get("line")?.as_u64()? as u32, v.get("character")?.as_u64()? as u32))
}

fn edits(v: &Value) -> Vec<TextEdit> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| Some(TextEdit { start: pos(e.pointer("/range/start")?)?, end: pos(e.pointer("/range/end")?)?, new_text: e.get("newText")?.as_str()?.to_string() }))
        .collect()
}

/// Both shapes: `changes` (uri -> edits) and `documentChanges` (edits and file renames, in order).
pub fn parse_workspace_edit(v: &Value) -> Vec<FileOp> {
    let mut ops = Vec::new();
    if let Some(list) = v.get("documentChanges").and_then(Value::as_array) {
        for c in list {
            match c.get("kind").and_then(Value::as_str) {
                Some("rename") => {
                    if let (Some(from), Some(to)) = (c.get("oldUri").and_then(Value::as_str).and_then(from_uri), c.get("newUri").and_then(Value::as_str).and_then(from_uri)) {
                        ops.push(FileOp::Rename { from, to });
                    }
                }
                Some(_) => {}
                None => {
                    if let Some(path) = c.pointer("/textDocument/uri").and_then(Value::as_str).and_then(from_uri) {
                        ops.push(FileOp::Edit { path, edits: edits(&c["edits"]) });
                    }
                }
            }
        }
    } else if let Some(map) = v.get("changes").and_then(Value::as_object) {
        for (uri, e) in map {
            if let Some(path) = from_uri(uri) {
                ops.push(FileOp::Edit { path, edits: edits(e) });
            }
        }
    }
    ops
}

/// Byte offset of an LSP position (UTF-16 column) in `text`.
fn offset(text: &str, (line, col): (u32, u32)) -> usize {
    let mut start = 0;
    for _ in 0..line {
        match text[start..].find('\n') {
            Some(i) => start += i + 1,
            None => return text.len(),
        }
    }
    let mut units = 0u32;
    for (i, ch) in text[start..].char_indices() {
        if units >= col || ch == '\n' {
            return start + i;
        }
        units += ch.len_utf16() as u32;
    }
    text.len()
}

pub fn apply_text_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut spans: Vec<(usize, usize, &str)> = edits.iter().map(|e| (offset(text, e.start), offset(text, e.end), e.new_text.as_str())).collect();
    spans.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = text.to_string();
    for (s, e, t) in spans {
        out.replace_range(s..e.max(s), t);
    }
    out
}

/// Writes the operations to disk, in order.
pub fn apply(ops: &[FileOp]) -> Result<()> {
    for op in ops {
        match op {
            FileOp::Edit { path, edits } => {
                let text = std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
                std::fs::write(path, apply_text_edits(&text, edits))?;
            }
            FileOp::Rename { from, to } => {
                if to.exists() {
                    bail!("{} already exists", to.display());
                }
                std::fs::rename(from, to)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(start: (u32, u32), end: (u32, u32), t: &str) -> TextEdit {
        TextEdit { start, end, new_text: t.into() }
    }

    #[test]
    fn applies_several_edits_back_to_front() {
        let text = "fn foo() {}\nfn main() { foo(); foo(); }\n";
        let out = apply_text_edits(text, &[edit((0, 3), (0, 6), "bar"), edit((1, 12), (1, 15), "bar"), edit((1, 19), (1, 22), "bar")]);
        assert_eq!(out, "fn bar() {}\nfn main() { bar(); bar(); }\n");
    }

    #[test]
    fn columns_are_utf16() {
        // "ż" is one UTF-16 unit but two bytes; "😀" is two units and four bytes.
        let text = "let ż😀 = x;\n";
        assert_eq!(apply_text_edits(text, &[edit((0, 10), (0, 11), "y")]), "let ż😀 = y;\n");
    }
}
