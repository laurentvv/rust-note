//! Rectangular (column) selection model and editing, like Notepad++ Alt+drag.
//!
//! Columns are *visual* columns: a tab is `TAB_WIDTH` columns wide, exactly as the editor draws it,
//! so a block stays a straight rectangle on screen even across lines that contain tabs.

/// Width of a tab character in visual columns (matches the editor rendering).
pub const TAB_WIDTH: usize = 4;

/// A rectangular selection in (0-based) line / visual-column space.
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

fn char_width(c: char) -> usize {
    if c == '\t' { TAB_WIDTH } else { 1 }
}

/// Converts a visual column to `(char index, padding)`: the index of the char at that column and
/// the number of columns by which the line is too short to reach it.
fn vis_to_idx(chars: &[char], vcol: usize) -> (usize, usize) {
    let mut w = 0;
    for (i, c) in chars.iter().enumerate() {
        if w >= vcol {
            return (i, 0);
        }
        let cw = char_width(*c);
        if vcol < w + cw {
            // Inside a tab: snap to the nearer edge
            return (if vcol - w >= cw / 2 { i + 1 } else { i }, 0);
        }
        w += cw;
    }
    (chars.len(), vcol.saturating_sub(w))
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
            let end = vis_to_idx(&chars, c2).0;
            let start = vis_to_idx(&chars, c1).0.min(end);
            chars[start..end].iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replaces the selected block on every line with `text` (padding short lines with spaces).
/// Returns the new content and the collapsed selection after the inserted text.
#[must_use]
pub fn insert(content: &str, sel: &ColumnSel, text: &str) -> (String, ColumnSel) {
    insert_each(content, sel, &[text])
}

/// Like [`insert`], but line `i` of the block receives `texts[i]` (or the single text for all
/// lines when only one is given). Used to paste multi-line clipboard text into a block.
#[must_use]
pub fn insert_each(content: &str, sel: &ColumnSel, texts: &[&str]) -> (String, ColumnSel) {
    let (l1, l2, c1, c2) = sel.bounds();
    let pieces: Vec<Vec<char>> = texts
        .iter()
        .map(|t| t.chars().filter(|c| *c != '\n' && *c != '\r').collect())
        .collect();
    let mut line_no = l1;
    let out = map_lines(content, sel, |chars| {
        let rel = line_no - l1;
        let ins: &[char] = if pieces.len() == 1 {
            &pieces[0]
        } else {
            pieces.get(rel).map_or(&[][..], Vec::as_slice)
        };
        let (mut start, pad) = vis_to_idx(chars, c1);
        if pad > 0 {
            chars.extend(std::iter::repeat_n(' ', pad));
            start = chars.len();
        }
        let end = vis_to_idx(chars, c2).0.max(start);
        chars.splice(start..end, ins.iter().copied());
        line_no += 1;
    });
    let width: usize = pieces
        .iter()
        .map(|p| p.iter().map(|c| char_width(*c)).sum())
        .max()
        .unwrap_or(0);
    (
        out,
        ColumnSel {
            anchor: (l1, c1 + width),
            head: (l2, c1 + width),
        },
    )
}

/// Char index of the selection head (last dragged position), clamped to its line.
#[must_use]
pub fn head_char_index(content: &str, sel: &ColumnSel) -> usize {
    let (line, col) = sel.head;
    let mut idx = 0;
    for (i, l) in content.split('\n').enumerate() {
        let (chars, _) = split_cr(l);
        if i == line {
            return idx + vis_to_idx(&chars, col).0;
        }
        idx += l.chars().count() + 1;
    }
    idx.saturating_sub(1)
}

/// Backspace (`forward == false`) or Delete (`forward == true`).
#[must_use]
pub fn delete(content: &str, sel: &ColumnSel, forward: bool) -> (String, ColumnSel) {
    let (l1, l2, c1, c2) = sel.bounds();
    if c1 != c2 {
        let out = map_lines(content, sel, |chars| {
            let end = vis_to_idx(chars, c2).0;
            let start = vis_to_idx(chars, c1).0.min(end);
            chars.drain(start..end);
        });
        return (
            out,
            ColumnSel {
                anchor: (l1, c1),
                head: (l2, c1),
            },
        );
    }
    if !forward && c1 == 0 {
        return (content.to_string(), *sel);
    }
    let mut removed_width = 1;
    let out = map_lines(content, sel, |chars| {
        let (idx, pad) = vis_to_idx(chars, c1);
        if pad > 0 {
            return;
        }
        let target = if forward {
            Some(idx)
        } else {
            idx.checked_sub(1)
        };
        if let Some(t) = target.filter(|t| *t < chars.len()) {
            removed_width = char_width(chars[t]);
            chars.remove(t);
        }
    });
    let col = if forward {
        c1
    } else {
        c1.saturating_sub(removed_width)
    };
    (
        out,
        ColumnSel {
            anchor: (l1, col),
            head: (l2, col),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(l1: usize, c1: usize, l2: usize, c2: usize) -> ColumnSel {
        ColumnSel {
            anchor: (l1, c1),
            head: (l2, c2),
        }
    }

    #[test]
    fn extract_block() {
        assert_eq!(
            extract("abcdef\nghijkl\nmn", &sel(0, 1, 2, 4)),
            "bcd\nhij\nn"
        );
    }

    #[test]
    fn insert_pads_and_replaces() {
        let (out, s) = insert("abcdef\nxy", &sel(0, 2, 1, 4), "-");
        assert_eq!(out, "ab-ef\nxy-");
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
    fn paste_distributes_lines() {
        let (out, _) = insert_each("aa\nbb\ncc", &sel(0, 1, 2, 1), &["X", "Y"]);
        assert_eq!(out, "aXa\nbYb\ncc");
    }

    #[test]
    fn delete_block_keeps_crlf() {
        let (out, _) = delete("abcd\r\nefgh\r\n", &sel(0, 1, 1, 3), true);
        assert_eq!(out, "ad\r\neh\r\n");
    }

    #[test]
    fn tabs_are_four_visual_columns() {
        // "\tabcd" and "    abcd" look identical: a block at columns 4..6 covers "ab" on both
        let text = "\tabcd\n    abcd";
        assert_eq!(extract(text, &sel(0, 4, 1, 6)), "ab\nab");
        let (out, _) = delete(text, &sel(0, 4, 1, 6), true);
        assert_eq!(out, "\tcd\n    cd");
        let (out, s) = insert(text, &sel(0, 4, 1, 4), "X");
        assert_eq!(out, "\tXabcd\n    Xabcd");
        assert_eq!(head_char_index(&out, &sel(1, 5, 0, 5)), 2);
        assert_eq!(s.bounds().2, 5);
    }
}
