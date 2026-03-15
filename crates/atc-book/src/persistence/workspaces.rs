use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::models::{Chart, ExtraTab, Workspace, WorkspaceChart};

/// List all workspaces, ordered by most recently updated first.
pub fn list_workspaces(conn: &Connection) -> Vec<Workspace> {
    let mut stmt = match conn.prepare(
        "SELECT id, name, airports, notes, notes_pinned, notes_panel_width, open_tabs, active_tab, created_at, updated_at, extra_tabs
         FROM workspaces ORDER BY updated_at DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })
        .ok()
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();

    rows.into_iter()
        .map(
            |(
                id,
                name,
                airports_json,
                notes,
                notes_pinned,
                notes_panel_width,
                tabs_json,
                active_tab,
                created,
                updated,
                extra_tabs_json,
            )| {
                let airports: Vec<String> =
                    serde_json::from_str(&airports_json).unwrap_or_default();
                let open_tabs: Vec<String> = serde_json::from_str(&tabs_json).unwrap_or_default();
                let extra_tabs: Vec<ExtraTab> = extra_tabs_json
                    .as_deref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();
                let chart_refs = load_workspace_charts(conn, &id);
                Workspace {
                    id,
                    name,
                    airports,
                    chart_refs,
                    open_tabs,
                    active_tab_index: active_tab.map(|i| i as usize),
                    extra_tabs,
                    notes,
                    notes_pinned: notes_pinned.map(|v| v != 0),
                    notes_panel_width: notes_panel_width.map(|v| v as i32),
                    created_at: created,
                    updated_at: updated,
                }
            },
        )
        .collect()
}

/// Create a new workspace.
pub fn create_workspace(conn: &Connection, ws: &Workspace) {
    let now = chrono::Utc::now().to_rfc3339();
    let airports_json = serde_json::to_string(&ws.airports).unwrap_or_default();
    let tabs_json = serde_json::to_string(&ws.open_tabs).unwrap_or_default();
    let extra_tabs_json = serde_json::to_string(&ws.extra_tabs).unwrap_or_else(|_| "[]".to_string());
    let _ = conn.execute(
        "INSERT INTO workspaces (id, name, airports, notes, notes_pinned, notes_panel_width, open_tabs, active_tab, extra_tabs, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            ws.id,
            ws.name,
            airports_json,
            ws.notes,
            ws.notes_pinned.map(|v| if v { 1 } else { 0 }),
            ws.notes_panel_width.map(|v| v as i64),
            tabs_json,
            ws.active_tab_index.map(|i| i as i64),
            extra_tabs_json,
            now,
            now
        ],
    );
    save_workspace_charts(conn, &ws.id, &ws.chart_refs);
}

/// Update workspace metadata, charts, and tab state.
#[allow(dead_code)]
pub fn update_workspace(conn: &Connection, ws: &Workspace) {
    let now = chrono::Utc::now().to_rfc3339();
    let airports_json = serde_json::to_string(&ws.airports).unwrap_or_default();
    let tabs_json = serde_json::to_string(&ws.open_tabs).unwrap_or_default();
    let _ = conn.execute(
        "UPDATE workspaces SET name = ?1, airports = ?2, notes = ?3,
         notes_pinned = ?4, notes_panel_width = ?5, open_tabs = ?6, active_tab = ?7, updated_at = ?8
         WHERE id = ?9",
        params![
            ws.name,
            airports_json,
            ws.notes,
            ws.notes_pinned.map(|v| if v { 1 } else { 0 }),
            ws.notes_panel_width.map(|v| v as i64),
            tabs_json,
            ws.active_tab_index.map(|i| i as i64),
            now,
            ws.id
        ],
    );
    save_workspace_charts(conn, &ws.id, &ws.chart_refs);
}

/// Delete a workspace and all associated data.
pub fn delete_workspace(conn: &Connection, workspace_id: &str) {
    let _ = conn.execute(
        "DELETE FROM workspaces WHERE id = ?1",
        params![workspace_id],
    );
}

/// Rename a workspace.
pub fn rename_workspace(conn: &Connection, workspace_id: &str, new_name: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_name, now, workspace_id],
    );
}

