use dioxus::prelude::*;

use crate::i18n::tr;
use crate::state::AppState;

#[component]
pub(super) fn EmptyState() -> Element {
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
