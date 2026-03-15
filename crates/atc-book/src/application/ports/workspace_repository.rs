use crate::models::{Chart, ExtraTab, Workspace};

pub trait WorkspaceRepository {
    fn list_workspaces(&self) -> Vec<Workspace>;
    fn add_chart_to_workspace(&self, workspace_id: &str, airport: &str, chart: &Chart);
    fn add_extra_tab(&self, workspace_id: &str, tab: &ExtraTab);
    fn remove_extra_tab(&self, workspace_id: &str, tab: &ExtraTab);
    fn save_tab_state(
        &self,
        workspace_id: &str,
        open_tabs: &[String],
        active_tab_index: Option<usize>,
        extra_tabs: &[ExtraTab],
    );
    fn load_popout_tab_state(&self, workspace_id: &str) -> (Vec<String>, Option<usize>);
    fn save_popout_tab_state(
        &self,
        workspace_id: &str,
        popout_tabs: &[String],
        active_tab_index: Option<usize>,
    );
}