/// Remove a specific chart from a workspace.
pub fn remove_chart_from_workspace(
    conn: &Connection,
    workspace_id: &str,
    airport: &str,
    chart_id: &str,
) {
    // Load current charts, remove matching one, re-save with correct positions
    let mut charts = load_workspace_charts(conn, workspace_id);
    charts.retain(|wc| !(wc.airport == airport && wc.chart.id == chart_id));
    save_workspace_charts(conn, workspace_id, &charts);

    // Remove airport from workspace if no more charts for it
    let has_airport = charts.iter().any(|wc| wc.airport == airport);
    if !has_airport {
        let mut ws_airports: Vec<String> = conn
            .query_row(
                "SELECT airports FROM workspaces WHERE id = ?1",
                params![workspace_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        ws_airports.retain(|a| a != airport);
        let json = serde_json::to_string(&ws_airports).unwrap_or_default();
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE workspaces SET airports = ?1, updated_at = ?2 WHERE id = ?3",
            params![json, now, workspace_id],
        );
    } else {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE workspaces SET updated_at = ?1 WHERE id = ?2",
            params![now, workspace_id],
        );
    }
}

/// Add a chart to a workspace under a given airport.
/// Does nothing if the chart is already present for that airport.
pub fn add_chart_to_workspace(conn: &Connection, workspace_id: &str, airport: &str, chart: &Chart) {
    // Dedup check: skip if chart already exists in this workspace+airport
    let existing = load_workspace_charts(conn, workspace_id);
    if existing
        .iter()
        .any(|wc| wc.airport == airport && wc.chart.id == chart.id)
    {
        return;
    }
    let next_pos: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM workspace_charts
             WHERE workspace_id = ?1 AND airport = ?2",
            params![workspace_id, airport],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let chart_json = serde_json::to_string(chart).unwrap_or_default();
    let _ = conn.execute(
        "INSERT INTO workspace_charts (workspace_id, airport, position, chart_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![workspace_id, airport, next_pos, chart_json],
    );
    // Add airport to workspace if not present
    let mut ws_airports: Vec<String> = conn
        .query_row(
            "SELECT airports FROM workspaces WHERE id = ?1",
            params![workspace_id],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();
    let airport_upper = airport.to_uppercase();
    if !ws_airports.contains(&airport_upper) {
        ws_airports.push(airport_upper);
        let json = serde_json::to_string(&ws_airports).unwrap_or_default();
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE workspaces SET airports = ?1, updated_at = ?2 WHERE id = ?3",
            params![json, now, workspace_id],
        );
    } else {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE workspaces SET updated_at = ?1 WHERE id = ?2",
            params![now, workspace_id],
        );
    }
}

/// Update the display name for a chart inside a workspace.
pub fn set_chart_custom_title(
    conn: &Connection,
    workspace_id: &str,
    chart_id: &str,
    title: Option<&str>,
) {
    let mut charts = load_workspace_charts(conn, workspace_id);
    let mut updated = false;
    for wc in charts.iter_mut() {
        if wc.chart.id == chart_id {
            wc.chart.custom_title = title
                .map(|t| t.to_string())
                .filter(|t| !t.trim().is_empty());
            updated = true;
        }
    }
    if updated {
        save_workspace_charts(conn, workspace_id, &charts);
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE workspaces SET updated_at = ?1 WHERE id = ?2",
            params![now, workspace_id],
        );
    }
}

/// Save workspace notes (briefing scratchpad).
pub fn save_notes(conn: &Connection, workspace_id: &str, notes: Option<&str>) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET notes = ?1, updated_at = ?2 WHERE id = ?3",
        params![notes, now, workspace_id],
    );
}

/// Save notes panel width for a workspace.
pub fn save_notes_panel_width(conn: &Connection, workspace_id: &str, width: i32) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET notes_panel_width = ?1, updated_at = ?2 WHERE id = ?3",
        params![width as i64, now, workspace_id],
    );
}

/// Save notes pinned state for a workspace.
pub fn save_notes_pinned(conn: &Connection, workspace_id: &str, pinned: bool) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET notes_pinned = ?1, updated_at = ?2 WHERE id = ?3",
        params![if pinned { 1 } else { 0 }, now, workspace_id],
    );
}

/// Save the current tab state to a workspace.
pub fn save_tab_state(
    conn: &Connection,
    workspace_id: &str,
    open_tabs: &[String],
    active_tab: Option<usize>,
    extra_tabs: &[ExtraTab],
) {
    let tabs_json = serde_json::to_string(open_tabs).unwrap_or_default();
    let extra_tabs_json = serde_json::to_string(extra_tabs).unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET open_tabs = ?1, active_tab = ?2, extra_tabs = ?3, updated_at = ?4 WHERE id = ?5",
        params![tabs_json, active_tab.map(|i| i as i64), extra_tabs_json, now, workspace_id],
    );
}

