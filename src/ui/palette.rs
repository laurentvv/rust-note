//! Command palette (Ctrl+Shift+P): fuzzy-searchable access to every action,
//! in the spirit of modern minimal editors.

use crate::app::RustNoteApp;
use crate::buffer::EolFormat;
use crate::editor::operations::{self, CaseMode};
use crate::i18n::tr;
use crate::plugins;
use crate::ui::themes::EditorTheme;
use eframe::egui::{self, Context, RichText, ScrollArea};

/// Palette UI state.
#[derive(Debug, Clone, Default)]
pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected: usize,
}

/// Identifiers of the actions reachable from the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    NewDoc,
    OpenFile,
    Save,
    SaveAs,
    SaveAll,
    CloseTab,
    CloseAll,
    RestoreClosedTab,
    ExportHtml,
    ExportRtf,
    Exit,
    Undo,
    Redo,
    DuplicateLine,
    MoveLineUp,
    MoveLineDown,
    SortLinesAsc,
    SortLinesDesc,
    JoinLines,
    ToggleComment,
    UpperCase,
    LowerCase,
    ProperCase,
    TrimTrailing,
    TrimLeading,
    TabToSpace,
    SpaceToTab,
    RemoveDuplicates,
    DateTimeShort,
    DateTimeLong,
    EolWindows,
    EolUnix,
    EolMac,
    Find,
    Replace,
    FindNext,
    FindPrev,
    GoToLine,
    Count,
    FindAllOpen,
    WordWrap,
    LineNumbers,
    FunctionList,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ThemeDarkMode,
    ThemeClassicLight,
    ThemeMonokai,
    ThemeSolarizedDark,
    ThemeSolarizedLight,
    ThemeZenburn,
    ThemeFluentDark,
    ThemeFluentLight,
    FollowWindowsTheme,
    Base64Encode,
    Base64Decode,
    UrlEncode,
    UrlDecode,
    AsciiToHex,
    HexToAscii,
    DecToHex,
    HexToDec,
    ConverterPanel,
    DocumentSummary,
    TrimAndSave,
    OpenInNewInstance,
    KeyboardShortcuts,
    About,
}

struct PaletteCommand {
    id: Cmd,
    /// English label key, translated with `tr` at render time.
    label: &'static str,
    shortcut: &'static str,
}

