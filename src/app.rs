//! Main rust-note application state, actions and per-frame orchestration.
//!
//! The actual widget rendering lives in the `ui` submodules (`ui::menu`, `ui::toolbar`,
//! `ui::tabs`, `ui::status`, `ui::sidebar`, `ui::editor`, `ui::windows`), each extending
//! `RustNoteApp` with one `render_*` method.

use std::path::PathBuf;

use crate::buffer::Document;
use crate::cli::CliArgs;
use crate::editor::CompletionState;
use crate::editor::operations::{self, CaseMode};
use crate::i18n::{self, tr};
use crate::plugins;
use crate::search::{self, SearchOptions};
use crate::settings::Settings;
use crate::syntax::Language;
use crate::ui::dialogs::{self, FindReplaceState, FindTab, GoToLineState, AboutState, SummaryState};
use eframe::egui::{self, Context};

pub use crate::ui::themes::EditorTheme;

pub struct RustNoteApp {
    pub docs: Vec<Document>,
    pub active_index: usize,
    pub cli_args: CliArgs,

    // Dialog states
    pub find_replace: FindReplaceState,
    pub goto_line: GoToLineState,
    pub about: AboutState,
    pub summary: SummaryState,
    pub conversion_panel: plugins::ConversionPanelState,
    pub symbol_filter: String,

    // Editor settings
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub current_language: Language,
    pub theme: EditorTheme,
    pub status_notification: Option<(String, std::time::Instant)>,
    pub closed_docs: Vec<Document>,
    pub show_function_list: bool,
    pub last_title: String,
    pub force_quit: bool,
    pub settings: Settings,
    pub completion: CompletionState,
    pub show_shortcuts: bool,
    pub(crate) symbol_cache: Option<(usize, u64, Language, Vec<crate::syntax::DocumentSymbol>)>,
    saved_settings: Settings,
}

