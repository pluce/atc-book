use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use dioxus::desktop::tao::window::{WindowBuilder, WindowId};
use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::Duration;

use crate::airac::AiracCycle;
use crate::adapters::workspace_repository_sqlite::SqliteWorkspaceRepository;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::application::tabs as tab_usecases;
use crate::application::workspace_windows;
use crate::i18n::get_language;
use crate::state::{AppState, Tab};

use super::workspace::Workspace;
use dioxus::desktop::tao::event::Event;
use dioxus::desktop::use_wry_event_handler;
use dioxus::desktop::WindowEvent;

const MAIN_CSS: &str = include_str!("../../assets/main.css");

#[derive(Clone)]
struct WorkspacePopoutMeta {
    window_id: WindowId,
    title: String,
}

#[derive(Clone)]
struct StandalonePopoutMeta {
    window_id: Option<WindowId>,
    title: String,
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    revision: u64,
}

static OPEN_POPOUTS: OnceLock<Mutex<HashMap<String, WorkspacePopoutMeta>>> = OnceLock::new();
static OPEN_STANDALONE_POPOUTS: OnceLock<Mutex<HashMap<String, StandalonePopoutMeta>>> =
    OnceLock::new();
static NEXT_POPOUT_WINDOW_NUMBER: AtomicUsize = AtomicUsize::new(2);
static NEXT_STANDALONE_ID: AtomicUsize = AtomicUsize::new(1);

fn popout_map() -> &'static Mutex<HashMap<String, WorkspacePopoutMeta>> {
    OPEN_POPOUTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn standalone_popout_map() -> &'static Mutex<HashMap<String, StandalonePopoutMeta>> {
    OPEN_STANDALONE_POPOUTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_popout_title() -> String {
    let n = NEXT_POPOUT_WINDOW_NUMBER.fetch_add(1, Ordering::Relaxed);
    format!("ATC-BOOK // Window #{n}")
}