const COMMANDS: &[PaletteCommand] = &[
    PaletteCommand { id: Cmd::NewDoc, label: "New", shortcut: "Ctrl+N" },
    PaletteCommand { id: Cmd::OpenFile, label: "Open...", shortcut: "Ctrl+O" },
    PaletteCommand { id: Cmd::Save, label: "Save", shortcut: "Ctrl+S" },
    PaletteCommand { id: Cmd::SaveAs, label: "Save As...", shortcut: "Ctrl+Alt+S" },
    PaletteCommand { id: Cmd::SaveAll, label: "Save All", shortcut: "Ctrl+Shift+S" },
    PaletteCommand { id: Cmd::CloseTab, label: "Close", shortcut: "Ctrl+W" },
    PaletteCommand { id: Cmd::CloseAll, label: "Close All", shortcut: "Ctrl+Shift+W" },
    PaletteCommand { id: Cmd::RestoreClosedTab, label: "Restore Closed Tab", shortcut: "Ctrl+Shift+T" },
    PaletteCommand { id: Cmd::ExportHtml, label: "Export to HTML...", shortcut: "" },
    PaletteCommand { id: Cmd::ExportRtf, label: "Export to RTF (Rich Text)...", shortcut: "" },
    PaletteCommand { id: Cmd::Exit, label: "Exit", shortcut: "Alt+F4" },
    PaletteCommand { id: Cmd::Undo, label: "Undo", shortcut: "Ctrl+Z" },
    PaletteCommand { id: Cmd::Redo, label: "Redo", shortcut: "Ctrl+Y" },
    PaletteCommand { id: Cmd::DuplicateLine, label: "Duplicate Current Line", shortcut: "Ctrl+D" },
    PaletteCommand { id: Cmd::MoveLineUp, label: "Move Line Up", shortcut: "" },
    PaletteCommand { id: Cmd::MoveLineDown, label: "Move Line Down", shortcut: "" },
    PaletteCommand { id: Cmd::SortLinesAsc, label: "Sort Lines Ascending", shortcut: "" },
    PaletteCommand { id: Cmd::SortLinesDesc, label: "Sort Lines Descending", shortcut: "" },
    PaletteCommand { id: Cmd::JoinLines, label: "Join Lines", shortcut: "Ctrl+J" },
    PaletteCommand { id: Cmd::ToggleComment, label: "Toggle Comment", shortcut: "Ctrl+Q" },
    PaletteCommand { id: Cmd::UpperCase, label: "UPPERCASE", shortcut: "Ctrl+Shift+U" },
    PaletteCommand { id: Cmd::LowerCase, label: "lowercase", shortcut: "Ctrl+U" },
    PaletteCommand { id: Cmd::ProperCase, label: "Proper Case", shortcut: "" },
    PaletteCommand { id: Cmd::TrimTrailing, label: "Trim Trailing Space", shortcut: "" },
    PaletteCommand { id: Cmd::TrimLeading, label: "Trim Leading Space", shortcut: "" },
    PaletteCommand { id: Cmd::TabToSpace, label: "TAB to Space", shortcut: "" },
    PaletteCommand { id: Cmd::SpaceToTab, label: "Space to TAB", shortcut: "" },
    PaletteCommand { id: Cmd::RemoveDuplicates, label: "Remove Duplicate Lines", shortcut: "" },
    PaletteCommand { id: Cmd::DateTimeShort, label: "Date Time (short)", shortcut: "" },
    PaletteCommand { id: Cmd::DateTimeLong, label: "Date Time (long)", shortcut: "" },
    PaletteCommand { id: Cmd::EolWindows, label: "Windows (CR LF)", shortcut: "" },
    PaletteCommand { id: Cmd::EolUnix, label: "Unix (LF)", shortcut: "" },
    PaletteCommand { id: Cmd::EolMac, label: "Macintosh (CR)", shortcut: "" },
    PaletteCommand { id: Cmd::Find, label: "Find...", shortcut: "Ctrl+F" },
    PaletteCommand { id: Cmd::Replace, label: "Replace...", shortcut: "Ctrl+H" },
    PaletteCommand { id: Cmd::FindNext, label: "Find Next", shortcut: "F3" },
    PaletteCommand { id: Cmd::FindPrev, label: "Find Previous", shortcut: "Shift+F3" },
    PaletteCommand { id: Cmd::GoToLine, label: "Go to Line...", shortcut: "Ctrl+G" },
    PaletteCommand { id: Cmd::Count, label: "Count", shortcut: "" },
    PaletteCommand { id: Cmd::FindAllOpen, label: "Find in Open Documents", shortcut: "" },
    PaletteCommand { id: Cmd::WordWrap, label: "Word Wrap", shortcut: "" },
    PaletteCommand { id: Cmd::LineNumbers, label: "Line Numbers", shortcut: "" },
    PaletteCommand { id: Cmd::FunctionList, label: "Function List", shortcut: "Ctrl+Alt+L" },
    PaletteCommand { id: Cmd::ZoomIn, label: "Zoom In", shortcut: "" },
    PaletteCommand { id: Cmd::ZoomOut, label: "Zoom Out", shortcut: "" },
    PaletteCommand { id: Cmd::ZoomReset, label: "Restore Default Zoom", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeDarkMode, label: "Theme: Obsidian Dark (default)", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeClassicLight, label: "Theme: Classic Light", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeMonokai, label: "Theme: Monokai", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeSolarizedDark, label: "Theme: Solarized Dark", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeSolarizedLight, label: "Theme: Solarized Light", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeZenburn, label: "Theme: Zenburn", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeFluentDark, label: "Theme: Fluent Dark (Windows 11)", shortcut: "" },
    PaletteCommand { id: Cmd::ThemeFluentLight, label: "Theme: Fluent Light (Windows 11)", shortcut: "" },
    PaletteCommand { id: Cmd::FollowWindowsTheme, label: "Follow Windows Theme", shortcut: "" },
    PaletteCommand { id: Cmd::Base64Encode, label: "Base64 Encode", shortcut: "" },
    PaletteCommand { id: Cmd::Base64Decode, label: "Base64 Decode", shortcut: "" },
    PaletteCommand { id: Cmd::UrlEncode, label: "URL Encode", shortcut: "" },
    PaletteCommand { id: Cmd::UrlDecode, label: "URL Decode", shortcut: "" },
    PaletteCommand { id: Cmd::AsciiToHex, label: "ASCII to HEX", shortcut: "" },
    PaletteCommand { id: Cmd::HexToAscii, label: "HEX to ASCII", shortcut: "" },
    PaletteCommand { id: Cmd::DecToHex, label: "DEC to HEX (0x...)", shortcut: "" },
    PaletteCommand { id: Cmd::HexToDec, label: "HEX to DEC", shortcut: "" },
    PaletteCommand { id: Cmd::ConverterPanel, label: "0x Quick Conversion Panel...", shortcut: "" },
    PaletteCommand { id: Cmd::DocumentSummary, label: "Document Summary", shortcut: "" },
    PaletteCommand { id: Cmd::TrimAndSave, label: "Trim Trailing Space and Save", shortcut: "Alt+Shift+S" },
    PaletteCommand { id: Cmd::OpenInNewInstance, label: "Open in New Instance", shortcut: "Alt+F6" },
    PaletteCommand { id: Cmd::KeyboardShortcuts, label: "Keyboard Shortcuts", shortcut: "" },
    PaletteCommand { id: Cmd::About, label: "About rust-note", shortcut: "" },
];

