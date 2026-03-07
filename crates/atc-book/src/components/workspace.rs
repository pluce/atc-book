use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::Duration;

use crate::i18n::tr;
use crate::state::{AppState, PdfState, TabContent};

/// Base image display width in pixels (at 100% zoom).
const BASE_IMG_WIDTH: f64 = 900.0;

pub(crate) fn chart_tab_state(tabs: &[crate::state::Tab], active_tab: Option<usize>) -> (Vec<String>, Option<usize>) {
    let mut ids = Vec::new();
    let mut active_chart_index = None;
    for (i, tab) in tabs.iter().enumerate() {
        if let Some(id) = tab.chart_id() {
            let idx = ids.len();
            ids.push(id.to_string());
            if Some(i) == active_tab {
                active_chart_index = Some(idx);
            }
        }
    }
    (ids, active_chart_index)
}

#[component]
pub fn Workspace() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let desktop_for_sync = dioxus::desktop::use_window();
    let tabs = state.read().tabs.clone();
    let active_tab = state.read().active_tab;

    // Zoom lives here so DocMeta and DocViewer share it
    let mut zoom = use_signal(|| 100u32);

    // Notes side panel width (session-only)
    let mut notes_width = use_signal(|| 380i32);
    let mut resizing = use_signal(|| false);
    let mut resize_start_x = use_signal(|| 0i32);
    let mut resize_start_w = use_signal(|| 380i32);
    let mut last_ws_id = use_signal(|| Option::<String>::None);
    let mut popout_opening = use_signal(|| false);

    // Auto-save tab state when a workspace is active
    use_effect(move || {
        let s = state.read();
        if let Some(ref ws_id) = s.active_workspace_id {
            // Only persist chart tabs (skip the notes tab)
            let (tab_ids, active) = chart_tab_state(&s.tabs, s.active_tab);
            let ws_id = ws_id.clone();
            let conn = crate::persistence::db().lock().unwrap();
            if s.is_popout {
                crate::persistence::workspaces::save_popout_tab_state(&conn, &ws_id, &tab_ids, active);
            } else {
                crate::persistence::workspaces::save_tab_state(&conn, &ws_id, &tab_ids, active);
            }
        }
    });

    // Restore per-chart zoom when switching tabs
    use_effect(move || {
        let s = state.read();
        if let Some(idx) = s.active_tab {
            if let Some(tab) = s.tabs.get(idx) {
                if let TabContent::Chart { chart, .. } = &tab.content {
                    let target = *s.chart_zoom.get(&chart.id).unwrap_or(&100);
                    if zoom() != target {
                        zoom.set(target);
                    }
                }
            }
        }
    });

    // Restore notes panel width when switching workspaces
    use_effect(move || {
        let s = state.read();
        if s.active_workspace_id != last_ws_id() {
            last_ws_id.set(s.active_workspace_id.clone());
            let width = s
                .active_workspace_id
                .as_deref()
                .and_then(|id| s.workspaces.iter().find(|w| w.id == id))
                .and_then(|w| w.notes_panel_width)
                .unwrap_or(380);
            notes_width.set(width);
        }
    });

    let notes_pinned = state.read().notes_pinned;
    let lang = state.read().language;
    let has_workspace = state.read().active_workspace_id.is_some();
    let quick_open = state.read().quick_switcher_open;
    let quick_query = state.read().quick_switcher_query.clone();
    let _is_popout = state.read().is_popout;

    let switcher_items: Vec<(String, String, crate::models::Chart, String)> = {
        let s = state.read();
        if let Some(ws_id) = s.active_workspace_id.as_deref() {
            if let Some(ws) = s.workspaces.iter().find(|w| w.id == ws_id) {
                ws.chart_refs
                    .iter()
                    .map(|wc| {
                        (
                            wc.chart.id.clone(),
                            format!("{} · {}", wc.airport, wc.chart.display_title()),
                            wc.chart.clone(),
                            wc.airport.clone(),
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            s.tabs
                .iter()
                .filter_map(|t| match &t.content {
                    TabContent::Chart { chart, airport } => Some((
                        chart.id.clone(),
                        format!("{} · {}", airport, chart.display_title()),
                        chart.clone(),
                        airport.clone(),
                    )),
                    _ => None,
                })
                .collect()
        }
    };

    let q = quick_query.to_lowercase();
    let filtered_items: Vec<(String, String, crate::models::Chart, String)> = switcher_items
        .into_iter()
        .filter(|(_, label, chart, airport)| {
            q.is_empty()
                || label.to_lowercase().contains(&q)
                || chart.id.to_lowercase().contains(&q)
                || airport.to_lowercase().contains(&q)
        })
        .collect();

    // Sync main window tabs after popout closes.
    use_effect(move || {
        let _desktop = desktop_for_sync.clone();
        spawn(async move {
            loop {
                Delay::new(Duration::from_millis(500)).await;
                let mut s = state.write();
                if s.is_popout || !s.popout_sync_pending {
                    continue;
                }
                let ws_id = match s.active_workspace_id.clone() {
                    Some(id) => id,
                    None => continue,
                };
                if crate::components::popout::has_workspace_popout(&ws_id) {
                    continue;
                }
                let conn = crate::persistence::db().lock().unwrap();
                let (pop_tabs, _) = crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);
                if !pop_tabs.is_empty() {
                    continue;
                }
                // Reload main tabs from persisted state
                let workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                if let Some(ws) = workspaces.iter().find(|w| w.id == ws_id) {
                    let open_tabs = ws.open_tabs.clone();
                    let active_tab_index = ws.active_tab_index;
                    let chart_refs = ws.chart_refs.clone();
                    s.workspaces = workspaces;
                    s.tabs.clear();
                    s.tabs.push(crate::state::Tab::notes());
                    for tab_id in &open_tabs {
                        if let Some(wc) = chart_refs.iter().find(|c| c.chart.id == *tab_id) {
                            s.tabs.push(crate::state::Tab::chart(wc.chart.clone(), wc.airport.clone()));
                        }
                    }
                    s.active_tab = active_tab_index.map(|i| (i + 1).min(s.tabs.len().saturating_sub(1)));
                    if s.active_tab.is_none() {
                        s.active_tab = Some(0);
                    }
                }
                s.popout_sync_pending = false;
            }
        });
    });

    // Auto-open popout window if this workspace has popout tabs and none is visible yet.
    use_effect(move || {
        let s = state.read();
        if s.is_popout {
            return;
        }
        let ws_id = s.active_workspace_id.clone();
        if ws_id.is_none() {
            popout_opening.set(false);
            return;
        }
        let ws_id = ws_id.unwrap();
        if crate::components::popout::has_workspace_popout(&ws_id) {
            popout_opening.set(false);
            return;
        }
        if popout_opening() {
            return;
        }
        let conn = crate::persistence::db().lock().unwrap();
        let (pop_tabs, _) = crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);
        if pop_tabs.is_empty() {
            return;
        }
        popout_opening.set(true);
        spawn(async move {
            crate::components::popout::open_workspace_popout(ws_id).await;
        });
    });

    rsx! {
        div { class: "workspace",
            TabBar {}
            div {
                class: "workspace-body",
                onmousemove: move |e: MouseEvent| {
                    if resizing() {
                        let delta = resize_start_x() - e.client_coordinates().x as i32;
                        let next = (resize_start_w() + delta).clamp(260, 800);
                        notes_width.set(next);
                    }
                },
                onmouseup: move |_| {
                    if resizing() {
                        resizing.set(false);
                        let ws_id = state.read().active_workspace_id.clone();
                        if let Some(ws_id) = ws_id {
                            let width = notes_width();
                            let conn = crate::persistence::db().lock().unwrap();
                            crate::persistence::workspaces::save_notes_panel_width(
                                &conn,
                                &ws_id,
                                width,
                            );
                            if let Some(ws) = state
                                .write()
                                .workspaces
                                .iter_mut()
                                .find(|w| w.id == ws_id)
                            {
                                ws.notes_panel_width = Some(width);
                            }
                        }
                    }
                },
                onmouseleave: move |_| {
                    if resizing() {
                        resizing.set(false);
                        let ws_id = state.read().active_workspace_id.clone();
                        if let Some(ws_id) = ws_id {
                            let width = notes_width();
                            let conn = crate::persistence::db().lock().unwrap();
                            crate::persistence::workspaces::save_notes_panel_width(
                                &conn,
                                &ws_id,
                                width,
                            );
                            if let Some(ws) = state
                                .write()
                                .workspaces
                                .iter_mut()
                                .find(|w| w.id == ws_id)
                            {
                                ws.notes_panel_width = Some(width);
                            }
                        }
                    }
                },
                div { class: "workspace-main",
                    if let Some(idx) = active_tab {
                        if let Some(tab) = tabs.get(idx) {
                            match &tab.content {
                                TabContent::Chart { chart, .. } => rsx! {
                                    DocMeta { chart: chart.clone(), zoom }
                                    DocViewer { chart: chart.clone(), zoom }
                                },
                                TabContent::Notes if !notes_pinned => rsx! {
                                    NotesEditor { pinned: false }
                                },
                                // Notes tab is active but pinned — show empty state prompting to use side panel
                                _ => rsx! {
                                    EmptyState {}
                                },
                            }
                        }
                    } else {
                        EmptyState {}
                    }
                }
                if notes_pinned && has_workspace {
                    div {
                        class: "notes-side-panel",
                        id: "notes-side-panel",
                        style: "width: {notes_width}px;",
                        div {
                            class: "notes-resize-handle",
                            onmousedown: move |e: MouseEvent| {
                                resizing.set(true);
                                resize_start_x.set(e.client_coordinates().x as i32);
                                resize_start_w.set(notes_width());
                            },
                        }
                        div { class: "notes-side-header",
                            span { "📝 {tr(lang, \"notes.title\")}" }
                            button {
                                class: "notes-pin-btn active",
                                title: "{tr(lang, \"notes.unpin\")}",
                                onclick: move |_| {
                                    let (ws_id, is_popout) = {
                                        let s = state.read();
                                        (s.active_workspace_id.clone(), s.is_popout)
                                    };
                                    state.write().notes_pinned = false;
                                    if !is_popout {
                                        if let Some(ws_id) = ws_id {
                                        let conn = crate::persistence::db().lock().unwrap();
                                        crate::persistence::workspaces::save_notes_pinned(&conn, &ws_id, false);
                                        if let Some(ws) = state
                                            .write()
                                            .workspaces
                                            .iter_mut()
                                            .find(|w| w.id == ws_id)
                                        {
                                            ws.notes_pinned = Some(false);
                                        }
                                    }
                                    }
                                },
                                "📌"
                            }
                        }
                        NotesEditor { pinned: true }
                    }
                }

                if quick_open {
                    div {
                        class: "quick-switcher-backdrop",
                        onclick: move |_| {
                            state.write().quick_switcher_open = false;
                        },
                        div {
                            class: "quick-switcher",
                            onclick: move |e: MouseEvent| e.stop_propagation(),
                            input {
                                class: "quick-switcher-input",
                                r#type: "text",
                                placeholder: "{tr(lang, \"quickswitch.placeholder\")}",
                                value: "{quick_query}",
                                oninput: move |e| {
                                    state.write().quick_switcher_query = e.value();
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Escape {
                                        state.write().quick_switcher_open = false;
                                    }
                                },
                            }
                            div { class: "quick-switcher-list",
                                if filtered_items.is_empty() {
                                    div { class: "quick-switcher-item muted", "{tr(lang, \"quickswitch.empty\")}" }
                                }
                                for (chart_id, label, chart, airport) in filtered_items {
                                    div {
                                        class: "quick-switcher-item",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            if let Some(idx) = s.tabs.iter().position(|t| t.id == chart_id) {
                                                s.active_tab = Some(idx);
                                            } else {
                                                s.tabs.push(crate::state::Tab::chart(chart.clone(), airport.clone()));
                                                s.active_tab = Some(s.tabs.len() - 1);
                                            }
                                            s.quick_switcher_open = false;
                                        },
                                        "{label}"
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
fn TabBar() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let tabs = state.read().tabs.clone();
    let active_tab = state.read().active_tab;

    if tabs.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "tab-bar",
            for (i, tab) in tabs.iter().enumerate() {
                {
                    let cls = if active_tab == Some(i) { "tab active" } else { "tab" };
                    let title = tab.title();
                    let tab_id = tab.id.clone();
                    let is_notes = tab.is_notes();
                    rsx! {
                        div {
                            class: "{cls}",
                            onclick: move |_| {
                                let mut s = state.write();
                                // Evict oldest rendered PDFs if cache exceeds limit
                                const MAX_RENDERED: usize = 5;
                                if s.pdf_cache.len() > MAX_RENDERED {
                                    // Keep only tabs that are still open
                                    let open_ids: std::collections::HashSet<String> =
                                        s.tabs.iter().map(|t| t.id.clone()).collect();
                                    s.pdf_cache.retain(|id, _| open_ids.contains(id));
                                    // If still over limit, remove entries not matching new active tab
                                    if s.pdf_cache.len() > MAX_RENDERED {
                                        let active_id = s.tabs.get(i).map(|t| t.id.clone());
                                        let to_remove: Vec<String> = s.pdf_cache.keys()
                                            .filter(|id| active_id.as_deref() != Some(id))
                                            .take(s.pdf_cache.len() - MAX_RENDERED)
                                            .cloned()
                                            .collect();
                                        for id in to_remove {
                                            s.pdf_cache.remove(&id);
                                        }
                                    }
                                }
                                s.active_tab = Some(i);
                            },
                            span { class: "tab-label", "{title}" }
                            // Notes tab cannot be closed
                            if !is_notes {
                                button {
                                    class: "tab-close",
                                    onclick: {
                                        let tab_id = tab_id.clone();
                                        move |e: MouseEvent| {
                                            e.stop_propagation();
                                            let mut s = state.write();
                                            if let Some(pos) = s.tabs.iter().position(|t| t.id == tab_id) {
                                                s.tabs.remove(pos);
                                                // Adjust active_tab
                                                match s.active_tab {
                                                    Some(a) if a == pos => {
                                                        s.active_tab = if s.tabs.is_empty() {
                                                            None
                                                        } else {
                                                            Some(a.min(s.tabs.len() - 1))
                                                        };
                                                    }
                                                    Some(a) if a > pos => {
                                                        s.active_tab = Some(a - 1);
                                                    }
                                                    _ => {}
                                                }
                                            }
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

#[component]
fn DocToolbar(chart: crate::models::Chart) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let mut show_menu = use_signal(|| false);
    let mut show_folder_menu = use_signal(|| false);
    let mut show_display_menu = use_signal(|| false);
    let mut workspace_filter = use_signal(|| String::new());
    let workspaces = state.read().workspaces.clone();
    // Get the airport from the active tab (correct even after switching search)
    let tab_airport = {
        let s = state.read();
        s.active_tab
            .and_then(|idx| s.tabs.get(idx))
            .and_then(|t| match &t.content {
                TabContent::Chart { airport, .. } => Some(airport.clone()),
                _ => None,
            })
            .unwrap_or_else(|| s.search_icao.clone())
    };

    let filter_value = workspace_filter().to_lowercase();
    let filtered_workspaces: Vec<&crate::models::Workspace> = workspaces
        .iter()
        .filter(|ws| {
            filter_value.is_empty()
                || ws.name.to_lowercase().contains(&filter_value)
        })
        .collect();
    let filtered_empty = filtered_workspaces.is_empty();
    let ws_id_for_popout = state.read().active_workspace_id.clone();
    let has_popout = ws_id_for_popout
        .as_deref()
        .map(crate::components::popout::has_workspace_popout)
        .unwrap_or(false);

    let active_ws = state.read().active_workspace_id.clone()
        .and_then(|id| workspaces.iter().find(|w| w.id == id).cloned());
    let chart_in_active = active_ws.as_ref().map(|ws| {
        ws.chart_refs.iter().any(|wc| wc.airport == tab_airport && wc.chart.id == chart.id)
    }).unwrap_or(false);
    let tab_airport_for_popout = tab_airport.clone();
    let chart_for_popout = chart.clone();
    let chart_id = chart.id.clone();

    rsx! {
        div { class: "doc-floating-menu",
            div {
                class: if show_menu() { "toolbar-dropdown-wrapper menu-expanded" } else { "toolbar-dropdown-wrapper" },
                onmouseleave: move |_| {
                    show_menu.set(false);
                    show_folder_menu.set(false);
                    show_display_menu.set(false);
                    workspace_filter.set(String::new());
                },
                button {
                    class: "toolbar-btn toolbar-btn-primary",
                    onclick: move |_| {
                        show_menu.toggle();
                        show_folder_menu.set(false);
                        show_display_menu.set(false);
                        if !show_menu() {
                            workspace_filter.set(String::new());
                        }
                    },
                    span { class: "icon toolbar-menu-icon", "☰" }
                }
                if show_menu() {
                    div {
                        class: "toolbar-menu-panel",
                        div {
                            class: "toolbar-menu-list",
                            div {
                                class: "toolbar-menu-item",
                                onmouseenter: move |_| {
                                    show_folder_menu.set(true);
                                    show_display_menu.set(false);
                                },
                                "{tr(lang, \"menu.send_to_workspace\")}"
                                span { class: "toolbar-menu-caret", ">" }
                            }
                            div {
                                class: "toolbar-menu-item",
                                onmouseenter: move |_| {
                                    show_display_menu.set(true);
                                    show_folder_menu.set(false);
                                },
                                "{tr(lang, \"menu.display\")}"
                                span { class: "toolbar-menu-caret", ">" }
                            }
                        }
                        if show_folder_menu() {
                            div {
                                class: "toolbar-submenu",
                                onmouseenter: move |_| { show_folder_menu.set(true); },
                                div {
                                    class: if active_ws.is_some() && !chart_in_active { "toolbar-menu-item" } else { "toolbar-menu-item disabled" },
                                    onclick: move |_| {
                                        if let Some(ws) = active_ws.clone() {
                                            if chart_in_active {
                                                return;
                                            }
                                            let conn = crate::persistence::db().lock().unwrap();
                                            crate::persistence::workspaces::add_chart_to_workspace(
                                                &conn,
                                                &ws.id,
                                                &tab_airport,
                                                &chart,
                                            );
                                            state.write().workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                                        }
                                    },
                                    if let Some(ws) = &active_ws {
                                        "{tr(lang, \"menu.current_workspace\")}: {ws.name}"
                                    } else {
                                        "{tr(lang, \"menu.current_workspace\")}: {tr(lang, \"menu.none\")}"
                                    }
                                }
                                div { class: "toolbar-submenu-divider" }
                                input {
                                    class: "toolbar-search",
                                    r#type: "text",
                                    placeholder: "{tr(lang, \"menu.filter\")}",
                                    value: "{workspace_filter}",
                                    oninput: move |e| workspace_filter.set(e.value()),
                                }
                                if workspaces.is_empty() {
                                    div { class: "toolbar-menu-item disabled",
                                        "{tr(lang, \"menu.no_workspace\")}"
                                    }
                                }
                                for ws in filtered_workspaces.iter() {
                                    {
                                        let ws_id = ws.id.clone();
                                        let ws_name = ws.name.clone();
                                        let chart = chart.clone();
                                        let airport = tab_airport.clone();
                                        rsx! {
                                            div {
                                                class: "toolbar-menu-item",
                                                onclick: move |_| {
                                                    let conn = crate::persistence::db().lock().unwrap();
                                                    crate::persistence::workspaces::add_chart_to_workspace(&conn, &ws_id, &airport, &chart);
                                                    state.write().workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                                                },
                                                "{ws_name}"
                                            }
                                        }
                                    }
                                }
                                if !workspaces.is_empty() && filtered_empty {
                                    div { class: "toolbar-menu-item disabled", "{tr(lang, \"menu.no_result\")}" }
                                }
                            }
                        }
                        if show_display_menu() {
                            div {
                                class: "toolbar-submenu",
                                onmouseenter: move |_| { show_display_menu.set(true); },
                                if !state.read().is_popout {
                                    div {
                                        class: "toolbar-menu-item",
                                        onclick: move |_| {
                                            let ws_id = state.read().active_workspace_id.clone();
                                            let chart_id = chart_id.clone();
                                            if let Some(ws_id) = ws_id {
                                                {
                                                    let conn = crate::persistence::db().lock().unwrap();
                                                    let (mut pop_ids, active) = crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);
                                                    if !pop_ids.contains(&chart_id) {
                                                        pop_ids.push(chart_id.clone());
                                                    }
                                                    crate::persistence::workspaces::save_popout_tab_state(&conn, &ws_id, &pop_ids, active);
                                                }

                                                // Move tab out of current window
                                                {
                                                    let mut s = state.write();
                                                    if let Some(pos) = s.tabs.iter().position(|t| t.id == chart_id) {
                                                        s.tabs.remove(pos);
                                                        if let Some(a) = s.active_tab {
                                                            s.active_tab = Some(a.min(s.tabs.len().saturating_sub(1)));
                                                        }
                                                    }
                                                    if !s.tabs.iter().any(|t| t.is_notes()) {
                                                        s.tabs.insert(0, crate::state::Tab::notes());
                                                    }
                                                    if s.active_tab.is_none() {
                                                        s.active_tab = Some(0);
                                                    }
                                                    // Persist main open tabs so popout close can merge safely
                                                    let (tab_ids, active) = chart_tab_state(&s.tabs, s.active_tab);
                                                    let conn = crate::persistence::db().lock().unwrap();
                                                    crate::persistence::workspaces::save_tab_state(&conn, &ws_id, &tab_ids, active);
                                                    if let Some(ws) = s.workspaces.iter_mut().find(|w| w.id == ws_id) {
                                                        ws.open_tabs = tab_ids.clone();
                                                        ws.active_tab_index = active;
                                                    }
                                                    s.popout_sync_pending = true;
                                                }

                                                if !crate::components::popout::has_workspace_popout(&ws_id) {
                                                    spawn(async move {
                                                        crate::components::popout::open_workspace_popout(ws_id).await;
                                                    });
                                                }
                                            } else {
                                                let chart = chart_for_popout.clone();
                                                let airport = tab_airport_for_popout.clone();
                                                spawn(async move {
                                                    crate::components::popout::open_chart_popout(chart, airport).await;
                                                });
                                            }
                                            show_menu.set(false);
                                        },
                                        if has_popout {
                                            "{tr(lang, \"menu.popout_existing\")}"
                                        } else {
                                            "{tr(lang, \"menu.popout_new\")}"
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
fn DocMeta(chart: crate::models::Chart, mut zoom: Signal<u32>) -> Element {
    let state = use_context::<Signal<AppState>>();
    let airac = use_context::<Memo<crate::airac::AiracCycle>>();
    let z = zoom();
    let chart_id_minus = chart.id.clone();
    let chart_id_plus = chart.id.clone();
    let chart_id_fit = chart.id.clone();
    let state_minus = state.clone();
    let state_plus = state.clone();
    let state_fit = state.clone();

    rsx! {
        div { class: "doc-meta",
            div { class: "doc-meta-left",
                "TYPE: {chart.source:?}"
                span { class: "separator", " | " }
                "REF: {chart.id}"
                span { class: "separator", " | " }
                "AIRAC {airac.read().code}"
            }
            div { class: "doc-meta-zoom",
                button {
                    class: "zoom-btn",
                    disabled: z <= 50,
                    onclick: move |_| {
                        let chart_id = chart_id_minus.clone();
                        let mut state = state_minus.clone();
                        let cur = zoom();
                        if cur > 50 {
                            let next = cur - 25;
                            zoom.set(next);
                            let ws_id = state.read().active_workspace_id.clone();
                            state.write().chart_zoom.insert(chart_id.clone(), next);
                            if let Some(ws_id) = ws_id {
                                let conn = crate::persistence::db().lock().unwrap();
                                crate::persistence::workspaces::save_chart_zoom(
                                    &conn,
                                    &ws_id,
                                    &chart_id,
                                    next,
                                );
                            }
                        }
                    },
                    "\u{2212}"
                }
                span { class: "zoom-label", "{z}%" }
                button {
                    class: "zoom-btn",
                    disabled: z >= 200,
                    onclick: move |_| {
                        let chart_id = chart_id_plus.clone();
                        let mut state = state_plus.clone();
                        let cur = zoom();
                        if cur < 200 {
                            let next = cur + 25;
                            zoom.set(next);
                            let ws_id = state.read().active_workspace_id.clone();
                            state.write().chart_zoom.insert(chart_id.clone(), next);
                            if let Some(ws_id) = ws_id {
                                let conn = crate::persistence::db().lock().unwrap();
                                crate::persistence::workspaces::save_chart_zoom(
                                    &conn,
                                    &ws_id,
                                    &chart_id,
                                    next,
                                );
                            }
                        }
                    },
                    "+"
                }
                button {
                    class: "zoom-btn fit-btn",
                    onclick: move |_| {
                        let chart_id = chart_id_fit.clone();
                        let mut state = state_fit.clone();
                        spawn(async move {
                            let js = format!(
                                "const el = document.getElementById('pdf-scroll'); \
                                 if (el) {{ return Math.floor((el.clientWidth - 48) / {} * 100); }} \
                                 return 100;",
                                BASE_IMG_WIDTH
                            );
                            let result = document::eval(&js).await;
                            if let Ok(val) = result {
                                if let Some(fit) = val.as_f64() {
                                    let fit = ((fit / 25.0).round() * 25.0) as u32;
                                    let fit = fit.clamp(50, 200);
                                    zoom.set(fit);
                                    let ws_id = state.read().active_workspace_id.clone();
                                    state.write().chart_zoom.insert(chart_id.clone(), fit);
                                    if let Some(ws_id) = ws_id {
                                        let conn = crate::persistence::db().lock().unwrap();
                                        crate::persistence::workspaces::save_chart_zoom(
                                            &conn,
                                            &ws_id,
                                            &chart_id,
                                            fit,
                                        );
                                    }
                                }
                            }
                        });
                    },
                    "Fit"
                }
            }
        }
    }
}

#[component]
fn DocViewer(chart: crate::models::Chart, zoom: Signal<u32>) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let chart_id = chart.id.clone();
    let pdf_state = state.read().pdf_cache.get(&chart_id).cloned();

    // Trigger rendering if not yet cached
    if pdf_state.is_none() {
        let chart_id = chart_id.clone();
        let url = chart.runtime_url();
        state.write().pdf_cache.insert(chart_id.clone(), PdfState::Loading);
        spawn(async move {
            match crate::pdf::fetch_and_render(&url).await {
                Ok(pages) => {
                    state.write().pdf_cache.insert(chart_id, PdfState::Rendered(pages));
                }
                Err(e) => {
                    state.write().pdf_cache.insert(chart_id, PdfState::Error(e));
                }
            }
        });
    }

    // Zoom levels: 50% → 200%, step 25%
    match pdf_state {
        Some(PdfState::Rendered(pages)) => {
            let z = zoom();
            let scale = z as f64 / 100.0;
            let img_width = format!("{}px", (BASE_IMG_WIDTH * scale) as i32);
            rsx! {
                div { class: "doc-viewer pdf-active",
                    DocToolbar { chart: chart.clone() }
                    div {
                        class: "pdf-scroll",
                        id: "pdf-scroll",
                        onmounted: move |_| {
                            spawn(async move {
                                let _ = document::eval(r#"
                                    const el = document.getElementById('pdf-scroll');
                                    if (el && !el._pan) {
                                        el._pan = true;
                                        let dragging = false, startX = 0, startY = 0, scrollL = 0, scrollT = 0;
                                        el.addEventListener('mousedown', e => {
                                            dragging = true;
                                            startX = e.clientX;
                                            startY = e.clientY;
                                            scrollL = el.scrollLeft;
                                            scrollT = el.scrollTop;
                                            el.classList.add('grabbing');
                                            e.preventDefault();
                                        });
                                        window.addEventListener('mousemove', e => {
                                            if (!dragging) return;
                                            el.scrollLeft = scrollL - (e.clientX - startX);
                                            el.scrollTop = scrollT - (e.clientY - startY);
                                        });
                                        window.addEventListener('mouseup', () => {
                                            dragging = false;
                                            el.classList.remove('grabbing');
                                        });
                                    }
                                "#).await;
                            });
                        },
                        div { class: "pdf-scroll-inner",
                            for page in &pages {
                                div { class: "pdf-page-container",
                                    div { class: "pdf-page-number", "Page {page.index + 1}" }
                                    img {
                                        class: "pdf-page-img",
                                        style: "width: {img_width}",
                                        src: "{page.data_url}",
                                        alt: "Page {page.index + 1}",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(PdfState::Error(msg)) => {
            rsx! {
                div { class: "doc-viewer",
                    DocToolbar { chart: chart.clone() }
                    div { class: "empty-state",
                        div { class: "icon", "⚠" }
                        p { "{tr(lang, \"doc.error\")}" }
                        p { style: "font-size: 12px; margin-top: 8px; color: var(--text-secondary);",
                            "{msg}"
                        }
                    }
                }
            }
        }
        _ => {
            rsx! {
                div { class: "doc-viewer",
                    DocToolbar { chart: chart.clone() }
                    div { class: "empty-state",
                        div { class: "loading-spinner" }
                        p { style: "margin-top: 16px;", "{tr(lang, \"doc.loading\")}" }
                    }
                }
            }
        }
    }
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

async fn editor_set_html(editor_id: &str, html: &str) {
    let js = format!(
        "const el = document.getElementById('{}'); if (el) {{ el.innerHTML = {}; }}",
        editor_id,
        js_string(html)
    );
    let _ = document::eval(&js).await;
}

async fn editor_get_html(editor_id: &str) -> Option<String> {
    let js = format!(
        "const el = document.getElementById('{}'); if (el) {{ return el.innerHTML; }} return '';",
        editor_id
    );
    document::eval(&js)
        .await
        .ok()
        .map(|v| v.as_str().unwrap_or("").to_string())
}

async fn editor_exec_command(editor_id: &str, cmd: &str, value: Option<&str>) {
    let cmd_js = if let Some(v) = value {
        format!("document.execCommand('{}', false, {});", cmd, js_string(v))
    } else {
        format!("document.execCommand('{}', false, null);", cmd)
    };
    let js = format!("document.getElementById('{}').focus(); {}", editor_id, cmd_js);
    let _ = document::eval(&js).await;
}

async fn editor_toggle_block(editor_id: &str, tag: &str) {
    let js = format!(
        "(function() {{ \
            var sel = window.getSelection(); \
            if (!sel.rangeCount) return; \
            var node = sel.anchorNode; \
            while (node && node.nodeType !== 1) node = node.parentNode; \
            while (node && node.id !== '{}') {{ \
                if (node.tagName && node.tagName.toLowerCase() === '{}') {{ \
                    document.execCommand('formatBlock', false, 'p'); \
                    return; \
                }} \
                node = node.parentNode; \
            }} \
            document.execCommand('formatBlock', false, '{}'); \
        }})()",
        editor_id, tag, tag
    );
    let _ = document::eval(&js).await;
}

#[component]
fn NotesEditor(pinned: bool) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let mut notes_dirty = use_signal(|| false);

    // Get workspace ID and current notes
    let ws_id = state.read().active_workspace_id.clone().unwrap_or_default();
    let initial_notes = state
        .read()
        .workspaces
        .iter()
        .find(|w| w.id == ws_id)
        .and_then(|w| w.notes.clone())
        .unwrap_or_default();

    // Use different element IDs so pinned and full-tab editors don't conflict
    let editor_id = if pinned { "notes-editor-pinned" } else { "notes-editor-content" };

    // Initialize the contenteditable editor with saved content
    use_effect({
        let initial_notes = initial_notes.clone();
        let editor_id = editor_id.to_string();
        move || {
            let html = initial_notes.clone();
            let editor_id = editor_id.clone();
            spawn(async move {
                editor_set_html(&editor_id, &html).await;
            });
        }
    });

    // Save notes via JS → get innerHTML → persist
    let save_notes = {
        let ws_id = ws_id.clone();
        let editor_id = editor_id.to_string();
        let notes_dirty = notes_dirty.clone();
        move |_: FocusEvent| {
            let ws_id = ws_id.clone();
            let editor_id = editor_id.clone();
            let mut notes_dirty = notes_dirty.clone();
            spawn(async move {
                if let Some(html) = editor_get_html(&editor_id).await {
                    let notes = if html.is_empty() || html == "<br>" { None } else { Some(html) };
                    {
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::workspaces::save_notes(
                            &conn,
                            &ws_id,
                            notes.as_deref(),
                        );
                    }
                    // Update in-memory workspace
                    let mut s = state.write();
                    if let Some(ws) = s.workspaces.iter_mut().find(|w| w.id == ws_id) {
                        ws.notes = notes;
                    }
                    notes_dirty.set(false);
                }
            });
        }
    };

    // Pull latest notes when editor gets focus (cross-window synchronization point)
    let refresh_notes = {
        let ws_id = ws_id.clone();
        let editor_id = editor_id.to_string();
        let notes_dirty = notes_dirty.clone();
        move |_: FocusEvent| {
            let ws_id = ws_id.clone();
            let editor_id = editor_id.clone();
            let notes_dirty = notes_dirty.clone();
            spawn(async move {
                if notes_dirty() {
                    return;
                }
                let current = editor_get_html(&editor_id).await.unwrap_or_default();
                let latest = {
                    let conn = crate::persistence::db().lock().unwrap();
                    crate::persistence::workspaces::list_workspaces(&conn)
                        .into_iter()
                        .find(|w| w.id == ws_id)
                        .and_then(|w| w.notes)
                        .unwrap_or_default()
                };
                if !current.is_empty() && latest.is_empty() {
                    return;
                }
                editor_set_html(&editor_id, &latest).await;
            });
        }
    };

    rsx! {
        div { class: "notes-container",
            if !pinned {
                NotesToolbar { pinned: false, editor_id: editor_id.to_string() }
            }
            div {
                class: "notes-editor-scroll",
                div {
                    class: "notes-editor-content",
                    id: "{editor_id}",
                    "data-placeholder": "{tr(lang, \"notes.placeholder\")}",
                    contenteditable: "true",
                    onfocus: refresh_notes,
                    onblur: save_notes,
                    oninput: move |_| notes_dirty.set(true),
                    onkeyup: move |_| notes_dirty.set(true),
                }
            }
        }
    }
}

#[component]
fn NotesToolbar(pinned: bool, editor_id: String) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let show_fg_color = use_signal(|| false);
    let show_bg_color = use_signal(|| false);

    let exec = {
        let editor_id = editor_id.clone();
        move |cmd: &str, value: Option<&str>| {
            let cmd = cmd.to_string();
            let value = value.map(|v| v.to_string());
            let editor_id = editor_id.clone();
            move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let cmd = cmd.clone();
                let value = value.clone();
                let editor_id = editor_id.clone();
                spawn(async move {
                    editor_exec_command(&editor_id, &cmd, value.as_deref()).await;
                });
            }
        }
    };

    // Toggle block format: if already in that tag, revert to <p>
    let toggle_block = {
        let editor_id = editor_id.clone();
        move |tag: &str| {
            let tag = tag.to_string();
            let editor_id = editor_id.clone();
            move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let tag = tag.clone();
                let editor_id = editor_id.clone();
                spawn(async move {
                    editor_toggle_block(&editor_id, &tag).await;
                });
            }
        }
    };

    rsx! {
        div { class: "notes-toolbar",
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h1\")}",
                onmousedown: toggle_block("h1"),
                "H1"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h2\")}",
                onmousedown: toggle_block("h2"),
                "H2"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h3\")}",
                onmousedown: toggle_block("h3"),
                "H3"
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.bold\")}",
                onmousedown: exec("bold", None),
                "B"
            }
            button {
                class: "notes-tool-btn notes-tool-italic",
                title: "{tr(lang, \"notes.italic\")}",
                onmousedown: exec("italic", None),
                "I"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.underline\")}",
                onmousedown: exec("underline", None),
                "U"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.strike\")}",
                onmousedown: exec("strikeThrough", None),
                "S̶"
            }
            span { class: "notes-tool-sep" }
            NotesColorPicker {
                editor_id: editor_id.clone(),
                picker_kind: "fg".to_string(),
                is_open: show_fg_color,
                other_open: show_bg_color,
            }
            NotesColorPicker {
                editor_id: editor_id.clone(),
                picker_kind: "bg".to_string(),
                is_open: show_bg_color,
                other_open: show_fg_color,
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.bullets\")}",
                onmousedown: exec("insertUnorderedList", None),
                "• —"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.numbered\")}",
                onmousedown: exec("insertOrderedList", None),
                "1."
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.quote\")}",
                onmousedown: toggle_block("blockquote"),
                "❝"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.code\")}",
                onmousedown: toggle_block("pre"),
                "<>"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.rule\")}",
                onmousedown: exec("insertHorizontalRule", None),
                "―"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.paragraph\")}",
                onmousedown: toggle_block("p"),
                "¶"
            }
            span { class: "notes-tool-sep notes-tool-spacer" }
            button {
                class: if pinned { "notes-tool-btn notes-pin-btn active" } else { "notes-tool-btn notes-pin-btn" },
                title: if pinned { tr(lang, "notes.unpin") } else { tr(lang, "notes.pin") },
                onclick: move |_| {
                    let mut s = state.write();
                    let now_pinned = !s.notes_pinned;
                    s.notes_pinned = now_pinned;
                    let ws_id = s.active_workspace_id.clone();
                    let is_popout = s.is_popout;
                    if now_pinned {
                        // Switch to first chart tab when pinning
                        if let Some(idx) = s.tabs.iter().position(|t| !t.is_notes()) {
                            s.active_tab = Some(idx);
                        }
                    }
                    drop(s);
                    if !is_popout {
                        if let Some(ws_id) = ws_id {
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::workspaces::save_notes_pinned(&conn, &ws_id, now_pinned);
                        if let Some(ws) = state
                            .write()
                            .workspaces
                            .iter_mut()
                            .find(|w| w.id == ws_id)
                        {
                            ws.notes_pinned = Some(now_pinned);
                        }
                    }
                    }
                },
                "📌"
            }
        }
    }
}

#[component]
fn NotesColorPicker(
    editor_id: String,
    picker_kind: String,
    mut is_open: Signal<bool>,
    mut other_open: Signal<bool>,
) -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let is_fg = picker_kind == "fg";
    let title = if is_fg { tr(lang, "notes.text_color") } else { tr(lang, "notes.highlight") };
    let exec_cmd = if is_fg { "foreColor" } else { "hiliteColor" };

    let colors: Vec<(&str, &str)> = if is_fg {
        vec![
            ("#1C1917", "Noir"),
            ("#DC2626", "Rouge"),
            ("#D97706", "Ambre"),
            ("#16A34A", "Vert"),
            ("#2563EB", "Bleu"),
            ("#7C3AED", "Violet"),
            ("#94A3B8", "Gris"),
        ]
    } else {
        vec![
            ("transparent", tr(lang, "notes.none")),
            ("#FEF3C7", "Jaune"),
            ("#FEE2E2", "Rouge"),
            ("#DCFCE7", "Vert"),
            ("#DBEAFE", "Bleu"),
            ("#F3E8FF", "Violet"),
            ("#F1F5F9", "Gris"),
        ]
    };

    rsx! {
        div { class: "notes-tool-dropdown-wrapper",
            button {
                class: "notes-tool-btn",
                title: "{title}",
                onmousedown: move |e| {
                    e.prevent_default();
                    e.stop_propagation();
                    is_open.toggle();
                    other_open.set(false);
                },
                if is_fg {
                    span { class: "notes-tool-color-icon",
                        "A"
                        span { class: "notes-tool-color-bar", style: "background: var(--amber);" }
                    }
                } else {
                    span { class: "notes-tool-color-icon", "🖍" }
                }
            }
            if is_open() {
                div { class: "notes-color-palette",
                    for (color, label) in colors {
                        {
                            let color = color.to_string();
                            let label = label.to_string();
                            let editor_id = editor_id.clone();
                            let exec_cmd = exec_cmd.to_string();
                            let is_remove = !is_fg && color == "transparent";
                            rsx! {
                                button {
                                    class: if is_remove { "notes-color-swatch notes-color-remove" } else { "notes-color-swatch" },
                                    title: "{label}",
                                    style: if !is_remove { format!("background: {};", color) } else { String::new() },
                                    onmousedown: move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let editor_id = editor_id.clone();
                                        let exec_cmd = exec_cmd.clone();
                                        let color = color.clone();
                                        spawn(async move {
                                            if is_remove {
                                                editor_exec_command(&editor_id, "removeFormat", None).await;
                                            } else {
                                                editor_exec_command(&editor_id, &exec_cmd, Some(&color)).await;
                                            }
                                        });
                                        is_open.set(false);
                                    },
                                    if is_remove { "✕" }
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
fn EmptyState() -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    rsx! {
        div { class: "doc-viewer",
            div { class: "empty-state",
                div { class: "icon", "✈" }
                p { "{tr(lang, \"empty.start\")}" }
                p { style: "font-size: 12px; margin-top: 8px; color: var(--text-secondary);",
                    "{tr(lang, \"empty.hint\")}" 
                }
            }
        }
    }
}
