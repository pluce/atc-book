use dioxus::prelude::*;

use crate::adapters::workspace_repository_sqlite::SqliteWorkspaceRepository;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::application::workspace_windows;
use crate::i18n::tr;
use crate::state::{AppState, TabContent};

fn add_chart_to_workspace_and_reload(
    state: &mut Signal<AppState>,
    workspace_id: &str,
    airport: &str,
    chart: &crate::models::Chart,
) {
    let conn = crate::persistence::db().lock().unwrap();
    let repo = SqliteWorkspaceRepository::new(&conn);
    repo.add_chart_to_workspace(workspace_id, airport, chart);
    state.write().workspaces = repo.list_workspaces();
}

fn add_extra_tab_to_workspace_and_reload(
    state: &mut Signal<AppState>,
    workspace_id: &str,
    tab: crate::models::ExtraTab,
) {
    let conn = crate::persistence::db().lock().unwrap();
    let repo = SqliteWorkspaceRepository::new(&conn);
    repo.add_extra_tab(workspace_id, &tab);
    state.write().workspaces = repo.list_workspaces();
}

#[component]
pub(super) fn TabActionsMenu() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let desktop = dioxus::desktop::use_window();
    let lang = state.read().language;
    let workspaces = state.read().workspaces.clone();
    let mut show_actions = use_signal(|| false);
    let mut show_workspace_menu = use_signal(|| false);
    let mut show_popout_menu = use_signal(|| false);
    let mut workspace_filter = use_signal(String::new);

    let active_tab = {
        let s = state.read();
        s.active_tab.and_then(|idx| s.tabs.get(idx).cloned())
    };

    let Some(tab) = active_tab else {
        return rsx! {};
    };

    let can_add_to_workspace = !matches!(tab.content, TabContent::Notes);
    let filter_value = workspace_filter().to_lowercase();
    let filtered_workspaces: Vec<&crate::models::Workspace> = workspaces
        .iter()
        .filter(|ws| filter_value.is_empty() || ws.name.to_lowercase().contains(&filter_value))
        .collect();

    let popout_targets = crate::components::popout::list_open_popout_targets();
    let supported_popout_targets: Vec<(String, String)> = popout_targets
        .into_iter()
        .filter(|(key, _)| {
            if key.starts_with("workspace:") {
                matches!(tab.content, TabContent::Chart { .. })
            } else {
                true
            }
        })
        .collect();
    let can_send_to_existing_popout = !supported_popout_targets.is_empty();

    rsx! {
        div {
            class: "viewer-actions-menu",
            div {
                class: if show_actions() { "toolbar-dropdown-wrapper menu-expanded" } else { "toolbar-dropdown-wrapper" },
                onmouseleave: move |_| {
                    show_actions.set(false);
                    show_workspace_menu.set(false);
                    show_popout_menu.set(false);
                    workspace_filter.set(String::new());
                },
                button {
                    class: "toolbar-btn toolbar-btn-primary",
                    title: "{tr(lang, \"menu.display\")}",
                    onclick: move |_| {
                        show_actions.toggle();
                        show_workspace_menu.set(false);
                        show_popout_menu.set(false);
                        if !show_actions() {
                            workspace_filter.set(String::new());
                        }
                    },
                    span { class: "icon toolbar-menu-icon", "☰" }
                }
                if show_actions() {
                    div { class: "toolbar-menu-panel",
                        div { class: "toolbar-menu-list",
                            div {
                                class: if can_add_to_workspace { "toolbar-menu-item" } else { "toolbar-menu-item disabled" },
                                onmouseenter: move |_| {
                                    show_popout_menu.set(false);
                                    if can_add_to_workspace {
                                        show_workspace_menu.set(true);
                                    } else {
                                        show_workspace_menu.set(false);
                                    }
                                },
                                "{tr(lang, \"menu.send_to_workspace\")}"
                                if can_add_to_workspace {
                                    span { class: "toolbar-menu-caret", ">" }
                                }
                            }
                            div {
                                class: if can_send_to_existing_popout { "toolbar-menu-item" } else { "toolbar-menu-item disabled" },
                                onmouseenter: move |_| {
                                    show_workspace_menu.set(false);
                                    show_popout_menu.set(can_send_to_existing_popout);
                                },
                                "{tr(lang, \"menu.popout_existing\")}"
                                if can_send_to_existing_popout {
                                    span { class: "toolbar-menu-caret", ">" }
                                }
                            }
                            div {
                                class: "toolbar-menu-item",
                                onmouseenter: move |_| {
                                    show_workspace_menu.set(false);
                                    show_popout_menu.set(false);
                                },
                                onclick: move |_| {
                                    let ws_id = state.read().active_workspace_id.clone();
                                    let active_tab = {
                                        let s = state.read();
                                        s.active_tab.and_then(|idx| s.tabs.get(idx).cloned())
                                    };
                                    if let Some(tab) = active_tab {
                                        match tab.content.clone() {
                                            TabContent::Chart { .. } => {
                                                if let Some(ws_id) = ws_id {
                                                    if crate::components::popout::has_workspace_popout(&ws_id) {
                                                        // "New window" should still create a dedicated popout.
                                                        let tab_for_popout = tab.clone();
                                                        let tab_id_for_close = tab.id.clone();
                                                        let mut state_for_close = state.clone();
                                                        spawn(async move {
                                                            crate::components::popout::open_tab_popout(tab_for_popout).await;
                                                            let mut s = state_for_close.write();
                                                            crate::application::tabs::close_tab_by_id(&mut s, &tab_id_for_close);
                                                        });
                                                    } else {
                                                        // No workspace popout exists yet: opening one is the natural first "new window".
                                                        let conn = crate::persistence::db().lock().unwrap();
                                                        let repo = SqliteWorkspaceRepository::new(&conn);
                                                        let mut s = state.write();
                                                        workspace_windows::move_chart_tab_to_workspace_popout(
                                                            &mut s,
                                                            &repo,
                                                            &ws_id,
                                                            &tab.id,
                                                        );
                                                        spawn(async move {
                                                            crate::components::popout::open_workspace_popout(ws_id).await;
                                                        });
                                                    }
                                                } else {
                                                    let tab_for_popout = tab.clone();
                                                    let tab_id_for_close = tab.id.clone();
                                                    let mut state_for_close = state.clone();
                                                    spawn(async move {
                                                        crate::components::popout::open_tab_popout(tab_for_popout).await;
                                                        let mut s = state_for_close.write();
                                                        crate::application::tabs::close_tab_by_id(&mut s, &tab_id_for_close);
                                                    });
                                                }
                                            }
                                            _ => {
                                                let tab_for_popout = tab.clone();
                                                let tab_id_for_close = tab.id.clone();
                                                let mut state_for_close = state.clone();
                                                spawn(async move {
                                                    crate::components::popout::open_tab_popout(tab_for_popout).await;
                                                    let mut s = state_for_close.write();
                                                    crate::application::tabs::close_tab_by_id(&mut s, &tab_id_for_close);
                                                });
                                            }
                                        }
                                    }
                                    show_actions.set(false);
                                },
                                "{tr(lang, \"menu.popout_new\")}"
                            }
                        }
                        if show_workspace_menu() {
                            div { class: "toolbar-submenu",
                                onmouseenter: move |_| { show_workspace_menu.set(true); },
                                input {
                                    class: "toolbar-search",
                                    r#type: "text",
                                    placeholder: "{tr(lang, \"menu.filter\")}",
                                    value: "{workspace_filter}",
                                    oninput: move |e| workspace_filter.set(e.value()),
                                }
                                if workspaces.is_empty() {
                                    div { class: "toolbar-menu-item disabled", "{tr(lang, \"menu.no_workspace\")}" }
                                }
                                for ws in filtered_workspaces.iter() {
                                    {
                                        let ws_id = ws.id.clone();
                                        let ws_name = ws.name.clone();
                                        let tab = tab.clone();
                                        rsx! {
                                            div {
                                                class: "toolbar-menu-item",
                                                onclick: move |_| {
                                                    match &tab.content {
                                                        TabContent::Chart { chart, airport } => {
                                                            add_chart_to_workspace_and_reload(
                                                                &mut state,
                                                                &ws_id,
                                                                airport,
                                                                chart,
                                                            );
                                                        }
                                                        TabContent::AipDoc { doc } => {
                                                            add_extra_tab_to_workspace_and_reload(
                                                                &mut state,
                                                                &ws_id,
                                                                crate::models::ExtraTab::AipDoc { doc: doc.clone() },
                                                            );
                                                        }
                                                        TabContent::Atis { icao } => {
                                                            add_extra_tab_to_workspace_and_reload(
                                                                &mut state,
                                                                &ws_id,
                                                                crate::models::ExtraTab::Atis { icao: icao.clone() },
                                                            );
                                                        }
                                                        TabContent::Notes => {}
                                                    }
                                                    show_actions.set(false);
                                                    show_workspace_menu.set(false);
                                                },
                                                "{ws_name}"
                                            }
                                        }
                                    }
                                }
                                if !workspaces.is_empty() && filtered_workspaces.is_empty() {
                                    div { class: "toolbar-menu-item disabled", "{tr(lang, \"menu.no_result\")}" }
                                }
                            }
                        } else if show_popout_menu() {
                            div { class: "toolbar-submenu",
                                onmouseenter: move |_| { show_popout_menu.set(true); },
                                if supported_popout_targets.is_empty() {
                                    div { class: "toolbar-menu-item disabled", "{tr(lang, \"menu.none\")}" }
                                }
                                for (target_key, target_label) in supported_popout_targets.iter() {
                                    {
                                        let target_key = target_key.clone();
                                        let target_label = target_label.clone();
                                        let tab = tab.clone();
                                        let desktop = desktop.clone();
                                        rsx! {
                                            div {
                                                class: "toolbar-menu-item",
                                                onclick: move |_| {
                                                    if let Some(workspace_id) = target_key.strip_prefix("workspace:") {
                                                        if let TabContent::Chart { .. } = tab.content.clone() {
                                                            let conn = crate::persistence::db().lock().unwrap();
                                                            let repo = SqliteWorkspaceRepository::new(&conn);
                                                            let mut s = state.write();
                                                            workspace_windows::move_chart_tab_to_workspace_popout(
                                                                &mut s,
                                                                &repo,
                                                                workspace_id,
                                                                &tab.id,
                                                            );
                                                            crate::components::popout::focus_popout_target(&desktop, &target_key);
                                                        }
                                                    } else if let Some(standalone_id) = target_key.strip_prefix("standalone:") {
                                                        if crate::components::popout::push_tab_to_standalone_popout(standalone_id, tab.clone()) {
                                                            let mut s = state.write();
                                                            crate::application::tabs::close_tab_by_id(&mut s, &tab.id);
                                                            crate::components::popout::focus_popout_target(&desktop, &target_key);
                                                        }
                                                    }
                                                    show_actions.set(false);
                                                    show_popout_menu.set(false);
                                                    show_workspace_menu.set(false);
                                                },
                                                "{target_label}"
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
}
