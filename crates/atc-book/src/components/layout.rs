use dioxus::prelude::*;
use dioxus::desktop::tao::event::Event;
use dioxus::desktop::use_wry_event_handler;
use dioxus::desktop::WindowEvent;
use futures_timer::Delay;
use std::time::Duration;

use crate::airac::AiracCycle;
use crate::i18n::{get_language, tr};
use crate::state::AppState;

use super::navigator::Navigator;
use super::sidebar::NanoSidebar;
use super::status_bar::StatusBar;
use super::workspace::Workspace;

#[component]
pub fn AppShell() -> Element {
    let desktop = dioxus::desktop::use_window();
    let mut restored_popouts = use_signal(|| false);

    {
        let desktop = desktop.clone();
        use_wry_event_handler(move |event, _| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            } => {
                crate::components::popout::close_all_popouts(&desktop);
            }
            _ => {}
        });
    }

    // Reopen persisted workspace popout windows when the app starts.
    use_effect(move || {
        if restored_popouts() {
            return;
        }
        restored_popouts.set(true);
        spawn(async move {
            let workspace_ids: Vec<String> = {
                let conn = crate::persistence::db().lock().unwrap();
                let workspaces = crate::persistence::workspaces::list_workspaces(&conn);
                workspaces
                    .iter()
                    .filter_map(|ws| {
                        let (tabs, _) = crate::persistence::workspaces::load_popout_tab_state(&conn, &ws.id);
                        if tabs.is_empty() {
                            None
                        } else {
                            Some(ws.id.clone())
                        }
                    })
                    .collect()
            };

            for ws_id in workspace_ids {
                if !crate::components::popout::has_workspace_popout(&ws_id) {
                    crate::components::popout::open_workspace_popout(ws_id).await;
                }
            }
        });
    });

    let mut state = use_signal(|| {
        // Load saved workspaces from DB at startup
        let mut app = AppState::default();
        let conn = crate::persistence::db().lock().unwrap();
        if let Some(lang) = crate::persistence::settings::load_language(&conn) {
            crate::i18n::set_language(lang);
        }
        if let Some(mode) = crate::persistence::settings::load_theme_mode(&conn) {
            crate::state::set_theme_mode(mode);
        }
        app.theme_mode = crate::state::get_theme_mode();
        app.night_mode = crate::state::get_night_mode();
        app.language = get_language();
        app.workspaces = crate::persistence::workspaces::list_workspaces(&conn);
        app
    });
    use_context_provider(|| state);

    let airac = use_memo(|| AiracCycle::current());
    use_context_provider(|| airac);

    use_effect(move || {
        let mut state = state.clone();
        spawn(async move {
            loop {
                Delay::new(Duration::from_millis(1000)).await;
                let global_lang = get_language();
                let global_theme_mode = crate::state::get_theme_mode();
                let global_night = crate::state::get_night_mode();
                let mut s = state.write();
                if s.language != global_lang {
                    s.language = global_lang;
                }
                if s.theme_mode != global_theme_mode {
                    s.theme_mode = global_theme_mode;
                }
                if s.night_mode != global_night {
                    s.night_mode = global_night;
                }
            }
        });
    });

    rsx! {
        div {
            class: if state.read().night_mode { "app theme-night" } else { "app theme-day" },
            tabindex: "0",
            onmounted: move |_| {
                spawn(async move {
                    let _ = document::eval(
                        "const el = document.querySelector('.app'); if (el) { el.focus(); }"
                    ).await;
                });
            },
            onkeydown: move |e: KeyboardEvent| {
                let mods = e.modifiers();
                if mods.ctrl() && mods.shift() {
                    if let Key::Character(c) = e.key() {
                        let key = c.to_ascii_lowercase();
                        if key == "n" {
                            let mut s = state.write();
                            let now_pinned = !s.notes_pinned;
                            s.notes_pinned = now_pinned;
                            let ws_id = s.active_workspace_id.clone();
                            let is_popout = s.is_popout;
                            if now_pinned {
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
                        } else if key == "b" {
                            let mut s = state.write();
                            s.nav_open = !s.nav_open;
                        }
                    }
                }
                if mods.ctrl() {
                    if let Key::Character(c) = e.key() {
                        let key = c.to_ascii_lowercase();
                        if key == "p" {
                            let mut s = state.write();
                            s.quick_switcher_open = true;
                            s.quick_switcher_query.clear();
                        }
                    }
                }
            },
            TitleBar {}
            div { class: "app-body",
                NanoSidebar {}
                Navigator {}
                Workspace {}
            }
            StatusBar {}
        }
    }
}

#[component]
fn TitleBar() -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    rsx! {
        div { class: "app-title-bar",
            div { class: "titlebar-left",
                span { "ATC-BOOK" }
                " // {tr(lang, \"title.app\")}"
            }
            div { class: "titlebar-actions" }
        }
    }
}
