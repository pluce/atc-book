use crate::models::{AipDocument, Chart, ExtraTab, WorkspaceChart};
use crate::state::{AppState, Tab};

fn hydrated_workspace_chart(state: &AppState, workspace_chart: &WorkspaceChart) -> Chart {
    let current = &workspace_chart.chart;
    if let Some(fresh) = state.charts.iter().find(|c| {
        c.source == current.source && c.provider_relative_url == current.provider_relative_url
    }) {
        let mut merged = fresh.clone();
        // Keep workspace identity/title to avoid breaking open_tab references.
        merged.id = current.id.clone();
        merged.custom_title = current.custom_title.clone();
        return merged;
    }

    current.clone()
}

pub fn chart_tab_state(tabs: &[Tab], active_tab: Option<usize>) -> (Vec<String>, Option<usize>) {
    let mut ids = Vec::new();
    let mut active_chart_index = None;
    for (i, tab) in tabs.iter().enumerate() {
        if let Some(id) = tab.chart_id() {
            let idx = ids.len();
            ids.push(id.to_string());
            if Some(i) == active_tab {
                active_chart_index = Some(idx);
            }
        }
    }
    (ids, active_chart_index)
}

pub fn open_or_focus(state: &mut AppState, tab: Tab) {
    let tab_id = tab.id.clone();
    if !state.tabs.iter().any(|t| t.id == tab_id) {
        state.tabs.push(tab);
    }
    if let Some(idx) = state.tabs.iter().position(|t| t.id == tab_id) {
        state.active_tab = Some(idx);
    }
}

pub fn ensure_notes_tab(state: &mut AppState) {
    if !state.tabs.iter().any(|t| t.is_notes()) {
        state.tabs.insert(0, Tab::notes());
    }
    if state.active_tab.is_none() {
        state.active_tab = Some(0);
    }
}

pub fn close_tab_by_id(state: &mut AppState, tab_id: &str) {
    if let Some(pos) = state.tabs.iter().position(|t| t.id == tab_id) {
        state.tabs.remove(pos);
        match state.active_tab {
            Some(a) if a == pos => {
                state.active_tab = if state.tabs.is_empty() {
                    None
                } else {
                    Some(a.min(state.tabs.len() - 1))
                };
            }
            Some(a) if a > pos => {
                state.active_tab = Some(a - 1);
            }
            _ => {}
        }
    }
}

pub fn open_or_focus_chart(state: &mut AppState, chart: Chart, airport: String) {
    open_or_focus(state, Tab::chart(chart, airport));
}

pub fn open_or_focus_aip(state: &mut AppState, doc: AipDocument) {
    open_or_focus(state, Tab::aip_doc(doc));
}

pub fn open_or_focus_atis(state: &mut AppState, icao: String) {
    open_or_focus(state, Tab::atis(icao));
}

pub fn open_all_workspace_charts(state: &mut AppState, chart_refs: &[WorkspaceChart]) {
    for wc in chart_refs {
        if !state.tabs.iter().any(|t| t.id == wc.chart.id) {
            let chart = hydrated_workspace_chart(state, wc);
            state
                .tabs
                .push(Tab::chart(chart, wc.airport.clone()));
        }
    }
    if let Some(first) = chart_refs.first() {
        if let Some(idx) = state.tabs.iter().position(|t| t.id == first.chart.id) {
            state.active_tab = Some(idx);
        }
    }
}

pub fn restore_extra_tabs(state: &mut AppState, extra_tabs: &[ExtraTab]) {
    for et in extra_tabs {
        match et {
            ExtraTab::Atis { icao } => {
                let tab_id = format!("atis_{}", icao);
                if !state.tabs.iter().any(|t| t.id == tab_id) {
                    state.tabs.push(Tab::atis(icao.clone()));
                }
            }
            ExtraTab::AipDoc { doc } => {
                if !state.tabs.iter().any(|t| t.id == doc.id) {
                    state.tabs.push(Tab::aip_doc(doc.clone()));
                }
            }
        }
    }
}

pub fn rebuild_workspace_tabs(
    state: &mut AppState,
    chart_refs: &[WorkspaceChart],
    open_tabs: &[String],
    active_tab_index: Option<usize>,
    extra_tabs: &[ExtraTab],
) {
    state.tabs.clear();
    state.tabs.push(Tab::notes());

    if !open_tabs.is_empty() {
        for tab_id in open_tabs {
            if let Some(wc) = chart_refs.iter().find(|c| c.chart.id == *tab_id) {
                let chart = hydrated_workspace_chart(state, wc);
                state
                    .tabs
                    .push(Tab::chart(chart, wc.airport.clone()));
            }
        }
        state.active_tab = active_tab_index.map(|i| (i + 1).min(state.tabs.len().saturating_sub(1)));
        if state.active_tab.is_none() {
            state.active_tab = Some(0);
        }
    } else {
        for wc in chart_refs {
            let chart = hydrated_workspace_chart(state, wc);
            state
                .tabs
                .push(Tab::chart(chart, wc.airport.clone()));
        }
        state.active_tab = Some(0);
    }

    restore_extra_tabs(state, extra_tabs);
}

pub fn rebuild_popout_tabs(
    state: &mut AppState,
    chart_refs: &[WorkspaceChart],
    popout_tabs: &[String],
    popout_active: Option<usize>,
) {
    state.tabs.clear();
    state.tabs.push(Tab::notes());

    for tab_id in popout_tabs {
        if let Some(wc) = chart_refs.iter().find(|c| c.chart.id == *tab_id) {
            let chart = hydrated_workspace_chart(state, wc);
            state
                .tabs
                .push(Tab::chart(chart, wc.airport.clone()));
        }
    }

    state.active_tab = popout_active
        .map(|i| (i + 1).min(state.tabs.len().saturating_sub(1)))
        .or(Some(0));
}
