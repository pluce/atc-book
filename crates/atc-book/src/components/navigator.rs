use dioxus::prelude::*;

use crate::adapters;
use crate::i18n::{set_language, tr, AppLanguage};
use crate::models::ChartCategory;
use crate::state::{set_theme_mode, AppState, SidebarMode, Tab, TabContent, ThemeMode};

fn commit_chart_custom_title(
    state: &mut Signal<AppState>,
    workspace_id: &str,
    chart_id: &str,
    value: &str,
) {
    let trimmed = value.trim().to_string();
    let title_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.as_str())
    };

    let workspaces = {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::workspaces::set_chart_custom_title(&conn, workspace_id, chart_id, title_opt);
        crate::persistence::workspaces::list_workspaces(&conn)
    };

    let mut s = state.write();
    s.workspaces = workspaces;
    for tab in s.tabs.iter_mut() {
        if let TabContent::Chart { chart, .. } = &mut tab.content {
            if chart.id == chart_id {
                chart.custom_title = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.clone())
                };
            }
        }
    }
}

fn reload_workspaces(state: &mut Signal<AppState>) {
    let conn = crate::persistence::db().lock().unwrap();
    state.write().workspaces = crate::persistence::workspaces::list_workspaces(&conn);
}

fn open_all_workspace_charts(state: &mut Signal<AppState>, chart_refs: &[crate::models::WorkspaceChart]) {
    let mut s = state.write();
    for wc in chart_refs {
        if !s.tabs.iter().any(|t| t.id == wc.chart.id) {
            s.tabs.push(Tab::chart(wc.chart.clone(), wc.airport.clone()));
        }
    }
    if let Some(first) = chart_refs.first() {
        if let Some(idx) = s.tabs.iter().position(|t| t.id == first.chart.id) {
            s.active_tab = Some(idx);
        }
    }
}

fn toggle_workspace_loaded(
    state: &mut Signal<AppState>,
    desktop: &dioxus::desktop::DesktopContext,
    ws_id: &str,
    workspace_notes_pinned: Option<bool>,
    chart_refs: &[crate::models::WorkspaceChart],
    open_tabs: &mut Vec<String>,
    active_tab_index: &mut Option<usize>,
) {
    let mut s = state.write();
    if s.active_workspace_id.as_deref() == Some(ws_id) {
        let (tab_ids, active) = crate::components::workspace::chart_tab_state(&s.tabs, s.active_tab);
        {
            let conn = crate::persistence::db().lock().unwrap();
            crate::persistence::workspaces::save_tab_state(&conn, ws_id, &tab_ids, active);
        }
        s.active_workspace_id = None;
        s.chart_zoom.clear();
        s.notes_pinned = false;
        crate::components::popout::close_workspace_popout(desktop, ws_id);
        return;
    }

    if let Some(prev_ws) = s.active_workspace_id.clone() {
        crate::components::popout::close_workspace_popout(desktop, &prev_ws);
    }

    s.tabs.clear();
    s.pdf_cache.clear();
    s.active_workspace_id = Some(ws_id.to_string());
    {
        let conn = crate::persistence::db().lock().unwrap();
        let fresh = crate::persistence::workspaces::list_workspaces(&conn);
        if let Some(ws) = fresh.iter().find(|w| w.id == ws_id) {
            *open_tabs = ws.open_tabs.clone();
            *active_tab_index = ws.active_tab_index;
        }
        s.workspaces = fresh;
    }
    {
        let conn = crate::persistence::db().lock().unwrap();
        s.chart_zoom = crate::persistence::workspaces::load_chart_zoom(&conn, ws_id);
    }
    s.notes_pinned = workspace_notes_pinned.unwrap_or(false);
    s.tabs.push(Tab::notes());
    if !open_tabs.is_empty() {
        for tab_id in open_tabs.iter() {
            if let Some(wc) = chart_refs.iter().find(|c| c.chart.id == *tab_id) {
                s.tabs.push(Tab::chart(wc.chart.clone(), wc.airport.clone()));
            }
        }
        s.active_tab = active_tab_index.map(|i| (i + 1).min(s.tabs.len() - 1));
        if s.active_tab.is_none() {
            s.active_tab = Some(0);
        }
    } else {
        for wc in chart_refs {
            s.tabs.push(Tab::chart(wc.chart.clone(), wc.airport.clone()));
        }
        s.active_tab = Some(0);
    }
}

