use dioxus::prelude::*;

use super::viewer_header::ViewerHeader;
use crate::i18n::tr;
use crate::state::{AppState, PdfState};

/// Base image display width in pixels (at 100% zoom).
const BASE_IMG_WIDTH: f64 = 900.0;

#[component]
pub(super) fn DocMeta(chart: crate::models::Chart, mut zoom: Signal<u32>) -> Element {
    let state = use_context::<Signal<AppState>>();
    let airac = use_context::<Memo<crate::airac::AiracCycle>>();
    let z = zoom();
    let chart_id_minus = chart.id.clone();
    let chart_id_plus = chart.id.clone();
    let chart_id_fit = chart.id.clone();
    let state_minus = state.clone();
    let state_plus = state.clone();
    let state_fit = state.clone();

    rsx! {
        ViewerHeader {
            div { class: "doc-meta",
                div { class: "doc-meta-left",
                    "TYPE: {chart.source:?}"
                    span { class: "separator", " | " }
                    "REF: {chart.id}"
                    span { class: "separator", " | " }
                    "AIRAC {airac.read().code}"
                }
                div { class: "doc-meta-zoom",
                    button {
                        class: "zoom-btn",
                        disabled: z <= 50,
                        onclick: move |_| {
                            let chart_id = chart_id_minus.clone();
                            let mut state = state_minus.clone();
                            let cur = zoom();
                            if cur > 50 {
                                let next = cur - 25;
                                zoom.set(next);
                                let ws_id = state.read().active_workspace_id.clone();
                                state.write().chart_zoom.insert(chart_id.clone(), next);
                                if let Some(ws_id) = ws_id {
                                    let conn = crate::persistence::db().lock().unwrap();
                                    crate::persistence::workspaces::save_chart_zoom(
                                        &conn,
                                        &ws_id,
                                        &chart_id,
                                        next,
                                    );
                                }
                            }
                        },
                        "\u{2212}"
                    }
                    span { class: "zoom-label", "{z}%" }
                    button {
                        class: "zoom-btn",
                        disabled: z >= 200,
                        onclick: move |_| {
                            let chart_id = chart_id_plus.clone();
                            let mut state = state_plus.clone();
                            let cur = zoom();
                            if cur < 200 {
                                let next = cur + 25;
                                zoom.set(next);
                                let ws_id = state.read().active_workspace_id.clone();
                                state.write().chart_zoom.insert(chart_id.clone(), next);
                                if let Some(ws_id) = ws_id {
                                    let conn = crate::persistence::db().lock().unwrap();
                                    crate::persistence::workspaces::save_chart_zoom(
                                        &conn,
                                        &ws_id,
                                        &chart_id,
                                        next,
                                    );
                                }
                            }
                        },
                        "+"
                    }
                    button {
                        class: "zoom-btn fit-btn",
                        onclick: move |_| {
                            let chart_id = chart_id_fit.clone();
                            let mut state = state_fit.clone();
                            spawn(async move {
                                let js = format!(
                                    "const el = document.getElementById('pdf-scroll'); \
                                     if (el) {{ return Math.floor((el.clientWidth - 48) / {} * 100); }} \
                                     return 100;",
                                    BASE_IMG_WIDTH
                                );
                                let result = document::eval(&js).await;
                                if let Ok(val) = result {
                                    if let Some(fit) = val.as_f64() {
                                        let fit = ((fit / 25.0).round() * 25.0) as u32;
                                        let fit = fit.clamp(50, 200);
                                        zoom.set(fit);
                                        let ws_id = state.read().active_workspace_id.clone();
                                        state.write().chart_zoom.insert(chart_id.clone(), fit);
                                        if let Some(ws_id) = ws_id {
                                            let conn = crate::persistence::db().lock().unwrap();
                                            crate::persistence::workspaces::save_chart_zoom(
                                                &conn,
                                                &ws_id,
                                                &chart_id,
                                                fit,
                                            );
                                        }
                                    }
                                }
                            });
                        },
                        "Fit"
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn DocViewer(chart: crate::models::Chart, zoom: Signal<u32>) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let chart_id = chart.id.clone();
    let pdf_state = state.read().pdf_cache.get(&chart_id).cloned();

    // Trigger rendering if not yet cached
    if pdf_state.is_none() {
        let chart_id = chart_id.clone();
        let effective_chart = {
            let s = state.read();
            s.charts
                .iter()
                .find(|c| {
                    c.source == chart.source
                        && c.provider_relative_url == chart.provider_relative_url
                })
                .cloned()
                .unwrap_or_else(|| chart.clone())
        };
        let urls = effective_chart.runtime_urls();
        state
            .write()
            .pdf_cache
            .insert(chart_id.clone(), PdfState::Loading);
        spawn(async move {
            let mut first_pages: Vec<crate::pdf::RenderedPage> = Vec::new();
            for (idx, url) in urls.iter().enumerate() {
                match crate::pdf::fetch_and_render_first_page(url).await {
                    Ok(mut page) => {
                        page.index = idx;
                        first_pages.push(page);
                        state
                            .write()
                            .pdf_cache
                            .insert(chart_id.clone(), PdfState::Partial(first_pages.clone()));
                    }
                    Err(e) => {
                        state.write().pdf_cache.insert(chart_id, PdfState::Error(e));
                        return;
                    }
                }
            }

            match crate::pdf::fetch_and_render_many(&urls).await {
                Ok(pages) => {
                    state
                        .write()
                        .pdf_cache
                        .insert(chart_id, PdfState::Rendered(pages));
                }
                Err(e) => {
                    state.write().pdf_cache.insert(chart_id, PdfState::Error(e));
                }
            }
        });
    }

    // Zoom levels: 50% -> 200%, step 25%
    match pdf_state {
        Some(PdfState::Rendered(pages)) => {
            let z = zoom();
            let scale = z as f64 / 100.0;
            let img_width = format!("{}px", (BASE_IMG_WIDTH * scale) as i32);
            rsx! {
                div { class: "doc-viewer pdf-active",
                    div {
                        class: "pdf-scroll",
                        id: "pdf-scroll",
                        onmounted: move |_| {
                            spawn(async move {
                                let _ = document::eval(r#"
                                    const el = document.getElementById('pdf-scroll');
                                    if (el && !el._pan) {
                                        el._pan = true;
                                        let dragging = false, startX = 0, startY = 0, scrollL = 0, scrollT = 0;
                                        el.addEventListener('mousedown', e => {
                                            dragging = true;
                                            startX = e.clientX;
                                            startY = e.clientY;
                                            scrollL = el.scrollLeft;
                                            scrollT = el.scrollTop;
                                            el.classList.add('grabbing');
                                            e.preventDefault();
                                        });
                                        window.addEventListener('mousemove', e => {
                                            if (!dragging) return;
                                            el.scrollLeft = scrollL - (e.clientX - startX);
                                            el.scrollTop = scrollT - (e.clientY - startY);
                                        });
                                        window.addEventListener('mouseup', () => {
                                            dragging = false;
                                            el.classList.remove('grabbing');
                                        });
                                    }
                                "#).await;
                            });
                        },
                        div { class: "pdf-scroll-inner",
                            for page in &pages {
                                div { class: "pdf-page-container",
                                    div { class: "pdf-page-number", "Page {page.index + 1}" }
                                    img {
                                        class: "pdf-page-img",
                                        style: "width: {img_width}",
                                        src: "{page.data_url}",
                                        alt: "Page {page.index + 1}",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(PdfState::Partial(pages)) => {
            let z = zoom();
            let scale = z as f64 / 100.0;
            let img_width = format!("{}px", (BASE_IMG_WIDTH * scale) as i32);
            rsx! {
                div { class: "doc-viewer pdf-active",
                    div {
                        class: "pdf-scroll",
                        id: "pdf-scroll",
                        div { class: "pdf-scroll-inner",
                            for page in &pages {
                                div { class: "pdf-page-container",
                                    div { class: "pdf-page-number", "Page {page.index + 1}" }
                                    img {
                                        class: "pdf-page-img",
                                        style: "width: {img_width}",
                                        src: "{page.data_url}",
                                        alt: "Page {page.index + 1}",
                                    }
                                }
                            }
                            div { class: "empty-state", style: "padding: 16px 0 8px;",
                                div { class: "loading-spinner" }
                                p { style: "margin-top: 12px;", "Loading remaining pages..." }
                            }
                        }
                    }
                }
            }
        }
        Some(PdfState::Error(msg)) => {
            rsx! {
                div { class: "doc-viewer",
                    div { class: "empty-state",
                        div { class: "icon", "⚠" }
                        p { "{tr(lang, \"doc.error\")}" }
                        p { style: "font-size: 12px; margin-top: 8px; color: var(--text-secondary);",
                            "{msg}"
                        }
                    }
                }
            }
        }
        _ => {
            rsx! {
                div { class: "doc-viewer",
                    div { class: "empty-state",
                        div { class: "loading-spinner" }
                        p { style: "margin-top: 16px;", "{tr(lang, \"doc.loading\")}" }
                    }
                }
            }
        }
    }
}
