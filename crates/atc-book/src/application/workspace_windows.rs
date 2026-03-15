use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::application::tabs as tab_usecases;
use crate::state::AppState;

pub fn move_chart_tab_to_workspace_popout(
    state: &mut AppState,
    repo: &dyn WorkspaceRepository,
    workspace_id: &str,
    chart_id: &str,
) {
    let (mut pop_ids, _pop_active) = repo.load_popout_tab_state(workspace_id);
    let target_idx = if let Some(idx) = pop_ids.iter().position(|id| id == chart_id) {
        idx
    } else {
        pop_ids.push(chart_id.to_string());
        pop_ids.len().saturating_sub(1)
    };
    repo.save_popout_tab_state(workspace_id, &pop_ids, Some(target_idx));

    tab_usecases::close_tab_by_id(state, chart_id);
    tab_usecases::ensure_notes_tab(state);

    let (tab_ids, active) = tab_usecases::chart_tab_state(&state.tabs, state.active_tab);
    let extra_tabs = state
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .map(|w| w.extra_tabs.clone())
        .unwrap_or_default();
    repo.save_tab_state(workspace_id, &tab_ids, active, &extra_tabs);

    if let Some(ws) = state.workspaces.iter_mut().find(|w| w.id == workspace_id) {
        ws.open_tabs = tab_ids.clone();
        ws.active_tab_index = active;
    }
    state.popout_sync_pending = true;
}

pub fn merge_workspace_popout_tabs_into_main(
    repo: &dyn WorkspaceRepository,
    workspace_id: &str,
) {
    let (pop_tabs, _) = repo.load_popout_tab_state(workspace_id);
    if let Some(ws) = repo
        .list_workspaces()
        .into_iter()
        .find(|w| w.id == workspace_id)
    {
        let mut merged = ws.open_tabs.clone();
        for id in pop_tabs {
            if !merged.contains(&id) {
                merged.push(id);
            }
        }
        repo.save_tab_state(
            workspace_id,
            &merged,
            ws.active_tab_index,
            &ws.extra_tabs,
        );
    }
    repo.save_popout_tab_state(workspace_id, &[], None);
}

