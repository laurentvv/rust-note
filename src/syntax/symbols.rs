//! Outline symbol extraction for the Function List panel.

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

        if lang == Language::Markdown {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level > 0 && level <= 6 && trimmed[level..].starts_with(' ') {
                let name = trimmed[level..].trim().to_string();
                symbols.push(DocumentSymbol { name, kind: if level == 1 { SymbolKind::StructOrClass } else { SymbolKind::Function }, line: line_num });
            }
            continue;
        }

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
                    if let Some(name) = extract_first_word(rest) {
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
                // Generic heuristic for the remaining languages (C-family, Ruby, Lua, PHP, shell, SQL...)
                const MODS: [&str; 18] = [
                    "public ", "private ", "protected ", "internal ", "static ", "async ", "export ",
                    "default ", "final ", "abstract ", "override ", "open ", "sealed ", "local ",
                    "extern ", "virtual ", "inline ", "partial ",
                ];
                let mut t = trimmed;
                while let Some(rest) = MODS.iter().find_map(|m| t.strip_prefix(m)) {
                    t = rest.trim_start();
                }
                let func_kw = ["def ", "function ", "func ", "fun ", "sub ", "fn ", "proc "];
                let class_kw = ["class ", "module ", "object ", "struct ", "record ", "namespace "];
                let iface_kw = ["interface ", "trait ", "protocol "];
                if let Some(rest) = func_kw.iter().find_map(|k| t.strip_prefix(k)) {
                    let name = if rest.contains('(') { extract_identifier_before(rest, '(') } else { extract_first_word(rest) };
                    if let Some(name) = name {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
                    }
                } else if let Some(rest) = class_kw.iter().find_map(|k| t.strip_prefix(k)) {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::StructOrClass, line: line_num });
                    }
                } else if let Some(rest) = iface_kw.iter().find_map(|k| t.strip_prefix(k)) {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::InterfaceOrTrait, line: line_num });
                    }
                } else if let Some(rest) = t.strip_prefix("enum ") {
                    if let Some(name) = extract_first_word(rest) {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Enum, line: line_num });
                    }
                } else if matches!(lang, Language::Sql) {
                    let prefix = "CREATE OR REPLACE ";
                    let normalized: String = if t.len() >= prefix.len() && t[..prefix.len()].eq_ignore_ascii_case(prefix) {
                        format!("CREATE {}", &t[prefix.len()..])
                    } else {
                        t.to_string()
                    };
                    let t = normalized.as_str();
                    let upper = t.to_uppercase();
                    for (kw, kind) in [
                        ("CREATE TABLE ", SymbolKind::StructOrClass),
                        ("CREATE VIEW ", SymbolKind::StructOrClass),
                        ("CREATE FUNCTION ", SymbolKind::Function),
                        ("CREATE PROCEDURE ", SymbolKind::Function),
                    ] {
                        if upper.starts_with(kw) {
                            let rest = &t[kw.len()..];
                            let rest = rest.strip_prefix("IF NOT EXISTS ").unwrap_or(rest);
                            if let Some(name) = rest.split(|c: char| c == '(' || c.is_whitespace()).next().filter(|n| !n.is_empty()) {
                                symbols.push(DocumentSymbol { name: name.trim_matches(|c| c == '[' || c == ']' || c == '`' || c == '"').to_string(), kind, line: line_num });
                            }
                            break;
                        }
                    }
                } else if matches!(lang, Language::Shell | Language::PowerShell) {
                    if let Some(idx) = t.find("()") {
                        let name = t[..idx].trim();
                        if !name.is_empty() && !name.contains(' ') {
                            symbols.push(DocumentSymbol { name: name.to_string(), kind: SymbolKind::Function, line: line_num });
                        }
                    }
                } else if matches!(lang, Language::C | Language::Cpp | Language::Cs | Language::Java | Language::Kotlin | Language::Swift | Language::Dart | Language::Php)
                    && !line.starts_with([' ', '\t'])
                    && t.contains('(')
                    && (t.ends_with(')') || t.ends_with('{'))
                    && !t.ends_with(';')
                    && !["if", "for", "while", "switch", "return", "else", "do", "catch"].iter().any(|k| t.starts_with(k))
                {
                    if let Some(name) = extract_identifier_before(t, '(') {
                        symbols.push(DocumentSymbol { name, kind: SymbolKind::Function, line: line_num });
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
    let clean: String = word
        .trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_')
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if clean.is_empty() { None } else { Some(clean) }
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

    #[test]
    fn test_python_class_with_bases() {
        let syms = extract_symbols("class Alpha(Base):
    pass
", Language::Python);
        assert_eq!(syms[0].name, "Alpha");
    }

    #[test]
    fn test_generic_symbols() {
        let rb = "class Foo\n  def bar(x)\n  end\nend\n";
        let syms = extract_symbols(rb, Language::Ruby);
        assert_eq!(syms.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["Foo", "bar"]);
        let c = "int main(int argc, char **argv) {\n  if (x) {\n  }\n}\nstatic void helper(void)\n{\n}\n";
        let syms = extract_symbols(c, Language::C);
        assert_eq!(syms.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["main", "helper"]);
        let md = "# Title\n## Part\ntext\n";
        assert_eq!(extract_symbols(md, Language::Markdown).len(), 2);
        let sql = "CREATE TABLE users (id int);\n";
        assert_eq!(extract_symbols(sql, Language::Sql)[0].name, "users");
    }
}
