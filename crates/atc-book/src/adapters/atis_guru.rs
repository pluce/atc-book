use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const ATIS_CACHE_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone)]
struct CachedAtisEntry {
    fetched_at: Instant,
    data: AtisData,
}

static ATIS_CACHE: OnceLock<Mutex<HashMap<String, CachedAtisEntry>>> = OnceLock::new();

fn atis_cache() -> &'static Mutex<HashMap<String, CachedAtisEntry>> {
    ATIS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtisMessage {
    pub title: String,
    pub timestamp: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AtisData {
    pub icao: String,
    pub arrival: Option<AtisMessage>,
    pub departure: Option<AtisMessage>,
    pub metar: Option<String>,
    pub taf: Option<String>,
}

pub async fn fetch_atis(icao: &str) -> Result<AtisData, Box<dyn std::error::Error>> {
    fetch_atis_with_cache_policy(icao, false).await
}

pub async fn fetch_atis_with_cache_policy(
    icao: &str,
    force_refresh: bool,
) -> Result<AtisData, Box<dyn std::error::Error>> {
    let key = icao.trim().to_uppercase();

    if !force_refresh {
        let cached = {
            let cache = atis_cache().lock().unwrap();
            cache.get(&key).cloned()
        };

        if let Some(entry) = cached {
            if entry.fetched_at.elapsed() < ATIS_CACHE_TTL {
                return Ok(entry.data);
            }
        }
    }

    let fresh = fetch_atis_network(&key).await?;
    {
        let mut cache = atis_cache().lock().unwrap();
        cache.insert(
            key,
            CachedAtisEntry {
                fetched_at: Instant::now(),
                data: fresh.clone(),
            },
        );
    }
    Ok(fresh)
}

async fn fetch_atis_network(icao: &str) -> Result<AtisData, Box<dyn std::error::Error>> {
    let url = format!("https://atis.guru/atis/{}", icao);
    let resp = reqwest::get(&url).await?.text().await?;
    let document = Html::parse_document(&resp);

    let mut data = AtisData {
        icao: icao.to_string(),
        ..Default::default()
    };

    let card_selector = Selector::parse(".card-body").unwrap();
    let title_selector = Selector::parse(".card-title").unwrap();
    let subtitle_selector = Selector::parse(".card-subtitle").unwrap();
    let atis_content_selector = Selector::parse(".atis").unwrap();

    for card in document.select(&card_selector) {
        let title = card
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let content = card
            .select(&atis_content_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.contains("Arrival ATIS") {
            let timestamp = card
                .select(&subtitle_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            data.arrival = Some(AtisMessage {
                title: "Arrival ATIS".to_string(),
                timestamp,
                content,
            });
        } else if title.contains("Departure ATIS") {
            let timestamp = card
                .select(&subtitle_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            data.departure = Some(AtisMessage {
                title: "Departure ATIS".to_string(),
                timestamp,
                content,
            });
        } else if title.contains("Combined ATIS") {
             let timestamp = card
                .select(&subtitle_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            // Treat combined as Arrival for simplicity, or handle separate field
            data.arrival = Some(AtisMessage {
                title: "Combined ATIS".to_string(),
                timestamp,
                content,
            });
        } else if title.contains("METAR") {
            data.metar = Some(content);
        } else if title.contains("TAF") {
            data.taf = Some(content);
        }
    }

    Ok(data)
}