/// Case-insensitive fuzzy score: every query char must appear in order.
/// Bonus for consecutive matches and word boundaries; `None` when it does not match.
#[must_use]
pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let target: Vec<char> = target.chars().collect();
    let mut score = 0i32;
    let mut ti = 0usize;
    let mut last_hit: Option<usize> = None;
    for qc in query.chars() {
        let qc = qc.to_lowercase().next().unwrap_or(qc);
        let mut found = None;
        while ti < target.len() {
            let tc = target[ti].to_lowercase().next().unwrap_or(target[ti]);
            ti += 1;
            if tc == qc {
                found = Some(ti - 1);
                break;
            }
        }
        let hit = found?;
        score += 1;
        if let Some(prev) = last_hit {
            if hit == prev + 1 {
                score += 10; // consecutive run
            }
        }
        let boundary = hit == 0
            || target[hit - 1] == ' '
            || target[hit - 1] == ':'
            || target[hit - 1] == '('
            || target[hit - 1].is_whitespace();
        if boundary {
            score += 15; // word start
        }
        last_hit = Some(hit);
    }
    // Prefer shorter targets for the same match quality
    score -= (target.len() as i32) / 16;
    Some(score)
}

impl RustNoteApp {
    /// Opens the palette with a clean filter.
    pub fn open_command_palette(&mut self) {
        self.palette.query.clear();
        self.palette.selected = 0;
        self.palette.is_open = true;
    }

