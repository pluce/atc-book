use reqwest::header::{COOKIE, SET_COOKIE};
use scraper::{Html, Selector};

use crate::airac::AiracCycle;
use crate::models::{Chart, ChartCategory, ChartSource};

/// Fetch SUP AIP charts for a given ICAO code.
pub async fn fetch_charts(icao: &str, airac: &AiracCycle) -> Result<Vec<Chart>, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    // Step 1: GET the search page to extract form_key and cookies
    let page_url = "https://www.sia.aviation-civile.gouv.fr/supplements-aip";
    let resp = client
        .get(page_url)
        .send()
        .await
        .map_err(|e| format!("SupAIP GET failed: {e}"))?;

    let cookies: Vec<String> = resp
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .collect();
    let cookie_header = cookies.join("; ");

    let html = resp
        .text()
        .await
        .map_err(|e| format!("SupAIP read error: {e}"))?;
    let doc = Html::parse_document(&html);

    // Extract form_key from hidden input
    let form_key = {
        let sel = Selector::parse("input[name=\"form_key\"]").unwrap();
        doc.select(&sel)
            .next()
            .and_then(|el| el.value().attr("value"))
            .map(|s| s.to_string())
            .ok_or_else(|| "form_key not found".to_string())?
    };

    // Step 2: POST search with ICAO location
    let params = [("form_key", form_key.as_str()), ("location", icao)];

    let resp = client
        .post(page_url)
        .header(COOKIE, &cookie_header)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("SupAIP POST failed: {e}"))?;

    let result_html = resp
        .text()
        .await
        .map_err(|e| format!("SupAIP result read error: {e}"))?;
    let result_doc = Html::parse_document(&result_html);

    // Step 3: Parse links with class "lien_sup_aip"
    let link_sel = Selector::parse("a.lien_sup_aip").unwrap();
    let mut charts = Vec::new();

    for (i, elem) in result_doc.select(&link_sel).enumerate() {
        let href = match elem.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        // Step 4: Resolve HEAD to get final PDF URL
        let resolved = client
            .head(href)
            .header(COOKIE, &cookie_header)
            .send()
            .await;

        let final_url = match resolved {
            Ok(r) => {
                let url = r.url().to_string();
                // Filter out non-PDF
                if !url.to_lowercase().ends_with(".pdf") {
                    continue;
                }
                url
            }
            Err(_) => continue,
        };

        let filename = final_url.rsplit('/').next().unwrap_or("").to_string();
        let title = elem.text().collect::<Vec<_>>().join(" ").trim().to_string();
        let provider_relative_url = final_url
            .strip_prefix("https://www.sia.aviation-civile.gouv.fr")
            .map(|p| p.to_string())
            .unwrap_or_else(|| final_url.clone());

        let id = format!("{}-SUPAIP-{:02}", icao.to_uppercase(), i);
        charts.push(Chart {
            id,
            source: ChartSource::SupAip,
            category: ChartCategory::SupAip,
            subtitle: if title.is_empty() {
                filename.clone()
            } else {
                title
            },
            filename,
            provider_relative_url,
            airac_code: airac.code.clone(),
            page: None,
            tags: vec!["SUP AIP".to_string()],
            runways: Vec::new(),
            custom_title: None,
        });
    }

    Ok(charts)
}