impl RustNoteApp {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, cli: CliArgs) -> Self {
        // Set visual style: crisp editor palette
        let visuals = egui::Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);

        let settings = Settings::load();
        i18n::set_lang(settings.lang);
        let mut docs = Vec::new();

        // Load files from CLI args if provided
        for file in &cli.files {
            match Document::from_file(file) {
                Ok(mut doc) => {
                    if cli.read_only {
                        doc.is_readonly = true;
                    }
                    if let Some(ref lang_str) = cli.language {
                        doc.language = Some(Language::parse_language(lang_str));
                    }
                    docs.push(doc);
                }
                Err(err) => {
                    eprintln!("Warning: Failed to open file {}: {err}", file.display());
                }
            }
        }

        // Restore the previous session when launched without files
        if docs.is_empty() && !cli.no_session {
            for file in &settings.session_files {
                if let Ok(doc) = Document::from_file(file) {
                    docs.push(doc);
                }
            }
        }

        // Default empty document if none opened or nosession
        if docs.is_empty() {
            let mut doc = Document::new_empty(Some("new 1"));
            if cli.read_only {
                doc.is_readonly = true;
            }
            if let Some(ref lang_str) = cli.language {
                doc.language = Some(Language::parse_language(lang_str));
            }
            docs.push(doc);
        }

        if let Some(doc) = docs.first_mut() {
            if let Some(l) = cli.line {
                doc.cursor_line = l;
            }
            if let Some(c) = cli.col {
                doc.cursor_col = c;
            }
            if cli.line.is_some() || cli.col.is_some() {
                let line_start = doc.char_index_of_line(doc.cursor_line.max(1));
                let pos = (line_start + doc.cursor_col.saturating_sub(1)).min(doc.char_count());
                doc.pending_selection = Some((pos, pos));
            } else if let Some(p) = cli.pos {
                doc.pending_selection = Some((p.min(doc.char_count()), p.min(doc.char_count())));
            }
        }

        let initial_lang = docs.first().and_then(|d| d.language).unwrap_or(Language::PlainText);

        let app = Self {
            docs,
            active_index: 0,
            cli_args: cli,
            find_replace: FindReplaceState::default(),
            goto_line: GoToLineState::default(),
            about: AboutState::default(),
            summary: SummaryState::default(),
            conversion_panel: plugins::ConversionPanelState::default(),
            symbol_filter: String::new(),
            font_size: settings.font_size.clamp(8.0, 40.0),
            show_line_numbers: settings.show_line_numbers,
            word_wrap: settings.word_wrap,
            current_language: initial_lang,
            theme: settings.theme,
            status_notification: None,
            closed_docs: Vec::new(),
            show_function_list: settings.show_function_list,
            last_title: String::new(),
            force_quit: false,
            completion: CompletionState::default(),
            show_shortcuts: false,
            symbol_cache: None,
            saved_settings: settings.clone(),
            settings,
        };
        app.apply_theme(&cc.egui_ctx);
        app
    }

    pub fn apply_theme(&self, ctx: &Context) {
        ctx.set_visuals(self.theme.visuals());
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        let msg: String = msg.into();
        self.status_notification = Some((tr(&msg), std::time::Instant::now()));
    }

    pub fn copy_full_path(&mut self, ctx: &Context) {
        if let Some(doc) = self.active_doc() {
            if let Some(ref path) = doc.path {
                let s = path.to_string_lossy().to_string();
                ctx.copy_text(s);
                self.set_notification("Copied full path to clipboard");
            } else {
                self.set_notification("Document has not been saved to disk yet");
            }
        }
    }

    pub fn active_doc(&self) -> Option<&Document> {
        self.docs.get(self.active_index)
    }

    pub fn active_doc_mut(&mut self) -> Option<&mut Document> {
        self.docs.get_mut(self.active_index)
    }

    // --- File Actions ---
    pub fn new_document(&mut self) {
        let count = self.docs.len() + 1;
        let title = format!("new {count}");
        self.docs.push(Document::new_empty(Some(&title)));
        self.active_index = self.docs.len() - 1;
        self.set_notification("New document created");
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open_path(path);
        }
    }

    pub fn open_path(&mut self, path: PathBuf) {
        // If already open, switch to it
        if let Some(idx) = self.docs.iter().position(|d| d.path.as_ref() == Some(&path)) {
            self.active_index = idx;
            return;
        }

        match Document::from_file(&path) {
            Ok(doc) => {
                self.settings.push_recent(&path);
                // Replace the pristine placeholder tab instead of keeping an empty one
                if self.docs.len() == 1 && self.docs[0].path.is_none() && !self.docs[0].is_dirty && self.docs[0].content.is_empty() {
                    self.docs.clear();
                }
                self.docs.push(doc);
                self.active_index = self.docs.len() - 1;
                self.set_notification(format!("Opened {}", path.display()));
            }
            Err(e) => {
                self.set_notification(format!("Error opening {}: {e}", path.display()));
            }
        }
    }

    pub fn save_current(&mut self) {
        if let Some(doc) = self.docs.get_mut(self.active_index) {
            if doc.path.is_some() {
                let title = doc.title.clone();
                let res = doc.save();
                match res {
                    Ok(()) => self.set_notification(format!("Saved {title}")),
                    Err(e) => self.set_notification(format!("Save error: {e}")),
                }
            } else {
                let _ = doc;
                self.save_as_current();
            }
        }
    }

    pub fn save_as_current(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            if let Some(doc) = self.active_doc_mut() {
                if let Err(e) = doc.save_to(&path) {
                    self.set_notification(format!("Save error: {e}"));
                } else {
                    self.set_notification(format!("Saved as {}", path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().to_string())));
                }
            }
        }
    }

    pub fn save_all(&mut self) {
        let mut count = 0;
        for doc in &mut self.docs {
            if doc.is_dirty && doc.path.is_some() {
                if doc.save().is_ok() {
                    count += 1;
                }
            }
        }
        self.set_notification(format!("Saved {count} documents"));
    }

    /// Exports the active document through a native save dialog.
    pub fn export_current_to_html(&mut self) {
        self.export_current(ExportKind::Html);
    }

    pub fn export_current_to_rtf(&mut self) {
        self.export_current(ExportKind::Rtf);
    }

    pub fn copy_filename(&mut self, ctx: &Context) {
        if let Some(doc) = self.active_doc() {
            let name = doc.title.clone();
            ctx.copy_text(name);
            self.set_notification("Copied filename to clipboard");
        }
    }

    pub fn copy_directory_path(&mut self, ctx: &Context) {
        if let Some(doc) = self.active_doc() {
            if let Some(ref path) = doc.path {
                if let Some(parent) = path.parent() {
                    let s = parent.to_string_lossy().to_string();
                    ctx.copy_text(s);
                    self.set_notification("Copied directory path to clipboard");
                }
            } else {
                self.set_notification("Document has not been saved to disk yet");
            }
        }
    }

    pub fn restore_closed_document(&mut self) {
        if let Some(doc) = self.closed_docs.pop() {
            let title = doc.title.clone();
            self.docs.push(doc);
            self.active_index = self.docs.len() - 1;
            self.set_notification(format!("Restored tab '{title}'"));
        } else {
            self.set_notification("No recently closed tabs to restore");
        }
    }

    /// Asks the user whether to save a modified document. Returns false if the close must be aborted.
    fn confirm_close(&mut self, idx: usize) -> bool {
        let Some(doc) = self.docs.get(idx) else { return true };
        if !doc.is_dirty {
            return true;
        }
        let title = doc.title.clone();
        let choice = rfd::MessageDialog::new()
            .set_title("rust-note")
            .set_description(format!("Save changes to \"{title}\"?"))
            .set_buttons(rfd::MessageButtons::YesNoCancel)
            .set_level(rfd::MessageLevel::Warning)
            .show();
        match choice {
            rfd::MessageDialogResult::Yes => {
                let prev = self.active_index;
                self.active_index = idx;
                self.save_current();
                self.active_index = prev;
                !self.docs.get(idx).is_some_and(|d| d.is_dirty)
            }
            rfd::MessageDialogResult::No => true,
            _ => false,
        }
    }

    fn remember_closed(&mut self, doc: Document) {
        if doc.path.is_some() || !doc.content.is_empty() {
            if self.closed_docs.len() >= 25 {
                self.closed_docs.remove(0);
            }
            self.closed_docs.push(doc);
        }
    }

    /// Closes the tab at `idx` (after confirmation), remembering it for restoration.
    pub fn close_at(&mut self, idx: usize) {
        if idx >= self.docs.len() || !self.confirm_close(idx) {
            return;
        }
        let doc = self.docs[idx].clone();
        self.remember_closed(doc);
        if self.docs.len() <= 1 {
            self.docs[0] = Document::new_empty(Some("new 1"));
            self.active_index = 0;
        } else {
            self.docs.remove(idx);
            if idx < self.active_index {
                self.active_index -= 1;
            }
            self.active_index = self.active_index.min(self.docs.len() - 1);
        }
        self.set_notification("Document closed");
    }

    pub fn close_current(&mut self) {
        self.close_at(self.active_index);
    }

    pub fn close_all(&mut self) {
        let mut i = 0;
        while i < self.docs.len() {
            if !self.confirm_close(i) {
                self.active_index = self.active_index.min(self.docs.len() - 1);
                return;
            }
            i += 1;
        }
        let docs = std::mem::take(&mut self.docs);
        for d in docs {
            self.remember_closed(d);
        }
        self.docs.push(Document::new_empty(Some("new 1")));
        self.active_index = 0;
        self.set_notification("All documents closed");
    }

    pub fn close_all_except(&mut self, keep: usize) {
        let Some(keep_id) = self.docs.get(keep).map(|d| d.id) else { return };
        let mut i = 0;
        while i < self.docs.len() {
            if self.docs[i].id == keep_id {
                i += 1;
                continue;
            }
            if !self.confirm_close(i) {
                return;
            }
            let doc = self.docs.remove(i);
            self.remember_closed(doc);
        }
        self.active_index = 0;
        self.set_notification("Closed other documents");
    }

    pub fn close_all_except_current(&mut self) {
        self.close_all_except(self.active_index);
    }

    /// True if any open document has unsaved changes.
    pub fn any_dirty(&self) -> bool {
        self.docs.iter().any(|d| d.is_dirty)
    }

    /// Selects a char range in the active document and scrolls the editor to it.
    pub fn select_chars(&mut self, start: usize, end: usize) {
        if let Some(doc) = self.active_doc_mut() {
            doc.pending_selection = Some((start, end));
        }
    }

    pub fn goto_line_in_active(&mut self, line: usize) {
        if let Some(doc) = self.active_doc_mut() {
            let l = line.clamp(1, doc.line_count());
            let start = doc.char_index_of_line(l);
            doc.cursor_line = l;
            doc.pending_selection = Some((start, start));
        }
    }

    /// Inserts text at the cursor (replacing the selection).
    pub fn insert_at_cursor(&mut self, text: &str) {
        if let Some(doc) = self.active_doc_mut() {
            let chars: Vec<char> = doc.content.chars().collect();
            let start = doc.cursor_char.min(chars.len());
            let end = (start + doc.selection_len).min(chars.len());
            let mut out: String = chars[..start].iter().collect();
            out.push_str(text);
            out.extend(chars[end..].iter());
            doc.set_content(out);
            let pos = start + text.chars().count();
            doc.pending_selection = Some((pos, pos));
        }
    }

    // --- Search & Replace Executions ---
    fn selection_byte_range(doc: &Document) -> (usize, usize) {
        let mut offsets = doc.content.char_indices().map(|(b, _)| b).chain(std::iter::once(doc.content.len()));
        let start = offsets.clone().nth(doc.cursor_char).unwrap_or(doc.content.len());
        let end = offsets.nth(doc.cursor_char + doc.selection_len).unwrap_or(doc.content.len());
        (start, end)
    }

    pub fn execute_find(&mut self, forward: bool) -> bool {
        let query = self.find_replace.query.clone();
        let opts = self.find_replace.options.clone();
        if query.is_empty() {
            self.find_replace.status_message = Some("Enter text to search for".into());
            return false;
        }
        let Some(doc) = self.docs.get(self.active_index) else { return false };
        let (sel_start, sel_end) = Self::selection_byte_range(doc);
        let from = if forward { sel_end } else { sel_start };
        let result = search::find_from(&doc.content, &query, &opts, from, forward);
        match result {
            Ok(Some((s, e, wrapped))) => {
                let cs = doc.char_index_of_byte(s);
                let ce = doc.char_index_of_byte(e);
                let line = doc.line_col_from_char_index(cs).0;
                self.select_chars(cs, ce);
                self.find_replace.status_message = Some(if wrapped {
                    format!("Found at line {line} (wrapped around the document)")
                } else {
                    format!("Found at line {line}")
                });
                true
            }
            Ok(None) => {
                self.find_replace.status_message = Some(format!("Can't find the text \"{query}\""));
                false
            }
            Err(e) => {
                self.find_replace.status_message = Some(e);
                false
            }
        }
    }

    pub fn execute_find_next(&mut self) {
        let forward = self.find_replace.options.search_down;
        self.execute_find(forward);
    }

    pub fn execute_count(&mut self) {
        let query = self.find_replace.query.clone();
        let opts = self.find_replace.options.clone();
        if let Some(doc) = self.active_doc() {
            self.find_replace.status_message = Some(match search::find_all(&doc.content, &query, &opts) {
                Ok(m) => format!("Count: {} match(es) in the current document", m.len()),
                Err(e) => e,
            });
        }
    }

    pub fn execute_replace_one(&mut self) {
        let query = self.find_replace.query.clone();
        let repl = self.find_replace.replacement.clone();
        let opts = self.find_replace.options.clone();
        if query.is_empty() {
            return;
        }
        // If the current selection is a match, replace it; then move to the next one.
        let mut replaced = false;
        if let Some(doc) = self.active_doc_mut() {
            if !doc.is_readonly && doc.selection_len > 0 {
                let (s, e) = Self::selection_byte_range(doc);
                if let Ok(Some((ms, me, _))) = search::find_from(&doc.content, &query, &opts, s, true) {
                    if ms == s && me == e {
                        let new_text = search::replacement_for(&doc.content[s..e], &query, &repl, &opts);
                        let mut out = doc.content.clone();
                        out.replace_range(s..e, &new_text);
                        let pos = doc.cursor_char + new_text.chars().count();
                        doc.set_content(out);
                        doc.cursor_char = pos;
                        doc.selection_len = 0;
                        doc.pending_selection = Some((pos, pos));
                        replaced = true;
                    }
                }
            }
        }
        let found = self.execute_find(true);
        if replaced {
            self.find_replace.status_message = Some(if found {
                "Replaced 1 occurrence; next match selected".into()
            } else {
                "Replaced 1 occurrence; no more matches".into()
            });
        }
    }

    pub fn execute_replace_all(&mut self) {
        let query = self.find_replace.query.clone();
        let repl = self.find_replace.replacement.clone();
        let opts = self.find_replace.options.clone();

        if let Some(doc) = self.active_doc_mut() {
            if doc.is_readonly {
                self.find_replace.status_message = Some("Document is read-only".into());
                return;
            }
            match search::replace_all(&doc.content, &query, &repl, &opts) {
                Ok((new_text, count)) => {
                    doc.set_content(new_text);
                    self.find_replace.status_message =
                        Some(format!("Replace All: {count} occurrence(s) were replaced."));
                    self.set_notification(format!("Replaced {count} occurrences"));
                }
                Err(e) => {
                    self.find_replace.status_message = Some(e);
                }
            }
        }
    }

    pub fn execute_find_all_open(&mut self) {
        let query = self.find_replace.query.clone();
        let opts = self.find_replace.options.clone();
        let mut hits = Vec::new();
        let mut error = None;
        for (idx, doc) in self.docs.iter().enumerate() {
            match search::find_all(&doc.content, &query, &SearchOptions { search_down: true, ..opts.clone() }) {
                Ok(ms) => {
                    for m in ms {
                        let line_text = doc.content.lines().nth(m.line.saturating_sub(1)).unwrap_or("").trim();
                        hits.push(dialogs::FindHit {
                            doc_index: idx,
                            doc_title: doc.title.clone(),
                            line: m.line,
                            byte_start: m.start,
                            byte_end: m.end,
                            preview: line_text.chars().take(90).collect(),
                        });
                    }
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        self.find_replace.status_message = Some(match error {
            Some(e) => e,
            None => format!("Find All: {} hit(s) in {} document(s)", hits.len(), self.docs.len()),
        });
        self.find_replace.results = hits;
    }

    pub fn jump_to_hit(&mut self, doc_index: usize, byte_start: usize, byte_end: usize) {
        if doc_index >= self.docs.len() {
            return;
        }
        self.active_index = doc_index;
        let (cs, ce) = {
            let d = &self.docs[doc_index];
            (d.char_index_of_byte(byte_start), d.char_index_of_byte(byte_end))
        };
        self.select_chars(cs, ce);
    }

    /// Opens the find dialog, pre-filling the query from the current selection.
    pub fn open_find(&mut self, tab: FindTab) {
        if let Some(doc) = self.active_doc() {
            if doc.selection_len > 0 && doc.selection_len < 200 {
                let sel: String = doc.content.chars().skip(doc.cursor_char).take(doc.selection_len).collect();
                if !sel.contains('\n') {
                    self.find_replace.query = sel;
                }
            }
        }
        self.find_replace.is_open = true;
        self.find_replace.tab = tab;
        self.find_replace.focus_query = true;
    }

    /// The selected text, or the word under the caret.
    pub fn current_word(&self) -> String {
        let Some(doc) = self.active_doc() else { return String::new() };
        let chars: Vec<char> = doc.content.chars().collect();
        if doc.selection_len > 0 {
            return chars.iter().skip(doc.cursor_char).take(doc.selection_len).collect::<String>().trim().to_string();
        }
        let at = doc.cursor_char.min(chars.len());
        let mut s = at;
        while s > 0 && is_word_char(chars[s - 1]) {
            s -= 1;
        }
        let mut e = at;
        while e < chars.len() && is_word_char(chars[e]) {
            e += 1;
        }
        chars[s..e].iter().collect()
    }

    /// Opens `base` + the current word in the default browser.
    pub fn web_lookup(&mut self, ctx: &Context, base: &str) {
        let word = self.current_word();
        if word.is_empty() {
            self.set_notification("Place the caret on a word first");
            return;
        }
        ctx.open_url(egui::OpenUrl::new_tab(format!("{base}{}", urlencoding::encode(&word))));
    }

    pub fn trim_and_save(&mut self) {
        self.apply_op(|d| operations::trim_trailing_spaces(&d.content));
        self.save_current();
    }

    /// Opens the active file in a second rust-note process.
    pub fn open_in_new_instance(&mut self) {
        let Some(path) = self.active_doc().and_then(|d| d.path.clone()) else {
            self.set_notification("Document has not been saved to disk yet");
            return;
        };
        match std::env::current_exe().and_then(|exe| {
            std::process::Command::new(exe).arg("-multiInst").arg("-nosession").arg(path).spawn()
        }) {
            Ok(_) => self.set_notification("Opened in a new instance"),
            Err(e) => self.set_notification(format!("Cannot start a new instance: {e}")),
        }
    }

    pub fn zoom(&mut self, delta: f32) {
        self.font_size = (self.font_size + delta).clamp(8.0, 40.0);
    }

    // --- Document Summary ---
    pub fn show_summary(&mut self) {
        if let Some(doc) = self.active_doc() {
            let words = doc.content.split_whitespace().count();
            self.summary = SummaryState {
                is_open: true,
                char_count: doc.char_count(),
                word_count: words,
                line_count: doc.line_count(),
                byte_count: doc.byte_len(),
                path: doc
                    .path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Not saved on disk".to_string()),
                encoding: doc.encoding.to_string(),
                eol: doc.eol.to_string(),
            };
        }
    }

    // --- Global Hotkey processing ---
    fn handle_shortcuts(&mut self, ctx: &Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        let cs = Modifiers::CTRL | Modifiers::SHIFT;
        let ca = Modifiers::CTRL | Modifiers::ALT;
        let hit = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)));

        if hit(cs, Key::S) {
            self.save_all();
        } else if hit(ca, Key::S) {
            self.save_as_current();
        } else if hit(Modifiers::CTRL, Key::S) {
            self.save_current();
        }
        if hit(Modifiers::CTRL, Key::N) {
            self.new_document();
        }
        if hit(Modifiers::CTRL, Key::O) {
            self.open_file_dialog();
        }
        if hit(cs, Key::W) {
            self.close_all();
        } else if hit(Modifiers::CTRL, Key::W) {
            self.close_current();
        }
        if hit(cs, Key::T) {
            self.restore_closed_document();
        }
        if hit(Modifiers::CTRL, Key::F) {
            self.open_find(FindTab::Find);
        }
        if hit(Modifiers::CTRL, Key::H) {
            self.open_find(FindTab::Replace);
        }
        if hit(Modifiers::CTRL, Key::G) {
            let max_l = self.active_doc().map(|d| d.line_count()).unwrap_or(1);
            self.goto_line.is_open = true;
            self.goto_line.max_lines = max_l;
            self.goto_line.focus = true;
        }
        if hit(Modifiers::SHIFT, Key::F3) {
            self.execute_find(false);
        } else if hit(Modifiers::NONE, Key::F3) {
            self.execute_find(true);
        }
        if hit(ca, Key::L) {
            self.show_function_list = !self.show_function_list;
        }
        // Undo / redo operate on our own history (typing is recorded into it).
        if hit(Modifiers::CTRL, Key::Z) {
            if let Some(doc) = self.active_doc_mut() {
                doc.undo();
            }
        }
        if hit(Modifiers::CTRL, Key::Y) || hit(cs, Key::Z) {
            if let Some(doc) = self.active_doc_mut() {
                doc.redo();
            }
        }
        // Line / text operations
        if hit(Modifiers::CTRL, Key::D) {
            self.apply_op(|d| operations::duplicate_line(&d.content, d.cursor_line));
        }
        // Esc closes the open Find / Go to Line dialogs
        if (self.goto_line.is_open || self.find_replace.is_open) && hit(Modifiers::NONE, Key::Escape) {
            if self.goto_line.is_open {
                self.goto_line.is_open = false;
            } else {
                self.find_replace.is_open = false;
            }
        }
        // Commands carried over from a typical Notepad++ setup (Run / Macro menus)
        if hit(Modifiers::ALT | Modifiers::SHIFT, Key::S) {
            self.trim_and_save();
        }
        if hit(Modifiers::ALT, Key::F1) {
            self.web_lookup(ctx, "https://www.php.net/");
        }
        if hit(Modifiers::ALT, Key::F3) {
            self.web_lookup(ctx, "https://en.wikipedia.org/wiki/Special:Search?search=");
        }
        if hit(Modifiers::ALT, Key::F6) {
            self.open_in_new_instance();
        }
        if hit(Modifiers::CTRL, Key::L) {
            self.apply_op(|d| {
                let mut lines: Vec<&str> = d.content.split('\n').collect();
                let idx = d.cursor_line.saturating_sub(1);
                if idx < lines.len() && lines.len() > 1 {
                    lines.remove(idx);
                } else if lines.len() == 1 {
                    lines[0] = "";
                }
                lines.join("\n")
            });
        }
        if hit(Modifiers::CTRL, Key::Q) {
            let prefix = self.current_language.comment_prefix();
            self.apply_op(move |d| operations::toggle_comment(&d.content, prefix));
        }
        if hit(Modifiers::CTRL, Key::J) {
            self.apply_op(|d| operations::join_lines(&d.content));
        }
        if hit(cs, Key::U) {
            self.apply_op(|d| operations::convert_case(&d.content, CaseMode::Uppercase));
        } else if hit(Modifiers::CTRL, Key::U) {
            self.apply_op(|d| operations::convert_case(&d.content, CaseMode::Lowercase));
        }
        if hit(cs, Key::ArrowUp) {
            self.apply_op(|d| operations::move_line_up(&d.content, d.cursor_line));
        }
        if hit(cs, Key::ArrowDown) {
            self.apply_op(|d| operations::move_line_down(&d.content, d.cursor_line));
        }
        // Zoom
        if hit(Modifiers::CTRL, Key::Plus) || hit(Modifiers::CTRL, Key::Equals) {
            self.zoom(1.0);
        }
        if hit(Modifiers::CTRL, Key::Minus) {
            self.zoom(-1.0);
        }
        if hit(Modifiers::CTRL, Key::Num0) {
            self.font_size = 14.0;
        }
        // Tab switching
        if hit(Modifiers::CTRL, Key::PageDown) && !self.docs.is_empty() {
            self.active_index = (self.active_index + 1) % self.docs.len();
        }
        if hit(Modifiers::CTRL, Key::PageUp) && !self.docs.is_empty() {
            self.active_index = (self.active_index + self.docs.len() - 1) % self.docs.len();
        }
        let scroll = ctx.input(|i| (i.modifiers.ctrl, i.raw_scroll_delta.y));
        if scroll.0 && scroll.1 != 0.0 {
            self.zoom(scroll.1.signum());
        }
    }

    /// Applies a whole-document transformation to the active document (unless read-only).
    pub fn apply_op(&mut self, f: impl FnOnce(&Document) -> String) {
        if let Some(doc) = self.active_doc_mut() {
            if doc.is_readonly {
                return;
            }
            let out = f(doc);
            let keep = doc.cursor_char;
            doc.set_content(out);
            let len = doc.char_count();
            doc.pending_selection = Some((keep.min(len), keep.min(len)));
        }
    }
}

