use std::sync::LazyLock;

use regex::Regex;
use scraper::{Html, Selector};

use crate::airac::AiracCycle;
use crate::models::{Chart, ChartCategory, ChartSource};

static EXCLUDED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(DATA|TEXT|TXT|VPE|PATC)").unwrap());
static RWY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"RWY_(\d{2}[LRC]?)").unwrap());
static CAT_ILS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)CAT_(I{1,3}(?:_I{1,3})*)").unwrap());
static CATEGORY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_(ADC|APDC|GMC|SID|STAR|IAC|VAC|VLC|TEM)_").unwrap());
static RWY_STRIP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"_?RWY_\w+").unwrap());
pub(crate) static INSTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)_INSTR_\d{2}\.pdf$").unwrap());

/// Base URL for SIA eAIP.
fn eaip_base_url(airac: &AiracCycle) -> String {
    format!(
        "https://www.sia.aviation-civile.gouv.fr/media/dvd/{}/FRANCE/{}/html/eAIP",
        airac.sia_cycle_name(),
        airac.sia_airac_date(),
    )
}

/// Build a shared HTTP client.
fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

/// Fetch charts from the SIA eAIP for a given ICAO code.
pub async fn fetch_charts(icao: &str, airac: &AiracCycle) -> Result<Vec<Chart>, String> {
    let base = eaip_base_url(airac);
    let url = format!("{}/FR-AD-2.{}-fr-FR.html", base, icao.to_uppercase());

    let client = http_client()?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let html = resp.text().await.map_err(|e| format!("Read error: {e}"))?;
    let charts = parse_sia_html(&html, icao, &airac.code);
    Ok(charts)
}

/// Parse the SIA HTML page and extract PDF chart links.
pub(crate) fn parse_sia_html(html: &str, icao: &str, airac_code: &str) -> Vec<Chart> {
    let doc = Html::parse_document(html);
    let a_sel = Selector::parse("a[href]").unwrap();

    let mut raw_charts: Vec<RawChart> = Vec::new();

    for elem in doc.select(&a_sel) {
        let href = match elem.value().attr("href") {
            Some(h) if h.ends_with(".pdf") => h,
            _ => continue,
        };

        let filename = href.rsplit('/').next().unwrap_or(href);

        // Exclude non-chart files
        if EXCLUDED_RE.is_match(filename) {
            continue;
        }

        let upper = filename.to_uppercase();
        let category = detect_category(&upper);
        let tags = extract_tags(&upper);
        let runways = extract_runways(&upper);
        let subtitle = extract_subtitle(&upper, icao);

        let provider_relative_url = if href.starts_with("http") {
            href.to_string()
        } else {
            href.trim_start_matches("./").to_string()
        };

        raw_charts.push(RawChart {
            category,
            subtitle,
            filename: filename.to_string(),
            provider_relative_url,
            airac_code: airac_code.to_string(),
            tags,
            runways,
        });
    }

    // Pagination: group by (category, subtitle), assign page numbers
    paginate(raw_charts, icao)
}

struct RawChart {
    category: ChartCategory,
    subtitle: String,
    filename: String,
    provider_relative_url: String,
    airac_code: String,
    tags: Vec<String>,
    runways: Vec<String>,
}

/// Detect chart category from filename patterns.
fn detect_category(upper: &str) -> ChartCategory {
    if upper.contains("_ADC_") || upper.contains("_ADCHART_") {
        ChartCategory::Aerodrome
    } else if upper.contains("_APDC_") || upper.contains("_PARKING") {
        ChartCategory::Parking
    } else if upper.contains("_GMC_") || upper.contains("_GROUND") {
        ChartCategory::Ground
    } else if upper.contains("_SID_") {
        ChartCategory::Sid
    } else if upper.contains("_STAR_") {
        ChartCategory::Star
    } else if upper.contains("_IAC_") {
        ChartCategory::Iac
    } else if upper.contains("_VAC_") {
        ChartCategory::Vac
    } else if upper.contains("_VLC_") {
        ChartCategory::Vlc
    } else if upper.contains("_TEM_") {
        ChartCategory::Tem
    } else {
        ChartCategory::Other
    }
}