pub fn list_open_popout_targets() -> Vec<(String, String)> {
    let mut out = Vec::new();
    {
        let guard = popout_map().lock().unwrap();
        for (workspace_id, meta) in guard.iter() {
            out.push((
                format!("workspace:{workspace_id}"),
                meta.title.clone(),
            ));
        }
    }
    {
        let guard = standalone_popout_map().lock().unwrap();
        for (standalone_id, meta) in guard.iter() {
            if meta.window_id.is_some() {
                out.push((
                    format!("standalone:{standalone_id}"),
                    meta.title.clone(),
                ));
            }
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

pub fn focus_popout_target(
    _desktop: &dioxus::desktop::DesktopContext,
    target_key: &str,
) -> bool {
    if let Some(workspace_id) = target_key.strip_prefix("workspace:") {
        let guard = popout_map().lock().unwrap();
        return guard.contains_key(workspace_id);
    }
    if let Some(standalone_id) = target_key.strip_prefix("standalone:") {
        let guard = standalone_popout_map().lock().unwrap();
        if let Some(meta) = guard.get(standalone_id) {
            return meta.window_id.is_some();
        }
    }
    false
}

pub fn push_tab_to_standalone_popout(standalone_id: &str, tab: Tab) -> bool {
    let mut guard = standalone_popout_map().lock().unwrap();
    if let Some(meta) = guard.get_mut(standalone_id) {
        if let Some(idx) = meta.tabs.iter().position(|t| t.id == tab.id) {
            meta.active_tab = Some(idx);
        } else {
            meta.tabs.push(tab);
            meta.active_tab = Some(meta.tabs.len().saturating_sub(1));
        }
        meta.revision = meta.revision.saturating_add(1);
        true
    } else {
        false
    }
}

fn standalone_snapshot(standalone_id: &str) -> Option<(Vec<Tab>, Option<usize>, u64)> {
    let guard = standalone_popout_map().lock().unwrap();
    guard
        .get(standalone_id)
        .map(|m| (m.tabs.clone(), m.active_tab, m.revision))
}

#[component]
pub fn PopoutRoot(
    workspace_id: Option<String>,
    standalone_popout_id: Option<String>,
    initial_tabs: Option<Vec<Tab>>,
    initial_active: Option<usize>,
) -> Element {
    let state = use_signal(|| {
        let mut app = AppState::default();
        app.theme_mode = crate::state::get_theme_mode();
        app.night_mode = crate::state::get_night_mode();
        app.language = get_language();
        app.is_popout = true;

        let conn = crate::persistence::db().lock().unwrap();
        let repo = SqliteWorkspaceRepository::new(&conn);
        app.workspaces = repo.list_workspaces();
        if let Some(ws_id) = workspace_id.clone() {
            app.active_workspace_id = Some(ws_id.clone());
            app.chart_zoom = crate::persistence::workspaces::load_chart_zoom(&conn, &ws_id);

            let (popout_tab_ids, popout_active) = repo.load_popout_tab_state(&ws_id);
            if let Some(chart_refs) = app
                .workspaces
                .iter()
                .find(|w| w.id == ws_id)
                .map(|w| w.chart_refs.clone())
            {
                tab_usecases::rebuild_popout_tabs(
                    &mut app,
                    &chart_refs,
                    &popout_tab_ids,
                    popout_active,
                );
            } else {
                app.tabs.push(Tab::notes());
                app.active_tab = Some(0);
            }
        } else if let Some(standalone_id) = standalone_popout_id.clone() {
            if let Some((tabs, active, _)) = standalone_snapshot(&standalone_id) {
                app.tabs = tabs;
                app.active_tab = active.or(Some(0));
            } else {
                app.tabs.push(Tab::notes());
                app.active_tab = Some(0);
            }
        } else if let Some(tabs) = initial_tabs.clone() {
            app.tabs = tabs;
            app.active_tab = initial_active.or(Some(0));
        } else {
            app.tabs.push(Tab::notes());
            app.active_tab = Some(0);
        }

        app
    });
    use_context_provider(|| state);

    let airac = use_memo(|| AiracCycle::current());
    use_context_provider(|| airac);

    let refresh_tabs = {
        let mut state = state.clone();
        let workspace_id = workspace_id.clone();
        move || {
            let workspace_id = match workspace_id.clone() {
                Some(id) => id,
                None => return,
            };
            let conn = crate::persistence::db().lock().unwrap();
            let repo = SqliteWorkspaceRepository::new(&conn);
            let (pop_tabs, pop_active) = repo.load_popout_tab_state(&workspace_id);
            let workspaces = repo.list_workspaces();
            let mut s = state.write();
            s.workspaces = workspaces;
            let chart_refs = s
                .workspaces
                .iter()
                .find(|w| w.id == workspace_id)
                .map(|w| w.chart_refs.clone())
                .unwrap_or_default();
            tab_usecases::rebuild_popout_tabs(&mut s, &chart_refs, &pop_tabs, pop_active);
        }
    };

    {
        let mut refresh_tabs = refresh_tabs.clone();
        let workspace_id_for_events = workspace_id.clone();
        let standalone_id_for_events = standalone_popout_id.clone();
        use_wry_event_handler(move |event, _| match event {
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                ..
            } => {
                refresh_tabs();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            } => {
                if let Some(workspace_id) = workspace_id_for_events.clone() {
                    let conn = crate::persistence::db().lock().unwrap();
                    let repo = SqliteWorkspaceRepository::new(&conn);
                    workspace_windows::merge_workspace_popout_tabs_into_main(&repo, &workspace_id);
                    popout_map().lock().unwrap().remove(&workspace_id);
                }
                if let Some(standalone_id) = standalone_id_for_events.clone() {
                    standalone_popout_map().lock().unwrap().remove(&standalone_id);
                }
            }
            _ => {}
        });
    }

    // Poll for changes so moves from the main window appear immediately.
    let poll_has_workspace = workspace_id.is_some();
    use_effect(move || {
        if !poll_has_workspace {
            return;
        }
        let mut refresh_tabs = refresh_tabs.clone();
        spawn(async move {
            loop {
                Delay::new(Duration::from_millis(500)).await;
                refresh_tabs();
            }
        });
    });

    // Poll for standalone popout updates so tabs can be pushed from another window.
    let poll_has_standalone = standalone_popout_id.is_some();
    use_effect(move || {
        if !poll_has_standalone {
            return;
        }
        let mut state = state.clone();
        let mut last_applied_revision = 0u64;
        let standalone_id = match standalone_popout_id.clone() {
            Some(id) => id,
            None => return,
        };
        spawn(async move {
            loop {
                Delay::new(Duration::from_millis(350)).await;
                if let Some((tabs, active, revision)) = standalone_snapshot(&standalone_id) {
                    if revision <= last_applied_revision {
                        continue;
                    }
                    last_applied_revision = revision;
                    let mut s = state.write();
                    s.tabs = tabs;
                    s.active_tab = active.or(Some(0));
                }
            }
        });
    });

    use_effect(move || {
        let mut state = state.clone();
        spawn(async move {
            loop {
                Delay::new(Duration::from_millis(1000)).await;
                let global_theme_mode = crate::state::get_theme_mode();
                let global_night = crate::state::get_night_mode();
                let global_lang = get_language();
                let mut s = state.write();
                if s.theme_mode != global_theme_mode {
                    s.theme_mode = global_theme_mode;
                }
                if s.night_mode != global_night {
                    s.night_mode = global_night;
                }
                if s.language != global_lang {
                    s.language = global_lang;
                }
            }
        });
    });

    rsx! {
        document::Style { {MAIN_CSS} }
        div { class: if state.read().night_mode { "app theme-night" } else { "app theme-day" },
            div { class: "app-body",
                Workspace {}
            }
        }
    }
}

pub async fn open_workspace_popout(workspace_id: String) {
    let title = next_popout_title();
    let dom = VirtualDom::new_with_props(
        PopoutRoot,
        PopoutRootProps {
            workspace_id: Some(workspace_id.clone()),
            standalone_popout_id: None,
            initial_tabs: None,
            initial_active: None,
        },
    );
    let cfg = dioxus::desktop::Config::new().with_window(
        WindowBuilder::new()
            .with_title(title.clone())
            .with_visible(true)
            .with_focused(true)
            .with_always_on_top(false),
    );

    let ctx = dioxus::desktop::window().new_window(dom, cfg).await;
    popout_map().lock().unwrap().remove(&workspace_id);
    popout_map().lock().unwrap().insert(
        workspace_id,
        WorkspacePopoutMeta {
            window_id: ctx.id(),
            title,
        },
    );
}

pub async fn open_tab_popout(tab: Tab) {
    let title = next_popout_title();
    let standalone_id = format!("sp-{}", NEXT_STANDALONE_ID.fetch_add(1, Ordering::Relaxed));
    let (tabs, active_tab) = if tab.is_notes() {
        (vec![Tab::notes()], Some(0))
    } else {
        (vec![Tab::notes(), tab], Some(1))
    };

    standalone_popout_map().lock().unwrap().insert(
        standalone_id.clone(),
        StandalonePopoutMeta {
            window_id: None,
            title: title.clone(),
            tabs,
            active_tab,
            revision: 1,
        },
    );

    let dom = VirtualDom::new_with_props(
        PopoutRoot,
        PopoutRootProps {
            workspace_id: None,
            standalone_popout_id: Some(standalone_id.clone()),
            initial_tabs: None,
            initial_active: None,
        },
    );
    let cfg = dioxus::desktop::Config::new().with_window(
        WindowBuilder::new()
            .with_title(title)
            .with_visible(true)
            .with_focused(true)
            .with_always_on_top(false),
    );

    let ctx = dioxus::desktop::window().new_window(dom, cfg).await;
    if let Some(meta) = standalone_popout_map().lock().unwrap().get_mut(&standalone_id) {
        meta.window_id = Some(ctx.id());
    }
}

pub fn close_workspace_popout(desktop: &dioxus::desktop::DesktopContext, workspace_id: &str) {
    if let Some(meta) = popout_map().lock().unwrap().remove(workspace_id) {
        desktop.close_window(meta.window_id);
    }
}

pub fn close_all_popouts(desktop: &dioxus::desktop::DesktopContext) {
    let workspace_ids: Vec<WindowId> = {
        let mut guard = popout_map().lock().unwrap();
        let ids = guard.values().map(|m| m.window_id).collect::<Vec<_>>();
        guard.clear();
        ids
    };
    for id in workspace_ids {
        desktop.close_window(id);
    }

    let standalone_ids: Vec<WindowId> = {
        let mut guard = standalone_popout_map().lock().unwrap();
        let ids = guard
            .values()
            .filter_map(|m| m.window_id)
            .collect::<Vec<_>>();
        guard.clear();
        ids
    };
    for id in standalone_ids {
        desktop.close_window(id);
    }
}

pub fn has_workspace_popout(workspace_id: &str) -> bool {
    popout_map().lock().unwrap().contains_key(workspace_id)
}