/// Export format shared by the HTML and RTF exporters.
enum ExportKind {
    Html,
    Rtf,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl RustNoteApp {
    /// Exports the active document through a native save dialog.
    fn export_current(&mut self, kind: ExportKind) {
        let (filter_name, exts, label) = match kind {
            ExportKind::Html => ("HTML Document", &["html", "htm"][..], "HTML"),
            ExportKind::Rtf => ("Rich Text Format", &["rtf"][..], "RTF"),
        };
        if let Some(path) = rfd::FileDialog::new().add_filter(filter_name, exts).save_file() {
            if let Some(doc) = self.active_doc() {
                let payload = match kind {
                    ExportKind::Html => {
                        plugins::export_to_html(&doc.title, &doc.content, self.current_language.name())
                    }
                    ExportKind::Rtf => plugins::export_to_rtf(&doc.title, &doc.content),
                };
                match std::fs::write(&path, payload) {
                    Ok(()) => self.set_notification(format!("Exported {label} to {}", path.display())),
                    Err(e) => self.set_notification(format!("Export error: {e}")),
                }
            }
        }
    }
}

impl eframe::App for RustNoteApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.active_index >= self.docs.len() {
            self.active_index = self.docs.len().saturating_sub(1);
        }
        // Language is per document
        if let Some(l) = self.active_doc().map(|d| d.language.unwrap_or(Language::PlainText)) {
            self.current_language = l;
        }

