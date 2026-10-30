//! Line diff shared by the review view and index staging, so both agree on line numbers.

use similar::{capture_diff_slices, Algorithm, DiffOp};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffOptions {
    /// Lines that differ only in whitespace count as unchanged.
    pub ignore_whitespace: bool,
}

/// Lines with their endings, so they can be joined back byte-for-byte.
pub fn split_lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

fn normalize(line: &str, opts: DiffOptions) -> Cow<'_, str> {
    if opts.ignore_whitespace {
        Cow::Owned(line.split_whitespace().collect::<Vec<_>>().join(" "))
    } else {
        Cow::Borrowed(line.trim_end_matches(['\n', '\r']))
    }
}

pub fn diff_ops(old: &[&str], new: &[&str], opts: DiffOptions) -> Vec<DiffOp> {
    let a: Vec<Cow<str>> = old.iter().map(|l| normalize(l, opts)).collect();
    let b: Vec<Cow<str>> = new.iter().map(|l| normalize(l, opts)).collect();
    capture_diff_slices(Algorithm::Patience, &a, &b)
}
