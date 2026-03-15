use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::Duration;

#[path = "workspace/aip_doc_viewer.rs"]
mod aip_doc_viewer;
#[path = "workspace/atis_viewer.rs"]
mod atis_viewer;
#[path = "workspace/notes_editor.rs"]
mod notes_editor;
#[path = "workspace/tab_actions.rs"]
mod tab_actions;
#[path = "workspace/document_view.rs"]
mod document_view;
#[path = "workspace/empty_state.rs"]
mod empty_state;
#[path = "workspace/tab_bar.rs"]
mod tab_bar;
#[path = "workspace/viewer_header.rs"]
mod viewer_header;

use crate::application::tabs as tab_usecases;
use crate::i18n::tr;
use crate::state::{AppState, TabContent};
use aip_doc_viewer::AipDocViewer;
use atis_viewer::AtisViewer;
use document_view::{DocMeta, DocViewer};
use empty_state::EmptyState;
use notes_editor::NotesEditor;
use tab_bar::TabBar;

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
            let (tab_ids, active) = tab_usecases::chart_tab_state(&s.tabs, s.active_tab);
            // Keep workspace-selected extra tabs (ATIS/eAIP) stable.
            // They are managed by explicit + / remove actions in Navigator.
            let extra_tabs: Vec<crate::models::ExtraTab> = s
                .workspaces
                .iter()
                .find(|w| w.id == *ws_id)
                .map(|w| w.extra_tabs.clone())
                .unwrap_or_default();
            let ws_id = ws_id.clone();
            let conn = crate::persistence::db().lock().unwrap();
            if s.is_popout {
                crate::persistence::workspaces::save_popout_tab_state(
                    &conn, &ws_id, &tab_ids, active,
                );
            } else {
                crate::persistence::workspaces::save_tab_state(
                    &conn, &ws_id, &tab_ids, active, &extra_tabs,
                );
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
                let (pop_tabs, _) =
                    crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);
                if !pop_tabs.is_empty() {
                    continue;
                }
                // Reload main tabs from persisted state
                let workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                if let Some(ws) = workspaces.iter().find(|w| w.id == ws_id) {
                    let open_tabs = ws.open_tabs.clone();
                    let active_tab_index = ws.active_tab_index;
                    let chart_refs = ws.chart_refs.clone();
                    let extra_tabs = ws.extra_tabs.clone();
                    s.workspaces = workspaces;
                    tab_usecases::rebuild_workspace_tabs(
                        &mut s,
                        &chart_refs,
                        &open_tabs,
                        active_tab_index,
                        &extra_tabs,
                    );
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
                                TabContent::AipDoc { doc } => rsx! {
                                    AipDocViewer {
                                        key: "{doc.id}",
                                        doc: doc.clone()
                                    }
                                },
                                TabContent::Atis { icao } => rsx! {
                                    AtisViewer {
                                        key: "atis_{icao}",
                                        icao: icao.clone()
                                    }
                                },
                                TabContent::Notes if !notes_pinned => rsx! {
                                    div { class: "notes-tab-view",
                                        NotesEditor { pinned: false }
                                    }
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
                                                tab_usecases::open_or_focus_chart(
                                                    &mut s,
                                                    chart.clone(),
                                                    airport.clone(),
                                                );
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

