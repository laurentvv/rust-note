//! Central editor panel: gutter, syntax-highlighted TextEdit, column selection,
//! auto-indent and the auto-completion popup.

use crate::app::RustNoteApp;
use crate::buffer::Document;
use crate::editor::completion::{apply_completion, completion_candidates};
use crate::editor::operations::{self, CaseMode};
use crate::i18n::tr;
use crate::plugins;
use crate::syntax::Language;
use crate::syntax::highlight::highlight as syntax_highlight;
use eframe::egui::{self, Color32, FontId, ScrollArea, Ui};

fn layout_font_id(size: f32) -> FontId {
    FontId::monospace(size)
}

impl RustNoteApp {
    pub(crate) fn render_editor(&mut self, ui: &mut Ui) {
        let mut completion = std::mem::take(&mut self.completion);
        let mut wrap_off = false;
        let editor_bg = self.theme.colors().editor;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(editor_bg)
                    .inner_margin(egui::Margin::same(4)),
            )
            .show(ui, |ui| {
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

                        let editor_id = egui::Id::unique(("editor", doc.id));
                        let pending = doc.pending_selection.take();
                        if let Some((a, b)) = pending {
                            let mut state =
                                egui::text_edit::TextEditState::load(ui.ctx(), editor_id)
                                    .unwrap_or_default();
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::two(
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
                        let force_complete = ui
                            .ctx()
                            .input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));
                        if !completion.items.is_empty()
                            && ui.ctx().memory(|m| m.has_focus(editor_id))
                        {
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
                        if let (Some(sel), true) =
                            (doc.column_sel, ui.ctx().memory(|m| m.has_focus(editor_id)))
                        {
                            let mut sel = sel;
                            let mut keep_sel = true;
                            let stamp_before = doc.last_edit;
                            let events = ui.ctx().input_mut(|i| std::mem::take(&mut i.events));
                            let mut kept = Vec::with_capacity(events.len());
                            for ev in events {
                                match &ev {
                                    egui::Event::Text(t) | egui::Event::Paste(t)
                                        if !doc.is_readonly =>
                                    {
                                        let lines: Vec<&str> = t.lines().collect();
                                        let (c, s) = if matches!(&ev, egui::Event::Paste(_))
                                            && lines.len() > 1
                                        {
                                            crate::editor::column::insert_each(
                                                &doc.content,
                                                &sel,
                                                &lines,
                                            )
                                        } else {
                                            crate::editor::column::insert(&doc.content, &sel, t)
                                        };
                                        doc.content = c;
                                        sel = s;
                                        doc.record_typing();
                                    }
                                    egui::Event::Key {
                                        key: egui::Key::Backspace | egui::Key::Delete,
                                        pressed: true,
                                        modifiers,
                                        ..
                                    } if modifiers.is_none() && !doc.is_readonly => {
                                        let fwd = matches!(
                                            &ev,
                                            egui::Event::Key {
                                                key: egui::Key::Delete,
                                                ..
                                            }
                                        );
                                        let (c, s) =
                                            crate::editor::column::delete(&doc.content, &sel, fwd);
                                        doc.content = c;
                                        sel = s;
                                        doc.record_typing();
                                    }
                                    egui::Event::Copy => {
                                        ui.ctx().copy_text(crate::editor::column::extract(
                                            &doc.content,
                                            &sel,
                                        ));
                                    }
                                    egui::Event::Cut if !doc.is_readonly => {
                                        ui.ctx().copy_text(crate::editor::column::extract(
                                            &doc.content,
                                            &sel,
                                        ));
                                        if !sel.is_caret_only() {
                                            let (c, s) = crate::editor::column::delete(
                                                &doc.content,
                                                &sel,
                                                true,
                                            );
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
                                let pos =
                                    crate::editor::column::head_char_index(&doc.content, &sel);
                                let mut st =
                                    egui::text_edit::TextEditState::load(ui.ctx(), editor_id)
                                        .unwrap_or_default();
                                st.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(pos),
                                )));
                                st.store(ui.ctx(), editor_id);
                            }
                            if !keep_sel {
                                doc.column_dragging = false;
                            }
                        }

                        // Auto-indent: Enter keeps the indentation of the current line (+4 after an opening brace)
                        let enter_pressed = !doc.is_readonly
                            && ui.ctx().memory(|m| m.has_focus(editor_id))
                            && ui.ctx().input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                            });
                        if enter_pressed {
                            let mut state =
                                egui::text_edit::TextEditState::load(ui.ctx(), editor_id)
                                    .unwrap_or_default();
                            if let Some(range) = state.cursor.char_range() {
                                let (a, b) = (
                                    usize::from(range.primary.index)
                                        .min(usize::from(range.secondary.index)),
                                    usize::from(range.primary.index)
                                        .max(usize::from(range.secondary.index)),
                                );
                                let chars: Vec<char> = doc.content.chars().collect();
                                let a = a.min(chars.len());
                                let b = b.min(chars.len());
                                let line_start = chars[..a]
                                    .iter()
                                    .rposition(|c| *c == '\n')
                                    .map_or(0, |p| p + 1);
                                let mut insert = String::from("\n");
                                insert.extend(
                                    chars[line_start..a]
                                        .iter()
                                        .take_while(|c| **c == ' ' || **c == '\t'),
                                );
                                if a > 0 && matches!(chars[a - 1], '{' | '(' | '[' | ':') {
                                    insert.push_str("    ");
                                }
                                let mut out: String = chars[..a].iter().collect();
                                out.push_str(&insert);
                                out.extend(chars[b..].iter());
                                doc.content = out;
                                doc.record_typing();
                                let pos = a + insert.chars().count();
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::two(
                                        egui::text::CCursor::new(pos),
                                        egui::text::CCursor::new(pos),
                                    )));
                                state.store(ui.ctx(), editor_id);
                            }
                        }

                        let mut gutter_numbers: Vec<(f32, usize)> = Vec::new();
                        let scroll_area = if word_wrap {
                            ScrollArea::vertical()
                        } else {
                            ScrollArea::both()
                        };
                        scroll_area
                            .id_salt("editor_scroll")
                            .auto_shrink([false, false])
                            .min_scrolled_height(editor_avail_size.y)
                            .min_scrolled_width(editor_avail_size.x)
                            .show(ui, |ui| {
                                let layout_font = font.clone();
                                let mut layouter =
                                    |ui: &egui::Ui,
                                     text: &dyn egui::TextBuffer,
                                     wrap_width: f32| {
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
                                    .min_size(egui::Vec2::new(
                                        editor_avail_size.x,
                                        editor_avail_size.y,
                                    ))
                                    .frame(egui::Frame::NONE)
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
                                    let (line, col) = doc
                                        .line_col_from_char_index(usize::from(range.primary.index));
                                    doc.cursor_line = line;
                                    doc.cursor_col = col;
                                    doc.cursor_char =
                                        usize::from(range.primary.index.min(range.secondary.index));
                                    doc.selection_len =
                                        range.primary.index.0.abs_diff(range.secondary.index.0);
                                }

                                let galley_pos = output.galley_pos;
                                // Current-line highlight and scroll-to-cursor after a programmatic jump
                                if let Some(range) = cursor_range {
                                    let crect = output
                                        .galley
                                        .pos_from_cursor(range.primary)
                                        .translate(galley_pos.to_vec2());
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
                                        ui.scroll_to_rect(
                                            crect.expand2(egui::vec2(0.0, 40.0)),
                                            Some(if pending.is_some() {
                                                egui::Align::Center
                                            } else {
                                                egui::Align::Max
                                            }),
                                        );
                                    }
                                }

                                // Column selection: Alt+drag creates it, a plain click clears it
                                {
                                    let cw =
                                        ui.fonts_mut(|f| f.glyph_width(&layout_font, 'M')).max(1.0);
                                    let (alt, pressed, down, pointer) = ui.input(|i| {
                                        (
                                            i.modifiers.alt,
                                            i.pointer.primary_pressed(),
                                            i.pointer.primary_down(),
                                            i.pointer.interact_pos(),
                                        )
                                    });
                                    let to_lc = |p: egui::Pos2, doc: &Document| -> (usize, usize) {
                                        let rel = p - output.galley_pos;
                                        let c = output.galley.cursor_from_pos(rel);
                                        let (line, _) =
                                            doc.line_col_from_char_index(usize::from(c.index));
                                        let line_start = doc.char_index_of_line(line);
                                        // On the first row of a line the column follows the pointer (virtual space
                                        // allowed); on wrapped continuation rows it is the real offset in the line.
                                        let first_row = output
                                            .galley
                                            .pos_from_cursor(egui::text::CCursor::new(line_start))
                                            .max
                                            .y;
                                        let col = if rel.y <= first_row {
                                            (rel.x / cw).round().max(0.0) as usize
                                        } else {
                                            c.index.0.saturating_sub(line_start)
                                        };
                                        (line - 1, col)
                                    };
                                    if let Some(p) = pointer {
                                        if pressed && response.rect.contains(p) {
                                            if alt {
                                                let lc = to_lc(p, doc);
                                                doc.column_sel =
                                                    Some(crate::editor::column::ColumnSel {
                                                        anchor: lc,
                                                        head: lc,
                                                    });
                                                doc.column_dragging = true;
                                                // Column blocks are straight rectangles: they need unwrapped lines
                                                wrap_off |= word_wrap;
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
                                                st.cursor.set_char_range(Some(
                                                    egui::text::CCursorRange::one(range.primary),
                                                ));
                                                st.store(ui.ctx(), editor_id);
                                            }
                                        }
                                        let (l1, l2, c1, c2) = sel.bounds();
                                        let fill =
                                            ui.visuals().selection.bg_fill.gamma_multiply(0.8);
                                        // Paint the block on every visual row of the selected logical lines
                                        let mut logical = 0usize;
                                        let mut new_line = true;
                                        for placed in &output.galley.rows {
                                            if new_line {
                                                new_line = false;
                                            }
                                            if logical >= l1 && logical <= l2 {
                                                let x0 = output.galley_pos.x + c1 as f32 * cw;
                                                let x1 = (output.galley_pos.x + c2 as f32 * cw)
                                                    .max(x0 + 2.0);
                                                let y0 = output.galley_pos.y + placed.pos.y;
                                                let rect = egui::Rect::from_min_max(
                                                    egui::pos2(x0, y0),
                                                    egui::pos2(x1, y0 + placed.row.height()),
                                                );
                                                ui.painter().rect_filled(rect, 0.0, fill);
                                            }
                                            if placed.ends_with_newline {
                                                logical += 1;
                                                new_line = true;
                                            }
                                        }
                                        let _ = (c1, new_line);
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
                                        completion.items = completion_candidates(
                                            &doc.content,
                                            usize::from(range.primary.index),
                                            doc_lang,
                                            force_complete,
                                        );
                                        completion.selected = 0;
                                        completion.at = usize::from(range.primary.index);
                                    } else if usize::from(range.primary.index) != completion.at
                                        || !response.has_focus()
                                    {
                                        completion.items.clear();
                                    }
                                    if !completion.items.is_empty() {
                                        let r = output
                                            .galley
                                            .pos_from_cursor(range.primary)
                                            .translate(output.galley_pos.to_vec2());
                                        let mut clicked = None;
                                        egui::Area::new(egui::Id::unique("completion_popup"))
                                            .order(egui::Order::Foreground)
                                            .fixed_pos(r.left_bottom() + egui::vec2(0.0, 2.0))
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                    for (i, item) in
                                                        completion.items.iter().enumerate()
                                                    {
                                                        let resp = ui.selectable_label(
                                                            i == completion.selected,
                                                            egui::RichText::new(item).monospace(),
                                                        );
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
                                            gutter_numbers
                                                .push((galley_pos.y + placed.pos.y, line_no));
                                            line_no += 1;
                                            new_line = false;
                                        }
                                        if placed.ends_with_newline {
                                            new_line = true;
                                        }
                                    }
                                }

                                let comment_prefix =
                                    doc.language.unwrap_or(current_lang).comment_prefix();
                                response.context_menu(|ui| {
                                    ui.label(
                                        egui::RichText::new(tr("rust-note Context Menu")).strong(),
                                    );
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
                                        if ui.button(tr("Duplicate Current Line\tCtrl+D")).clicked()
                                        {
                                            let cur = doc.cursor_line;
                                            let dup = operations::duplicate_line(&doc.content, cur);
                                            doc.set_content(dup);
                                            ui.close();
                                        }
                                        if ui.button(tr("Move Line Up")).clicked() {
                                            let cur = doc.cursor_line;
                                            let moved = operations::move_line_up(&doc.content, cur);
                                            doc.set_content(moved);
                                            doc.cursor_line =
                                                doc.cursor_line.saturating_sub(1).max(1);
                                            ui.close();
                                        }
                                        if ui.button(tr("Move Line Down")).clicked() {
                                            let cur = doc.cursor_line;
                                            let moved =
                                                operations::move_line_down(&doc.content, cur);
                                            doc.set_content(moved);
                                            doc.cursor_line += 1;
                                            ui.close();
                                        }
                                        if ui.button(tr("Remove Duplicate Lines")).clicked() {
                                            let dedup = operations::remove_duplicate_lines(
                                                &doc.content,
                                                false,
                                            );
                                            doc.set_content(dedup);
                                            ui.close();
                                        }
                                        if ui.button(tr("Sort Lines Ascending")).clicked() {
                                            let sorted = operations::sort_lines(
                                                &doc.content,
                                                true,
                                                true,
                                                false,
                                            );
                                            doc.set_content(sorted);
                                            ui.close();
                                        }
                                    });
                                    ui.menu_button(tr("Convert Case"), |ui| {
                                        if ui.button(tr("UPPERCASE\tCtrl+Shift+U")).clicked() {
                                            let c = operations::convert_case(
                                                &doc.content,
                                                CaseMode::Uppercase,
                                            );
                                            doc.set_content(c);
                                            ui.close();
                                        }
                                        if ui.button(tr("lowercase\tCtrl+U")).clicked() {
                                            let c = operations::convert_case(
                                                &doc.content,
                                                CaseMode::Lowercase,
                                            );
                                            doc.set_content(c);
                                            ui.close();
                                        }
                                        if ui.button(tr("Proper Case")).clicked() {
                                            let c = operations::convert_case(
                                                &doc.content,
                                                CaseMode::ProperCase,
                                            );
                                            doc.set_content(c);
                                            ui.close();
                                        }
                                    });
                                    ui.separator();
                                    if ui
                                        .button(format!(
                                            "{} ({comment_prefix})",
                                            tr("Toggle Comment")
                                        ))
                                        .clicked()
                                    {
                                        let c = operations::toggle_comment(
                                            &doc.content,
                                            comment_prefix,
                                        );
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
                                                    ui.ctx().copy_text(
                                                        parent.to_string_lossy().to_string(),
                                                    );
                                                }
                                            }
                                            ui.close();
                                        }
                                    });
                                });
                            });

                        // Gutter painted next to the editor, aligned on its rows
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
        if wrap_off {
            self.word_wrap = false;
            self.set_notification("Word wrap turned off for column selection");
        }
    }
}
