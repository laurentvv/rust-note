//! Toolbar strip below the menu bar (Fluent icons, minimal footprint).

use crate::app::RustNoteApp;
use crate::i18n::tr;
use crate::ui::dialogs::FindTab;
use crate::ui::fonts::{glyph, icon};
use crate::ui::themes::EditorTheme;
use eframe::egui::{self, Context, Layout, Ui};

impl RustNoteApp {
    pub(crate) fn render_toolbar(&mut self, ui: &mut Ui) {
        let ctx: Context = ui.ctx().clone();
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // Group 1: Files
                ui.horizontal(|ui| {
                    if ui.button(icon(glyph::ADD)).on_hover_text(tr("New Document (Ctrl+N)")).clicked() {
                        self.new_document();
                    }
                    if ui.button(icon(glyph::OPEN_FILE)).on_hover_text(tr("Open File (Ctrl+O)")).clicked() {
                        self.open_file_dialog();
                    }
                    if ui.button(icon(glyph::SAVE)).on_hover_text(tr("Save (Ctrl+S)")).clicked() {
                        self.save_current();
                    }
                    if ui.button(icon(glyph::CANCEL)).on_hover_text(tr("Close tab (Ctrl+W)")).clicked() {
                        self.close_current();
                    }
                });

                ui.separator();

                // Group 2: History
                ui.horizontal(|ui| {
                    if ui.button(icon(glyph::UNDO)).on_hover_text(tr("Undo (Ctrl+Z)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.undo();
                        }
                    }
                    if ui.button(icon(glyph::REDO)).on_hover_text(tr("Redo (Ctrl+Y)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.redo();
                        }
                    }
                });

                ui.separator();

                // Group 3: Search
                ui.horizontal(|ui| {
                    if ui.button(icon(glyph::SEARCH)).on_hover_text(tr("Find (Ctrl+F)")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Find;
                    }
                    if ui.button(icon(glyph::SYNC)).on_hover_text(tr("Replace (Ctrl+H)")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Replace;
                    }
                });

                ui.separator();

                // Group 4: Tools & Converters (from NppConverter.dll)
                ui.horizontal(|ui| {
                    let conv = ui.selectable_label(
                        self.conversion_panel.is_open,
                        icon(glyph::SETTINGS),
                    ).on_hover_text(tr("Real-Time Multi-Base Quick Converter"));
                    if conv.clicked() {
                        self.conversion_panel.is_open = !self.conversion_panel.is_open;
                    }
                    let syms = ui.selectable_label(
                        self.show_function_list,
                        icon(glyph::LIST),
                    ).on_hover_text(tr("Function List Outline (Ctrl+Alt+L)"));
                    if syms.clicked() {
                        self.show_function_list = !self.show_function_list;
                    }
                    if ui.button(icon(glyph::INFO)).on_hover_text(tr("Document Properties")).clicked() {
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
                    if ui.selectable_label(self.word_wrap, tr("↩ Wrap")).on_hover_text(tr("Word Wrap")).clicked() {
                        self.word_wrap = !self.word_wrap;
                    }
                    if ui.button(icon(glyph::COMMAND)).on_hover_text(tr("Command Palette (Ctrl+Shift+P)")).clicked() {
                        self.open_command_palette();
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
                        })
                        .response
                        .on_hover_text(tr("Theme"));
                    if chosen != self.theme {
                        self.set_theme(chosen, &ctx);
                    }
                });
            });
        });
    }
}
