//! Document and buffer management.

pub mod encoding;
pub mod eol;
pub mod history;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub use encoding::Encoding;
pub use eol::EolFormat;
pub use history::History;

static NEXT_DOC_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct Document {
    pub id: usize,
    pub title: String,
    pub path: Option<PathBuf>,
    pub content: String,
    pub encoding: Encoding,
    pub eol: EolFormat,
    pub is_dirty: bool,
    pub is_readonly: bool,
    pub is_pinned: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub language: Option<crate::syntax::Language>,
    pub history: History,
    /// Cursor position as a char index (kept in sync with the editor widget).
    pub cursor_char: usize,
    /// Selection length in chars.
    pub selection_len: usize,
    /// One-shot request to move the editor cursor/selection (start, end) in chars.
    pub pending_selection: Option<(usize, usize)>,
    /// Content as of the last recorded edit, used for undo grouping.
    pub snapshot: String,
    pub last_edit: Option<std::time::Instant>,
    /// Active rectangular (Alt+drag) selection.
    pub column_sel: Option<crate::editor::column::ColumnSel>,
    pub column_dragging: bool,
}

impl Document {
    /// Fresh document skeleton with neutral defaults (single source of truth for the fields).
    fn base(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            path: None,
            content: String::new(),
            encoding: Encoding::Utf8,
            eol: EolFormat::Windows,
            is_dirty: false,
            is_readonly: false,
            is_pinned: false,
            cursor_line: 1,
            cursor_col: 1,
            language: None,
            history: History::default(),
            cursor_char: 0,
            selection_len: 0,
            pending_selection: None,
            snapshot: String::new(),
            last_edit: None,
            column_sel: None,
            column_dragging: false,
        }
    }

    #[must_use]
    pub fn new_empty(title: Option<&str>) -> Self {
        let id = NEXT_DOC_ID.fetch_add(1, Ordering::Relaxed);
        let title = title
            .map(str::to_owned)
            .unwrap_or_else(|| format!("new {id}"));
        Self::base(id, title)
    }

    /// Loads a document from disk, decoding encoding and EOL.
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref)?;
        let (content, enc) = encoding::decode(&bytes);
        let eol = eol::detect(&content);
        let language = Some(crate::syntax::Language::from_path(path_ref));

        let title = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let id = NEXT_DOC_ID.fetch_add(1, Ordering::Relaxed);

        let mut doc = Self::base(id, title);
        doc.path = Some(path_ref.to_path_buf());
        doc.content = content;
        doc.snapshot = doc.content.clone();
        doc.encoding = enc;
        doc.eol = eol;
        doc.language = language;
        Ok(doc)
    }

    /// Saves the document to its path or a designated target.
    pub fn save_to<P: AsRef<Path>>(&mut self, target_path: P) -> std::io::Result<()> {
        let path_ref = target_path.as_ref();
        let bytes = encoding::encode(&self.content, self.encoding);
        std::fs::write(path_ref, bytes)?;
        self.path = Some(path_ref.to_path_buf());
        self.title = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.title)
            .to_string();
        self.is_dirty = false;
        Ok(())
    }

    /// Saves the document using its existing path.
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(ref p) = self.path.clone() {
            self.save_to(p)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path specified for document",
            ))
        }
    }

    /// Records current content into history and updates content.
    pub fn set_content(&mut self, new_content: String) {
        if self.content != new_content {
            self.history.push(self.content.clone());
            self.content = new_content;
            self.snapshot = self.content.clone();
            self.is_dirty = true;
        }
    }

    /// Undo operation.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.undo(self.content.clone()) {
            self.content = prev;
            self.snapshot = self.content.clone();
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Redo operation.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.history.redo(self.content.clone()) {
            self.content = next;
            self.snapshot = self.content.clone();
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Called after the editor widget changed `content` (typing): groups keystrokes into undo steps.
    pub fn record_typing(&mut self) {
        let now = std::time::Instant::now();
        let expired = self
            .last_edit
            .is_none_or(|t| now.duration_since(t) > std::time::Duration::from_millis(900));
        if expired {
            self.history.push(self.snapshot.clone());
        }
        self.snapshot = self.content.clone();
        self.last_edit = Some(now);
        self.is_dirty = true;
    }

    /// Converts a 1-based line number to the char index of its first char.
    #[must_use]
    pub fn char_index_of_line(&self, line: usize) -> usize {
        let mut cur = 1;
        if line <= 1 {
            return 0;
        }
        for (i, ch) in self.content.chars().enumerate() {
            if ch == '\n' {
                cur += 1;
                if cur == line {
                    return i + 1;
                }
            }
        }
        self.char_count()
    }

    /// Converts a byte offset into a char index.
    #[must_use]
    pub fn char_index_of_byte(&self, byte: usize) -> usize {
        self.content[..byte.min(self.content.len())].chars().count()
    }

    /// Total lines in document.
    #[must_use]
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            1
        } else {
            self.content.bytes().filter(|b| *b == b'\n').count() + 1
        }
    }

    /// Total characters count.
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }

    /// Total byte length.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.content.len()
    }

    /// Returns (line, column) for a given character index (1-indexed).
    #[must_use]
    pub fn line_col_from_char_index(&self, char_idx: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.content.chars().enumerate() {
            if i >= char_idx {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_from_char_index() {
        let mut doc = Document::new_empty(None);
        doc.content = "hello\nworld\n!".to_string();

        assert_eq!(doc.line_col_from_char_index(0), (1, 1));
        assert_eq!(doc.line_col_from_char_index(4), (1, 5));
        assert_eq!(doc.line_col_from_char_index(5), (1, 6)); // '\n'
        assert_eq!(doc.line_col_from_char_index(6), (2, 1)); // 'w'
        assert_eq!(doc.line_col_from_char_index(11), (2, 6)); // '\n'
        assert_eq!(doc.line_col_from_char_index(12), (3, 1)); // '!'
    }
}
