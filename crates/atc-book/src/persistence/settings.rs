use rusqlite::{params, Connection};

use crate::i18n::AppLanguage;
use crate::state::ThemeMode;

const KEY_LANGUAGE: &str = "language";
const KEY_THEME_MODE: &str = "theme_mode";

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) {
    let _ = conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    );
}

pub fn load_language(conn: &Connection) -> Option<AppLanguage> {
    match get_setting(conn, KEY_LANGUAGE)?.as_str() {
        "fr" => Some(AppLanguage::Fr),
        "uk" => Some(AppLanguage::Uk),
        _ => None,
    }
}

pub fn save_language(conn: &Connection, lang: AppLanguage) {
    let value = match lang {
        AppLanguage::Fr => "fr",
        AppLanguage::Uk => "uk",
    };
    set_setting(conn, KEY_LANGUAGE, value);
}

pub fn load_theme_mode(conn: &Connection) -> Option<ThemeMode> {
    match get_setting(conn, KEY_THEME_MODE)?.as_str() {
        "light" => Some(ThemeMode::Light),
        "dark" => Some(ThemeMode::Dark),
        "auto_time" => Some(ThemeMode::AutoTime),
        "auto_system" => Some(ThemeMode::AutoSystem),
        _ => None,
    }
}

pub fn save_theme_mode(conn: &Connection, mode: ThemeMode) {
    let value = match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::AutoTime => "auto_time",
        ThemeMode::AutoSystem => "auto_system",
    };
    set_setting(conn, KEY_THEME_MODE, value);
}
