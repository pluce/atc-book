use dioxus::prelude::*;
use futures_timer::Delay;
use std::time::Duration;

use super::viewer_header::ViewerHeader;
use crate::i18n::tr;
use crate::state::AppState;

#[component]
pub(super) fn AipDocViewer(doc: crate::models::AipDocument) -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;

    let html = use_signal(String::new);
    let loading = use_signal(|| true);
    let error = use_signal(|| Option::<String>::None);
    let mut query = use_signal(String::new);
    let match_count = use_signal(|| 0usize);
    let mut active_hit = use_signal(|| 0usize);

    // Load HTML doc
    use_effect({
        let doc = doc.clone();
        let mut html = html.clone();
        let mut loading = loading.clone();
        let mut error = error.clone();
        move || {
            loading.set(true);
            error.set(None);
            html.set(String::new());
            let doc = doc.clone();
            spawn(async move {
                match crate::aip_doc::load_doc_html(&doc).await {
                    Ok(content) => {
                        html.set(content);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        }
    });

    // Highlight query terms and count matches
    {
        let html = html.clone();
        let query = query.clone();
        let mut active_hit = active_hit.clone();
        let mut match_count = match_count.clone();
        use_effect(move || {
            let html_text = html();
            let q = query();
            if html_text.is_empty() {
                return;
            }
            let terms: Vec<String> = if q.trim().len() >= 2 {
                q.split_whitespace()
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| s.len() >= 2)
                    .take(10)
                    .collect()
            } else {
                Vec::new()
            };
            let terms_json = serde_json::to_string(&terms).unwrap_or_else(|_| "[]".to_string());
            let html_json = serde_json::to_string(&html_text).unwrap_or_else(|_| "\"\"".to_string());
            spawn(async move {
                let js_mark = format!(
                    r#"
                        (function() {{
                            try {{
                            const root = document.getElementById('aip-doc-content');
                            if (!root) return;
                            const original = {html_json};
                            root.dataset.originalHtml = original;
                            root.innerHTML = original;
                            const terms = {terms_json};
                            if (!terms.length) return;
                            const escapeRegExp = (s) => s.replace(/[.*+?^${{}}()|[\]\\]/g, '\\$&');
                            const collectTextNodes = (node) => {{
                                const out = [];
                                const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
                                let n;
                                while ((n = walker.nextNode())) out.push(n);
                                return out;
                            }};
                            for (const term of terms) {{
                                const re = new RegExp(escapeRegExp(term), 'ig');
                                for (const textNode of collectTextNodes(root)) {{
                                    const text = textNode.nodeValue;
                                    if (!text || !re.test(text)) continue;
                                    re.lastIndex = 0;
                                    const frag = document.createDocumentFragment();
                                    let last = 0;
                                    text.replace(re, (m, off) => {{
                                        if (off > last) frag.appendChild(document.createTextNode(text.slice(last, off)));
                                        const mark = document.createElement('mark');
                                        mark.className = 'aip-hit';
                                        mark.textContent = m;
                                        frag.appendChild(mark);
                                        last = off + m.length;
                                        return m;
                                    }});
                                    if (last < text.length) frag.appendChild(document.createTextNode(text.slice(last)));
                                    textNode.parentNode.replaceChild(frag, textNode);
                                }}
                            }}
                            }} catch (e) {{
                                console.error('aip search mark error', e);
                                return;
                            }}
                        }})();
                    "#,
                );
                let js_count = "const root = document.getElementById('aip-doc-content'); if (!root) return -1; return root.querySelectorAll('mark.aip-hit').length;";
                let mut count = 0usize;
                for _ in 0..8 {
                    let _ = document::eval(&js_mark).await;
                    let parsed = match document::eval(js_count).await {
                        Ok(val) => val
                            .as_i64()
                            .map(|n| n as f64)
                            .or_else(|| val.as_u64().map(|n| n as f64))
                            .or_else(|| val.as_f64())
                            .or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok()))
                            .unwrap_or(0.0),
                        Err(_) => 0.0,
                    };
                    if parsed >= 0.0 {
                        count = parsed.max(0.0) as usize;
                        break;
                    }
                    Delay::new(Duration::from_millis(25)).await;
                }
                match_count.set(count);
                let cur = active_hit();
                if count == 0 {
                    active_hit.set(0);
                } else if cur >= count {
                    active_hit.set(count - 1);
                }
            });
        });
    }

    // Move active highlight when active_hit changes (without rebuilding marks)
    {
        let active_hit = active_hit.clone();
        let mut match_count = match_count.clone();
        use_effect(move || {
            let active = active_hit();
            spawn(async move {
                let js_nav = format!(
                    r#"
                        (function() {{
                            const root = document.getElementById('aip-doc-content');
                            if (!root) return;
                            const marks = Array.from(root.querySelectorAll('mark.aip-hit'));
                            marks.forEach(m => m.classList.remove('active-hit'));
                            if (!marks.length) return;
                            const idx = Math.max(0, Math.min({active}, marks.length - 1));
                            const cur = marks[idx];
                            if (cur) {{
                                cur.classList.add('active-hit');
                                cur.scrollIntoView({{ block: 'center', behavior: 'smooth' }});
                            }}
                        }})();
                    "#,
                );
                let js_count = "const root = document.getElementById('aip-doc-content'); if (!root) return -1; return root.querySelectorAll('mark.aip-hit').length;";
                let mut count = 0usize;
                for _ in 0..8 {
                    let _ = document::eval(&js_nav).await;
                    let parsed = match document::eval(js_count).await {
                        Ok(val) => val
                            .as_i64()
                            .map(|n| n as f64)
                            .or_else(|| val.as_u64().map(|n| n as f64))
                            .or_else(|| val.as_f64())
                            .or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok()))
                            .unwrap_or(0.0),
                        Err(_) => 0.0,
                    };
                    if parsed >= 0.0 {
                        count = parsed.max(0.0) as usize;
                        break;
                    }
                    Delay::new(Duration::from_millis(25)).await;
                }
                match_count.set(count);
            });
        });
    }

    let query_text = query();
    let n_matches = match_count();

    rsx! {
        div { class: "doc-viewer aip-doc-viewer",
            ViewerHeader {
                div { class: "aip-topbar",
                    input {
                        class: "aip-question-input",
                        r#type: "text",
                        placeholder: tr(lang, "aip.ask.placeholder"),
                        value: query_text.clone(),
                        oninput: move |e| {
                            query.set(e.value());
                            active_hit.set(0);
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter && n_matches > 0 {
                                e.prevent_default();
                                let cur = active_hit();
                                if e.modifiers().shift() {
                                    active_hit.set((cur + n_matches - 1) % n_matches);
                                } else {
                                    active_hit.set((cur + 1) % n_matches);
                                }
                            }
                        }
                    }
                    button {
                        class: "zoom-btn",
                        disabled: n_matches == 0,
                        onclick: move |_| {
                            if n_matches > 0 {
                                let cur = active_hit();
                                active_hit.set((cur + n_matches - 1) % n_matches);
                            }
                        },
                        "↑"
                    }
                    button {
                        class: "zoom-btn",
                        disabled: n_matches == 0,
                        onclick: move |_| {
                            if n_matches > 0 {
                                let cur = active_hit();
                                active_hit.set((cur + 1) % n_matches);
                            }
                        },
                        "↓"
                    }
                    span {
                        class: "pdf-nav-label",
                        if query_text.trim().len() >= 2 {
                            {format!("{}: {}", tr(lang, "aip.search.matches"), n_matches)}
                        }
                    }
                }
            }
            if !loading() && query_text.trim().len() >= 2 && n_matches == 0 {
                div { class: "nav-section-title", {tr(lang, "aip.search.no_match")} }
            }
            if loading() {
                div { class: "empty-state",
                    div { class: "loading-spinner" }
                    p { style: "margin-top: 16px;", {tr(lang, "doc.loading")} }
                }
            } else if let Some(err) = error() {
                div { class: "empty-state",
                    div { class: "icon", "⚠" }
                    p { {tr(lang, "doc.error")} }
                    p { style: "font-size: 12px; margin-top: 8px; color: var(--text-secondary);", "{err}" }
                }
            } else {
                div {
                    class: "aip-doc-scroll",
                    div {
                        class: "aip-doc-content",
                        id: "aip-doc-content",
                        dangerous_inner_html: html(),
                    }
                }
            }
        }
    }
}
