//! Scrollable tab bar with per-tab context menu.

use crate::app::RustNoteApp;
use crate::i18n::tr;
use crate::ui::{language_icon, tc};
use eframe::egui::{self, Context, RichText, ScrollArea};

impl RustNoteApp {
    pub(crate) fn render_tab_bar(&mut self, ctx: &Context) {
        let light = !self.theme.is_dark();
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
}
