//! Dialog windows: Find & Replace, Go To Line, About, Document Summary.

use crate::search::SearchOptions;
use eframe::egui::{self, Context, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindTab {
    Find,
    Replace,
    FindAll,
}

/// One hit of "Find All in Open Documents".
#[derive(Debug, Clone)]
pub struct FindHit {
    pub doc_index: usize,
    pub doc_title: String,
    pub line: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct FindReplaceState {
    pub is_open: bool,
    pub tab: FindTab,
    pub query: String,
    pub replacement: String,
    pub options: SearchOptions,
    pub status_message: Option<String>,
    pub focus_query: bool,
    pub results: Vec<FindHit>,
}

impl Default for FindReplaceState {
    fn default() -> Self {
        Self {
            is_open: false,
            tab: FindTab::Find,
            query: String::new(),
            replacement: String::new(),
            options: SearchOptions::default(),
            status_message: None,
            focus_query: false,
            results: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GoToLineState {
    pub is_open: bool,
    pub line_input: String,
    pub max_lines: usize,
    pub focus: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AboutState {
    pub is_open: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SummaryState {
    pub is_open: bool,
    pub char_count: usize,
    pub word_count: usize,
    pub line_count: usize,
    pub byte_count: usize,
    pub path: String,
    pub encoding: String,
    pub eol: String,
}

pub enum DialogAction {
    None,
    FindNext,
    FindPrev,
    ReplaceOne,
    ReplaceAll,
    Count,
    FindAllOpen,
    JumpTo(usize, usize, usize),
    GoToLine(usize),
}

/// Renders the Find & Replace tabbed dialog.
pub fn render_find_replace(
    ctx: &Context,
    state: &mut FindReplaceState,
) -> DialogAction {
    if !state.is_open {
        return DialogAction::None;
    }

    let mut action = DialogAction::None;
    let mut is_open = state.is_open;
    let mut close_clicked = false;

    let screen = ctx.content_rect();
    Window::new("Find / Replace")
        .open(&mut is_open)
        .default_pos(egui::pos2((screen.right() - 520.0).max(10.0), screen.top() + 90.0))
        .resizable(true)
        .default_size([480.0, 300.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.tab, FindTab::Find, "Find");
                ui.selectable_value(&mut state.tab, FindTab::Replace, "Replace");
                ui.selectable_value(&mut state.tab, FindTab::FindAll, "Find in Open Documents");
            });
            ui.separator();

            let mut enter_pressed = false;
            egui::Grid::new("find_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Find what:");
                let r = ui.add(egui::TextEdit::singleline(&mut state.query).desired_width(300.0));
                if state.focus_query {
                    r.request_focus();
                    state.focus_query = false;
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    enter_pressed = true;
                    r.request_focus();
                }
                ui.end_row();
                if state.tab == FindTab::Replace {
                    ui.label("Replace with:");
                    ui.add(egui::TextEdit::singleline(&mut state.replacement).desired_width(300.0));
                    ui.end_row();
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut state.options.match_case, "Match case");
                    ui.checkbox(&mut state.options.whole_word, "Match whole word only");
                    ui.checkbox(&mut state.options.use_regex, "Regular expression");
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.radio_value(&mut state.options.search_down, true, "Down");
                    ui.radio_value(&mut state.options.search_down, false, "Up");
                });
            });

            ui.add_space(6.0);
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                if ui.button("Find Next").clicked() || (enter_pressed && state.tab != FindTab::FindAll) {
                    action = DialogAction::FindNext;
                }
                if ui.button("Find Previous").clicked() {
                    action = DialogAction::FindPrev;
                }
                if ui.button("Count").clicked() {
                    action = DialogAction::Count;
                }
                if state.tab == FindTab::Replace {
                    if ui.button("Replace").clicked() {
                        action = DialogAction::ReplaceOne;
                    }
                    if ui.button("Replace All").clicked() {
                        action = DialogAction::ReplaceAll;
                    }
                }
                if state.tab == FindTab::FindAll
                    && (ui.button("Find All").clicked() || enter_pressed)
                {
                    action = DialogAction::FindAllOpen;
                }
                if ui.button("Close").clicked() {
                    close_clicked = true;
                }
            });

            if let Some(ref msg) = state.status_message {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(msg).color(egui::Color32::LIGHT_BLUE));
            }

            if state.tab == FindTab::FindAll && !state.results.is_empty() {
                ui.separator();
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for hit in &state.results {
                        let text = format!("{}  (line {}):  {}", hit.doc_title, hit.line, hit.preview);
                        if ui.selectable_label(false, egui::RichText::new(text).monospace()).clicked() {
                            action = DialogAction::JumpTo(hit.doc_index, hit.byte_start, hit.byte_end);
                        }
                    }
                });
            }
        });

    state.is_open = is_open && !close_clicked;
    action
}

