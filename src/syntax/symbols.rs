//! Outline symbol extraction for the Function List panel.
//! Inspired by Notepad++ `functionList/*.xml` parser patterns.

use super::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    StructOrClass,
    InterfaceOrTrait,
    Enum,
}

impl SymbolKind {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Function => "ƒ",
            Self::StructOrClass => "◈",
            Self::InterfaceOrTrait => "◇",
            Self::Enum => "≡",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize, // 1-based line number
}

/// Extract symbols (functions, classes, structs) from document content.
#[must_use]
pub fn extract_symbols(content: &str, lang: Language) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for (zero_idx, line) in content.lines().enumerate() {
        let line_num = zero_idx + 1;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("--")
        {
            continue;
        }

        match lang {
            Language::Rust => {
                if let Some(rest) = trimmed.strip_prefix("pub fn ").or_else(|| trimmed.strip_prefix("fn ")) {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub struct ").or_else(|| trimmed.strip_prefix("struct ")) {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub enum ").or_else(|| trimmed.strip_prefix("enum ")) {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Enum, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub trait ").or_else(|| trimmed.strip_prefix("trait ")) {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::InterfaceOrTrait, line: line_num });
                    }
                }
            }
            Language::Python => {
                if let Some(rest) = trimmed.strip_prefix("def ") {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("class ") {
                    if let Some(name) = extract_identifier_before(rest, ':').or_else(|| extract_identifier_before(rest, '(')) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                }
            }
            Language::JavaScript | Language::TypeScript => {
                if let Some(rest) = trimmed.strip_prefix("function ") {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("class ") {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("export function ") {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("interface ") {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::InterfaceOrTrait, line: line_num });
                    }
                }
            }
            Language::Go => {
                if let Some(rest) = trimmed.strip_prefix("func ") {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if trimmed.contains("struct {") {
                    if let Some(name) = extract_first_word(trimmed.strip_prefix("type ").unwrap_or(trimmed)) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                }
            }
            Language::GdScript => {
                if let Some(rest) = trimmed.strip_prefix("func ") {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("class_name ") {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                }
            }
            _ => {
                // Generic heuristic for C/C++/Java/C#/PHP/Ruby/Lua
                if let Some(rest) = trimmed.strip_prefix("def ").or_else(|| trimmed.strip_prefix("function ")) {
                    if let Some(name) = extract_identifier_before(rest, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("class ") {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                }
            }
        }
    }

    symbols
}

fn extract_identifier_before(s: &str, delimiter: char) -> Option<String> {
    let part = s.split(delimiter).next()?.trim();
    // take the last word before delimiter, e.g. "pub async fn my_func" or "my_func"
    let word = part.split_whitespace().last()?;
    let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}

fn extract_first_word(s: &str) -> Option<String> {
    let word = s.split_whitespace().next()?;
    let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_symbols() {
        let code = r#"
pub struct EditorConfig {
    pub font_size: f32,
}

pub fn render_ui() {
    println!("Hello");
}

fn helper_calc() -> usize {
    42
}
"#;
        let symbols = extract_symbols(code, Language::Rust);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "EditorConfig");
        assert_eq!(symbols[0].kind, SymbolKind::StructOrClass);
        assert_eq!(symbols[1].name, "render_ui");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert_eq!(symbols[2].name, "helper_calc");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_python_symbols() {
        let code = r#"
class DocumentParser:
    def parse_file(self):
        pass

def main():
    print("start")
"#;
        let symbols = extract_symbols(code, Language::Python);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "DocumentParser");
        assert_eq!(symbols[1].name, "parse_file");
        assert_eq!(symbols[2].name, "main");
    }
}