    /// Executes a palette command.
    fn run_command(&mut self, cmd: Cmd, ctx: &Context) {
        match cmd {
            Cmd::NewDoc => self.new_document(),
            Cmd::OpenFile => self.open_file_dialog(),
            Cmd::Save => self.save_current(),
            Cmd::SaveAs => self.save_as_current(),
            Cmd::SaveAll => self.save_all(),
            Cmd::CloseTab => self.close_current(),
            Cmd::CloseAll => self.close_all(),
            Cmd::RestoreClosedTab => self.restore_closed_document(),
            Cmd::ExportHtml => self.export_current_to_html(),
            Cmd::ExportRtf => self.export_current_to_rtf(),
            Cmd::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Cmd::Undo => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.undo();
                }
            }
            Cmd::Redo => {
                if let Some(doc) = self.active_doc_mut() {
                    doc.redo();
                }
            }
            Cmd::DuplicateLine => {
                self.apply_op(|d| operations::duplicate_line(&d.content, d.cursor_line));
            }
            Cmd::MoveLineUp => {
                self.apply_op(|d| operations::move_line_up(&d.content, d.cursor_line));
            }
            Cmd::MoveLineDown => {
                self.apply_op(|d| operations::move_line_down(&d.content, d.cursor_line));
            }
            Cmd::SortLinesAsc => {
                self.apply_op(|d| operations::sort_lines(&d.content, true, true, false));
            }
            Cmd::SortLinesDesc => {
                self.apply_op(|d| operations::sort_lines(&d.content, false, true, false));
            }
            Cmd::JoinLines => self.apply_op(|d| operations::join_lines(&d.content)),
            Cmd::ToggleComment => {
                let prefix = self.current_language.comment_prefix();
                self.apply_op(move |d| operations::toggle_comment(&d.content, prefix));
            }
            Cmd::UpperCase => {
                self.apply_op(|d| operations::convert_case(&d.content, CaseMode::Uppercase));
            }
            Cmd::LowerCase => {
                self.apply_op(|d| operations::convert_case(&d.content, CaseMode::Lowercase));
            }
            Cmd::ProperCase => {
                self.apply_op(|d| operations::convert_case(&d.content, CaseMode::ProperCase));
            }
            Cmd::TrimTrailing => self.apply_op(|d| operations::trim_trailing_spaces(&d.content)),
            Cmd::TrimLeading => self.apply_op(|d| operations::trim_leading_spaces(&d.content)),
            Cmd::TabToSpace => self.apply_op(|d| operations::tabs_to_spaces(&d.content, 4)),
            Cmd::SpaceToTab => self.apply_op(|d| operations::spaces_to_tabs(&d.content, 4)),
            Cmd::RemoveDuplicates => {
                self.apply_op(|d| operations::remove_duplicate_lines(&d.content, false));
            }
            Cmd::DateTimeShort | Cmd::DateTimeLong => {
                let now = chrono::Local::now();
                let stamp = if cmd == Cmd::DateTimeShort {
                    now.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    now.format("%Y-%m-%d %H:%M:%S").to_string()
                };
                if let Some(doc) = self.active_doc_mut() {
                    let mut c = doc.content.clone();
                    c.push_str(&stamp);
                    doc.set_content(c);
                }
            }
            Cmd::EolWindows | Cmd::EolUnix | Cmd::EolMac => {
                let fmt = match cmd {
                    Cmd::EolWindows => EolFormat::Windows,
                    Cmd::EolUnix => EolFormat::Unix,
                    _ => EolFormat::Mac,
                };
                if let Some(doc) = self.active_doc_mut() {
                    let converted = crate::buffer::eol::convert(&doc.content, fmt);
                    doc.set_content(converted);
                    doc.eol = fmt;
                }
            }
            Cmd::Find => {
                let _ = self.open_find(crate::ui::dialogs::FindTab::Find);
                self.palette.is_open = false;
                return;
            }
            Cmd::Replace => {
                let _ = self.open_find(crate::ui::dialogs::FindTab::Replace);
                self.palette.is_open = false;
                return;
            }
            Cmd::FindNext => {
                self.execute_find(true);
            }
            Cmd::FindPrev => {
                self.execute_find(false);
            }
            Cmd::GoToLine => {
                let max_l = self.active_doc().map_or(1, |d| d.line_count());
                self.goto_line.is_open = true;
                self.goto_line.max_lines = max_l;
                self.goto_line.focus = true;
                self.palette.is_open = false;
                return;
            }
            Cmd::Count => self.execute_count(),
            Cmd::FindAllOpen => self.execute_find_all_open(),
            Cmd::WordWrap => self.word_wrap = !self.word_wrap,
            Cmd::LineNumbers => self.show_line_numbers = !self.show_line_numbers,
            Cmd::FunctionList => self.show_function_list = !self.show_function_list,
            Cmd::ZoomIn => self.zoom(2.0),
            Cmd::ZoomOut => self.zoom(-2.0),
            Cmd::ZoomReset => self.font_size = 14.0,
            Cmd::ThemeDarkMode => self.set_theme(EditorTheme::DarkMode, ctx),
            Cmd::ThemeClassicLight => self.set_theme(EditorTheme::ClassicLight, ctx),
            Cmd::ThemeMonokai => self.set_theme(EditorTheme::Monokai, ctx),
            Cmd::ThemeSolarizedDark => self.set_theme(EditorTheme::SolarizedDark, ctx),
            Cmd::ThemeSolarizedLight => self.set_theme(EditorTheme::SolarizedLight, ctx),
            Cmd::ThemeZenburn => self.set_theme(EditorTheme::Zenburn, ctx),
            Cmd::ThemeFluentDark => self.set_theme(EditorTheme::FluentDark, ctx),
            Cmd::ThemeFluentLight => self.set_theme(EditorTheme::FluentLight, ctx),
            Cmd::FollowWindowsTheme => {
                self.settings.follow_windows_theme = !self.settings.follow_windows_theme;
                if self.settings.follow_windows_theme {
                    self.sync_theme_with_windows(ctx);
                }
            }
            Cmd::Base64Encode => self.apply_op(|d| plugins::base64_encode(&d.content)),
            Cmd::Base64Decode => {
                if let Some(doc) = self.active_doc() {
                    if let Ok(dec) = plugins::base64_decode(&doc.content) {
                        self.apply_op(|_| dec);
                    }
                }
            }
            Cmd::UrlEncode => self.apply_op(|d| plugins::url_encode(&d.content)),
            Cmd::UrlDecode => {
                if let Some(doc) = self.active_doc() {
                    if let Ok(dec) = plugins::url_decode(&doc.content) {
                        self.apply_op(|_| dec);
                    }
                }
            }
            Cmd::AsciiToHex => self.apply_op(|d| plugins::ascii_to_hex(&d.content)),
            Cmd::HexToAscii => {
                if let Some(doc) = self.active_doc() {
                    if let Ok(asc) = plugins::hex_to_ascii(&doc.content) {
                        self.apply_op(|_| asc);
                    }
                }
            }
            Cmd::DecToHex => {
                if let Some(doc) = self.active_doc() {
                    if let Ok(hex) = plugins::dec_to_hex(&doc.content) {
                        self.apply_op(|_| hex);
                    }
                }
            }
            Cmd::HexToDec => {
                if let Some(doc) = self.active_doc() {
                    if let Ok(dec) = plugins::hex_to_dec(&doc.content) {
                        self.apply_op(|_| dec);
                    }
                }
            }
            Cmd::ConverterPanel => self.conversion_panel.is_open = true,
            Cmd::DocumentSummary => self.show_summary(),
            Cmd::TrimAndSave => self.trim_and_save(),
            Cmd::OpenInNewInstance => self.open_in_new_instance(),
            Cmd::KeyboardShortcuts => self.show_shortcuts = true,
            Cmd::About => self.about.is_open = true,
        }
        self.palette.is_open = false;
        self.palette.query.clear();
    }

    /// Applies a theme and notifies the user.
    pub fn set_theme(&mut self, theme: EditorTheme, ctx: &Context) {
        self.theme = theme;
        self.apply_theme(ctx);
        self.set_notification(format!("Theme: {}", theme.name()));
    }

    /// Picks Fluent Light/Dark from the Windows "apps" theme when available.
    pub fn sync_theme_with_windows(&mut self, ctx: &Context) {
        let Some(light) = crate::ui::fonts::windows_apps_light() else {
            return;
        };
        let wanted = if light { EditorTheme::FluentLight } else { EditorTheme::FluentDark };
        if self.theme != wanted {
            self.set_theme(wanted, ctx);
        }
    }

    pub(crate) fn render_command_palette(&mut self, ctx: &Context) {
        if !self.palette.is_open {
            return;
        }
        let mut open = self.palette.is_open;
        let mut run: Option<Cmd> = None;

        // Filter (on the translated labels, so French queries match French commands)
        let query = self.palette.query.to_lowercase();
        let mut filtered: Vec<(usize, i32)> = COMMANDS
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if query.is_empty() {
                    Some((i, 0))
                } else {
                    fuzzy_score(&query, &tr(c.label).to_lowercase()).map(|s| (i, s))
                }
            })
            .collect();
        if !query.is_empty() {
            filtered.sort_by_key(|(_, score)| -*score);
        }
        if self.palette.selected >= filtered.len() {
            self.palette.selected = filtered.len().saturating_sub(1);
        }

        egui::Window::new(tr("Command Palette"))
            .open(&mut open)
            .resizable(false)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 48.0])
            .show(ctx, |ui| {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut self.palette.query)
                        .hint_text(tr("Type a command..."))
                        .desired_width(f32::INFINITY),
                );
                edit.request_focus();

                ui.separator();

                if filtered.is_empty() {
                    ui.label(RichText::new(tr("No matching command")).weak());
                } else {
                    ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        for (row, (idx, _)) in filtered.iter().enumerate() {
                            let cmd = &COMMANDS[*idx];
                            let is_selected = row == self.palette.selected;
                            let label = if cmd.shortcut.is_empty() {
                                tr(cmd.label)
                            } else {
                                format!("{}\t{}", tr(cmd.label), cmd.shortcut)
                            };
                            let text = RichText::new(label);
                            let resp = ui.selectable_label(is_selected, if is_selected { text.strong() } else { text });
                            if resp.clicked() {
                                run = Some(cmd.id);
                            }
                        }
                    });
                }

                ui.separator();
                ui.label(
                    RichText::new(format!("{}  ↑↓   Enter   Esc", tr("Navigate"))).weak().small(),
                );
            });

        // Keyboard navigation (consumed after the TextEdit had its chance)
        ctx.input_mut(|i| {
            use egui::{Key, Modifiers};
            if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                self.palette.selected = (self.palette.selected + 1).min(filtered.len().saturating_sub(1));
            }
            if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                self.palette.selected = self.palette.selected.saturating_sub(1);
            }
            if i.consume_key(Modifiers::NONE, Key::Enter) && !filtered.is_empty() {
                if let Some((idx, _)) = filtered.get(self.palette.selected) {
                    run = Some(COMMANDS[*idx].id);
                }
            }
            if i.consume_key(Modifiers::NONE, Key::Escape) {
                self.palette.is_open = false;
            }
        });

        if let Some(cmd) = run {
            ctx.input_mut(|i| i.events.clear());
            self.run_command(cmd, ctx);
        }
        if !open {
            self.palette.is_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_subsequence_with_boundaries_first() {
        assert!(fuzzy_score("stl", "Sort Lines Ascending").is_some());
        assert!(fuzzy_score("stl", "Sort Lines Ascending").unwrap() > fuzzy_score("stl", "Best Lines").unwrap_or(0));
        assert!(fuzzy_score("savea", "Save All").is_some());
        assert!(fuzzy_score("xyz", "Save All").is_none());
        assert_eq!(fuzzy_score("", "anything"), Some(0));
        // Word-start bonus: "w" should rank Word Wrap over "New"
        assert!(fuzzy_score("w", "Word Wrap").unwrap() > fuzzy_score("w", "New").unwrap());
    }

    #[test]
    fn every_command_has_a_translation_key() {
        crate::i18n::set_lang(crate::i18n::UiLang::French);
        let untranslated: Vec<&str> = COMMANDS
            .iter()
            .map(|c| c.label)
            .filter(|l| tr(l) == *l && !matches!(*l, "lowercase" | "UPPERCASE" | "Windows (CR LF)" | "Unix (LF)" | "Macintosh (CR)"))
            .collect();
        crate::i18n::set_lang(crate::i18n::UiLang::English);
        assert!(untranslated.is_empty(), "untranslated palette labels: {untranslated:#?}");
    }
}
