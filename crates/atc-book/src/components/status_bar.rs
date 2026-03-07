use dioxus::prelude::*;

use crate::airac::AiracCycle;
use crate::i18n::tr;
use crate::state::AppState;

#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let airac = use_context::<Memo<AiracCycle>>();
    let cycle = airac.read();
    let active = cycle.is_active();

    let dot_cls = if active { "status-dot active" } else { "status-dot expired" };
    let label = if active { tr(lang, "status.active") } else { tr(lang, "status.expired") };

    rsx! {
        div { class: "status-bar",
            div { class: "status-item",
                span { class: "{dot_cls}" }
                "AIRAC {cycle.code} ({label})"
            }
            div { class: "status-item",
                "{tr(lang, \"status.network\")}"
            }
        }
    }
}
