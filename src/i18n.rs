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
    let (head, tab) = match s.split_once('\t') {
        Some((h, t)) => (h, Some(t)),
        None => (s, None),
    };
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
    let translated = fr(core).unwrap_or(core);
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
        "OK" => "OK",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_translation_keeps_icons_shortcuts_and_dots() {
        set_lang(UiLang::French);
        assert_eq!(tr("Open...\tCtrl+O"), "Ouvrir...\tCtrl+O");
        assert_eq!(tr("📂 Open"), "📂 Ouvrir");
        assert_eq!(tr("Save (Ctrl+S)"), "Enregistrer (Ctrl+S)");
        assert_eq!(tr("Unknown thing"), "Unknown thing");
        set_lang(UiLang::English);
        assert_eq!(tr("File"), "File");
    }
}
