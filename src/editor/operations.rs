//! Text and line editing operations matching Notepad++ Edit menu commands.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    Uppercase,
    Lowercase,
    ProperCase,
    InvertCase,
    RandomCase,
}

/// Increases line indent with tab or spaces.
#[must_use]
pub fn increase_indent(text: &str, indent_str: &str) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    let mut first = true;
    for line in text.split('\n') {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(indent_str);
        out.push_str(line.trim_end_matches('\r'));
    }
    out
}

/// Decreases line indent.
#[must_use]
pub fn decrease_indent(text: &str, indent_str: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut first = true;
    for line in text.split('\n') {
        if !first {
            out.push('\n');
        }
        first = false;
        let clean = line.trim_end_matches('\r');
        if let Some(stripped) = clean.strip_prefix(indent_str) {
            out.push_str(stripped);
        } else if let Some(stripped) = clean.strip_prefix('\t') {
            out.push_str(stripped);
        } else if clean.starts_with(' ') {
            let trimmed = clean.trim_start_matches(' ');
            let spaces_removed = (clean.len() - trimmed.len()).min(indent_str.len().max(4));
            out.push_str(&clean[spaces_removed..]);
        } else {
            out.push_str(clean);
        }
    }
    out
}

/// Duplicates the line at 1-based `line_num`.
#[must_use]
pub fn duplicate_line(text: &str, line_num: usize) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.is_empty() {
        return text.to_string();
    }
    let target = line_num.saturating_sub(1).min(lines.len() - 1);
    let mut out = Vec::with_capacity(lines.len() + 1);

    for (idx, line) in lines.iter().enumerate() {
        out.push(*line);
        if idx == target {
            out.push(*line);
        }
    }
    out.join("\n")
}

/// Removes duplicate lines (either consecutive only or globally across whole text).
#[must_use]
pub fn remove_duplicate_lines(text: &str, consecutive_only: bool) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut last_seen: Option<&str> = None;
    let mut out = Vec::new();

    for line in text.split('\n') {
        let clean = line.trim_end_matches('\r');
        if consecutive_only {
            if last_seen != Some(clean) {
                out.push(clean);
                last_seen = Some(clean);
            }
        } else if seen.insert(clean) {
            out.push(clean);
        }
    }
    out.join("\n")
}

/// Moves 1-based `line_num` up one position.
#[must_use]
pub fn move_line_up(text: &str, line_num: usize) -> String {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if line_num <= 1 || lines.len() < 2 {
        return text.to_string();
    }
    let idx = (line_num - 1).min(lines.len() - 1);
    lines.swap(idx, idx - 1);
    lines.join("\n")
}

/// Moves 1-based `line_num` down one position.
#[must_use]
pub fn move_line_down(text: &str, line_num: usize) -> String {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if line_num >= lines.len() || lines.len() < 2 {
        return text.to_string();
    }
    let idx = (line_num - 1).min(lines.len() - 2);
    lines.swap(idx, idx + 1);
    lines.join("\n")
}

/// Sorts lines with options.
#[must_use]
pub fn sort_lines(
    text: &str,
    ascending: bool,
    case_sensitive: bool,
    by_length: bool,
) -> String {
    let mut lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();

    lines.sort_by(|a, b| {
        let order = if by_length {
            a.len().cmp(&b.len()).then_with(|| a.cmp(b))
        } else if case_sensitive {
            a.cmp(b)
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        };
        if ascending {
            order
        } else {
            order.reverse()
        }
    });

    lines.join("\n")
}

