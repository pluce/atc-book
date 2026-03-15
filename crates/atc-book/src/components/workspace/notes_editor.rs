use dioxus::prelude::*;

use crate::i18n::tr;
use crate::state::AppState;

fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

async fn editor_set_html(editor_id: &str, html: &str) {
    let js = format!(
        "const el = document.getElementById('{}'); if (el) {{ el.innerHTML = {}; }}",
        editor_id,
        js_string(html)
    );
    let _ = document::eval(&js).await;
}

async fn editor_get_html(editor_id: &str) -> Option<String> {
    let js = format!(
        "const el = document.getElementById('{}'); if (el) {{ return el.innerHTML; }} return '';",
        editor_id
    );
    document::eval(&js)
        .await
        .ok()
        .map(|v| v.as_str().unwrap_or("").to_string())
}

async fn editor_exec_command(editor_id: &str, cmd: &str, value: Option<&str>) {
    let cmd_js = if let Some(v) = value {
        format!("document.execCommand('{}', false, {});", cmd, js_string(v))
    } else {
        format!("document.execCommand('{}', false, null);", cmd)
    };
    let js = format!(
        "document.getElementById('{}').focus(); {}",
        editor_id, cmd_js
    );
    let _ = document::eval(&js).await;
}

async fn editor_toggle_block(editor_id: &str, tag: &str) {
    let js = format!(
        "(function() {{ \
            var sel = window.getSelection(); \
            if (!sel.rangeCount) return; \
            var node = sel.anchorNode; \
            while (node && node.nodeType !== 1) node = node.parentNode; \
            while (node && node.id !== '{}') {{ \
                if (node.tagName && node.tagName.toLowerCase() === '{}') {{ \
                    document.execCommand('formatBlock', false, 'p'); \
                    return; \
                }} \
                node = node.parentNode; \
            }} \
            document.execCommand('formatBlock', false, '{}'); \
        }})()",
        editor_id, tag, tag
    );
    let _ = document::eval(&js).await;
}

#[component]
pub(super) fn NotesEditor(pinned: bool) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let mut notes_dirty = use_signal(|| false);

    // Get workspace ID and current notes
    let ws_id = state.read().active_workspace_id.clone().unwrap_or_default();
    let initial_notes = state
        .read()
        .workspaces
        .iter()
        .find(|w| w.id == ws_id)
        .and_then(|w| w.notes.clone())
        .unwrap_or_default();

    // Use different element IDs so pinned and full-tab editors don't conflict
    let editor_id = if pinned {
        "notes-editor-pinned"
    } else {
        "notes-editor-content"
    };

    // Initialize the contenteditable editor with saved content
    use_effect({
        let initial_notes = initial_notes.clone();
        let editor_id = editor_id.to_string();
        move || {
            let html = initial_notes.clone();
            let editor_id = editor_id.clone();
            spawn(async move {
                editor_set_html(&editor_id, &html).await;
            });
        }
    });

    // Save notes via JS -> get innerHTML -> persist
    let save_notes = {
        let ws_id = ws_id.clone();
        let editor_id = editor_id.to_string();
        let notes_dirty = notes_dirty.clone();
        move |_: FocusEvent| {
            let ws_id = ws_id.clone();
            let editor_id = editor_id.clone();
            let mut notes_dirty = notes_dirty.clone();
            spawn(async move {
                if let Some(html) = editor_get_html(&editor_id).await {
                    let notes = if html.is_empty() || html == "<br>" {
                        None
                    } else {
                        Some(html)
                    };
                    {
                        let conn = crate::persistence::db().lock().unwrap();
                        crate::persistence::workspaces::save_notes(&conn, &ws_id, notes.as_deref());
                    }
                    // Update in-memory workspace
                    let mut s = state.write();
                    if let Some(ws) = s.workspaces.iter_mut().find(|w| w.id == ws_id) {
                        ws.notes = notes;
                    }
                    notes_dirty.set(false);
                }
            });
        }
    };

    // Pull latest notes when editor gets focus (cross-window synchronization point)
    let refresh_notes = {
        let ws_id = ws_id.clone();
        let editor_id = editor_id.to_string();
        let notes_dirty = notes_dirty.clone();
        move |_: FocusEvent| {
            let ws_id = ws_id.clone();
            let editor_id = editor_id.clone();
            let notes_dirty = notes_dirty.clone();
            spawn(async move {
                if notes_dirty() {
                    return;
                }
                let current = editor_get_html(&editor_id).await.unwrap_or_default();
                let latest = {
                    let conn = crate::persistence::db().lock().unwrap();
                    crate::persistence::workspaces::list_workspaces(&conn)
                        .into_iter()
                        .find(|w| w.id == ws_id)
                        .and_then(|w| w.notes)
                        .unwrap_or_default()
                };
                if !current.is_empty() && latest.is_empty() {
                    return;
                }
                editor_set_html(&editor_id, &latest).await;
            });
        }
    };

    rsx! {
        div { class: "notes-container",
            if !pinned {
                NotesToolbar { pinned: false, editor_id: editor_id.to_string() }
            }
            div {
                class: "notes-editor-scroll",
                div {
                    class: "notes-editor-content",
                    id: "{editor_id}",
                    "data-placeholder": "{tr(lang, \"notes.placeholder\")}",
                    contenteditable: "true",
                    onfocus: refresh_notes,
                    onblur: save_notes,
                    oninput: move |_| notes_dirty.set(true),
                    onkeyup: move |_| notes_dirty.set(true),
                }
            }
        }
    }
}