#[component]
pub fn Navigator() -> Element {
    let state = use_context::<Signal<AppState>>();
    let nav_open = state.read().nav_open;
    let mode = state.read().sidebar_mode;

    let cls = if nav_open { "navigator" } else { "navigator collapsed" };

    rsx! {
        div { class: "{cls}",
            NavHeader { mode }
            div { class: "nav-content",
                match mode {
                    SidebarMode::Airports => rsx! { AirportsPanel {} },
                    SidebarMode::Workspaces => rsx! { WorkspacesPanel {} },
                    SidebarMode::Settings => rsx! { SettingsPanel {} },
                }
            }
        }
    }
}

#[component]
fn NavHeader(mode: SidebarMode) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;

    let title = match mode {
        SidebarMode::Airports => tr(lang, "nav.airports"),
        SidebarMode::Workspaces => tr(lang, "nav.workspaces"),
        SidebarMode::Settings => tr(lang, "nav.settings"),
    };

    rsx! {
        div { class: "nav-header",
            span { "{title}" }
            button {
                class: "nav-close",
                onclick: move |_| { state.write().nav_open = false; },
                "✕"
            }
        }
    }
}

#[component]
fn AirportsPanel() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let icao = state.read().search_icao.clone();
    let loading = state.read().loading;
    let charts = state.read().charts.clone();
    let error = state.read().error.clone();

    let airac_memo = use_context::<Memo<crate::airac::AiracCycle>>();

    let run_search = move |_| {
        let icao = state.read().search_icao.clone();
        if icao.len() < 4 {
            return;
        }
        let airac = airac_memo.read().clone();
        spawn(async move {
            state.write().loading = true;
            state.write().error = None;
            state.write().charts.clear();

            let result = adapters::search_airport(&icao, &airac).await;

            let mut s = state.write();
            s.charts = result.charts;
            s.notices = result.notices;
            s.error = result.errors.into_iter().next(); // Show first error if any
            s.loading = false;
        });
    };

    rsx! {
        div { class: "search-box",
            input {
                class: "search-input",
                r#type: "text",
                placeholder: "{tr(lang, \"search.icao\")}",
                maxlength: "4",
                value: "{icao}",
                oninput: move |e| {
                    state.write().search_icao = e.value().to_uppercase();
                },
                onkeydown: {
                    let run_search = run_search.clone();
                    move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            run_search(());
                        }
                    }
                },
            }
            button {
                class: "search-btn",
                disabled: icao.len() < 4 || loading,
                onclick: move |_| run_search(()),
                if loading { "{tr(lang, \"search.loading\")}" } else { "{tr(lang, \"search.go\")}" }
            }
        }

        if let Some(err) = &error {
            div { class: "nav-section-title", style: "color: var(--red);",
                "⚠ {err}"
            }
        }

        if loading {
            div { style: "display: flex; justify-content: center; padding: 24px;",
                div { class: "loading-spinner" }
            }
        }

        if !charts.is_empty() {
            div { class: "nav-airport-header",
                span { class: "icao", "{icao}" }
                " — {charts.len()} {tr(lang, \"search.charts\") }"
            }
            ChartTree { charts, show_quick_add: state.read().active_workspace_id.is_some() }
        }
    }
}

#[component]
fn ChartTree(charts: Vec<crate::models::Chart>, show_quick_add: bool) -> Element {
    // Group by category
    let mut categories: Vec<ChartCategory> = charts.iter().map(|c| c.category.clone()).collect();
    categories.sort_by_key(|c| c.sort_order());
    categories.dedup();

    rsx! {
        div {
            for cat in categories {
                ChartGroup {
                    category: cat.clone(),
                    charts: charts.iter().filter(|c| c.category == cat).cloned().collect::<Vec<_>>(),
                    show_quick_add,
                }
            }
        }
    }
}

