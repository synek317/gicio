use super::file_filter::FileFilter;
use super::line_hit::LineHit;
use super::matcher::Matcher;
use super::query::SearchQuery;
use super::results::{FileHits, SearchResults};
use crate::domain::{FilePath, RepoRoot};
use crate::git;
use anyhow::Result;

/// Matches after which the search stops.
const MAX_MATCHES: u32 = 20_000;
/// Larger files (generated data, dumps) are not searched.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// Searches the working tree: tracked files and untracked ones git does not ignore.
pub fn search_project(root: &RepoRoot, query: &SearchQuery) -> Result<SearchResults> {
    let matcher = Matcher::new(query)?;
    let filter = FileFilter::new(&query.include, &query.exclude)?;
    let mut out = SearchResults::default();
    for path in git::listed_files(root)?.into_iter().filter(|p| filter.allows(p)) {
        let Some(text) = read_text(root, &path) else { continue };
        let lines = search_text(&matcher, &text);
        if lines.is_empty() {
            continue;
        }
        out.matches += lines.iter().map(|l| l.matches).sum::<u32>();
        out.files.push(FileHits { abs_path: root.join(&path), path, lines });
        if out.matches >= MAX_MATCHES {
            out.truncated = true;
            break;
        }
    }
    Ok(out)
}

/// The file as text; None for what is gone, too large or binary.
fn read_text(root: &RepoRoot, path: &FilePath) -> Option<String> {
    let abs = root.join(path);
    if std::fs::metadata(&abs).ok()?.len() > MAX_FILE_BYTES {
        return None;
    }
    String::from_utf8(std::fs::read(abs).ok()?).ok().filter(|t| !t.contains('\0'))
}

fn search_text(matcher: &Matcher, text: &str) -> Vec<LineHit> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let found = matcher.find(line);
            (!found.is_empty()).then(|| LineHit::new(i as u32 + 1, line, &found))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::query::SearchPattern;

    #[test]
    fn numbers_lines_from_one_and_skips_lines_without_matches() {
        let query = SearchQuery { pattern: SearchPattern::new("b").unwrap(), case_sensitive: true, whole_word: false, regex: false, include: String::new(), exclude: String::new() };
        let hits = search_text(&Matcher::new(&query).unwrap(), "ab\r\nc\nbb\n");
        assert_eq!(hits.iter().map(|h| (h.line, h.matches, h.preview.as_str())).collect::<Vec<_>>(), vec![(1, 1, "ab"), (3, 2, "bb")]);
    }
}
