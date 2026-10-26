//! Optional auto-completion dictionaries read from an installed Notepad++
//! (`autoCompletion\<language>.xml`). Nothing is bundled: the files are only read at runtime
//! when they exist on the machine.

use super::Language;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn file_stem(lang: Language) -> Option<&'static str> {
    Some(match lang {
        Language::Rust => "rust",
        Language::C => "c",
        Language::Cpp => "cpp",
        Language::Cs => "cs",
        Language::Java => "java",
        Language::Python => "python",
        Language::JavaScript => "javascript",
        Language::TypeScript => "typescript",
        Language::Go => "go",
        Language::Html => "html",
        Language::Css => "css",
        Language::Xml => "xml",
        Language::Sql => "sql",
        Language::PowerShell => "powershell",
        Language::Batch => "batch",
        Language::Php => "php",
        Language::Lua => "lua",
        Language::Perl => "perl",
        Language::GdScript => "gdscript",
        _ => return None,
    })
}

fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for var in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Some(p) = std::env::var_os(var) {
            dirs.push(PathBuf::from(p).join("Notepad++").join("autoCompletion"));
        }
    }
    if let Some(p) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(p).join("Notepad++").join("autoCompletion"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.join("autoCompletion"));
        }
    }
    dirs
}

/// Extracts every `<KeyWord name="...">` entry of a Notepad++ auto-completion file.
#[must_use]
pub fn parse_keywords(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("<KeyWord ") {
        rest = &rest[pos + 9..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let tag = &rest[..tag_end];
        if let Some(n) = tag.find("name=\"") {
            let value = &tag[n + 6..];
            if let Some(end) = value.find('"') {
                let name = value[..end]
                    .replace("&amp;", "&")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&quot;", "\"");
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':') {
                    out.push(name);
                }
            }
        }
        rest = &rest[tag_end..];
    }
    out
}

/// Returns the (cached) word list for a language, empty when no dictionary is installed.
#[must_use]
pub fn dictionary(lang: Language) -> Vec<String> {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Vec<String>>>> = OnceLock::new();
    let Some(stem) = file_stem(lang) else { return Vec::new() };
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut map) = cache.lock() else { return Vec::new() };
    if let Some(v) = map.get(stem) {
        return v.clone();
    }
    let words = search_dirs()
        .into_iter()
        .map(|d| d.join(format!("{stem}.xml")))
        .find_map(|p| std::fs::read_to_string(p).ok())
        .map(|xml| parse_keywords(&xml))
        .unwrap_or_default();
    map.insert(stem, words.clone());
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keyword_entries() {
        let xml = r#"<AutoComplete><KeyWord name="fn" /><KeyWord name="a&amp;b" func="yes"><Overload/></KeyWord><KeyWord name="bad name"/></AutoComplete>"#;
        assert_eq!(parse_keywords(xml), vec!["fn".to_string()]);
    }
}