/// Extract tags from filename.
fn extract_tags(upper: &str) -> Vec<String> {
    let mut tags = Vec::new();

    if upper.contains("_FNA") {
        tags.push("App. Finale".to_string());
    }
    if upper.contains("_INA") {
        tags.push("App. Initiale".to_string());
    }
    if upper.contains("_VPT") {
        tags.push("VPT".to_string());
    }
    if upper.contains("_MVL") {
        tags.push("MVL".to_string());
    }
    if upper.contains("_NIGHT") {
        tags.push("Nuit".to_string());
    }
    if upper.contains("_RNAV") {
        tags.push("RNAV".to_string());
    }
    if upper.contains("_RNP") {
        tags.push("RNP".to_string());
    }
    if upper.contains("_LOC") {
        tags.push("LOC".to_string());
    }
    if upper.contains("_ILS") || upper.contains("ILS") {
        tags.push("ILS".to_string());
    }
    if upper.contains("_DME") {
        tags.push("DME".to_string());
    }

    // ILS categories
    for cap in CAT_ILS_RE.captures_iter(upper) {
        let cat = cap[1].replace('_', "/");
        tags.push(format!("CAT {cat}"));
    }

    // Runways as tags
    for cap in RWY_RE.captures_iter(upper) {
        tags.push(cap[1].to_string());
    }

    tags.sort();
    tags.dedup();
    tags
}

/// Extract runway identifiers from filename.
fn extract_runways(upper: &str) -> Vec<String> {
    let mut rwys: Vec<String> = RWY_RE
        .captures_iter(upper)
        .map(|c| c[1].to_string())
        .collect();

    // Disambiguate: remove "26" if "26L" or "26R" exists
    let suffixed: Vec<String> = rwys
        .iter()
        .filter(|r| r.len() == 3)
        .map(|r| r[..2].to_string())
        .collect();
    rwys.retain(|r| r.len() > 2 || !suffixed.contains(r));

    rwys.sort();
    rwys.dedup();
    rwys
}

/// Extract a subtitle from the filename by removing structural parts.
fn extract_subtitle(upper: &str, icao: &str) -> String {
    let name = upper
        .trim_end_matches(".PDF")
        .replace(&format!("AD_2_{}_", icao.to_uppercase()), "")
        .replace("AD_2_", "")
        .replace(&icao.to_uppercase(), "");

    // Remove category codes
    let cleaned = CATEGORY_RE.replace_all(&name, "_");

    // Remove RWY parts
    let cleaned = RWY_STRIP_RE.replace_all(&cleaned, "");

    // Clean up underscores
    let cleaned = cleaned
        .trim_matches('_')
        .replace("__", "_")
        .replace('_', " ");

    cleaned.trim().to_string()
}

