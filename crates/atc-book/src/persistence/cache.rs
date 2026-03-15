use rusqlite::{params, Connection};

use crate::models::{Chart, Notice};

/// Retrieve cached search results for the given ICAO + AIRAC cycle.
/// Returns `None` if no cache entry exists.
pub fn get_cached_search(
    conn: &Connection,
    icao: &str,
    airac_code: &str,
) -> Option<(Vec<Chart>, Vec<Notice>)> {
    let mut stmt = conn
        .prepare(
            "SELECT charts_json, notices_json FROM chart_cache WHERE icao = ?1 AND airac_code = ?2",
        )
        .ok()?;
    stmt.query_row(params![icao, airac_code], |row| {
        let charts_json: String = row.get(0)?;
        let notices_json: String = row.get(1)?;
        Ok((charts_json, notices_json))
    })
    .ok()
    .and_then(|(cj, nj)| {
        let charts: Vec<Chart> = serde_json::from_str(&cj).ok()?;
        let notices: Vec<Notice> = serde_json::from_str(&nj).ok()?;
        Some((charts, notices))
    })
}

/// Store search results in the cache, replacing any existing entry.
pub fn put_cached_search(
    conn: &Connection,
    icao: &str,
    airac_code: &str,
    charts: &[Chart],
    notices: &[Notice],
) {
    let charts_json = serde_json::to_string(charts).unwrap_or_default();
    let notices_json = serde_json::to_string(notices).unwrap_or_default();
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO chart_cache (icao, airac_code, charts_json, notices_json, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![icao, airac_code, charts_json, notices_json, now],
    );
}

/// Record a cached PDF file's metadata.
pub fn put_pdf_entry(conn: &Connection, url: &str, local_path: &str, size_bytes: u64) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO pdf_cache (url, local_path, fetched_at, size_bytes)
         VALUES (?1, ?2, ?3, ?4)",
        params![url, local_path, now, size_bytes as i64],
    );
}

pub fn put_html_doc(conn: &Connection, url: &str, html: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO html_doc_cache (url, html, fetched_at) VALUES (?1, ?2, ?3)",
        params![url, html, now],
    );
}

pub fn get_html_doc(conn: &Connection, url: &str) -> Option<String> {
    conn.query_row(
        "SELECT html FROM html_doc_cache WHERE url = ?1",
        params![url],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

/// Look up a cached PDF file by URL.
/// Returns the local file path if the entry exists **and** the file still exists on disk.
pub fn get_pdf_path(conn: &Connection, url: &str) -> Option<String> {
    let mut stmt = conn
        .prepare("SELECT local_path FROM pdf_cache WHERE url = ?1")
        .ok()?;
    let path: String = stmt.query_row(params![url], |row| row.get(0)).ok()?;
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        // Stale entry — file was deleted. Remove from DB.
        let _ = conn.execute("DELETE FROM pdf_cache WHERE url = ?1", params![url]);
        None
    }
}

/// Store a rendered page PNG for a given PDF URL.
pub fn put_rendered_page(conn: &Connection, url: &str, page_index: usize, local_path: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO rendered_pdf_cache (url, page_index, local_path, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![url, page_index as i64, local_path, now],
    );
}

/// Load all rendered page paths for a given PDF URL.
/// Returns None if any file is missing.
pub fn get_rendered_pages(conn: &Connection, url: &str) -> Option<Vec<(usize, String)>> {
    let mut stmt = conn
        .prepare(
            "SELECT page_index, local_path FROM rendered_pdf_cache WHERE url = ?1 ORDER BY page_index",
        )
        .ok()?;
    let rows: Vec<(i64, String)> = stmt
        .query_map(params![url], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok()?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(rows.len());
    for (idx, path) in rows {
        if !std::path::Path::new(&path).exists() {
            let _ = conn.execute(
                "DELETE FROM rendered_pdf_cache WHERE url = ?1",
                params![url],
            );
            return None;
        }
        out.push((idx as usize, path));
    }
    Some(out)
}

/// Prune rendered PNG cache to keep at most `max_pages` entries.
pub fn prune_rendered_cache(conn: &Connection, max_pages: usize) {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM rendered_pdf_cache", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    let max_pages = max_pages as i64;
    if count <= max_pages {
        return;
    }
    let to_delete = count - max_pages;
    let mut stmt = match conn.prepare(
        "SELECT url, page_index, local_path FROM rendered_pdf_cache ORDER BY created_at ASC LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return,
    };
    let rows: Vec<(String, i64, String)> = stmt
        .query_map(params![to_delete], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .ok()
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    for (url, page_index, path) in rows {
        let _ = std::fs::remove_file(&path);
        let _ = conn.execute(
            "DELETE FROM rendered_pdf_cache WHERE url = ?1 AND page_index = ?2",
            params![url, page_index],
        );
    }
}

/// Clear cached PDF files and rendered page images from disk and database.
pub fn clear_file_caches(conn: &Connection) {
    let pdf_dir = crate::persistence::pdf_cache_dir();
    let rendered_dir = crate::persistence::rendered_cache_dir();

    let _ = std::fs::read_dir(&pdf_dir).map(|entries| {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    });

    let _ = std::fs::read_dir(&rendered_dir).map(|entries| {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    });

    let _ = conn.execute("DELETE FROM pdf_cache", []);
    let _ = conn.execute("DELETE FROM rendered_pdf_cache", []);
    let _ = conn.execute("DELETE FROM html_doc_cache", []);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::persistence::test_db;

    #[test]
    fn test_chart_cache_roundtrip() {
        let conn = test_db();
        let charts = vec![Chart {
            id: "c1".into(),
            source: ChartSource::Sia,
            category: ChartCategory::Aerodrome,
            subtitle: "ADC".into(),
            filename: "test.pdf".into(),
            provider_relative_url: "charts/test.pdf".into(),
            airac_code: "2602".into(),
            page: None,
            tags: vec!["ILS".into()],
            runways: vec!["09L".into()],
            custom_title: None,
        }];
        let notices = vec![];

        assert!(get_cached_search(&conn, "LFPG", "2602").is_none());

        put_cached_search(&conn, "LFPG", "2602", &charts, &notices);

        let (cached_charts, cached_notices) =
            get_cached_search(&conn, "LFPG", "2602").expect("should be cached");
        assert_eq!(cached_charts.len(), 1);
        assert_eq!(cached_charts[0].id, "c1");
        assert!(cached_notices.is_empty());
    }

    #[test]
    fn test_chart_cache_different_airac() {
        let conn = test_db();
        put_cached_search(&conn, "LFPG", "2602", &[], &[]);
        assert!(get_cached_search(&conn, "LFPG", "2603").is_none());
    }

    #[test]
    fn test_pdf_cache_roundtrip() {
        let conn = test_db();
        assert!(get_pdf_path(&conn, "https://example.com/chart.pdf").is_none());

        // Use a path that doesn't exist on disk — should return None even after insert
        put_pdf_entry(
            &conn,
            "https://example.com/chart.pdf",
            "/tmp/nonexistent_vaccfr_test.pdf",
            1024,
        );
        assert!(get_pdf_path(&conn, "https://example.com/chart.pdf").is_none());
    }
}