#[component]
fn ChartGroup(category: ChartCategory, charts: Vec<crate::models::Chart>, show_quick_add: bool) -> Element {
    let mut open = use_signal(|| true);
    let label = category.label();
    let count = charts.len();

    rsx! {
        div { class: "tree-group",
            div {
                class: "tree-group-header",
                onclick: move |_| { open.toggle(); },
                span {
                    class: if open() { "chevron open" } else { "chevron" },
                    "▶"
                }
                "{label} ({count})"
            }
            if open() {
                for chart in &charts {
                    ChartItem { chart: chart.clone(), airport: String::new(), show_quick_add }
                }
            }
        }
    }
}

#[component]
fn ChartItem(chart: crate::models::Chart, airport: String, show_quick_add: bool) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let title = chart.display_title().to_string();
    let resolved_airport = if airport.is_empty() {
        state.read().search_icao.clone()
    } else {
        airport.clone()
    };
    let active_ws_id = state.read().active_workspace_id.clone();
    let in_active_ws = if let Some(ws_id) = active_ws_id.as_deref() {
        state
            .read()
            .workspaces
            .iter()
            .find(|w| w.id == ws_id)
            .map(|ws| ws.chart_refs.iter().any(|wc| wc.chart.id == chart.id && wc.airport == resolved_airport))
            .unwrap_or(false)
    } else {
        false
    };

    rsx! {
        div {
            class: if show_quick_add && in_active_ws { "tree-item tree-item-in-workspace" } else { "tree-item" },
            onclick: {
                let chart = chart.clone();
                let airport = resolved_airport.clone();
                move |_| {
                    let chart = chart.clone();
                    let mut s = state.write();
                    if !s.tabs.iter().any(|t| t.id == chart.id) {
                        s.tabs.push(Tab::chart(chart.clone(), airport.clone()));
                    }
                    let idx = s.tabs.iter().position(|t| t.id == chart.id).unwrap();
                    s.active_tab = Some(idx);
                }
            },
            span { class: "badge", "{chart.category.label()}" }
            span { class: "title", "{title}" }
            if show_quick_add && !in_active_ws {
                button {
                    class: "tree-item-action",
                    title: "{tr(state.read().language, \"chart.quick_add\")}",
                    disabled: active_ws_id.is_none(),
                    onclick: {
                        let chart = chart.clone();
                        let airport = resolved_airport.clone();
                        move |e: MouseEvent| {
                            e.stop_propagation();
                            let ws_id = match state.read().active_workspace_id.clone() {
                                Some(id) => id,
                                None => return,
                            };
                            let conn = crate::persistence::db().lock().unwrap();
                            crate::persistence::workspaces::add_chart_to_workspace(
                                &conn,
                                &ws_id,
                                &airport,
                                &chart,
                            );
                            state.write().workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                        }
                    },
                    "+"
                }
            }
        }
    }
}

#[component]
fn WorkspacesPanel() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let workspaces = state.read().workspaces.clone();

    let mut creating = use_signal(|| false);
    let mut new_name = use_signal(|| String::new());

    // Reload workspaces from DB
    let mut reload = move || reload_workspaces(&mut state);

    // Load on first render
    use_effect(move || {
        reload();
    });

    // Helper: create workspace and refresh list
    let mut create_workspace = move || {
        if new_name().is_empty() {
            return;
        }
        let ws = crate::models::Workspace {
            id: uuid::Uuid::new_v4().to_string(),
            name: new_name().clone(),
            airports: Vec::new(),
            chart_refs: Vec::new(),
            open_tabs: Vec::new(),
            active_tab_index: None,
            notes: None,
            notes_pinned: Some(false),
            notes_panel_width: Some(380),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        {
            let conn = crate::persistence::db().lock().unwrap();
            crate::persistence::workspaces::create_workspace(&conn, &ws);
        }
        creating.set(false);
        reload();
    };

    rsx! {
        div { class: "folders-actions",
            button {
                class: "btn-primary",
                onclick: move |_| {
                    creating.set(true);
                    new_name.set(String::new());
                },
                "{tr(state.read().language, \"workspace.new\")}"
            }
        }

        if creating() {
            div { class: "search-box",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "{tr(state.read().language, \"workspace.name.placeholder\")}",
                    value: "{new_name}",
                    oninput: move |e| new_name.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            create_workspace();
                        } else if e.key() == Key::Escape {
                            creating.set(false);
                        }
                    },
                }
                button {
                    class: "search-btn",
                    disabled: new_name().is_empty(),
                    onclick: move |_| { create_workspace(); },
                    "{tr(state.read().language, \"common.ok\")}"
                }
            }
        }

        if workspaces.is_empty() && !creating() {
            div { class: "nav-section-title",
                "{tr(state.read().language, \"workspace.none\")}"
            }
        }

        for ws in &workspaces {
            WorkspaceItem { workspace: ws.clone() }
        }
    }
}

