use super::helpers::{
    area_sort_rank, build_stack_group_layouts, default_panel_state,
};
use super::*;

impl StudioGuiWindowLayoutModel {
    pub fn from_window_model(window: &StudioGuiWindowModel) -> Self {
        Self::from_window_model_with_layout_state(window, &window.layout_state)
    }

    pub fn from_window_model_with_layout_state(
        window: &StudioGuiWindowModel,
        layout_state: &StudioGuiWindowLayoutState,
    ) -> Self {
        Self::from_areas(
            layout_state,
            &window.header,
            &window.commands,
            &window.canvas,
            &window.runtime,
        )
    }

    pub fn panel(&self, area_id: StudioGuiWindowAreaId) -> Option<&StudioGuiWindowPanelLayout> {
        self.panels.iter().find(|panel| panel.area_id == area_id)
    }

    pub fn panels_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowPanelLayout> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| (panel.stack_group, panel.order, area_sort_rank(panel.area_id)));
        panels
    }

    pub fn panels_in_stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Vec<&StudioGuiWindowPanelLayout> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| (panel.order, area_sort_rank(panel.area_id)));
        panels
    }

    pub fn stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Option<&StudioGuiWindowStackGroupLayout> {
        self.stack_groups
            .iter()
            .find(|group| group.dock_region == dock_region && group.stack_group == stack_group)
    }

    pub fn stack_groups_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowStackGroupLayout> {
        let mut groups = self
            .stack_groups
            .iter()
            .filter(|group| group.dock_region == dock_region)
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| group.stack_group);
        groups
    }

    pub fn drop_target_for_mutation(
        &self,
        mutation: &StudioGuiWindowLayoutMutation,
    ) -> Option<StudioGuiWindowDropTarget> {
        self.state.drop_target_for_mutation(mutation)
    }

    pub fn drop_target_for_query(
        &self,
        query: &StudioGuiWindowDropTargetQuery,
    ) -> Option<StudioGuiWindowDropTarget> {
        self.state.drop_target_for_query(query)
    }

    pub fn preview_layout_state_for_query(
        &self,
        query: &StudioGuiWindowDropTargetQuery,
    ) -> Option<StudioGuiWindowLayoutState> {
        self.state.preview_layout_state_for_query(query)
    }

    fn from_areas(
        state: &StudioGuiWindowLayoutState,
        header: &StudioGuiWindowHeaderModel,
        commands: &StudioGuiWindowCommandAreaModel,
        canvas: &StudioGuiWindowCanvasAreaModel,
        runtime: &StudioGuiWindowRuntimeAreaModel,
    ) -> Self {
        let titlebar = StudioGuiWindowTitlebarModel {
            title: header.title.to_string(),
            subtitle: header.status_line.clone(),
            foreground_window_id: header.foreground_window_id,
            registered_window_count: header.registered_window_count,
            close_enabled: header.registered_window_count > 0,
        };
        let panels = vec![
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Commands,
                commands.title,
                Some(commands.total_command_count.to_string()),
                format!(
                    "{} commands, {} enabled",
                    commands.total_command_count, commands.enabled_command_count
                ),
            ),
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Canvas,
                canvas.title,
                Some(canvas.suggestion_count.to_string()),
                format!(
                    "{} suggestions, {} actions enabled",
                    canvas.suggestion_count, canvas.enabled_action_count
                ),
            ),
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Runtime,
                runtime.title,
                runtime
                    .platform_notice
                    .as_ref()
                    .map(|_| "!".to_string())
                    .or_else(|| Some(runtime.gui_activity_lines.len().to_string())),
                {
                    let mut summary = format!(
                        "status={:?}, logs={}, activity={}, entitlement={}",
                        runtime.control_state.run_status,
                        runtime.log_entries.len(),
                        runtime.gui_activity_lines.len(),
                        if runtime.entitlement_host.is_some() {
                            "attached"
                        } else {
                            "none"
                        }
                    );
                    if runtime
                        .platform_timer_lines
                        .iter()
                        .any(|line| !line.ends_with("None"))
                    {
                        summary.push_str(", platform-timer=active");
                    }
                    if let Some(notice) = runtime.platform_notice.as_ref() {
                        summary.push_str(&format!(", platform={:?}: {}", notice.level, notice.title));
                    }
                    summary
                },
            ),
        ];
        let stack_groups = build_stack_group_layouts(state, &panels);

        Self {
            state: state.clone(),
            titlebar,
            panels,
            stack_groups,
            region_weights: state.region_weights.clone(),
            center_area: state.center_area,
            default_focus_area: state.default_focus_area,
        }
    }
}

impl StudioGuiWindowModel {
    pub fn layout(&self) -> StudioGuiWindowLayoutModel {
        StudioGuiWindowLayoutModel::from_window_model(self)
    }
}

fn build_panel_layout(
    state: &StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
    title: &'static str,
    badge: Option<String>,
    summary: String,
) -> StudioGuiWindowPanelLayout {
    let panel_state = state
        .panel(area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(area_id));
    let stack_tab_count = state
        .panels_in_stack_group(panel_state.dock_region, panel_state.stack_group)
        .len();
    let active_in_stack = state
        .active_panel_in_stack(panel_state.dock_region, panel_state.stack_group)
        == Some(area_id);
    let display_mode = if stack_tab_count <= 1 {
        StudioGuiWindowPanelDisplayMode::Standalone
    } else if active_in_stack {
        StudioGuiWindowPanelDisplayMode::ActiveTab
    } else {
        StudioGuiWindowPanelDisplayMode::InactiveTab
    };
    StudioGuiWindowPanelLayout {
        area_id,
        title,
        dock_region: panel_state.dock_region,
        stack_group: panel_state.stack_group,
        order: panel_state.order,
        display_mode,
        active_in_stack,
        visible: panel_state.visible,
        collapsed: panel_state.collapsed,
        badge,
        summary,
    }
}
