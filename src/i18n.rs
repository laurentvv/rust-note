//! Minimal UI localization (English / French). French vocabulary follows the classic
//! Notepad++ French interface so the menus feel familiar.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiLang {
    #[default]
    English,
    French,
}

impl UiLang {
    pub const ALL: [Self; 2] = [Self::English, Self::French];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::French => "Français",
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(lang: UiLang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

#[must_use]
pub fn lang() -> UiLang {
    if CURRENT.load(Ordering::Relaxed) == 1 { UiLang::French } else { UiLang::English }
}

/// Translates a UI string. Handles a leading icon, a trailing `...`, a `\tShortcut` suffix
/// and a trailing ` (Ctrl+X)` hint; unknown strings are returned unchanged.
#[must_use]
pub fn tr(s: &str) -> String {
    if lang() == UiLang::English {
        return s.to_string();
    }
    if s.starts_with("Jumped to '") || s.starts_with("Found at line ") {
        return message(s);
    }
    let (head, tab) = match s.split_once('\t') {
        Some((h, t)) => (h, Some(t)),
        None => (s, None),
    };
    // Entries whose English text itself contains parentheses are looked up verbatim first
    if let Some(t) = fr(head) {
        return match tab {
            Some(tab) => format!("{t}	{tab}"),
            None => t.to_string(),
        };
    }
    let (head, hint) = match head.rfind(" (") {
        Some(i) if head.ends_with(')') => (&head[..i], Some(&head[i..])),
        _ => (head, None),
    };
    let start = head.find(|c: char| c.is_alphanumeric()).unwrap_or(head.len());
    let (icon, rest) = head.split_at(start);
    let (core, dots) = match rest.strip_suffix("...") {
        Some(c) => (c, "..."),
        None => (rest, ""),
    };
    let translated = fr(core).map_or_else(|| message(core), str::to_string);
    let mut out = format!("{icon}{translated}{dots}");
    if let Some(h) = hint {
        out.push_str(h);
    }
    if let Some(t) = tab {
        out.push('\t');
        out.push_str(t);
    }
    out
}

/// Translates status messages that embed a value (`"Saved as C:\\x"`, `"Replaced 3 occurrences"`...).
fn message(s: &str) -> String {
    const PREFIXES: [(&str, &str); 11] = [
        ("Error opening ", "Erreur d'ouverture de "),
        ("Export error: ", "Erreur d'export : "),
        ("Exported HTML to ", "HTML exporté vers "),
        ("Exported RTF to ", "RTF exporté vers "),
        ("Jumped to line ", "Saut à la ligne "),
        ("Opened ", "Ouvert : "),
        ("Restored tab ", "Onglet restauré : "),
        ("Save error: ", "Erreur d'enregistrement : "),
        ("Saved as ", "Enregistré sous "),
        ("Theme: ", "Thème : "),
        ("Cannot start a new instance: ", "Impossible de lancer une nouvelle instance : "),
    ];
    for (en, fr) in PREFIXES {
        if let Some(rest) = s.strip_prefix(en) {
            return format!("{fr}{rest}");
        }
    }
    // Messages of the Find / Replace dialog
    if let Some(r) = s.strip_prefix("Found at line ") {
        return match r.strip_suffix(" (wrapped around the document)") {
            Some(n) => format!("Trouvé à la ligne {n} (retour au début du document)"),
            None => format!("Trouvé à la ligne {r}"),
        };
    }
    if let Some(r) = s.strip_prefix("Can't find the text ") {
        return format!("Texte introuvable : {r}");
    }
    if let Some(n) = s.strip_prefix("Count: ").and_then(|r| r.strip_suffix(" match(es) in the current document")) {
        return format!("Total : {n} occurrence(s) dans le document actuel");
    }
    if let Some(n) = s.strip_prefix("Replace All: ").and_then(|r| r.strip_suffix(" occurrence(s) were replaced.")) {
        return format!("Remplacer tout : {n} occurrence(s) remplacée(s).");
    }
    if let Some(r) = s.strip_prefix("Find All: ") {
        if let Some((hits, docs)) = r.split_once(" hit(s) in ") {
            return format!("Tout rechercher : {hits} résultat(s) dans {}", docs.replace(" document(s)", " document(s)"));
        }
    }
    if let Some(r) = s.strip_prefix("Invalid regular expression: ") {
        return format!("Expression régulière invalide : {r}");
    }
    if let Some(n) = s.strip_prefix("Replaced ").and_then(|r| r.strip_suffix(" occurrences")) {
        return format!("{n} occurrences remplacées");
    }
    if let Some(n) = s.strip_prefix("Saved ").and_then(|r| r.strip_suffix(" documents")) {
        return format!("{n} documents enregistrés");
    }
    if let Some(name) = s.strip_prefix("Saved ") {
        return format!("Enregistré : {name}");
    }
    if s.starts_with("Jumped to '") {
        return s.replacen("Jumped to '", "Saut à '", 1).replace(" (line ", " (ligne ");
    }
    s.to_string()
}

fn fr(s: &str) -> Option<&'static str> {
    Some(match s {
        // Main menus
        "File" => "Fichier",
        "Edit" => "Édition",
        "Search" => "Recherche",
        "View" => "Affichage",
        "Encoding" => "Encodage",
        "Language" => "Langage",
        "Plugins" => "Modules",
        "Tools" => "Outils",
        // File
        "New" => "Nouveau",
        "Open" => "Ouvrir",
        "Save" => "Enregistrer",
        "Save As" => "Enregistrer sous",
        "Save All" => "Enregistrer tout",
        "Close" => "Fermer",
        "Close All" => "Fermer tout",
        "Close All BUT Current" => "Fermer tout sauf le document actuel",
        "Close Tab" => "Fermer l'onglet",
        "Close Other Tabs" => "Fermer les autres onglets",
        "Close tab" => "Fermer l'onglet",
        "Restore Closed Tab" => "Restaurer l'onglet fermé",
        "Recent Files" => "Fichiers récents",
        "Clear Recent Files" => "Effacer les fichiers récents",
        "Exit" => "Quitter",
        "Export to HTML" => "Exporter en HTML",
        "Export to RTF (Rich Text)" => "Exporter en RTF (texte enrichi)",
        "Copy to Clipboard" => "Copier dans le presse-papiers",
        "Copy Full File Path" => "Copier le chemin complet",
        "Copy Filename" => "Copier le nom du fichier",
        "Copy Directory Path" => "Copier le chemin du dossier",
        "Copy Path" => "Copier le chemin",
        "New Document" => "Nouveau document",
        "New tab" => "Nouvel onglet",
        "Open File" => "Ouvrir un fichier",
        // Edit
        "Undo" => "Annuler",
        "Redo" => "Rétablir",
        "Line Operations" => "Ligne",
        "Duplicate Current Line" => "Dupliquer la ligne courante",
        "Remove Duplicate Lines" => "Supprimer les doublons",
        "Remove Consecutive Duplicate Lines" => "Supprimer les doublons consécutifs",
        "Move Line Up" => "Déplacer la ligne vers le haut",
        "Move Line Down" => "Déplacer la ligne vers le bas",
        "Sort Lines Ascending" => "Trier les lignes (croissant)",
        "Sort Lines Descending" => "Trier les lignes (décroissant)",
        "Join Lines" => "Joindre les lignes",
        "Convert Case to" | "Convert Case" => "MAJUSCULE/minuscule",
        "UPPERCASE" => "MAJUSCULE",
        "lowercase" => "minuscule",
        "Proper Case" => "Première lettre en majuscule",
        "Invert Case" => "Inverser la casse",
        "Random Case" => "Casse aléatoire",
        "Blank Operations" => "Traitement des espacements",
        "Trim Trailing Space" => "Supprimer les espaces en fin de ligne",
        "Trim Leading Space" => "Supprimer les espaces en début de ligne",
        "TAB to Space" => "Tabulation vers espaces",
        "Space to TAB" => "Espaces vers tabulation",
        "Insert Date Time" => "Insérer la date et l'heure",
        "Date Time (short)" => "Date et heure (court)",
        "Date Time (long)" => "Date et heure (long)",
        "EOL Conversion" => "Convertir les sauts de ligne",
        "Windows (CR LF)" => "Windows (CR LF)",
        "Unix (LF)" => "Unix (LF)",
        "Macintosh (CR)" => "Macintosh (CR)",
        "Toggle Comment" => "Commenter / décommenter",
        // Search
        "Find" => "Rechercher",
        "Find..." => "Rechercher...",
        "Replace" => "Remplacer",
        "Find Next" => "Rechercher le suivant",
        "Find Previous" => "Rechercher le précédent",
        "Go to Line" => "Aller à la ligne",
        "Find All" => "Tout rechercher",
        "Find in Open Documents" => "Rechercher dans les documents ouverts",
        "Replace All" => "Remplacer tout",
        "Count" => "Compter",
        "Match case" => "Respecter la casse",
        "Match whole word only" => "Mot entier uniquement",
        "Regular expression" => "Expression régulière",
        "Down" => "Bas",
        "Up" => "Haut",
        "Cancel" => "Annuler",
        // View
        "Word Wrap" => "Retour automatique à la ligne",
        "Line Numbers" => "Numéros de ligne",
        "Function List" => "Liste des fonctions",
        "Function List Outline" => "Plan des fonctions",
        "Document Map (Minimap)" => "Carte du document (minimap)",
        "Zoom In" => "Zoom avant",
        "Zoom Out" => "Zoom arrière",
        "Zoom in" => "Zoom avant",
        "Zoom out" => "Zoom arrière",
        "Restore Default Zoom" => "Restaurer le zoom par défaut",
        "Theme" => "Thème",
        "Themes" => "Thèmes",
        "Interface language" => "Langue de l'interface",
        // Plugins / tools
        "MIME Tools" | "MIME / Tools" => "Outils MIME",
        "Base64 Encode" => "Encoder en Base64",
        "Base64 Decode" => "Décoder le Base64",
        "URL Encode" => "Encoder l'URL",
        "URL Decode" => "Décoder l'URL",
        "Quoted-Printable Encode" => "Encoder en Quoted-Printable",
        "Converter" => "Convertisseur",
        "0x Quick Conversion Panel" => "Panneau de conversion rapide 0x",
        "ASCII to HEX" => "ASCII vers HEX",
        "HEX to ASCII" => "HEX vers ASCII",
        "DEC to HEX" => "DEC vers HEX",
        "DEC to HEX (0x...)" => "DEC vers HEX (0x...)",
        "HEX to DEC" => "HEX vers DEC",
        "Copy DEC" => "Copier DEC",
        "Copy HEX" => "Copier HEX",
        "Document Summary" => "Résumé du document",
        "Document Properties" => "Propriétés du document",
        "Real-Time Multi-Base Quick Converter" => "Convertisseur multi-bases en temps réel",
        // Help
        "Keyboard Shortcuts" => "Raccourcis clavier",
        "About rust-note" => "À propos de rust-note",
        "Documentation & Sources" => "Documentation et sources",
        // Toolbar
        "New " | "Nouveau " => "Nouveau",
        "Open " => "Ouvrir",
        "All" => "Tout",
        "Symbols" => "Symboles",
        "Summary" => "Résumé",
        "Wrap" => "Césure",
        "0x Converter" => "Convertisseur 0x",
        "Find what:" => "Rechercher :",
        "Replace with:" => "Remplacer par :",
        "Find / Replace" => "Rechercher / Remplacer",
        "Line number" => "Numéro de ligne",
        "A modern, elegant, and high-performance code editor built with Rust and egui." => "Un éditeur de code moderne, élégant et performant, écrit en Rust avec egui.",
        "Author:" => "Auteur :",
        "Engine Architecture: High-performance multi-tab text buffer" => "Architecture : tampon de texte multi-onglets haute performance",
        "Source & Docs" => "Sources et documentation",
        "Current Document Properties" => "Propriétés du document actuel",
        "File Path:" => "Chemin :",
        "Encoding:" => "Encodage :",
        "EOL:" => "Fin de ligne :",
        "Total Characters:" => "Caractères :",
        "Total Words:" => "Mots :",
        "Total Lines:" => "Lignes :",
        "Total Bytes:" => "Octets :",
        "Ln" => "Lig",
        "Col" => "Col",
        "Column" => "Colonne",
        "Sel" => "Sél",
        "chars" => "car.",
        "lines" => "lignes",
        "B" => "o",
        "READ ONLY" => "LECTURE SEULE",
        "Jump to line" => "Aller à la ligne",
        "Help on Internet" => "Aide sur Internet",
        "Get PHP help" => "Aide PHP",
        "Wikipedia Search" => "Recherche Wikipédia",
        "Search the web" => "Rechercher sur le web",
        "Open in New Instance" => "Ouvrir dans une nouvelle instance",
        "Trim Trailing Space and Save" => "Supprimer les espaces de fin et enregistrer",
        "Saved" => "Enregistré",
        "Document closed" => "Document fermé",
        "Place the caret on a word first" => "Placez le curseur sur un mot",
        "Opened in a new instance" => "Ouvert dans une nouvelle instance",
        "All documents closed" => "Tous les documents sont fermés",
        "Closed other documents" => "Les autres documents sont fermés",
        "Copied DEC to clipboard" => "DEC copié dans le presse-papiers",
        "Copied HEX to clipboard" => "HEX copié dans le presse-papiers",
        "Copied directory path to clipboard" => "Chemin du dossier copié",
        "Copied filename to clipboard" => "Nom du fichier copié",
        "Copied full path to clipboard" => "Chemin complet copié",
        "Document has not been saved to disk yet" => "Le document n'est pas encore enregistré",
        "New document created" => "Nouveau document créé",
        "No recently closed tabs to restore" => "Aucun onglet récemment fermé à restaurer",
        "Enter text to search for" => "Saisissez le texte à rechercher",
        "Document is read-only" => "Le document est en lecture seule",
        "Replaced 1 occurrence; next match selected" => "1 occurrence remplacée ; suivante sélectionnée",
        "Replaced 1 occurrence; no more matches" => "1 occurrence remplacée ; plus aucune correspondance",
        "line" => "ligne",
        "New / Open / Save" => "Nouveau / Ouvrir / Enregistrer",
        "Save All / Save As" => "Tout enregistrer / Enregistrer sous",
        "Close tab / Restore tab" => "Fermer / restaurer l'onglet",
        "Next / previous tab" => "Onglet suivant / précédent",
        "Find next / previous" => "Suivant / précédent",
        "Duplicate line" => "Dupliquer la ligne",
        "Delete line" => "Supprimer la ligne",
        "Move line up / down" => "Déplacer la ligne haut / bas",
        "Join lines" => "Joindre les lignes",
        "Toggle comment" => "Commenter / décommenter",
        "UPPERCASE / lowercase" => "MAJUSCULE / minuscule",
        "Undo / Redo" => "Annuler / Rétablir",
        "Select word / line" => "Sélectionner mot / ligne",
        "Extend selection" => "Étendre la sélection",
        "Select by word" => "Sélection par mot",
        "Column selection" => "Sélection en colonne",
        "Auto-completion" => "Auto-complétion",
        "Accept / navigate completion" => "Valider / parcourir la complétion",
        "Zoom" => "Zoom",
        "Function list" => "Liste des fonctions",
        "Go to line" => "Aller à la ligne",
        "0x Quick Converter" => "Convertisseur rapide 0x",
        "Bidirectional Real-Time Multi-Base Converter" => "Convertisseur multi-bases bidirectionnel en temps réel",
        "rust-note Context Menu" => "Menu contextuel rust-note",
        "INS" => "INS",
        "OK" => "OK",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The UI language is a process-wide setting: serialize the tests that change it
    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn french_translation_keeps_icons_shortcuts_and_dots() {
        let _guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        set_lang(UiLang::French);
        assert_eq!(tr("Open...\tCtrl+O"), "Ouvrir...\tCtrl+O");
        assert_eq!(tr("📂 Open"), "📂 Ouvrir");
        assert_eq!(tr("Save (Ctrl+S)"), "Enregistrer (Ctrl+S)");
        assert_eq!(tr("Unknown thing"), "Unknown thing");
        assert_eq!(tr("Replaced 3 occurrences"), "3 occurrences remplacées");
        assert_eq!(tr("Saved as C:/x.txt"), "Enregistré sous C:/x.txt");
        assert_eq!(tr("Saved main.rs"), "Enregistré : main.rs");
        assert_eq!(tr("Replace All: 3 occurrence(s) were replaced."), "Remplacer tout : 3 occurrence(s) remplacée(s).");
        assert_eq!(tr("Found at line 7"), "Trouvé à la ligne 7");
        assert_eq!(tr("Found at line 2 (wrapped around the document)"), "Trouvé à la ligne 2 (retour au début du document)");
        assert_eq!(tr("Count: 2 match(es) in the current document"), "Total : 2 occurrence(s) dans le document actuel");
        assert_eq!(tr("Jumped to 'run' (line 4)"), "Saut à 'run' (ligne 4)");
        set_lang(UiLang::English);
        assert_eq!(tr("File"), "File");
    }

    /// Every literal passed to `tr("...")` in the UI code must have a French translation
    /// (or be language-neutral), so no static label is silently left in English.
    #[test]
    fn all_ui_literals_are_translated() {
        let _guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let sources = [include_str!("app.rs"), include_str!("ui/dialogs.rs")];
        // Names, formats and symbols that are identical in French
        let neutral = [
            "?", "+", "×", "➕", "➖", "OK", "UTF-8", "UTF-8-BOM", "UTF-16 LE", "UTF-16 BE", "ANSI (Windows-1252)",
            "Windows (CR LF)", "Unix (LF)", "Macintosh (CR)", "NppExport", "NppShell", "Col", "Sel", "    ",
            "↩ Wrap", "PowerShell", "MD5", "✏ INS",
        ];
        set_lang(UiLang::French);
        let mut missing = Vec::new();
        for src in sources {
            let mut rest = src;
            while let Some(i) = rest.find("tr(\"") {
                rest = &rest[i + 4..];
                let Some(end) = rest.find('"') else { break };
                let lit = rest[..end].replace("\\t", "\t");
                rest = &rest[end..];
                if neutral.contains(&lit.as_str()) || lit.is_empty() {
                    continue;
                }
                if tr(&lit) == lit {
                    missing.push(lit);
                }
            }
        }
        set_lang(UiLang::English);
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "untranslated UI strings: {missing:#?}");
    }

    /// Every status message the application can show must have a French form.
    #[test]
    fn all_status_messages_are_translated() {
        let _guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let messages = [
            "All documents closed", "Closed other documents", "Copied DEC to clipboard",
            "Copied HEX to clipboard", "Copied directory path to clipboard", "Copied filename to clipboard",
            "Copied full path to clipboard", "Document closed", "Document has not been saved to disk yet",
            "New document created", "No recently closed tabs to restore", "Place the caret on a word first",
            "Error opening x.txt: denied", "Export error: denied", "Exported HTML to a.html", "Exported RTF to a.rtf",
            "Jumped to 'run' (line 3)", "Jumped to line 3", "Opened a.txt", "Replaced 4 occurrences",
            "Restored tab 'a.txt'", "Save error: denied", "Saved as a.txt", "Saved 2 documents", "Saved a.txt",
            "Theme: Monokai", "Document is read-only", "Enter text to search for", "Can't find the text \"x\"",
            "Found at line 2", "Found at line 2 (wrapped around the document)",
            "Count: 3 match(es) in the current document", "Replace All: 3 occurrence(s) were replaced.",
            "Find All: 4 hit(s) in 2 document(s)", "Replaced 1 occurrence; next match selected",
            "Replaced 1 occurrence; no more matches", "Opened in a new instance",
        ];
        set_lang(UiLang::French);
        let untranslated: Vec<&str> = messages.iter().copied().filter(|m| tr(m) == *m).collect();
        set_lang(UiLang::English);
        assert!(untranslated.is_empty(), "untranslated messages: {untranslated:#?}");
    }
}
