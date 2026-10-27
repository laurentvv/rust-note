# rust-note

**A fast, native, Notepad++-style code and text editor written in pure Rust.**

rust-note is a lightweight desktop editor built with [egui/eframe](https://github.com/emilk/egui). It brings the familiar Notepad++ workflow — tabs, find & replace, line operations, encoding and EOL conversion, a function list and handy text tools — to a single dependency-light binary with a modern dark/light UI.

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust) ![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue) ![License](https://img.shields.io/badge/license-MIT-green)

**Keywords:** rust, text-editor, code-editor, notepad-plus-plus, notepad++, egui, eframe, gui, syntax-highlighting, desktop-app, cross-platform, find-and-replace, regex

---

## Features

### Editing
- Multi-tab interface with dirty indicators, middle-click / `×` close, tab context menu, **pinned tabs**, **Restore Closed Tab** (`Ctrl+Shift+T`) and **drag & drop** to open files. Tabs use rounded Windows 11 styling.
- **Command palette** (`Ctrl+Shift+P`): fuzzy-search every action of the editor — files, line operations, case, encodings, themes, converters — without digging through menus.
- **Syntax highlighting** for Rust, C/C++, C#, Java, Python, JavaScript/TypeScript, Go, SQL, HTML/XML, CSS, JSON, YAML, TOML, shell, PowerShell, Batch, Markdown and more.
- Line-number gutter aligned with wrapped lines, current-line highlight, word wrap toggle.
- **Auto-completion** popup (language keywords + words from the document; `Ctrl+Space`, `Tab`/`Enter` to accept).
- If Notepad++ is installed, its `autoCompletion\*.xml` dictionaries are read at runtime to enrich completions (nothing is bundled).
- Tools menu: *Trim Trailing Space and Save* (`Alt+Shift+S`), *Open in New Instance* (`Alt+F6`), look up the current word on the web (`Alt+F1` PHP manual, `Alt+F3` Wikipedia).
- **Auto-indent** on Enter (extra level after `{`, `(`, `[`, `:`).
- Full **undo / redo** history with typing grouped into natural steps.
- **Column (rectangular) selection** with `Alt`+drag: type, delete, copy, cut and paste across all selected lines.
- **8 color themes** (Obsidian Dark, Classic Light, Monokai, Solarized Dark/Light, Zenburn, **Fluent Dark / Fluent Light**), switchable from the toolbar, View menu or command palette.
- **Follow Windows Theme**: automatically switches between Fluent Light and Fluent Dark with the Windows 11 apps theme (View menu).
- Windows typography: **Segoe UI Variable** for the interface, **Cascadia Mono** for the editor (with safe fallbacks), and Fluent vector icons in the toolbar.
- Keyboard and mouse selection: `Shift`+arrows, `Ctrl+Shift`+arrows (word), `Shift`+click, double-click (word), triple-click (line), `Ctrl+A`.
- Zoom with `Ctrl`+mouse wheel, `Ctrl`+`+` / `-` / `0`.
- Confirmation prompt before closing modified documents, also on window close (pinned tabs are protected from *Close All* / *Close Others*).
- **English / French interface** (View > Interface language), using familiar Notepad++ French terminology.
- If Notepad++ is installed, its `functionList\*.xml` parsers (all 47 compile; PCRE subroutine calls, numbered recursion and look-ahead conditionals are emulated, recursion is unrolled three levels) drive the Function List; the built-in rules are the fallback.
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

## Install

### Windows installer (MSI)

Build and install in two steps:

```bash
cargo build --release
wix build -arch x64 -d ProductVersion=0.2.0 installer/rust-note.wxs -o target/wix/rust-note-0.2.0-x64.msi
```

Then run `target/wix/rust-note-0.2.0-x64.msi`. The installer (per-machine, requires elevation):

- installs to `Program Files\rust-note` with a Start Menu shortcut and an *Apps & Features* entry (with icon and project URL);
- optional feature **"Explorer context menu entry"** (checked by default) adds:
  - **Edit with rust-note** on any file,
  - **Open rust-note here** on folders and folder backgrounds,
  - rust-note in the **Open with** list of 35 common text/code extensions (.txt, .md, .log, .rs, .toml, .json, …).

Everything is removed cleanly on uninstall (the MSI tracks the registry keys it writes). On Windows 11 the entries appear in the classic menu — click **"Show more options"** or press `Shift+F10`, exactly like Notepad++'s shell integration. An entry in the first-level Windows 11 menu would require a signed MSIX sparse package with an `IExplorerCommand` shell extension (documented as a future avenue).

The `.exe` embeds the application icon and a `PerMonitorV2` DPI-aware, `longPathAware` manifest (via `build.rs` + `winres`).

### From source

Requires a recent stable Rust toolchain (edition 2024) — on Windows, the VS Build Tools for the icon/manifest embedding step.

```bash
git clone https://github.com/laurentvv/rust-note.git
cd rust-note
cargo run --release -- path/to/file.rs
```

The optimized binary is produced at `target/release/rust-note` (`rust-note.exe` on Windows).

## Keyboard shortcuts

| Action | Shortcut |
|---|---|
| Command palette | `Ctrl+Shift+P` |
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
  lib.rs        crate root (bin + lib split, reusable by examples)
  main.rs       eframe bootstrap (window, icon, CLI)
  app.rs        application state, actions, shortcuts, per-frame logic
  buffer/       documents, encodings, EOL, undo history
  editor/       line/column operations, auto-completion engine
  search/       regex search & replace engine
  syntax/       language detection, highlighter, symbol extraction
  plugins/      Base64 / URL / hex converters and exporters
  ui/           rendering modules: menu, toolbar, tabs, status, sidebar,
                editor, dialogs, windows, palette, themes, fonts, icon
  settings.rs   persisted preferences and session
examples/
  gen_icon.rs   one-shot generator for assets/icon.ico (256→16 px)
installer/
  rust-note.wxs WiX (v4/v5) MSI definition with Explorer context-menu feature
assets/         icon.ico, app.manifest (embedded by build.rs via winres)
```

Run the test suite with `cargo test`.

## Known limitations

- Starting a column selection (Alt+drag) turns word wrap off, so the block is always a straight rectangle (tabs are handled as 4 visual columns).
- Notepad++ function-list parsers are approximated: nesting deeper than three levels of recursion is not recognised.
- The undo history stores full snapshots (up to 100 steps); fine for notes and source files, not designed for very large files.

## License

Released under the MIT License — see [LICENSE](LICENSE). rust-note is an independent project and is not affiliated with Notepad++.
