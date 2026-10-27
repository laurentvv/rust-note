//! Top menu bar (File / Edit / Search / View / Encoding / Language / Plugins / Tools / ?).

use crate::app::RustNoteApp;
use crate::buffer::{Encoding, EolFormat};
use crate::editor::operations::{self, CaseMode};
use crate::i18n::{self, tr, UiLang};
use crate::plugins;
use crate::syntax::Language;
use crate::ui::dialogs::FindTab;
use crate::ui::themes::EditorTheme;
use eframe::egui::{self, Context, ScrollArea, Ui};

impl RustNoteApp {
    pub(crate) fn render_menu_bar(&mut self, ui: &mut Ui) {
        let ctx: Context = ui.ctx().clone();
        egui::Panel::top("top_menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // --- FILE ---
                ui.menu_button(tr("File"), |ui| {
                    if ui.button(tr("New\tCtrl+N")).clicked() {
                        self.new_document();
                        ui.close();
                    }
                    if ui.button(tr("Open...\tCtrl+O")).clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    if ui.button(tr("Save\tCtrl+S")).clicked() {
                        self.save_current();
                        ui.close();
                    }
                    if ui.button(tr("Save As...\tCtrl+Alt+S")).clicked() {
                        self.save_as_current();
                        ui.close();
                    }
                    if ui.button(tr("Save All\tCtrl+Shift+S")).clicked() {
                        self.save_all();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Close\tCtrl+W")).clicked() {
                        self.close_current();
                        ui.close();
                    }
                    if ui.button(tr("Close All\tCtrl+Shift+W")).clicked() {
                        self.close_all();
                        ui.close();
                    }
                    if ui.button(tr("Close All BUT Current")).clicked() {
                        self.close_all_except_current();
                        ui.close();
                    }
                    if ui.button(tr("Restore Closed Tab\tCtrl+Shift+T")).clicked() {
                        self.restore_closed_document();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Export to HTML...")).clicked() {
                        self.export_current_to_html();
                        ui.close();
                    }
                    if ui.button(tr("Export to RTF (Rich Text)...")).clicked() {
                        self.export_current_to_rtf();
                        ui.close();
                    }
                    ui.separator();
                    ui.add_enabled_ui(!self.settings.recent_files.is_empty(), |ui| {
                        ui.menu_button(tr("Recent Files"), |ui| {
                            let recents = self.settings.recent_files.clone();
                            for (n, path) in recents.iter().enumerate() {
                                let name = path.file_name().map_or_else(|| path.display().to_string(), |f| f.to_string_lossy().to_string());
                                if ui.button(format!("{}  {name}", n + 1)).on_hover_text(path.display().to_string()).clicked() {
                                    self.open_path(path.clone());
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button(tr("Clear Recent Files")).clicked() {
                                self.settings.recent_files.clear();
                                ui.close();
                            }
                        });
                    });
                    ui.menu_button(tr("Copy to Clipboard"), |ui| {
                        if ui.button(tr("Copy Full File Path")).clicked() {
                            self.copy_full_path(&ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Filename")).clicked() {
                            self.copy_filename(&ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Directory Path")).clicked() {
                            self.copy_directory_path(&ctx);
                            ui.close();
                        }
                    });
                    ui.separator();
                    if ui.button(tr("Exit\tAlt+F4")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });

                // --- EDIT ---
                ui.menu_button(tr("Edit"), |ui| {
                    if ui.button(tr("Undo\tCtrl+Z")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.undo();
                        }
                        ui.close();
                    }
                    if ui.button(tr("Redo\tCtrl+Y")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.redo();
                        }
                        ui.close();
                    }
                    ui.separator();

                    ui.menu_button(tr("Line Operations"), |ui| {
                        if ui.button(tr("Duplicate Current Line\tCtrl+D")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let dup = operations::duplicate_line(&doc.content, cur);
                                doc.set_content(dup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Remove Duplicate Lines")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, false);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Remove Consecutive Duplicate Lines")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let dedup = operations::remove_duplicate_lines(&doc.content, true);
                                doc.set_content(dedup);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Move Line Up")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_up(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line = doc.cursor_line.saturating_sub(1).max(1);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Move Line Down")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let cur = doc.cursor_line;
                                let moved = operations::move_line_down(&doc.content, cur);
                                doc.set_content(moved);
                                doc.cursor_line += 1;
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(tr("Sort Lines Ascending")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, true, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Sort Lines Descending")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let sorted = operations::sort_lines(&doc.content, false, true, false);
                                doc.set_content(sorted);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Join Lines\tCtrl+J")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let joined = operations::join_lines(&doc.content);
                                doc.set_content(joined);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Convert Case to"), |ui| {
                        if ui.button(tr("UPPERCASE\tCtrl+Shift+U")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Uppercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("lowercase\tCtrl+U")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::Lowercase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Proper Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::ProperCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Invert Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::InvertCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Random Case")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let c = operations::convert_case(&doc.content, CaseMode::RandomCase);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Blank Operations"), |ui| {
                        if ui.button(tr("Trim Trailing Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_trailing_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Trim Leading Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::trim_leading_spaces(&doc.content);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("TAB to Space")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::tabs_to_spaces(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Space to TAB")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let t = operations::spaces_to_tabs(&doc.content, 4);
                                doc.set_content(t);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Insert Date Time"), |ui| {
                        let now = chrono::Local::now();
                        if ui.button(tr("Date Time (short)")).clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Date Time (long)")).clicked() {
                            let stamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                            if let Some(doc) = self.active_doc_mut() {
                                let mut c = doc.content.clone();
                                c.push_str(&stamp);
                                doc.set_content(c);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("EOL Conversion"), |ui| {
                        if ui.button(tr("Windows (CR LF)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Windows);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Windows;
                            }
                            ui.close();
                        }
                        if ui.button(tr("Unix (LF)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Unix);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Unix;
                            }
                            ui.close();
                        }
                        if ui.button(tr("Macintosh (CR)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let converted =
                                    crate::buffer::eol::convert(&doc.content, EolFormat::Mac);
                                doc.set_content(converted);
                                doc.eol = EolFormat::Mac;
                            }
                            ui.close();
                        }
                    });

                    if ui.button(tr("Toggle Comment\tCtrl+Q")).clicked() {
                        let prefix = self.current_language.comment_prefix();
                        if let Some(doc) = self.active_doc_mut() {
                            let c = operations::toggle_comment(&doc.content, prefix);
                            doc.set_content(c);
                        }
                        ui.close();
                    }
                });

                // --- SEARCH ---
                ui.menu_button(tr("Search"), |ui| {
                    if ui.button(tr("Find...\tCtrl+F")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Find;
                        ui.close();
                    }
                    if ui.button(tr("Replace...\tCtrl+H")).clicked() {
                        self.find_replace.is_open = true;
                        self.find_replace.tab = FindTab::Replace;
                        ui.close();
                    }
                    if ui.button(tr("Find Next\tF3")).clicked() {
                        self.execute_find(true);
                        ui.close();
                    }
                    if ui.button(tr("Find Previous\tShift+F3")).clicked() {
                        self.execute_find(false);
                        ui.close();
                    }
                    if ui.button(tr("Go to Line...\tCtrl+G")).clicked() {
                        let max_l = self.active_doc().map(|d| d.line_count()).unwrap_or(1);
                        self.goto_line.is_open = true;
                        self.goto_line.max_lines = max_l;
                        ui.close();
                    }
                });

                // --- VIEW ---
                ui.menu_button(tr("View"), |ui| {
                    ui.checkbox(&mut self.word_wrap, tr("Word Wrap"));
                    ui.checkbox(&mut self.show_line_numbers, tr("Line Numbers"));
                    ui.checkbox(&mut self.show_function_list, tr("Function List (Ctrl+Alt+L)"));
                    ui.separator();
                    if ui.button(tr("Zoom In (Ctrl+Num+)")).clicked() {
                        self.font_size = (self.font_size + 2.0).min(36.0);
                        ui.close();
                    }
                    if ui.button(tr("Zoom Out (Ctrl+Num-)")).clicked() {
                        self.font_size = (self.font_size - 2.0).max(8.0);
                        ui.close();
                    }
                    if ui.button(tr("Restore Default Zoom")).clicked() {
                        self.font_size = 14.0;
                        ui.close();
                    }
                    ui.separator();
                    ui.menu_button(tr("Interface language"), |ui| {
                        for l in UiLang::ALL {
                            if ui.selectable_label(i18n::lang() == l, l.name()).clicked() {
                                i18n::set_lang(l);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button(tr("Theme"), |ui| {
                        for t in EditorTheme::ALL {
                            if ui.selectable_label(self.theme == t, t.name()).clicked() {
                                self.theme = t;
                                self.apply_theme(&ctx);
                                self.set_notification(format!("Theme: {}", t.name()));
                                ui.close();
                            }
                        }
                    });
                });

                // --- ENCODING ---
                ui.menu_button(tr("Encoding"), |ui| {
                    if ui.button(tr("UTF-8")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-8-BOM")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf8Bom;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-16 LE")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Le;
                        }
                        ui.close();
                    }
                    if ui.button(tr("UTF-16 BE")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Utf16Be;
                        }
                        ui.close();
                    }
                    if ui.button(tr("ANSI (Windows-1252)")).clicked() {
                        if let Some(doc) = self.active_doc_mut() {
                            doc.encoding = Encoding::Ansi;
                        }
                        ui.close();
                    }
                });

                // --- LANGUAGE ---
                ui.menu_button(tr("Language"), |ui| {
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
                        Language::Nim,
                    ];
                    // A solid (non floating) scrollbar reserves its own space instead of covering long names
                    ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
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
                ui.menu_button(tr("Plugins"), |ui| {
                    ui.menu_button(tr("MIME Tools"), |ui| {
                        if ui.button(tr("Base64 Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::base64_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button(tr("Base64 Decode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::base64_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("URL Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::url_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                        if ui.button(tr("URL Decode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::url_decode(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("Quoted-Printable Encode")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let enc = plugins::quoted_printable_encode(&doc.content);
                                doc.set_content(enc);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("Converter"), |ui| {
                        if ui.button(tr("0x Quick Conversion Panel...")).clicked() {
                            self.conversion_panel.is_open = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(tr("ASCII to HEX")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                let hex = plugins::ascii_to_hex(&doc.content);
                                doc.set_content(hex);
                            }
                            ui.close();
                        }
                        if ui.button(tr("HEX to ASCII")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(asc) = plugins::hex_to_ascii(&doc.content) {
                                    doc.set_content(asc);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("DEC to HEX (0x...)")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(hex) = plugins::dec_to_hex(&doc.content) {
                                    doc.set_content(hex);
                                }
                            }
                            ui.close();
                        }
                        if ui.button(tr("HEX to DEC")).clicked() {
                            if let Some(doc) = self.active_doc_mut() {
                                if let Ok(dec) = plugins::hex_to_dec(&doc.content) {
                                    doc.set_content(dec);
                                }
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("NppExport"), |ui| {
                        if ui.button(tr("Export to HTML...")).clicked() {
                            self.export_current_to_html();
                            ui.close();
                        }
                        if ui.button(tr("Export to RTF (Rich Text)...")).clicked() {
                            self.export_current_to_rtf();
                            ui.close();
                        }
                    });

                    ui.menu_button(tr("NppShell"), |ui| {
                        if ui.button(tr("Copy Full File Path")).clicked() {
                            self.copy_full_path(&ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Filename")).clicked() {
                            self.copy_filename(&ctx);
                            ui.close();
                        }
                        if ui.button(tr("Copy Directory Path")).clicked() {
                            self.copy_directory_path(&ctx);
                            ui.close();
                        }
                    });
                });

                // --- TOOLS ---
                ui.menu_button(tr("Tools"), |ui| {
                    if ui.button(tr("Document Summary")).clicked() {
                        self.show_summary();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr("Trim Trailing Space and Save\tAlt+Shift+S")).clicked() {
                        self.trim_and_save();
                        ui.close();
                    }
                    if ui.button(tr("Open in New Instance\tAlt+F6")).clicked() {
                        self.open_in_new_instance();
                        ui.close();
                    }
                    ui.menu_button(tr("Help on Internet"), |ui| {
                        if ui.button(tr("Get PHP help\tAlt+F1")).clicked() {
                            self.web_lookup(&ctx, "https://www.php.net/");
                            ui.close();
                        }
                        if ui.button(tr("Wikipedia Search\tAlt+F3")).clicked() {
                            self.web_lookup(&ctx, "https://en.wikipedia.org/wiki/Special:Search?search=");
                            ui.close();
                        }
                        if ui.button(tr("Search the web")).clicked() {
                            self.web_lookup(&ctx, "https://duckduckgo.com/?q=");
                            ui.close();
                        }
                    });
                });

                // --- HELP (?) ---
                ui.menu_button(tr("?"), |ui| {
                    if ui.button(tr("Keyboard Shortcuts")).clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    if ui.button(tr("About rust-note")).clicked() {
                        self.about.is_open = true;
                        ui.close();
                    }
                    if ui.button(tr("Documentation & Sources")).clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab("https://github.com/laurentvv/rust-note"));
                        ui.close();
                    }
                });
            });
        });
    }
}
