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
}

impl Document {
    #[must_use]
    pub fn new_empty(title: Option<&str>) -> Self {
        let id = NEXT_DOC_ID.fetch_add(1, Ordering::Relaxed);
        let title = title
            .map(str::to_owned)
            .unwrap_or_else(|| format!("new {id}"));

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
        }
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

        Ok(Self {
            id,
            title,
            path: Some(path_ref.to_path_buf()),
            content,
            encoding: enc,
            eol,
            is_dirty: false,
            is_readonly: false,
            is_pinned: false,
            cursor_line: 1,
            cursor_col: 1,
            language,
            history: History::default(),
        })
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
            self.is_dirty = true;
        }
    }

    /// Undo operation.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.undo(self.content.clone()) {
            self.content = prev;
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
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Total lines in document.
    #[must_use]
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            1
        } else {
            self.content.lines().count().max(1)
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
}
