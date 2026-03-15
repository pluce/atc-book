use std::collections::HashMap;

use base64::Engine;
use regex::Regex;

use crate::models::AipDocument;

fn log_info(message: impl AsRef<str>) {
    println!("[aip-doc] {}", message.as_ref());
}

pub async fn load_doc_html(doc: &AipDocument) -> Result<String, String> {
    let url = doc.runtime_url();
    log_info(format!("load_doc_html start icao={} url={}", doc.icao, url));

    {
        let conn = crate::persistence::db().lock().unwrap();
        if let Some(html) = crate::persistence::cache::get_html_doc(&conn, &url) {
            log_info(format!(
                "html cache hit url={} bytes={}",
                url,
                html.len()
            ));
            return Ok(html);
        }
    }

    log_info(format!("html cache miss url={}", url));

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTML request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTML status error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("HTML read failed: {e}"))?;

    log_info(format!("html fetched url={} bytes={}", url, html.len()));

    let inlined = inline_images(&client, &url, &html).await;
    let inlined = inline_stylesheets(&client, &url, &inlined).await;
    let inlined = scope_embedded_styles(&inlined);
    log_info(format!("html inlined url={} bytes={}", url, inlined.len()));

    let conn = crate::persistence::db().lock().unwrap();
    crate::persistence::cache::put_html_doc(&conn, &url, &inlined);
    log_info(format!("html cached url={}", url));

    Ok(inlined)
}

fn scope_embedded_styles(html: &str) -> String {
    let style_re = Regex::new(r#"(?is)<style([^>]*)>(.*?)</style>"#).unwrap();
    let mut out = html.to_string();
    for caps in style_re.captures_iter(html) {
        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let css = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let scoped = scope_css(css, ".aip-doc-content");
        let replacement = format!("<style{}>{}</style>", attrs, scoped);
        out = out.replace(full, &replacement);
    }
    out
}

fn scope_css(css: &str, scope: &str) -> String {
    let mut out = String::new();
    let bytes = css.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let open = match css[i..].find('{') {
            Some(pos) => i + pos,
            None => {
                out.push_str(&css[i..]);
                break;
            }
        };

        let prelude = css[i..open].trim();
        let mut depth = 1usize;
        let mut j = open + 1;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            out.push_str(&css[i..]);
            break;
        }

        let body = &css[open + 1..j - 1];

        if prelude.starts_with('@') {
            if prelude.starts_with("@media")
                || prelude.starts_with("@supports")
                || prelude.starts_with("@document")
                || prelude.starts_with("@layer")
            {
                out.push_str(prelude);
                out.push('{');
                out.push_str(&scope_css(body, scope));
                out.push('}');
            } else {
                out.push_str(prelude);
                out.push('{');
                out.push_str(body);
                out.push('}');
            }
        } else {
            let selectors = prelude
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|sel| scope_selector(sel, scope))
                .collect::<Vec<_>>()
                .join(", ");

            out.push_str(&selectors);
            out.push('{');
            out.push_str(body);
            out.push('}');
        }

        i = j;
    }

    out
}

fn scope_selector(sel: &str, scope: &str) -> String {
    if sel.starts_with(scope) {
        return sel.to_string();
    }

    let mut normalized = sel.trim().to_string();
    for root in ["html", "body", ":root"] {
        if normalized == root {
            return scope.to_string();
        }
        if let Some(rest) = normalized.strip_prefix(&format!("{} ", root)) {
            normalized = rest.trim().to_string();
            break;
        }
        if let Some(rest) = normalized.strip_prefix(&format!("{}>", root)) {
            normalized = format!(">{}", rest.trim());
            break;
        }
    }

    if normalized == "*" {
        scope.to_string()
    } else {
        format!("{} {}", scope, normalized)
    }
}

async fn inline_stylesheets(client: &reqwest::Client, base_url: &str, html: &str) -> String {
    let link_re = Regex::new(r#"(?i)<link([^>]*?rel=[\"']stylesheet[\"'][^>]*?)>"#).unwrap();
    let href_re = Regex::new(r#"(?i)href=[\"']([^\"']+)[\"']"#).unwrap();

    let mut out = html.to_string();
    for caps in link_re.captures_iter(html) {
        let full_tag = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let href = match href_re.captures(attrs) {
            Some(c) => c.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
            None => continue,
        };
        if href.is_empty() || href.starts_with("data:") {
            continue;
        }
        let abs = absolute_url(base_url, &href);
        let css = match client.get(&abs).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(r) => match r.text().await {
                    Ok(t) => t,
                    Err(_) => continue,
                },
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let style_tag = format!("<style>/* inlined: {} */\n{}</style>", abs, css);
        out = out.replace(full_tag, &style_tag);
    }
    out
}

async fn inline_images(client: &reqwest::Client, base_url: &str, html: &str) -> String {
    let img_re = Regex::new(r#"(?i)<img([^>]*?)src=[\"']([^\"']+)[\"']([^>]*)>"#).unwrap();
    let mut cache: HashMap<String, String> = HashMap::new();

    let mut out = html.to_string();
    for caps in img_re.captures_iter(html) {
        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let src = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        if src.is_empty() || src.starts_with("data:") {
            continue;
        }

        let abs = absolute_url(base_url, &src);
        let data_uri = if let Some(v) = cache.get(&abs) {
            v.clone()
        } else {
            let uri = match fetch_data_uri(client, &abs).await {
                Ok(u) => u,
                Err(_) => continue,
            };
            cache.insert(abs.clone(), uri.clone());
            uri
        };

        let replaced = full.replacen(&src, &data_uri, 1);
        out = out.replace(full, &replaced);
    }

    out
}

fn absolute_url(base: &str, src: &str) -> String {
    if src.starts_with("http://") || src.starts_with("https://") {
        return src.to_string();
    }
    if src.starts_with("//") {
        return format!("https:{}", src);
    }

    let base_dir = base
        .rsplit_once('/')
        .map(|(b, _)| b)
        .unwrap_or(base)
        .trim_end_matches('/');

    if src.starts_with('/') {
        let origin = base
            .split_once("//")
            .and_then(|(scheme, rem)| {
                rem.split_once('/')
                    .map(|(host, _)| format!("{}//{}", scheme, host))
            })
            .unwrap_or_else(|| base_dir.to_string());
        return format!("{}{}", origin, src);
    }

    format!("{}/{}", base_dir, src.trim_start_matches("./"))
}

async fn fetch_data_uri(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("img request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("img status failed: {e}"))?;

    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("image/png")
        .split(';')
        .next()
        .unwrap_or("image/png")
        .to_string();

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("img read failed: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

