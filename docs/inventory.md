# Notepad++ 8.9.8.1 — Clean-Room Inventory & Architecture Reference

Reconnaissance and mapping performed using `universal-asset-ripper` on `C:\Program Files\Notepad++`.
Clean-room implementation codebase: `d:\git\notepadpp-rs`.

## 1. Upstream Suite Profile

| Parameter | Observed Value |
|---|---|
| **Binary** | `notepad++.exe` (x64 Windows PE) |
| **Product Version** | `8.9.8.1` |
| **Legal Copyright** | `Copyleft 1998-2023 by Don HO` |
| **Original Compiler** | MSVC (Native C++) / Scintilla-based buffer / Win32 GUI |
| **Total PE Resources** | 2487 items (341 textures/icons/bitmaps, 72 dialog templates/configs, 3 version/manifests) |
| **Manifest** | Per-Monitor DPI aware v2, Common Controls v6, UTF-8 active code page |

---

## 2. Decoded Win32 Dialogs (RT_DIALOG Templates)

All 70 dialog templates from `notepad++.exe` were parsed and decoded to JSON templates under `.scratch/notepadpp-resources/dialogs/`. Key dialogs replicated in the Rust architecture:

| Dialog ID | Title / Purpose | Control Count | Replicated in Rust |
|---|---|---|---|
| `dialog_1600.json` | **Find & Replace** (Find, Replace, Find in Files, Mark) | 53 controls | Modal / Floating dock with regex, match case, whole word, search scope |
| `dialog_2000.json` | **Preferences** (General, Editing, Dark Mode, File Association) | 38 controls | Options settings panel |
| `dialog_2020.json` | **Go To Line** (Ctrl+G) | 6 controls | Line / Character offset prompt with bounds clamp |
| `dialog_6000.json` | **About Notepad++** | 12 controls | About modal with version, author, build info |
| `dialog_6100.json` | **Summary / Document Properties** | 14 controls | Characters, Words, Lines, Bytes, Path, Encoding, EOL |
| `dialog_22000.json`| **Run / Execute Command** | 18 controls | External command runner |
| `dialog_25000.json`| **Windows / Window List** | 16 controls | Open tabs manager |

---

## 3. Command & Menu Surface (`localization/english.xml`)

Each command has a canonical numeric ID matching Notepad++ internal message IDs:

### File Menu
- `41001` : New document (`Ctrl+N`)
- `41002` : Open document (`Ctrl+O`)
- `41003` : Close active document (`Ctrl+W`)
- `41004` : Close All (`Ctrl+Shift+W`)
- `41005` : Close All BUT Current
- `41006` : Save (`Ctrl+S`)
- `41007` : Save All (`Ctrl+Shift+S`)
- `41008` : Save As... (`Ctrl+Alt+S`)
- `41011` : Exit (`Alt+F4`)
- `41014` : Reload from Disk
- `41017` : Rename...
- `41021` : Restore Recent Closed File (`Ctrl+Shift+T`)

### Edit Menu
- `42001` : Cut (`Ctrl+X`)
- `42002` : Copy (`Ctrl+C`)
- `42003` : Undo (`Ctrl+Z`)
- `42004` : Redo (`Ctrl+Y`)
- `42005` : Paste (`Ctrl+V`)
- `42006` : Delete (`Del`)
- `42007` : Select All (`Ctrl+A`)
- `42008` : Increase Line Indent (`Tab`)
- `42009` : Decrease Line Indent (`Shift+Tab`)
- `42010` : Duplicate Current Line (`Ctrl+D`)
- `42012` : Split Lines (`Ctrl+I`)
- `42013` : Join Lines (`Ctrl+J`)
- `42014` : Move Up Current Line (`Ctrl+Shift+Up`)
- `42015` : Move Down Current Line (`Ctrl+Shift+Down`)
- `42059` : Sort Lines Lexicographically Ascending
- `42060` : Sort Lines Lexicographically Descending
- `42077` : Remove Consecutive Duplicate Lines
- `42079` : Remove Duplicate Lines
- `42084` : Date Time (Short: `YYYY-MM-DD HH:MM`)
- `42085` : Date Time (Long: `YYYY-MM-DD HH:MM:SS`)
- `42067` : Convert Case to UPPERCASE (`Ctrl+Shift+U`)
- `42068` : Convert Case to lowercase (`Ctrl+U`)
- `42073` : Convert Case to Proper Case
- `42022` : Line Comment / Uncomment (`Ctrl+Q`)
- `42023` : Block Comment (`Ctrl+Shift+K`)
- `42024` : Trim Trailing Spaces
- `42025` : Trim Leading Spaces
- `42028` : TAB to Space
- `42029` : Space to TAB
- `42030` : EOL to Windows (CR LF)
- `42031` : EOL to Unix (LF)
- `42032` : EOL to Macintosh (CR)

### Search Menu
- `43001` : Find... (`Ctrl+F`)
- `43003` : Replace... (`Ctrl+H`)
- `43004` : Find Next (`F3`)
- `43005` : Find Previous (`Shift+F3`)
- `43009` : Go To Line... (`Ctrl+G`)
- `43022` : Style All Occurrences of Token
- `43032` : Clear All Styles / Marks

### View Menu
- `44010` : Word Wrap Toggle (`Alt+Z`)
- `44022` : Show All Characters (White spaces, tabs, EOLs)
- `44034` : Zoom In (`Ctrl+Num+`)
- `44033` : Zoom Out (`Ctrl+Num-`)
- `44035` : Restore Default Zoom (`Ctrl+Num/`)
- `44080` : Document Map (Minimap) Toggle
- `44084` : Function List Toggle

