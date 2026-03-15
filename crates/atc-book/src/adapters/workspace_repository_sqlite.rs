use rusqlite::Connection;

use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::models::{Chart, ExtraTab, Workspace};

pub struct SqliteWorkspaceRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteWorkspaceRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl WorkspaceRepository for SqliteWorkspaceRepository<'_> {
    fn list_workspaces(&self) -> Vec<Workspace> {
        crate::persistence::workspaces::list_workspaces(self.conn)
    }

    fn add_chart_to_workspace(&self, workspace_id: &str, airport: &str, chart: &Chart) {
        crate::persistence::workspaces::add_chart_to_workspace(self.conn, workspace_id, airport, chart);
    }

    fn add_extra_tab(&self, workspace_id: &str, tab: &ExtraTab) {
        crate::persistence::workspaces::add_extra_tab(self.conn, workspace_id, tab);
    }

    fn remove_extra_tab(&self, workspace_id: &str, tab: &ExtraTab) {
        crate::persistence::workspaces::remove_extra_tab(self.conn, workspace_id, tab);
    }

    fn save_tab_state(
        &self,
        workspace_id: &str,
        open_tabs: &[String],
        active_tab_index: Option<usize>,
        extra_tabs: &[ExtraTab],
    ) {
        crate::persistence::workspaces::save_tab_state(
            self.conn,
            workspace_id,
            open_tabs,
            active_tab_index,
            extra_tabs,
        );
    }

    fn load_popout_tab_state(&self, workspace_id: &str) -> (Vec<String>, Option<usize>) {
        crate::persistence::workspaces::load_popout_tab_state(self.conn, workspace_id)
    }

    fn save_popout_tab_state(
        &self,
        workspace_id: &str,
        popout_tabs: &[String],
        active_tab_index: Option<usize>,
    ) {
        crate::persistence::workspaces::save_popout_tab_state(
            self.conn,
            workspace_id,
            popout_tabs,
            active_tab_index,
        );
    }
}
