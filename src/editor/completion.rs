//! Auto-completion engine: candidate collection and popup state.

use crate::buffer::Document;
use crate::syntax::Language;
use eframe::egui;

/// Auto-completion popup state (candidates for the word being typed).
#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    pub items: Vec<String>,
    pub selected: usize,
    /// Caret position (char index) the candidates were computed for.
    pub at: usize,
}

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Keywords of the language plus words already present in the document that start with the
/// word before the caret.
pub(crate) fn completion_candidates(content: &str, caret: usize, lang: Language, forced: bool) -> Vec<String> {
    if content.len() > 2_000_000 {
        return Vec::new();
    }
    let chars: Vec<char> = content.chars().collect();
    let end = caret.min(chars.len());
    let mut start = end;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let prefix: String = chars[start..end].iter().collect();
    if prefix.chars().count() < 1 {
        return Vec::new();
    }
    let ci = matches!(lang, Language::Sql | Language::Batch);
    let matches_prefix = |w: &str| {
        let ok = if ci { w.to_lowercase().starts_with(&prefix.to_lowercase()) } else { w.starts_with(&prefix) };
        ok && w != prefix
    };
    let mut set = std::collections::BTreeSet::new();
    for kw in lang.keywords() {
        if matches_prefix(kw) {
            set.insert((*kw).to_string());
        }
    }
    let _ = forced;
    for kw in crate::syntax::dict::dictionary(lang) {
        if matches_prefix(&kw) {
            set.insert(kw);
        }
    }
    let mut word = String::new();
    let mut word_start = 0usize;
    for (i, c) in chars.iter().copied().chain(std::iter::once(' ')).enumerate() {
        if is_word_char(c) {
            if word.is_empty() {
                word_start = i;
            }
            word.push(c);
        } else if !word.is_empty() {
            // Skip the word currently being typed
            if word_start != start && word.chars().count() >= 3 && matches_prefix(&word) {
                set.insert(word.clone());
            }
            word.clear();
        }
    }
    set.into_iter().take(8).collect()
}

pub(crate) fn apply_completion(doc: &mut Document, item: &str, ctx: &egui::Context, id: egui::Id) {
    let chars: Vec<char> = doc.content.chars().collect();
    let end = doc.cursor_char.min(chars.len());
    let mut start = end;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut out: String = chars[..start].iter().collect();
    out.push_str(item);
    out.extend(chars[end..].iter());
    doc.content = out;
    doc.record_typing();
    let pos = start + item.chars().count();
    let mut st = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    st.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(pos))));
    st.store(ctx, id);
    doc.cursor_char = pos;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_uses_keywords_and_document_words() {
        let text = "let alphabet = 1;\nal";
        let caret = text.chars().count();
        let items = completion_candidates(text, caret, Language::Rust, false);
        assert!(items.contains(&"alphabet".to_string()));
        let items = completion_candidates("le", 2, Language::Rust, false);
        assert!(items.contains(&"let".to_string()));
        assert!(completion_candidates("", 0, Language::Rust, false).is_empty());
    }
}