#[component]
fn WorkspaceItem(workspace: crate::models::Workspace) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let desktop = dioxus::desktop::use_window();
    let mut open = use_signal(|| false);
    let mut confirming_delete = use_signal(|| false);
    let mut renaming = use_signal(|| false);
    let mut rename_value = use_signal(|| String::new());
    let editing_chart = use_signal(|| Option::<String>::None);
    let editing_value = use_signal(|| String::new());

    let chart_count = workspace.chart_refs.len();
    let ws_id = workspace.id.clone();
    let is_active = state.read().active_workspace_id.as_deref() == Some(&workspace.id);

    // Build (airport, chart) pairs for tab restoration
    let ws_chart_refs = workspace.chart_refs.clone();

    // Group charts by airport for display
    let mut airports_map: std::collections::BTreeMap<String, Vec<crate::models::WorkspaceChart>> = std::collections::BTreeMap::new();
    for wc in &workspace.chart_refs {
        airports_map.entry(wc.airport.clone()).or_default().push(wc.clone());
    }
    let airport_groups: Vec<(String, Vec<crate::models::WorkspaceChart>)> = airports_map.into_iter().collect();

    rsx! {
        div { class: "tree-group",
            div {
                class: if is_active { "tree-group-header active" } else { "tree-group-header" },
                onclick: move |_| { open.toggle(); },
                span {
                    class: if open() { "chevron open" } else { "chevron" },
                    "▶"
                }
                "{workspace.name} ({chart_count})"
                // Load workspace button
                button {
                    class: "dock-action-btn",
                    title: if is_active { tr(lang, "workspace.unload") } else { tr(lang, "workspace.load") },
                    onclick: {
                        let chart_refs = ws_chart_refs.clone();
                        let ws_id = ws_id.clone();
                        let mut open_tabs = workspace.open_tabs.clone();
                        let mut active_tab_index = workspace.active_tab_index;
                        move |e: MouseEvent| {
                            e.stop_propagation();
                            toggle_workspace_loaded(
                                &mut state,
                                &desktop,
                                &ws_id,
                                workspace.notes_pinned,
                                &chart_refs,
                                &mut open_tabs,
                                &mut active_tab_index,
                            );

                            // If just loaded, warm caches and restore linked popout.
                            if state.read().active_workspace_id.as_deref() == Some(&ws_id) {
                                let preload_urls: Vec<String> = chart_refs
                                    .iter()
                                    .map(|wc| wc.chart.runtime_url())
                                    .collect();
                                spawn(async move {
                                    for url in preload_urls {
                                        let _ = crate::pdf::pre_render_pdf(&url).await;
                                    }
                                });

                                let conn = crate::persistence::db().lock().unwrap();
                                let (pop_tabs, _) = crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);
                                if !pop_tabs.is_empty() && !crate::components::popout::has_workspace_popout(&ws_id) {
                                    let ws_id_clone = ws_id.clone();
                                    spawn(async move {
                                        crate::components::popout::open_workspace_popout(ws_id_clone).await;
                                    });
                                }
                            }
                        }
                    },
                    if is_active { "⏹" } else { "▶" }
                }
                if !workspace.chart_refs.is_empty() {
                    button {
                        class: "dock-action-btn",
                        title: "{tr(lang, \"workspace.open_all\")}",
                        onclick: {
                            let chart_refs = ws_chart_refs.clone();
                            move |e: MouseEvent| {
                                e.stop_propagation();
                                open_all_workspace_charts(&mut state, &chart_refs);
                            }
                        },
                        "▶▶"
                    }
                }
                // Rename button
                button {
                    class: "dock-action-btn",
                    title: "{tr(lang, \"workspace.rename\")}",
                    onclick: {
                        let name = workspace.name.clone();
                        move |e: MouseEvent| {
                            e.stop_propagation();
                            rename_value.set(name.clone());
                            renaming.set(true);
                        }
                    },
                    "✎"
                }
                button {
                    class: "tab-close",
                    onclick: {
                        move |e: MouseEvent| {
                            e.stop_propagation();
                            confirming_delete.set(true);
                        }
                    },
                    "✕"
                }
            }
            // Rename inline editor
            if renaming() {
                div { class: "search-box",
                    input {
                        class: "search-input",
                        r#type: "text",
                        value: "{rename_value}",
                        oninput: move |e| rename_value.set(e.value()),
                        onkeydown: {
                            let ws_id = ws_id.clone();
                            move |e: KeyboardEvent| {
                                if e.key() == Key::Enter && !rename_value().is_empty() {
                                    {
                                        let conn = crate::persistence::db().lock().unwrap();
                                        crate::persistence::workspaces::rename_workspace(&conn, &ws_id, &rename_value());
                                    }
                                    renaming.set(false);
                                    reload_workspaces(&mut state);
                                } else if e.key() == Key::Escape {
                                    renaming.set(false);
                                }
                            }
                        },
                    }
                }
            }
            if confirming_delete() {
                div { class: "confirm-bar",
                    span { "{tr(lang, \"workspace.delete.confirm\")}" }
                    button {
                        class: "confirm-yes",
                        onclick: {
                            let ws_id = ws_id.clone();
                            move |_| {
                                {
                                    let conn = crate::persistence::db().lock().unwrap();
                                    crate::persistence::workspaces::delete_workspace(&conn, &ws_id);
                                }
                                let conn = crate::persistence::db().lock().unwrap();
                                let mut s = state.write();
                                s.workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                                if s.active_workspace_id.as_deref() == Some(&ws_id) {
                                    s.active_workspace_id = None;
                                }
                            }
                        },
                        "{tr(lang, \"common.yes\")}"
                    }
                    button {
                        class: "confirm-no",
                        onclick: move |_| { confirming_delete.set(false); },
                        "{tr(lang, \"common.no\")}"
                    }
                }
            }
            if open() {
                for (airport, wcharts) in &airport_groups {
                    {
                        let count = wcharts.len();
                        let ws_id = ws_id.clone();
                        rsx! {
                            div { class: "tree-subgroup",
                                div { class: "tree-subgroup-header",
                                    "✈ {airport} ({count})"
                                }
                                for wc in wcharts {
                                    WorkspaceChartRow {
                                        ws_id: ws_id.clone(),
                                        airport: wc.airport.clone(),
                                        chart: wc.chart.clone(),
                                        editing_chart,
                                        editing_value,
                                    }
                                }
                            }
                        }
                    }
                }
                if workspace.chart_refs.is_empty() {
                    div { class: "tree-item", style: "opacity: 0.5;",
                        "{tr(lang, \"workspace.empty\")}"
                    }
                }

            }
        }
    }
}

