use std::collections::BTreeSet;

mod drop_preview;
mod result_inspector;
use crate::{
    EntitlementSessionHostRuntimeOutput, StudioExampleProjectModel, StudioGuiCanvasWidgetModel,
    StudioGuiCommandEntry, StudioGuiCommandMenuNode, StudioGuiCommandRegistry,
    StudioGuiCommandSection, StudioGuiDiagnosticStreamSnapshot,
    StudioGuiFailureDiagnosticContextSnapshot, StudioGuiSnapshot, StudioGuiWindowAreaId,
    StudioGuiWindowDockRegion, StudioGuiWindowDropTarget, StudioGuiWindowDropTargetKind,
    StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutModel, StudioGuiWindowLayoutState,
    StudioGuiWorkspaceDocumentSnapshot, StudioWindowHostId, WorkspaceControlState,
};
use drop_preview::{build_drop_preview_overlay, changed_area_ids_for_preview};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowHeaderModel {
    pub title: &'static str,
    pub status_line: String,
    pub registered_window_count: usize,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer_owner_window_id: Option<StudioWindowHostId>,
    pub has_parked_entitlement_timer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandAreaModel {
    pub title: &'static str,
    pub command_list_sections: Vec<StudioGuiWindowCommandListSectionModel>,
    pub toolbar_sections: Vec<StudioGuiWindowToolbarSectionModel>,
    pub menu_tree: Vec<StudioGuiCommandMenuNode>,
    pub total_command_count: usize,
    pub enabled_command_count: usize,
    sections: Vec<StudioGuiCommandSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandListSectionModel {
    pub title: &'static str,
    pub items: Vec<StudioGuiWindowCommandListItemModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandListItemModel {
    pub command_id: String,
    pub enabled: bool,
    pub label: String,
    pub detail: String,
    pub menu_path_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowToolbarSectionModel {
    pub title: &'static str,
    pub items: Vec<StudioGuiWindowToolbarItemModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowToolbarItemModel {
    pub command_id: String,
    pub enabled: bool,
    pub label: String,
    pub hover_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandPaletteItemModel {
    pub command_id: String,
    pub enabled: bool,
    pub label: String,
    pub detail: String,
    pub menu_path_text: String,
    pub hover_text: String,
}

impl StudioGuiWindowCommandAreaModel {
    fn filtered_commands(&self, query: &str) -> Vec<&StudioGuiCommandEntry> {
        self.sections
            .iter()
            .flat_map(|section| section.commands.iter())
            .filter(|command| command.matches_palette_query(query))
            .collect()
    }

    pub fn palette_items(&self, query: &str) -> Vec<StudioGuiWindowCommandPaletteItemModel> {
        self.filtered_commands(query)
            .into_iter()
            .map(|command| {
                let presentation = command.presentation();
                StudioGuiWindowCommandPaletteItemModel {
                    command_id: command.command_id.clone(),
                    enabled: command.enabled,
                    label: presentation.palette_label,
                    detail: command.detail.clone(),
                    menu_path_text: presentation.menu_path_text,
                    hover_text: presentation.hover_text,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowCanvasAreaModel {
    pub title: &'static str,
    pub widget: StudioGuiCanvasWidgetModel,
    pub focused_suggestion_id: Option<String>,
    pub suggestion_count: usize,
    pub enabled_action_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowRuntimeAreaModel {
    pub title: &'static str,
    pub workspace_document: StudioGuiWorkspaceDocumentSnapshot,
    pub example_projects: Vec<StudioExampleProjectModel>,
    pub control_state: WorkspaceControlState,
    pub run_panel: rf_ui::RunPanelWidgetModel,
    pub latest_solve_snapshot: Option<StudioGuiWindowSolveSnapshotModel>,
    pub latest_failure: Option<StudioGuiWindowFailureResultModel>,
    pub active_inspector_target: Option<StudioGuiWindowInspectorTargetModel>,
    pub active_inspector_detail: Option<StudioGuiWindowInspectorTargetDetailModel>,
    pub entitlement_host: Option<EntitlementSessionHostRuntimeOutput>,
    pub platform_notice: Option<rf_ui::RunPanelNotice>,
    pub platform_timer_lines: Vec<String>,
    pub gui_activity_lines: Vec<String>,
    pub log_entries: Vec<rf_ui::AppLogEntry>,
    pub latest_log_entry: Option<rf_ui::AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowSolveSnapshotModel {
    pub snapshot_id: String,
    pub sequence: u64,
    pub status_label: &'static str,
    pub summary: String,
    pub diagnostic_count: usize,
    pub step_count: usize,
    pub stream_count: usize,
    pub streams: Vec<StudioGuiWindowStreamResultModel>,
    pub steps: Vec<StudioGuiWindowSolveStepModel>,
    pub diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowFailureResultModel {
    pub status_label: &'static str,
    pub title: String,
    pub message: String,
    pub diagnostic_detail: Option<StudioGuiWindowFailureDiagnosticDetailModel>,
    pub recovery_title: Option<&'static str>,
    pub recovery_detail: Option<&'static str>,
    pub recovery_action: Option<StudioGuiWindowCommandActionModel>,
    pub recovery_target: Option<StudioGuiWindowInspectorTargetModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
    pub latest_log_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowFailureDiagnosticDetailModel {
    pub document_revision: u64,
    pub severity_label: &'static str,
    pub primary_code: Option<String>,
    pub diagnostic_count: usize,
    pub related_units: Vec<StudioGuiWindowInspectorTargetModel>,
    pub related_streams: Vec<StudioGuiWindowInspectorTargetModel>,
    pub related_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub related_ports: Vec<StudioGuiWindowFailureDiagnosticPortTargetModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowFailureDiagnosticPortTargetModel {
    pub unit_id: String,
    pub port_name: String,
    pub summary: String,
    pub unit_action: StudioGuiWindowCommandActionModel,
    pub stream_result: Option<StudioGuiWindowStreamResultReferenceModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandActionModel {
    pub label: String,
    pub hover_text: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorTargetModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub summary: String,
    pub command_id: String,
    pub action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDiagnosticTargetActionModel {
    pub source_label: &'static str,
    pub target_label: &'static str,
    pub summary: String,
    pub action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowInspectorTargetDetailModel {
    pub target: StudioGuiWindowInspectorTargetModel,
    pub title: String,
    pub summary_rows: Vec<StudioGuiWindowInspectorTargetSummaryRowModel>,
    pub property_fields: Vec<StudioGuiWindowInspectorTargetFieldModel>,
    pub property_notices: Vec<StudioGuiWindowInspectorPropertyNoticeModel>,
    pub property_composition_summary: Option<StudioGuiWindowInspectorCompositionSummaryModel>,
    pub property_batch_commit_command_id: Option<String>,
    pub property_batch_discard_command_id: Option<String>,
    pub property_composition_normalize_command_id: Option<String>,
    pub property_composition_component_actions:
        Vec<StudioGuiWindowInspectorCompositionComponentActionModel>,
    pub unit_ports: Vec<StudioGuiWindowInspectorTargetPortModel>,
    pub latest_unit_result: Option<StudioGuiWindowUnitExecutionResultModel>,
    pub latest_stream_result: Option<StudioGuiWindowStreamResultModel>,
    pub related_steps: Vec<StudioGuiWindowSolveStepModel>,
    pub related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorTargetSummaryRowModel {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorTargetFieldModel {
    pub key: String,
    pub label: String,
    pub value_kind_label: &'static str,
    pub original_value: String,
    pub current_value: String,
    pub status_label: &'static str,
    pub is_dirty: bool,
    pub draft_update_command_id: String,
    pub commit_command_id: Option<String>,
    pub discard_command_id: Option<String>,
    pub remove_command_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorPropertyNoticeModel {
    pub status_label: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorCompositionSummaryModel {
    pub current_sum_text: String,
    pub normalized_preview_text: String,
    pub status_label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorCompositionComponentActionModel {
    pub component_id: String,
    pub component_name: String,
    pub action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorTargetPortModel {
    pub name: String,
    pub direction: String,
    pub kind: String,
    pub stream_id: Option<String>,
    pub stream_action: Option<StudioGuiWindowCommandActionModel>,
    pub attention_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowUnitExecutionResultModel {
    pub unit_id: String,
    pub step_index: usize,
    pub status_label: &'static str,
    pub summary: String,
    pub consumed_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub consumed_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
    pub produced_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub produced_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStreamResultReferenceModel {
    pub stream_id: String,
    pub summary: String,
    pub focus_action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowStreamResultModel {
    pub stream_id: String,
    pub label: String,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub total_molar_flow_mol_s: f64,
    pub molar_enthalpy_j_per_mol: Option<f64>,
    pub temperature_text: String,
    pub pressure_text: String,
    pub molar_flow_text: String,
    pub molar_enthalpy_text: Option<String>,
    pub bubble_dew_window: Option<StudioGuiWindowBubbleDewWindowModel>,
    pub summary_rows: Vec<StudioGuiWindowStreamSummaryRowModel>,
    pub composition_rows: Vec<StudioGuiWindowCompositionResultModel>,
    pub phase_rows: Vec<StudioGuiWindowPhaseResultModel>,
    pub composition_text: String,
    pub phase_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowBubbleDewWindowModel {
    pub phase_region: String,
    pub bubble_pressure_pa: f64,
    pub dew_pressure_pa: f64,
    pub bubble_temperature_k: f64,
    pub dew_temperature_k: f64,
    pub bubble_pressure_text: String,
    pub dew_pressure_text: String,
    pub bubble_temperature_text: String,
    pub dew_temperature_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStreamSummaryRowModel {
    pub label: &'static str,
    pub detail_label: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowCompositionResultModel {
    pub component_id: String,
    pub fraction: f64,
    pub fraction_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowPhaseResultModel {
    pub label: String,
    pub phase_fraction: f64,
    pub phase_fraction_text: String,
    pub molar_flow_mol_s: f64,
    pub molar_flow_text: String,
    pub composition_text: String,
    pub molar_enthalpy_j_per_mol: Option<f64>,
    pub molar_enthalpy_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowSolveStepModel {
    pub index: usize,
    pub unit_id: String,
    pub summary: String,
    pub execution_status_label: &'static str,
    pub consumed_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub consumed_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
    pub unit_action: StudioGuiWindowCommandActionModel,
    pub produced_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub produced_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
}

impl StudioGuiWindowSolveStepModel {
    fn consumed_stream_ids(&self) -> impl Iterator<Item = &str> {
        self.consumed_stream_results
            .iter()
            .map(|stream| stream.stream_id.as_str())
    }

    fn produced_stream_ids(&self) -> impl Iterator<Item = &str> {
        self.produced_stream_results
            .iter()
            .map(|stream| stream.stream_id.as_str())
    }

    fn has_related_stream_id(&self, stream_id: &str) -> bool {
        self.consumed_stream_ids()
            .chain(self.produced_stream_ids())
            .any(|candidate| candidate == stream_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDiagnosticModel {
    pub severity_label: &'static str,
    pub code: String,
    pub message: String,
    pub related_unit_ids: Vec<String>,
    pub related_stream_ids: Vec<String>,
    pub related_stream_results: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub target_candidates: Vec<StudioGuiWindowInspectorTargetModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
    pub related_units_text: Option<String>,
    pub related_streams_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowResultInspectorModel {
    pub snapshot_id: String,
    pub selected_stream_id: Option<String>,
    pub selected_stream: Option<StudioGuiWindowStreamResultModel>,
    pub comparison_stream_id: Option<String>,
    pub comparison_stream: Option<StudioGuiWindowStreamResultModel>,
    pub stream_options: Vec<StudioGuiWindowResultInspectorStreamOptionModel>,
    pub comparison_options: Vec<StudioGuiWindowResultInspectorStreamOptionModel>,
    pub comparison: Option<StudioGuiWindowResultInspectorComparisonModel>,
    pub related_steps: Vec<StudioGuiWindowSolveStepModel>,
    pub related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
    pub has_stale_selection: bool,
    pub has_stale_comparison: bool,
    pub unit_options: Vec<StudioGuiWindowResultInspectorUnitOptionModel>,
    pub selected_unit_id: Option<String>,
    pub selected_unit: Option<StudioGuiWindowUnitExecutionResultModel>,
    pub unit_related_steps: Vec<StudioGuiWindowSolveStepModel>,
    pub unit_related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub unit_diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
    pub has_stale_unit_selection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowResultInspectorStreamOptionModel {
    pub stream_id: String,
    pub label: String,
    pub summary: String,
    pub is_selected: bool,
    pub focus_action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowResultInspectorUnitOptionModel {
    pub unit_id: String,
    pub status_label: &'static str,
    pub step_index: usize,
    pub summary: String,
    pub is_selected: bool,
    pub focus_action: StudioGuiWindowCommandActionModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowResultInspectorComparisonModel {
    pub base_stream_id: String,
    pub compared_stream_id: String,
    pub base_stream_focus_action: StudioGuiWindowCommandActionModel,
    pub compared_stream_focus_action: StudioGuiWindowCommandActionModel,
    pub summary_rows: Vec<StudioGuiWindowResultInspectorComparisonRowModel>,
    pub composition_rows: Vec<StudioGuiWindowResultInspectorCompositionComparisonRowModel>,
    pub phase_rows: Vec<StudioGuiWindowResultInspectorPhaseComparisonRowModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowResultInspectorComparisonRowModel {
    pub label: &'static str,
    pub detail_label: &'static str,
    pub base_value: String,
    pub compared_value: String,
    pub delta_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowResultInspectorCompositionComparisonRowModel {
    pub component_id: String,
    pub base_fraction_text: String,
    pub compared_fraction_text: String,
    pub delta_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowResultInspectorPhaseComparisonRowModel {
    pub phase_label: String,
    pub base_fraction_text: String,
    pub compared_fraction_text: String,
    pub fraction_delta_text: String,
    pub base_molar_flow_text: String,
    pub compared_molar_flow_text: String,
    pub molar_flow_delta_text: String,
    pub base_molar_enthalpy_text: String,
    pub compared_molar_enthalpy_text: String,
    pub molar_enthalpy_delta_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDropPreviewState {
    pub query: StudioGuiWindowDropTargetQuery,
    pub drop_target: StudioGuiWindowDropTarget,
    pub preview_layout_state: StudioGuiWindowLayoutState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDropPreviewOverlayModel {
    pub drag_area_id: StudioGuiWindowAreaId,
    pub kind: StudioGuiWindowDropTargetKind,
    pub target_dock_region: StudioGuiWindowDockRegion,
    pub target_stack_group: u8,
    pub target_group_index: usize,
    pub target_tab_index: usize,
    pub target_stack_area_ids: Vec<StudioGuiWindowAreaId>,
    pub target_stack_active_area_id: StudioGuiWindowAreaId,
    pub highlighted_area_ids: Vec<StudioGuiWindowAreaId>,
    pub anchor_area_id: Option<StudioGuiWindowAreaId>,
    pub creates_new_stack: bool,
    pub merges_into_existing_stack: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDropPreviewModel {
    pub query: StudioGuiWindowDropTargetQuery,
    pub drop_target: StudioGuiWindowDropTarget,
    pub overlay: StudioGuiWindowDropPreviewOverlayModel,
    pub preview_layout_state: StudioGuiWindowLayoutState,
    pub preview_layout: StudioGuiWindowLayoutModel,
    pub changed_area_ids: Vec<StudioGuiWindowAreaId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowModel {
    pub header: StudioGuiWindowHeaderModel,
    pub commands: StudioGuiWindowCommandAreaModel,
    pub canvas: StudioGuiWindowCanvasAreaModel,
    pub runtime: StudioGuiWindowRuntimeAreaModel,
    pub layout_state: StudioGuiWindowLayoutState,
    pub drop_preview: Option<StudioGuiWindowDropPreviewModel>,
}

impl StudioGuiWindowModel {
    pub fn from_snapshot(snapshot: &StudioGuiSnapshot) -> Self {
        Self::from_snapshot_for_window(snapshot, None)
    }

    pub fn from_snapshot_for_window(
        snapshot: &StudioGuiSnapshot,
        window_id: Option<StudioWindowHostId>,
    ) -> Self {
        let derived_layout_state =
            StudioGuiWindowLayoutState::from_snapshot_for_window(snapshot, window_id);
        let layout_state =
            if snapshot.layout_state.scope.layout_key == derived_layout_state.scope.layout_key {
                snapshot.layout_state.clone()
            } else {
                derived_layout_state
            };
        let drop_preview_state = snapshot
            .window_drop_previews
            .get(&layout_state.scope.layout_key)
            .cloned()
            .or_else(|| {
                layout_state
                    .scope
                    .legacy_layout_key()
                    .as_ref()
                    .and_then(|layout_key| snapshot.window_drop_previews.get(layout_key))
                    .cloned()
            });
        let mut window = Self {
            header: header_from_snapshot(snapshot),
            commands: commands_from_registry(&snapshot.command_registry),
            canvas: canvas_from_snapshot(snapshot),
            runtime: runtime_from_snapshot(snapshot),
            layout_state,
            drop_preview: None,
        };
        window.drop_preview = drop_preview_state.map(|preview| {
            let preview_layout = StudioGuiWindowLayoutModel::from_window_model_with_layout_state(
                &window,
                &preview.preview_layout_state,
            );
            StudioGuiWindowDropPreviewModel {
                query: preview.query,
                overlay: build_drop_preview_overlay(&preview_layout, &preview.drop_target),
                drop_target: preview.drop_target,
                changed_area_ids: changed_area_ids_for_preview(
                    &window.layout_state,
                    &preview.preview_layout_state,
                ),
                preview_layout_state: preview.preview_layout_state,
                preview_layout,
            }
        });
        window
    }

    pub fn with_layout_state(&self, layout_state: StudioGuiWindowLayoutState) -> Self {
        let mut window = self.clone();
        window.layout_state = layout_state;
        window.drop_preview = None;
        window
    }
}

impl StudioGuiSnapshot {
    pub fn window_model(&self) -> StudioGuiWindowModel {
        StudioGuiWindowModel::from_snapshot(self)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        StudioGuiWindowModel::from_snapshot_for_window(self, window_id)
    }
}

fn header_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowHeaderModel {
    let state = &snapshot.app_host_state;
    let registered_window_count = state.registered_windows.len();
    let foreground_window_id = state.foreground_window_id;
    let entitlement_timer_owner_window_id = state.entitlement_timer_owner_window_id();
    let has_parked_entitlement_timer = state.parked_entitlement_timer().is_some();
    let status_line = [
        format!("registered windows: {registered_window_count}"),
        foreground_window_id
            .map(|window_id| format!("foreground: #{window_id}"))
            .unwrap_or_else(|| "foreground: none".to_string()),
        match entitlement_timer_owner_window_id {
            Some(window_id) => format!("timer owner: #{window_id}"),
            None if has_parked_entitlement_timer => "timer owner: parked".to_string(),
            None => "timer owner: none".to_string(),
        },
    ]
    .join(" | ");

    StudioGuiWindowHeaderModel {
        title: "RadishFlow Studio",
        status_line,
        registered_window_count,
        foreground_window_id,
        entitlement_timer_owner_window_id,
        has_parked_entitlement_timer,
    }
}

fn commands_from_registry(registry: &StudioGuiCommandRegistry) -> StudioGuiWindowCommandAreaModel {
    let total_command_count = registry
        .sections
        .iter()
        .map(|section| section.commands.len())
        .sum();
    let enabled_command_count = registry
        .sections
        .iter()
        .flat_map(|section| section.commands.iter())
        .filter(|command| command.enabled)
        .count();

    StudioGuiWindowCommandAreaModel {
        title: "Commands",
        sections: registry.sections.clone(),
        command_list_sections: command_list_sections_from_sections(&registry.sections),
        toolbar_sections: toolbar_sections_from_sections(&registry.sections),
        menu_tree: registry.menu_tree(),
        total_command_count,
        enabled_command_count,
    }
}

fn command_list_sections_from_sections(
    sections: &[StudioGuiCommandSection],
) -> Vec<StudioGuiWindowCommandListSectionModel> {
    sections
        .iter()
        .filter_map(|section| {
            let items = section
                .commands
                .iter()
                .map(|command| {
                    let presentation = command.presentation();
                    StudioGuiWindowCommandListItemModel {
                        command_id: command.command_id.clone(),
                        enabled: command.enabled,
                        label: presentation.label_with_shortcut,
                        detail: command.detail.clone(),
                        menu_path_text: presentation.menu_path_text,
                    }
                })
                .collect::<Vec<_>>();
            if items.is_empty() {
                None
            } else {
                Some(StudioGuiWindowCommandListSectionModel {
                    title: section.title,
                    items,
                })
            }
        })
        .collect()
}

fn toolbar_sections_from_sections(
    sections: &[StudioGuiCommandSection],
) -> Vec<StudioGuiWindowToolbarSectionModel> {
    sections
        .iter()
        .filter_map(|section| {
            let items = section
                .commands
                .iter()
                .map(|command| {
                    let presentation = command.presentation();
                    StudioGuiWindowToolbarItemModel {
                        command_id: command.command_id.clone(),
                        enabled: command.enabled,
                        label: presentation.label,
                        hover_text: presentation.hover_text,
                    }
                })
                .collect::<Vec<_>>();
            if items.is_empty() {
                None
            } else {
                Some(StudioGuiWindowToolbarSectionModel {
                    title: section.title,
                    items,
                })
            }
        })
        .collect()
}

fn canvas_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowCanvasAreaModel {
    let widget = snapshot.canvas.clone();
    let focused_suggestion_id = widget.view().focused_suggestion_id.clone();
    let suggestion_count = widget.view().suggestion_count;
    let enabled_action_count = widget
        .actions
        .iter()
        .filter(|action| action.enabled)
        .count();

    StudioGuiWindowCanvasAreaModel {
        title: "Canvas",
        widget,
        focused_suggestion_id,
        suggestion_count,
        enabled_action_count,
    }
}

fn runtime_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowRuntimeAreaModel {
    let latest_solve_snapshot = snapshot
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .map(solve_snapshot_model_from_ui);
    let latest_failure = latest_solve_snapshot
        .is_none()
        .then(|| {
            failure_result_model_from_control_state(
                &snapshot.runtime.control_state,
                snapshot.runtime.latest_failure_diagnostic_context.as_ref(),
            )
        })
        .flatten();
    let active_inspector_detail = snapshot
        .runtime
        .active_inspector_detail
        .as_ref()
        .map(|detail| {
            let latest_diagnostic_for_document = snapshot
                .runtime
                .control_state
                .latest_diagnostic
                .as_ref()
                .filter(|diagnostic| {
                    diagnostic.document_revision == snapshot.runtime.workspace_document.revision
                });
            inspector_target_detail_model_from_snapshot(
                detail,
                latest_solve_snapshot.as_ref(),
                latest_diagnostic_for_document,
            )
        });

    StudioGuiWindowRuntimeAreaModel {
        title: "Runtime",
        workspace_document: snapshot.runtime.workspace_document.clone(),
        example_projects: snapshot.runtime.example_projects.clone(),
        control_state: snapshot.runtime.control_state.clone(),
        run_panel: snapshot.runtime.run_panel.clone(),
        latest_solve_snapshot,
        latest_failure,
        active_inspector_target: snapshot
            .runtime
            .active_inspector_target
            .as_ref()
            .map(inspector_target_model_from_ui),
        active_inspector_detail,
        entitlement_host: snapshot.runtime.entitlement_host.clone(),
        platform_notice: snapshot.runtime.platform_notice.clone(),
        platform_timer_lines: snapshot.runtime.platform_timer_lines.clone(),
        gui_activity_lines: snapshot.runtime.gui_activity_lines.clone(),
        latest_log_entry: snapshot.runtime.log_entries.last().cloned(),
        log_entries: snapshot.runtime.log_entries.clone(),
    }
}

fn failure_result_model_from_control_state(
    control_state: &WorkspaceControlState,
    diagnostic_context: Option<&StudioGuiFailureDiagnosticContextSnapshot>,
) -> Option<StudioGuiWindowFailureResultModel> {
    if !matches!(control_state.run_status, rf_ui::RunStatus::Error) {
        return None;
    }

    let notice = control_state.notice.as_ref()?;
    if !matches!(notice.level, rf_ui::RunPanelNoticeLevel::Error) {
        return None;
    }

    let recovery_action = notice
        .recovery_action
        .as_ref()
        .map(failure_recovery_action_model);
    let recovery_target = notice
        .recovery_action
        .as_ref()
        .and_then(inspector_target_model_from_recovery_action);
    let mut diagnostic_actions = Vec::new();
    if let Some(action) = recovery_action.as_ref() {
        diagnostic_actions.push(StudioGuiWindowDiagnosticTargetActionModel {
            source_label: "Recovery",
            target_label: "Run panel",
            summary: notice.title.clone(),
            action: action.clone(),
        });
    }
    if let Some(target) = recovery_target.as_ref() {
        diagnostic_actions.push(diagnostic_target_action_from_target(
            "Recovery target",
            target,
        ));
    }
    let diagnostic_detail = control_state
        .latest_diagnostic
        .as_ref()
        .map(|summary| failure_diagnostic_detail_model_from_summary(summary, diagnostic_context));
    if let Some(detail) = diagnostic_detail.as_ref() {
        diagnostic_actions.extend(failure_diagnostic_actions(detail));
    }
    diagnostic_actions = dedupe_diagnostic_actions(diagnostic_actions);

    Some(StudioGuiWindowFailureResultModel {
        status_label: run_status_label(control_state.run_status),
        title: notice.title.clone(),
        message: notice.message.clone(),
        diagnostic_detail,
        recovery_title: notice.recovery_action.as_ref().map(|action| action.title),
        recovery_detail: notice.recovery_action.as_ref().map(|action| action.detail),
        recovery_action,
        recovery_target,
        diagnostic_actions,
        latest_log_message: control_state
            .latest_log_entry
            .as_ref()
            .map(|entry| entry.message.clone()),
    })
}

fn failure_diagnostic_detail_model_from_summary(
    summary: &rf_ui::DiagnosticSummary,
    diagnostic_context: Option<&StudioGuiFailureDiagnosticContextSnapshot>,
) -> StudioGuiWindowFailureDiagnosticDetailModel {
    StudioGuiWindowFailureDiagnosticDetailModel {
        document_revision: summary.document_revision,
        severity_label: diagnostic_severity_label(summary.highest_severity),
        primary_code: summary.primary_code.clone(),
        diagnostic_count: summary.diagnostic_count,
        related_units: summary
            .related_unit_ids
            .iter()
            .map(|unit_id| {
                inspector_target_model_from_ui(&rf_ui::InspectorTarget::Unit(unit_id.clone()))
            })
            .collect(),
        related_streams: summary
            .related_stream_ids
            .iter()
            .map(|stream_id| {
                inspector_target_model_from_ui(&rf_ui::InspectorTarget::Stream(stream_id.clone()))
            })
            .collect(),
        related_stream_results: diagnostic_context
            .map(|context| {
                context
                    .related_streams
                    .iter()
                    .map(stream_result_reference_model_from_diagnostic_snapshot)
                    .collect()
            })
            .unwrap_or_default(),
        related_ports: summary
            .related_port_targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let unit_id = target.unit_id.as_str().to_string();
                let port_name = target.port_name.clone();
                StudioGuiWindowFailureDiagnosticPortTargetModel {
                    unit_id: unit_id.clone(),
                    port_name: port_name.clone(),
                    summary: format!("Unit {unit_id} port {port_name}"),
                    unit_action: inspector_unit_action(&unit_id),
                    stream_result: diagnostic_context
                        .and_then(|context| context.related_ports.get(index))
                        .and_then(|port| port.stream.as_ref())
                        .map(stream_result_reference_model_from_diagnostic_snapshot),
                }
            })
            .collect(),
    }
}

fn failure_diagnostic_actions(
    detail: &StudioGuiWindowFailureDiagnosticDetailModel,
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let unit_actions = detail
        .related_units
        .iter()
        .map(|target| diagnostic_target_action_from_target("Failure diagnostic", target));
    let stream_actions = detail
        .related_streams
        .iter()
        .map(|target| diagnostic_target_action_from_target("Failure diagnostic", target));
    let port_actions = detail.related_ports.iter().map(|port| {
        diagnostic_target_action_from_action(
            "Failure port",
            "Port",
            port.summary.clone(),
            &port.unit_action,
        )
    });

    dedupe_diagnostic_actions(unit_actions.chain(stream_actions).chain(port_actions))
}

fn failure_recovery_action_model(
    action: &rf_ui::RunPanelRecoveryAction,
) -> StudioGuiWindowCommandActionModel {
    StudioGuiWindowCommandActionModel {
        label: action.title.to_string(),
        hover_text: action.detail.to_string(),
        command_id: "run_panel.recover_failure".to_string(),
    }
}

fn inspector_target_model_from_recovery_action(
    action: &rf_ui::RunPanelRecoveryAction,
) -> Option<StudioGuiWindowInspectorTargetModel> {
    if let Some(unit_id) = action.target_unit_id.as_ref() {
        return Some(inspector_target_model_from_ui(
            &rf_ui::InspectorTarget::Unit(unit_id.clone()),
        ));
    }
    if let Some(stream_id) = action.target_stream_id.as_ref() {
        return Some(inspector_target_model_from_ui(
            &rf_ui::InspectorTarget::Stream(stream_id.clone()),
        ));
    }
    None
}

fn inspector_target_model_from_ui(
    target: &rf_ui::InspectorTarget,
) -> StudioGuiWindowInspectorTargetModel {
    match target {
        rf_ui::InspectorTarget::Unit(unit_id) => {
            let target_id = unit_id.as_str().to_string();
            let summary = format!("Unit {target_id}");
            let command_id = crate::inspector_target_command_id(target);
            StudioGuiWindowInspectorTargetModel {
                kind_label: "Unit",
                target_id: target_id.clone(),
                summary: summary.clone(),
                command_id: command_id.clone(),
                action: StudioGuiWindowCommandActionModel {
                    label: format!("Unit {target_id}"),
                    hover_text: summary,
                    command_id,
                },
            }
        }
        rf_ui::InspectorTarget::Stream(stream_id) => {
            let target_id = stream_id.as_str().to_string();
            let summary = format!("Stream {target_id}");
            let command_id = crate::inspector_target_command_id(target);
            StudioGuiWindowInspectorTargetModel {
                kind_label: "Stream",
                target_id: target_id.clone(),
                summary: summary.clone(),
                command_id: command_id.clone(),
                action: StudioGuiWindowCommandActionModel {
                    label: format!("Stream {target_id}"),
                    hover_text: summary,
                    command_id,
                },
            }
        }
    }
}

fn inspector_target_detail_model_from_snapshot(
    detail: &crate::StudioGuiInspectorTargetDetailSnapshot,
    latest_solve_snapshot: Option<&StudioGuiWindowSolveSnapshotModel>,
    latest_diagnostic: Option<&rf_ui::DiagnosticSummary>,
) -> StudioGuiWindowInspectorTargetDetailModel {
    let target = inspector_target_model_from_ui(&detail.target);
    let latest_unit_result = latest_solve_snapshot
        .and_then(|snapshot| latest_unit_result_for_target(snapshot, &detail.target));
    let latest_stream_result = match &detail.target {
        rf_ui::InspectorTarget::Stream(stream_id) => latest_solve_snapshot.and_then(|snapshot| {
            snapshot
                .streams
                .iter()
                .find(|stream| stream.stream_id == stream_id.as_str())
                .cloned()
        }),
        rf_ui::InspectorTarget::Unit(_) => None,
    };
    let related_steps = latest_solve_snapshot
        .map(|snapshot| related_steps_for_target(snapshot, &detail.target))
        .unwrap_or_default();
    let related_diagnostics = latest_solve_snapshot
        .map(|snapshot| related_diagnostics_for_target(snapshot, &detail.target))
        .unwrap_or_default();
    let diagnostic_actions = inspector_detail_diagnostic_actions(
        &target,
        latest_unit_result.as_ref(),
        &related_steps,
        &related_diagnostics,
    );

    StudioGuiWindowInspectorTargetDetailModel {
        target,
        title: detail.title.clone(),
        summary_rows: detail
            .summary_rows
            .iter()
            .map(|row| StudioGuiWindowInspectorTargetSummaryRowModel {
                label: row.label.clone(),
                value: row.value.clone(),
            })
            .collect(),
        property_fields: detail
            .property_fields
            .iter()
            .map(inspector_field_model_from_snapshot)
            .collect(),
        property_notices: detail
            .property_notices
            .iter()
            .map(inspector_property_notice_model_from_snapshot)
            .collect(),
        property_composition_summary: detail
            .property_composition_summary
            .as_ref()
            .map(inspector_composition_summary_model_from_snapshot),
        property_batch_commit_command_id: detail.property_batch_commit_command_id.clone(),
        property_batch_discard_command_id: detail.property_batch_discard_command_id.clone(),
        property_composition_normalize_command_id: detail
            .property_composition_normalize_command_id
            .clone(),
        property_composition_component_actions: detail
            .property_composition_component_actions
            .iter()
            .map(inspector_composition_component_action_model_from_snapshot)
            .collect(),
        unit_ports: detail
            .unit_ports
            .iter()
            .map(|port| StudioGuiWindowInspectorTargetPortModel {
                name: port.name.clone(),
                direction: port.direction.clone(),
                kind: port.kind.clone(),
                stream_id: port.stream_id.clone(),
                stream_action: port
                    .stream_id
                    .as_ref()
                    .map(|stream_id| inspector_stream_action(stream_id)),
                attention_summary: inspector_port_attention_summary(
                    &detail.target,
                    &port.name,
                    latest_diagnostic,
                ),
            })
            .collect(),
        latest_unit_result,
        latest_stream_result,
        related_steps,
        related_diagnostics,
        diagnostic_actions,
    }
}

fn inspector_port_attention_summary(
    target: &rf_ui::InspectorTarget,
    port_name: &str,
    latest_diagnostic: Option<&rf_ui::DiagnosticSummary>,
) -> Option<String> {
    let rf_ui::InspectorTarget::Unit(unit_id) = target else {
        return None;
    };
    let diagnostic = latest_diagnostic?;
    if !matches!(
        diagnostic.highest_severity,
        rf_ui::DiagnosticSeverity::Warning | rf_ui::DiagnosticSeverity::Error
    ) {
        return None;
    }
    if !diagnostic.related_port_targets.iter().any(|port_target| {
        port_target.unit_id.as_str() == unit_id.as_str()
            && port_target.port_name.as_str() == port_name
    }) {
        return None;
    }

    let mut parts = vec![
        diagnostic_severity_label(diagnostic.highest_severity).to_string(),
        format!("port {}:{}", unit_id.as_str(), port_name),
    ];
    if let Some(code) = diagnostic.primary_code.as_ref() {
        parts.push(format!("code {code}"));
    }
    parts.push(format!("count {}", diagnostic.diagnostic_count));

    Some(format!("attention: {}", parts.join("; ")))
}

fn inspector_property_notice_model_from_snapshot(
    notice: &crate::StudioGuiInspectorPropertyNoticeSnapshot,
) -> StudioGuiWindowInspectorPropertyNoticeModel {
    StudioGuiWindowInspectorPropertyNoticeModel {
        status_label: notice.status_label,
        message: notice.message.clone(),
    }
}

fn inspector_composition_summary_model_from_snapshot(
    summary: &crate::StudioGuiInspectorCompositionSummarySnapshot,
) -> StudioGuiWindowInspectorCompositionSummaryModel {
    StudioGuiWindowInspectorCompositionSummaryModel {
        current_sum_text: summary.current_sum_text.clone(),
        normalized_preview_text: summary.normalized_preview_text.clone(),
        status_label: summary.status_label,
    }
}

fn inspector_composition_component_action_model_from_snapshot(
    action: &crate::StudioGuiInspectorCompositionComponentActionSnapshot,
) -> StudioGuiWindowInspectorCompositionComponentActionModel {
    StudioGuiWindowInspectorCompositionComponentActionModel {
        component_id: action.component_id.clone(),
        component_name: action.component_name.clone(),
        action: StudioGuiWindowCommandActionModel {
            label: format!("Add {}", action.component_name),
            hover_text: format!(
                "Add component `{}` to this stream composition with an explicit zero mole fraction",
                action.component_id
            ),
            command_id: action.command_id.clone(),
        },
    }
}

fn inspector_field_model_from_snapshot(
    field: &crate::StudioGuiInspectorTargetFieldSnapshot,
) -> StudioGuiWindowInspectorTargetFieldModel {
    StudioGuiWindowInspectorTargetFieldModel {
        key: field.key.clone(),
        label: field.label.clone(),
        value_kind_label: inspector_field_kind_label(field.value_kind),
        original_value: field.original_value.clone(),
        current_value: field.current_value.clone(),
        status_label: inspector_field_status_label(field.validation, field.is_dirty),
        is_dirty: field.is_dirty,
        draft_update_command_id: field.draft_update_command_id.clone(),
        commit_command_id: field.commit_command_id.clone(),
        discard_command_id: field.discard_command_id.clone(),
        remove_command_id: field.remove_command_id.clone(),
    }
}

fn inspector_field_kind_label(
    value_kind: crate::StudioGuiInspectorTargetFieldValueKindSnapshot,
) -> &'static str {
    match value_kind {
        crate::StudioGuiInspectorTargetFieldValueKindSnapshot::Text => "Text",
        crate::StudioGuiInspectorTargetFieldValueKindSnapshot::Number => "Number",
        crate::StudioGuiInspectorTargetFieldValueKindSnapshot::Choice => "Choice",
    }
}

fn inspector_field_status_label(
    validation: crate::StudioGuiInspectorTargetFieldValidationSnapshot,
    is_dirty: bool,
) -> &'static str {
    match validation {
        crate::StudioGuiInspectorTargetFieldValidationSnapshot::Invalid => "Invalid",
        crate::StudioGuiInspectorTargetFieldValidationSnapshot::Valid if is_dirty => "Draft",
        crate::StudioGuiInspectorTargetFieldValidationSnapshot::Valid => "Valid",
        crate::StudioGuiInspectorTargetFieldValidationSnapshot::Unknown if is_dirty => "Draft",
        crate::StudioGuiInspectorTargetFieldValidationSnapshot::Unknown => "Synced",
    }
}

fn inspector_stream_action(stream_id: &str) -> StudioGuiWindowCommandActionModel {
    let target = rf_ui::InspectorTarget::Stream(rf_types::StreamId::new(stream_id.to_string()));
    StudioGuiWindowCommandActionModel {
        label: stream_id.to_string(),
        hover_text: format!("Stream {stream_id}"),
        command_id: crate::inspector_target_command_id(&target),
    }
}

fn inspector_unit_action(unit_id: &str) -> StudioGuiWindowCommandActionModel {
    let target = rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(unit_id.to_string()));
    StudioGuiWindowCommandActionModel {
        label: unit_id.to_string(),
        hover_text: format!("Unit {unit_id}"),
        command_id: crate::inspector_target_command_id(&target),
    }
}

fn related_steps_for_target(
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    target: &rf_ui::InspectorTarget,
) -> Vec<StudioGuiWindowSolveStepModel> {
    match target {
        rf_ui::InspectorTarget::Unit(unit_id) => snapshot
            .steps
            .iter()
            .filter(|step| step.unit_id == unit_id.as_str())
            .cloned()
            .collect(),
        rf_ui::InspectorTarget::Stream(stream_id) => snapshot
            .steps
            .iter()
            .filter(|step| step.has_related_stream_id(stream_id.as_str()))
            .cloned()
            .collect(),
    }
}

fn related_diagnostics_for_target(
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    target: &rf_ui::InspectorTarget,
) -> Vec<StudioGuiWindowDiagnosticModel> {
    match target {
        rf_ui::InspectorTarget::Unit(unit_id) => snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|candidate| candidate == unit_id.as_str())
            })
            .cloned()
            .collect(),
        rf_ui::InspectorTarget::Stream(stream_id) => snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|candidate| candidate == stream_id.as_str())
            })
            .cloned()
            .collect(),
    }
}

fn latest_unit_result_for_target(
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    target: &rf_ui::InspectorTarget,
) -> Option<StudioGuiWindowUnitExecutionResultModel> {
    let rf_ui::InspectorTarget::Unit(unit_id) = target else {
        return None;
    };
    snapshot
        .steps
        .iter()
        .rev()
        .find(|step| step.unit_id == unit_id.as_str())
        .map(|step| StudioGuiWindowUnitExecutionResultModel {
            unit_id: step.unit_id.clone(),
            step_index: step.index,
            status_label: step.execution_status_label,
            summary: step.summary.clone(),
            consumed_stream_results: step.consumed_stream_results.clone(),
            consumed_stream_actions: step.consumed_stream_actions.clone(),
            produced_stream_results: step.produced_stream_results.clone(),
            produced_stream_actions: step.produced_stream_actions.clone(),
        })
}

fn solve_snapshot_model_from_ui(
    snapshot: &rf_ui::SolveSnapshot,
) -> StudioGuiWindowSolveSnapshotModel {
    StudioGuiWindowSolveSnapshotModel {
        snapshot_id: snapshot.id.as_str().to_string(),
        sequence: snapshot.sequence,
        status_label: run_status_label(snapshot.status),
        summary: snapshot.summary.primary_message.clone(),
        diagnostic_count: snapshot.diagnostics.len(),
        step_count: snapshot.steps.len(),
        stream_count: snapshot.streams.len(),
        streams: snapshot
            .streams
            .iter()
            .map(stream_result_model_from_ui)
            .collect(),
        steps: snapshot
            .steps
            .iter()
            .map(|step| {
                let produced_stream_results = step
                    .streams
                    .iter()
                    .map(stream_result_reference_model_from_ui)
                    .collect::<Vec<_>>();
                let produced_streams = produced_stream_results
                    .iter()
                    .map(|stream| stream.stream_id.clone())
                    .collect::<Vec<_>>();
                let consumed_stream_results = step
                    .consumed_streams
                    .iter()
                    .map(stream_result_reference_model_from_ui)
                    .collect::<Vec<_>>();
                let consumed_streams = consumed_stream_results
                    .iter()
                    .map(|stream| stream.stream_id.clone())
                    .collect::<Vec<_>>();
                let unit_action = inspector_unit_action(step.unit_id.as_str());
                let consumed_stream_actions = consumed_stream_results
                    .iter()
                    .map(|stream| stream.focus_action.clone())
                    .collect::<Vec<_>>();
                let produced_stream_actions = produced_stream_results
                    .iter()
                    .map(|stream| stream.focus_action.clone())
                    .collect::<Vec<_>>();
                let diagnostic_actions = solve_step_diagnostic_actions(
                    step.index,
                    &step.unit_id,
                    &unit_action,
                    &consumed_streams,
                    &consumed_stream_actions,
                    &produced_streams,
                    &produced_stream_actions,
                );
                StudioGuiWindowSolveStepModel {
                    index: step.index,
                    unit_id: step.unit_id.as_str().to_string(),
                    summary: step.summary.clone(),
                    execution_status_label: run_status_label(step.execution.status),
                    consumed_stream_results,
                    consumed_stream_actions,
                    unit_action,
                    produced_stream_results,
                    produced_stream_actions,
                    diagnostic_actions,
                }
            })
            .collect(),
        diagnostics: snapshot
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic_model_from_ui(diagnostic, &snapshot.streams))
            .collect(),
    }
}

fn diagnostic_model_from_ui(
    diagnostic: &rf_ui::DiagnosticSnapshot,
    streams: &[rf_ui::StreamStateSnapshot],
) -> StudioGuiWindowDiagnosticModel {
    let related_unit_ids = diagnostic
        .related_unit_ids
        .iter()
        .map(|unit_id| unit_id.as_str().to_string())
        .collect::<Vec<_>>();
    let related_stream_ids = diagnostic
        .related_stream_ids
        .iter()
        .map(|stream_id| stream_id.as_str().to_string())
        .collect::<Vec<_>>();

    let target_candidates = diagnostic_target_candidates_from_ui(diagnostic);
    let diagnostic_actions = target_candidates
        .iter()
        .map(|target| diagnostic_target_action_from_target("Diagnostic", target))
        .collect();
    let related_stream_results = streams
        .iter()
        .filter(|stream| diagnostic.related_stream_ids.contains(&stream.stream_id))
        .map(stream_result_reference_model_from_ui)
        .collect();

    StudioGuiWindowDiagnosticModel {
        severity_label: diagnostic_severity_label(diagnostic.severity),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        related_stream_results,
        target_candidates,
        diagnostic_actions,
        related_units_text: non_empty_join(related_unit_ids.iter().map(String::as_str).collect()),
        related_streams_text: non_empty_join(
            related_stream_ids.iter().map(String::as_str).collect(),
        ),
        related_unit_ids,
        related_stream_ids,
    }
}

fn diagnostic_target_candidates_from_ui(
    diagnostic: &rf_ui::DiagnosticSnapshot,
) -> Vec<StudioGuiWindowInspectorTargetModel> {
    diagnostic
        .related_unit_ids
        .iter()
        .map(|unit_id| {
            inspector_target_model_from_ui(&rf_ui::InspectorTarget::Unit(unit_id.clone()))
        })
        .chain(diagnostic.related_stream_ids.iter().map(|stream_id| {
            inspector_target_model_from_ui(&rf_ui::InspectorTarget::Stream(stream_id.clone()))
        }))
        .collect()
}

fn diagnostic_target_action_from_target(
    source_label: &'static str,
    target: &StudioGuiWindowInspectorTargetModel,
) -> StudioGuiWindowDiagnosticTargetActionModel {
    StudioGuiWindowDiagnosticTargetActionModel {
        source_label,
        target_label: target.kind_label,
        summary: target.summary.clone(),
        action: target.action.clone(),
    }
}

fn diagnostic_target_action_from_action(
    source_label: &'static str,
    target_label: &'static str,
    summary: String,
    action: &StudioGuiWindowCommandActionModel,
) -> StudioGuiWindowDiagnosticTargetActionModel {
    StudioGuiWindowDiagnosticTargetActionModel {
        source_label,
        target_label,
        summary,
        action: action.clone(),
    }
}

fn dedupe_diagnostic_actions(
    actions: impl IntoIterator<Item = StudioGuiWindowDiagnosticTargetActionModel>,
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let mut seen = BTreeSet::new();
    actions
        .into_iter()
        .filter(|action| {
            seen.insert((
                action.source_label,
                action.target_label,
                action.summary.clone(),
                action.action.command_id.clone(),
            ))
        })
        .collect()
}

fn solve_step_diagnostic_actions(
    step_index: usize,
    unit_id: &rf_types::UnitId,
    unit_action: &StudioGuiWindowCommandActionModel,
    consumed_streams: &[String],
    consumed_stream_actions: &[StudioGuiWindowCommandActionModel],
    produced_streams: &[String],
    produced_stream_actions: &[StudioGuiWindowCommandActionModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let unit_summary = format!("Step #{step_index} unit {}", unit_id.as_str());
    let unit =
        diagnostic_target_action_from_action("Solve step", "Unit", unit_summary, unit_action);
    let consumed = consumed_streams
        .iter()
        .zip(consumed_stream_actions.iter())
        .map(|(stream_id, action)| {
            diagnostic_target_action_from_action(
                "Solve step",
                "Stream",
                format!("Step #{step_index} input stream {stream_id}"),
                action,
            )
        });
    let streams = produced_streams
        .iter()
        .zip(produced_stream_actions.iter())
        .map(|(stream_id, action)| {
            diagnostic_target_action_from_action(
                "Solve step",
                "Stream",
                format!("Step #{step_index} output stream {stream_id}"),
                action,
            )
        });
    dedupe_diagnostic_actions(std::iter::once(unit).chain(consumed).chain(streams))
}

fn inspector_detail_diagnostic_actions(
    target: &StudioGuiWindowInspectorTargetModel,
    latest_unit_result: Option<&StudioGuiWindowUnitExecutionResultModel>,
    related_steps: &[StudioGuiWindowSolveStepModel],
    related_diagnostics: &[StudioGuiWindowDiagnosticModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let active_target = std::iter::once(diagnostic_target_action_from_target(
        "Inspector target",
        target,
    ));
    let latest_result_actions = latest_unit_result.into_iter().flat_map(|unit| {
        unit.produced_stream_results
            .iter()
            .zip(unit.produced_stream_actions.iter())
            .map(|(stream, action)| {
                diagnostic_target_action_from_action(
                    "Latest result",
                    "Stream",
                    format!("Latest result output stream {}", stream.stream_id),
                    action,
                )
            })
    });
    let step_actions = related_steps
        .iter()
        .flat_map(|step| step.diagnostic_actions.iter().cloned());
    let diagnostic_actions = related_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.diagnostic_actions.iter().cloned());

    dedupe_diagnostic_actions(
        active_target
            .chain(latest_result_actions)
            .chain(step_actions)
            .chain(diagnostic_actions),
    )
}

fn stream_result_model_from_ui(
    stream: &rf_ui::StreamStateSnapshot,
) -> StudioGuiWindowStreamResultModel {
    let temperature_text = format_temperature(stream.temperature_k);
    let pressure_text = format_pressure(stream.pressure_pa);
    let molar_flow_text = format_molar_flow(stream.total_molar_flow_mol_s);
    let molar_enthalpy_j_per_mol = overall_molar_enthalpy_j_per_mol(&stream.phases);
    let molar_enthalpy_text = molar_enthalpy_j_per_mol.map(format_molar_enthalpy);
    let mut summary_rows = vec![
        StudioGuiWindowStreamSummaryRowModel {
            label: "T",
            detail_label: "Temperature",
            value: temperature_text.clone(),
        },
        StudioGuiWindowStreamSummaryRowModel {
            label: "P",
            detail_label: "Pressure",
            value: pressure_text.clone(),
        },
        StudioGuiWindowStreamSummaryRowModel {
            label: "F",
            detail_label: "Molar flow",
            value: molar_flow_text.clone(),
        },
    ];
    if let Some(value) = molar_enthalpy_text.clone() {
        summary_rows.push(StudioGuiWindowStreamSummaryRowModel {
            label: "H",
            detail_label: "Molar enthalpy",
            value,
        });
    }
    StudioGuiWindowStreamResultModel {
        stream_id: stream.stream_id.as_str().to_string(),
        label: stream.label.clone(),
        temperature_k: stream.temperature_k,
        pressure_pa: stream.pressure_pa,
        total_molar_flow_mol_s: stream.total_molar_flow_mol_s,
        molar_enthalpy_j_per_mol,
        temperature_text: temperature_text.clone(),
        pressure_text: pressure_text.clone(),
        molar_flow_text: molar_flow_text.clone(),
        molar_enthalpy_text,
        bubble_dew_window: stream
            .bubble_dew_window
            .as_ref()
            .map(bubble_dew_window_model_from_ui),
        summary_rows,
        composition_rows: stream
            .overall_mole_fractions
            .iter()
            .map(
                |(component_id, fraction)| StudioGuiWindowCompositionResultModel {
                    component_id: component_id.clone(),
                    fraction: *fraction,
                    fraction_text: format_fraction(*fraction),
                },
            )
            .collect(),
        phase_rows: stream
            .phases
            .iter()
            .map(|phase| {
                let molar_flow_mol_s = phase.phase_fraction * stream.total_molar_flow_mol_s;
                StudioGuiWindowPhaseResultModel {
                    label: phase.label.clone(),
                    phase_fraction: phase.phase_fraction,
                    phase_fraction_text: format_fraction(phase.phase_fraction),
                    molar_flow_mol_s,
                    molar_flow_text: format_molar_flow(molar_flow_mol_s),
                    composition_text: format_phase_composition(&phase.composition),
                    molar_enthalpy_j_per_mol: phase.molar_enthalpy_j_per_mol,
                    molar_enthalpy_text: phase.molar_enthalpy_j_per_mol.map(format_molar_enthalpy),
                }
            })
            .collect(),
        composition_text: format_composition(&stream.overall_mole_fractions),
        phase_text: format_phases(&stream.phases, stream.total_molar_flow_mol_s),
    }
}

fn stream_result_reference_model_from_ui(
    stream: &rf_ui::StreamStateSnapshot,
) -> StudioGuiWindowStreamResultReferenceModel {
    let temperature_text = format_temperature(stream.temperature_k);
    let pressure_text = format_pressure(stream.pressure_pa);
    let molar_flow_text = format_molar_flow(stream.total_molar_flow_mol_s);
    let molar_enthalpy_text =
        overall_molar_enthalpy_j_per_mol(&stream.phases).map(format_molar_enthalpy);
    StudioGuiWindowStreamResultReferenceModel {
        stream_id: stream.stream_id.as_str().to_string(),
        summary: stream_result_numeric_summary(
            &temperature_text,
            &pressure_text,
            &molar_flow_text,
            molar_enthalpy_text.as_deref(),
            None,
        ),
        focus_action: inspector_stream_action(stream.stream_id.as_str()),
    }
}

fn stream_result_reference_model_from_diagnostic_snapshot(
    stream: &StudioGuiDiagnosticStreamSnapshot,
) -> StudioGuiWindowStreamResultReferenceModel {
    let temperature_text = format!("{:.2} K", stream.temperature_k);
    let pressure_text = format!("{:.0} Pa", stream.pressure_pa);
    let molar_flow_text = format_molar_flow(stream.total_molar_flow_mol_s);
    let composition_text = format_composition(&stream.overall_mole_fractions);
    StudioGuiWindowStreamResultReferenceModel {
        stream_id: stream.stream_id.clone(),
        summary: stream_result_numeric_summary(
            &temperature_text,
            &pressure_text,
            &molar_flow_text,
            None,
            Some(&composition_text),
        ),
        focus_action: inspector_stream_action(&stream.stream_id),
    }
}

fn overall_molar_enthalpy_j_per_mol(phases: &[rf_ui::PhaseStateSnapshot]) -> Option<f64> {
    phases
        .iter()
        .find(|phase| phase.label == "overall")
        .and_then(|phase| phase.molar_enthalpy_j_per_mol)
}

fn bubble_dew_window_model_from_ui(
    window: &rf_ui::BubbleDewWindowSnapshot,
) -> StudioGuiWindowBubbleDewWindowModel {
    StudioGuiWindowBubbleDewWindowModel {
        phase_region: window.phase_region.as_str().to_string(),
        bubble_pressure_pa: window.bubble_pressure_pa,
        dew_pressure_pa: window.dew_pressure_pa,
        bubble_temperature_k: window.bubble_temperature_k,
        dew_temperature_k: window.dew_temperature_k,
        bubble_pressure_text: format_pressure(window.bubble_pressure_pa),
        dew_pressure_text: format_pressure(window.dew_pressure_pa),
        bubble_temperature_text: format_temperature(window.bubble_temperature_k),
        dew_temperature_text: format_temperature(window.dew_temperature_k),
    }
}

fn format_temperature(value: f64) -> String {
    format!("{value:.2} K")
}

fn format_pressure(value: f64) -> String {
    format!("{value:.0} Pa")
}

fn format_molar_enthalpy(value: f64) -> String {
    format!("{value:.3} J/mol")
}

fn format_molar_flow(value: f64) -> String {
    format!("{value:.6} mol/s")
}

fn stream_result_numeric_summary(
    temperature_text: &str,
    pressure_text: &str,
    molar_flow_text: &str,
    molar_enthalpy_text: Option<&str>,
    composition_text: Option<&str>,
) -> String {
    let mut parts = vec![
        format!("T {temperature_text}"),
        format!("P {pressure_text}"),
        format!("F {molar_flow_text}"),
    ];
    if let Some(molar_enthalpy_text) = molar_enthalpy_text {
        parts.push(format!("H {molar_enthalpy_text}"));
    }
    if let Some(composition_text) = composition_text {
        parts.push(composition_text.to_string());
    }
    parts.join(" | ")
}

fn format_composition(composition: &[(String, f64)]) -> String {
    if composition.is_empty() {
        return "z: none".to_string();
    }

    format!(
        "z: {}",
        composition
            .iter()
            .map(|(component_id, fraction)| format!("{component_id}={fraction:.4}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn format_phases(phases: &[rf_ui::PhaseStateSnapshot], total_molar_flow_mol_s: f64) -> String {
    if phases.is_empty() {
        return "phases: none".to_string();
    }

    format!(
        "phases: {}",
        phases
            .iter()
            .map(|phase| {
                let molar_flow_mol_s = phase.phase_fraction * total_molar_flow_mol_s;
                format!(
                    "{}={} ({})",
                    phase.label,
                    format_fraction(phase.phase_fraction),
                    format_molar_flow(molar_flow_mol_s)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn format_phase_composition(composition: &[(String, f64)]) -> String {
    if composition.is_empty() {
        return "z: none".to_string();
    }

    composition
        .iter()
        .map(|(component_id, fraction)| format!("{component_id}={}", format_fraction(*fraction)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_fraction(value: f64) -> String {
    format!("{value:.4}")
}

fn non_empty_join(values: Vec<&str>) -> Option<String> {
    (!values.is_empty()).then(|| values.join(", "))
}

fn run_status_label(status: rf_ui::RunStatus) -> &'static str {
    match status {
        rf_ui::RunStatus::Idle => "Idle",
        rf_ui::RunStatus::Dirty => "Dirty",
        rf_ui::RunStatus::Checking => "Checking",
        rf_ui::RunStatus::Runnable => "Runnable",
        rf_ui::RunStatus::Solving => "Solving",
        rf_ui::RunStatus::Converged => "Converged",
        rf_ui::RunStatus::UnderSpecified => "Under-specified",
        rf_ui::RunStatus::OverSpecified => "Over-specified",
        rf_ui::RunStatus::Unconverged => "Unconverged",
        rf_ui::RunStatus::Error => "Error",
    }
}

fn diagnostic_severity_label(severity: rf_ui::DiagnosticSeverity) -> &'static str {
    match severity {
        rf_ui::DiagnosticSeverity::Info => "Info",
        rf_ui::DiagnosticSeverity::Warning => "Warning",
        rf_ui::DiagnosticSeverity::Error => "Error",
    }
}

#[doc(hidden)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
