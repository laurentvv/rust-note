//! Main Notepad++ Application state and GUI rendering with eframe/egui.

use std::path::PathBuf;

use crate::buffer::{Document, Encoding, EolFormat};
use crate::cli::CliArgs;
use crate::editor::operations::{self, CaseMode};
use crate::plugins;
use crate::search;
use crate::syntax::Language;
use crate::ui::dialogs::{
    self, AboutState, DialogAction, FindReplaceState, FindTab, GoToLineState, SummaryState,
};
use eframe::egui::{self, Color32, Context, FontId, Layout, RichText, ScrollArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTheme {
    #[default]
    DarkMode,
    ClassicLight,
}

pub struct NotepadApp {
    pub docs: Vec<Document>,
    pub active_index: usize,
    pub cli_args: CliArgs,

    // Dialog states
    pub find_replace: FindReplaceState,
    pub goto_line: GoToLineState,
    pub about: AboutState,
    pub summary: SummaryState,

    // Editor settings
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub show_symbols: bool,
    pub document_map: bool,
    pub current_language: Language,
    pub theme: EditorTheme,
    pub status_notification: Option<(String, std::time::Instant)>,
}

impl NotepadApp {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, cli: CliArgs) -> Self {
        // Set visual style: crisp editor palette
        let visuals = egui::Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);

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

        let initial_lang = docs.first().and_then(|d| d.language).unwrap_or(Language::PlainText);

        Self {
            docs,
            active_index: 0,
            cli_args: cli,
            find_replace: FindReplaceState::default(),
            goto_line: GoToLineState::default(),
            about: AboutState::default(),
            summary: SummaryState::default(),
            font_size: 14.0,
            show_line_numbers: true,
            word_wrap: true,
            show_symbols: false,
            document_map: false,
            current_language: initial_lang,
            theme: EditorTheme::DarkMode,
            status_notification: None,
        }
    }

    pub fn apply_theme(&self, ctx: &Context) {
        match self.theme {
            EditorTheme::DarkMode => {
                let mut visuals = egui::Visuals::dark();
                visuals.panel_fill = Color32::from_rgb(32, 32, 32);
                visuals.window_fill = Color32::from_rgb(45, 45, 45);
                ctx.set_visuals(visuals);
            }
            EditorTheme::ClassicLight => {
                let mut visuals = egui::Visuals::light();
                visuals.panel_fill = Color32::from_rgb(240, 240, 240);
                visuals.window_fill = Color32::from_rgb(255, 255, 255);
                ctx.set_visuals(visuals);
            }
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), std::time::Instant::now()));
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

    pub fn close_current(&mut self) {
        if self.docs.len() <= 1 {
            // Keep at least one empty document
            self.docs[0] = Document::new_empty(Some("new 1"));
            self.set_notification("Document closed");
            return;
        }

        self.docs.remove(self.active_index);
        if self.active_index >= self.docs.len() {
            self.active_index = self.docs.len() - 1;
        }
        self.set_notification("Document closed");
    }

    pub fn close_all(&mut self) {
        self.docs.clear();
        self.docs.push(Document::new_empty(Some("new 1")));
        self.active_index = 0;
        self.set_notification("All documents closed");
    }

    pub fn close_all_except_current(&mut self) {
        if self.docs.is_empty() {
            return;
        }
        let current = self.docs.remove(self.active_index);
        self.docs.clear();
        self.docs.push(current);
        self.active_index = 0;
        self.set_notification("Closed other documents");
    }

    // --- Search & Replace Executions ---
    pub fn execute_find_next(&mut self) {
        let query = self.find_replace.query.clone();
        let opts = self.find_replace.options.clone();
        if query.is_empty() {
            return;
        }

        if let Some(doc) = self.active_doc_mut() {
            match search::find_all(&doc.content, &query, &opts) {
                Ok(matches) => {
                    if matches.is_empty() {
                        self.find_replace.status_message =
                            Some("Can't find the text \"".to_string() + &query + "\"");
                    } else {
                        let m = &matches[0];
                        doc.cursor_line = m.line;
                        self.find_replace.status_message =
                            Some(format!("Found {} matches. At line {}", matches.len(), m.line));
                    }
                }
                Err(e) => {
                    self.find_replace.status_message = Some(e);
                }
            }
        }
    }

    pub fn execute_replace_all(&mut self) {
        let query = self.find_replace.query.clone();
        let repl = self.find_replace.replacement.clone();
        let opts = self.find_replace.options.clone();

        if let Some(doc) = self.active_doc_mut() {
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
        let input = ctx.input(|i| i.clone());

        if input.modifiers.ctrl {
            if input.key_pressed(egui::Key::N) {
                self.new_document();
            } else if input.key_pressed(egui::Key::O) {
                self.open_file_dialog();
            } else if input.key_pressed(egui::Key::S) {
                if input.modifiers.shift {
                    self.save_all();
                } else if input.modifiers.alt {
                    self.save_as_current();
                } else {
                    self.save_current();
                }
            } else if input.key_pressed(egui::Key::W) {
                if input.modifiers.shift {
                    self.close_all();
                } else {
                    self.close_current();
                }
            } else if input.key_pressed(egui::Key::F) {
                self.find_replace.is_open = true;
                self.find_replace.tab = FindTab::Find;
            } else if input.key_pressed(egui::Key::H) {
                self.find_replace.is_open = true;
                self.find_replace.tab = FindTab::Replace;
            } else if input.key_pressed(egui::Key::G) {
                let max_l = self.active_doc().map(|d| d.line_count()).unwrap_or(1);
                self.goto_line.is_open = true;
                self.goto_line.max_lines = max_l;
            } else if input.key_pressed(egui::Key::D) {
                // Duplicate line
                if let Some(doc) = self.active_doc_mut() {
                    let cur_line = doc.cursor_line;
                    let dup = operations::duplicate_line(&doc.content, cur_line);
                    doc.set_content(dup);
                }
            } else if input.key_pressed(egui::Key::Q) {
                // Toggle comment
                let prefix = self.current_language.comment_prefix();
                if let Some(doc) = self.active_doc_mut() {
                    let commented = operations::toggle_comment(&doc.content, prefix);
                    doc.set_content(commented);
                }
            } else if input.key_pressed(egui::Key::J) {
                // Join lines
                if let Some(doc) = self.active_doc_mut() {
                    let joined = operations::join_lines(&doc.content);
                    doc.set_content(joined);
                }
            } else if input.key_pressed(egui::Key::Z) {
                if let Some(doc) = self.active_doc_mut() {
                    doc.undo();
                }
            } else if input.key_pressed(egui::Key::Y) {
                if let Some(doc) = self.active_doc_mut() {
                    doc.redo();
                }
            }
        } else if input.key_pressed(egui::Key::F3) {
            self.execute_find_next();
        }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        static ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[6] First update() frame rendered successfully!");
        }

        self.handle_shortcuts(ctx);

        // Clear expired notification
        if let Some((_, time)) = self.status_notification {
            if time.elapsed() > std::time::Duration::from_secs(4) {
                self.status_notification = None;
            }
        }

        // 1. Menu Bar
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // --- FILE ---
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        self.new_document();
                        ui.close();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_current();
                        ui.close();
                    }
                    if ui.button("Save As...\tCtrl+Alt+S").clicked() {
                        self.save_as_current();
                        ui.close();
                    }
                    if ui.button("Save All\tCtrl+Shift+S").clicked() {
                        self.save_all();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Close\tCtrl+W").clicked() {
                        self.close_current();
                        ui.close();
                    }
                    if ui.button("Close All\tCtrl+Shift+W").clicked() {
                        self.close_all();
                        ui.close();
                    }
                    if ui.button("Close All BUT Current").clicked() {
                        self.close_all_except_current();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit\tAlt+F4").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // --- EDIT ---
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo\tCtrl+Z").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.undo();
                        }
                        ui.close();
                    }
                    if ui.button("Redo\tCtrl+Y").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.redo();
                        }
                        ui.close();
                    }
                    ui.separator();

                    ui.menu_button("Line Operations", |ui| {
                        if ui.button("Duplicate Current Line\tCtrl+D").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let dup = operations::duplicate_line(&doc.content, cur);
                                doc.set_content(dup);
                            }
                            ui.close();
                        }
                        if ui.button("Remove Duplicate Lines").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, false);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button("Remove Consecutive Duplicate Lines").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, true);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button("Move Line Up").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_up(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line = doc.cursor_line.saturating_sub(1).max(1);
                            }
                            ui.close();
                        }
                        if ui.button("Move Line Down").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_down(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line += 1;
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Sort Lines Ascending").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, true, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button("Sort Lines Descending").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, false, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button("Join Lines\tCtrl+J").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let joined = operations::join_lines(&doc.content);
                                doc.set_content(joined);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Convert Case to", |ui| {
                        if ui.button("UPPERCASE\tCtrl+Shift+U").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Uppercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button("lowercase\tCtrl+U").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Lowercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button("Proper Case").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::ProperCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button("Invert Case").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::InvertCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button("Random Case").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::RandomCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Blank Operations", |ui| {
                        if ui.button("Trim Trailing Space").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_trailing_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button("Trim Leading Space").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_leading_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button("TAB to Space").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::tabs_to_spaces(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button("Space to TAB").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::spaces_to_tabs(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Insert Date Time", |ui| {
                        let now = chrono::Local::now();
                        if ui.button("Date Time (short)").clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button("Date Time (long)").clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("EOL Conversion", |ui| {
                        if ui.button("Windows (CR LF)").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Windows);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Windows;
                            }
                            ui.close();
                        }
                        if ui.button("Unix (LF)").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Unix);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Unix;
                            }
                            ui.close();
                        }
                        if ui.button("Macintosh (CR)").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Mac);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Mac;
                            }
                            ui.close();
                        }
                    });

                    if ui.button("Toggle Comment\tCtrl+Q").clicked() {
                        let prefix = self.current_language.comment_prefix();
                        if let Some(doc) = self.active_doc_mut() {
                            let c = operations::toggle_comment(&doc.content, prefix);
                            doc.set_content(c);
                        }
                        ui.close();
                    }
                });

                // --- SEARCH ---
                ui.menu_button("Search", |ui| {
                    if ui.button("Find...\tCtrl+F").clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Find;
                        ui.close();
                    }
                    if ui.button("Replace...\tCtrl+H").clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Replace;
                        ui.close();
                    }
                    if ui.button("Find Next\tF3").clicked() {
                        self.execute_find_next();
                        ui.close();
                    }
                    if ui.button("Go to Line...\tCtrl+G").clicked() {
                        let max_l = self.active_doc().map(|d| d.line_count()).unwrap_or(1);
                        self.goto_line.is_open = true;
                        self.goto_line.max_lines = max_l;
                        ui.close();
                    }
                });

                // --- VIEW ---
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.word_wrap, "Word Wrap");
                    ui.checkbox(&mut self.show_line_numbers, "Line Numbers");
                    ui.checkbox(&mut self.document_map, "Document Map (Minimap)");
                    ui.separator();
                    if ui.button("Zoom In (Ctrl+Num+)").clicked() {
                        self.font_size = (self.font_size + 2.0).min(36.0);
                        ui.close();
                    }
                    if ui.button("Zoom Out (Ctrl+Num-)").clicked() {
                        self.font_size = (self.font_size - 2.0).max(8.0);
                        ui.close();
                    }
                    if ui.button("Restore Default Zoom").clicked() {
                        self.font_size = 14.0;
                        ui.close();
                    }
                    ui.separator();
                    ui.menu_button("Theme", |ui| {
                        if ui.selectable_label(self.theme == EditorTheme::DarkMode, "Notepad++ Dark Mode").clicked() {
                            self.theme = EditorTheme::DarkMode;
                            self.apply_theme(ctx);
                            self.set_notification("Theme: Notepad++ Dark Mode");
                            ui.close();
                        }
                        if ui.selectable_label(self.theme == EditorTheme::ClassicLight, "Notepad++ Classic Light").clicked() {
                            self.theme = EditorTheme::ClassicLight;
                            self.apply_theme(ctx);
                            self.set_notification("Theme: Notepad++ Classic Light");
                            ui.close();
                        }
                    });
                });

                // --- ENCODING ---
                ui.menu_button("Encoding", |ui| {
                    if ui.button("UTF-8").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8;
                        }
                        ui.close();
                    }
                    if ui.button("UTF-8-BOM").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8Bom;
                        }
                        ui.close();
                    }
                    if ui.button("UTF-16 LE").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Le;
                        }
                        ui.close();
                    }
                    if ui.button("UTF-16 BE").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Be;
                        }
                        ui.close();
                    }
                    if ui.button("ANSI (Windows-1252)").clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Ansi;
                        }
                        ui.close();
                    }
                });

                // --- LANGUAGE ---
                ui.menu_button("Language", |ui| {
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
                ui.menu_button("Plugins", |ui| {
                    ui.menu_button("MIME Tools", |ui| {
                        if ui.button("Base64 Encode").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::base64_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button("Base64 Decode").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::base64_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                        if ui.button("URL Encode").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::url_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button("URL Decode").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::url_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("Converter", |ui| {
                        if ui.button("ASCII to HEX").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let hex = plugins::ascii_to_hex(&doc.content);
                                doc.set_content(hex);
                            }
                            ui.close();
                        }
                        if ui.button("HEX to ASCII").clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(asc) = plugins::hex_to_ascii(&doc.content) {
                                    doc.set_content(asc);
                                }
                            }
                            ui.close();
                        }
                    });
                });

                // --- TOOLS ---
                ui.menu_button("Tools", |ui| {
                    if ui.button("Document Summary").clicked() {
                        self.show_summary();
                        ui.close();
                    }
                });

                // --- HELP (?) ---
                ui.menu_button("?", |ui| {
                    if ui.button("About Notepad++").clicked() {
                        self.about.is_open = true;
                        ui.close();
                    }
                    if ui.button("Notepad++ Home").clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab("https://notepad-plus-plus.org/"));
                        ui.close();
                    }
                });
            });
        });

        // 2. Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📄 New").on_hover_text("New (Ctrl+N)").clicked() {
                    self.new_document();
                }
                if ui.button("📂 Open").on_hover_text("Open (Ctrl+O)").clicked() {
                    self.open_file_dialog();
                }
                if ui.button("💾 Save").on_hover_text("Save (Ctrl+S)").clicked() {
                    self.save_current();
                }
                if ui.button("💾💾 All").on_hover_text("Save All (Ctrl+Shift+S)").clicked() {
                    self.save_all();
                }
                if ui.button("❌ Close").on_hover_text("Close (Ctrl+W)").clicked() {
                    self.close_current();
                }

                ui.separator();

                if ui.button("↩ Undo").on_hover_text("Undo (Ctrl+Z)").clicked() {
                    if let Some(doc) = self.active_doc_mut() {
                        doc.undo();
                    }
                }
                if ui.button("↪ Redo").on_hover_text("Redo (Ctrl+Y)").clicked() {
                    if let Some(doc) = self.active_doc_mut() {
                        doc.redo();
                    }
                }

                ui.separator();

                if ui.button("🔍 Find").on_hover_text("Find (Ctrl+F)").clicked() {
                    self.find_replace.is_open = true;
                    self.find_replace.tab = FindTab::Find;
                }
                if ui.button("🔁 Replace").on_hover_text("Replace (Ctrl+H)").clicked() {
                    self.find_replace.is_open = true;
                    self.find_replace.tab = FindTab::Replace;
                }

                ui.separator();

                if ui.button("➕ Zoom").on_hover_text("Zoom in").clicked() {
                    self.font_size = (self.font_size + 2.0).min(36.0);
                }
                if ui.button("➖ Zoom").on_hover_text("Zoom out").clicked() {
                    self.font_size = (self.font_size - 2.0).max(8.0);
                }
                if ui.selectable_label(self.word_wrap, "↩ Wrap").clicked() {
                    self.word_wrap = !self.word_wrap;
                }
                if ui.button("📊 Summary").on_hover_text("Document Properties").clicked() {
                    self.show_summary();
                }
            });
        });

        // 3. Tab Bar
        if !self.cli_args.no_tab_bar {
            egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut to_close = None;
                            for (idx, doc) in self.docs.iter().enumerate() {
                                let is_active = idx == self.active_index;
                                let marker = if doc.is_dirty { "• " } else { "" };
                                let label = format!("{}{}", marker, doc.title);

                                ui.horizontal(|ui| {
                                    if ui.selectable_label(is_active, label).clicked() {
                                        self.active_index = idx;
                                    }
                                    if ui.small_button("×").on_hover_text("Close tab").clicked() {
                                        to_close = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = to_close {
                                if self.docs.len() > 1 {
                                    self.docs.remove(idx);
                                    if self.active_index >= self.docs.len() {
                                        self.active_index = self.docs.len() - 1;
                                    }
                                } else {
                                    self.docs[0] = Document::new_empty(Some("new 1"));
                                }
                            }

                            if ui.button("+").on_hover_text("New tab").clicked() {
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

                    ui.label(format!("length: {chars}   lines: {lines}   bytes: {bytes}"));
                    ui.separator();
                    ui.label(format!("Ln: {}  Col: {}", doc.cursor_line, doc.cursor_col));
                    ui.separator();
                    ui.label(eol);
                    ui.separator();
                    ui.label(enc);
                    ui.separator();
                    ui.label(self.current_language.name());
                    ui.separator();
                    ui.label(if doc.is_readonly { "READ ONLY" } else { "INS" });
                }

                if let Some((ref msg, _)) = self.status_notification {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(msg).color(Color32::YELLOW));
                    });
                }
            });
        });

        // 5. Central Panel (Editor Canvas)
        egui::CentralPanel::default().show(ctx, |ui| {
            let active_idx = self.active_index;
            if active_idx >= self.docs.len() {
                return;
            }

            let doc = &mut self.docs[active_idx];
            let font = FontId::monospace(self.font_size);

            ui.horizontal(|ui| {
                // Line numbers gutter
                if self.show_line_numbers {
                    let total_lines = doc.line_count();
                    let gutter_text: String = (1..=total_lines)
                        .map(|n| format!("{n}\n"))
                        .collect();

                    ui.vertical(|ui| {
                        ScrollArea::vertical()
                            .id_salt("gutter_scroll")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(gutter_text)
                                            .font(font.clone())
                                            .color(Color32::from_rgb(110, 110, 110)),
                                    )
                                    .selectable(false),
                                );
                            });
                    });
                    ui.separator();
                }

                // Editor Text Area
                ScrollArea::both()
                    .id_salt("editor_scroll")
                    .show(ui, |ui| {
                        let text_edit = egui::TextEdit::multiline(&mut doc.content)
                            .font(font)
                            .desired_width(f32::INFINITY)
                            .desired_rows(35)
                            .lock_focus(true);

                        let response = ui.add(text_edit);

                        if response.changed() {
                            doc.is_dirty = true;
                        }

                        let comment_prefix = doc.language.unwrap_or(self.current_language).comment_prefix();
                        response.context_menu(|ui| {
                            ui.label(RichText::new("Notepad++ Context Menu").strong());
                            ui.separator();
                            if ui.button("Undo\tCtrl+Z").clicked() {
                                doc.undo();
                                ui.close();
                            }
                            if ui.button("Redo\tCtrl+Y").clicked() {
                                doc.redo();
                                ui.close();
                            }
                            ui.separator();
                            ui.menu_button("Line Operations", |ui| {
                                if ui.button("Duplicate Current Line\tCtrl+D").clicked() {
                                    let cur = doc.cursor_line;
                                    let dup = operations::duplicate_line(&doc.content, cur);
                                    doc.set_content(dup);
                                    ui.close();
                                }
                                if ui.button("Move Line Up").clicked() {
                                    let cur = doc.cursor_line;
                                    let moved = operations::move_line_up(&doc.content, cur);
                                    doc.set_content(moved);
                                    doc.cursor_line = doc.cursor_line.saturating_sub(1).max(1);
                                    ui.close();
                                }
                                if ui.button("Move Line Down").clicked() {
                                    let cur = doc.cursor_line;
                                    let moved = operations::move_line_down(&doc.content, cur);
                                    doc.set_content(moved);
                                    doc.cursor_line += 1;
                                    ui.close();
                                }
                                if ui.button("Remove Duplicate Lines").clicked() {
                                    let dedup = operations::remove_duplicate_lines(&doc.content, false);
                                    doc.set_content(dedup);
                                    ui.close();
                                }
                                if ui.button("Sort Lines Ascending").clicked() {
                                    let sorted = operations::sort_lines(&doc.content, true, true, false);
                                    doc.set_content(sorted);
                                    ui.close();
                                }
                            });
                            ui.menu_button("Convert Case", |ui| {
                                if ui.button("UPPERCASE\tCtrl+Shift+U").clicked() {
                                    let c = operations::convert_case(&doc.content, CaseMode::Uppercase);
                                    doc.set_content(c);
                                    ui.close();
                                }
                                if ui.button("lowercase\tCtrl+U").clicked() {
                                    let c = operations::convert_case(&doc.content, CaseMode::Lowercase);
                                    doc.set_content(c);
                                    ui.close();
                                }
                                if ui.button("Proper Case").clicked() {
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
                            ui.menu_button("MIME / Tools", |ui| {
                                if ui.button("Base64 Encode").clicked() {
                                    let enc = plugins::base64_encode(&doc.content);
                                    doc.set_content(enc);
                                    ui.close();
                                }
                                if ui.button("Base64 Decode").clicked() {
                                    if let Ok(dec) = plugins::base64_decode(&doc.content) {
                                        doc.set_content(dec);
                                    }
                                    ui.close();
                                }
                                if ui.button("URL Encode").clicked() {
                                    let enc = plugins::url_encode(&doc.content);
                                    doc.set_content(enc);
                                    ui.close();
                                }
                                if ui.button("URL Decode").clicked() {
                                    if let Ok(dec) = plugins::url_decode(&doc.content) {
                                        doc.set_content(dec);
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });
            });
        });

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

        dialogs::render_about(ctx, &mut self.about);
        dialogs::render_summary(ctx, &mut self.summary);
    }
}
