pub mod cache;
pub mod settings;
pub mod workspaces;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use rusqlite::Connection;

/// Global database connection (single-writer SQLite).
static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

/// Return the application data directory, creating it if needed.
pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("atc-book");
    std::fs::create_dir_all(&dir).expect("cannot create data directory");
    dir
}

/// Return the PDF cache directory, creating it if needed.
pub fn pdf_cache_dir() -> PathBuf {
    let dir = data_dir().join("pdfs");
    std::fs::create_dir_all(&dir).expect("cannot create pdf cache directory");
    dir
}

/// Return the rendered PNG cache directory, creating it if needed.
pub fn rendered_cache_dir() -> PathBuf {
    let dir = data_dir().join("rendered");
    std::fs::create_dir_all(&dir).expect("cannot create rendered cache directory");
    dir
}

/// Get a reference to the global database mutex.
/// Initialises the database on first call.
pub fn db() -> &'static Mutex<Connection> {
    DB.get_or_init(|| {
        let path = data_dir().join("atc-book.db");
        let conn = Connection::open(&path).expect("cannot open database");
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .expect("cannot set pragmas");
        migrate(&conn);
        Mutex::new(conn)
    })
}

fn migrate(conn: &Connection) {
    conn.execute_batch(
        "
        -- Workspaces (controller positions / sessions)
        CREATE TABLE IF NOT EXISTS workspaces (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            airports    TEXT NOT NULL DEFAULT '[]',
            notes       TEXT,
            notes_pinned INTEGER DEFAULT 0,
            notes_panel_width INTEGER DEFAULT 380,
            open_tabs   TEXT NOT NULL DEFAULT '[]',
            active_tab  INTEGER,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workspace_charts (
            workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            airport      TEXT NOT NULL,
            position     INTEGER NOT NULL,
            chart_json   TEXT NOT NULL,
            PRIMARY KEY (workspace_id, airport, position)
        );

        CREATE TABLE IF NOT EXISTS workspace_chart_zoom (
            workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            chart_id     TEXT NOT NULL,
            zoom         INTEGER NOT NULL,
            PRIMARY KEY (workspace_id, chart_id)
        );

        CREATE TABLE IF NOT EXISTS workspace_popout_tabs (
            workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
            tab_ids_json TEXT NOT NULL DEFAULT '[]',
            active_tab   INTEGER,
            updated_at   TEXT NOT NULL
        );

        -- Caches
        CREATE TABLE IF NOT EXISTS chart_cache (
            icao        TEXT NOT NULL,
            airac_code  TEXT NOT NULL,
            charts_json TEXT NOT NULL,
            notices_json TEXT NOT NULL,
            fetched_at  TEXT NOT NULL,
            PRIMARY KEY (icao, airac_code)
        );

        CREATE TABLE IF NOT EXISTS pdf_cache (
            url         TEXT PRIMARY KEY,
            local_path  TEXT NOT NULL,
            fetched_at  TEXT NOT NULL,
            size_bytes  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS rendered_pdf_cache (
            url         TEXT NOT NULL,
            page_index  INTEGER NOT NULL,
            local_path  TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            PRIMARY KEY (url, page_index)
        );

        CREATE TABLE IF NOT EXISTS html_doc_cache (
            url         TEXT PRIMARY KEY,
            html        TEXT NOT NULL,
            fetched_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )
    .expect("database migration failed");

    // Add missing column for older databases
    let _ = conn.execute(
        "ALTER TABLE workspaces ADD COLUMN notes_panel_width INTEGER DEFAULT 380",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE workspaces ADD COLUMN notes_pinned INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE workspaces ADD COLUMN extra_tabs TEXT NOT NULL DEFAULT '[]'",
        [],
    );
}

#[cfg(test)]
pub(crate) fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("cannot open in-memory db");
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    migrate(&conn);
    conn
}
