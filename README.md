# rust-note

**A fast, native, Notepad++-style code and text editor written in pure Rust.**

rust-note is a lightweight desktop editor built with [egui/eframe](https://github.com/emilk/egui). It brings the familiar Notepad++ workflow — tabs, find & replace, line operations, encoding and EOL conversion, a function list and handy text tools — to a single dependency-light binary with a modern dark/light UI.

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust) ![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue) ![License](https://img.shields.io/badge/license-MIT-green)

**Keywords:** rust, text-editor, code-editor, notepad-plus-plus, notepad++, egui, eframe, gui, syntax-highlighting, desktop-app, cross-platform, find-and-replace, regex

---

## Features

### Editing
- Multi-tab interface with dirty indicators, middle-click / `×` close, tab context menu, **Restore Closed Tab** (`Ctrl+Shift+T`) and **drag & drop** to open files.
- **Syntax highlighting** for Rust, C/C++, C#, Java, Python, JavaScript/TypeScript, Go, SQL, HTML/XML, CSS, JSON, YAML, TOML, shell, PowerShell, Batch, Markdown and more.
- Line-number gutter aligned with wrapped lines, current-line highlight, word wrap toggle.
- **Auto-completion** popup (language keywords + words from the document; `Ctrl+Space`, `Tab`/`Enter` to accept).
- If Notepad++ is installed, its `autoCompletion\*.xml` dictionaries are read at runtime to enrich completions (nothing is bundled).
- Tools menu: *Trim Trailing Space and Save* (`Alt+Shift+S`), *Open in New Instance* (`Alt+F6`), look up the current word on the web (`Alt+F1` PHP manual, `Alt+F3` Wikipedia).
- **Auto-indent** on Enter (extra level after `{`, `(`, `[`, `:`).
- Full **undo / redo** history with typing grouped into natural steps.
- **Column (rectangular) selection** with `Alt`+drag: type, delete, copy, cut and paste across all selected lines.
- **6 color themes** (Obsidian Dark, Classic Light, Monokai, Solarized Dark/Light, Zenburn), switchable from the toolbar or View menu.
- Keyboard and mouse selection: `Shift`+arrows, `Ctrl+Shift`+arrows (word), `Shift`+click, double-click (word), triple-click (line), `Ctrl+A`.
- Zoom with `Ctrl`+mouse wheel, `Ctrl`+`+` / `-` / `0`.
- Confirmation prompt before closing modified documents, also on window close.
- **English / French interface** (View > Interface language), using familiar Notepad++ French terminology.
- If Notepad++ is installed, its `functionList\*.xml` parsers (45 of 47 compile; PCRE subroutine calls are emulated by inlining, recursive ones are unrolled two levels) drive the Function List; the built-in rules are the fallback.
- Function List for Rust, Python, JS/TS, Go, C-family, Ruby, Lua, PHP, shell, SQL tables and Markdown headings.
- Session restore (reopens last files), recent files list, persistent settings (theme, zoom, wrap, gutter).

### Search
- Find / Replace with **regular expressions** (capture groups in replacements), match case, whole word, up/down direction and wrap-around.
- Find Next / Previous (`F3` / `Shift+F3`), Replace one, Replace All, Count.
- **Find in Open Documents** with a clickable results list.
- Go to Line (`Ctrl+G`).

### Text tools
- Line operations: duplicate (`Ctrl+D`), delete (`Ctrl+L`), move up/down (`Ctrl+Shift+↑/↓`), join (`Ctrl+J`), sort, remove duplicate lines.
- Case conversion (`Ctrl+U`, `Ctrl+Shift+U`), toggle comment (`Ctrl+Q`), trim spaces, tabs ↔ spaces, insert date/time at the cursor.
- Encodings: UTF-8, UTF-8 BOM, UTF-16 LE/BE, Windows-1252. EOL: CRLF, LF, CR.
- Function List sidebar with symbol filter and one-click navigation.
- Plugins-style tools: Base64, URL and Quoted-Printable encode/decode, ASCII↔HEX, DEC↔HEX, a live **0x multi-base converter**, HTML and RTF export.

### Command line
`-multiInst`, `-nosession`, `-notabbar`, `-ro`, `-alwaysOnTop`, `-n<line>`, `-c<col>`, `-p<pos>`, `-l<language>`, plus files to open.

---

## Install & run

Requires a recent stable Rust toolchain (edition 2024).

```bash
git clone https://github.com/laurentvv/rust-note.git
cd rust-note
cargo run --release -- path/to/file.rs
```

The optimized binary is produced at `target/release/rust-note` (`rust-note.exe` on Windows).

## Keyboard shortcuts

| Action | Shortcut |
|---|---|
| New / Open / Save / Save All | `Ctrl+N` / `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` |
| Close tab / Restore closed tab | `Ctrl+W` / `Ctrl+Shift+T` |
| Next / previous tab | `Ctrl+PageDown` / `Ctrl+PageUp` |
| Find / Replace / Go to line | `Ctrl+F` / `Ctrl+H` / `Ctrl+G` |
| Find next / previous | `F3` / `Shift+F3` |
| Duplicate / delete / move line | `Ctrl+D` / `Ctrl+L` / `Ctrl+Shift+↑↓` |
| Comment / join lines | `Ctrl+Q` / `Ctrl+J` |
| Upper / lower case | `Ctrl+Shift+U` / `Ctrl+U` |
| Function list | `Ctrl+Alt+L` |
| Column selection | `Alt`+drag |
| Auto-completion | `Ctrl+Space` |
| Zoom | `Ctrl+Wheel`, `Ctrl+±`, `Ctrl+0` |

## Project layout

```
src/
  app.rs        egui application, menus, shortcuts, dialogs
  buffer/       documents, encodings, EOL, undo history
  search/       regex search & replace engine
  syntax/       language detection, highlighter, symbol extraction
  plugins/      Base64 / URL / hex converters and exporters
  settings.rs   persisted preferences and session
```

Run the test suite with `cargo test`.

## Known limitations

- With word wrap on, a column block applies to whole logical lines (tabs are handled as 4 visual columns).
- Two Notepad++ function-list parsers (Nim, SQL) need numbered recursion or PCRE conditionals and fall back to the built-in rules.

## License

Released under the MIT License — see [LICENSE](LICENSE). rust-note is an independent project and is not affiliated with Notepad++.
