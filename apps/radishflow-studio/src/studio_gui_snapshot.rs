use std::collections::BTreeMap;

use crate::{
    EntitlementSessionHostRuntimeOutput, StudioAppHostState, StudioAppHostUiCommandModel,
    StudioExampleProjectModel, StudioGuiCanvasWidgetModel, StudioGuiCommandRegistry,
    StudioGuiWindowDropPreviewState, StudioGuiWindowLayoutState, WorkspaceControlState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWorkspaceDocumentSnapshot {
    pub document_id: String,
    pub title: String,
    pub flowsheet_name: String,
    pub revision: u64,
    pub last_saved_revision: Option<u64>,
    pub has_unsaved_changes: bool,
    pub project_path: Option<String>,
    pub unit_count: usize,
    pub stream_count: usize,
    pub snapshot_history_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiRuntimeSnapshot {
    pub workspace_document: StudioGuiWorkspaceDocumentSnapshot,
    pub example_projects: Vec<StudioExampleProjectModel>,
    pub control_state: WorkspaceControlState,
    pub run_panel: rf_ui::RunPanelWidgetModel,
    pub latest_solve_snapshot: Option<rf_ui::SolveSnapshot>,
    pub active_inspector_target: Option<rf_ui::InspectorTarget>,
    pub active_inspector_detail: Option<StudioGuiInspectorTargetDetailSnapshot>,
    pub entitlement_host: Option<EntitlementSessionHostRuntimeOutput>,
    pub platform_notice: Option<rf_ui::RunPanelNotice>,
    pub platform_timer_lines: Vec<String>,
    pub gui_activity_lines: Vec<String>,
    pub log_entries: Vec<rf_ui::AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorTargetDetailSnapshot {
    pub target: rf_ui::InspectorTarget,
    pub title: String,
    pub summary_rows: Vec<StudioGuiInspectorTargetSummaryRowSnapshot>,
    pub property_fields: Vec<StudioGuiInspectorTargetFieldSnapshot>,
    pub property_notices: Vec<StudioGuiInspectorPropertyNoticeSnapshot>,
    pub property_composition_summary: Option<StudioGuiInspectorCompositionSummarySnapshot>,
    pub property_batch_commit_command_id: Option<String>,
    pub property_composition_normalize_command_id: Option<String>,
    pub unit_ports: Vec<StudioGuiInspectorTargetPortSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorTargetSummaryRowSnapshot {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorTargetFieldSnapshot {
    pub key: String,
    pub label: String,
    pub value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot,
    pub original_value: String,
    pub current_value: String,
    pub is_dirty: bool,
    pub validation: StudioGuiInspectorTargetFieldValidationSnapshot,
    pub draft_update_command_id: String,
    pub commit_command_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorPropertyNoticeSnapshot {
    pub status_label: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorCompositionSummarySnapshot {
    pub current_sum_text: String,
    pub normalized_preview_text: String,
    pub status_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiInspectorTargetFieldValueKindSnapshot {
    Text,
    Number,
    Choice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiInspectorTargetFieldValidationSnapshot {
    Unknown,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiInspectorTargetPortSnapshot {
    pub name: String,
    pub direction: String,
    pub kind: String,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiSnapshot {
    pub app_host_state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasWidgetModel,
    pub runtime: StudioGuiRuntimeSnapshot,
    pub layout_state: StudioGuiWindowLayoutState,
    pub window_drop_previews: BTreeMap<String, StudioGuiWindowDropPreviewState>,
}

impl StudioGuiSnapshot {
    pub fn new(
        app_host_state: StudioAppHostState,
        ui_commands: StudioAppHostUiCommandModel,
        command_registry: StudioGuiCommandRegistry,
        canvas: StudioGuiCanvasWidgetModel,
        runtime: StudioGuiRuntimeSnapshot,
        window_drop_previews: BTreeMap<String, StudioGuiWindowDropPreviewState>,
    ) -> Self {
        let mut snapshot = Self {
            app_host_state,
            ui_commands,
            command_registry,
            canvas,
            runtime,
            layout_state: StudioGuiWindowLayoutState::default(),
            window_drop_previews,
        };
        snapshot.layout_state = StudioGuiWindowLayoutState::from_snapshot(&snapshot);
        snapshot
    }
}
