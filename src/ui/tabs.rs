//! Scrollable tab bar with Windows 11 rounded tabs, pinning and context menu.

use crate::app::RustNoteApp;
use crate::i18n::tr;
use crate::ui::fonts::{glyph, icon};
use crate::ui::{language_icon, tc};
use eframe::egui::{self, Button, Color32, Context, RichText, ScrollArea, Ui};

impl RustNoteApp {
    pub(crate) fn render_tab_bar(&mut self, ui: &mut Ui) {
        let ctx: Context = ui.ctx().clone();
        let light = !self.theme.is_dark();
        egui::Panel::top("tab_bar").show(ui, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ScrollArea::horizontal()
                    .id_salt("tab_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut to_close = None;
                            let mut close_others = None;
                            let mut copy_path_idx = None;
                            let mut copy_name_idx = None;
                            let mut toggle_pin_idx = None;

                            for (idx, doc) in self.docs.iter().enumerate() {
                                let is_active = idx == self.active_index;
                                let lang_icon = language_icon(doc.language);
                                let dirty_dot = if doc.is_dirty { " ●" } else { "" };
                                let pin = if doc.is_pinned {
                                    format!("{}", glyph::PIN)
                                } else {
                                    String::new()
                                };
                                let label = format!("{pin}{lang_icon} {}{dirty_dot}", doc.title);

                                let text_color = if is_active {
                                    tc(light, 137, 180, 250) // Soft Accent Blue
                                } else if doc.is_dirty {
                                    tc(light, 250, 179, 135) // Amber
                                } else {
                                    tc(light, 186, 194, 222) // Soft Text
                                };

                                // Windows 11 style: rounded pill, filled when active
                                let fill = if is_active {
                                    if light { Color32::from_rgb(238, 242, 254) } else { Color32::from_rgb(45, 48, 82) }
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.horizontal(|ui| {
                                    let tab_btn = ui.add(
                                        Button::new(RichText::new(label).color(text_color).strong())
                                            .fill(fill)
                                            .corner_radius(6),
                                    );
                                    if tab_btn.clicked() {
                                        self.active_index = idx;
                                    }
                                    // Middle-click closes (never a pinned tab)
                                    if tab_btn.middle_clicked() && !doc.is_pinned {
                                        to_close = Some(idx);
                                    }

                                    tab_btn.context_menu(|ui| {
                                        let pin_label = if doc.is_pinned { tr("Unpin Tab") } else { tr("Pin Tab") };
                                        if ui.button(pin_label).clicked() {
                                            toggle_pin_idx = Some(idx);
                                            ui.close();
                                        }
                                        ui.separator();
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

                                    if ui
                                        .small_button(RichText::new("×").color(text_color))
                                        .on_hover_text(tr("Close tab (Ctrl+W)"))
                                        .clicked()
                                    {
                                        to_close = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = toggle_pin_idx {
                                if let Some(doc) = self.docs.get_mut(idx) {
                                    doc.is_pinned = !doc.is_pinned;
                                    let pinned = doc.is_pinned;
                                    self.set_notification(if pinned {
                                        "Tab pinned"
                                    } else {
                                        "Tab unpinned"
                                    });
                                }
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

                            ui.add_space(2.0);
                            if ui.button(icon(glyph::ADD)).on_hover_text(tr("New tab (Ctrl+N)")).clicked() {
                                self.new_document();
                            }
                        });
                    });
            });
        });
    }
}
