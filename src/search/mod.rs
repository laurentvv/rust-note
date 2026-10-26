//! Search & Replace engine with Regex, Match Case, and Whole Word support.

use regex::{Regex, RegexBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    pub match_case: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub search_down: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            match_case: false,
            whole_word: false,
            use_regex: false,
            search_down: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub matched_text: String,
}

/// Compiles a query into a regex according to options.
pub fn build_regex(query: &str, options: &SearchOptions) -> Result<Regex, String> {
    if query.is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let pattern = if options.use_regex {
        if options.whole_word {
            format!(r"\b(?:{query})\b")
        } else {
            query.to_string()
        }
    } else {
        let escaped = regex::escape(query);
        if options.whole_word {
            format!(r"\b{escaped}\b")
        } else {
            escaped
        }
    };

    RegexBuilder::new(&pattern)
        .case_insensitive(!options.match_case)
        .build()
        .map_err(|e| format!("Invalid regular expression: {e}"))
}

/// Finds all occurrences of the query in text.
pub fn find_all(text: &str, query: &str, options: &SearchOptions) -> Result<Vec<SearchMatch>, String> {
    let re = build_regex(query, options)?;
    let mut matches = Vec::new();

    // Map byte offsets to line numbers
    let mut line_starts = vec![0];
    for (idx, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            line_starts.push(idx + 1);
        }
    }

    for m in re.find_iter(text) {
        let start = m.start();
        let end = m.end();
        let line = match line_starts.binary_search(&start) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };

        matches.push(SearchMatch {
            start,
            end,
            line,
            matched_text: m.as_str().to_string(),
        });
    }

    if !options.search_down {
        matches.reverse();
    }

    Ok(matches)
}

/// Finds the next/previous match starting at byte offset `from` (wrapping around).
/// Returns `(start, end, wrapped)` in byte offsets.
pub fn find_from(
    text: &str,
    query: &str,
    options: &SearchOptions,
    from: usize,
    forward: bool,
) -> Result<Option<(usize, usize, bool)>, String> {
    let re = build_regex(query, options)?;
    let from = from.min(text.len());
    if forward {
        let from = (from..=text.len()).find(|i| text.is_char_boundary(*i)).unwrap_or(text.len());
        if let Some(m) = re.find_at(text, from) {
            return Ok(Some((m.start(), m.end(), false)));
        }
        return Ok(re.find(text).map(|m| (m.start(), m.end(), true)));
    }
    let mut last_before: Option<(usize, usize)> = None;
    let mut last_any: Option<(usize, usize)> = None;
    for m in re.find_iter(text) {
        last_any = Some((m.start(), m.end()));
        if m.end() <= from {
            last_before = Some((m.start(), m.end()));
        } else {
            break;
        }
    }
    Ok(match (last_before, last_any) {
        (Some((s, e)), _) => Some((s, e, false)),
        (None, Some(_)) => {
            let m = re.find_iter(text).last().map(|m| (m.start(), m.end()));
            m.map(|(s, e)| (s, e, true))
        }
        _ => None,
    })
}

/// Expands a replacement for a single match (supports `$1` groups in regex mode).
pub fn replacement_for(matched: &str, query: &str, replacement: &str, options: &SearchOptions) -> String {
    if options.use_regex {
        if let Ok(re) = build_regex(query, options) {
            return re.replace(matched, replacement).to_string();
        }
    }
    replacement.to_string()
}

/// Replaces all occurrences of query with replacement string.
pub fn replace_all(
    text: &str,
    query: &str,
    replacement: &str,
    options: &SearchOptions,
) -> Result<(String, usize), String> {
    let re = build_regex(query, options)?;
    let matches: Vec<_> = re.find_iter(text).collect();
    let count = matches.len();
    if count == 0 {
        return Ok((text.to_string(), 0));
    }

    let new_text = if options.use_regex {
        re.replace_all(text, replacement).to_string()
    } else {
        re.replace_all(text, |_caps: &regex::Captures| replacement)
            .to_string()
    };

    Ok((new_text, count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_search_and_replace() {
        let text = "rust-note is great. rust-note in rust!";
        let opts = SearchOptions {
            match_case: false,
            whole_word: false,
            use_regex: false,
            search_down: true,
        };

        let matches = find_all(text, "rust-note", &opts).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);

        let (replaced, count) = replace_all(text, "rust-note", "editor", &opts).unwrap();
        assert_eq!(count, 2);
        assert_eq!(replaced, "editor is great. editor in rust!");
    }

    #[test]
    fn test_regex_search() {
        let text = "Version 1.2.3 and Version 4.5.6";
        let opts = SearchOptions {
            match_case: true,
            whole_word: false,
            use_regex: true,
            search_down: true,
        };

        let matches = find_all(text, r"Version (\d+\.\d+\.\d+)", &opts).unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_from_wraps_and_goes_backwards() {
        let text = "a1 b a2 b a3";
        let opts = SearchOptions::default();
        let (s, _, wrapped) = find_from(text, "b", &opts, 0, true).unwrap().unwrap();
        assert_eq!((s, wrapped), (3, false));
        let (s, _, wrapped) = find_from(text, "b", &opts, 8, true).unwrap().unwrap();
        assert_eq!((s, wrapped), (8, false));
        let (s, _, wrapped) = find_from(text, "b", &opts, 11, true).unwrap().unwrap();
        assert_eq!((s, wrapped), (3, true));
        let (s, _, wrapped) = find_from(text, "b", &opts, 8, false).unwrap().unwrap();
        assert_eq!((s, wrapped), (3, false));
        let (s, _, wrapped) = find_from(text, "b", &opts, 0, false).unwrap().unwrap();
        assert_eq!((s, wrapped), (8, true));
        assert!(find_from(text, "zzz", &opts, 0, true).unwrap().is_none());
    }
}
