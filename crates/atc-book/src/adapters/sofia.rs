use reqwest::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use serde_json::Value;

use crate::models::{Notice, NoticeSource};

/// Fetch NOTAMs from SOFIA for a given ICAO code.
pub async fn fetch_notices(icao: &str) -> Result<Vec<Notice>, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    // Step 1: HEAD to get JSESSIONID
    let sofia_url = "https://sofia-briefing.aviation-civile.gouv.fr/sofia/pages/homepage.html";
    let resp = client
        .head(sofia_url)
        .send()
        .await
        .map_err(|e| format!("SOFIA HEAD failed: {e}"))?;

    let session_cookie = resp
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|s| s.contains("JSESSIONID"))
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .ok_or_else(|| "JSESSIONID not found".to_string())?;

    // Step 2: POST PIB query
    let query_url = "https://sofia-briefing.aviation-civile.gouv.fr/sofia/api/notam";
    let body = format!(
        "operation=search&typeTraffic=VI&flLower=0&flUpper=999&radius=25&locations={}",
        icao.to_uppercase()
    );

    let resp = client
        .post(query_url)
        .header(COOKIE, &session_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("SOFIA POST failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("SOFIA HTTP {}", resp.status()));
    }

    let json: Value = resp
        .json()
        .await
        .map_err(|e| format!("SOFIA JSON parse error: {e}"))?;

    // Recursively find NOTAM objects
    let mut notices = Vec::new();
    extract_notams(&json, icao, &mut notices);

    // Deduplicate by id
    notices.sort_by(|a, b| a.id.cmp(&b.id));
    notices.dedup_by(|a, b| a.id == b.id);

    // Sort by valid_from descending
    notices.sort_by(|a, b| b.valid_from.cmp(&a.valid_from));

    Ok(notices)
}

/// Recursively walk JSON to find NOTAM objects.
fn extract_notams(value: &Value, icao: &str, out: &mut Vec<Notice>) {
    match value {
        Value::Object(map) => {
            // Check if this looks like a NOTAM object (has "id" and "itemE")
            if let (Some(id_val), Some(item_e)) = (map.get("id"), map.get("itemE")) {
                let id = id_val.as_str().unwrap_or("").to_string();
                if !id.is_empty() {
                    let serie = map.get("serie").and_then(|v| v.as_str()).unwrap_or("");
                    let number = map.get("number").and_then(|v| v.as_str()).unwrap_or("");
                    let year = map.get("year").and_then(|v| v.as_str()).unwrap_or("");

                    let identifier = if !serie.is_empty() {
                        format!("{}{}/{}", serie, number, year)
                    } else {
                        id.clone()
                    };

                    let notice_type = map
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("NOTAM")
                        .to_string();

                    let valid_from = map
                        .get("startValidity")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let valid_to = map
                        .get("endValidity")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let content = item_e.as_str().unwrap_or("").to_string();

                    let category = map
                        .get("qLine")
                        .and_then(|q| q.get("code23"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    out.push(Notice {
                        id,
                        icao: icao.to_uppercase(),
                        source: NoticeSource::Sofia,
                        identifier,
                        notice_type,
                        valid_from,
                        valid_to,
                        content,
                        category,
                    });
                }
            }
            // Recurse into all values
            for v in map.values() {
                extract_notams(v, icao, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                extract_notams(v, icao, out);
            }
        }
        _ => {}
    }
}
