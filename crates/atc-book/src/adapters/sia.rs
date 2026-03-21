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
static RWY_GROUP_RE: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r"(?i)_RWY_?(?:ALL|\d{2}[LRC]?)(?:-(?:ALL|\d{2}[LRC]?))*").unwrap()
    });
static STEM_RWY_PARTS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?P<prefix>.+)_RWY_?(?P<group>(?:ALL|\d{2}[LRC]?)(?:-(?:ALL|\d{2}[LRC]?))*)_(?P<suffix>.+)$")
        .unwrap()
});
static INSTR_FILE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?P<base>.+)_INSTR_(?P<page>\d{2})\.pdf$").unwrap());

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

    println!(
        "[sia] fetch_charts icao={} airac={} url={} status={}",
        icao.to_uppercase(),
        airac.code,
        url,
        resp.status()
    );

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let html = resp.text().await.map_err(|e| format!("Read error: {e}"))?;
    let charts = parse_sia_html(&html, icao, &airac.code);
    println!(
        "[sia] parse result icao={} airac={} charts={}",
        icao.to_uppercase(),
        airac.code,
        charts.len()
    );
    Ok(charts)
}

/// Parse the SIA HTML page and extract PDF chart links.
pub(crate) fn parse_sia_html(html: &str, icao: &str, airac_code: &str) -> Vec<Chart> {
    use std::collections::HashMap;

    let doc = Html::parse_document(html);
    let a_sel = Selector::parse("a[href]").unwrap();

    let mut raw_charts: Vec<RawChart> = Vec::new();
    let mut instr_by_parent: HashMap<String, Vec<InstrDoc>> = HashMap::new();
    let mut instr_detected = 0usize;

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

        if let Some((parent_stem, page_index)) = parse_instr_parent(filename) {
            instr_detected += 1;
            println!(
                "[sia] instr detected icao={} file={} parent={} page={}",
                icao.to_uppercase(),
                filename,
                parent_stem,
                page_index
            );
            instr_by_parent
                .entry(parent_stem)
                .or_default()
                .push(InstrDoc {
                    page_index,
                    provider_relative_url,
                });
            continue;
        }

        raw_charts.push(RawChart {
            category,
            subtitle,
            filename: filename.to_string(),
            provider_relative_url,
            linked_provider_relative_urls: Vec::new(),
            airac_code: airac_code.to_string(),
            tags,
            runways,
        });
    }

    for chart in raw_charts.iter_mut() {
        let stem = filename_stem_upper(&chart.filename);
        let canonical_stem = strip_numeric_suffix(&stem);

        let mut linked: Vec<InstrDoc> = instr_by_parent.remove(&stem).unwrap_or_default();

        if linked.is_empty() {
            linked = instr_by_parent
                .remove(&canonical_stem)
                .unwrap_or_default();
        }

        if linked.is_empty() {
            let matching_keys: Vec<String> = instr_by_parent
                .keys()
                .filter(|key| {
                    let key_canonical = strip_numeric_suffix(key.as_str());
                    stems_match_for_instr_link(&canonical_stem, &key_canonical)
                })
                .cloned()
                .collect();

            for key in matching_keys {
                if let Some(mut docs) = instr_by_parent.remove(&key) {
                    linked.append(&mut docs);
                }
            }
        }

        linked.sort_by_key(|doc| doc.page_index);
        chart.linked_provider_relative_urls = linked
            .into_iter()
            .map(|doc| doc.provider_relative_url)
            .collect();

        if !chart.linked_provider_relative_urls.is_empty() {
            println!(
                "[sia] instr linked icao={} chart={} linked_count={} linked={:?}",
                icao.to_uppercase(),
                chart.filename,
                chart.linked_provider_relative_urls.len(),
                chart.linked_provider_relative_urls
            );
        }
    }

    let mut unlinked_instr = 0usize;
    for docs in instr_by_parent.values() {
        unlinked_instr += docs.len();
    }
    if unlinked_instr > 0 {
        let keys: Vec<String> = instr_by_parent.keys().cloned().collect();
        println!(
            "[sia] instr unlinked icao={} count={} parent_keys={:?}",
            icao.to_uppercase(),
            unlinked_instr,
            keys
        );
    }
    println!(
        "[sia] summary icao={} airac={} raw_charts={} instr_detected={} instr_unlinked={}",
        icao.to_uppercase(),
        airac_code,
        raw_charts.len(),
        instr_detected,
        unlinked_instr
    );

    // Pagination: group by (category, subtitle), assign page numbers
    paginate(raw_charts, icao)
}

