use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use chrono::Timelike;
use dark_light::Mode;

use crate::i18n::AppLanguage;
use crate::models::{AipDocument, Chart, Notice, Workspace};
use crate::pdf::RenderedPage;

/// State of a PDF rendering
#[derive(Debug, Clone, PartialEq)]
pub enum PdfState {
    Loading,
    Partial(Vec<RenderedPage>),
    Rendered(Vec<RenderedPage>),
    Error(String),
}

/// Which mode the nano sidebar is showing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarMode {
    Airports,
    Workspaces,
    Settings,
    Help,
}

/// Theme behavior selected by the user
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    AutoTime,
    AutoSystem,
}

/// Content of an open tab
#[derive(Debug, Clone, PartialEq)]
pub enum TabContent {
    /// A PDF chart
    Chart { chart: Chart, airport: String },
    /// A textual AIP HTML document
    AipDoc { doc: AipDocument },
    /// ATIS / METAR / TAF from atis.guru
    Atis { icao: String },
    /// Workspace notes editor
    Notes,
}

/// An open tab in the workspace
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub id: String,
    pub content: TabContent,
}

/// Well-known tab ID for the notes tab
pub const NOTES_TAB_ID: &str = "__notes__";

impl Tab {
    /// Create a chart tab
    pub fn chart(chart: Chart, airport: String) -> Self {
        Self {
            id: chart.id.clone(),
            content: TabContent::Chart { chart, airport },
        }
    }

    /// Create the notes tab
    pub fn notes() -> Self {
        Self {
            id: NOTES_TAB_ID.to_string(),
            content: TabContent::Notes,
        }
    }

    pub fn aip_doc(doc: AipDocument) -> Self {
        Self {
            id: doc.id.clone(),
            content: TabContent::AipDoc { doc },
        }
    }

    pub fn atis(icao: String) -> Self {
        Self {
            id: format!("atis_{}", icao),
            content: TabContent::Atis { icao },
        }
    }

    /// Whether this is the notes tab
    pub fn is_notes(&self) -> bool {
        matches!(self.content, TabContent::Notes)
    }

    /// Display title for the tab bar
    pub fn title(&self) -> String {
        match &self.content {
            TabContent::Chart { chart, .. } => chart.display_title().to_string(),
            TabContent::AipDoc { doc } => doc.title(),
            TabContent::Atis { icao } => format!("ATIS {}", icao),
            TabContent::Notes => "Notes".to_string(),
        }
    }

    /// Get the chart ID if this is a chart tab (for persistence)
    pub fn chart_id(&self) -> Option<&str> {
        match &self.content {
            TabContent::Chart { chart, .. } => Some(&chart.id),
            TabContent::AipDoc { .. } => None,
            TabContent::Atis { .. } => None,
            TabContent::Notes => None,
        }
    }
}

/// Global application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current sidebar mode
    pub sidebar_mode: SidebarMode,
    /// Whether the navigator panel is visible
    pub nav_open: bool,
    /// Current ICAO search
    pub search_icao: String,
    /// Whether a search is in progress
    pub loading: bool,
    /// Search error message
    pub error: Option<String>,
    /// Charts from last search
    pub charts: Vec<Chart>,
    /// Notices from last search
    pub notices: Vec<Notice>,
    /// Airport AIP textual document from SIA/NATS
    pub aip_doc: Option<AipDocument>,
    /// Open tabs in the workspace
    pub tabs: Vec<Tab>,
    /// Index of the active tab
    pub active_tab: Option<usize>,
    /// Saved workspaces
    pub workspaces: Vec<Workspace>,
    /// Currently loaded workspace ID (None = no workspace loaded)
    pub active_workspace_id: Option<String>,
    /// PDF download cache (chart_id -> PdfState)
    pub pdf_cache: HashMap<String, PdfState>,
    /// Per-workspace zoom level (chart_id -> zoom%)
    pub chart_zoom: HashMap<String, u32>,
    /// Whether notes are pinned as a side panel
    pub notes_pinned: bool,
    /// Quick chart switcher overlay visibility
    pub quick_switcher_open: bool,
    /// Quick chart switcher query
    pub quick_switcher_query: String,
    /// Whether this state is rendered in a popout window
    pub is_popout: bool,
    /// Whether main window should sync tabs after popout close
    pub popout_sync_pending: bool,
    /// Whether night mode is enabled for charts
    pub night_mode: bool,
    /// Current theme mode preference
    pub theme_mode: ThemeMode,
    /// Current UI language
    pub language: AppLanguage,
}

static THEME_MODE: OnceLock<Mutex<Option<ThemeMode>>> = OnceLock::new();

fn detect_night_mode_from_time() -> bool {
    let hour = chrono::Local::now().hour();
    hour >= 20 || hour < 7
}

fn detect_night_mode_from_system() -> bool {
    match dark_light::detect() {
        Mode::Dark => true,
        Mode::Light => false,
        Mode::Default => detect_night_mode_from_time(),
    }
}

pub fn resolve_night_mode(mode: ThemeMode) -> bool {
    match mode {
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
        ThemeMode::AutoTime => detect_night_mode_from_time(),
        ThemeMode::AutoSystem => detect_night_mode_from_system(),
    }
}

pub fn get_theme_mode() -> ThemeMode {
    let mut guard = THEME_MODE.get_or_init(|| Mutex::new(None)).lock().unwrap();
    if let Some(mode) = *guard {
        mode
    } else {
        let mode = ThemeMode::AutoTime;
        *guard = Some(mode);
        mode
    }
}

pub fn set_theme_mode(mode: ThemeMode) {
    *THEME_MODE.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(mode);
}

pub fn get_night_mode() -> bool {
    resolve_night_mode(get_theme_mode())
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sidebar_mode: SidebarMode::Airports,
            nav_open: true,
            search_icao: String::new(),
            loading: false,
            error: None,
            charts: Vec::new(),
            notices: Vec::new(),
            aip_doc: None,
            tabs: Vec::new(),
            active_tab: None,
            workspaces: Vec::new(),
            active_workspace_id: None,
            pdf_cache: HashMap::new(),
            chart_zoom: HashMap::new(),
            notes_pinned: false,
            quick_switcher_open: false,
            quick_switcher_query: String::new(),
            is_popout: false,
            popout_sync_pending: false,
            night_mode: false,
            theme_mode: ThemeMode::AutoTime,
            language: AppLanguage::Fr,
        }
    }
}
