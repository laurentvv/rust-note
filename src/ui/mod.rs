//! User interface components.

pub mod dialogs;
pub mod editor;
pub mod fonts;
pub mod icon;
pub mod menu;
pub mod palette;
pub mod sidebar;
pub mod status;
pub mod tabs;
pub mod themes;
pub mod toolbar;
pub mod windows;

use crate::syntax::Language;
use eframe::egui::Color32;

/// Darkens pastel UI accents on the light theme so they stay readable.
pub(crate) fn tc(light: bool, r: u8, g: u8, b: u8) -> Color32 {
    if light {
        Color32::from_rgb(
            (f32::from(r) * 0.45) as u8,
            (f32::from(g) * 0.45) as u8,
            (f32::from(b) * 0.55) as u8,
        )
    } else {
        Color32::from_rgb(r, g, b)
    }
}

/// Emoji marker for a document language (tabs, status bar).
pub(crate) fn language_icon(lang: Option<Language>) -> &'static str {
    match lang {
        Some(Language::Rust) => "🦀",
        Some(Language::Python) => "🐍",
        Some(Language::JavaScript | Language::TypeScript) => "⚡",
        Some(Language::C | Language::Cpp | Language::Cs) => "⚙",
        Some(Language::Html | Language::Xml) => "🌐",
        Some(Language::Markdown) => "📝",
        Some(Language::Shell | Language::PowerShell | Language::Batch) => "💻",
        Some(Language::Sql) => "🗄",
        Some(Language::Json | Language::Toml | Language::Yaml | Language::Ini) => "🔧",
        Some(Language::Java | Language::Kotlin) => "☕",
        Some(Language::Go) => "🐹",
        Some(Language::Lua | Language::GdScript) => "🎮",
        Some(Language::Assembly) => "🔩",
        _ => "📄",
    }
}
