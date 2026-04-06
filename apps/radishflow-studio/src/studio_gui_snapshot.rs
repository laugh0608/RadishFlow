use crate::{
    EntitlementSessionHostRuntimeOutput, StudioAppHostState, StudioAppHostUiCommandModel,
    StudioGuiCanvasWidgetModel, StudioGuiCommandRegistry, WorkspaceControlState,
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
}
