# Plan d'amélioration rust-note — revue, modernisation graphique/fonctionnelle et packaging MSI Windows 11

## Constats de la revue de code (résumé)

**Points forts** : code idiomatique (édition 2024), zéro `unwrap()` en production, 47 tests, i18n EN/FR testée, fonctionnalités riches (36 langages, sélect colonne, encodages, émulation function-list Notepad++).

**Dettes identifiées** :
- `src/app.rs` = monolithe de 2 728 lignes dont `update()` fait ~1 710 lignes d'une traite (menus, toolbar, onglets, statut, sidebar, éditeur, 4 fenêtres flottantes imbriquées sur 8-10 niveaux)
- Champs morts : `show_symbols`, `document_map`, `scroll_offset`, `is_pinned` (jamais câblé), flags CLI `multi_inst`/`no_plugin` sans effet, `History::can_undo/can_redo/clear` utilisés seulement par les tests
- Dépendances déclarées inutilisées : `clap` (CLI réécrit à la main) et `anyhow`
- Doublons : `export_current_to_html/rtf`, constructeurs `Document`, motif menu répété ~60 fois
- Mélange offsets octets (recherche) / chars (curseur egui) avec conversions manuelles — egui 0.35+ apporte `CharIndex`/`ByteIndex` typés qui éliminent cette classe de bugs
- Aucun `[profile.release]` (pas de LTO/strip), aucun packaging : pas de `.ico` embarqué, pas de manifest Windows, pas d'installeur, pas de menu contextuel Explorer

**Recherche internet (sources clés)** : Rust stable = **1.99.0** ; eframe/egui latest = **0.37.0** (le projet est en 0.33.3) ; `rfd 0.17.2` déjà à jour ; `cargo-wix` reste bloqué sur WiX v3 → on utilisera **WiX v6 directement** (`dotnet tool`) ; l'entrée registre classique du menu contextuel apparaît dans « Afficher plus options » (comme Notepad++), le menu Windows 11 de premier niveau exigerait IExplorerCommand + package MSIX sparse signé (hors scope, noté en piste future). Inspiration produit : **Notepads** (Fluent, minimal, rapide) et **Sublime Text** (palette de commandes) — cohérent avec « moderne, rapide, efficace, sans multitude d'options ».

## Étapes d'implémentation

