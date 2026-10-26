//! Rectangular (column) selection model and editing, like Notepad++ Alt+drag.

/// A rectangular selection in (0-based) line / character-column space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSel {
    pub anchor: (usize, usize),
    pub head: (usize, usize),
}

impl ColumnSel {
    /// Returns `(first_line, last_line, left_col, right_col)`.
    #[must_use]
    pub fn bounds(&self) -> (usize, usize, usize, usize) {
        (
            self.anchor.0.min(self.head.0),
            self.anchor.0.max(self.head.0),
            self.anchor.1.min(self.head.1),
            self.anchor.1.max(self.head.1),
        )
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        let (a, b, _, _) = self.bounds();
        b - a + 1
    }

    #[must_use]
    pub fn is_caret_only(&self) -> bool {
        let (_, _, c1, c2) = self.bounds();
        c1 == c2
    }
}

/// Splits a line into its body chars and its `\r` suffix (if any).
fn split_cr(line: &str) -> (Vec<char>, &'static str) {
    match line.strip_suffix('\r') {
        Some(body) => (body.chars().collect(), "\r"),
        None => (line.chars().collect(), ""),
    }
}

fn map_lines(content: &str, sel: &ColumnSel, mut f: impl FnMut(&mut Vec<char>)) -> String {
    let (l1, l2, _, _) = sel.bounds();
    let mut out: Vec<String> = Vec::new();
    for (i, line) in content.split('\n').enumerate() {
        if i >= l1 && i <= l2 {
            let (mut chars, cr) = split_cr(line);
            f(&mut chars);
            let mut s: String = chars.into_iter().collect();
            s.push_str(cr);
            out.push(s);
        } else {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

/// Text covered by the selection, one line per selected line.
#[must_use]
pub fn extract(content: &str, sel: &ColumnSel) -> String {
    let (l1, l2, c1, c2) = sel.bounds();
    content
        .split('\n')
        .enumerate()
        .filter(|(i, _)| *i >= l1 && *i <= l2)
        .map(|(_, line)| {
            let (chars, _) = split_cr(line);
            let end = c2.min(chars.len());
            let start = c1.min(end);
            chars[start..end].iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replaces the selected block on every line with `text` (padding short lines with spaces).
/// Returns the new content and the collapsed selection after the inserted text.
#[must_use]
pub fn insert(content: &str, sel: &ColumnSel, text: &str) -> (String, ColumnSel) {
    let (l1, l2, c1, c2) = sel.bounds();
    let ins: Vec<char> = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    let out = map_lines(content, sel, |chars| {
        if chars.len() < c1 {
            chars.resize(c1, ' ');
        }
        let end = c2.min(chars.len());
        chars.splice(c1..end, ins.iter().copied());
    });
    let col = c1 + ins.len();
    (out, ColumnSel { anchor: (l1, col), head: (l2, col) })
}

/// Backspace (`forward == false`) or Delete (`forward == true`).
#[must_use]
pub fn delete(content: &str, sel: &ColumnSel, forward: bool) -> (String, ColumnSel) {
    let (l1, l2, c1, c2) = sel.bounds();
    if c1 != c2 {
        let out = map_lines(content, sel, |chars| {
            let end = c2.min(chars.len());
            let start = c1.min(end);
            chars.drain(start..end);
        });
        return (out, ColumnSel { anchor: (l1, c1), head: (l2, c1) });
    }
    if !forward && c1 == 0 {
        return (content.to_string(), *sel);
    }
    let target = if forward { c1 } else { c1 - 1 };
    let out = map_lines(content, sel, |chars| {
        if target < chars.len() {
            chars.remove(target);
        }
    });
    let col = if forward { c1 } else { c1 - 1 };
    (out, ColumnSel { anchor: (l1, col), head: (l2, col) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(l1: usize, c1: usize, l2: usize, c2: usize) -> ColumnSel {
        ColumnSel { anchor: (l1, c1), head: (l2, c2) }
    }

    #[test]
    fn extract_block() {
        assert_eq!(extract("abcdef\nghijkl\nmn", &sel(0, 1, 2, 4)), "bcd\nhij\nn");
    }

    #[test]
    fn insert_pads_and_replaces() {
        let (out, s) = insert("abcdef\nxy", &sel(0, 2, 1, 4), "-");
        assert_eq!(out, "ab-ef\nxy  -");
        assert_eq!(s.bounds(), (0, 1, 3, 3));
    }

    #[test]
    fn caret_insert_and_backspace() {
        let (out, s) = insert("a\nb\nc", &sel(0, 1, 2, 1), "X");
        assert_eq!(out, "aX\nbX\ncX");
        let (out, _) = delete(&out, &s, false);
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn delete_block_keeps_crlf() {
        let (out, _) = delete("abcd\r\nefgh\r\n", &sel(0, 1, 1, 3), true);
        assert_eq!(out, "ad\r\neh\r\n");
    }
}
