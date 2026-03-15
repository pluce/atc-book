use dioxus::prelude::*;

use crate::i18n::tr;
use crate::state::{AppState, SidebarMode};

#[component]
pub fn NanoSidebar() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let mode = state.read().sidebar_mode;
    let lang = state.read().language;

    let mut toggle = move |target: SidebarMode| {
        let mut s = state.write();
        if s.sidebar_mode == target {
            s.nav_open = !s.nav_open;
        } else {
            s.sidebar_mode = target;
            s.nav_open = true;
        }
    };

    rsx! {
        div { class: "nano-sidebar",
            SidebarIcon {
                icon: "✈",
                icon_class: "icon icon-airport",
                label: tr(lang, "sidebar.airports"),
                active: mode == SidebarMode::Airports,
                onclick: move |_| toggle(SidebarMode::Airports),
            }
            SidebarIcon {
                icon: "📁",
                icon_class: "icon",
                label: tr(lang, "sidebar.workspaces"),
                active: mode == SidebarMode::Workspaces,
                onclick: move |_| toggle(SidebarMode::Workspaces),
            }
            SidebarIcon {
                icon: "⚙",
                icon_class: "icon icon-settings",
                label: tr(lang, "sidebar.settings"),
                active: mode == SidebarMode::Settings,
                onclick: move |_| toggle(SidebarMode::Settings),
            }
            SidebarIcon {
                icon: "❔",
                icon_class: "icon",
                label: tr(lang, "sidebar.help"),
                active: mode == SidebarMode::Help,
                onclick: move |_| toggle(SidebarMode::Help),
            }

            div { class: "sidebar-info",
                "AIRAC"
                br {}
                {
                    let airac = use_context::<Memo<crate::airac::AiracCycle>>();
                    rsx! { "{airac.read().code}" }
                }
            }
        }
    }
}

#[component]
fn SidebarIcon(
    icon: String,
    icon_class: String,
    label: String,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let cls = if active {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };
    rsx! {
        div {
            class: "{cls}",
            onclick: move |e| onclick.call(e),
            span { class: "{icon_class}", "{icon}" }
            "{label}"
        }
    }
}
