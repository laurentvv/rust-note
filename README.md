# notepadpp-rs — Notepad++ Clean-Room Reimplementation in Rust

A pure-Rust, modern reimplementation of the Notepad++ source code editor suite, featuring complete GUI interface fidelity, Scintilla-inspired multi-tab buffer architecture, comprehensive text manipulation functions, encoding & EOL conversion, robust search/replace engine, and CLI parameter compatibility.

Created via clean-room reverse engineering and resource reconnaissance with `universal-asset-ripper`.

---

## 🚀 Features

- 📑 **Multi-Document Tab Interface :**
  - Smooth tab switching with file type glyphs (🦀 Rust, 🐍 Python, ⚡ JS/TS, 🌐 HTML/XML, ⚙ C/C++, etc.).
  - Glowing amber dirty indicators (`●`) and hoverable tab close buttons (`×`).
  - Comprehensive tab context menus (Close, Close Others, Copy Full Path, Copy Filename).
  - Recently closed tab restoration (`Ctrl+Shift+T`).
- 🎨 **Modern Aesthetic & Theme Engine :**
  - Obsidian Slate & Tokyo Night dark theme palette (`#181825`, `#1e1e2e`, `#11111b`) and Classic Light theme with instant toggle.
  - Colored status bar pill badges: Line & Column (amber), Document statistics, EOL format, Encoding (lavender), Language (cyan), and Edit Mode (`INS` emerald / `READ ONLY` crimson).
  - Real-time status notification toasts.
- 📋 **Function List Outline & Symbol Navigation :**
  - Sidebar with real-time symbol search filter (`🔍 Filter symbols...`).
  - Colored symbol kinds: Functions (`ƒ`), Structs/Classes (`◈`), Traits/Interfaces (`◇`), Enums/Constants (`≡`).
  - One-click navigation jumping directly to symbol line in document.
- 🧩 **Built-in Tools & Plugins (Recovered via Native NSA Ghidra Decompiler) :**
  - **NppConverter :** Interactive floating **0x Quick Converter** panel with bidirectional real-time synchronization between Decimal, Hexadecimal, Binary, Octal, and ASCII Char; ASCII to HEX, HEX to ASCII, DEC to HEX, HEX to DEC.
  - **NppExport :** RTF export (`.rtf` Rich Text Format) and HTML export with syntax formatting.
  - **NppShell :** Copy Full File Path, Copy Filename, and Copy Directory Path directly to clipboard.
  - **MIME Tools :** Base64 Encode / Decode, URL Encode / Decode, Quoted-Printable Encode.
  - Full Undo / Redo history stack.
  - Multi-encoding support: UTF-8, UTF-8 with BOM, UTF-16 LE, UTF-16 BE, Windows-1252 (ANSI).
  - Multi-EOL support: Windows (`CRLF`), Unix (`LF`), Macintosh (`CR`) with real-time conversion.
  - Line numbers margin, cursor coordinates, and selection statistics.
- 🎨 **Syntax Highlighting & Language Detection :**
  - Built-in syntax highlighting profiles for Rust, Python, C/C++, JavaScript/TypeScript, HTML, XML, JSON, SQL, Shell, and Markdown.
- 🔍 **Full-featured Search & Replace :**
  - Find, Replace, Mark, and Find in Open Files.
  - Regular Expressions, Match Case, Whole Word, and Down/Up direction options.
  - Go to Line dialog (`Ctrl+G`).
- 🛠️ **Notepad++ Text & Line Operations :**
  - Indent & Unindent (`Tab` / `Shift+Tab`).
  - Duplicate Current Line (`Ctrl+D`).
  - Remove Duplicate Lines & Remove Consecutive Duplicates.
  - Move Line Up (`Ctrl+Shift+Up`) & Move Line Down (`Ctrl+Shift+Down`).
  - Sort Lines Lexicographically (Ascending / Descending, Case Sensitive / Insensitive, by Length).
  - Split Lines & Join Lines (`Ctrl+J`).
  - Convert Case: UPPERCASE (`Ctrl+Shift+U`), lowercase (`Ctrl+U`), Proper Case, Invert Case, Random Case.
  - Comments: Line Comment (`Ctrl+Q`), Block Comment (`Ctrl+Shift+K`).
  - Blank Operations: Trim Trailing Spaces, Trim Leading Spaces, TAB to Space, Space to TAB.
  - Date / Time insertion: Short, Long, Customized.
- 🧩 **Built-in Tools & Plugins :**
  - MIME Tools: Base64 Encode / Decode, URL Encode / Decode.
  - Converter: ASCII to HEX, HEX to ASCII.
- 💻 **Notepad++ CLI Flags Compatibility :**
  - `-multiInst`, `-nosession`, `-notabbar`, `-ro`, `-alwaysOnTop`
  - `-n<line>` / `-n=<line>` : Jump directly to line
  - `-c<col>` / `-c=<col>` : Jump directly to column
  - `-p<pos>` : Jump directly to character offset
  - `-l<lang>` : Force syntax highlighting language
  - Files to open as arguments

---

## 📦 Building and Running

Prerequisites: Rust 1.80+ (edition 2024).

```bash
# Build
cargo build --release

# Run GUI
cargo run --release

# Run with files and line target
cargo run --release -- -n42 src/main.rs
```

---

## 📜 Documentation

- [Clean-Room Inventory Reference](docs/inventory.md)
