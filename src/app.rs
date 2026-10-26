//! Main rust-note application state and GUI rendering with eframe/egui.

use std::path::PathBuf;

use crate::buffer::{Document, Encoding, EolFormat};
use crate::cli::CliArgs;
use crate::editor::operations::{self, CaseMode};
use crate::i18n::{self, tr, UiLang};
use crate::plugins;
use crate::search::{self, SearchOptions};
use crate::settings::Settings;
use crate::syntax::highlight::highlight as syntax_highlight;
use crate::syntax::Language;
use crate::ui::dialogs::{
    self, AboutState, DialogAction, FindReplaceState, FindTab, GoToLineState, SummaryState,
};
use eframe::egui::{self, Color32, Context, FontId, Layout, RichText, ScrollArea};

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
    pub show_symbols: bool,
    pub document_map: bool,
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
    symbol_cache: Option<(usize, u64, Language, Vec<crate::syntax::DocumentSymbol>)>,
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
            show_symbols: false,
            document_map: false,
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
                    self.set_notification(format!("Saved as {}", path.display()));
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

    pub fn export_current_to_html(&mut self) {
        if let Some(path) = rfd::FileDialog::new().add_filter("HTML Document", &["html", "htm"]).save_file() {
            if let Some(doc) = self.active_doc() {
                let html = plugins::export_to_html(&doc.title, &doc.content, self.current_language.name());
                if let Err(e) = std::fs::write(&path, html) {
                    self.set_notification(format!("Export error: {e}"));
                } else {
                    self.set_notification(format!("Exported HTML to {}", path.display()));
                }
            }
        }
    }

    pub fn export_current_to_rtf(&mut self) {
        if let Some(path) = rfd::FileDialog::new().add_filter("Rich Text Format", &["rtf"]).save_file() {
            if let Some(doc) = self.active_doc() {
                let rtf = plugins::export_to_rtf(&doc.title, &doc.content);
                if let Err(e) = std::fs::write(&path, rtf) {
                    self.set_notification(format!("Export error: {e}"));
                } else {
                    self.set_notification(format!("Exported RTF to {}", path.display()));
                }
            }
        }
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

/// Auto-completion popup state (candidates for the word being typed).
#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    pub items: Vec<String>,
    pub selected: usize,
    /// Caret position (char index) the candidates were computed for.
    pub at: usize,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Keywords of the language plus words already present in the document that start with the
/// word before the caret.
fn completion_candidates(content: &str, caret: usize, lang: Language, forced: bool) -> Vec<String> {
    if content.len() > 2_000_000 {
        return Vec::new();
    }
    let chars: Vec<char> = content.chars().collect();
    let end = caret.min(chars.len());
    let mut start = end;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let prefix: String = chars[start..end].iter().collect();
    if prefix.chars().count() < 1 {
        return Vec::new();
    }
    let ci = matches!(lang, Language::Sql | Language::Batch);
    let matches_prefix = |w: &str| {
        let ok = if ci { w.to_lowercase().starts_with(&prefix.to_lowercase()) } else { w.starts_with(&prefix) };
        ok && w != prefix
    };
    let mut set = std::collections::BTreeSet::new();
    for kw in lang.keywords() {
        if matches_prefix(kw) {
            set.insert((*kw).to_string());
        }
    }
    let _ = forced;
    for kw in crate::syntax::dict::dictionary(lang) {
        if matches_prefix(&kw) {
            set.insert(kw);
        }
    }
    let mut word = String::new();
    let mut word_start = 0usize;
    for (i, c) in chars.iter().copied().chain(std::iter::once(' ')).enumerate() {
        if is_word_char(c) {
            if word.is_empty() {
                word_start = i;
            }
            word.push(c);
        } else if !word.is_empty() {
            // Skip the word currently being typed
            if word_start != start && word.chars().count() >= 3 && matches_prefix(&word) {
                set.insert(word.clone());
            }
            word.clear();
        }
    }
    set.into_iter().take(8).collect()
}

fn apply_completion(doc: &mut Document, item: &str, ctx: &Context, id: egui::Id) {
    let chars: Vec<char> = doc.content.chars().collect();
    let end = doc.cursor_char.min(chars.len());
    let mut start = end;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut out: String = chars[..start].iter().collect();
    out.push_str(item);
    out.extend(chars[end..].iter());
    doc.content = out;
    doc.record_typing();
    let pos = start + item.chars().count();
    let mut st = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    st.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(pos))));
    st.store(ctx, id);
    doc.cursor_char = pos;
}

/// Darkens pastel UI accents on the light theme so they stay readable.
fn tc(light: bool, r: u8, g: u8, b: u8) -> Color32 {
    if light {
        Color32::from_rgb((f32::from(r) * 0.45) as u8, (f32::from(g) * 0.45) as u8, (f32::from(b) * 0.55) as u8)
    } else {
        Color32::from_rgb(r, g, b)
    }
}

fn layout_font_id(size: f32) -> FontId {
    FontId::monospace(size)
}

