use crate::diff::{diff_ops, split_lines, DiffOptions, Side};
use similar::DiffTag;

/// The file as it would look with only the accepted changes applied to the old version:
/// accepted removals are dropped, accepted additions are inserted, everything else stays old.
pub fn partial_content(old: &str, new: &str, opts: DiffOptions, accepted: impl Fn(Side, u32) -> bool) -> String {
    let (ol, nl) = (split_lines(old), split_lines(new));
    let mut out = String::with_capacity(old.len().max(new.len()));
    let mut push = |line: &str| {
        // A kept last line without a newline must not glue onto a following added line.
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(line);
    };
    for op in diff_ops(&ol, &nl, opts) {
        let (o, n) = (op.old_range(), op.new_range());
        match op.tag() {
            // With whitespace ignored, "equal" lines may still differ: keep the old text, the
            // whitespace change was never reviewed.
            DiffTag::Equal => o.for_each(|i| push(ol[i])),
            _ => {
                o.filter(|i| !accepted(Side::Old, *i as u32 + 1)).for_each(|i| push(ol[i]));
                n.filter(|j| accepted(Side::New, *j as u32 + 1)).for_each(|j| push(nl[j]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(old: &str, new: &str, accepted: &[(Side, u32)]) -> String {
        partial_content(old, new, DiffOptions::default(), |s, l| accepted.contains(&(s, l)))
    }

    #[test]
    fn nothing_accepted_keeps_old() {
        assert_eq!(stage("a\nb\n", "a\nx\nb\ny\n", &[]), "a\nb\n");
    }

    #[test]
    fn accepted_addition_only() {
        assert_eq!(stage("a\nb\n", "a\nx\nb\ny\n", &[(Side::New, 4)]), "a\nb\ny\n");
    }

    #[test]
    fn replacement_half_accepted() {
        // Accepting only the removal drops the old line without adding the new one.
        assert_eq!(stage("a\nold\nc\n", "a\nnew\nc\n", &[(Side::Old, 2)]), "a\nc\n");
        assert_eq!(stage("a\nold\nc\n", "a\nnew\nc\n", &[(Side::Old, 2), (Side::New, 2)]), "a\nnew\nc\n");
    }

    #[test]
    fn missing_final_newline_does_not_glue_lines() {
        assert_eq!(stage("a", "a\nb\n", &[(Side::New, 2)]), "a\nb\n");
    }
}
