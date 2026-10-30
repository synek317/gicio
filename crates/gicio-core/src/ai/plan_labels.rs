//! Hunk labels used in planner prompts: `H3` (whole hunk), `H3:10-25` (rows 10..=25), `H3:12`.

pub struct Label {
    /// 0-based hunk index.
    pub hunk: usize,
    /// 0-based, end-exclusive row range within the hunk.
    pub rows: Option<(usize, usize)>,
}

pub fn parse_label(raw: &str) -> Option<Label> {
    let raw = raw.trim();
    let (h, rows) = match raw.split_once(':') {
        Some((h, r)) => (h, Some(r)),
        None => (raw, None),
    };
    let hunk = h.trim().trim_start_matches(['H', 'h']).parse::<usize>().ok()?.checked_sub(1)?;
    let rows = match rows {
        None => None,
        Some(r) => {
            let (a, b) = r.split_once('-').unwrap_or((r, r));
            let (a, b) = (a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?);
            Some((a.min(b).checked_sub(1)?, a.max(b)))
        }
    };
    Some(Label { hunk, rows })
}
