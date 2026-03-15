use dioxus::prelude::*;

use super::tab_actions::TabActionsMenu;

#[component]
pub(super) fn ViewerHeader(children: Element) -> Element {
    rsx! {
        div { class: "viewer-header",
            div { class: "viewer-header-fixed",
                TabActionsMenu {}
            }
            div { class: "viewer-header-content", {children} }
        }
    }
}
