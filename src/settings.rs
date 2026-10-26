//! Persistent user settings and session (stored as JSON in the user's config directory).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub theme: crate::ui::themes::EditorTheme,
    pub lang: crate::i18n::UiLang,
    pub font_size: f32,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_function_list: bool,
    pub recent_files: Vec<PathBuf>,
    pub session_files: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: crate::ui::themes::EditorTheme::DarkMode,
            lang: crate::i18n::UiLang::English,
            font_size: 14.0,
            word_wrap: true,
            show_line_numbers: true,
            show_function_list: false,
            recent_files: Vec::new(),
            session_files: Vec::new(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("rust-note").join("settings.json"))
}

impl Settings {
    #[must_use]
    pub fn load() -> Self {
        settings_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(p) = settings_path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, json);
            }
        }
    }

    /// Moves `path` to the front of the recent files list (max 12 entries).
    pub fn push_recent(&mut self, path: &std::path::Path) {
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_path_buf());
        self.recent_files.truncate(12);
    }
}
