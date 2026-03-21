use dioxus::prelude::*;

use crate::adapters;
use crate::application::tabs as tab_usecases;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::adapters::workspace_repository_sqlite::SqliteWorkspaceRepository;
use crate::i18n::{set_language, tr, AppLanguage};
use crate::models::ChartCategory;
use crate::state::{set_theme_mode, AppState, SidebarMode, TabContent, ThemeMode};

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
        crate::persistence::workspaces::set_chart_custom_title(
            &conn,
            workspace_id,
            chart_id,
            title_opt,
        );
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
    let repo = SqliteWorkspaceRepository::new(&conn);
    state.write().workspaces = repo.list_workspaces();
}

fn add_chart_to_workspace_and_reload(
    state: &mut Signal<AppState>,
    ws_id: &str,
    airport: &str,
    chart: &crate::models::Chart,
) {
    let conn = crate::persistence::db().lock().unwrap();
    let repo = SqliteWorkspaceRepository::new(&conn);
    repo.add_chart_to_workspace(ws_id, airport, chart);
    state.write().workspaces = repo.list_workspaces();
}

fn add_extra_tab_to_workspace_and_reload(
    state: &mut Signal<AppState>,
    ws_id: &str,
    tab: crate::models::ExtraTab,
) {
    let conn = crate::persistence::db().lock().unwrap();
    let repo = SqliteWorkspaceRepository::new(&conn);
    repo.add_extra_tab(ws_id, &tab);
    state.write().workspaces = repo.list_workspaces();
}

fn remove_extra_tab_from_workspace_and_reload(
    state: &mut Signal<AppState>,
    ws_id: &str,
    tab: crate::models::ExtraTab,
) {
    let conn = crate::persistence::db().lock().unwrap();
    let repo = SqliteWorkspaceRepository::new(&conn);
    repo.remove_extra_tab(ws_id, &tab);
    state.write().workspaces = repo.list_workspaces();
}

fn trigger_airport_search(
    state: &mut Signal<AppState>,
    airac: crate::airac::AiracCycle,
    icao: String,
    last_requested_icao: &mut Signal<String>,
    last_completed_icao: &mut Signal<String>,
) {
    if icao.len() < 4 {
        return;
    }

    if last_requested_icao() == icao && (state.read().loading || last_completed_icao() == icao) {
        return;
    }

    last_requested_icao.set(icao.clone());

    let mut state = state.clone();
    let mut last_completed_icao = last_completed_icao.clone();
    spawn(async move {
        {
            let mut s = state.write();
            s.loading = true;
            s.error = None;
            s.charts.clear();
            s.notices.clear();
            s.aip_doc = None;
        }

        let result = adapters::search_airport(&icao, &airac).await;

        {
            let mut s = state.write();
            s.charts = result.charts;
            s.notices = result.notices;
            s.aip_doc = result.aip_doc;
            s.error = result.errors.into_iter().next();
            s.loading = false;
        }

        last_completed_icao.set(icao);
    });
}

