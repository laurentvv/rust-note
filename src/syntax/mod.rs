//! Syntax highlighting definitions and token classification.
//! Recovered from Notepad++ `langs.model.xml`.

use std::path::Path;

pub mod symbols;
pub use symbols::{extract_symbols, DocumentSymbol, SymbolKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    PlainText,
    Rust,
    C,
    Cpp,
    Cs,
    Java,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Html,
    Css,
    Xml,
    Json,
    Yaml,
    Toml,
    Sql,
    Shell,
    PowerShell,
    Batch,
    Php,
    Ruby,
    Perl,
    Lua,
    Swift,
    Kotlin,
    Dart,
    R,
    GdScript,
    Markdown,
    Assembly,
    Ini,
    Diff,
    Makefile,
}

impl std::str::FromStr for Language {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "rust" | "rs" => Self::Rust,
            "c" => Self::C,
            "cpp" | "c++" | "cxx" => Self::Cpp,
            "cs" | "c#" | "csharp" => Self::Cs,
            "java" => Self::Java,
            "python" | "py" => Self::Python,
            "javascript" | "js" => Self::JavaScript,
            "typescript" | "ts" => Self::TypeScript,
            "go" | "golang" => Self::Go,
            "html" => Self::Html,
            "css" => Self::Css,
            "xml" => Self::Xml,
            "json" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "sql" => Self::Sql,
            "sh" | "bash" | "shell" => Self::Shell,
            "powershell" | "ps" | "ps1" => Self::PowerShell,
            "batch" | "bat" | "cmd" => Self::Batch,
            "php" => Self::Php,
            "ruby" | "rb" => Self::Ruby,
            "perl" | "pl" => Self::Perl,
            "lua" => Self::Lua,
            "swift" => Self::Swift,
            "kotlin" | "kt" => Self::Kotlin,
            "dart" => Self::Dart,
            "r" => Self::R,
            "gdscript" | "gd" => Self::GdScript,
            "md" | "markdown" => Self::Markdown,
            "asm" | "assembly" => Self::Assembly,
            "ini" | "cfg" => Self::Ini,
            "diff" | "patch" => Self::Diff,
            "makefile" | "make" => Self::Makefile,
            _ => Self::PlainText,
        })
    }
}

