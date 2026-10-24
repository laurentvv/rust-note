//! Syntax highlighting definitions and token classification.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    PlainText,
    Rust,
    C,
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Html,
    Xml,
    Json,
    Sql,
    Shell,
    Markdown,
}

impl std::str::FromStr for Language {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "rust" | "rs" => Self::Rust,
            "c" => Self::C,
            "cpp" | "c++" | "cxx" => Self::Cpp,
            "python" | "py" => Self::Python,
            "javascript" | "js" => Self::JavaScript,
            "typescript" | "ts" => Self::TypeScript,
            "html" => Self::Html,
            "xml" => Self::Xml,
            "json" => Self::Json,
            "sql" => Self::Sql,
            "sh" | "bash" | "shell" => Self::Shell,
            "md" | "markdown" => Self::Markdown,
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
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Html => "HTML",
            Self::Xml => "XML",
            Self::Json => "JSON",
            Self::Sql => "SQL",
            Self::Shell => "Bash / Shell",
            Self::Markdown => "Markdown",
        }
    }

    /// Detects language by file extension.
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
            "c" | "h" => Self::C,
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Self::Cpp,
            "py" | "pyw" => Self::Python,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "html" | "htm" => Self::Html,
            "xml" | "svg" | "xaml" | "res" => Self::Xml,
            "json" => Self::Json,
            "sql" => Self::Sql,
            "sh" | "bash" | "zsh" => Self::Shell,
            "md" | "markdown" => Self::Markdown,
            _ => Self::PlainText,
        }
    }

    /// Parses language from string (e.g. CLI `-l<lang>`).
    #[must_use]
    pub fn parse_language(s: &str) -> Self {
        s.parse().unwrap_or(Self::PlainText)
    }

    #[must_use]
    pub fn comment_prefix(self) -> &'static str {
        match self {
            Self::Rust | Self::C | Self::Cpp | Self::JavaScript | Self::TypeScript | Self::Json => "//",
            Self::Python | Self::Shell => "#",
            Self::Sql => "--",
            Self::Html | Self::Xml | Self::Markdown => "<!--",
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
            Self::Sql => &[
                "SELECT", "FROM", "WHERE", "INSERT", "INTO", "UPDATE", "DELETE", "JOIN", "LEFT",
                "RIGHT", "INNER", "OUTER", "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT",
                "OFFSET", "CREATE", "TABLE", "DROP", "ALTER", "PRIMARY", "KEY", "FOREIGN",
            ],
            _ => &[],
        }
    }
}
