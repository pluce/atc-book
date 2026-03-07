use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLanguage {
    Fr,
    Uk,
}

static APP_LANGUAGE: OnceLock<Mutex<Option<AppLanguage>>> = OnceLock::new();

fn detect_language() -> AppLanguage {
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_lowercase();
    if locale.starts_with("fr") {
        AppLanguage::Fr
    } else {
        AppLanguage::Uk
    }
}

pub fn get_language() -> AppLanguage {
    let mut guard = APP_LANGUAGE.get_or_init(|| Mutex::new(None)).lock().unwrap();
    if let Some(lang) = *guard {
        lang
    } else {
        let lang = detect_language();
        *guard = Some(lang);
        lang
    }
}

pub fn set_language(lang: AppLanguage) {
    *APP_LANGUAGE.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(lang);
}

pub fn tr(lang: AppLanguage, key: &'static str) -> &'static str {
    match (lang, key) {
        (AppLanguage::Fr, "title.app") => "GESTIONNAIRE DE DOCUMENTS",
        (AppLanguage::Uk, "title.app") => "DOCUMENT MANAGER",
        (AppLanguage::Fr, "theme.night") => "Nuit",
        (AppLanguage::Uk, "theme.night") => "Night",
        (AppLanguage::Fr, "theme.day") => "Jour",
        (AppLanguage::Uk, "theme.day") => "Day",
        (AppLanguage::Fr, "lang.fr") => "FR",
        (AppLanguage::Uk, "lang.fr") => "FR",
        (AppLanguage::Fr, "lang.uk") => "UK",
        (AppLanguage::Uk, "lang.uk") => "UK",
        (AppLanguage::Fr, "sidebar.airports") => "Aeroports",
        (AppLanguage::Uk, "sidebar.airports") => "Airports",
        (AppLanguage::Fr, "sidebar.workspaces") => "Dossiers",
        (AppLanguage::Uk, "sidebar.workspaces") => "Workspaces",
        (AppLanguage::Fr, "sidebar.settings") => "Reglages",
        (AppLanguage::Uk, "sidebar.settings") => "Settings",
        (AppLanguage::Fr, "nav.airports") => "AEROPORTS",
        (AppLanguage::Uk, "nav.airports") => "AIRPORTS",
        (AppLanguage::Fr, "nav.workspaces") => "DOSSIERS",
        (AppLanguage::Uk, "nav.workspaces") => "WORKSPACES",
        (AppLanguage::Fr, "nav.settings") => "REGLAGES",
        (AppLanguage::Uk, "nav.settings") => "SETTINGS",
        (AppLanguage::Fr, "search.icao") => "Code OACI...",
        (AppLanguage::Uk, "search.icao") => "ICAO code...",
        (AppLanguage::Fr, "search.go") => "GO",
        (AppLanguage::Uk, "search.go") => "GO",
        (AppLanguage::Fr, "search.loading") => "...",
        (AppLanguage::Uk, "search.loading") => "...",
        (AppLanguage::Fr, "search.charts") => "cartes",
        (AppLanguage::Uk, "search.charts") => "charts",
        (AppLanguage::Fr, "workspace.new") => "+ Nouveau dossier",
        (AppLanguage::Uk, "workspace.new") => "+ New workspace",
        (AppLanguage::Fr, "workspace.name.placeholder") => "Nom du dossier...",
        (AppLanguage::Uk, "workspace.name.placeholder") => "Workspace name...",
        (AppLanguage::Fr, "common.ok") => "OK",
        (AppLanguage::Uk, "common.ok") => "OK",
        (AppLanguage::Fr, "workspace.none") => "Aucun dossier sauvegarde",
        (AppLanguage::Uk, "workspace.none") => "No saved workspace",
        (AppLanguage::Fr, "workspace.load") => "Charger ce dossier",
        (AppLanguage::Uk, "workspace.load") => "Load workspace",
        (AppLanguage::Fr, "workspace.unload") => "Decharger",
        (AppLanguage::Uk, "workspace.unload") => "Unload",
        (AppLanguage::Fr, "workspace.open_all") => "Ouvrir tout",
        (AppLanguage::Uk, "workspace.open_all") => "Open all",
        (AppLanguage::Fr, "workspace.rename") => "Renommer",
        (AppLanguage::Uk, "workspace.rename") => "Rename",
        (AppLanguage::Fr, "workspace.remove") => "Retirer du dossier",
        (AppLanguage::Uk, "workspace.remove") => "Remove from workspace",
        (AppLanguage::Fr, "workspace.delete.confirm") => "Supprimer ce dossier ?",
        (AppLanguage::Uk, "workspace.delete.confirm") => "Delete this workspace?",
        (AppLanguage::Fr, "common.yes") => "Oui",
        (AppLanguage::Uk, "common.yes") => "Yes",
        (AppLanguage::Fr, "common.no") => "Non",
        (AppLanguage::Uk, "common.no") => "No",
        (AppLanguage::Fr, "workspace.empty") => "Dossier vide",
        (AppLanguage::Uk, "workspace.empty") => "Empty workspace",
        (AppLanguage::Fr, "chart.quick_add") => "Ajouter au dossier courant",
        (AppLanguage::Uk, "chart.quick_add") => "Add to current workspace",
        (AppLanguage::Fr, "chart.in_workspace") => "Deja dans le dossier",
        (AppLanguage::Uk, "chart.in_workspace") => "Already in workspace",
        (AppLanguage::Fr, "quickswitch.placeholder") => "Rechercher une carte (Ctrl+P)",
        (AppLanguage::Uk, "quickswitch.placeholder") => "Search chart (Ctrl+P)",
        (AppLanguage::Fr, "quickswitch.empty") => "Aucun resultat",
        (AppLanguage::Uk, "quickswitch.empty") => "No result",
        (AppLanguage::Fr, "notes.title") => "Notes",
        (AppLanguage::Uk, "notes.title") => "Notes",
        (AppLanguage::Fr, "notes.placeholder") => "Commencez a ecrire vos notes de briefing...",
        (AppLanguage::Uk, "notes.placeholder") => "Start writing your briefing notes...",
        (AppLanguage::Fr, "notes.unpin") => "Detacher le panneau",
        (AppLanguage::Uk, "notes.unpin") => "Unpin side panel",
        (AppLanguage::Fr, "notes.pin") => "Epingler en panneau lateral",
        (AppLanguage::Uk, "notes.pin") => "Pin as side panel",
        (AppLanguage::Fr, "notes.h1") => "Titre 1",
        (AppLanguage::Uk, "notes.h1") => "Heading 1",
        (AppLanguage::Fr, "notes.h2") => "Titre 2",
        (AppLanguage::Uk, "notes.h2") => "Heading 2",
        (AppLanguage::Fr, "notes.h3") => "Titre 3",
        (AppLanguage::Uk, "notes.h3") => "Heading 3",
        (AppLanguage::Fr, "notes.bold") => "Gras",
        (AppLanguage::Uk, "notes.bold") => "Bold",
        (AppLanguage::Fr, "notes.italic") => "Italique",
        (AppLanguage::Uk, "notes.italic") => "Italic",
        (AppLanguage::Fr, "notes.underline") => "Souligne",
        (AppLanguage::Uk, "notes.underline") => "Underline",
        (AppLanguage::Fr, "notes.strike") => "Barre",
        (AppLanguage::Uk, "notes.strike") => "Strike",
        (AppLanguage::Fr, "notes.text_color") => "Couleur du texte",
        (AppLanguage::Uk, "notes.text_color") => "Text color",
        (AppLanguage::Fr, "notes.highlight") => "Surligner",
        (AppLanguage::Uk, "notes.highlight") => "Highlight",
        (AppLanguage::Fr, "notes.bullets") => "Liste a puces",
        (AppLanguage::Uk, "notes.bullets") => "Bulleted list",
        (AppLanguage::Fr, "notes.numbered") => "Liste numerotee",
        (AppLanguage::Uk, "notes.numbered") => "Numbered list",
        (AppLanguage::Fr, "notes.quote") => "Citation",
        (AppLanguage::Uk, "notes.quote") => "Quote",
        (AppLanguage::Fr, "notes.code") => "Code",
        (AppLanguage::Uk, "notes.code") => "Code",
        (AppLanguage::Fr, "notes.rule") => "Ligne horizontale",
        (AppLanguage::Uk, "notes.rule") => "Horizontal rule",
        (AppLanguage::Fr, "notes.paragraph") => "Paragraphe normal",
        (AppLanguage::Uk, "notes.paragraph") => "Normal paragraph",
        (AppLanguage::Fr, "notes.none") => "Aucun",
        (AppLanguage::Uk, "notes.none") => "None",
        (AppLanguage::Fr, "doc.loading") => "Chargement du PDF...",
        (AppLanguage::Uk, "doc.loading") => "Loading PDF...",
        (AppLanguage::Fr, "doc.error") => "Erreur de chargement",
        (AppLanguage::Uk, "doc.error") => "Load error",
        (AppLanguage::Fr, "empty.start") => "Recherchez un aerodrome pour commencer",
        (AppLanguage::Uk, "empty.start") => "Search an airport to start",
        (AppLanguage::Fr, "empty.hint") => "Entrez un code OACI (ex: LFPG, EGLL)",
        (AppLanguage::Uk, "empty.hint") => "Type an ICAO code (e.g. LFPG, EGLL)",
        (AppLanguage::Fr, "menu.send_to_workspace") => "Envoyer vers le dossier",
        (AppLanguage::Uk, "menu.send_to_workspace") => "Send to workspace",
        (AppLanguage::Fr, "menu.display") => "Affichage",
        (AppLanguage::Uk, "menu.display") => "Display",
        (AppLanguage::Fr, "menu.current_workspace") => "Dossier courant",
        (AppLanguage::Uk, "menu.current_workspace") => "Current workspace",
        (AppLanguage::Fr, "menu.none") => "Aucun",
        (AppLanguage::Uk, "menu.none") => "None",
        (AppLanguage::Fr, "menu.filter") => "Filtrer...",
        (AppLanguage::Uk, "menu.filter") => "Filter...",
        (AppLanguage::Fr, "menu.no_workspace") => "Aucun dossier",
        (AppLanguage::Uk, "menu.no_workspace") => "No workspace",
        (AppLanguage::Fr, "menu.no_result") => "Aucun resultat",
        (AppLanguage::Uk, "menu.no_result") => "No result",
        (AppLanguage::Fr, "menu.popout_new") => "Pop vers une nouvelle fenetre",
        (AppLanguage::Uk, "menu.popout_new") => "Pop to a new window",
        (AppLanguage::Fr, "menu.popout_existing") => "Pop vers la fenetre #2",
        (AppLanguage::Uk, "menu.popout_existing") => "Pop to window #2",
        (AppLanguage::Fr, "status.active") => "En vigueur",
        (AppLanguage::Uk, "status.active") => "Active",
        (AppLanguage::Fr, "status.expired") => "Expire",
        (AppLanguage::Uk, "status.expired") => "Expired",
        (AppLanguage::Fr, "status.network") => "RESEAU: -",
        (AppLanguage::Uk, "status.network") => "NETWORK: -",
        (AppLanguage::Fr, "settings.preferences") => "Preferences",
        (AppLanguage::Uk, "settings.preferences") => "Preferences",
        (AppLanguage::Fr, "settings.language") => "Langue",
        (AppLanguage::Uk, "settings.language") => "Language",
        (AppLanguage::Fr, "settings.lang.fr") => "Francais",
        (AppLanguage::Uk, "settings.lang.fr") => "French",
        (AppLanguage::Fr, "settings.lang.uk") => "Anglais",
        (AppLanguage::Uk, "settings.lang.uk") => "English",
        (AppLanguage::Fr, "settings.theme_mode") => "Mode de theme",
        (AppLanguage::Uk, "settings.theme_mode") => "Theme mode",
        (AppLanguage::Fr, "settings.theme.light") => "Toujours clair",
        (AppLanguage::Uk, "settings.theme.light") => "Always light",
        (AppLanguage::Fr, "settings.theme.dark") => "Toujours sombre",
        (AppLanguage::Uk, "settings.theme.dark") => "Always dark",
        (AppLanguage::Fr, "settings.theme.auto_time") => "Selon l'heure locale",
        (AppLanguage::Uk, "settings.theme.auto_time") => "Follow local time",
        (AppLanguage::Fr, "settings.theme.auto_system") => "Selon le systeme",
        (AppLanguage::Uk, "settings.theme.auto_system") => "Follow system setting",
        _ => key,
    }
}