/// Group charts by (category, subtitle) and assign page numbers.
fn paginate(raw: Vec<RawChart>, icao: &str) -> Vec<Chart> {
    use std::collections::HashMap;

    // Group by key
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, chart) in raw.iter().enumerate() {
        let key = format!("{}|{}", chart.category.label(), chart.subtitle);
        groups.entry(key).or_default().push(i);
    }

    let mut charts = Vec::new();
    for (i, rc) in raw.iter().enumerate() {
        let key = format!("{}|{}", rc.category.label(), rc.subtitle);
        let group = &groups[&key];
        let page = if group.len() > 1 {
            let pos = group.iter().position(|&x| x == i).unwrap() + 1;
            Some(format!("{}/{}", pos, group.len()))
        } else {
            None
        };

        let id = format!("{}-{}-{:02}", icao.to_uppercase(), rc.category.label(), i);
        charts.push(Chart {
            id,
            source: ChartSource::Sia,
            category: rc.category.clone(),
            subtitle: rc.subtitle.clone(),
            filename: rc.filename.clone(),
            provider_relative_url: rc.provider_relative_url.clone(),
            airac_code: rc.airac_code.clone(),
            page,
            tags: rc.tags.clone(),
            runways: rc.runways.clone(),
            custom_title: None,
        });
    }

    charts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_category() {
        assert_eq!(
            detect_category("AD_2_LFPG_ADC_01.PDF"),
            ChartCategory::Aerodrome
        );
        assert_eq!(
            detect_category("AD_2_LFPG_APDC_01.PDF"),
            ChartCategory::Parking
        );
        assert_eq!(
            detect_category("AD_2_LFPG_GMC_01.PDF"),
            ChartCategory::Ground
        );
        assert_eq!(
            detect_category("AD_2_LFPG_SID_RWY_27L.PDF"),
            ChartCategory::Sid
        );
        assert_eq!(
            detect_category("AD_2_LFPG_STAR_RWY_09R.PDF"),
            ChartCategory::Star
        );
        assert_eq!(
            detect_category("AD_2_LFPG_IAC_RWY_27L.PDF"),
            ChartCategory::Iac
        );
        assert_eq!(detect_category("AD_2_LFPG_VAC_01.PDF"), ChartCategory::Vac);
        assert_eq!(
            detect_category("SOME_RANDOM_FILE.PDF"),
            ChartCategory::Other
        );
    }

    #[test]
    fn test_extract_runways() {
        let rwys = extract_runways("AD_2_LFPG_SID_RWY_27L_RWY_27R.PDF");
        assert!(rwys.contains(&"27L".to_string()));
        assert!(rwys.contains(&"27R".to_string()));
        // Bare "27" should be filtered out when "27L" or "27R" exist
        let rwys = extract_runways("AD_2_LFPG_SID_RWY_27_RWY_27L.PDF");
        assert!(!rwys.contains(&"27".to_string()));
        assert!(rwys.contains(&"27L".to_string()));
    }

    #[test]
    fn test_extract_tags() {
        let tags = extract_tags("AD_2_LFPG_IAC_ILS_RWY_27L_CAT_II_III.PDF");
        assert!(tags.contains(&"ILS".to_string()));
        assert!(tags.contains(&"27L".to_string()));
        assert!(tags.contains(&"CAT II/III".to_string()));
    }

    #[test]
    fn test_excluded_files() {
        assert!(EXCLUDED_RE.is_match("AD_2_LFPG_DATA_01.PDF"));
        assert!(EXCLUDED_RE.is_match("AD_2_LFPG_TEXT_01.PDF"));
        assert!(!EXCLUDED_RE.is_match("AD_2_LFPG_ADC_01.PDF"));
    }

    #[test]
    fn test_instr_filter() {
        assert!(INSTR_RE.is_match("AD_2_LFPG_INSTR_01.pdf"));
        assert!(!INSTR_RE.is_match("AD_2_LFPG_IAC_01.pdf"));
    }

    #[test]
    fn test_parse_sia_html_extracts_pdf_links() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFBO_ADC_01.pdf">Chart 1</a>
                <a href="./AD_2_LFBO_SID_RWY_14L_01.pdf">SID</a>
                <a href="./AD_2_LFBO_DATA_01.pdf">DATA excluded</a>
                <a href="./styles.css">Not a PDF</a>
            </body></html>
        "#;
        let charts = parse_sia_html(html, "LFBO", "2602");
        assert_eq!(charts.len(), 2);
        assert_eq!(charts[0].category, ChartCategory::Aerodrome);
        assert_eq!(charts[1].category, ChartCategory::Sid);
        assert!(charts[1].runways.contains(&"14L".to_string()));
        assert_eq!(charts[0].provider_relative_url, "AD_2_LFBO_ADC_01.pdf");
    }

    #[test]
    fn test_pagination() {
        // Same subtitle → pages get numbered
        let html = r#"
            <html><body>
                <a href="./AD_2_LFBO_ADC.pdf">Page 1</a>
                <a href="./AD_2_LFBO_ADC.pdf">Page 2</a>
            </body></html>
        "#;
        let charts = parse_sia_html(html, "LFBO", "2602");
        assert_eq!(charts.len(), 2);
        assert_eq!(charts[0].page, Some("1/2".to_string()));
        assert_eq!(charts[1].page, Some("2/2".to_string()));
    }

    #[test]
    fn test_different_subtitles_not_paginated() {
        // Different subtitles → each is its own chart (no pagination)
        let html = r#"
            <html><body>
                <a href="./AD_2_LFBO_ADC_01.pdf">P1</a>
                <a href="./AD_2_LFBO_ADC_02.pdf">P2</a>
            </body></html>
        "#;
        let charts = parse_sia_html(html, "LFBO", "2602");
        assert_eq!(charts.len(), 2);
        assert_eq!(charts[0].page, None);
        assert_eq!(charts[1].page, None);
    }
}