fn language_icon(lang: Option<Language>) -> &'static str {
    match lang {
        Some(Language::Rust) => "🦀",
        Some(Language::Python) => "🐍",
        Some(Language::JavaScript | Language::TypeScript) => "⚡",
        Some(Language::C | Language::Cpp | Language::Cs) => "⚙",
        Some(Language::Html | Language::Xml) => "🌐",
        Some(Language::Markdown) => "📝",
        Some(Language::Shell | Language::PowerShell | Language::Batch) => "💻",
        Some(Language::Sql) => "🗄",
        Some(Language::Json | Language::Toml | Language::Yaml | Language::Ini) => "🔧",
        Some(Language::Java | Language::Kotlin) => "☕",
        Some(Language::Go) => "🐹",
        Some(Language::Lua | Language::GdScript) => "🎮",
        Some(Language::Assembly) => "🔩",
        _ => "📄",
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

        let light = !self.theme.is_dark();
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

        // 1. Menu Bar
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // --- FILE ---
                ui.menu_button(tr("File"), |ui| {
                    if ui.button(tr("New\tCtrl+N")).clicked() {
                        self.new_document();
                        ui.close();
                    }
                    if ui.button(tr("Open...\tCtrl+O")).clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    if ui.button(tr("Save\tCtrl+S")).clicked() {
                        self.save_current();
                        ui.close();
                    }
                    if ui.button(tr("Save As...\tCtrl+Alt+S")).clicked() {
                        self.save_as_current();
                        ui.close();
                    }
                    if ui.button(tr("Save All\tCtrl+Shift+S")).clicked() {
                        self.save_all();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Close\tCtrl+W")).clicked() {
                        self.close_current();
                        ui.close();
                    }
                    if ui.button(tr("Close All\tCtrl+Shift+W")).clicked() {
                        self.close_all();
                        ui.close();
                    }
                    if ui.button(tr("Close All BUT Current")).clicked() {
                        self.close_all_except_current();
                        ui.close();
                    }
                    if ui.button(tr("Restore Closed Tab\tCtrl+Shift+T")).clicked() {
                        self.restore_closed_document();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Export to HTML...")).clicked() {
                        self.export_current_to_html();
                        ui.close();
                    }
                    if ui.button(tr("Export to RTF (Rich Text)...")).clicked() {
                        self.export_current_to_rtf();
                        ui.close();
                    }
                    ui.separator();
                    ui.add_enabled_ui(!self.settings.recent_files.is_empty(), |ui| {
                        ui.menu_button(tr("Recent Files"), |ui| {
                            let recents = self.settings.recent_files.clone();
                            for (n, path) in recents.iter().enumerate() {
                                let name = path.file_name().map_or_else(|| path.display().to_string(), |f| f.to_string_lossy().to_string());
                                if ui.button(format!("{}  {name}", n + 1)).on_hover_text(path.display().to_string()).clicked() {
                                    self.open_path(path.clone());
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button(tr("Clear Recent Files")).clicked() {
                                self.settings.recent_files.clear();
                                ui.close();
                            }
                        });
                    });
                    ui.menu_button(tr("Copy to Clipboard"), |ui| {
                        if ui.button(tr("Copy Full File Path")).clicked() {
                            self.copy_full_path(ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Filename")).clicked() {
                            self.copy_filename(ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Directory Path")).clicked() {
                            self.copy_directory_path(ctx);
                            ui.close();
                        }
                    });
                    ui.separator();
                    if ui.button(tr("Exit\tAlt+F4")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });

                // --- EDIT ---
                ui.menu_button(tr("Edit"), |ui| {
                    if ui.button(tr("Undo\tCtrl+Z")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.undo();
                        }
                        ui.close();
                    }
                    if ui.button(tr("Redo\tCtrl+Y")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.redo();
                        }
                        ui.close();
                    }
                    ui.separator();

                    ui.menu_button(tr("Line Operations"), |ui| {
                        if ui.button(tr("Duplicate Current Line\tCtrl+D")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let dup = operations::duplicate_line(&doc.content, cur);
                                doc.set_content(dup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Remove Duplicate Lines")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, false);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Remove Consecutive Duplicate Lines")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, true);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Move Line Up")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_up(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line = doc.cursor_line.saturating_sub(1).max(1);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Move Line Down")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_down(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line += 1;
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(tr("Sort Lines Ascending")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, true, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Sort Lines Descending")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, false, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Join Lines\tCtrl+J")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let joined = operations::join_lines(&doc.content);
                                doc.set_content(joined);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Convert Case to"), |ui| {
                        if ui.button(tr("UPPERCASE\tCtrl+Shift+U")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Uppercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("lowercase\tCtrl+U")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Lowercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Proper Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::ProperCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Invert Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::InvertCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Random Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::RandomCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Blank Operations"), |ui| {
                        if ui.button(tr("Trim Trailing Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_trailing_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Trim Leading Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_leading_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("TAB to Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::tabs_to_spaces(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Space to TAB")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::spaces_to_tabs(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Insert Date Time"), |ui| {
                        let now = chrono::Local::now();
                        if ui.button(tr("Date Time (short)")).clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Date Time (long)")).clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("EOL Conversion"), |ui| {
                        if ui.button(tr("Windows (CR LF)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Windows);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Windows;
                            }
                            ui.close();
                        }
                        if ui.button(tr("Unix (LF)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Unix);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Unix;
                            }
                            ui.close();
                        }
                        if ui.button(tr("Macintosh (CR)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Mac);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Mac;
                            }
                            ui.close();
                        }
                    });

                    if ui.button(tr("Toggle Comment\tCtrl+Q")).clicked() {
                        let prefix = self.current_language.comment_prefix();
                        if let Some(doc) = self.active_doc_mut() {
                            let c = operations::toggle_comment(&doc.content, prefix);
                            doc.set_content(c);
                        }
                        ui.close();
                    }
                });

                // --- SEARCH ---
                ui.menu_button(tr("Search"), |ui| {
                    if ui.button(tr("Find...\tCtrl+F")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Find;
                        ui.close();
                    }
                    if ui.button(tr("Replace...\tCtrl+H")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Replace;
                        ui.close();
                    }
                    if ui.button(tr("Find Next\tF3")).clicked() {
                        self.execute_find(true);
                        ui.close();
                    }
                    if ui.button(tr("Find Previous\tShift+F3")).clicked() {
                        self.execute_find(false);
                        ui.close();
                    }
                    if ui.button(tr("Go to Line...\tCtrl+G")).clicked() {
                        let max_l = self.active_doc().map(|d| d.line_count()).unwrap_or(1);
                        self.goto_line.is_open = true;
                        self.goto_line.max_lines = max_l;
                        ui.close();
                    }
                });

                // --- VIEW ---
                ui.menu_button(tr("View"), |ui| {
                    ui.checkbox(&mut self.word_wrap, tr("Word Wrap"));
                    ui.checkbox(&mut self.show_line_numbers, tr("Line Numbers"));
                    ui.checkbox(&mut self.show_function_list, tr("Function List (Ctrl+Alt+L)"));
                    ui.separator();
                    if ui.button(tr("Zoom In (Ctrl+Num+)")).clicked() {
                        self.font_size = (self.font_size + 2.0).min(36.0);
                        ui.close();
                    }
                    if ui.button(tr("Zoom Out (Ctrl+Num-)")).clicked() {
                        self.font_size = (self.font_size - 2.0).max(8.0);
                        ui.close();
                    }
                    if ui.button(tr("Restore Default Zoom")).clicked() {
                        self.font_size = 14.0;
                        ui.close();
                    }
                    ui.separator();
                    ui.menu_button(tr("Interface language"), |ui| {
                        for l in UiLang::ALL {
                            if ui.selectable_label(i18n::lang() == l, l.name()).clicked() {
                                i18n::set_lang(l);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button(tr("Theme"), |ui| {
                        for t in EditorTheme::ALL {
                            if ui.selectable_label(self.theme == t, t.name()).clicked() {
                                self.theme = t;
                                self.apply_theme(ctx);
                                self.set_notification(format!("Theme: {}", t.name()));
                                ui.close();
                            }
                        }
                    });
                });

                // --- ENCODING ---
                ui.menu_button(tr("Encoding"), |ui| {
                    if ui.button(tr("UTF-8")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-8-BOM")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8Bom;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-16 LE")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Le;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-16 BE")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Be;
                        }
                        ui.close();
                    }
                    if ui.button(tr("ANSI (Windows-1252)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Ansi;
                        }
                        ui.close();
                    }
                });

                // --- LANGUAGE ---
                ui.menu_button(tr("Language"), |ui| {
                    let langs = [
                        Language::PlainText,
                        Language::Rust,
                        Language::C,
                        Language::Cpp,
                        Language::Cs,
                        Language::Java,
                        Language::Python,
                        Language::JavaScript,
                        Language::TypeScript,
                        Language::Go,
                        Language::Html,
                        Language::Css,
                        Language::Xml,
                        Language::Json,
                        Language::Yaml,
                        Language::Toml,
                        Language::Sql,
                        Language::Shell,
                        Language::PowerShell,
                        Language::Batch,
                        Language::Php,
                        Language::Ruby,
                        Language::Perl,
                        Language::Lua,
                        Language::Swift,
                        Language::Kotlin,
                        Language::Dart,
                        Language::R,
                        Language::GdScript,
                        Language::Markdown,
                        Language::Assembly,
                        Language::Ini,
                        Language::Diff,
                        Language::Makefile,
                    ];
                    ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                        for l in langs {
                            if ui.selectable_label(self.current_language == l, l.name()).clicked() {
                                self.current_language = l;
                                if let Some(doc) = self.active_doc_mut() {
                                    doc.language = Some(l);
                                }
                                ui.close();
                            }
                        }
                    });
                });

                // --- PLUGINS & TOOLS ---
                ui.menu_button(tr("Plugins"), |ui| {
                    ui.menu_button(tr("MIME Tools"), |ui| {
                        if ui.button(tr("Base64 Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::base64_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Base64 Decode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::base64_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("URL Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::url_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button(tr("URL Decode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::url_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("Quoted-Printable Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::quoted_printable_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Converter"), |ui| {
                        if ui.button(tr("0x Quick Conversion Panel...")).clicked() {
                            self.conversion_panel.is_open = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(tr("ASCII to HEX")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let hex = plugins::ascii_to_hex(&doc.content);
                                doc.set_content(hex);
                            }
                            ui.close();
                        }
                        if ui.button(tr("HEX to ASCII")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(asc) = plugins::hex_to_ascii(&doc.content) {
                                    doc.set_content(asc);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("DEC to HEX (0x...)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(hex) = plugins::dec_to_hex(&doc.content) {
                                    doc.set_content(hex);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("HEX to DEC")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::hex_to_dec(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("NppExport"), |ui| {
                        if ui.button(tr("Export to HTML...")).clicked() {
                            self.export_current_to_html();
                            ui.close();
                        }
                        if ui.button(tr("Export to RTF (Rich Text)...")).clicked() {
                            self.export_current_to_rtf();
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("NppShell"), |ui| {
                        if ui.button(tr("Copy Full File Path")).clicked() {
                            self.copy_full_path(ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Filename")).clicked() {
                            self.copy_filename(ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Directory Path")).clicked() {
                            self.copy_directory_path(ctx);
                            ui.close();
                        }
                    });
                });

                // --- TOOLS ---
                ui.menu_button(tr("Tools"), |ui| {
                    if ui.button(tr("Document Summary")).clicked() {
                        self.show_summary();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Trim Trailing Space and Save\tAlt+Shift+S")).clicked() {
                        self.trim_and_save();
                        ui.close();
                    }
                    if ui.button(tr("Open in New Instance\tAlt+F6")).clicked() {
                        self.open_in_new_instance();
                        ui.close();
                    }
                    ui.menu_button(tr("Help on Internet"), |ui| {
                        if ui.button(tr("Get PHP help\tAlt+F1")).clicked() {
                            self.web_lookup(ctx, "https://www.php.net/");
                            ui.close();
                        }
                        if ui.button(tr("Wikipedia Search\tAlt+F3")).clicked() {
                            self.web_lookup(ctx, "https://en.wikipedia.org/wiki/Special:Search?search=");
                            ui.close();
                        }
                        if ui.button(tr("Search the web")).clicked() {
                            self.web_lookup(ctx, "https://duckduckgo.com/?q=");
                            ui.close();
                        }
                    });
                });

                // --- HELP (?) ---
                ui.menu_button(tr("?"), |ui| {
                    if ui.button(tr("Keyboard Shortcuts")).clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    if ui.button(tr("About rust-note")).clicked() {
                        self.about.is_open = true;
                        ui.close();
                    }
                    if ui.button(tr("Documentation & Sources")).clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab("https://github.com/laurentvv/rust-note"));
                        ui.close();
                    }
                });
            });
        });

        // 2. Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Group 1: Files
                ui.horizontal(|ui| {
                    if ui.button(tr("📄 New")).on_hover_text(tr("New Document (Ctrl+N)")).clicked() {
                        self.new_document();
                    }
                    if ui.button(tr("📂 Open")).on_hover_text(tr("Open File (Ctrl+O)")).clicked() {
                        self.open_file_dialog();
                    }
                    if ui.button(tr("💾 Save")).on_hover_text(tr("Save (Ctrl+S)")).clicked() {
                        self.save_current();
                    }
                    if ui.button(tr("💾💾 All")).on_hover_text(tr("Save All (Ctrl+Shift+S)")).clicked() {
                        self.save_all();
                    }
                    if ui.button(tr("❌ Close")).on_hover_text(tr("Close Tab (Ctrl+W)")).clicked() {
                        self.close_current();
                    }
                });

                ui.separator();

                // Group 2: History
                ui.horizontal(|ui| {
                    if ui.button(tr("↩ Undo")).on_hover_text(tr("Undo (Ctrl+Z)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.undo();
                        }
                    }
                    if ui.button(tr("↪ Redo")).on_hover_text(tr("Redo (Ctrl+Y)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.redo();
                        }
                    }
                });

                ui.separator();

                // Group 3: Search
                ui.horizontal(|ui| {
                    if ui.button(tr("🔍 Find")).on_hover_text(tr("Find (Ctrl+F)")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Find;
                    }
                    if ui.button(tr("🔁 Replace")).on_hover_text(tr("Replace (Ctrl+H)")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Replace;
                    }
                });

                ui.separator();

                // Group 4: Tools & Converters (from NppConverter.dll)
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.conversion_panel.is_open, tr("🧮 0x Converter")).on_hover_text(tr("Real-Time Multi-Base Quick Converter")).clicked() {
                        self.conversion_panel.is_open = !self.conversion_panel.is_open;
                    }
                    if ui.selectable_label(self.show_function_list, tr("📋 Symbols")).on_hover_text(tr("Function List Outline (Ctrl+Alt+L)")).clicked() {
                        self.show_function_list = !self.show_function_list;
                    }
                    if ui.button(tr("📊 Summary")).on_hover_text(tr("Document Properties")).clicked() {
                        self.show_summary();
                    }
                });

                ui.separator();

                // Group 5: View & Zoom
                ui.horizontal(|ui| {
                    if ui.button(tr("➕")).on_hover_text(tr("Zoom in (Ctrl+Num+)")).clicked() {
                        self.font_size = (self.font_size + 2.0).min(36.0);
                    }
                    if ui.button(tr("➖")).on_hover_text(tr("Zoom out (Ctrl+Num-)")).clicked() {
                        self.font_size = (self.font_size - 2.0).max(8.0);
                    }
                    if ui.selectable_label(self.word_wrap, tr("↩ Wrap")).clicked() {
                        self.word_wrap = !self.word_wrap;
                    }
                });

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut chosen = self.theme;
                    egui::ComboBox::from_id_salt("theme_combo")
                        .width(110.0)
                        .selected_text(tr("🎨 Theme"))
                        .show_ui(ui, |ui| {
                            for t in EditorTheme::ALL {
                                ui.selectable_value(&mut chosen, t, t.name());
                            }
                        });
                    if chosen != self.theme {
                        self.theme = chosen;
                        self.apply_theme(ctx);
                    }
                });
            });
        });

        // 3. Tab Bar
        if !self.cli_args.no_tab_bar {
            egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut to_close = None;
                            let mut close_others = None;
                            let mut copy_path_idx = None;
                            let mut copy_name_idx = None;

                            for (idx, doc) in self.docs.iter().enumerate() {
                                let is_active = idx == self.active_index;
                                let icon = language_icon(doc.language);
                                let dirty_dot = if doc.is_dirty { " ●" } else { "" };
                                let label_text = format!("{icon} {}{dirty_dot}", doc.title);

                                let text_color = if is_active {
                                    tc(light, 137, 180, 250) // Soft Accent Blue
                                } else if doc.is_dirty {
                                    tc(light, 250, 179, 135) // Amber
                                } else {
                                    tc(light, 186, 194, 222) // Soft Text
                                };

                                ui.horizontal(|ui| {
                                    let tab_btn = ui.selectable_label(
                                        is_active,
                                        RichText::new(&label_text).color(text_color).strong(),
                                    );
                                    if tab_btn.clicked() {
                                        self.active_index = idx;
                                    }
                                    if tab_btn.middle_clicked() {
                                        to_close = Some(idx);
                                    }

                                    tab_btn.context_menu(|ui| {
                                        if ui.button(tr("Close Tab\tCtrl+W")).clicked() {
                                            to_close = Some(idx);
                                            ui.close();
                                        }
                                        if ui.button(tr("Close Other Tabs")).clicked() {
                                            close_others = Some(idx);
                                            ui.close();
                                        }
                                        ui.separator();
                                        if ui.button(tr("Copy Full File Path")).clicked() {
                                            copy_path_idx = Some(idx);
                                            ui.close();
                                        }
                                        if ui.button(tr("Copy Filename")).clicked() {
                                            copy_name_idx = Some(idx);
                                            ui.close();
                                        }
                                    });

                                    if ui.small_button(tr("×")).on_hover_text(tr("Close tab (Ctrl+W)")).clicked() {
                                        to_close = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = copy_path_idx {
                                if let Some(doc) = self.docs.get(idx) {
                                    if let Some(ref p) = doc.path {
                                        ctx.copy_text(p.to_string_lossy().to_string());
                                        self.set_notification("Copied full path to clipboard");
                                    }
                                }
                            }

                            if let Some(idx) = copy_name_idx {
                                if let Some(doc) = self.docs.get(idx) {
                                    ctx.copy_text(doc.title.clone());
                                    self.set_notification("Copied filename to clipboard");
                                }
                            }

                            if let Some(keep_idx) = close_others {
                                self.close_all_except(keep_idx);
                            }

                            if let Some(idx) = to_close {
                                self.close_at(idx);
                            }

                            if ui.button(tr("+")).on_hover_text(tr("New tab (Ctrl+N)")).clicked() {
                                self.new_document();
                            }
                        });
                    });
                });
            });
        }

        // 4. Status Bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(doc) = self.active_doc() {
                    let (lines, chars, bytes) = (doc.line_count(), doc.char_count(), doc.byte_len());
                    let enc = doc.encoding.to_string();
                    let eol = doc.eol.to_string();
                    let icon = language_icon(doc.language);

                    ui.label(
                        RichText::new(format!("{} {}, {} {}", tr("Ln"), doc.cursor_line, tr("Col"), doc.cursor_col))
                            .strong()
                            .color(tc(light, 250, 179, 135)),
                    );
                    if let Some(cs) = doc.column_sel {
                        let (l1, l2, c1, c2) = cs.bounds();
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{}: {}×{}", tr("Column"), l2 - l1 + 1, c2 - c1))
                                .strong()
                                .color(tc(light, 245, 194, 231)),
                        );
                    }
                    if doc.selection_len > 0 {
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{}: {}", tr("Sel"), doc.selection_len))
                                .strong()
                                .color(tc(light, 245, 194, 231)),
                        );
                    }
                    ui.separator();
                    ui.label(
                        RichText::new(format!("📊 {chars} {} | {lines} {} | {bytes} {}", tr("chars"), tr("lines"), tr("B")))
                            .color(tc(light, 166, 173, 200)),
                    );
                    ui.separator();
                    ui.label(RichText::new(eol).color(tc(light, 147, 153, 178)));
                    ui.separator();
                    ui.label(RichText::new(enc).strong().color(tc(light, 203, 166, 247)));
                    ui.separator();
                    ui.label(
                        RichText::new(format!("{icon} {}", self.current_language.name()))
                            .strong()
                            .color(tc(light, 137, 220, 235)),
                    );
                    ui.separator();
                    if doc.is_readonly {
                        ui.label(RichText::new(tr("🔒 READ ONLY")).strong().color(tc(light, 243, 139, 168)));
                    } else {
                        ui.label(RichText::new("✏ INS").strong().color(tc(light, 166, 227, 161)));
                    }
                }

                if let Some((ref msg, _)) = self.status_notification {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("🔔 {msg}")).strong().color(tc(light, 250, 179, 135)));
                    });
                }
            });
        });

        // 4b. Function List Sidebar
        if self.show_function_list {
            egui::SidePanel::right("function_list_sidebar")
                .default_width(240.0)
                .min_width(180.0)
                .max_width(450.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(tr("📋 Function List"));
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(tr("×")).clicked() {
                                self.show_function_list = false;
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.text_edit_singleline(&mut self.symbol_filter);
                        if !self.symbol_filter.is_empty() && ui.small_button(tr("×")).clicked() {
                            self.symbol_filter.clear();
                        }
                    });
                    ui.separator();

                    let active_idx = self.active_index;
                    if let Some(doc) = self.docs.get(active_idx) {
                        let hash = {
                            use std::hash::{Hash, Hasher};
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            doc.content.hash(&mut h);
                            h.finish()
                        };
                        let lang = self.current_language;
                        let doc_id = doc.id;
                        let fresh = !matches!(&self.symbol_cache, Some((i, hh, l, _)) if *i == doc_id && *hh == hash && *l == lang);
                        if fresh {
                            let syms = crate::syntax::fnlist::symbols_for(&doc.content, lang);
                            self.symbol_cache = Some((doc_id, hash, lang, syms));
                        }
                        let symbols = self.symbol_cache.as_ref().map(|c| c.3.clone()).unwrap_or_default();
                        let filter_lower = self.symbol_filter.to_lowercase();
                        let filtered: Vec<_> = symbols
                            .into_iter()
                            .filter(|s| filter_lower.is_empty() || s.name.to_lowercase().contains(&filter_lower))
                            .collect();

                        ui.label(
                            RichText::new(format!("{}: {}", tr("Symbols"), filtered.len()))
                                .color(tc(light, 166, 173, 200)),
                        );
                        ui.separator();

                        ScrollArea::vertical()
                            .id_salt("function_list_scroll")
                            .show(ui, |ui| {
                                for sym in filtered {
                                    let color = match sym.kind {
                                        crate::syntax::SymbolKind::Function => tc(light, 137, 220, 235),
                                        crate::syntax::SymbolKind::StructOrClass => tc(light, 166, 227, 161),
                                        crate::syntax::SymbolKind::InterfaceOrTrait => tc(light, 203, 166, 247),
                                        crate::syntax::SymbolKind::Enum => tc(light, 250, 179, 135),
                                    };

                                    let label = RichText::new(format!("{} {}", sym.kind.icon(), sym.name)).color(color);
                                    if ui.button(label).on_hover_text(format!("{} {}", tr("Jump to line"), sym.line)).clicked() {
                                        if let Some(doc_mut) = self.docs.get_mut(active_idx) {
                                            doc_mut.cursor_line = sym.line;
                                            doc_mut.cursor_col = 1;
                                        }
                                        self.set_notification(format!("Jumped to '{}' (line {})", sym.name, sym.line));
                                    }
                                }
                            });
                    }
                });
        }

        // 5. Central Panel (Editor Canvas)
        let mut completion = std::mem::take(&mut self.completion);
        let editor_bg = self.theme.colors().editor;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(editor_bg).inner_margin(egui::Margin::same(4)))
            .show(ctx, |ui| {
            let active_idx = self.active_index;
            if active_idx >= self.docs.len() {
                return;
            }

            let available_size = ui.available_size();
            let font = FontId::monospace(self.font_size);
            let current_lang = self.current_language;
            let show_line_nums = self.show_line_numbers;
            let word_wrap = self.word_wrap;
            let find_open = self.find_replace.is_open;
            let font_size = self.font_size;

            ui.allocate_ui_with_layout(
                available_size,
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.set_min_height(available_size.y);

                    let doc = &mut self.docs[active_idx];
                    let pal = self.theme.colors().palette;
                    let doc_lang = doc.language.unwrap_or(Language::PlainText);

                    // Line numbers gutter (painted after the editor so it lines up with wrapped rows)
                    let gutter_rect = if show_line_nums {
                        let total_lines = doc.line_count();
                        let max_digits = total_lines.to_string().len().max(2);
                        let gutter_width = (max_digits as f32 * font.size * 0.62) + 20.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(gutter_width, available_size.y),
                            egui::Sense::hover(),
                        );
                        Some(rect)
                    } else {
                        None
                    };

                    // Editor Text Area (Takes all remaining width and 100% full height)
                    let editor_avail_size = ui.available_size();

                    ui.style_mut().wrap_mode = if word_wrap {
                        Some(egui::TextWrapMode::Wrap)
                    } else {
                        Some(egui::TextWrapMode::Extend)
                    };

                    let editor_id = egui::Id::new(("editor", doc.id));
                    let pending = doc.pending_selection.take();
                    if let Some((a, b)) = pending {
                        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), editor_id).unwrap_or_default();
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(a),
                            egui::text::CCursor::new(b),
                        )));
                        state.store(ui.ctx(), editor_id);
                        if !find_open {
                            ui.ctx().memory_mut(|m| m.request_focus(editor_id));
                        }
                    }

                    // Auto-completion popup: navigation / accept keys are handled before the editor sees them
                    let mut accept_idx: Option<usize> = None;
                    let force_complete = ui.ctx().input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));
                    if !completion.items.is_empty() && ui.ctx().memory(|m| m.has_focus(editor_id)) {
                        let n = completion.items.len();
                        ui.ctx().input_mut(|i| {
                            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                                completion.selected = (completion.selected + 1) % n;
                            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                                completion.selected = (completion.selected + n - 1) % n;
                            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                                || i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                            {
                                accept_idx = Some(completion.selected);
                            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                                completion.items.clear();
                            }
                        });
                    }
                    if let Some(idx) = accept_idx {
                        if let Some(item) = completion.items.get(idx).cloned() {
                            apply_completion(doc, &item, ui.ctx(), editor_id);
                        }
                        completion.items.clear();
                    }

                    // Column (rectangular) selection: typing / delete / copy / cut / paste act on the block
                    if let (Some(sel), true) = (doc.column_sel, ui.ctx().memory(|m| m.has_focus(editor_id))) {
                        let mut sel = sel;
                        let mut keep_sel = true;
                        let stamp_before = doc.last_edit;
                        let events = ui.ctx().input_mut(|i| std::mem::take(&mut i.events));
                        let mut kept = Vec::with_capacity(events.len());
                        for ev in events {
                            match &ev {
                                egui::Event::Text(t) | egui::Event::Paste(t) if !doc.is_readonly => {
                                    let lines: Vec<&str> = t.lines().collect();
                                    let (c, s) = if matches!(&ev, egui::Event::Paste(_)) && lines.len() > 1 {
                                        crate::editor::column::insert_each(&doc.content, &sel, &lines)
                                    } else {
                                        crate::editor::column::insert(&doc.content, &sel, t)
                                    };
                                    doc.content = c;
                                    sel = s;
                                    doc.record_typing();
                                }
                                egui::Event::Key { key: egui::Key::Backspace | egui::Key::Delete, pressed: true, modifiers, .. }
                                    if modifiers.is_none() && !doc.is_readonly =>
                                {
                                    let fwd = matches!(&ev, egui::Event::Key { key: egui::Key::Delete, .. });
                                    let (c, s) = crate::editor::column::delete(&doc.content, &sel, fwd);
                                    doc.content = c;
                                    sel = s;
                                    doc.record_typing();
                                }
                                egui::Event::Copy => {
                                    ui.ctx().copy_text(crate::editor::column::extract(&doc.content, &sel));
                                }
                                egui::Event::Cut if !doc.is_readonly => {
                                    ui.ctx().copy_text(crate::editor::column::extract(&doc.content, &sel));
                                    if !sel.is_caret_only() {
                                        let (c, s) = crate::editor::column::delete(&doc.content, &sel, true);
                                        doc.content = c;
                                        sel = s;
                                        doc.record_typing();
                                    }
                                }
                                egui::Event::Key {
                                    key:
                                        egui::Key::Enter
                                        | egui::Key::Escape
                                        | egui::Key::ArrowLeft
                                        | egui::Key::ArrowRight
                                        | egui::Key::ArrowUp
                                        | egui::Key::ArrowDown
                                        | egui::Key::Home
                                        | egui::Key::End
                                        | egui::Key::PageUp
                                        | egui::Key::PageDown,
                                    pressed: true,
                                    ..
                                } => {
                                    keep_sel = false;
                                    kept.push(ev);
                                }
                                _ => kept.push(ev),
                            }
                        }
                        ui.ctx().input_mut(|i| i.events = kept);
                        doc.column_sel = if keep_sel { Some(sel) } else { None };
                        if keep_sel && doc.last_edit != stamp_before {
                            // Keep the text caret at the block head after an edit
                            let pos = crate::editor::column::head_char_index(&doc.content, &sel);
                            let mut st = egui::text_edit::TextEditState::load(ui.ctx(), editor_id).unwrap_or_default();
                            st.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(pos))));
                            st.store(ui.ctx(), editor_id);
                        }
                        if !keep_sel {
                            doc.column_dragging = false;
                        }
                    }

                    // Auto-indent: Enter keeps the indentation of the current line (+4 after an opening brace)
                    let enter_pressed = !doc.is_readonly
                        && ui.ctx().memory(|m| m.has_focus(editor_id))
                        && ui.ctx().input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    if enter_pressed {
                        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), editor_id).unwrap_or_default();
                        if let Some(range) = state.cursor.char_range() {
                            let (a, b) = (range.primary.index.min(range.secondary.index), range.primary.index.max(range.secondary.index));
                            let chars: Vec<char> = doc.content.chars().collect();
                            let a = a.min(chars.len());
                            let b = b.min(chars.len());
                            let line_start = chars[..a].iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
                            let mut insert = String::from("\n");
                            insert.extend(chars[line_start..a].iter().take_while(|c| **c == ' ' || **c == '\t'));
                            if a > 0 && matches!(chars[a - 1], '{' | '(' | '[' | ':') {
                                insert.push_str("    ");
                            }
                            let mut out: String = chars[..a].iter().collect();
                            out.push_str(&insert);
                            out.extend(chars[b..].iter());
                            doc.content = out;
                            doc.record_typing();
                            let pos = a + insert.chars().count();
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                                egui::text::CCursor::new(pos),
                                egui::text::CCursor::new(pos),
                            )));
                            state.store(ui.ctx(), editor_id);
                        }
                    }

                    let mut gutter_numbers: Vec<(f32, usize)> = Vec::new();
                    let scroll_area = if word_wrap { ScrollArea::vertical() } else { ScrollArea::both() };
                    let scroll_output = scroll_area
                        .id_salt("editor_scroll")
                        .auto_shrink([false, false])
                        .min_scrolled_height(editor_avail_size.y)
                        .min_scrolled_width(editor_avail_size.x)
                        .show(ui, |ui| {
                            let layout_font = font.clone();
                            let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                let job = syntax_highlight(
                                    text.as_str(),
                                    doc_lang,
                                    &layout_font,
                                    pal,
                                    if word_wrap { wrap_width } else { f32::INFINITY },
                                );
                                ui.fonts_mut(|f| f.layout_job(job))
                            };
                            let output = egui::TextEdit::multiline(&mut doc.content)
                                .id(editor_id)
                                .font(font)
                                .text_color(pal.text)
                                .desired_width(f32::INFINITY)
                                .min_size(egui::Vec2::new(editor_avail_size.x, editor_avail_size.y))
                                .frame(false)
                                .interactive(!doc.is_readonly)
                                .layouter(&mut layouter)
                                .lock_focus(true)
                                .show(ui);
                            let response = output.response.clone();

                            if response.changed() {
                                doc.record_typing();
                            }

                            // Synchronize cursor position (Ln / Col) from text selection state
                            let cursor_range = output.state.cursor.char_range();
                            if let Some(range) = cursor_range {
                                let (line, col) = doc.line_col_from_char_index(range.primary.index);
                                doc.cursor_line = line;
                                doc.cursor_col = col;
                                doc.cursor_char = range.primary.index.min(range.secondary.index);
                                doc.selection_len = range.primary.index.abs_diff(range.secondary.index);
                            }

                            let galley_pos = output.galley_pos;
                            // Current-line highlight and scroll-to-cursor after a programmatic jump
                            if let Some(range) = cursor_range {
                                let crect = output.galley.pos_from_cursor(range.primary).translate(galley_pos.to_vec2());
                                if range.primary == range.secondary {
                                    let line_rect = egui::Rect::from_min_max(
                                        egui::pos2(ui.clip_rect().left(), crect.top()),
                                        egui::pos2(ui.clip_rect().right(), crect.bottom()),
                                    );
                                    ui.painter().rect_filled(
                                        line_rect,
                                        0.0,
                                        Color32::from_rgba_unmultiplied(128, 128, 160, 18),
                                    );
                                }
                                if pending.is_some() || enter_pressed {
                                    ui.scroll_to_rect(crect.expand2(egui::vec2(0.0, 40.0)), Some(if pending.is_some() { egui::Align::Center } else { egui::Align::Max }));
                                }
                            }

                            // Column selection: Alt+drag creates it, a plain click clears it
                            {
                                let cw = ui.fonts_mut(|f| f.glyph_width(&layout_font, 'M')).max(1.0);
                                let (alt, pressed, down, pointer) = ui.input(|i| {
                                    (i.modifiers.alt, i.pointer.primary_pressed(), i.pointer.primary_down(), i.pointer.interact_pos())
                                });
                                let to_lc = |p: egui::Pos2, doc: &Document| -> (usize, usize) {
                                    let rel = p - output.galley_pos;
                                    let c = output.galley.cursor_from_pos(rel);
                                    let (line, _) = doc.line_col_from_char_index(c.index);
                                    (line - 1, (rel.x / cw).round().max(0.0) as usize)
                                };
                                if let Some(p) = pointer {
                                    if pressed && response.rect.contains(p) {
                                        if alt {
                                            let lc = to_lc(p, doc);
                                            doc.column_sel = Some(crate::editor::column::ColumnSel { anchor: lc, head: lc });
                                            doc.column_dragging = true;
                                        } else {
                                            doc.column_sel = None;
                                        }
                                    }
                                    if doc.column_dragging {
                                        if down {
                                            let lc = to_lc(p, doc);
                                            if let Some(sel) = doc.column_sel.as_mut() {
                                                sel.head = lc;
                                            }
                                        } else {
                                            doc.column_dragging = false;
                                        }
                                    }
                                }
                                if let Some(sel) = doc.column_sel {
                                    // Hide the regular selection while a block is active
                                    if let Some(range) = cursor_range {
                                        if range.primary != range.secondary {
                                            let mut st = output.state.clone();
                                            st.cursor.set_char_range(Some(egui::text::CCursorRange::one(range.primary)));
                                            st.store(ui.ctx(), editor_id);
                                        }
                                    }
                                    let (l1, l2, c1, c2) = sel.bounds();
                                    let starts: Vec<usize> = std::iter::once(0)
                                        .chain(doc.content.chars().enumerate().filter(|(_, c)| *c == '\n').map(|(i, _)| i + 1))
                                        .collect();
                                    let fill = ui.visuals().selection.bg_fill.gamma_multiply(0.8);
                                    for l in l1..=l2 {
                                        if let Some(start) = starts.get(l) {
                                            let r = output.galley.pos_from_cursor(egui::text::CCursor::new(*start));
                                            let x0 = output.galley_pos.x + c1 as f32 * cw;
                                            let x1 = (output.galley_pos.x + c2 as f32 * cw).max(x0 + 2.0);
                                            let y0 = output.galley_pos.y + r.min.y;
                                            let rect = egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y0 + r.height()));
                                            ui.painter().rect_filled(rect, 0.0, fill);
                                        }
                                    }
                                }
                                let _ = alt;
                            }

                            // Auto-completion: refresh candidates after typing, close when the caret moves away
                            if let Some(range) = cursor_range {
                                if (response.changed() || force_complete)
                                    && !doc.is_readonly
                                    && doc.column_sel.is_none()
                                    && range.primary == range.secondary
                                {
                                    completion.items = completion_candidates(&doc.content, range.primary.index, doc_lang, force_complete);
                                    completion.selected = 0;
                                    completion.at = range.primary.index;
                                } else if range.primary.index != completion.at || !response.has_focus() {
                                    completion.items.clear();
                                }
                                if !completion.items.is_empty() {
                                    let r = output.galley.pos_from_cursor(range.primary).translate(output.galley_pos.to_vec2());
                                    let mut clicked = None;
                                    egui::Area::new(egui::Id::new("completion_popup"))
                                        .order(egui::Order::Foreground)
                                        .fixed_pos(r.left_bottom() + egui::vec2(0.0, 2.0))
                                        .show(ui.ctx(), |ui| {
                                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                for (i, item) in completion.items.iter().enumerate() {
                                                    let resp = ui.selectable_label(i == completion.selected, RichText::new(item).monospace());
                                                    if resp.clicked() {
                                                        clicked = Some(i);
                                                    }
                                                }
                                            });
                                        });
                                    if let Some(i) = clicked {
                                        if let Some(item) = completion.items.get(i).cloned() {
                                            apply_completion(doc, &item, ui.ctx(), editor_id);
                                        }
                                        completion.items.clear();
                                    }
                                }
                            }

                            if gutter_rect.is_some() {
                                let mut line_no = 1usize;
                                let mut new_line = true;
                                for placed in &output.galley.rows {
                                    if new_line {
                                        gutter_numbers.push((galley_pos.y + placed.pos.y, line_no));
                                        line_no += 1;
                                        new_line = false;
                                    }
                                    if placed.row.ends_with_newline {
                                        new_line = true;
                                    }
                                }
                            }

                            let comment_prefix = doc.language.unwrap_or(current_lang).comment_prefix();
                            response.context_menu(|ui| {
                                ui.label(RichText::new("rust-note Context Menu").strong());
                                ui.separator();
                                if ui.button(tr("Undo\tCtrl+Z")).clicked() {
                                    doc.undo();
                                    ui.close();
                                }
                            if ui.button(tr("Redo\tCtrl+Y")).clicked() {
                                doc.redo();
                                ui.close();
                            }
                            ui.separator();
                            ui.menu_button(tr("Line Operations"), |ui| {
                                if ui.button(tr("Duplicate Current Line\tCtrl+D")).clicked() {
                                    let cur = doc.cursor_line;
                                    let dup = operations::duplicate_line(&doc.content, cur);
                                    doc.set_content(dup);
                                    ui.close();
                                }
                                if ui.button(tr("Move Line Up")).clicked() {
                                    let cur = doc.cursor_line;
                                    let moved = operations::move_line_up(&doc.content, cur);
                                    doc.set_content(moved);
                                    doc.cursor_line = doc.cursor_line.saturating_sub(1).max(1);
                                    ui.close();
                                }
                                if ui.button(tr("Move Line Down")).clicked() {
                                    let cur = doc.cursor_line;
                                    let moved = operations::move_line_down(&doc.content, cur);
                                    doc.set_content(moved);
                                    doc.cursor_line += 1;
                                    ui.close();
                                }
                                if ui.button(tr("Remove Duplicate Lines")).clicked() {
                                    let dedup = operations::remove_duplicate_lines(&doc.content, false);
                                    doc.set_content(dedup);
                                    ui.close();
                                }
                                if ui.button(tr("Sort Lines Ascending")).clicked() {
                                    let sorted = operations::sort_lines(&doc.content, true, true, false);
                                    doc.set_content(sorted);
                                    ui.close();
                                }
                            });
                            ui.menu_button(tr("Convert Case"), |ui| {
                                if ui.button(tr("UPPERCASE\tCtrl+Shift+U")).clicked() {
                                    let c = operations::convert_case(&doc.content, CaseMode::Uppercase);
                                    doc.set_content(c);
                                    ui.close();
                                }
                                if ui.button(tr("lowercase\tCtrl+U")).clicked() {
                                    let c = operations::convert_case(&doc.content, CaseMode::Lowercase);
                                    doc.set_content(c);
                                    ui.close();
                                }
                                if ui.button(tr("Proper Case")).clicked() {
                                    let c = operations::convert_case(&doc.content, CaseMode::ProperCase);
                                    doc.set_content(c);
                                    ui.close();
                                }
                            });
                            ui.separator();
                            if ui.button(format!("Toggle Comment ({comment_prefix})")).clicked() {
                                let c = operations::toggle_comment(&doc.content, comment_prefix);
                                doc.set_content(c);
                                ui.close();
                            }
                            ui.separator();
                            ui.menu_button(tr("MIME / Tools"), |ui| {
                                if ui.button(tr("Base64 Encode")).clicked() {
                                    let enc = plugins::base64_encode(&doc.content);
                                    doc.set_content(enc);
                                    ui.close();
                                }
                                if ui.button(tr("Base64 Decode")).clicked() {
                                    if let Ok(dec) = plugins::base64_decode(&doc.content) {
                                        doc.set_content(dec);
                                    }
                                    ui.close();
                                }
                                if ui.button(tr("URL Encode")).clicked() {
                                    let enc = plugins::url_encode(&doc.content);
                                    doc.set_content(enc);
                                    ui.close();
                                }
                                if ui.button(tr("URL Decode")).clicked() {
                                    if let Ok(dec) = plugins::url_decode(&doc.content) {
                                        doc.set_content(dec);
                                    }
                                    ui.close();
                                }
                            });
                            ui.separator();
                            let doc_path = doc.path.clone();
                            let doc_title = doc.title.clone();
                            ui.menu_button(tr("Copy Path"), |ui| {
                                if ui.button(tr("Copy Full File Path")).clicked() {
                                    if let Some(ref p) = doc_path {
                                        ui.ctx().copy_text(p.to_string_lossy().to_string());
                                    }
                                    ui.close();
                                }
                                if ui.button(tr("Copy Filename")).clicked() {
                                    ui.ctx().copy_text(doc_title.clone());
                                    ui.close();
                                }
                                if ui.button(tr("Copy Directory Path")).clicked() {
                                    if let Some(ref p) = doc_path {
                                        if let Some(parent) = p.parent() {
                                            ui.ctx().copy_text(parent.to_string_lossy().to_string());
                                        }
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });

                    // Sync vertical scroll offset for gutter and persistence
                    doc.scroll_offset = scroll_output.state.offset.y;
                    if let Some(rect) = gutter_rect {
                        let painter = ui.painter().with_clip_rect(rect);
                        for (y, n) in gutter_numbers {
                            let color = if n == doc.cursor_line {
                                Color32::from_rgb(250, 179, 135)
                            } else {
                                Color32::from_rgb(110, 115, 140)
                            };
                            painter.text(
                                egui::pos2(rect.right() - 8.0, y),
                                egui::Align2::RIGHT_TOP,
                                n.to_string(),
                                layout_font_id(font_size),
                                color,
                            );
                        }
                    }
                },
            );
        });

        self.completion = completion;

        // 6. Dialog Renderings
        let action = dialogs::render_find_replace(ctx, &mut self.find_replace);
        match action {
            DialogAction::FindNext => self.execute_find_next(),
            DialogAction::ReplaceAll => self.execute_replace_all(),
            DialogAction::ReplaceOne => self.execute_find_next(),
            _ => {}
        }

        let goto_action = dialogs::render_go_to_line(ctx, &mut self.goto_line);
        if let DialogAction::GoToLine(target) = goto_action {
            let target_line = if let Some(doc) = self.docs.get_mut(self.active_index) {
                let clamped = target.min(doc.line_count()).max(1);
                doc.cursor_line = clamped;
                clamped
            } else {
                1
            };
            self.set_notification(format!("Jumped to line {target_line}"));
        }

        if self.show_shortcuts {
            let mut open = true;
            egui::Window::new("Keyboard Shortcuts").open(&mut open).resizable(false).show(ctx, |ui| {
                let rows: [(&str, &str); 22] = [
                    ("New / Open / Save", "Ctrl+N / Ctrl+O / Ctrl+S"),
                    ("Save All / Save As", "Ctrl+Shift+S / Ctrl+Alt+S"),
                    ("Close tab / Restore tab", "Ctrl+W / Ctrl+Shift+T"),
                    ("Next / previous tab", "Ctrl+PageDown / Ctrl+PageUp"),
                    ("Find / Replace", "Ctrl+F / Ctrl+H"),
                    ("Find next / previous", "F3 / Shift+F3"),
                    ("Go to line", "Ctrl+G"),
                    ("Duplicate line", "Ctrl+D"),
                    ("Delete line", "Ctrl+L"),
                    ("Move line up / down", "Ctrl+Shift+Up / Down"),
                    ("Join lines", "Ctrl+J"),
                    ("Toggle comment", "Ctrl+Q"),
                    ("UPPERCASE / lowercase", "Ctrl+Shift+U / Ctrl+U"),
                    ("Undo / Redo", "Ctrl+Z / Ctrl+Y"),
                    ("Select word / line", "Double-click / Triple-click"),
                    ("Extend selection", "Shift+click, Shift+arrows"),
                    ("Select by word", "Ctrl+Shift+Left / Right"),
                    ("Column selection", "Alt+drag"),
                    ("Auto-completion", "type 2+ letters, Ctrl+Space"),
                    ("Accept / navigate completion", "Tab or Enter / Up, Down"),
                    ("Zoom", "Ctrl+Wheel, Ctrl+Plus / Minus / 0"),
                    ("Function list", "Ctrl+Alt+L"),
                ];
                egui::Grid::new("shortcut_grid").num_columns(2).spacing([24.0, 4.0]).striped(true).show(ui, |ui| {
                    for (what, keys) in rows {
                        ui.label(what);
                        ui.label(RichText::new(keys).monospace().strong());
                        ui.end_row();
                    }
                });
            });
            self.show_shortcuts = open;
        }

        dialogs::render_about(ctx, &mut self.about);
        dialogs::render_summary(ctx, &mut self.summary);

        // 7. 0x Quick Conversion Panel (recovered from NppConverter.dll)
        if self.conversion_panel.is_open {
            let mut is_open = true;
            egui::Window::new("🧮 0x Quick Converter (NppConverter)")
                .open(&mut is_open)
                .resizable(false)
                .default_size([380.0, 240.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Bidirectional Real-Time Multi-Base Converter").strong());
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("DEC: ");
                        let mut d = self.conversion_panel.dec_val.clone();
                        if ui.text_edit_singleline(&mut d).changed() {
                            self.conversion_panel.sync_from_dec(&d);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("HEX: ");
                        let mut h = self.conversion_panel.hex_val.clone();
                        if ui.text_edit_singleline(&mut h).changed() {
                            self.conversion_panel.sync_from_hex(&h);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("BIN: ");
                        let mut b = self.conversion_panel.bin_val.clone();
                        if ui.text_edit_singleline(&mut b).changed() {
                            self.conversion_panel.sync_from_bin(&b);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("OCT: ");
                        ui.label(RichText::new(&self.conversion_panel.oct_val).monospace());
                    });
                    ui.horizontal(|ui| {
                        ui.label("ASCII:");
                        ui.label(RichText::new(&self.conversion_panel.char_val).monospace().strong());
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr("Copy HEX")).clicked() {
                            ctx.copy_text(format!("0x{}", self.conversion_panel.hex_val));
                            self.set_notification("Copied HEX to clipboard");
                        }
                        if ui.button(tr("Copy DEC")).clicked() {
                            ctx.copy_text(self.conversion_panel.dec_val.clone());
                            self.set_notification("Copied DEC to clipboard");
                        }
                        if ui.button(tr("Close")).clicked() {
                            self.conversion_panel.is_open = false;
                        }
                    });
                });
            if !is_open {
                self.conversion_panel.is_open = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_uses_keywords_and_document_words() {
        let text = "let alphabet = 1;
al";
        let caret = text.chars().count();
        let items = completion_candidates(text, caret, Language::Rust, false);
        assert!(items.contains(&"alphabet".to_string()));
        let items = completion_candidates("le", 2, Language::Rust, false);
        assert!(items.contains(&"let".to_string()));
        assert!(completion_candidates("", 0, Language::Rust, false).is_empty());
    }
}
