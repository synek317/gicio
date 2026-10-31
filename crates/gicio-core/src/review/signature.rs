//! A fingerprint of a change: one hash per changed line (path, side, text). Comparing the
//! fingerprints of two sessions tells "a few edits" from "a completely different change".

use crate::diff::FileDiff;

pub fn line_signature(files: &[FileDiff]) -> Vec<u64> {
    let mut out: Vec<u64> = files
        .iter()
        .flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().map(move |l| (f, l))))
        .filter_map(|(f, l)| {
            let (side, _) = l.review_pos()?;
            let mut h: u64 = 0xcbf29ce484222325;
            for b in f.path.as_str().bytes().chain([0, side as u8, 0]).chain(l.text.trim().bytes()) {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            Some(h)
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Sizes of (common, only in `old`, only in `new`) for two sorted signatures.
pub fn compare(old: &[u64], new: &[u64]) -> (usize, usize, usize) {
    let (mut i, mut j, mut common) = (0, 0, 0);
    while i < old.len() && j < new.len() {
        match old[i].cmp(&new[j]) {
            std::cmp::Ordering::Equal => {
                common += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    (common, old.len() - common, new.len() - common)
}

/// Share of changed lines the two sessions have in common (Jaccard index).
pub fn similarity(old: &[u64], new: &[u64]) -> f64 {
    let (common, gone, added) = compare(old, new);
    let all = common + gone + added;
    if all == 0 { 1.0 } else { common as f64 / all as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similarity_of_signatures() {
        assert_eq!(similarity(&[1, 2, 3, 4], &[1, 2, 3, 4]), 1.0);
        assert_eq!(similarity(&[1, 2, 3, 4], &[1, 2, 3, 5]), 3.0 / 5.0);
        assert_eq!(similarity(&[1, 2], &[3, 4]), 0.0);
        assert_eq!(compare(&[1, 2, 3], &[2, 3, 4, 5]), (2, 1, 2));
    }
}