#[component]
fn WorkspaceChartRow(
    ws_id: String,
    airport: String,
    chart: crate::models::Chart,
    mut editing_chart: Signal<Option<String>>,
    mut editing_value: Signal<String>,
) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let chart_id = chart.id.clone();
    let is_editing = editing_chart().as_deref() == Some(&chart_id);

    rsx! {
        div { class: "tree-item-removable",
            if is_editing {
                input {
                    class: "tree-item-rename-input",
                    r#type: "text",
                    value: "{editing_value}",
                    oninput: move |e| editing_value.set(e.value()),
                    onkeydown: {
                        let chart_id = chart_id.clone();
                        let ws_id = ws_id.clone();
                        move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                e.prevent_default();
                                e.stop_propagation();
                                let value = editing_value();
                                commit_chart_custom_title(&mut state, &ws_id, &chart_id, &value);
                                editing_chart.set(None);
                            } else if e.key() == Key::Escape {
                                e.prevent_default();
                                e.stop_propagation();
                                editing_chart.set(None);
                            }
                        }
                    },
                    onblur: {
                        let chart_id = chart_id.clone();
                        let ws_id = ws_id.clone();
                        move |_| {
                            if editing_chart().as_deref() != Some(&chart_id) {
                                return;
                            }
                            let value = editing_value();
                            commit_chart_custom_title(&mut state, &ws_id, &chart_id, &value);
                            editing_chart.set(None);
                        }
                    }
                }
            } else {
                ChartItem { chart: chart.clone(), airport: airport.clone(), show_quick_add: false }
            }
            button {
                class: "tree-item-edit",
                title: "{tr(lang, \"workspace.rename\")}",
                onclick: {
                    let chart_id = chart_id.clone();
                    let title = chart.display_title().to_string();
                    move |e: MouseEvent| {
                        e.stop_propagation();
                        editing_value.set(title.clone());
                        editing_chart.set(Some(chart_id.clone()));
                    }
                },
                "✎"
            }
            button {
                class: "tree-item-remove",
                title: "{tr(lang, \"workspace.remove\")}",
                onclick: {
                    let ws_id = ws_id.clone();
                    let airport = airport.clone();
                    let chart_id = chart_id.clone();
                    move |e: MouseEvent| {
                        e.stop_propagation();
                        {
                            let conn = crate::persistence::db().lock().unwrap();
                            crate::persistence::workspaces::remove_chart_from_workspace(
                                &conn, &ws_id, &airport, &chart_id,
                            );
                        }
                        reload_workspaces(&mut state);
                    }
                },
                "✕"
            }
        }
    }
}