/// Add an ExtraTab (ATIS or AipDoc) to a workspace.
/// No-op if the tab is already present (identified by ICAO / doc.id).
pub fn add_extra_tab(conn: &Connection, workspace_id: &str, tab: &ExtraTab) {
    let current: Vec<ExtraTab> = conn
        .query_row(
            "SELECT COALESCE(extra_tabs, '[]') FROM workspaces WHERE id = ?1",
            params![workspace_id],
            |row| {
                let json: String = row.get(0)?;
                Ok(serde_json::from_str::<Vec<ExtraTab>>(&json).unwrap_or_default())
            },
        )
        .unwrap_or_default();

    let already = match tab {
        ExtraTab::Atis { icao } => current
            .iter()
            .any(|t| matches!(t, ExtraTab::Atis { icao: i } if i == icao)),
        ExtraTab::AipDoc { doc } => current
            .iter()
            .any(|t| matches!(t, ExtraTab::AipDoc { doc: d } if d.id == doc.id)),
    };
    if already {
        return;
    }

    let mut updated = current;
    updated.push(tab.clone());
    let json = serde_json::to_string(&updated).unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET extra_tabs = ?1, updated_at = ?2 WHERE id = ?3",
        params![json, now, workspace_id],
    );
}

/// Remove an ExtraTab from a workspace.
pub fn remove_extra_tab(conn: &Connection, workspace_id: &str, tab: &ExtraTab) {
    let current: Vec<ExtraTab> = conn
        .query_row(
            "SELECT COALESCE(extra_tabs, '[]') FROM workspaces WHERE id = ?1",
            params![workspace_id],
            |row| {
                let json: String = row.get(0)?;
                Ok(serde_json::from_str::<Vec<ExtraTab>>(&json).unwrap_or_default())
            },
        )
        .unwrap_or_default();

    let updated: Vec<ExtraTab> = current
        .into_iter()
        .filter(|t| match (t, tab) {
            (ExtraTab::Atis { icao: a }, ExtraTab::Atis { icao: b }) => a != b,
            (ExtraTab::AipDoc { doc: a }, ExtraTab::AipDoc { doc: b }) => a.id != b.id,
            _ => true,
        })
        .collect();

    let json = serde_json::to_string(&updated).unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE workspaces SET extra_tabs = ?1, updated_at = ?2 WHERE id = ?3",
        params![json, now, workspace_id],
    );
}

/// Load popout tabs for a workspace (window #2).
pub fn load_popout_tab_state(
    conn: &Connection,
    workspace_id: &str,
) -> (Vec<String>, Option<usize>) {
    conn.query_row(
        "SELECT tab_ids_json, active_tab FROM workspace_popout_tabs WHERE workspace_id = ?1",
        params![workspace_id],
        |row| {
            let tabs_json: String = row.get(0)?;
            let active_tab: Option<i64> = row.get(1)?;
            let tabs: Vec<String> = serde_json::from_str(&tabs_json).unwrap_or_default();
            Ok((tabs, active_tab.map(|v| v as usize)))
        },
    )
    .unwrap_or_else(|_| (Vec::new(), None))
}

/// Save popout tabs for a workspace (window #2).
pub fn save_popout_tab_state(
    conn: &Connection,
    workspace_id: &str,
    tab_ids: &[String],
    active_tab: Option<usize>,
) {
    let now = chrono::Utc::now().to_rfc3339();
    let json = serde_json::to_string(tab_ids).unwrap_or_else(|_| "[]".to_string());
    let _ = conn.execute(
        "INSERT INTO workspace_popout_tabs (workspace_id, tab_ids_json, active_tab, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(workspace_id) DO UPDATE
         SET tab_ids_json = excluded.tab_ids_json,
             active_tab = excluded.active_tab,
             updated_at = excluded.updated_at",
        params![workspace_id, json, active_tab.map(|v| v as i64), now],
    );
}

