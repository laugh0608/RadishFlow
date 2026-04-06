use crate::{
    EntitlementSessionHostRuntimeOutput, StudioAppHostState, StudioAppHostUiCommandModel,
    StudioGuiCanvasWidgetModel, StudioGuiCommandRegistry, StudioGuiWindowLayoutState,
    WorkspaceControlState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiRuntimeSnapshot {
    pub control_state: WorkspaceControlState,
    pub run_panel: rf_ui::RunPanelWidgetModel,
    pub entitlement_host: Option<EntitlementSessionHostRuntimeOutput>,
    pub log_entries: Vec<rf_ui::AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiSnapshot {
    pub app_host_state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasWidgetModel,
    pub runtime: StudioGuiRuntimeSnapshot,
    pub layout_state: StudioGuiWindowLayoutState,
}

impl StudioGuiSnapshot {
    pub fn new(
        app_host_state: StudioAppHostState,
        ui_commands: StudioAppHostUiCommandModel,
        command_registry: StudioGuiCommandRegistry,
        canvas: StudioGuiCanvasWidgetModel,
        runtime: StudioGuiRuntimeSnapshot,
    ) -> Self {
        let mut snapshot = Self {
            app_host_state,
            ui_commands,
            command_registry,
            canvas,
            runtime,
            layout_state: StudioGuiWindowLayoutState::default(),
        };
        snapshot.layout_state = StudioGuiWindowLayoutState::from_snapshot(&snapshot);
        snapshot
    }
}
