use serde::{Deserialize, Serialize};

use crate::airac::AiracCycle;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChartSource {
    Sia,
    Atlas,
    SofiaVac,
    SupAip,
    Uk,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AipDocSource {
    Sia,
    Uk,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChartCategory {
    Aerodrome,
    Parking,
    Ground,
    Sid,
    Star,
    Iac,
    Vac,
    Vlc,
    Tem,
    SupAip,
    Other,
}

impl ChartCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Aerodrome => "Aerodrome",
            Self::Parking => "Parking",
            Self::Ground => "Ground",
            Self::Sid => "SID",
            Self::Star => "STAR",
            Self::Iac => "IAC",
            Self::Vac => "VAC",
            Self::Vlc => "VLC",
            Self::Tem => "TEM",
            Self::SupAip => "SupAIP",
            Self::Other => "Other",
        }
    }

    pub fn sort_order(&self) -> u8 {
        match self {
            Self::Aerodrome => 0,
            Self::Parking => 1,
            Self::Ground => 2,
            Self::Sid => 3,
            Self::Star => 4,
            Self::Iac => 5,
            Self::Vac => 6,
            Self::Vlc => 7,
            Self::Tem => 8,
            Self::SupAip => 9,
            Self::Other => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub id: String,
    pub source: ChartSource,
    pub category: ChartCategory,
    pub subtitle: String,
    pub filename: String,
    /// Provider-relative path used to rebuild a valid URL for any AIRAC cycle.
    pub provider_relative_url: String,
    /// AIRAC code associated with this chart when it was discovered (ex: 2603).
    pub airac_code: String,
    pub page: Option<String>,
    pub tags: Vec<String>,
    pub runways: Vec<String>,
    pub custom_title: Option<String>,
}

impl Chart {
    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or_else(|| {
            if self.subtitle.is_empty() {
                self.category.label()
            } else {
                &self.subtitle
            }
        })
    }

    fn provider_base_for_airac(&self, airac: &AiracCycle) -> String {
        match self.source {
            ChartSource::Sia | ChartSource::Atlas => format!(
                "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/FRANCE/{}/html/eAIP",
                airac.sia_cycle_name(),
                airac.sia_airac_date(),
            ),
            ChartSource::SofiaVac => format!(
                "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/Atlas-VAC/PDF_AIPparSSection/VAC/AD",
                airac.sia_cycle_name(),
            ),
            ChartSource::Uk => format!(
                "https://www.aurora.nats.co.uk/htmlAIP/Publications/{}/html/eAIP",
                airac.nats_airac_part(),
            ),
            ChartSource::SupAip => "https://www.sia.aviation-civile.gouv.fr".to_string(),
        }
    }

    /// Resolve the effective URL for a target AIRAC cycle.
    pub fn url_for_airac(&self, airac: &AiracCycle) -> String {
        if self.provider_relative_url.starts_with("http://")
            || self.provider_relative_url.starts_with("https://")
        {
            return self.provider_relative_url.clone();
        }

        let base = self.provider_base_for_airac(airac);
        let rel = self.provider_relative_url.trim_start_matches('/');
        format!("{}/{}", base.trim_end_matches('/'), rel)
    }

    /// Resolve URL using the current AIRAC cycle.
    pub fn runtime_url(&self) -> String {
        self.url_for_airac(&AiracCycle::current())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoticeSource {
    Sofia,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notice {
    pub id: String,
    pub icao: String,
    pub source: NoticeSource,
    pub identifier: String,
    pub notice_type: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub content: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    /// Aéroports couverts par ce workspace (ex: ["LFPG", "LFPB"])
    pub airports: Vec<String>,
    /// Cartes sélectionnées, regroupées par ICAO
    pub chart_refs: Vec<WorkspaceChart>,
    /// IDs des onglets ouverts (dans l'ordre)
    pub open_tabs: Vec<String>,
    /// Index de l'onglet actif
    pub active_tab_index: Option<usize>,
    /// Onglets eAIP et ATIS persistés (non-carte)
    pub extra_tabs: Vec<ExtraTab>,
    /// Notes de briefing
    pub notes: Option<String>,
    /// Etat d'epingle du panneau Notes pour ce workspace
    pub notes_pinned: Option<bool>,
    /// Largeur du panneau Notes (px)
    pub notes_panel_width: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Un onglet non-carte (eAIP ou ATIS) persisté dans un workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtraTab {
    Atis { icao: String },
    AipDoc { doc: AipDocument },
}

/// Référence à une carte dans un workspace, associée à un aéroport
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceChart {
    pub airport: String,
    pub chart: Chart,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AipDocument {
    pub id: String,
    pub icao: String,
    pub source: AipDocSource,
    pub provider_relative_url: String,
    pub airac_code: String,
}

impl AipDocument {
    pub fn title(&self) -> String {
        match self.source {
            AipDocSource::Sia => format!("AIP {} (SIA)", self.icao),
            AipDocSource::Uk => format!("AIP {} (NATS)", self.icao),
        }
    }

    fn provider_base_for_airac(&self, airac: &AiracCycle) -> String {
        match self.source {
            AipDocSource::Sia => format!(
                "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/FRANCE/{}/html/eAIP",
                airac.sia_cycle_name(),
                airac.sia_airac_date(),
            ),
            AipDocSource::Uk => format!(
                "https://www.aurora.nats.co.uk/htmlAIP/Publications/{}/html/eAIP",
                airac.nats_airac_part(),
            ),
        }
    }

    pub fn url_for_airac(&self, airac: &AiracCycle) -> String {
        if self.provider_relative_url.starts_with("http://")
            || self.provider_relative_url.starts_with("https://")
        {
            return self.provider_relative_url.clone();
        }

        let base = self.provider_base_for_airac(airac);
        let rel = self.provider_relative_url.trim_start_matches('/');
        format!("{}/{}", base.trim_end_matches('/'), rel)
    }

    pub fn runtime_url(&self) -> String {
        self.url_for_airac(&AiracCycle::current())
    }
}