#[component]
fn NotesToolbar(pinned: bool, editor_id: String) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let show_fg_color = use_signal(|| false);
    let show_bg_color = use_signal(|| false);

    let exec = {
        let editor_id = editor_id.clone();
        move |cmd: &str, value: Option<&str>| {
            let cmd = cmd.to_string();
            let value = value.map(|v| v.to_string());
            let editor_id = editor_id.clone();
            move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let cmd = cmd.clone();
                let value = value.clone();
                let editor_id = editor_id.clone();
                spawn(async move {
                    editor_exec_command(&editor_id, &cmd, value.as_deref()).await;
                });
            }
        }
    };

    // Toggle block format: if already in that tag, revert to <p>
    let toggle_block = {
        let editor_id = editor_id.clone();
        move |tag: &str| {
            let tag = tag.to_string();
            let editor_id = editor_id.clone();
            move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let tag = tag.clone();
                let editor_id = editor_id.clone();
                spawn(async move {
                    editor_toggle_block(&editor_id, &tag).await;
                });
            }
        }
    };

    rsx! {
        div { class: "notes-toolbar",
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h1\")}",
                onmousedown: toggle_block("h1"),
                "H1"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h2\")}",
                onmousedown: toggle_block("h2"),
                "H2"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.h3\")}",
                onmousedown: toggle_block("h3"),
                "H3"
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.bold\")}",
                onmousedown: exec("bold", None),
                "B"
            }
            button {
                class: "notes-tool-btn notes-tool-italic",
                title: "{tr(lang, \"notes.italic\")}",
                onmousedown: exec("italic", None),
                "I"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.underline\")}",
                onmousedown: exec("underline", None),
                "U"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.strike\")}",
                onmousedown: exec("strikeThrough", None),
                "S̶"
            }
            span { class: "notes-tool-sep" }
            NotesColorPicker {
                editor_id: editor_id.clone(),
                picker_kind: "fg".to_string(),
                is_open: show_fg_color,
                other_open: show_bg_color,
            }
            NotesColorPicker {
                editor_id: editor_id.clone(),
                picker_kind: "bg".to_string(),
                is_open: show_bg_color,
                other_open: show_fg_color,
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.bullets\")}",
                onmousedown: exec("insertUnorderedList", None),
                "• —"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.numbered\")}",
                onmousedown: exec("insertOrderedList", None),
                "1."
            }
            span { class: "notes-tool-sep" }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.quote\")}",
                onmousedown: toggle_block("blockquote"),
                "❝"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.code\")}",
                onmousedown: toggle_block("pre"),
                "<>"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.rule\")}",
                onmousedown: exec("insertHorizontalRule", None),
                "―"
            }
            button {
                class: "notes-tool-btn",
                title: "{tr(lang, \"notes.paragraph\")}",
                onmousedown: toggle_block("p"),
                "¶"
            }
            span { class: "notes-tool-sep notes-tool-spacer" }
            button {
                class: if pinned { "notes-tool-btn notes-pin-btn active" } else { "notes-tool-btn notes-pin-btn" },
                title: if pinned { tr(lang, "notes.unpin") } else { tr(lang, "notes.pin") },
                onclick: move |_| {
                    let mut s = state.write();
                    let now_pinned = !s.notes_pinned;
                    s.notes_pinned = now_pinned;
                    let ws_id = s.active_workspace_id.clone();
                    let is_popout = s.is_popout;
                    if now_pinned {
                        // Switch to first chart tab when pinning
                        if let Some(idx) = s.tabs.iter().position(|t| !t.is_notes()) {
                            s.active_tab = Some(idx);
                        }
                    }
                    drop(s);
                    if !is_popout {
                        if let Some(ws_id) = ws_id {
                            let conn = crate::persistence::db().lock().unwrap();
                            crate::persistence::workspaces::save_notes_pinned(&conn, &ws_id, now_pinned);
                            if let Some(ws) = state.write().workspaces.iter_mut().find(|w| w.id == ws_id) {
                                ws.notes_pinned = Some(now_pinned);
                            }
                        }
                    }
                },
                "📌"
            }
        }
    }
}