### Encodings Menu
- `45001` : ANSI (Windows-1252)
- `45008` : UTF-8
- `45009` : UTF-8 with BOM
- `45010` : UTF-16 BE
- `45011` : UTF-16 LE
- Convert to UTF-8 / UTF-16 / ANSI

### Plugins Menu
- **MIME Tools** : Base64 Encode, Base64 Decode, URL Encode, URL Decode
- **Converter** : ASCII to HEX, HEX to ASCII, Dec/Hex/Bin inspector

---

## 4. CLI Arguments Specification

Compatible with Notepad++ command line options:

```bash
notepadpp-rs [OPTIONS] [FILES]...
```

| Option | Flag in Notepad++ | Behavior in `notepadpp-rs` |
|---|---|---|
| `-multiInst` | `-multiInst` | Permits multiple application processes |
| `-noPlugin` | `-noPlugin` | Disables plugin modules |
| `-nosession` | `-nosession` | Starts with empty buffer without loading previous session |
| `-notabbar` | `-notabbar` | Hides the document tab bar |
| `-ro` | `-ro` | Marks initial buffers as Read-Only |
| `-alwaysOnTop` | `-alwaysOnTop` | Keeps window pinned on top of other windows |
| `-n<line>` | `-n42` or `-n=42` | Jumps to line 42 on launch |
| `-c<col>` | `-c10` or `-c=10` | Jumps to column 10 on launch |
| `-p<pos>` | `-p100` | Jumps to character position 100 on launch |
| `-l<lang>` | `-lrust`, `-lpy` | Forces syntax highlighting language |
| `[FILES]...` | positional | Opens one or more files in tab bar |

---

## 5. Rust Architecture & Module Map

```text
notepadpp-rs/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point & eframe application loop
│   ├── cli.rs             # Notepad++ command line arguments parser
│   ├── app.rs             # Main GUI state (menus, toolbar, tabs, status bar, dialogs)
│   ├── buffer/
│   │   ├── mod.rs         # Text document representation, line indices, cursor
│   │   ├── history.rs     # Undo / Redo command stack
│   │   ├── encoding.rs    # UTF-8, UTF-16 LE/BE, ANSI Windows-1252, BOM detection
│   │   └── eol.rs         # CRLF, LF, CR conversion & detection
│   ├── editor/
│   │   ├── mod.rs         # Editor canvas with line numbers, cursor, selection
│   │   └── operations.rs  # Line duplication, sort, indent, case transform, trim
│   ├── syntax/
│   │   └── mod.rs         # Language detection, keywords, comments, syntax coloring
│   ├── search/
│   │   └── mod.rs         # Find & replace engine (plain text, regex, whole word, match case)
│   ├── plugins/
│   │   └── mod.rs         # Built-in MIME tools (Base64, URL) and ASCII/HEX conversion
│   └── ui/
│       ├── menu_bar.rs    # Menus matching Notepad++ IDs & hotkeys
│       ├── toolbar.rs     # Toolbar buttons with classic actions
│       ├── tab_bar.rs     # Document tabs with dirty flags & close buttons
│       ├── status_bar.rs  # Line, Col, Sel, Length, Lines, EOL, Encoding, INS/OVR
│       └── dialogs.rs     # Modals: Find/Replace, Go To Line, About, Summary
```

---

## 6. REA Deep Reverse Engineering Recovery (2026-10-25)

Using **REA** (`morluto/rea` v6.8.0) and **Universal Asset Ripper** PE recon over `C:\Program Files\Notepad++`:

### A. Syntax Definition Recovery (`langs.model.xml`)
- Audited all 95 programming languages declared in upstream `langs.model.xml`.
- Extracted exact file extensions, comment markers (`//`, `#`, `--`, `;`, `REM`, `<!-- -->`), and language identifiers.
- Expanded `syntax::Language` to support 34 core languages natively with extension auto-detection and comment toggling.

### B. Theme Palette Recovery (`stylers.model.xml` & `DarkModeDefault.xml`)
- Decoded the Scintilla styles and color tables:
  * **Notepad++ Classic Light Theme**: `#FFFFFF` canvas, `#000000` text, `#008000` comments, `#0000FF` keywords, `#F0F0F0` line numbers.
  * **Notepad++ Dark Mode Theme**: `#202020` canvas, `#DCDCCC` text, `#7F9F7F` comments, `#DFC47D` keywords, `#2A2A2A` line numbers.
- Added live theme switching under `View -> Theme`.

### C. Right-Click Context Menu (`contextMenu.xml`)
- Mapped the editor canvas popup menu directly from `contextMenu.xml`:
  * Undo / Redo
  * Line operations: Duplicate Line, Move Line Up/Down, Remove Duplicates, Sort Ascending
  * Case conversion: UPPERCASE, lowercase, Proper Case
  * Comment / Uncomment with active language prefix
  * MIME / Converter: Base64 and URL encode/decode

### D. Function List Symbol Outline (`functionList/*.xml`)
- Recovered outline extraction patterns for functions, methods, classes, and structs across languages.
- Implemented `syntax::symbols::extract_symbols` and interactive `Function List` sidebar (`Ctrl+Alt+L` / `View -> Function List`).
- Clicking symbols instantly scrolls to and positions the cursor at the target line.

### E. Tab History & Session Restoration
- Implemented `closed_docs` tab history stack with `Ctrl+Shift+T` (`File -> Restore Closed Tab`).
- Capped at 25 recently closed tabs with content and path preserved.


