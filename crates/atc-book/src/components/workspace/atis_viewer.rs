use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::{Duration, Instant};

use crate::adapters::atis_guru::{AtisData, fetch_atis, fetch_atis_with_cache_policy};
use crate::i18n::tr;
use crate::state::AppState;
use super::viewer_header::ViewerHeader;

const ATIS_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Copy, PartialEq, Eq)]
struct RefreshRequest {
    seq: u64,
    force: bool,
}

fn request_atis_fetch(
    icao: String,
    force_refresh: bool,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut data: Signal<Option<AtisData>>,
    mut last_refresh_at: Signal<Option<Instant>>,
    mut timer_generation: Signal<u64>,
    refresh_request: Signal<RefreshRequest>,
) {
    loading.set(true);
    error.set(None);

    spawn(async move {
        let fetch_result = if force_refresh {
            fetch_atis_with_cache_policy(&icao, true).await
        } else {
            fetch_atis(&icao).await
        };

        match fetch_result {
            Ok(d) => {
                data.set(Some(d));
                last_refresh_at.set(Some(Instant::now()));
                loading.set(false);
            }
            Err(e) => {
                error.set(Some(e.to_string()));
                loading.set(false);
            }
        }

        let next_generation = timer_generation() + 1;
        timer_generation.set(next_generation);

        let timer_generation_for_task = timer_generation.clone();
        let mut refresh_request_for_task = refresh_request.clone();
        spawn(async move {
            Delay::new(ATIS_REFRESH_INTERVAL).await;
            if timer_generation_for_task() == next_generation {
                refresh_request_for_task.with_mut(|request| {
                    request.seq += 1;
                    request.force = false;
                });
            }
        });
    });
}

#[component]
pub(super) fn AtisViewer(icao: String) -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;

    let loading = use_signal(|| true);
    let error: Signal<Option<String>> = use_signal(|| None);
    let data: Signal<Option<AtisData>> = use_signal(|| None);
    let mut refresh_request = use_signal(|| RefreshRequest {
        seq: 0,
        force: false,
    });
    let timer_generation = use_signal(|| 0u64);
    let last_refresh_at: Signal<Option<Instant>> = use_signal(|| None);

    use_effect({
        let icao = icao.clone();
        let refresh_request_for_effect = refresh_request.clone();
        let loading = loading.clone();
        let error = error.clone();
        let data = data.clone();
        let last_refresh_at = last_refresh_at.clone();
        let timer_generation = timer_generation.clone();
        let refresh_request = refresh_request.clone();
        move || {
            let request = refresh_request_for_effect();
            request_atis_fetch(
                icao.clone(),
                request.force,
                loading.clone(),
                error.clone(),
                data.clone(),
                last_refresh_at.clone(),
                timer_generation.clone(),
                refresh_request.clone(),
            );
        }
    });

    let refresh_status = if loading() {
        tr(lang, "atis.refreshing").to_string()
    } else if let Some(last) = last_refresh_at() {
        let elapsed_minutes = last.elapsed().as_secs() / 60;
        if elapsed_minutes == 0 {
            tr(lang, "atis.updated.just_now").to_string()
        } else {
            format!("{} {} min", tr(lang, "atis.updated"), elapsed_minutes)
        }
    } else {
        tr(lang, "atis.updated.pending").to_string()
    };

    rsx! {
        div { class: "atis-viewer",
            ViewerHeader {
                div { class: "atis-header-row",
                    div { class: "atis-header-meta",
                        span { class: "atis-header-title", "ATIS / MET" }
                        span { class: "separator", "|" }
                        span { class: "atis-header-icao", "{icao.to_uppercase()}" }
                    }
                    div { class: "atis-header-actions",
                        span { class: "atis-refresh-status", "{refresh_status}" }
                        button {
                            class: "toolbar-btn atis-refresh-btn",
                            disabled: loading(),
                            onclick: move |_| {
                                refresh_request.with_mut(|request| {
                                    request.seq += 1;
                                    request.force = true;
                                });
                            },
                            "{tr(lang, \"atis.refresh\")}"
                        }
                    }
                }
            }
            div { class: "atis-body",
                if loading() {
                    div { class: "atis-loading", "Chargement ATIS..." }
                } else if let Some(err) = error() {
                    div { class: "atis-error", "Erreur : {err}" }
                } else if let Some(d) = data() {
                    div { class: "atis-content",
                        h2 { class: "atis-title", "ATIS / MET - {d.icao}" }

                        if let Some(arr) = &d.arrival {
                            div { class: "atis-card",
                                div { class: "atis-card-header",
                                    span { class: "atis-card-title", "{arr.title}" }
                                    if let Some(ts) = &arr.timestamp {
                                        span { class: "atis-card-ts", "{ts}" }
                                    }
                                }
                                pre { class: "atis-text", "{arr.content}" }
                            }
                        }

                        if let Some(dep) = &d.departure {
                            div { class: "atis-card",
                                div { class: "atis-card-header",
                                    span { class: "atis-card-title", "{dep.title}" }
                                    if let Some(ts) = &dep.timestamp {
                                        span { class: "atis-card-ts", "{ts}" }
                                    }
                                }
                                pre { class: "atis-text", "{dep.content}" }
                            }
                        }

                        if let Some(metar) = &d.metar {
                            div { class: "atis-card",
                                div { class: "atis-card-header",
                                    span { class: "atis-card-title", "METAR" }
                                }
                                pre { class: "atis-text atis-metar", "{metar}" }
                            }
                        }

                        if let Some(taf) = &d.taf {
                            div { class: "atis-card",
                                div { class: "atis-card-header",
                                    span { class: "atis-card-title", "TAF" }
                                }
                                pre { class: "atis-text atis-taf", "{taf}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