/// Load per-chart zoom levels for a workspace.
pub fn load_chart_zoom(conn: &Connection, workspace_id: &str) -> HashMap<String, u32> {
    let mut stmt = match conn
        .prepare("SELECT chart_id, zoom FROM workspace_chart_zoom WHERE workspace_id = ?1")
    {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    stmt.query_map(params![workspace_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })
    .ok()
    .map(|rows| {
        rows.filter_map(|r| r.ok())
            .map(|(id, zoom)| (id, zoom as u32))
            .collect()
    })
    .unwrap_or_default()
}

/// Persist a zoom level for a given chart inside a workspace.
pub fn save_chart_zoom(conn: &Connection, workspace_id: &str, chart_id: &str, zoom: u32) {
    let _ = conn.execute(
        "INSERT INTO workspace_chart_zoom (workspace_id, chart_id, zoom)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(workspace_id, chart_id) DO UPDATE SET zoom = excluded.zoom",
        params![workspace_id, chart_id, zoom as i64],
    );
}

// -- Internal helpers --

fn load_workspace_charts(conn: &Connection, workspace_id: &str) -> Vec<WorkspaceChart> {
    let mut stmt = match conn.prepare(
        "SELECT airport, chart_json FROM workspace_charts
         WHERE workspace_id = ?1 ORDER BY airport, position",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![workspace_id], |row| {
        let airport: String = row.get(0)?;
        let json: String = row.get(1)?;
        Ok((airport, json))
    })
    .ok()
    .map(|rows| {
        rows.filter_map(|r| r.ok())
            .filter_map(|(airport, j)| {
                serde_json::from_str::<Chart>(&j)
                    .ok()
                    .map(|chart| WorkspaceChart { airport, chart })
            })
            .collect()
    })
    .unwrap_or_default()
}