impl Language {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PlainText => "Normal Text",
            Self::Rust => "Rust",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Cs => "C#",
            Self::Java => "Java",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Xml => "XML",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Sql => "SQL",
            Self::Shell => "Bash / Shell",
            Self::PowerShell => "PowerShell",
            Self::Batch => "Batch",
            Self::Php => "PHP",
            Self::Ruby => "Ruby",
            Self::Perl => "Perl",
            Self::Lua => "Lua",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::Dart => "Dart",
            Self::R => "R",
            Self::GdScript => "GDScript",
            Self::Markdown => "Markdown",
            Self::Assembly => "Assembly",
            Self::Ini => "INI / Config",
            Self::Diff => "Diff / Patch",
            Self::Makefile => "Makefile",
        }
    }

    /// Detects language by file extension from Notepad++ `langs.model.xml`.
    #[must_use]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => Self::Rust,
            "c" | "h" | "lex" => Self::C,
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "ino" => Self::Cpp,
            "cs" => Self::Cs,
            "java" => Self::Java,
            "py" | "pyw" => Self::Python,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "go" => Self::Go,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "xml" | "svg" | "xaml" | "res" => Self::Xml,
            "json" | "json5" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "sql" => Self::Sql,
            "sh" | "bash" | "zsh" | "bsh" => Self::Shell,
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
            "bat" | "cmd" | "nt" => Self::Batch,
            "php" | "phtml" | "php3" | "php4" | "php5" => Self::Php,
            "rb" | "rbw" => Self::Ruby,
            "pl" | "pm" | "pod" => Self::Perl,
            "lua" => Self::Lua,
            "swift" => Self::Swift,
            "kt" | "kts" => Self::Kotlin,
            "dart" => Self::Dart,
            "r" => Self::R,
            "gd" => Self::GdScript,
            "md" | "markdown" => Self::Markdown,
            "asm" | "s" => Self::Assembly,
            "ini" | "cfg" | "conf" | "properties" => Self::Ini,
            "diff" | "patch" => Self::Diff,
            "mk" | "mak" => Self::Makefile,
            _ => {
                let filename = path
                    .as_ref()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if filename.eq_ignore_ascii_case("makefile") {
                    Self::Makefile
                } else if filename.eq_ignore_ascii_case("dockerfile") {
                    Self::Shell
                } else {
                    Self::PlainText
                }
            }
        }
    }

    /// Parses language from string (e.g. CLI `-l<lang>`).
    #[must_use]
    pub fn parse_language(s: &str) -> Self {
        s.parse().unwrap_or(Self::PlainText)
    }

    #[must_use]
    pub const fn comment_prefix(self) -> &'static str {
        match self {
            Self::Rust
            | Self::C
            | Self::Cpp
            | Self::Cs
            | Self::Java
            | Self::JavaScript
            | Self::TypeScript
            | Self::Go
            | Self::Php
            | Self::Swift
            | Self::Kotlin
            | Self::Dart
            | Self::Json => "//",
            Self::Python
            | Self::Shell
            | Self::PowerShell
            | Self::Yaml
            | Self::Toml
            | Self::Ruby
            | Self::Perl
            | Self::R
            | Self::GdScript
            | Self::Makefile => "#",
            Self::Sql | Self::Lua => "--",
            Self::Batch => "REM ",
            Self::Assembly | Self::Ini => ";",
            Self::Html | Self::Css | Self::Xml | Self::Markdown => "<!--",
            Self::Diff => " ",
            Self::PlainText => "//",
        }
    }

    #[must_use]
    pub fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[
                "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
                "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
                "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
                "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
            ],
            Self::C | Self::Cpp => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
                "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
                "union", "unsigned", "void", "volatile", "while", "class", "public", "private",
                "protected", "virtual", "namespace", "using", "template", "typename", "new", "delete",
            ],
            Self::Cs => &[
                "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
                "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
                "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
                "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is",
                "lock", "long", "namespace", "new", "null", "object", "operator", "out", "override",
                "params", "private", "protected", "public", "readonly", "ref", "return", "sbyte",
                "sealed", "short", "sizeof", "stackalloc", "static", "string", "struct", "switch",
                "this", "throw", "true", "try", "typeof", "uint", "ulong", "unchecked", "unsafe",
                "ushort", "using", "virtual", "void", "volatile", "while", "async", "await", "var",
            ],
            Self::Java => &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private", "protected", "public",
                "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
                "throw", "throws", "transient", "try", "void", "volatile", "while", "true", "false", "null",
            ],
            Self::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global",
                "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass",
                "raise", "return", "True", "try", "while", "with", "yield",
            ],
            Self::JavaScript | Self::TypeScript => &[
                "async", "await", "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "export", "extends", "false", "finally", "for",
                "function", "if", "import", "in", "instanceof", "let", "new", "null", "return",
                "super", "switch", "this", "throw", "true", "try", "typeof", "var", "void",
                "while", "with", "yield", "interface", "type", "namespace",
            ],
            Self::Go => &[
                "break", "default", "func", "interface", "select", "case", "defer", "go", "map",
                "struct", "chan", "else", "goto", "package", "switch", "const", "fallthrough",
                "if", "range", "type", "continue", "for", "import", "return", "var", "true", "false", "nil",
            ],
            Self::Sql => &[
                "SELECT", "FROM", "WHERE", "INSERT", "INTO", "UPDATE", "DELETE", "JOIN", "LEFT",
                "RIGHT", "INNER", "OUTER", "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT",
                "OFFSET", "CREATE", "TABLE", "DROP", "ALTER", "PRIMARY", "KEY", "FOREIGN",
            ],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path("main.rs"), Language::Rust);
        assert_eq!(Language::from_path("script.py"), Language::Python);
        assert_eq!(Language::from_path("app.ts"), Language::TypeScript);
        assert_eq!(Language::from_path("server.go"), Language::Go);
        assert_eq!(Language::from_path("style.css"), Language::Css);
        assert_eq!(Language::from_path("config.toml"), Language::Toml);
        assert_eq!(Language::from_path("deploy.yaml"), Language::Yaml);
        assert_eq!(Language::from_path("tool.ps1"), Language::PowerShell);
        assert_eq!(Language::from_path("build.bat"), Language::Batch);
        assert_eq!(Language::from_path("player.gd"), Language::GdScript);
        assert_eq!(Language::from_path("Makefile"), Language::Makefile);
        assert_eq!(Language::from_path("notes.txt"), Language::PlainText);
    }

    #[test]
    fn test_comment_prefixes() {
        assert_eq!(Language::Rust.comment_prefix(), "//");
        assert_eq!(Language::Python.comment_prefix(), "#");
        assert_eq!(Language::PowerShell.comment_prefix(), "#");
        assert_eq!(Language::Batch.comment_prefix(), "REM ");
        assert_eq!(Language::Sql.comment_prefix(), "--");
        assert_eq!(Language::Assembly.comment_prefix(), ";");
    }

    #[test]
    fn test_parse_language_flag() {
        assert_eq!(Language::parse_language("rust"), Language::Rust);
        assert_eq!(Language::parse_language("python"), Language::Python);
        assert_eq!(Language::parse_language("c#"), Language::Cs);
        assert_eq!(Language::parse_language("powershell"), Language::PowerShell);
        assert_eq!(Language::parse_language("golang"), Language::Go);
    }
}