fn open_all_workspace_charts(
    state: &mut Signal<AppState>,
    chart_refs: &[crate::models::WorkspaceChart],
) {
    let mut s = state.write();
    tab_usecases::open_all_workspace_charts(&mut s, chart_refs);
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
        let (tab_ids, active) = tab_usecases::chart_tab_state(&s.tabs, s.active_tab);
        let extra_tabs: Vec<crate::models::ExtraTab> = s
            .workspaces
            .iter()
            .find(|w| w.id == ws_id)
            .map(|w| w.extra_tabs.clone())
            .unwrap_or_default();
        {
            let conn = crate::persistence::db().lock().unwrap();
            crate::persistence::workspaces::save_tab_state(
                &conn, ws_id, &tab_ids, active, &extra_tabs,
            );
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
    let mut extra_tabs: Vec<crate::models::ExtraTab> = Vec::new();
    {
        let conn = crate::persistence::db().lock().unwrap();
        let fresh = crate::persistence::workspaces::list_workspaces(&conn);
        if let Some(ws) = fresh.iter().find(|w| w.id == ws_id) {
            *open_tabs = ws.open_tabs.clone();
            *active_tab_index = ws.active_tab_index;
            extra_tabs = ws.extra_tabs.clone();
        }
        s.workspaces = fresh;
    }
    {
        let conn = crate::persistence::db().lock().unwrap();
        s.chart_zoom = crate::persistence::workspaces::load_chart_zoom(&conn, ws_id);
    }
    s.notes_pinned = workspace_notes_pinned.unwrap_or(false);
    tab_usecases::rebuild_workspace_tabs(
        &mut s,
        chart_refs,
        open_tabs,
        *active_tab_index,
        &extra_tabs,
    );
}

#[component]
pub fn Navigator() -> Element {
    let state = use_context::<Signal<AppState>>();
    let nav_open = state.read().nav_open;
    let mode = state.read().sidebar_mode;

    let cls = if nav_open {
        "navigator"
    } else {
        "navigator collapsed"
    };

    rsx! {
        div { class: "{cls}",
            NavHeader { mode }
            div { class: "nav-content",
                match mode {
                    SidebarMode::Airports => rsx! { AirportsPanel {} },
                    SidebarMode::Workspaces => rsx! { WorkspacesPanel {} },
                    SidebarMode::Settings => rsx! { SettingsPanel {} },
                    SidebarMode::Help => rsx! { HelpPanel {} },
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
        SidebarMode::Help => tr(lang, "nav.help"),
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
    let aip_doc = state.read().aip_doc.clone();
    let error = state.read().error.clone();

    let airac_memo = use_context::<Memo<crate::airac::AiracCycle>>();
    let mut last_requested_icao = use_signal(String::new);
    let mut last_completed_icao = use_signal(String::new);

    let mut run_search = move |_| {
        let icao = state.read().search_icao.clone();
        let airac = airac_memo.read().clone();
        trigger_airport_search(
            &mut state,
            airac,
            icao,
            &mut last_requested_icao,
            &mut last_completed_icao,
        );
    };

    // Pre-compute workspace membership for eAIP and ATIS (before rsx!)
    let active_ws_id = state.read().active_workspace_id.clone();
    let has_active_ws = active_ws_id.is_some();
    let aip_doc_in_ws = {
        let s = state.read();
        active_ws_id.as_deref().and_then(|ws_id| {
            s.workspaces.iter().find(|w| w.id == ws_id).map(|ws| {
                aip_doc.as_ref().map(|doc| {
                    ws.extra_tabs.iter().any(|t| matches!(t, crate::models::ExtraTab::AipDoc { doc: d } if d.id == doc.id))
                }).unwrap_or(false)
            })
        }).unwrap_or(false)
    };
    let atis_in_ws = {
        let s = state.read();
        active_ws_id.as_deref().and_then(|ws_id| {
            s.workspaces.iter().find(|w| w.id == ws_id).map(|ws| {
                ws.extra_tabs.iter().any(|t| matches!(t, crate::models::ExtraTab::Atis { icao: i } if i == &icao))
            })
        }).unwrap_or(false)
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
                    let icao = e.value().to_uppercase();
                    state.write().search_icao = icao.clone();
                    if icao.len() == 4 {
                        let airac = airac_memo.read().clone();
                        trigger_airport_search(
                            &mut state,
                            airac,
                            icao,
                            &mut last_requested_icao,
                            &mut last_completed_icao,
                        );
                    }
                },
                onkeydown: {
                    let mut run_search = run_search.clone();
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

        if !loading
            && error.is_none()
            && icao.len() == 4
            && charts.is_empty()
            && aip_doc.is_none()
            && last_completed_icao() == icao
        {
            div { class: "nav-section-title",
                "{tr(lang, \"search.no_results\") }"
            }
        }

        if !charts.is_empty() || aip_doc.is_some() {
            div { class: "nav-airport-header",
                span { class: "icao", "{icao}" }
                " — {charts.len()} {tr(lang, \"search.charts\") }"
            }
            if let Some(doc) = aip_doc {
                div {
                    class: if aip_doc_in_ws { "tree-item tree-item-in-workspace" } else { "tree-item" },
                    onclick: {
                        let doc = doc.clone();
                        move |_| {
                            let mut s = state.write();
                            tab_usecases::open_or_focus_aip(&mut s, doc.clone());
                        }
                    },
                    span { class: "badge", "DOC" }
                    span { class: "title", "{doc.title()}" }
                    if has_active_ws && !aip_doc_in_ws {
                        button {
                            class: "tree-item-action",
                            title: "{tr(lang, \"chart.quick_add\")}",
                            onclick: {
                                let doc = doc.clone();
                                move |e: MouseEvent| {
                                    e.stop_propagation();
                                    let ws_id = match state.read().active_workspace_id.clone() {
                                        Some(id) => id,
                                        None => return,
                                    };
                                    add_extra_tab_to_workspace_and_reload(
                                        &mut state,
                                        &ws_id,
                                        crate::models::ExtraTab::AipDoc { doc: doc.clone() },
                                    );
                                }
                            },
                            "+"
                        }
                    }
                }
            }
            // ATIS / METAR / TAF button — always shown when ICAO is known
            if icao.len() == 4 && !loading {
                div {
                    class: if atis_in_ws { "tree-item tree-item-in-workspace" } else { "tree-item" },
                    onclick: {
                        let icao = icao.clone();
                        move |_| {
                            let mut s = state.write();
                            tab_usecases::open_or_focus_atis(&mut s, icao.clone());
                        }
                    },
                    span { class: "badge badge-atis", "ATIS" }
                    span { class: "title", "ATIS / MET — {icao}" }
                    if has_active_ws && !atis_in_ws {
                        button {
                            class: "tree-item-action",
                            title: "{tr(lang, \"chart.quick_add\")}",
                            onclick: {
                                let icao = icao.clone();
                                move |e: MouseEvent| {
                                    e.stop_propagation();
                                    let ws_id = match state.read().active_workspace_id.clone() {
                                        Some(id) => id,
                                        None => return,
                                    };
                                    add_extra_tab_to_workspace_and_reload(
                                        &mut state,
                                        &ws_id,
                                        crate::models::ExtraTab::Atis { icao: icao.clone() },
                                    );
                                }
                            },
                            "+"
                        }
                    }
                }
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
fn ChartGroup(
    category: ChartCategory,
    charts: Vec<crate::models::Chart>,
    show_quick_add: bool,
) -> Element {
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
            .map(|ws| {
                ws.chart_refs
                    .iter()
                    .any(|wc| wc.chart.id == chart.id && wc.airport == resolved_airport)
            })
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
                    let mut s = state.write();
                    tab_usecases::open_or_focus_chart(&mut s, chart.clone(), airport.clone());
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
                            add_chart_to_workspace_and_reload(&mut state, &ws_id, &airport, &chart);
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
            extra_tabs: Vec::new(),
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
    let mut airports_map: std::collections::BTreeMap<String, Vec<crate::models::WorkspaceChart>> =
        std::collections::BTreeMap::new();
    for wc in &workspace.chart_refs {
        airports_map
            .entry(wc.airport.clone())
            .or_default()
            .push(wc.clone());
    }
    let airport_groups: Vec<(String, Vec<crate::models::WorkspaceChart>)> =
        airports_map.into_iter().collect();

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
                                let fresh_charts = state.read().charts.clone();
                                let preload_urls: Vec<String> = chart_refs
                                    .iter()
                                    .flat_map(|wc| {
                                        fresh_charts
                                            .iter()
                                            .find(|c| {
                                                c.source == wc.chart.source
                                                    && c.provider_relative_url
                                                        == wc.chart.provider_relative_url
                                            })
                                            .map(|c| c.runtime_urls())
                                            .unwrap_or_else(|| wc.chart.runtime_urls())
                                    })
                                    .collect();
                                spawn(async move {
                                    let _ = crate::pdf::pre_render_pdf_many(&preload_urls).await;
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
                if workspace.chart_refs.is_empty() && workspace.extra_tabs.is_empty() {
                    div { class: "tree-item", style: "opacity: 0.5;",
                        "{tr(lang, \"workspace.empty\")}"
                    }
                }

                // eAIP and ATIS extra tabs
                if !workspace.extra_tabs.is_empty() {
                    div { class: "tree-subgroup",
                        div { class: "tree-subgroup-header", "📡 ATIS / DOC" }
                        for et in &workspace.extra_tabs {
                            {
                                let ws_id = ws_id.clone();
                                let et = et.clone();
                                let (label, badge, badge_class) = match &et {
                                    crate::models::ExtraTab::Atis { icao } => (
                                        format!("ATIS / MET — {}", icao),
                                        "ATIS",
                                        "badge badge-atis",
                                    ),
                                    crate::models::ExtraTab::AipDoc { doc } => (
                                        doc.title(),
                                        "DOC",
                                        "badge",
                                    ),
                                };
                                rsx! {
                                    div { class: "tree-item-removable",
                                        div {
                                            class: "tree-item",
                                            style: "flex: 1;",
                                            onclick: {
                                                let et = et.clone();
                                                move |_| {
                                                    let mut s = state.write();
                                                    match &et {
                                                        crate::models::ExtraTab::Atis { icao } => {
                                                            tab_usecases::open_or_focus_atis(&mut s, icao.clone())
                                                        }
                                                        crate::models::ExtraTab::AipDoc { doc } => {
                                                            tab_usecases::open_or_focus_aip(&mut s, doc.clone())
                                                        }
                                                    }
                                                }
                                            },
                                            span { class: "{badge_class}", "{badge}" }
                                            span { class: "title", "{label}" }
                                        }
                                        button {
                                            class: "tree-item-remove",
                                            title: "{tr(lang, \"workspace.remove\")}",
                                            onclick: {
                                                let ws_id = ws_id.clone();
                                                let et = et.clone();
                                                move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    remove_extra_tab_from_workspace_and_reload(&mut state, &ws_id, et.clone());
                                                }
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
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
        div { class: "tree-item",
            span { class: "badge", "CACHE" }
            span { class: "title", "{tr(lang, \"settings.cache\")}" }
        }
        div { class: "settings-options-row",
            button {
                class: "theme-toggle",
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::cache::clear_file_caches(&conn);
                        state.write().pdf_cache.clear();
                    }
                },
                "{tr(lang, \"settings.cache.clear\") }"
            }
        }
    }
}

#[component]
fn HelpPanel() -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;

    rsx! {
        div { class: "help-panel",
            div { class: "help-section",
                div { class: "nav-section-title", "{tr(lang, \"help.section.data\")}" }
                p { class: "help-text", "{tr(lang, \"help.data.intro\")}" }
                ul { class: "help-list",
                    li { "{tr(lang, \"help.data.charts\")}" }
                    li { "{tr(lang, \"help.data.aip\")}" }
                    li { "{tr(lang, \"help.data.atis\")}" }
                    li { "{tr(lang, \"help.data.notices\")}" }
                }
            }

            div { class: "help-section",
                div { class: "nav-section-title", "{tr(lang, \"help.section.workspaces\")}" }
                p { class: "help-text", "{tr(lang, \"help.workspaces.intro\")}" }
                ul { class: "help-list",
                    li { "{tr(lang, \"help.workspaces.create\")}" }
                    li { "{tr(lang, \"help.workspaces.load\")}" }
                    li { "{tr(lang, \"help.workspaces.add\")}" }
                    li { "{tr(lang, \"help.workspaces.persist\")}" }
                }
            }

            div { class: "help-section",
                div { class: "nav-section-title", "{tr(lang, \"help.section.view\")}" }
                p { class: "help-text", "{tr(lang, \"help.view.intro\")}" }
                ul { class: "help-list",
                    li { "{tr(lang, \"help.view.tab_actions\")}" }
                    li { "{tr(lang, \"help.view.zoom\")}" }
                    li { "{tr(lang, \"help.view.notes\")}" }
                    li { "{tr(lang, \"help.view.theme\")}" }
                }
            }
        }
    }
}