fn save_workspace_charts(conn: &Connection, workspace_id: &str, charts: &[WorkspaceChart]) {
    let _ = conn.execute(
        "DELETE FROM workspace_charts WHERE workspace_id = ?1",
        params![workspace_id],
    );
    // Group by airport to assign per-airport positions
    let mut by_airport: std::collections::HashMap<&str, Vec<&Chart>> =
        std::collections::HashMap::new();
    for wc in charts {
        by_airport.entry(&wc.airport).or_default().push(&wc.chart);
    }
    for (airport, charts) in &by_airport {
        for (i, chart) in charts.iter().enumerate() {
            let chart_json = serde_json::to_string(chart).unwrap_or_default();
            let _ = conn.execute(
                "INSERT INTO workspace_charts (workspace_id, airport, position, chart_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![workspace_id, *airport, i as i64, chart_json],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::persistence::test_db;

    fn make_chart(id: &str) -> Chart {
        Chart {
            id: id.into(),
            source: ChartSource::Sia,
            category: ChartCategory::Aerodrome,
            subtitle: "ADC".into(),
            filename: "test.pdf".into(),
            provider_relative_url: format!("charts/{id}.pdf"),
            airac_code: "2602".into(),
            page: None,
            tags: vec![],
            runways: vec![],
            custom_title: None,
        }
    }

    fn make_workspace(id: &str, name: &str) -> Workspace {
        Workspace {
            id: id.into(),
            name: name.into(),
            airports: vec![],
            chart_refs: vec![],
            open_tabs: vec![],
            active_tab_index: None,
            notes: None,
            notes_pinned: Some(false),
            notes_panel_width: Some(380),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_create_and_list() {
        let conn = test_db();
        let mut ws = make_workspace("w1", "LFPG APP S");
        ws.airports = vec!["LFPG".into(), "LFPB".into()];
        ws.chart_refs = vec![
            WorkspaceChart {
                airport: "LFPG".into(),
                chart: make_chart("c1"),
            },
            WorkspaceChart {
                airport: "LFPB".into(),
                chart: make_chart("c2"),
            },
        ];
        create_workspace(&conn, &ws);

        let list = list_workspaces(&conn);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "LFPG APP S");
        assert_eq!(list[0].airports, vec!["LFPG", "LFPB"]);
        assert_eq!(list[0].chart_refs.len(), 2);
        assert_eq!(list[0].chart_refs[0].airport, "LFPB"); // sorted by airport
        assert_eq!(list[0].chart_refs[1].airport, "LFPG");
    }

    #[test]
    fn test_update_workspace() {
        let conn = test_db();
        let mut ws = make_workspace("w1", "Original");
        create_workspace(&conn, &ws);

        ws.name = "Updated".into();
        ws.notes = Some("Briefing notes".into());
        ws.open_tabs = vec!["c1".into(), "c2".into()];
        ws.active_tab_index = Some(1);
        update_workspace(&conn, &ws);

        let list = list_workspaces(&conn);
        assert_eq!(list[0].name, "Updated");
        assert_eq!(list[0].notes.as_deref(), Some("Briefing notes"));
        assert_eq!(list[0].open_tabs, vec!["c1", "c2"]);
        assert_eq!(list[0].active_tab_index, Some(1));
    }

    #[test]
    fn test_delete_workspace() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "A"));
        create_workspace(&conn, &make_workspace("w2", "B"));
        assert_eq!(list_workspaces(&conn).len(), 2);

        delete_workspace(&conn, "w1");
        let list = list_workspaces(&conn);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "w2");
    }

    #[test]
    fn test_add_chart_to_workspace() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));

        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c2"));
        add_chart_to_workspace(&conn, "w1", "LFPB", &make_chart("c3"));

        let list = list_workspaces(&conn);
        assert_eq!(list[0].airports, vec!["LFPG", "LFPB"]);
        assert_eq!(list[0].chart_refs.len(), 3);
    }

    #[test]
    fn test_save_tab_state() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));

        save_tab_state(&conn, "w1", &["c1".into(), "c2".into()], Some(0));

        let list = list_workspaces(&conn);
        assert_eq!(list[0].open_tabs, vec!["c1", "c2"]);
        assert_eq!(list[0].active_tab_index, Some(0));
    }

    #[test]
    fn test_rename_workspace() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Old Name"));
        rename_workspace(&conn, "w1", "New Name");
        let list = list_workspaces(&conn);
        assert_eq!(list[0].name, "New Name");
    }

    #[test]
    fn test_remove_chart_from_workspace() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c2"));
        add_chart_to_workspace(&conn, "w1", "LFPB", &make_chart("c3"));
        assert_eq!(list_workspaces(&conn)[0].chart_refs.len(), 3);

        remove_chart_from_workspace(&conn, "w1", "LFPG", "c1");
        let ws = &list_workspaces(&conn)[0];
        assert_eq!(ws.chart_refs.len(), 2);
        assert!(ws.airports.contains(&"LFPG".to_string())); // still has c2

        remove_chart_from_workspace(&conn, "w1", "LFPB", "c3");
        let ws = &list_workspaces(&conn)[0];
        assert_eq!(ws.chart_refs.len(), 1);
        assert!(!ws.airports.contains(&"LFPB".to_string())); // removed
    }

    #[test]
    fn test_add_chart_dedup() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1")); // duplicate
        let ws = &list_workspaces(&conn)[0];
        assert_eq!(ws.chart_refs.len(), 1); // should NOT duplicate
    }

    #[test]
    fn test_set_chart_custom_title() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1"));

        set_chart_custom_title(&conn, "w1", "c1", Some("Final APP RWY 26"));

        let ws = &list_workspaces(&conn)[0];
        let chart = ws
            .chart_refs
            .iter()
            .find(|wc| wc.chart.id == "c1")
            .expect("chart c1 should exist");
        assert_eq!(
            chart.chart.custom_title.as_deref(),
            Some("Final APP RWY 26")
        );
    }

    #[test]
    fn test_clear_chart_custom_title_with_empty_value() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));
        add_chart_to_workspace(&conn, "w1", "LFPG", &make_chart("c1"));
        set_chart_custom_title(&conn, "w1", "c1", Some("Temp title"));

        set_chart_custom_title(&conn, "w1", "c1", Some("   "));

        let ws = &list_workspaces(&conn)[0];
        let chart = ws
            .chart_refs
            .iter()
            .find(|wc| wc.chart.id == "c1")
            .expect("chart c1 should exist");
        assert_eq!(chart.chart.custom_title, None);
    }

    #[test]
    fn test_save_notes() {
        let conn = test_db();
        create_workspace(&conn, &make_workspace("w1", "Test"));

        assert_eq!(list_workspaces(&conn)[0].notes, None);

        save_notes(&conn, "w1", Some("Briefing: RWY 26L in use"));
        assert_eq!(
            list_workspaces(&conn)[0].notes.as_deref(),
            Some("Briefing: RWY 26L in use")
        );

        save_notes(&conn, "w1", None);
        assert_eq!(list_workspaces(&conn)[0].notes, None);
    }
}
