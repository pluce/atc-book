use crate::airac::AiracCycle;
use crate::models::{Chart, ChartCategory, ChartSource};

/// Check if an Atlas VAC PDF exists for a given ICAO code.
pub async fn fetch_charts(icao: &str, airac: &AiracCycle) -> Result<Vec<Chart>, String> {
    let base = format!(
        "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/FRANCE/{}/html/eAIP",
        airac.sia_cycle_name(),
        airac.sia_airac_date(),
    );
    let url = format!(
        "{}/Cartes/{}/AD-2.{}.pdf",
        base,
        icao.to_uppercase(),
        icao.to_uppercase()
    );
    let provider_relative_url = format!(
        "Cartes/{}/AD-2.{}.pdf",
        icao.to_uppercase(),
        icao.to_uppercase()
    );

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .head(&url)
        .send()
        .await
        .map_err(|e| format!("Atlas request failed: {e}"))?;

    if resp.status().is_success() || resp.status().as_u16() == 302 {
        let id = format!("{}-ATLAS-VAC", icao.to_uppercase());
        Ok(vec![Chart {
            id,
            source: ChartSource::Atlas,
            category: ChartCategory::Vac,
            subtitle: format!("Atlas VAC {}", icao.to_uppercase()),
            filename: format!("AD-2.{}.pdf", icao.to_uppercase()),
            provider_relative_url,
            linked_provider_relative_urls: Vec::new(),
            airac_code: airac.code.clone(),
            page: None,
            tags: vec!["VAC".to_string()],
            runways: Vec::new(),
            custom_title: None,
        }])
    } else {
        Ok(Vec::new())
    }
}
