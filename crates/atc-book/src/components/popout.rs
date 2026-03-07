use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use dioxus::desktop::tao::window::{WindowBuilder, WindowId};
use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::Duration;

use crate::airac::AiracCycle;
use crate::i18n::get_language;
use crate::state::{AppState, Tab};

use super::workspace::Workspace;
use dioxus::desktop::use_wry_event_handler;
use dioxus::desktop::tao::event::Event;
use dioxus::desktop::WindowEvent;

const MAIN_CSS: &str = include_str!("../../assets/main.css");

static OPEN_POPOUTS: OnceLock<Mutex<HashMap<String, WindowId>>> = OnceLock::new();

fn popout_map() -> &'static Mutex<HashMap<String, WindowId>> {
    OPEN_POPOUTS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[component]
pub fn PopoutRoot(
    workspace_id: Option<String>,
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
        app.workspaces = crate::persistence::workspaces::list_workspaces(&conn);
        if let Some(ws_id) = workspace_id.clone() {
            app.active_workspace_id = Some(ws_id.clone());
            app.chart_zoom = crate::persistence::workspaces::load_chart_zoom(&conn, &ws_id);

            let (popout_tab_ids, popout_active) =
                crate::persistence::workspaces::load_popout_tab_state(&conn, &ws_id);

            app.tabs.push(Tab::notes());
            if let Some(ws) = app.workspaces.iter().find(|w| w.id == ws_id) {
                for tab_id in &popout_tab_ids {
                    if let Some(wc) = ws.chart_refs.iter().find(|c| c.chart.id == *tab_id) {
                        app.tabs.push(Tab::chart(wc.chart.clone(), wc.airport.clone()));
                    }
                }
            }

            app.active_tab = popout_active
                .map(|i| (i + 1).min(app.tabs.len().saturating_sub(1)))
                .or(Some(0));
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
            let (pop_tabs, pop_active) =
                crate::persistence::workspaces::load_popout_tab_state(&conn, &workspace_id);
            let workspaces = crate::persistence::workspaces::list_workspaces(&conn);
            let mut s = state.write();
            s.workspaces = workspaces;
            let chart_refs = s
                .workspaces
                .iter()
                .find(|w| w.id == workspace_id)
                .map(|w| w.chart_refs.clone())
                .unwrap_or_default();
            s.tabs.clear();
            s.tabs.push(Tab::notes());
            for tab_id in &pop_tabs {
                if let Some(wc) = chart_refs.iter().find(|c| c.chart.id == *tab_id) {
                    s.tabs.push(Tab::chart(wc.chart.clone(), wc.airport.clone()));
                }
            }
            s.active_tab = pop_active
                .map(|i| (i + 1).min(s.tabs.len().saturating_sub(1)))
                .or(Some(0));
        }
    };

    {
        let mut refresh_tabs = refresh_tabs.clone();
        let workspace_id_for_events = workspace_id.clone();
        use_wry_event_handler(move |event, _| {
            match event {
                Event::WindowEvent { event: WindowEvent::Focused(true), .. } => {
                    refresh_tabs();
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
                | Event::WindowEvent { event: WindowEvent::Destroyed, .. } => {
                    let workspace_id = match workspace_id_for_events.clone() {
                        Some(id) => id,
                        None => return,
                    };
                    let conn = crate::persistence::db().lock().unwrap();
                    let (pop_tabs, _) =
                        crate::persistence::workspaces::load_popout_tab_state(&conn, &workspace_id);
                    if let Some(ws) = crate::persistence::workspaces::list_workspaces(&conn)
                        .into_iter()
                        .find(|w| w.id == workspace_id)
                    {
                        let mut merged = ws.open_tabs.clone();
                        for id in pop_tabs {
                            if !merged.contains(&id) {
                                merged.push(id);
                            }
                        }
                        crate::persistence::workspaces::save_tab_state(
                            &conn,
                            &workspace_id,
                            &merged,
                            ws.active_tab_index,
                        );
                    }
                    crate::persistence::workspaces::save_popout_tab_state(
                        &conn,
                        &workspace_id,
                        &[],
                        None,
                    );
                    popout_map().lock().unwrap().remove(&workspace_id);
                }
                _ => {}
            }
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
    let dom = VirtualDom::new_with_props(
        PopoutRoot,
        PopoutRootProps {
            workspace_id: Some(workspace_id.clone()),
            initial_tabs: None,
            initial_active: None,
        },
    );
    let cfg = dioxus::desktop::Config::new().with_window(
        WindowBuilder::new()
            .with_title("ATC-BOOK // Window #2")
            .with_visible(true)
            .with_focused(true)
            .with_always_on_top(false),
    );

    let ctx = dioxus::desktop::window().new_window(dom, cfg).await;
    popout_map()
        .lock()
        .unwrap()
        .remove(&workspace_id);
    popout_map()
        .lock()
        .unwrap()
        .insert(workspace_id, ctx.id());
}

pub async fn open_chart_popout(chart: crate::models::Chart, airport: String) {
    let tabs = vec![Tab::notes(), Tab::chart(chart, airport)];
    let dom = VirtualDom::new_with_props(
        PopoutRoot,
        PopoutRootProps {
            workspace_id: None,
            initial_tabs: Some(tabs),
            initial_active: Some(1),
        },
    );
    let cfg = dioxus::desktop::Config::new().with_window(
        WindowBuilder::new()
            .with_title("ATC-BOOK // Window #2")
            .with_visible(true)
            .with_focused(true)
            .with_always_on_top(false),
    );

    let _ = dioxus::desktop::window().new_window(dom, cfg).await;
}

pub fn close_workspace_popout(desktop: &dioxus::desktop::DesktopContext, workspace_id: &str) {
    if let Some(id) = popout_map().lock().unwrap().remove(workspace_id) {
        desktop.close_window(id);
    }
}

pub fn has_workspace_popout(workspace_id: &str) -> bool {
    popout_map().lock().unwrap().contains_key(workspace_id)
}
