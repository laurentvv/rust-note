//! System font integration (Windows 11 typography) and Fluent icon helpers.
//!
//! Loads, when present on the machine:
//! - **Segoe UI Variable** for the proportional UI text,
//! - **Cascadia Mono** for the editor (falls back to Consolas then to egui defaults),
//! - **Segoe MDL2 Assets** as an `icons` family used by the toolbar and tabs.

use eframe::egui::{self, Context, FontData, FontFamily, RichText};
use std::sync::Arc;

/// Family registered for Fluent (Segoe MDL2) glyphs.
pub const ICONS_FAMILY: &str = "icons";

fn icons_family() -> FontFamily {
    FontFamily::Name(Arc::from(ICONS_FAMILY))
}

fn load_first(
    fonts: &mut egui::FontDefinitions,
    family: FontFamily,
    name: &str,
    candidates: &[&str],
) -> bool {
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_string(), FontData::from_owned(bytes).into());
            // Create the family if egui has no entry for it yet (named families start empty)
            fonts.families.entry(family).or_default().insert(0, name.to_string());
            return true;
        }
    }
    false
}

/// Installs the Windows system fonts. Missing files keep the egui defaults.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    let ui_ok = load_first(
        &mut fonts,
        FontFamily::Proportional,
        "segoe_ui_variable",
        &[
            r"C:\Windows\Fonts\SegUIVar.ttf",
            r"C:\Windows\Fonts\segoeui.ttf",
        ],
    );
    let mono_ok = load_first(
        &mut fonts,
        FontFamily::Monospace,
        "cascadia_mono",
        &[
            r"C:\Windows\Fonts\CascadiaMono.ttf",
            r"C:\Windows\Fonts\consola.ttf",
        ],
    );
    load_first(
        &mut fonts,
        icons_family(),
        "segoe_mdl2",
        &[r"C:\Windows\Fonts\segmdl2.ttf"],
    );
    if ui_ok || mono_ok {
        ctx.set_fonts(fonts);
    }
}

/// True when the Windows apps theme is Light (None when unavailable / non-Windows).
pub fn windows_apps_light() -> Option<bool> {
    #[cfg(windows)]
    {
        let value = windows_registry::CURRENT_USER
            .open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
            .ok()?
            .get_u32("AppsUseLightTheme")
            .ok()?;
        Some(value != 0)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Fluent (Segoe MDL2 Assets) glyph codes used by the UI.
pub mod glyph {
    pub const ADD: char = '\u{E710}';
    pub const OPEN_FILE: char = '\u{E8E5}';
    pub const SAVE: char = '\u{E74E}';
    pub const CANCEL: char = '\u{E711}';
    pub const UNDO: char = '\u{E7A7}';
    pub const REDO: char = '\u{E7A6}';
    pub const SEARCH: char = '\u{E71E}';
    pub const SYNC: char = '\u{E895}'; // replace
    pub const SETTINGS: char = '\u{E713}';
    pub const LIST: char = '\u{E8FD}';
    pub const INFO: char = '\u{E946}';
    pub const PIN: char = '\u{E718}';
    pub const UNPIN: char = '\u{E77A}';
    pub const COMMAND: char = '\u{E945}'; // command palette / keyboard
}

/// A Fluent icon as styled text, ready for a button or label.
#[must_use]
pub fn icon(code: char) -> RichText {
    RichText::new(code.to_string()).family(icons_family())
}

#[cfg(test)]
mod tests {
    #[test]
    fn icon_glyphs_are_in_the_private_use_area() {
        // Segoe MDL2 codes live in the U+E000..U+F8FF plane; a stray plain
        // character here would silently render as text in the toolbar.
        for code in [
            super::glyph::ADD,
            super::glyph::OPEN_FILE,
            super::glyph::SAVE,
            super::glyph::CANCEL,
            super::glyph::UNDO,
            super::glyph::REDO,
            super::glyph::SEARCH,
            super::glyph::SYNC,
            super::glyph::SETTINGS,
            super::glyph::LIST,
            super::glyph::INFO,
            super::glyph::PIN,
            super::glyph::UNPIN,
            super::glyph::COMMAND,
        ] {
            let c = code as u32;
            assert!(
                (0xE000..=0xF8FF).contains(&c),
                "glyph U+{c:X} is not a PUA icon code"
            );
        }
    }
}
