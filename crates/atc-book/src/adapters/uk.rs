use regex::Regex;
use scraper::{Html, Selector};

use chrono::Duration;

use crate::airac::AiracCycle;
use crate::models::{Chart, ChartCategory, ChartSource};

/// Fetch charts from the UK NATS eAIP (Aurora) for a given ICAO code.
pub async fn fetch_charts(icao: &str, airac: &AiracCycle) -> Result<Vec<Chart>, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;
    let mut tried_urls = Vec::new();

    for (part, code) in candidate_airac_parts(airac) {
        let url = build_uk_url(&part, icao);
        tried_urls.push(url.clone());

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("UK NATS request failed: {e}"))?;

        if resp.status().is_success() {
            let html = resp.text().await.map_err(|e| format!("Read error: {e}"))?;
            return parse_uk_html(&html, icao, &code);
        }

        if resp.status().as_u16() != 404 {
            return Err(format!("HTTP {}", resp.status()));
        }
    }

    Err(format!("HTTP 404 (tried {})", tried_urls.join(", ")))
}

fn build_uk_url(airac_part: &str, icao: &str) -> String {
    format!(
        "https://www.aurora.nats.co.uk/htmlAIP/Publications/{}/html/eAIP/EG-AD-2.{}-en-GB.html",
        airac_part,
        icao.to_uppercase()
    )
}

fn candidate_airac_parts(airac: &AiracCycle) -> Vec<(String, String)> {
    let mut parts = Vec::new();
    let mut date = airac.start_date;
    for _ in 0..3 {
        let cycle = AiracCycle::for_date(date);
        parts.push((cycle.nats_airac_part(), cycle.code));
        date = date - Duration::days(28);
    }
    parts
}

fn parse_uk_html(html: &str, icao: &str, airac_code: &str) -> Result<Vec<Chart>, String> {
    let doc = Html::parse_document(html);
    let a_sel = Selector::parse("a[href]").unwrap();
    let rwy_re = Regex::new(r"(?i)(?:RWY|RUNWAY)\s*(\d{2}[LRC]?)").unwrap();

    let mut charts = Vec::new();

    for (i, elem) in doc.select(&a_sel).enumerate() {
        let href = match elem.value().attr("href") {
            Some(h) if h.to_lowercase().ends_with(".pdf") => h,
            _ => continue,
        };

        let link_text = elem.text().collect::<Vec<_>>().join(" ");
        let upper = link_text.to_uppercase();

        let category = detect_uk_category(&upper);
        let tags = extract_uk_tags(&upper, &rwy_re);
        let runways = extract_uk_runways(&upper, &rwy_re);

        let provider_relative_url = if href.starts_with("http") {
            href.to_string()
        } else {
            href.trim_start_matches("./").to_string()
        };

        let filename = href.rsplit('/').next().unwrap_or(href).to_string();
        let subtitle = link_text.trim().to_string();
        let id = format!("{}-UK-{:02}", icao.to_uppercase(), i);

        charts.push(Chart {
            id,
            source: ChartSource::Uk,
            category,
            subtitle,
            filename,
            provider_relative_url,
            linked_provider_relative_urls: Vec::new(),
            airac_code: airac_code.to_string(),
            page: None,
            tags,
            runways,
            custom_title: None,
        });
    }

    Ok(charts)
}

fn detect_uk_category(upper: &str) -> ChartCategory {
    if upper.contains("AERODROME CHART") || upper.contains("ADC") {
        ChartCategory::Aerodrome
    } else if upper.contains("AIRCRAFT PARKING") || upper.contains("PARKING") {
        ChartCategory::Parking
    } else if upper.contains("GROUND MOVEMENT") || upper.contains("GMC") {
        ChartCategory::Ground
    } else if upper.contains("STANDARD DEPARTURE") || upper.contains("SID") {
        ChartCategory::Sid
    } else if upper.contains("STANDARD ARRIVAL") || upper.contains("STAR") {
        ChartCategory::Star
    } else if upper.contains("INSTRUMENT APPROACH") || upper.contains("IAC") {
        ChartCategory::Iac
    } else if upper.contains("VISUAL APPROACH") {
        ChartCategory::Vac
    } else {
        ChartCategory::Other
    }
}

fn extract_uk_tags(upper: &str, rwy_re: &Regex) -> Vec<String> {
    let mut tags = Vec::new();

    if upper.contains("ILS") {
        tags.push("ILS".to_string());
    }
    if upper.contains("LOC") {
        tags.push("LOC".to_string());
    }
    if upper.contains("RNP") {
        tags.push("RNP".to_string());
    }
    if upper.contains("RNAV") {
        tags.push("RNAV".to_string());
    }
    if upper.contains("VOR") {
        tags.push("VOR".to_string());
    }
    if upper.contains("NDB") {
        tags.push("NDB".to_string());
    }
    if upper.contains("DME") {
        tags.push("DME".to_string());
    }
    if upper.contains("VISUAL") {
        tags.push("VISUAL".to_string());
    }
    if upper.contains("CAT II") || upper.contains("CAT III") {
        if upper.contains("CAT III") {
            tags.push("CAT III".to_string());
        } else {
            tags.push("CAT II".to_string());
        }
    }

    for cap in rwy_re.captures_iter(upper) {
        tags.push(cap[1].to_string());
    }

    tags.sort();
    tags.dedup();
    tags
}

fn extract_uk_runways(upper: &str, rwy_re: &Regex) -> Vec<String> {
    let mut rwys: Vec<String> = rwy_re
        .captures_iter(upper)
        .map(|c| c[1].to_string())
        .collect();
    rwys.sort();
    rwys.dedup();
    rwys
}