struct RawChart {
    category: ChartCategory,
    subtitle: String,
    filename: String,
    provider_relative_url: String,
    linked_provider_relative_urls: Vec<String>,
    airac_code: String,
    tags: Vec<String>,
    runways: Vec<String>,
}

#[derive(Debug, Clone)]
struct InstrDoc {
    page_index: u32,
    provider_relative_url: String,
}

fn parse_instr_parent(filename: &str) -> Option<(String, u32)> {
    let cap = INSTR_FILE_RE.captures(filename)?;
    let base = cap.name("base")?.as_str().to_uppercase();
    let page_index = cap
        .name("page")
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .unwrap_or(0);
    Some((base, page_index))
}

fn filename_stem_upper(filename: &str) -> String {
    filename
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF")
        .to_uppercase()
}

fn strip_numeric_suffix(stem: &str) -> String {
    static NUM_SUFFIX_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?P<base>.+)_\d{2}$").unwrap());
    NUM_SUFFIX_RE
        .captures(stem)
        .and_then(|c| c.name("base").map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| stem.to_string())
}

fn normalize_stem_for_instr_match(stem: &str) -> String {
    RWY_GROUP_RE.replace(stem, "_RWY").to_string()
}

fn parse_stem_runway_parts(stem: &str) -> Option<(String, Vec<String>, String)> {
    let cap = STEM_RWY_PARTS_RE.captures(stem)?;
    let prefix = cap.name("prefix")?.as_str().to_string();
    let suffix = cap.name("suffix")?.as_str().to_string();
    let runways = cap
        .name("group")?
        .as_str()
        .split('-')
        .map(|s| s.to_uppercase())
        .collect::<Vec<_>>();
    Some((prefix, runways, suffix))
}

