//! Function List driven by an installed Notepad++ `functionList\<language>.xml` parser.
//!
//! The XML files are only read at runtime when they exist on the machine (nothing is bundled).
//! Their PCRE patterns are run with `fancy-regex`; a parser that cannot be compiled is ignored and
//! the built-in symbol extractor is used instead.

use super::symbols::{extract_symbols, DocumentSymbol, SymbolKind};
use super::Language;
use fancy_regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

struct Rule {
    main: Regex,
    /// Name extraction expressions, applied one after the other.
    name: Vec<Regex>,
}

struct Parser {
    comment: Option<Regex>,
    classes: Vec<Rule>,
    functions: Vec<Rule>,
}

fn file_stem(lang: Language) -> Option<&'static str> {
    Some(match lang {
        Language::Rust => "rust",
        Language::C => "c",
        Language::Cpp => "cpp",
        Language::Cs => "cs",
        Language::Java => "java",
        Language::Python => "python",
        Language::JavaScript => "javascript.js",
        Language::TypeScript => "typescript",
        Language::Css => "css",
        Language::Xml => "xml",
        Language::Sql => "sql",
        Language::Shell => "bash",
        Language::PowerShell => "powershell",
        Language::Batch => "batch",
        Language::Php => "php",
        Language::Ruby => "ruby",
        Language::Perl => "perl",
        Language::Lua => "lua",
        Language::GdScript => "gdscript",
        Language::Ini => "ini",
        Language::Makefile => "makefile",
        Language::Toml => "toml",
        Language::Assembly => "asm",
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// Minimal raw XML tokenizer (keeps attribute values verbatim: multi-line (?x) patterns matter)
// ---------------------------------------------------------------------------------------------

enum Tok {
    Open(String, Vec<(String, String)>, bool),
    Close(String),
}

fn decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(end) = rest.find(';') {
            let ent = &rest[1..end];
            let rep = match ent {
                "lt" => Some('<'),
                "gt" => Some('>'),
                "amp" => Some('&'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => ent
                    .strip_prefix("#x")
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                    .and_then(char::from_u32),
            };
            if let Some(c) = rep {
                out.push(c);
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

fn tokenize(xml: &str) -> Vec<Tok> {
    let b = xml.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        if xml[i..].starts_with("<!--") {
            i = xml[i..].find("-->").map_or(b.len(), |p| i + p + 3);
            continue;
        }
        if xml[i..].starts_with("<?") || xml[i..].starts_with("<!") {
            i = xml[i..].find('>').map_or(b.len(), |p| i + p + 1);
            continue;
        }
        if b.get(i + 1) == Some(&b'/') {
            let end = xml[i..].find('>').map_or(b.len(), |p| i + p);
            toks.push(Tok::Close(xml[i + 2..end].trim().to_string()));
            i = end + 1;
            continue;
        }
        // Opening tag: name, attributes (quoted values may contain '>')
        let mut j = i + 1;
        while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' && b[j] != b'/' {
            j += 1;
        }
        let name = xml[i + 1..j].to_string();
        let mut attrs = Vec::new();
        let mut self_close = false;
        loop {
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() {
                break;
            }
            if b[j] == b'>' {
                j += 1;
                break;
            }
            if b[j] == b'/' {
                self_close = true;
                j += 1;
                continue;
            }
            let ks = j;
            while j < b.len() && b[j] != b'=' && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                j += 1;
            }
            let key = xml[ks..j].to_string();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < b.len() && b[j] == b'=' {
                j += 1;
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    let vs = j + 1;
                    let ve = b[vs..].iter().position(|c| *c == q).map_or(b.len(), |p| vs + p);
                    attrs.push((key, decode(&xml[vs..ve])));
                    j = (ve + 1).min(b.len());
                }
            }
        }
        toks.push(Tok::Open(name, attrs, self_close));
        i = j;
    }
    toks
}

/// Converts the PCRE-only spellings used by Notepad++ to what `fancy-regex` understands.
fn pcre(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut rest = pattern;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("(?'") {
            if let Some(end) = r.find('\'') {
                out.push_str("(?P<");
                out.push_str(&r[..end]);
                out.push('>');
                rest = &r[end + 1..];
                continue;
            }
        }
        if let Some(r) = rest.strip_prefix("\\k'") {
            if let Some(end) = r.find('\'') {
                out.push_str("(?P=");
                out.push_str(&r[..end]);
                out.push(')');
                rest = &r[end + 1..];
                continue;
            }
        }
        let c = rest.chars().next().unwrap_or(' ');
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

fn compile(p: &str) -> Option<Regex> {
    Regex::new(&format!("(?m){}", pcre(p))).ok()
}

fn parse_parser(xml: &str) -> Option<Parser> {
    let toks = tokenize(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut started = false;
    let mut parser = Parser { comment: None, classes: Vec::new(), functions: Vec::new() };
    // Index of the rule currently receiving name expressions
    enum Target {
        Class(usize),
        Func(usize),
    }
    let mut target: Option<Target> = None;
    for tok in toks {
        match tok {
            Tok::Open(name, attrs, self_close) => {
                let attr = |k: &str| attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
                match name.as_str() {
                    "parser" => {
                        if started {
                            break; // only the first parser of the file
                        }
                        started = true;
                        parser.comment = attr("commentExpr").and_then(compile);
                    }
                    "classRange" if started => {
                        if let Some(re) = attr("mainExpr").and_then(compile) {
                            parser.classes.push(Rule { main: re, name: Vec::new() });
                            target = Some(Target::Class(parser.classes.len() - 1));
                        } else {
                            target = None;
                        }
                    }
                    "function" if started => {
                        if let Some(re) = attr("mainExpr").and_then(compile) {
                            parser.functions.push(Rule { main: re, name: Vec::new() });
                            target = Some(Target::Func(parser.functions.len() - 1));
                        } else {
                            target = None;
                        }
                    }
                    "nameExpr" | "funcNameExpr" if started => {
                        if let (Some(expr), Some(t)) = (attr("expr").and_then(compile), target.as_ref()) {
                            let in_class_name = stack.last().is_some_and(|s| s == "className");
                            let in_func_name = stack.last().is_some_and(|s| s == "functionName");
                            match t {
                                Target::Class(i) if in_class_name => parser.classes[*i].name.push(expr),
                                Target::Func(i) if in_func_name => parser.functions[*i].name.push(expr),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                if !self_close {
                    stack.push(name);
                }
            }
            Tok::Close(name) => {
                if stack.last().is_some_and(|s| *s == name) {
                    stack.pop();
                }
                if name == "classRange" {
                    target = None;
                }
            }
        }
    }
    if parser.functions.is_empty() && parser.classes.is_empty() { None } else { Some(parser) }
}

fn load_parser(lang: Language) -> Option<Arc<Parser>> {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Option<Arc<Parser>>>>> = OnceLock::new();
    let stem = file_stem(lang)?;
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock().ok()?;
    if let Some(p) = map.get(stem) {
        return p.clone();
    }
    let parser = super::dict::search_dirs("functionList")
        .into_iter()
        .map(|d: PathBuf| d.join(format!("{stem}.xml")))
        .find_map(|p| std::fs::read_to_string(p).ok())
        .and_then(|xml| parse_parser(&xml))
        .map(Arc::new);
    map.insert(stem, parser.clone());
    parser
}

fn blank_out(text: &str, re: &Regex) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text).flatten().take(20_000) {
        out.push_str(&text[last..m.start()]);
        out.extend(text[m.start()..m.end()].chars().map(|c| if c == '\n' { '\n' } else { ' ' }));
        last = m.end();
    }
    out.push_str(&text[last..]);
    out
}

fn name_of(rule: &Rule, matched: &str) -> Option<String> {
    let mut current = matched.to_string();
    for re in &rule.name {
        let found = {
            let caps = re.captures(&current).ok().flatten()?;
            caps.get(1).or_else(|| caps.get(0))?.as_str().to_string()
        };
        current = found;
    }
    let name = current.lines().next().unwrap_or("").trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

fn run(parser: &Parser, content: &str) -> Vec<DocumentSymbol> {
    let text = match &parser.comment {
        Some(re) => blank_out(content, re),
        None => content.to_string(),
    };
    let mut starts = vec![0usize];
    starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    let line_of = |pos: usize| match starts.binary_search(&pos) {
        Ok(i) => i + 1,
        Err(i) => i,
    };
    let mut out: Vec<DocumentSymbol> = Vec::new();
    for (rules, kind) in [(&parser.classes, SymbolKind::StructOrClass), (&parser.functions, SymbolKind::Function)] {
        for rule in rules {
            for m in rule.main.find_iter(&text).flatten().take(5_000) {
                if let Some(name) = name_of(rule, m.as_str()) {
                    out.push(DocumentSymbol { name, kind, line: line_of(m.start()) });
                }
            }
        }
    }
    out.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.name.cmp(&b.name)));
    out.dedup_by(|a, b| a.line == b.line && a.name == b.name);
    out
}

/// Symbols for the Function List: Notepad++ parser when installed, built-in rules otherwise.
#[must_use]
pub fn symbols_for(content: &str, lang: Language) -> Vec<DocumentSymbol> {
    if content.len() <= 400_000 {
        if let Some(parser) = load_parser(lang) {
            let mut syms = run(&parser, content);
            if !syms.is_empty() {
                // Keep the built-in type/class entries when the Notepad++ parser has no class rule for them
                let known: Vec<usize> = syms.iter().map(|s| s.line).collect();
                syms.extend(
                    extract_symbols(content, lang)
                        .into_iter()
                        .filter(|s| s.kind != SymbolKind::Function && !known.contains(&s.line)),
                );
                syms.sort_by_key(|s| s.line);
                return syms;
            }
        }
    }
    extract_symbols(content, lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<NotepadPlus><functionList>
        <parser id="t" commentExpr="(?x) (?m-s:\#.*$)">
            <function mainExpr="^def\s+(?'NAME'\w+)\s*\(">
                <functionName><nameExpr expr="\w+\s*\(" /><nameExpr expr="\w+" /></functionName>
            </function>
        </parser></functionList></NotepadPlus>"#;

    #[test]
    fn runs_a_notepad_parser() {
        let parser = parse_parser(XML).expect("parser compiles");
        let code = "# def hidden(x)\ndef visible(a):\n    pass\n";
        let syms = run(&parser, code);
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].line, 2);
    }

    /// Informational: how many of the installed Notepad++ parsers compile (skipped when absent).
    #[test]
    fn installed_parsers_report() {
        let Some(dir) = super::super::dict::search_dirs("functionList").into_iter().find(|d| d.is_dir()) else { return };
        let (mut ok, mut total) = (0, 0);
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            if e.path().extension().is_some_and(|x| x == "xml") && e.file_name() != "overrideMap.xml" {
                total += 1;
                let xml = std::fs::read_to_string(e.path()).unwrap_or_default();
                if parse_parser(&xml).is_some() {
                    ok += 1;
                } else {
                    println!("not usable: {:?}", e.file_name());
                }
            }
        }
        println!("functionList parsers usable: {ok}/{total}");
    }
}
