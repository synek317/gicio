use serde::Serialize;
use std::ops::Range;

/// Characters kept before the first match when a long line is cut.
const LEAD_CHARS: usize = 30;
/// Characters of a line shown at most.
const MAX_CHARS: usize = 200;
const ELLIPSIS: &str = "…";

/// A line with matches, as the search panel lists it: a short preview with the matches marked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineHit {
    /// 1-based.
    pub line: u32,
    /// UTF-16 column of the first match in the whole line, where the editor puts the cursor.
    pub column: u32,
    /// UTF-16 length of the first match.
    pub length: u32,
    /// The line without its indentation, cut around the first match when it is long.
    pub preview: String,
    /// `[from, to)` UTF-16 offsets of the matches inside `preview`.
    pub ranges: Vec<(u32, u32)>,
    /// Matches on the line, including any cut off the preview.
    pub matches: u32,
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// Byte index `n` characters before `at` (or the start of `s`).
fn chars_before(s: &str, at: usize, n: usize) -> usize {
    s[..at].char_indices().rev().nth(n.saturating_sub(1)).map_or(0, |(i, _)| i)
}

/// Byte index `n` characters after `from` (or the end of `s`).
fn chars_after(s: &str, from: usize, n: usize) -> usize {
    s[from..].char_indices().nth(n).map_or(s.len(), |(i, _)| from + i)
}

impl LineHit {
    /// `matches`: byte ranges in `text`, in order and not empty.
    pub fn new(line: u32, text: &str, matches: &[Range<usize>]) -> Self {
        let first = matches.first().cloned().unwrap_or(0..0);
        let indent = text.len() - text.trim_start().len();
        let lead = indent.min(first.start);
        let start = if text[lead..first.start].chars().count() > LEAD_CHARS { chars_before(text, first.start, LEAD_CHARS) } else { lead };
        let end = chars_after(text, start, MAX_CHARS);
        let window = text[start..end].trim_end();
        let end = start + window.len();
        let prefix = if start > lead { ELLIPSIS } else { "" };
        let suffix = if end < text.trim_end().len() { ELLIPSIS } else { "" };
        let shift = utf16_len(prefix);
        let ranges = matches
            .iter()
            .filter(|r| r.start < end && r.end > start)
            .map(|r| (shift + utf16_len(&text[start..r.start.max(start)]), shift + utf16_len(&text[start..r.end.min(end)])))
            .collect();
        Self {
            line,
            column: utf16_len(&text[..first.start]),
            length: utf16_len(&text[first.clone()]),
            preview: format!("{prefix}{window}{suffix}"),
            ranges,
            matches: matches.len() as u32,
        }
    }
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;

    #[test]
    fn short_line_drops_indentation() {
        let h = LineHit::new(3, "    let x = foo(foo);  ", &[12..15, 16..19]);
        assert_eq!(h.preview, "let x = foo(foo);");
        assert_eq!(h.ranges, vec![(8, 11), (12, 15)]);
        assert_eq!((h.line, h.column, h.length, h.matches), (3, 12, 3, 2));
    }

    #[test]
    fn long_line_is_cut_around_the_first_match() {
        let text = format!("{}needle{}", "a".repeat(100), "b".repeat(300));
        let h = LineHit::new(1, &text, &[100..106]);
        assert!(h.preview.starts_with('…') && h.preview.ends_with('…'));
        assert_eq!(&h.preview[3..3 + 30], "a".repeat(30));
        assert_eq!(h.ranges, vec![(31, 37)]);
        assert_eq!(h.column, 100);
        assert_eq!(h.preview.chars().count(), 1 + MAX_CHARS + 1);
    }

    #[test]
    fn match_inside_indentation_keeps_it() {
        let h = LineHit::new(1, "  \tx", &[1..3]);
        assert_eq!(h.preview, " \tx");
        assert_eq!(h.ranges, vec![(0, 2)]);
    }

    #[test]
    fn offsets_count_utf16_units() {
        let text = "ż😀 foo";
        let h = LineHit::new(1, text, &[7..10]);
        assert_eq!(h.column, 4);
        assert_eq!(h.ranges, vec![(4, 7)]);
    }

    #[test]
    fn matches_beyond_the_preview_are_counted_but_not_marked() {
        let text = format!("x {} x", "y".repeat(400));
        let h = LineHit::new(1, &text, &[0..1, text.len() - 1..text.len()]);
        assert_eq!(h.ranges, vec![(0, 1)]);
        assert_eq!(h.matches, 2);
    }
}