fn stems_match_for_instr_link(chart_stem: &str, instr_parent_stem: &str) -> bool {
    if chart_stem == instr_parent_stem {
        return true;
    }

    let chart_norm = normalize_stem_for_instr_match(chart_stem);
    let instr_norm = normalize_stem_for_instr_match(instr_parent_stem);
    if chart_norm != instr_norm {
        return false;
    }

    let Some((chart_prefix, chart_runways, chart_suffix)) = parse_stem_runway_parts(chart_stem) else {
        return false;
    };
    let Some((instr_prefix, instr_runways, instr_suffix)) = parse_stem_runway_parts(instr_parent_stem) else {
        return false;
    };

    if chart_prefix != instr_prefix || chart_suffix != instr_suffix {
        return false;
    }

    if chart_runways.iter().any(|r| r == "ALL") {
        return true;
    }

    chart_runways
        .iter()
        .any(|rwy| instr_runways.iter().any(|k| k == rwy))
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
            linked_provider_relative_urls: rc.linked_provider_relative_urls.clone(),
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
        assert!(INSTR_FILE_RE.is_match("AD_2_LFPG_INSTR_01.pdf"));
        assert!(!INSTR_FILE_RE.is_match("AD_2_LFPG_IAC_01.pdf"));
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

    #[test]
    fn test_instr_attached_when_main_has_no_numeric_suffix() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFXX_IAC_RWY_27L.pdf">Main</a>
                <a href="./AD_2_LFXX_IAC_RWY_27L_01_INSTR_01.pdf">Instr 1</a>
                <a href="./AD_2_LFXX_IAC_RWY_27L_01_INSTR_02.pdf">Instr 2</a>
            </body></html>
        "#;

        let charts = parse_sia_html(html, "LFXX", "2602");
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].filename, "AD_2_LFXX_IAC_RWY_27L.pdf");
        assert_eq!(charts[0].linked_provider_relative_urls.len(), 2);
        assert_eq!(
            charts[0].linked_provider_relative_urls[0],
            "AD_2_LFXX_IAC_RWY_27L_01_INSTR_01.pdf"
        );
        assert_eq!(
            charts[0].linked_provider_relative_urls[1],
            "AD_2_LFXX_IAC_RWY_27L_01_INSTR_02.pdf"
        );
    }

    #[test]
    fn test_instr_attached_when_main_has_numeric_suffix() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFYY_IAC_RWY_09_01.pdf">Main</a>
                <a href="./AD_2_LFYY_IAC_RWY_09_INSTR_01.pdf">Instr 1</a>
            </body></html>
        "#;

        let charts = parse_sia_html(html, "LFYY", "2602");
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].filename, "AD_2_LFYY_IAC_RWY_09_01.pdf");
        assert_eq!(charts[0].linked_provider_relative_urls.len(), 1);
        assert_eq!(
            charts[0].linked_provider_relative_urls[0],
            "AD_2_LFYY_IAC_RWY_09_INSTR_01.pdf"
        );
    }

    #[test]
    fn test_instr_attached_for_combined_runway_chart() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFPO_SID_RWY06-07_RNAV_SOUTH.pdf">Main</a>
                <a href="./AD_2_LFPO_SID_RWY06_RNAV_SOUTH_INSTR_01.pdf">Instr 06</a>
                <a href="./AD_2_LFPO_SID_RWY07_RNAV_SOUTH_INSTR_02.pdf">Instr 07</a>
            </body></html>
        "#;

        let charts = parse_sia_html(html, "LFPO", "2602");
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].linked_provider_relative_urls.len(), 2);
        assert_eq!(
            charts[0].linked_provider_relative_urls[0],
            "AD_2_LFPO_SID_RWY06_RNAV_SOUTH_INSTR_01.pdf"
        );
        assert_eq!(
            charts[0].linked_provider_relative_urls[1],
            "AD_2_LFPO_SID_RWY07_RNAV_SOUTH_INSTR_02.pdf"
        );
    }

    #[test]
    fn test_instr_not_attached_from_unrelated_runway_group() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFPO_SID_RWY06-07_RNAV_SOUTH.pdf">Main</a>
                <a href="./AD_2_LFPO_SID_RWY06_RNAV_SOUTH_INSTR_01.pdf">Instr 06</a>
                <a href="./AD_2_LFPO_SID_RWY25_RNAV_SOUTH_INSTR_01.pdf">Instr 25</a>
            </body></html>
        "#;

        let charts = parse_sia_html(html, "LFPO", "2602");
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].linked_provider_relative_urls.len(), 1);
        assert_eq!(
            charts[0].linked_provider_relative_urls[0],
            "AD_2_LFPO_SID_RWY06_RNAV_SOUTH_INSTR_01.pdf"
        );
    }

    #[test]
    fn test_instr_attached_to_sid_all() {
        let html = r#"
            <html><body>
                <a href="./AD_2_LFBL_SID_RWY_ALL_RNAV.pdf">SID ALL</a>
                <a href="./AD_2_LFBL_SID_RWY03_RNAV_INSTR_01.pdf">INSTR 03</a>
                <a href="./AD_2_LFBL_SID_RWY21_RNAV_INSTR_01.pdf">INSTR 21</a>
            </body></html>
        "#;

        let charts = parse_sia_html(html, "LFBL", "2602");
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].filename, "AD_2_LFBL_SID_RWY_ALL_RNAV.pdf");
        assert_eq!(charts[0].linked_provider_relative_urls.len(), 2);
        assert_eq!(
            charts[0].linked_provider_relative_urls[0],
            "AD_2_LFBL_SID_RWY03_RNAV_INSTR_01.pdf"
        );
        assert_eq!(
            charts[0].linked_provider_relative_urls[1],
            "AD_2_LFBL_SID_RWY21_RNAV_INSTR_01.pdf"
        );
    }
}