#[component]
fn NotesColorPicker(
    editor_id: String,
    picker_kind: String,
    mut is_open: Signal<bool>,
    mut other_open: Signal<bool>,
) -> Element {
    let state = use_context::<Signal<AppState>>();
    let lang = state.read().language;
    let is_fg = picker_kind == "fg";
    let title = if is_fg {
        tr(lang, "notes.text_color")
    } else {
        tr(lang, "notes.highlight")
    };
    let exec_cmd = if is_fg { "foreColor" } else { "hiliteColor" };

    let colors: Vec<(&str, &str)> = if is_fg {
        vec![
            ("#1C1917", "Noir"),
            ("#DC2626", "Rouge"),
            ("#D97706", "Ambre"),
            ("#16A34A", "Vert"),
            ("#2563EB", "Bleu"),
            ("#7C3AED", "Violet"),
            ("#94A3B8", "Gris"),
        ]
    } else {
        vec![
            ("transparent", tr(lang, "notes.none")),
            ("#FEF3C7", "Jaune"),
            ("#FEE2E2", "Rouge"),
            ("#DCFCE7", "Vert"),
            ("#DBEAFE", "Bleu"),
            ("#F3E8FF", "Violet"),
            ("#F1F5F9", "Gris"),
        ]
    };

    rsx! {
        div { class: "notes-tool-dropdown-wrapper",
            button {
                class: "notes-tool-btn",
                title: "{title}",
                onmousedown: move |e| {
                    e.prevent_default();
                    e.stop_propagation();
                    is_open.toggle();
                    other_open.set(false);
                },
                if is_fg {
                    span { class: "notes-tool-color-icon",
                        "A"
                        span { class: "notes-tool-color-bar", style: "background: var(--amber);" }
                    }
                } else {
                    span { class: "notes-tool-color-icon", "🖍" }
                }
            }
            if is_open() {
                div { class: "notes-color-palette",
                    for (color, label) in colors {
                        {
                            let color = color.to_string();
                            let label = label.to_string();
                            let editor_id = editor_id.clone();
                            let exec_cmd = exec_cmd.to_string();
                            let is_remove = !is_fg && color == "transparent";
                            rsx! {
                                button {
                                    class: if is_remove { "notes-color-swatch notes-color-remove" } else { "notes-color-swatch" },
                                    title: "{label}",
                                    style: if !is_remove { format!("background: {};", color) } else { String::new() },
                                    onmousedown: move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let editor_id = editor_id.clone();
                                        let exec_cmd = exec_cmd.clone();
                                        let color = color.clone();
                                        spawn(async move {
                                            if is_remove {
                                                editor_exec_command(&editor_id, "removeFormat", None).await;
                                            } else {
                                                editor_exec_command(&editor_id, &exec_cmd, Some(&color)).await;
                                            }
                                        });
                                        is_open.set(false);
                                    },
                                    if is_remove { "✕" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
