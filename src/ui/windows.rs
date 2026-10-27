//! Floating windows: Find/Replace, Go to line, Keyboard Shortcuts, About, Summary
//! and the 0x Quick Conversion panel.

use crate::app::RustNoteApp;
use crate::i18n::{self, tr, UiLang};
use crate::ui::dialogs::{self, DialogAction};
use eframe::egui::{self, Context, RichText};

/// Translates the descriptive words of a shortcut hint ("Double-click / Triple-click"...).
fn tr_keys(keys: &str) -> String {
    if i18n::lang() == UiLang::English {
        return keys.to_string();
    }
    keys.replace("Double-click", "Double-clic")
        .replace("Triple-click", "Triple-clic")
        .replace("Shift+click", "Maj+clic")
        .replace("Wheel", "Molette")
        .replace("type 2+ letters", "saisir 1+ lettre")
        .replace(" or ", " ou ")
        .replace("Up, Down", "Haut, Bas")
        .replace("Down", "Bas")
        .replace("Up", "Haut")
        .replace("Left", "Gauche")
        .replace("Right", "Droite")
        .replace("Shift", "Maj")
        .replace("Plus / Minus", "Plus / Moins")
        .replace("arrows", "flèches")
        .replace("drag", "glisser")
}

impl RustNoteApp {
    pub(crate) fn render_dialogs(&mut self, ctx: &Context) {
        // Find / Replace
        let action = dialogs::render_find_replace(ctx, &mut self.find_replace);
        if !matches!(action, DialogAction::None) {
            // The dialog shows the result message on the next frame: make sure one is drawn
            ctx.request_repaint();
        }
        match action {
            DialogAction::FindNext => self.execute_find_next(),
            DialogAction::ReplaceAll => self.execute_replace_all(),
            DialogAction::ReplaceOne => self.execute_replace_one(),
            DialogAction::FindPrev => {
                self.execute_find(false);
            }
            DialogAction::Count => self.execute_count(),
            DialogAction::FindAllOpen => self.execute_find_all_open(),
            DialogAction::JumpTo(d, s, e) => self.jump_to_hit(d, s, e),
            _ => {}
        }

        // Go to line
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

        // Keyboard shortcuts
        if self.show_shortcuts {
            let mut open = true;
            egui::Window::new(tr("Keyboard Shortcuts")).open(&mut open).resizable(false).show(ctx, |ui| {
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
                        ui.label(tr(what));
                        ui.label(RichText::new(tr_keys(keys)).monospace().strong());
                        ui.end_row();
                    }
                });
            });
            self.show_shortcuts = open;
        }

        dialogs::render_about(ctx, &mut self.about);
        dialogs::render_summary(ctx, &mut self.summary);

        // 0x Quick Conversion Panel (recovered from NppConverter.dll)
        if self.conversion_panel.is_open {
            let mut is_open = true;
            egui::Window::new(tr("🧮 0x Quick Converter"))
                .open(&mut is_open)
                .resizable(false)
                .default_size([380.0, 240.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new(tr("Bidirectional Real-Time Multi-Base Converter")).strong());
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
