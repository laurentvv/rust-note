//! Editor color themes (palettes inspired by the classic Notepad++ theme set).

use crate::syntax::highlight::Palette;
use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EditorTheme {
    #[default]
    DarkMode,
    ClassicLight,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    Zenburn,
}

pub struct ThemeColors {
    pub dark: bool,
    pub panel: Color32,
    pub window: Color32,
    pub editor: Color32,
    pub selection: Color32,
    pub accent: Color32,
    pub palette: Palette,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl EditorTheme {
    pub const ALL: [Self; 6] = [
        Self::DarkMode,
        Self::ClassicLight,
        Self::Monokai,
        Self::SolarizedDark,
        Self::SolarizedLight,
        Self::Zenburn,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DarkMode => "Obsidian Dark (default)",
            Self::ClassicLight => "Classic Light",
            Self::Monokai => "Monokai",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::Zenburn => "Zenburn",
        }
    }

    #[must_use]
    pub fn is_dark(self) -> bool {
        self.colors().dark
    }

    #[must_use]
    pub fn colors(self) -> ThemeColors {
        match self {
            Self::DarkMode => ThemeColors {
                dark: true,
                panel: rgb(24, 24, 37),
                window: rgb(30, 30, 46),
                editor: rgb(17, 17, 27),
                selection: rgb(69, 71, 110),
                accent: rgb(137, 180, 250),
                palette: Palette::DARK,
            },
            Self::ClassicLight => ThemeColors {
                dark: false,
                panel: rgb(240, 240, 240),
                window: rgb(255, 255, 255),
                editor: rgb(255, 255, 255),
                selection: rgb(173, 214, 255),
                accent: rgb(0, 102, 204),
                palette: Palette::LIGHT,
            },
            Self::Monokai => ThemeColors {
                dark: true,
                panel: rgb(30, 31, 28),
                window: rgb(39, 40, 34),
                editor: rgb(39, 40, 34),
                selection: rgb(73, 72, 62),
                accent: rgb(166, 226, 46),
                palette: Palette {
                    text: rgb(248, 248, 242),
                    keyword: rgb(249, 38, 114),
                    string: rgb(230, 219, 116),
                    comment: rgb(117, 113, 94),
                    number: rgb(174, 129, 255),
                    type_name: rgb(102, 217, 239),
                },
            },
            Self::SolarizedDark => ThemeColors {
                dark: true,
                panel: rgb(0, 43, 54),
                window: rgb(7, 54, 66),
                editor: rgb(0, 43, 54),
                selection: rgb(7, 80, 100),
                accent: rgb(38, 139, 210),
                palette: Palette {
                    text: rgb(147, 161, 161),
                    keyword: rgb(133, 153, 0),
                    string: rgb(42, 161, 152),
                    comment: rgb(88, 110, 117),
                    number: rgb(211, 54, 130),
                    type_name: rgb(38, 139, 210),
                },
            },
            Self::SolarizedLight => ThemeColors {
                dark: false,
                panel: rgb(238, 232, 213),
                window: rgb(253, 246, 227),
                editor: rgb(253, 246, 227),
                selection: rgb(220, 212, 180),
                accent: rgb(38, 139, 210),
                palette: Palette {
                    text: rgb(88, 110, 117),
                    keyword: rgb(133, 153, 0),
                    string: rgb(42, 161, 152),
                    comment: rgb(147, 161, 161),
                    number: rgb(211, 54, 130),
                    type_name: rgb(38, 139, 210),
                },
            },
            Self::Zenburn => ThemeColors {
                dark: true,
                panel: rgb(50, 50, 50),
                window: rgb(63, 63, 63),
                editor: rgb(63, 63, 63),
                selection: rgb(80, 80, 100),
                accent: rgb(240, 223, 175),
                palette: Palette {
                    text: rgb(220, 220, 204),
                    keyword: rgb(240, 223, 175),
                    string: rgb(204, 147, 147),
                    comment: rgb(127, 159, 127),
                    number: rgb(140, 208, 211),
                    type_name: rgb(223, 175, 143),
                },
            },
        }
    }

    /// Builds the egui visuals for this theme.
    #[must_use]
    pub fn visuals(self) -> egui::Visuals {
        let c = self.colors();
        let mut v = if c.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        v.panel_fill = c.panel;
        v.window_fill = c.window;
        v.extreme_bg_color = c.editor;
        v.selection.bg_fill = c.selection;
        v.selection.stroke = egui::Stroke::new(1.0_f32, c.accent);
        v.hyperlink_color = c.accent;
        v.override_text_color = Some(c.palette.text);
        v
    }
}