/// Renders the Go To Line dialog (Ctrl+G).
pub fn render_go_to_line(
    ctx: &Context,
    state: &mut GoToLineState,
) -> DialogAction {
    if !state.is_open {
        return DialogAction::None;
    }

    let mut action = DialogAction::None;
    let mut open_window = true;
    let mut close_requested = false;

    Window::new("Go to Line")
        .open(&mut open_window)
        .collapsible(false)
        .resizable(false)
        .default_size([280.0, 140.0])
        .show(ctx, |ui| {
            ui.label(format!("Line number (1 - {}):", state.max_lines.max(1)));
            let response = ui.text_edit_singleline(&mut state.line_input);
            if state.focus {
                response.request_focus();
                state.focus = false;
            }
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(num) = state.line_input.trim().parse::<usize>() {
                    action = DialogAction::GoToLine(num);
                    close_requested = true;
                }
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Go to Line").clicked() {
                    if let Ok(num) = state.line_input.trim().parse::<usize>() {
                        action = DialogAction::GoToLine(num);
                        close_requested = true;
                    }
                }
                if ui.button("Cancel").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open_window || close_requested {
        state.is_open = false;
    }
    action
}

/// Renders the About rust-note dialog.
pub fn render_about(ctx: &Context, state: &mut AboutState) {
    if !state.is_open {
        return;
    }

    let mut open_window = true;
    let mut close_requested = false;

    Window::new("About rust-note")
        .open(&mut open_window)
        .resizable(false)
        .default_size([420.0, 220.0])
        .show(ctx, |ui| {
            ui.heading("rust-note v0.1.0");
            ui.label("A modern, elegant, and high-performance code editor built with Rust and egui.");
            ui.separator();
            ui.label("Author: Laurent VOLFF");
            ui.label("Engine Architecture: High-performance multi-tab text buffer");
            ui.separator();
            ui.hyperlink_to("Source & Docs in D:\\GIT\\rust-note", "https://github.com/rust-note/rust-note");
            ui.add_space(8.0);
            if ui.button("OK").clicked() {
                close_requested = true;
            }
        });

    if !open_window || close_requested {
        state.is_open = false;
    }
}

/// Renders Document Summary modal.
pub fn render_summary(ctx: &Context, state: &mut SummaryState) {
    if !state.is_open {
        return;
    }

    let mut open_window = true;
    let mut close_requested = false;

    Window::new("Document Summary")
        .open(&mut open_window)
        .resizable(false)
        .default_size([380.0, 240.0])
        .show(ctx, |ui| {
            ui.heading("Current Document Properties");
            ui.separator();
            ui.label(format!("File Path: {}", state.path));
            ui.label(format!("Encoding:  {}", state.encoding));
            ui.label(format!("EOL:       {}", state.eol));
            ui.separator();
            ui.label(format!("Total Characters: {}", state.char_count));
            ui.label(format!("Total Words:      {}", state.word_count));
            ui.label(format!("Total Lines:      {}", state.line_count));
            ui.label(format!("Total Bytes:      {} bytes", state.byte_count));
            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                close_requested = true;
            }
        });

    if !open_window || close_requested {
        state.is_open = false;
    }
}