#[component]
fn SettingsPanel() -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let current_theme_mode = state.read().theme_mode;

    rsx! {
        div { class: "nav-section-title", "{tr(lang, \"settings.preferences\")}" }

        div { class: "tree-item",
            span { class: "badge", "LANG" }
            span { class: "title", "{tr(lang, \"settings.language\")}" }
        }
        div { class: "settings-options-row",
            button {
                class: if lang == AppLanguage::Fr { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        state.write().language = AppLanguage::Fr;
                        set_language(AppLanguage::Fr);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_language(&conn, AppLanguage::Fr);
                    }
                },
                "{tr(lang, \"settings.lang.fr\")}"
            }
            button {
                class: if lang == AppLanguage::Uk { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        state.write().language = AppLanguage::Uk;
                        set_language(AppLanguage::Uk);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_language(&conn, AppLanguage::Uk);
                    }
                },
                "{tr(lang, \"settings.lang.uk\")}"
            }
        }

        div { class: "tree-item",
            span { class: "badge", "THEME" }
            span { class: "title", "{tr(lang, \"settings.theme_mode\")}" }
        }
        div { class: "settings-options-grid",
            button {
                class: if current_theme_mode == ThemeMode::Light { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        let mode = ThemeMode::Light;
                        set_theme_mode(mode);
                        let mut s = state.write();
                        s.theme_mode = mode;
                        s.night_mode = crate::state::resolve_night_mode(mode);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_theme_mode(&conn, mode);
                    }
                },
                "{tr(lang, \"settings.theme.light\")}"
            }
            button {
                class: if current_theme_mode == ThemeMode::Dark { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        let mode = ThemeMode::Dark;
                        set_theme_mode(mode);
                        let mut s = state.write();
                        s.theme_mode = mode;
                        s.night_mode = crate::state::resolve_night_mode(mode);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_theme_mode(&conn, mode);
                    }
                },
                "{tr(lang, \"settings.theme.dark\")}"
            }
            button {
                class: if current_theme_mode == ThemeMode::AutoTime { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        let mode = ThemeMode::AutoTime;
                        set_theme_mode(mode);
                        let mut s = state.write();
                        s.theme_mode = mode;
                        s.night_mode = crate::state::resolve_night_mode(mode);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_theme_mode(&conn, mode);
                    }
                },
                "{tr(lang, \"settings.theme.auto_time\")}"
            }
            button {
                class: if current_theme_mode == ThemeMode::AutoSystem { "theme-toggle active" } else { "theme-toggle" },
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        let mode = ThemeMode::AutoSystem;
                        set_theme_mode(mode);
                        let mut s = state.write();
                        s.theme_mode = mode;
                        s.night_mode = crate::state::resolve_night_mode(mode);
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::settings::save_theme_mode(&conn, mode);
                    }
                },
                "{tr(lang, \"settings.theme.auto_system\")}"
            }
        }
    }
}