/// Changes text casing.
#[must_use]
pub fn convert_case(text: &str, mode: CaseMode) -> String {
    match mode {
        CaseMode::Uppercase => text.to_uppercase(),
        CaseMode::Lowercase => text.to_lowercase(),
        CaseMode::ProperCase => {
            let mut out = String::with_capacity(text.len());
            let mut capitalize_next = true;
            for c in text.chars() {
                if c.is_alphabetic() {
                    if capitalize_next {
                        out.extend(c.to_uppercase());
                        capitalize_next = false;
                    } else {
                        out.extend(c.to_lowercase());
                    }
                } else {
                    out.push(c);
                    capitalize_next = c.is_whitespace() || c == '_' || c == '-';
                }
            }
            out
        }
        CaseMode::InvertCase => text
            .chars()
            .map(|c| {
                if c.is_uppercase() {
                    c.to_lowercase().next().unwrap_or(c)
                } else if c.is_lowercase() {
                    c.to_uppercase().next().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect(),
        CaseMode::RandomCase => {
            let mut flag = false;
            text.chars()
                .map(|c| {
                    flag = !flag;
                    if flag {
                        c.to_uppercase().next().unwrap_or(c)
                    } else {
                        c.to_lowercase().next().unwrap_or(c)
                    }
                })
                .collect()
        }
    }
}

/// Comments or uncomments lines using the given prefix (e.g. `//` or `#`).
#[must_use]
pub fn toggle_comment(text: &str, prefix: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let prefix_space = format!("{prefix} ");

    // Check if all non-empty lines already start with prefix
    let all_commented = lines.iter().all(|l| {
        let trimmed = l.trim_start();
        trimmed.is_empty() || trimmed.starts_with(prefix)
    });

    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let clean = line.trim_end_matches('\r');
        if all_commented {
            let trimmed = clean.trim_start();
            let indent = &clean[..(clean.len() - trimmed.len())];
            if let Some(rest) = trimmed.strip_prefix(&prefix_space) {
                out.push(format!("{indent}{rest}"));
            } else if let Some(rest) = trimmed.strip_prefix(prefix) {
                out.push(format!("{indent}{rest}"));
            } else {
                out.push(clean.to_string());
            }
        } else if clean.trim().is_empty() {
            out.push(clean.to_string());
        } else {
            let trimmed = clean.trim_start();
            let indent = &clean[..(clean.len() - trimmed.len())];
            out.push(format!("{indent}{prefix_space}{trimmed}"));
        }
    }
    out.join("\n")
}

/// Trims trailing whitespaces on every line.
#[must_use]
pub fn trim_trailing_spaces(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let clean = line.trim_end_matches('\r').trim_end_matches([' ', '\t']);
        out.push(clean);
    }
    out.join("\n")
}

/// Trims leading whitespaces on every line.
#[must_use]
pub fn trim_leading_spaces(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let clean = line.trim_end_matches('\r').trim_start_matches([' ', '\t']);
        out.push(clean);
    }
    out.join("\n")
}

/// Converts TABs to N spaces.
#[must_use]
pub fn tabs_to_spaces(text: &str, spaces_per_tab: usize) -> String {
    let spaces = " ".repeat(spaces_per_tab);
    text.replace('\t', &spaces)
}

/// Converts groups of N spaces to TABs.
#[must_use]
pub fn spaces_to_tabs(text: &str, spaces_per_tab: usize) -> String {
    let spaces = " ".repeat(spaces_per_tab);
    text.replace(&spaces, "\t")
}

/// Joins lines into a single line.
#[must_use]
pub fn join_lines(text: &str) -> String {
    let parts: Vec<&str> = text
        .split('\n')
        .map(|l| l.trim_end_matches('\r').trim())
        .filter(|l| !l.is_empty())
        .collect();
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_and_move_line() {
        let src = "line1\nline2\nline3";
        let dup = duplicate_line(src, 2);
        assert_eq!(dup, "line1\nline2\nline2\nline3");

        let moved_up = move_line_up(src, 2);
        assert_eq!(moved_up, "line2\nline1\nline3");

        let moved_down = move_line_down(src, 2);
        assert_eq!(moved_down, "line1\nline3\nline2");
    }

    #[test]
    fn test_sort_and_duplicates() {
        let src = "b\na\nb\nc";
        let sorted = sort_lines(src, true, true, false);
        assert_eq!(sorted, "a\nb\nb\nc");

        let dedup = remove_duplicate_lines(&sorted, false);
        assert_eq!(dedup, "a\nb\nc");
    }

    #[test]
    fn test_case_conversion() {
        let s = "hello_world";
        assert_eq!(convert_case(s, CaseMode::Uppercase), "HELLO_WORLD");
        assert_eq!(convert_case("HELLO", CaseMode::Lowercase), "hello");
        assert_eq!(convert_case("hello world", CaseMode::ProperCase), "Hello World");
    }

    #[test]
    fn test_toggle_comment() {
        let code = "let x = 1;\nlet y = 2;";
        let commented = toggle_comment(code, "//");
        assert_eq!(commented, "// let x = 1;\n// let y = 2;");
        let uncommented = toggle_comment(&commented, "//");
        assert_eq!(uncommented, code);
    }
}
