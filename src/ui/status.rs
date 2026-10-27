//! Bottom status bar (cursor position, counts, EOL, encoding, language, notifications).

use crate::app::RustNoteApp;
use crate::i18n::tr;
use crate::ui::{language_icon, tc};
use eframe::egui::{self, Layout, RichText, Ui};

impl RustNoteApp {
    pub(crate) fn render_status_bar(&mut self, ui: &mut Ui) {
        let light = !self.theme.is_dark();
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(doc) = self.active_doc() {
                    let (lines, chars, bytes) =
                        (doc.line_count(), doc.char_count(), doc.byte_len());
                    let enc = doc.encoding.to_string();
                    let eol = doc.eol.to_string();
                    let icon = language_icon(doc.language);

                    ui.label(
                        RichText::new(format!(
                            "{} {}, {} {}",
                            tr("Ln"),
                            doc.cursor_line,
                            tr("Col"),
                            doc.cursor_col
                        ))
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
                        RichText::new(format!(
                            "📊 {chars} {} | {lines} {} | {bytes} {}",
                            tr("chars"),
                            tr("lines"),
                            tr("B")
                        ))
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
                        ui.label(
                            RichText::new(tr("🔒 READ ONLY"))
                                .strong()
                                .color(tc(light, 243, 139, 168)),
                        );
                    } else {
                        ui.label(
                            RichText::new(tr("✏ INS"))
                                .strong()
                                .color(tc(light, 166, 227, 161)),
                        );
                    }
                }

                if let Some((ref msg, _)) = self.status_notification {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("🔔 {msg}"))
                                .strong()
                                .color(tc(light, 250, 179, 135)),
                        );
                    });
                }
            });
        });
    }
}