        self.handle_shortcuts(ctx);

        // Drag & drop files to open them
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        for p in dropped {
            self.open_path(p);
        }

        // Window title reflects the active document
        let title = match self.active_doc() {
            Some(d) => {
                let full = d.path.as_ref().map_or_else(|| d.title.clone(), |p| p.display().to_string());
                format!("{}{full} - rust-note", if d.is_dirty { "* " } else { "" })
            }
            None => "rust-note".to_string(),
        };
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }

        // Closing the window with unsaved documents: ask first
        if ctx.input(|i| i.viewport().close_requested()) && !self.force_quit {
            if self.any_dirty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                let mut ok = true;
                for i in 0..self.docs.len() {
                    if !self.confirm_close(i) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    self.force_quit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Clear expired notification
        if let Some((_, time)) = self.status_notification {
            if time.elapsed() > std::time::Duration::from_secs(4) {
                self.status_notification = None;
            }
        }

        // Persist settings / session when they change
        {
            let mut cur = self.settings.clone();
            cur.theme = self.theme;
            cur.lang = i18n::lang();
            cur.font_size = self.font_size;
            cur.word_wrap = self.word_wrap;
            cur.show_line_numbers = self.show_line_numbers;
            cur.show_function_list = self.show_function_list;
            cur.session_files = self.docs.iter().filter_map(|d| d.path.clone()).collect();
            if cur != self.saved_settings {
                cur.save();
                self.saved_settings = cur.clone();
                self.settings = cur;
            }
        }

        // UI: each panel is rendered by its dedicated ui::* module.
        self.render_menu_bar(ctx);
        self.render_toolbar(ctx);
        if !self.cli_args.no_tab_bar {
            self.render_tab_bar(ctx);
        }
        self.render_status_bar(ctx);
        if self.show_function_list {
            self.render_function_list(ctx);
        }
        self.render_editor(ctx);
        self.render_dialogs(ctx);
    }
}
