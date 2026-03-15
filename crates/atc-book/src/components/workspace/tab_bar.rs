use dioxus::prelude::*;

use crate::application::tabs as tab_usecases;
use crate::state::AppState;

#[component]
pub(super) fn TabBar() -> Element {
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
                                            tab_usecases::close_tab_by_id(&mut s, &tab_id);
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