### Étape 1 — Nettoyage Cargo.toml et code mort
- Retirer `clap` et `anyhow` des dépendances
- Ajouter `[profile.release]` : `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `opt-level = 3`
- `cargo update` pour passer toutes les autres deps au latest compatible
- Supprimer `show_symbols`, `document_map`, `scroll_offset` ; câbler `multi_inst` (spawn d'une nouvelle instance via `std::process::Command` sur le propre exe avec l'argument fichier) ; supprimer `no_plugin` ; conserver `is_pinned` (réutilisé à l'étape 4)
- Factoriser `export_current_to_html/rtf` en une fonction commune paramétrée par le format ; unifier les constructeurs `Document`
- `cargo test` après chaque lot

### Étape 2 — Découpage du monolithe app.rs
- Créer des fichiers `src/ui/menu.rs`, `toolbar.rs`, `tabs.rs`, `status.rs`, `sidebar.rs`, `editor.rs`, `windows.rs` (fenêtres flottantes) contenant chacun des blocs `impl RustNoteApp { fn render_... }` (les impl partiels dans plusieurs fichiers sont légaux dans un même crate)
- `update()` réduit à ~100 lignes d'orchestration : état → raccourcis → drag&drop → autosave → appel des sous-rendus
- Aucun changement de comportement : `cargo test` + test manuel doivent être identiques

### Étape 3 — Migration Rust 1.99 + eframe/egui 0.33 → 0.37
- `rustup update stable` (1.99.0, ≥ MSRV 1.95 exigé par egui 0.36+)
- eframe 0.37 avec features `["default_fonts", "glow"]` (+ provider de polices système natif si nécessaire pour l'étape 4)
- Points de migration identifiés dans le changelog :
  - 0.34 : `TopBottomPanel`/`SidePanel` dépréciés → `Panel` unifié ; `App::update` → split `logic`/`ui` ; renommages `Context` ; refonte `Ui`-centrique
  - 0.35 : suppression de tout ce qui est déprécié ; indices typés `CharIndex`/`ByteIndex` (adapter `selection_byte_range`, `jump_to_hit`, le layouter) ; noyaux `Id`/`IdSalt` en préparation
  - 0.36 : `Modifiers` devient un `egui::Event` ; MSRV 1.95
  - 0.37 : `default_fonts` n'est plus une feature par défaut (déjà listée explicitement ✓) ; `Id::new` déprécié → `Id::unique`/`IdSaltSet` ; constructeurs `Panel` prennent `impl AsIdSalt`
- **Point de vigilance majeur** : le code pilote l'état interne `egui::text_edit::TextEditState::load/store` — vérifier cette API à chaque cran de version et adapter (curseur/sélection programmatiques, sélect colonne, auto-indent)
- Remplacer la popup d'auto-complétion maison par le nouveau `CompletionPopup` officiel d'egui 0.37 si l'API s'y prête, sinon conserver l'implémentation actuelle
- `cargo test` + test manuel complet (recherche regex, sélect colonne, undo groupé, encodages, multi-octet CJK/émoji, session)

### Étape 4 — Améliorations graphiques et fonctionnelles (philosophie Notepads : Fluent, minimal)
- **Polices système** : Segoe UI Variable pour l'UI, Cascadia Mono/Code pour l'éditeur, via le provider de polices système natif d'eframe 0.37 (avec fallback polices embarquées)
- **Icônes vectorielles** : remplacer les emoji de la toolbar par de vraies icônes (crate compatible egui 0.37 type `egui_iconify` avec jeu Fluent, sinon formes `epaint` dessinées dans `ui/icon.rs` étendu)
- **Onglets façon Windows 11** : coins arrondis, bouton fermer visible au survol, et câblage des **onglets épinglés** (champ `is_pinned` existant + épingle dans le menu contextuel d'onglet)
- **Palette de commandes (Ctrl+Shift+P)** : registre centralisé d'actions (`Command { id, libellé traduit, raccourci, action }`) couvrant fichier/édition/ops de lignes/outils/thèmes/langues, avec recherche floue — remplace la profusion de menus sans les supprimer
- **Thème clair/sombre automatique** : lecture de `AppsUseLightTheme` (crate `windows-registry`, officielle et légère) + deux nouveaux thèmes « Fluent Light » / « Fluent Dark » ; sélecteur de thème avec pastilles d'aperçu
- Barre de statut épurée, espacements/marges cohérents sur tous les panneaux

### Étape 5 — Packaging Windows : icône, manifest, MSI, menu contextuel
- **Icône .ico** : exemple one-shot `examples/gen_icon.rs` réutilisant `ui/icon.rs` + crate `image`/`ico` pour produire `assets/icon.ico` (256/48/32/16 px)
- **build.rs** avec crate `winres` : embarquer l'icône dans l'exe + manifest Windows (DPI PerMonitorV2, `longPathAware`, comctl32 v6)
- **MSI WiX v6** (`dotnet tool install -g wix`, prérequis .NET SDK à vérifier sur la machine) : `installer/rust-note.wxs`
  - Installation dans `Program Files\rust-note`, entrée « Applications installées » avec icône/auteur/URL
  - **Feature optionnelle (cochée par défaut) « Menu contextuel de l'Explorateur »** : clés registre `HKLM\Software\Classes\*\shell\EditWithRustNote\command = "...rust-note.exe" "%1"` (icône = exe), plus `Directory\Background\shell` pour « Ouvrir rust-note ici » sur le fond d'un dossier ; désinstallation propre automatique (le MSI retire ses clés)
  - Associations « Ouvrir avec » (non par défaut) pour .txt .md .log .rs .toml .json .xml .html .css .js .py
  - Build : `cargo build --release && wix build installer/rust-note.wxs -o target/wix/rust-note-<version>-x64.msi`
- Noter dans le README : l'entrée apparaît dans le menu classique (accessible aussi via « Afficher plus options »), exactement comme Notepad++ ; l'entrée dans le menu Win11 de premier niveau exigerait MSIX sparse signé (piste future documentée)

### Étape 6 — Validation et livraison
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` (les 47 tests existants + nouveaux pour palette de commandes/thème/registry) doivent passer
- Test manuel : gros fichier (5 Mo+), texte multi-octet, undo long, sélect colonne, encodages, restauration de session
- Installation réelle du MSI : vérifier Programs & Features, clic droit sur un .txt et sur le fond d'un dossier, puis désinstallation propre
- Mise à jour du README (captures, installation, menu contextuel) et bump version **0.2.0**

## Hors scope (explicitement)
- Réécriture du modèle de données (rope/gap buffer) pour l'undo snapshot — le coût/bénéfice ne se justifie pas pour un éditeur de notes
- Extension shell IExplorerCommand + MSIX sparse pour le menu Win11 moderne (nécessite un certificat de signature de code)
- Markdown preview, plugins dynamiques