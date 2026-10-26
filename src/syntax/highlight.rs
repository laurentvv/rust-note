//! Lightweight lexical highlighter producing an egui `LayoutJob`.

use super::Language;
use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId};

#[derive(Clone, Copy)]
pub struct Palette {
    pub text: Color32,
    pub keyword: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub number: Color32,
    pub type_name: Color32,
}

impl Palette {
    pub const DARK: Self = Self {
        text: Color32::from_rgb(205, 214, 244),
        keyword: Color32::from_rgb(203, 166, 247),
        string: Color32::from_rgb(166, 227, 161),
        comment: Color32::from_rgb(108, 112, 134),
        number: Color32::from_rgb(250, 179, 135),
        type_name: Color32::from_rgb(137, 220, 235),
    };
    pub const LIGHT: Self = Self {
        text: Color32::from_rgb(40, 42, 54),
        keyword: Color32::from_rgb(136, 57, 239),
        string: Color32::from_rgb(64, 160, 43),
        comment: Color32::from_rgb(140, 143, 161),
        number: Color32::from_rgb(223, 142, 29),
        type_name: Color32::from_rgb(4, 165, 229),
    };
}

fn block_comment(lang: Language) -> Option<(&'static str, &'static str)> {
    match lang {
        Language::Rust | Language::C | Language::Cpp | Language::Cs | Language::Java
        | Language::JavaScript | Language::TypeScript | Language::Go | Language::Css
        | Language::Php | Language::Swift | Language::Kotlin | Language::Dart | Language::Sql => {
            Some(("/*", "*/"))
        }
        Language::Html | Language::Xml | Language::Markdown => Some(("<!--", "-->")),
        _ => None,
    }
}

fn line_comment(lang: Language) -> Option<&'static str> {
    match lang {
        Language::PlainText | Language::Html | Language::Xml | Language::Markdown | Language::Css
        | Language::Diff => None,
        Language::Batch => Some("REM "),
        other => Some(other.comment_prefix()),
    }
}

fn push(job: &mut LayoutJob, text: &str, color: Color32, font: &FontId) {
    if text.is_empty() {
        return;
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font.clone(),
            color,
            ..Default::default()
        },
    );
}

/// Builds a colored layout job for `text`.
pub fn highlight(text: &str, lang: Language, font: &FontId, pal: Palette, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let keywords = lang.keywords();
    if lang == Language::PlainText || text.len() > 600_000 {
        push(&mut job, text, pal.text, font);
        return job;
    }
    let case_insensitive = matches!(lang, Language::Sql);
    let lc = line_comment(lang);
    let bc = block_comment(lang);
    let hash_strings = !matches!(lang, Language::Batch | Language::Ini | Language::Diff);

    let bytes = text.as_bytes();
    let mut i = 0;
    let mut plain_start = 0;
    macro_rules! flush {
        ($end:expr) => {
            push(&mut job, &text[plain_start..$end], pal.text, font);
        };
    }

    while i < text.len() {
        let rest = &text[i..];
        let b = bytes[i];

        if let Some((open, close)) = bc {
            if rest.starts_with(open) {
                flush!(i);
                let end = rest[open.len()..].find(close).map_or(text.len(), |p| i + open.len() + p + close.len());
                push(&mut job, &text[i..end], pal.comment, font);
                i = end;
                plain_start = i;
                continue;
            }
        }
        if let Some(prefix) = lc {
            let p = prefix.trim_end();
            let is_word = p.chars().all(char::is_alphabetic);
            if rest.starts_with(p) && (!is_word || rest.len() == p.len() || rest[p.len()..].starts_with(' ')) {
                flush!(i);
                let end = rest.find('\n').map_or(text.len(), |p| i + p);
                push(&mut job, &text[i..end], pal.comment, font);
                i = end;
                plain_start = i;
                continue;
            }
        }
        if hash_strings && (b == b'"' || b == b'\'' || b == b'`') {
            // Apostrophes in Rust lifetimes/chars: only treat as string if closed on same line.
            let line_end = rest.find('\n').map_or(text.len(), |p| i + p);
            let mut j = i + 1;
            let mut closed = None;
            while j < line_end {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if bytes[j] == b {
                    closed = Some(j + 1);
                    break;
                }
                j += 1;
            }
            let multi = b == b'`' || (b == b'"' && closed.is_none() && matches!(lang, Language::Python | Language::Rust | Language::JavaScript | Language::TypeScript));
            let end = if let Some(e) = closed {
                Some(e.min(text.len()))
            } else if multi {
                Some(text[i + 1..].find(b as char).map_or(text.len(), |p| i + 1 + p + 1))
            } else {
                None
            };
            let ok = end.is_some() && !(b == b'\'' && lang == Language::Rust && end.is_some_and(|e| e - i > 4));
            if ok {
                let end = end.unwrap_or(i + 1);
                flush!(i);
                push(&mut job, &text[i..end], pal.string, font);
                i = end;
                plain_start = i;
                continue;
            }
        }
        if b.is_ascii_digit() && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_')) {
            flush!(i);
            let mut j = i;
            while j < text.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'.' || bytes[j] == b'_') {
                j += 1;
            }
            push(&mut job, &text[i..j], pal.number, font);
            i = j;
            plain_start = i;
            continue;
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            let mut j = i;
            while j < text.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let word = &text[i..j];
            let is_kw = if case_insensitive {
                keywords.iter().any(|k| k.eq_ignore_ascii_case(word))
            } else {
                keywords.contains(&word)
            };
            let types_ok = !matches!(
                lang,
                Language::Markdown | Language::Html | Language::Xml | Language::Ini | Language::Diff
                    | Language::Json | Language::Yaml | Language::Toml | Language::Css
            );
            let is_type = types_ok && !is_kw && word.len() > 1 && word.as_bytes()[0].is_ascii_uppercase()
                && word.bytes().any(|c| c.is_ascii_lowercase());
            if is_kw || is_type {
                flush!(i);
                push(&mut job, word, if is_kw { pal.keyword } else { pal.type_name }, font);
                plain_start = j;
            }
            i = j;
            continue;
        }
        // advance one UTF-8 char
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    flush!(text.len());
    job
}
