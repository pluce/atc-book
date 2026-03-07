use crate::airac::AiracCycle;
use crate::models::{Chart, ChartCategory, ChartSource};

/// Check if a VAC chart exists via the legacy SOFIA/Atlas VAC publication path.
pub async fn fetch_charts(icao: &str, airac: &AiracCycle) -> Result<Vec<Chart>, String> {
    let icao = icao.to_uppercase();
    let url = format!(
        "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/Atlas-VAC/PDF_AIPparSSection/VAC/AD/AD-2.{}.pdf",
        airac.sia_cycle_name(),
        icao
    );
    let provider_relative_url = format!("AD-2.{icao}.pdf");

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .head(&url)
        .send()
        .await
        .map_err(|e| format!("SOFIA VAC request failed: {e}"))?;

    if resp.status().is_success() || resp.status().as_u16() == 302 {
        return Ok(vec![Chart {
            id: format!("{}-SOFIA-VAC", icao),
            source: ChartSource::SofiaVac,
            category: ChartCategory::Vac,
            subtitle: format!("Sofia VAC {icao}"),
            filename: format!("AD-2.{icao}.pdf"),
            provider_relative_url,
            airac_code: airac.code.clone(),
            page: None,
            tags: vec!["VAC".to_string()],
            runways: Vec::new(),
            custom_title: None,
        }]);
    }

    Ok(Vec::new())
}
