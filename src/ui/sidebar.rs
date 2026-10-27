//! Function list sidebar (document outline with fuzzy filter).

use crate::app::RustNoteApp;
use crate::i18n::tr;
use crate::ui::tc;
use eframe::egui::{self, Context, Layout, RichText, ScrollArea};

impl RustNoteApp {
    pub(crate) fn render_function_list(&mut self, _ctx: &Context) {
        let light = !self.theme.is_dark();
        egui::SidePanel::right("function_list_sidebar")
            .default_width(240.0)
            .min_width(180.0)
            .max_width(450.0)
            .show(_ctx, |ui| {
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
}
