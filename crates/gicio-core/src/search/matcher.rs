use super::query::SearchQuery;
use anyhow::{Context, Result};
use regex::{Regex, RegexBuilder};
use std::ops::Range;

/// A compiled search: finds the matches of a query in one line of text.
#[derive(Debug)]
pub struct Matcher {
    re: Regex,
    whole_word: bool,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Matcher {
    pub fn new(query: &SearchQuery) -> Result<Self> {
        let raw = query.pattern.as_str();
        let source = if query.regex { raw.to_string() } else { regex::escape(raw) };
        let re = RegexBuilder::new(&source)
            .case_insensitive(!query.case_sensitive)
            .multi_line(true)
            .build()
            .with_context(|| format!("invalid regular expression {raw:?}"))?;
        Ok(Self { re, whole_word: query.whole_word })
    }

    /// Byte ranges of the non-empty matches in `line`. With whole words, a match must not touch a
    /// letter, digit or `_` on either side.
    pub fn find(&self, line: &str) -> Vec<Range<usize>> {
        let mut out = Vec::new();
        let mut at = 0;
        while at <= line.len() {
            let Some(m) = self.re.find_at(line, at) else { break };
            if m.is_empty() || (self.whole_word && !self.stands_alone(line, m.range())) {
                // Try again one character further on.
                at = m.start() + line[m.start()..].chars().next().map_or(1, char::len_utf8);
                continue;
            }
            out.push(m.range());
            at = m.end();
        }
        out
    }

    fn stands_alone(&self, line: &str, r: Range<usize>) -> bool {
        let before = line[..r.start].chars().next_back().is_some_and(is_word);
        let after = line[r.end..].chars().next().is_some_and(is_word);
        !before && !after
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::query::SearchPattern;

    fn matcher(pattern: &str, case_sensitive: bool, whole_word: bool, regex: bool) -> Matcher {
        let query = SearchQuery { pattern: SearchPattern::new(pattern).unwrap(), case_sensitive, whole_word, regex, include: String::new(), exclude: String::new() };
        Matcher::new(&query).unwrap()
    }

    #[test]
    fn plain_text_is_literal() {
        assert_eq!(matcher("a.b", true, false, false).find("axb a.b"), vec![4..7]);
    }

    #[test]
    fn case_insensitive_by_default() {
        assert_eq!(matcher("foo", false, false, false).find("Foo FOO foo"), vec![0..3, 4..7, 8..11]);
        assert_eq!(matcher("foo", true, false, false).find("Foo FOO foo"), vec![8..11]);
    }

    #[test]
    fn whole_words_skip_parts_of_words() {
        let m = matcher("aa", true, true, false);
        assert_eq!(m.find("aaa aa _aa aa_ (aa)"), vec![4..6, 16..18]);
    }

    #[test]
    fn whole_words_with_punctuation_at_the_edges() {
        assert_eq!(matcher("foo(", true, true, false).find("xfoo( foo("), vec![6..10]);
    }

    #[test]
    fn regex_and_unicode() {
        assert_eq!(matcher(r"\d+", true, false, true).find("ąb 12 ż345"), vec![4..6, 9..12]);
        assert_eq!(matcher("żółw", false, true, false).find("Żółw żółwie"), vec![0..7]);
    }

    #[test]
    fn empty_matches_are_skipped() {
        assert_eq!(matcher("x*", true, false, true).find("ab xx"), vec![3..5]);
    }

    #[test]
    fn invalid_regex_is_an_error() {
        let query = SearchQuery { pattern: SearchPattern::new("(").unwrap(), case_sensitive: true, whole_word: false, regex: true, include: String::new(), exclude: String::new() };
        assert!(Matcher::new(&query).is_err());
    }
}
