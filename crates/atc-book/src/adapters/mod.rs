pub mod atlas;
pub mod sia;
pub mod sofia;
pub mod sofia_vac;
pub mod supaip;
pub mod uk;

use crate::airac::AiracCycle;
use crate::models::{AipDocSource, AipDocument, Chart, Notice};
use crate::persistence;

/// Result of a full airport search across all adapters.
pub struct SearchResult {
    pub charts: Vec<Chart>,
    pub notices: Vec<Notice>,
    pub aip_doc: Option<AipDocument>,
    pub errors: Vec<String>,
}

/// Search all relevant adapters for an ICAO code.
/// Returns cached results when available for the current AIRAC cycle.
pub async fn search_airport(icao: &str, airac: &AiracCycle) -> SearchResult {
    let icao = icao.to_uppercase();

    // Try cache first
    {
        let conn = persistence::db().lock().unwrap();
        if let Some((charts, notices)) =
            persistence::cache::get_cached_search(&conn, &icao, &airac.code)
        {
            return SearchResult {
                charts,
                notices,
                aip_doc: build_aip_doc_ref(&icao, airac),
                errors: Vec::new(),
            };
        }
    }

    // Cache miss — fetch from adapters
    let mut charts = Vec::new();
    let mut notices = Vec::new();
    let mut errors = Vec::new();

    if icao.starts_with("EG") {
        match uk::fetch_charts(&icao, airac).await {
            Ok(c) => charts.extend(c),
            Err(e) => errors.push(format!("UK NATS: {e}")),
        }
    } else {
        match sia::fetch_charts(&icao, airac).await {
            Ok(c) => charts.extend(c),
            Err(e) => errors.push(format!("SIA: {e}")),
        }

        if icao.starts_with("LF") {
            if let Ok(c) = atlas::fetch_charts(&icao, airac).await {
                charts.extend(c);
            }
            if let Ok(c) = sofia_vac::fetch_charts(&icao, airac).await {
                charts.extend(c);
            }
            if let Ok(c) = supaip::fetch_charts(&icao, airac).await {
                charts.extend(c);
            }
        }
    }

    // Filter out INSTR charts
    charts.retain(|c| !sia::INSTR_RE.is_match(&c.filename));

    // Deduplicate by effective URL for the requested AIRAC.
    charts.sort_by(|a, b| a.url_for_airac(airac).cmp(&b.url_for_airac(airac)));
    charts.dedup_by(|a, b| a.url_for_airac(airac) == b.url_for_airac(airac));

    // Sort by category then subtitle
    charts.sort_by(|a, b| {
        a.category
            .sort_order()
            .cmp(&b.category.sort_order())
            .then_with(|| a.subtitle.cmp(&b.subtitle))
    });

    // Fetch NOTAMs for LF*
    if icao.starts_with("LF") {
        match sofia::fetch_notices(&icao).await {
            Ok(n) => notices = n,
            Err(_) => {} // Non-critical
        }
    }

    // Store in cache (only if no errors — partial results should not be cached)
    if errors.is_empty() {
        let conn = persistence::db().lock().unwrap();
        persistence::cache::put_cached_search(&conn, &icao, &airac.code, &charts, &notices);
    }

    SearchResult {
        charts,
        notices,
        aip_doc: build_aip_doc_ref(&icao, airac),
        errors,
    }
}

fn build_aip_doc_ref(icao: &str, airac: &AiracCycle) -> Option<AipDocument> {
    let icao = icao.to_uppercase();
    if icao.starts_with("EG") {
        return Some(AipDocument {
            id: format!("{}-AIP-UK", icao),
            icao: icao.clone(),
            source: AipDocSource::Uk,
            provider_relative_url: format!("EG-AD-2.{}-en-GB.html", icao),
            airac_code: airac.code.clone(),
        });
    }

    if icao.starts_with('L') {
        return Some(AipDocument {
            id: format!("{}-AIP-SIA", icao),
            icao: icao.clone(),
            source: AipDocSource::Sia,
            provider_relative_url: format!("FR-AD-2.{}-fr-FR.html", icao),
            airac_code: airac.code.clone(),
        });
    }

    None
}
pub mod atis_guru;
pub mod workspace_repository_sqlite;
