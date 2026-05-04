use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EntitlementSessionHostRuntimeOutput, StudioExampleProjectModel, StudioGuiCanvasWidgetModel,
    StudioGuiCommandEntry, StudioGuiCommandMenuNode, StudioGuiCommandRegistry,
    StudioGuiCommandSection, StudioGuiSnapshot, StudioGuiWindowAreaId, StudioGuiWindowDockRegion,
    StudioGuiWindowDropTarget, StudioGuiWindowDropTargetKind, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutState, StudioGuiWorkspaceDocumentSnapshot,
    StudioWindowHostId, WorkspaceControlState,
};

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
    pub related_ports: Vec<StudioGuiWindowFailureDiagnosticPortTargetModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowFailureDiagnosticPortTargetModel {
    pub unit_id: String,
    pub port_name: String,
    pub summary: String,
    pub unit_action: StudioGuiWindowCommandActionModel,
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
    pub property_batch_commit_command_id: Option<String>,
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
    pub produced_stream_ids: Vec<String>,
    pub produced_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowStreamResultModel {
    pub stream_id: String,
    pub label: String,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub total_molar_flow_mol_s: f64,
    pub temperature_text: String,
    pub pressure_text: String,
    pub molar_flow_text: String,
    pub summary_rows: Vec<StudioGuiWindowStreamSummaryRowModel>,
    pub composition_rows: Vec<StudioGuiWindowCompositionResultModel>,
    pub phase_rows: Vec<StudioGuiWindowPhaseResultModel>,
    pub composition_text: String,
    pub phase_text: String,
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
    pub phase_fraction_text: String,
    pub composition_text: String,
    pub molar_enthalpy_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowSolveStepModel {
    pub index: usize,
    pub unit_id: String,
    pub summary: String,
    pub execution_status_label: &'static str,
    pub produced_streams: Vec<String>,
    pub unit_action: StudioGuiWindowCommandActionModel,
    pub produced_stream_actions: Vec<StudioGuiWindowCommandActionModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDiagnosticModel {
    pub severity_label: &'static str,
    pub code: String,
    pub message: String,
    pub related_unit_ids: Vec<String>,
    pub related_stream_ids: Vec<String>,
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
    pub summary_rows: Vec<StudioGuiWindowResultInspectorComparisonRowModel>,
    pub composition_rows: Vec<StudioGuiWindowResultInspectorCompositionComparisonRowModel>,
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
        let layout_state =
            StudioGuiWindowLayoutState::from_snapshot_for_window(snapshot, window_id);
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

impl StudioGuiWindowSolveSnapshotModel {
    pub fn result_inspector(
        &self,
        requested_stream_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        self.result_inspector_with_comparison(requested_stream_id, None)
    }

    pub fn result_inspector_with_comparison(
        &self,
        requested_stream_id: Option<&str>,
        requested_comparison_stream_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        self.result_inspector_with_unit(requested_stream_id, requested_comparison_stream_id, None)
    }

    pub fn result_inspector_with_unit(
        &self,
        requested_stream_id: Option<&str>,
        requested_comparison_stream_id: Option<&str>,
        requested_unit_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        let selected_stream_id = requested_stream_id
            .filter(|stream_id| {
                self.streams
                    .iter()
                    .any(|stream| stream.stream_id == *stream_id)
            })
            .map(str::to_string)
            .or_else(|| self.streams.first().map(|stream| stream.stream_id.clone()));
        let has_stale_selection = requested_stream_id.is_some()
            && requested_stream_id.map(str::to_string) != selected_stream_id;
        let selected_stream = selected_stream_id
            .as_deref()
            .and_then(|selected_id| {
                self.streams
                    .iter()
                    .find(|stream| stream.stream_id == selected_id)
            })
            .cloned();
        let comparison_stream_id = requested_comparison_stream_id
            .filter(|stream_id| {
                selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id != *stream_id)
                    .unwrap_or(true)
                    && self
                        .streams
                        .iter()
                        .any(|stream| stream.stream_id == *stream_id)
            })
            .map(str::to_string);
        let has_stale_comparison = requested_comparison_stream_id.is_some()
            && requested_comparison_stream_id.map(str::to_string) != comparison_stream_id;
        let comparison_stream = comparison_stream_id
            .as_deref()
            .and_then(|comparison_id| {
                self.streams
                    .iter()
                    .find(|stream| stream.stream_id == comparison_id)
            })
            .cloned();
        let comparison = selected_stream
            .as_ref()
            .zip(comparison_stream.as_ref())
            .map(|(base, compared)| result_inspector_comparison_model(base, compared));
        let related_steps: Vec<StudioGuiWindowSolveStepModel> = selected_stream_id
            .as_deref()
            .map(|selected_id| {
                self.steps
                    .iter()
                    .filter(|step| {
                        step.produced_streams
                            .iter()
                            .any(|stream_id| stream_id == selected_id)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let related_step_unit_ids = related_steps
            .iter()
            .map(|step| step.unit_id.as_str())
            .collect::<BTreeSet<_>>();
        let related_diagnostics: Vec<StudioGuiWindowDiagnosticModel> = selected_stream_id
            .as_deref()
            .map(|selected_id| {
                self.diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic
                            .related_stream_ids
                            .iter()
                            .any(|stream_id| stream_id == selected_id)
                            || diagnostic
                                .related_unit_ids
                                .iter()
                                .any(|unit_id| related_step_unit_ids.contains(unit_id.as_str()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let diagnostic_actions = result_inspector_diagnostic_actions(
            selected_stream_id.as_deref(),
            &related_steps,
            &related_diagnostics,
        );
        let stream_options = self
            .streams
            .iter()
            .map(|stream| StudioGuiWindowResultInspectorStreamOptionModel {
                stream_id: stream.stream_id.clone(),
                label: stream.label.clone(),
                summary: format!(
                    "{} | T {} | P {} | F {}",
                    stream.stream_id,
                    stream.temperature_text,
                    stream.pressure_text,
                    stream.molar_flow_text
                ),
                is_selected: selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id == stream.stream_id)
                    .unwrap_or(false),
            })
            .collect();
        let comparison_options = self
            .streams
            .iter()
            .filter(|stream| {
                selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id != stream.stream_id)
                    .unwrap_or(true)
            })
            .map(|stream| StudioGuiWindowResultInspectorStreamOptionModel {
                stream_id: stream.stream_id.clone(),
                label: stream.label.clone(),
                summary: format!(
                    "{} | T {} | P {} | F {}",
                    stream.stream_id,
                    stream.temperature_text,
                    stream.pressure_text,
                    stream.molar_flow_text
                ),
                is_selected: comparison_stream_id
                    .as_deref()
                    .map(|comparison_id| comparison_id == stream.stream_id)
                    .unwrap_or(false),
            })
            .collect();

        let unit_id_order: Vec<String> = self.steps.iter().map(|step| step.unit_id.clone()).fold(
            Vec::new(),
            |mut acc, unit_id| {
                if !acc.iter().any(|existing| existing == &unit_id) {
                    acc.push(unit_id);
                }
                acc
            },
        );
        let selected_unit_id = requested_unit_id
            .filter(|unit_id| unit_id_order.iter().any(|existing| existing == *unit_id))
            .map(str::to_string)
            .or_else(|| unit_id_order.first().cloned());
        let has_stale_unit_selection = requested_unit_id.is_some()
            && requested_unit_id.map(str::to_string) != selected_unit_id;
        let unit_related_steps: Vec<StudioGuiWindowSolveStepModel> = selected_unit_id
            .as_deref()
            .map(|selected_unit| {
                self.steps
                    .iter()
                    .filter(|step| step.unit_id == selected_unit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let unit_related_diagnostics: Vec<StudioGuiWindowDiagnosticModel> = selected_unit_id
            .as_deref()
            .map(|selected_unit| {
                let produced_streams: BTreeSet<&str> = unit_related_steps
                    .iter()
                    .flat_map(|step| step.produced_streams.iter().map(String::as_str))
                    .collect();
                self.diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic
                            .related_unit_ids
                            .iter()
                            .any(|unit_id| unit_id == selected_unit)
                            || diagnostic
                                .related_stream_ids
                                .iter()
                                .any(|stream_id| produced_streams.contains(stream_id.as_str()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let selected_unit = selected_unit_id.as_deref().and_then(|selected_unit| {
            unit_related_steps
                .last()
                .map(|step| StudioGuiWindowUnitExecutionResultModel {
                    unit_id: selected_unit.to_string(),
                    step_index: step.index,
                    status_label: step.execution_status_label,
                    summary: step.summary.clone(),
                    produced_stream_ids: step.produced_streams.clone(),
                    produced_stream_actions: step.produced_stream_actions.clone(),
                })
        });
        let unit_options: Vec<StudioGuiWindowResultInspectorUnitOptionModel> = unit_id_order
            .iter()
            .map(|unit_id| {
                let last_step = self
                    .steps
                    .iter()
                    .rev()
                    .find(|step| &step.unit_id == unit_id);
                let status_label = last_step
                    .map(|step| step.execution_status_label)
                    .unwrap_or("Idle");
                let step_index = last_step.map(|step| step.index).unwrap_or(0);
                let produced_streams_text = last_step
                    .map(|step| step.produced_streams.join(", "))
                    .unwrap_or_default();
                let summary = if produced_streams_text.is_empty() {
                    format!("{unit_id} | {status_label} | step #{step_index}")
                } else {
                    format!(
                        "{unit_id} | {status_label} | step #{step_index} | -> {produced_streams_text}"
                    )
                };
                StudioGuiWindowResultInspectorUnitOptionModel {
                    unit_id: unit_id.clone(),
                    status_label,
                    step_index,
                    summary,
                    is_selected: selected_unit_id
                        .as_deref()
                        .map(|selected_unit| selected_unit == unit_id)
                        .unwrap_or(false),
                    focus_action: inspector_unit_action(unit_id),
                }
            })
            .collect();
        let unit_diagnostic_actions = result_inspector_unit_diagnostic_actions(
            selected_unit_id.as_deref(),
            &unit_related_steps,
            &unit_related_diagnostics,
        );

        StudioGuiWindowResultInspectorModel {
            snapshot_id: self.snapshot_id.clone(),
            selected_stream_id,
            selected_stream,
            comparison_stream_id,
            comparison_stream,
            stream_options,
            comparison_options,
            comparison,
            related_steps,
            related_diagnostics,
            diagnostic_actions,
            has_stale_selection,
            has_stale_comparison,
            unit_options,
            selected_unit_id,
            selected_unit,
            unit_related_steps,
            unit_related_diagnostics,
            unit_diagnostic_actions,
            has_stale_unit_selection,
        }
    }
}

fn build_drop_preview_overlay(
    preview_layout: &StudioGuiWindowLayoutModel,
    drop_target: &StudioGuiWindowDropTarget,
) -> StudioGuiWindowDropPreviewOverlayModel {
    let target_stack =
        preview_layout.stack_group(drop_target.dock_region, drop_target.target_stack_group);
    let target_stack_area_ids = target_stack
        .map(|group| group.tabs.iter().map(|tab| tab.area_id).collect::<Vec<_>>())
        .unwrap_or_else(|| drop_target.preview_area_ids.clone());
    let target_stack_active_area_id = target_stack
        .map(|group| group.active_area_id)
        .unwrap_or(drop_target.preview_active_area_id);

    StudioGuiWindowDropPreviewOverlayModel {
        drag_area_id: drop_target.area_id,
        kind: drop_target.kind,
        target_dock_region: drop_target.dock_region,
        target_stack_group: drop_target.target_stack_group,
        target_group_index: drop_target.target_group_index,
        target_tab_index: drop_target.target_tab_index,
        target_stack_area_ids: target_stack_area_ids.clone(),
        target_stack_active_area_id,
        highlighted_area_ids: target_stack_area_ids,
        anchor_area_id: drop_target.anchor_area_id,
        creates_new_stack: drop_target.creates_new_stack,
        merges_into_existing_stack: drop_target.merges_into_existing_stack,
    }
}

fn changed_area_ids_for_preview(
    current_layout_state: &StudioGuiWindowLayoutState,
    preview_layout_state: &StudioGuiWindowLayoutState,
) -> Vec<StudioGuiWindowAreaId> {
    let mut changed = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
        StudioGuiWindowAreaId::Runtime,
    ]
    .into_iter()
    .filter(|area_id| {
        current_layout_state.panel(*area_id) != preview_layout_state.panel(*area_id)
            || area_is_active_in_stack(current_layout_state, *area_id)
                != area_is_active_in_stack(preview_layout_state, *area_id)
    })
    .collect::<Vec<_>>();
    changed.sort_by_key(|area_id| match area_id {
        StudioGuiWindowAreaId::Commands => 0,
        StudioGuiWindowAreaId::Canvas => 1,
        StudioGuiWindowAreaId::Runtime => 2,
    });
    changed
}

fn area_is_active_in_stack(
    layout_state: &StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    layout_state
        .panel(area_id)
        .map(|panel| {
            layout_state.active_panel_in_stack(panel.dock_region, panel.stack_group)
                == Some(area_id)
        })
        .unwrap_or(false)
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
        .then(|| failure_result_model_from_control_state(&snapshot.runtime.control_state))
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
        .map(failure_diagnostic_detail_model_from_summary);
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
        related_ports: summary
            .related_port_targets
            .iter()
            .map(|target| {
                let unit_id = target.unit_id.as_str().to_string();
                let port_name = target.port_name.clone();
                StudioGuiWindowFailureDiagnosticPortTargetModel {
                    unit_id: unit_id.clone(),
                    port_name: port_name.clone(),
                    summary: format!("Unit {unit_id} port {port_name}"),
                    unit_action: inspector_unit_action(&unit_id),
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
        property_batch_commit_command_id: detail.property_batch_commit_command_id.clone(),
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
            .filter(|step| {
                step.produced_streams
                    .iter()
                    .any(|candidate| candidate == stream_id.as_str())
            })
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
            produced_stream_ids: step.produced_streams.clone(),
            produced_stream_actions: step
                .produced_streams
                .iter()
                .map(|stream_id| inspector_stream_action(stream_id))
                .collect(),
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
                let produced_streams = step
                    .streams
                    .iter()
                    .map(|stream| stream.stream_id.as_str().to_string())
                    .collect::<Vec<_>>();
                let unit_action = inspector_unit_action(step.unit_id.as_str());
                let produced_stream_actions = produced_streams
                    .iter()
                    .map(|stream_id| inspector_stream_action(stream_id))
                    .collect::<Vec<_>>();
                let diagnostic_actions = solve_step_diagnostic_actions(
                    step.index,
                    &step.unit_id,
                    &unit_action,
                    &produced_streams,
                    &produced_stream_actions,
                );
                StudioGuiWindowSolveStepModel {
                    index: step.index,
                    unit_id: step.unit_id.as_str().to_string(),
                    summary: step.summary.clone(),
                    execution_status_label: run_status_label(step.execution.status),
                    produced_streams,
                    unit_action,
                    produced_stream_actions,
                    diagnostic_actions,
                }
            })
            .collect(),
        diagnostics: snapshot
            .diagnostics
            .iter()
            .map(diagnostic_model_from_ui)
            .collect(),
    }
}

fn diagnostic_model_from_ui(
    diagnostic: &rf_ui::DiagnosticSnapshot,
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

    StudioGuiWindowDiagnosticModel {
        severity_label: diagnostic_severity_label(diagnostic.severity),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
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
    produced_streams: &[String],
    produced_stream_actions: &[StudioGuiWindowCommandActionModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let unit_summary = format!("Step #{step_index} unit {}", unit_id.as_str());
    let unit =
        diagnostic_target_action_from_action("Solve step", "Unit", unit_summary, unit_action);
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
    dedupe_diagnostic_actions(std::iter::once(unit).chain(streams))
}

fn result_inspector_diagnostic_actions(
    selected_stream_id: Option<&str>,
    related_steps: &[StudioGuiWindowSolveStepModel],
    related_diagnostics: &[StudioGuiWindowDiagnosticModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let selected_stream = selected_stream_id.map(|stream_id| {
        diagnostic_target_action_from_action(
            "Selected stream",
            "Stream",
            format!("Selected result stream {stream_id}"),
            &inspector_stream_action(stream_id),
        )
    });
    let step_actions = related_steps
        .iter()
        .flat_map(|step| step.diagnostic_actions.iter().cloned());
    let diagnostic_actions = related_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.diagnostic_actions.iter().cloned());
    dedupe_diagnostic_actions(
        selected_stream
            .into_iter()
            .chain(step_actions)
            .chain(diagnostic_actions),
    )
}

fn result_inspector_unit_diagnostic_actions(
    selected_unit_id: Option<&str>,
    related_steps: &[StudioGuiWindowSolveStepModel],
    related_diagnostics: &[StudioGuiWindowDiagnosticModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let selected_unit = selected_unit_id.map(|unit_id| {
        diagnostic_target_action_from_action(
            "Selected unit",
            "Unit",
            format!("Selected result unit {unit_id}"),
            &inspector_unit_action(unit_id),
        )
    });
    let step_actions = related_steps
        .iter()
        .flat_map(|step| step.diagnostic_actions.iter().cloned());
    let diagnostic_actions = related_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.diagnostic_actions.iter().cloned());
    dedupe_diagnostic_actions(
        selected_unit
            .into_iter()
            .chain(step_actions)
            .chain(diagnostic_actions),
    )
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
        unit.produced_stream_ids
            .iter()
            .zip(unit.produced_stream_actions.iter())
            .map(|(stream_id, action)| {
                diagnostic_target_action_from_action(
                    "Latest result",
                    "Stream",
                    format!("Latest result output stream {stream_id}"),
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
    let temperature_text = format!("{:.2} K", stream.temperature_k);
    let pressure_text = format!("{:.0} Pa", stream.pressure_pa);
    let molar_flow_text = format!("{:.6} mol/s", stream.total_molar_flow_mol_s);
    StudioGuiWindowStreamResultModel {
        stream_id: stream.stream_id.as_str().to_string(),
        label: stream.label.clone(),
        temperature_k: stream.temperature_k,
        pressure_pa: stream.pressure_pa,
        total_molar_flow_mol_s: stream.total_molar_flow_mol_s,
        temperature_text: temperature_text.clone(),
        pressure_text: pressure_text.clone(),
        molar_flow_text: molar_flow_text.clone(),
        summary_rows: vec![
            StudioGuiWindowStreamSummaryRowModel {
                label: "T",
                detail_label: "Temperature",
                value: temperature_text,
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "P",
                detail_label: "Pressure",
                value: pressure_text,
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "F",
                detail_label: "Molar flow",
                value: molar_flow_text,
            },
        ],
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
            .map(|phase| StudioGuiWindowPhaseResultModel {
                label: phase.label.clone(),
                phase_fraction_text: format_fraction(phase.phase_fraction),
                composition_text: format_phase_composition(&phase.composition),
                molar_enthalpy_text: phase
                    .molar_enthalpy_j_per_mol
                    .map(|value| format!("{value:.3} J/mol")),
            })
            .collect(),
        composition_text: format_composition(&stream.overall_mole_fractions),
        phase_text: format_phases(&stream.phases),
    }
}

fn result_inspector_comparison_model(
    base: &StudioGuiWindowStreamResultModel,
    compared: &StudioGuiWindowStreamResultModel,
) -> StudioGuiWindowResultInspectorComparisonModel {
    let base_composition = base
        .composition_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.fraction))
        .collect::<BTreeMap<_, _>>();
    let compared_composition = compared
        .composition_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.fraction))
        .collect::<BTreeMap<_, _>>();
    let component_ids = base_composition
        .keys()
        .chain(compared_composition.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    StudioGuiWindowResultInspectorComparisonModel {
        base_stream_id: base.stream_id.clone(),
        compared_stream_id: compared.stream_id.clone(),
        summary_rows: vec![
            StudioGuiWindowResultInspectorComparisonRowModel {
                label: "T",
                detail_label: "Temperature",
                base_value: base.temperature_text.clone(),
                compared_value: compared.temperature_text.clone(),
                delta_text: format_signed_delta(
                    compared.temperature_k - base.temperature_k,
                    "K",
                    2,
                ),
            },
            StudioGuiWindowResultInspectorComparisonRowModel {
                label: "P",
                detail_label: "Pressure",
                base_value: base.pressure_text.clone(),
                compared_value: compared.pressure_text.clone(),
                delta_text: format_signed_delta(compared.pressure_pa - base.pressure_pa, "Pa", 0),
            },
            StudioGuiWindowResultInspectorComparisonRowModel {
                label: "F",
                detail_label: "Molar flow",
                base_value: base.molar_flow_text.clone(),
                compared_value: compared.molar_flow_text.clone(),
                delta_text: format_signed_delta(
                    compared.total_molar_flow_mol_s - base.total_molar_flow_mol_s,
                    "mol/s",
                    6,
                ),
            },
        ],
        composition_rows: component_ids
            .into_iter()
            .map(|component_id| {
                let base_fraction = base_composition.get(component_id).copied().unwrap_or(0.0);
                let compared_fraction = compared_composition
                    .get(component_id)
                    .copied()
                    .unwrap_or(0.0);
                StudioGuiWindowResultInspectorCompositionComparisonRowModel {
                    component_id: component_id.to_string(),
                    base_fraction_text: format_fraction(base_fraction),
                    compared_fraction_text: format_fraction(compared_fraction),
                    delta_text: format_signed_delta(compared_fraction - base_fraction, "", 4),
                }
            })
            .collect(),
    }
}

fn format_signed_delta(value: f64, unit: &str, decimals: usize) -> String {
    let sign = if value >= 0.0 { "+" } else { "" };
    let number = format!("{value:.decimals$}");
    if unit.is_empty() {
        format!("{sign}{number}")
    } else {
        format!("{sign}{number} {unit}")
    }
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

fn format_phases(phases: &[rf_ui::PhaseStateSnapshot]) -> String {
    if phases.is_empty() {
        return "phases: none".to_string();
    }

    format!(
        "phases: {}",
        phases
            .iter()
            .map(|phase| format!("{}={:.4}", phase.label, phase.phase_fraction))
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiHostCommandOutcome,
        StudioGuiWindowAreaId, StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion,
        StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutScopeKind, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-window-model-{timestamp}.rfproj.json"
        ));
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project).expect("expected local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("examples")
                .join("flowsheets")
                .join("failures")
                .join("unbound-outlet-port.rfproj.json"),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
        }
    }

    #[test]
    fn studio_gui_window_model_groups_snapshot_into_window_regions() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window = dispatch.snapshot.window_model();

        assert_eq!(window.header.title, "RadishFlow Studio");
        assert_eq!(window.header.registered_window_count, 1);
        assert_eq!(window.header.foreground_window_id, Some(1));
        assert_eq!(window.header.entitlement_timer_owner_window_id, Some(1));
        assert!(window.header.status_line.contains("registered windows: 1"));
        assert!(window.header.status_line.contains("foreground: #1"));
        assert!(window.header.status_line.contains("timer owner: #1"));

        assert_eq!(
            window.commands.total_command_count,
            dispatch
                .snapshot
                .command_registry
                .sections
                .iter()
                .map(|section| section.commands.len())
                .sum::<usize>()
        );
        assert!(
            window.commands.enabled_command_count >= 1,
            "expected at least one enabled command"
        );
        assert_eq!(
            window
                .commands
                .command_list_sections
                .first()
                .map(|section| section.title),
            Some("File")
        );
        assert!(
            window
                .commands
                .command_list_sections
                .iter()
                .any(|section| section.title == "Canvas"),
            "expected canvas command section when suggestions exist"
        );

        assert_eq!(window.canvas.title, "Canvas");
        assert_eq!(window.canvas.suggestion_count, 3);
        assert_eq!(window.canvas.enabled_action_count, 10);
        assert_eq!(
            window.canvas.focused_suggestion_id.as_deref(),
            Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
        );
        assert_eq!(
            window.canvas.widget.primary_action().label,
            "Accept suggestion"
        );

        assert_eq!(window.runtime.title, "Runtime");
        assert_eq!(
            window.runtime.control_state.run_status,
            rf_ui::RunStatus::Idle
        );
        assert_eq!(
            window.runtime.run_panel.view().primary_action.label,
            "Resume"
        );
        assert_eq!(window.runtime.example_projects.len(), 7);
        assert_eq!(
            window
                .runtime
                .example_projects
                .iter()
                .find(|example| example.is_current)
                .map(|example| example.id),
            None,
            "temporary edited project should not be marked as a bundled example"
        );
        assert!(window.runtime.entitlement_host.is_some());
        assert!(window.runtime.platform_timer_lines.is_empty());
        assert!(window.runtime.gui_activity_lines.is_empty());
        assert_eq!(
            window.runtime.latest_log_entry,
            window.runtime.log_entries.last().cloned()
        );
        assert_eq!(
            window.layout_state.scope.kind,
            StudioGuiWindowLayoutScopeKind::Window
        );
        assert_eq!(window.layout_state.scope.layout_slot, Some(1));
        assert_eq!(
            window.layout_state.scope.layout_key,
            "studio.window.owner.slot-1"
        );
        assert_eq!(window.drop_preview, None);

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_model_surfaces_workspace_results_and_diagnostics() {
        let config = StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..StudioRuntimeConfig::default()
        };
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.run_manual".to_string(),
            })
            .expect("expected run dispatch");
        let window = dispatch.window;

        assert_eq!(window.layout_state.scope.window_id, Some(window_id));
        assert_eq!(window.runtime.workspace_document.revision, 0);
        assert_eq!(window.runtime.workspace_document.unit_count, 3);
        assert_eq!(window.runtime.workspace_document.snapshot_history_count, 1);

        let snapshot = window
            .runtime
            .latest_solve_snapshot
            .expect("expected latest solve snapshot");
        assert_eq!(snapshot.status_label, "Converged");
        assert_eq!(snapshot.stream_count, 4);
        assert_eq!(snapshot.step_count, 3);
        assert_eq!(snapshot.diagnostic_count, 4);
        let heated_stream = snapshot
            .streams
            .iter()
            .find(|stream| stream.stream_id == "stream-heated")
            .expect("expected heated stream");
        assert_eq!(heated_stream.temperature_text, "345.00 K");
        assert!(heated_stream.composition_text.contains("component-a="));
        assert_eq!(heated_stream.summary_rows.len(), 3);
        assert!(heated_stream.summary_rows.iter().any(|row| row.label == "T"
            && row.detail_label == "Temperature"
            && row.value == "345.00 K"));
        assert!(
            heated_stream
                .composition_rows
                .iter()
                .any(|row| row.component_id == "component-a" && !row.fraction_text.is_empty())
        );
        assert!(
            snapshot
                .streams
                .iter()
                .flat_map(|stream| stream.phase_rows.iter())
                .any(|row| row.phase_fraction_text == "1.0000"
                    && row.composition_text.contains("component-a="))
        );
        assert!(
            snapshot
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "solver.unit_executed")
        );
        assert!(
            snapshot.diagnostics.iter().any(|diagnostic| {
                diagnostic.target_candidates.iter().any(|target| {
                    target.kind_label == "Unit"
                        && target.target_id == "heater-1"
                        && target.command_id == "inspector.focus_unit:heater-1"
                        && target.action.command_id == "inspector.focus_unit:heater-1"
                        && target.action.label == "Unit heater-1"
                })
            }),
            "expected diagnostics to expose unit inspector target candidates"
        );
        let inspector = snapshot.result_inspector(Some("stream-heated"));
        assert_eq!(
            inspector.selected_stream_id.as_deref(),
            Some("stream-heated")
        );
        assert_eq!(
            inspector
                .selected_stream
                .as_ref()
                .map(|stream| stream.temperature_text.as_str()),
            Some("345.00 K")
        );
        assert!(
            inspector
                .stream_options
                .iter()
                .any(|option| option.stream_id == "stream-heated"
                    && option.is_selected
                    && option.summary.contains("P 95000 Pa"))
        );
        assert!(
            inspector
                .related_steps
                .iter()
                .any(|step| step.unit_id == "heater-1")
        );
        let related_heater_step = inspector
            .related_steps
            .iter()
            .find(|step| step.unit_id == "heater-1")
            .expect("expected related heater step");
        assert_eq!(
            related_heater_step.unit_action.command_id,
            "inspector.focus_unit:heater-1"
        );
        assert!(related_heater_step.diagnostic_actions.iter().any(|action| {
            action.source_label == "Solve step"
                && action.target_label == "Unit"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        }));
        assert!(
            related_heater_step
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
        );
        assert!(inspector.diagnostic_actions.iter().any(|action| {
            action.source_label == "Selected stream"
                && action.action.command_id == "inspector.focus_stream:stream-heated"
        }));
        assert!(inspector.diagnostic_actions.iter().any(|action| {
            action.source_label == "Solve step"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        }));
        assert!(
            inspector.related_diagnostics.iter().any(|diagnostic| {
                diagnostic.target_candidates.iter().any(|target| {
                    target.kind_label == "Stream"
                        && target.target_id == "stream-heated"
                        && target.command_id == "inspector.focus_stream:stream-heated"
                })
            }),
            "expected related diagnostics to expose stream target candidates"
        );
        assert!(
            inspector.related_diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "solver.unit_executed"
                    && diagnostic
                        .related_unit_ids
                        .iter()
                        .any(|unit_id| unit_id == "heater-1")
            }),
            "expected result inspector to include diagnostics from the unit that produced the selected stream"
        );
        let stream_target_command_id = inspector
            .related_diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.target_candidates.iter())
            .find(|target| target.target_id == "stream-heated")
            .map(|target| target.command_id.clone())
            .expect("expected stream target command id");

        let target_dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: stream_target_command_id,
            })
            .expect("expected inspector target dispatch");
        assert_eq!(
            target_dispatch
                .window
                .runtime
                .active_inspector_target
                .as_ref()
                .map(|target| (target.kind_label, target.target_id.as_str())),
            Some(("Stream", "stream-heated"))
        );
        let active_detail = target_dispatch
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active stream inspector detail");
        assert!(
            active_detail.property_fields.iter().any(|field| {
                field.key == "stream:stream-heated:temperature_k"
                    && field.value_kind_label == "Number"
                    && field.status_label == "Synced"
                    && !field.is_dirty
                    && field.draft_update_command_id
                        == "inspector.update_stream_draft:stream:stream-heated:temperature_k"
                    && field.commit_command_id.is_none()
            }),
            "expected stream inspector detail to expose field-level property presentation"
        );
        assert!(
            active_detail.property_fields.iter().any(|field| {
                field.key == "stream:stream-heated:overall_mole_fraction:component-a"
                    && field.label == "Overall mole fraction (component-a)"
                    && field.value_kind_label == "Number"
                    && field.draft_update_command_id
                        == "inspector.update_stream_draft:stream:stream-heated:overall_mole_fraction:component-a"
            }),
            "expected stream inspector detail to expose composition field presentation"
        );
        assert_eq!(
            active_detail
                .latest_stream_result
                .as_ref()
                .map(|stream| stream.stream_id.as_str()),
            Some("stream-heated")
        );
        assert!(
            active_detail
                .related_steps
                .iter()
                .any(|step| step.unit_id == "heater-1")
        );
        assert!(active_detail.related_steps.iter().any(|step| {
            step.unit_action.command_id == "inspector.focus_unit:heater-1"
                && step
                    .produced_stream_actions
                    .iter()
                    .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
        }));
        assert!(active_detail.diagnostic_actions.iter().any(|action| {
            action.source_label == "Inspector target"
                && action.action.command_id == "inspector.focus_stream:stream-heated"
        }));
        assert!(active_detail.diagnostic_actions.iter().any(|action| {
            action.source_label == "Solve step"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        }));
        assert!(
            active_detail
                .related_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "solver.unit_executed")
        );

        let unit_target_dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "inspector.focus_unit:heater-1".to_string(),
            })
            .expect("expected unit inspector target dispatch");
        let unit_detail = unit_target_dispatch
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active unit inspector detail");
        let unit_result = unit_detail
            .latest_unit_result
            .as_ref()
            .expect("expected active unit detail to expose latest execution result");
        assert_eq!(unit_result.unit_id, "heater-1");
        assert_eq!(unit_result.status_label, "Converged");
        assert_eq!(unit_result.step_index, 1);
        assert_eq!(
            unit_result.produced_stream_ids,
            vec!["stream-heated".to_string()]
        );
        assert!(unit_result.summary.contains("stream-heated"));
        assert!(
            unit_result
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
        );
        assert_eq!(unit_detail.latest_stream_result, None);

        let fallback_inspector = snapshot.result_inspector(Some("missing-stream"));
        assert!(fallback_inspector.has_stale_selection);
        assert_eq!(
            fallback_inspector.selected_stream_id.as_deref(),
            snapshot
                .streams
                .first()
                .map(|stream| stream.stream_id.as_str())
        );

        let comparison_inspector =
            snapshot.result_inspector_with_comparison(Some("stream-feed"), Some("stream-heated"));
        assert_eq!(
            comparison_inspector.selected_stream_id.as_deref(),
            Some("stream-feed")
        );
        assert_eq!(
            comparison_inspector.comparison_stream_id.as_deref(),
            Some("stream-heated")
        );
        assert!(
            comparison_inspector
                .comparison_options
                .iter()
                .all(|option| option.stream_id != "stream-feed")
        );
        let comparison = comparison_inspector
            .comparison
            .as_ref()
            .expect("expected stream comparison model");
        assert!(comparison.summary_rows.iter().any(|row| {
            row.label == "T"
                && row.detail_label == "Temperature"
                && row.base_value.ends_with(" K")
                && row.compared_value.ends_with(" K")
                && row.delta_text.ends_with(" K")
        }));
        assert!(comparison.composition_rows.iter().any(|row| {
            row.component_id == "component-a"
                && !row.base_fraction_text.is_empty()
                && !row.compared_fraction_text.is_empty()
                && (row.delta_text.starts_with('+') || row.delta_text.starts_with('-'))
        }));
        let stale_comparison =
            snapshot.result_inspector_with_comparison(Some("stream-feed"), Some("stream-feed"));
        assert!(stale_comparison.has_stale_comparison);
        assert_eq!(stale_comparison.comparison, None);

        // unit-centric Result Inspector contract
        let default_inspector = snapshot.result_inspector(Some("stream-heated"));
        assert!(
            !default_inspector.unit_options.is_empty(),
            "expected unit options in result inspector"
        );
        assert!(
            default_inspector
                .unit_options
                .iter()
                .any(|option| option.unit_id == "feed-1"
                    && option.focus_action.command_id == "inspector.focus_unit:feed-1"
                    && option.summary.contains("step #")),
            "expected unit option to expose canonical inspector command"
        );
        let default_unit_id = default_inspector
            .selected_unit_id
            .as_deref()
            .expect("expected default unit selection");
        assert!(
            default_inspector
                .unit_options
                .iter()
                .find(|option| option.unit_id == default_unit_id)
                .is_some_and(|option| option.is_selected),
            "expected default unit option to be marked selected"
        );
        assert!(!default_inspector.has_stale_unit_selection);

        let unit_inspector =
            snapshot.result_inspector_with_unit(Some("stream-heated"), None, Some("heater-1"));
        assert_eq!(unit_inspector.selected_unit_id.as_deref(), Some("heater-1"));
        let selected_unit = unit_inspector
            .selected_unit
            .as_ref()
            .expect("expected selected unit execution result");
        assert_eq!(selected_unit.unit_id, "heater-1");
        assert_eq!(selected_unit.status_label, "Converged");
        assert!(
            selected_unit
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == "inspector.focus_stream:stream-heated"),
            "expected unit execution result to expose produced stream actions"
        );
        assert!(
            unit_inspector
                .unit_related_steps
                .iter()
                .all(|step| step.unit_id == "heater-1"),
            "expected unit-related steps to be filtered to the selected unit"
        );
        assert!(
            unit_inspector
                .unit_related_diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == "heater-1")
                    || diagnostic
                        .related_stream_ids
                        .iter()
                        .any(|stream_id| stream_id == "stream-heated")),
            "expected unit-related diagnostics to include the selected unit's outputs"
        );
        assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
            action.source_label == "Selected unit"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        }));
        assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
            action.source_label == "Solve step"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        }));
        assert!(!unit_inspector.has_stale_unit_selection);

        let stale_unit =
            snapshot.result_inspector_with_unit(Some("stream-heated"), None, Some("missing-unit"));
        assert!(stale_unit.has_stale_unit_selection);
        assert_eq!(
            stale_unit.selected_unit_id.as_deref(),
            snapshot.steps.first().map(|step| step.unit_id.as_str())
        );
    }

    #[test]
    fn studio_gui_window_model_surfaces_failure_result_until_rerun_succeeds() {
        let mut driver =
            StudioGuiDriver::new(&unbound_outlet_failure_synced_config()).expect("expected driver");
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let failed = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.run_manual".to_string(),
            })
            .expect("expected failed run dispatch");
        assert_eq!(
            failed.window.runtime.control_state.run_status,
            rf_ui::RunStatus::Error
        );
        assert_eq!(failed.window.runtime.latest_solve_snapshot, None);
        let failure = failed
            .window
            .runtime
            .latest_failure
            .expect("expected visible failure result");
        assert_eq!(failure.status_label, "Error");
        assert_eq!(failure.title, "Unbound outlet port");
        assert!(
            failure.message.contains("unbound_outlet_port"),
            "expected solver diagnostic in failure message, got {}",
            failure.message
        );
        let diagnostic_detail = failure
            .diagnostic_detail
            .as_ref()
            .expect("expected structured failure diagnostic detail");
        assert_eq!(
            diagnostic_detail.primary_code.as_deref(),
            Some("solver.connection_validation.unbound_outlet_port")
        );
        assert_eq!(diagnostic_detail.document_revision, 0);
        assert_eq!(diagnostic_detail.severity_label, "Error");
        assert_eq!(diagnostic_detail.diagnostic_count, 1);
        assert!(
            diagnostic_detail
                .related_units
                .iter()
                .any(|target| target.target_id == "feed-1"
                    && target.action.command_id == "inspector.focus_unit:feed-1")
        );
        assert!(diagnostic_detail.related_ports.iter().any(|target| {
            target.unit_id == "feed-1"
                && target.port_name == "outlet"
                && target.unit_action.command_id == "inspector.focus_unit:feed-1"
        }));
        assert_eq!(failure.recovery_title, Some("Create outlet stream"));
        assert!(failure.recovery_detail.is_some());
        assert_eq!(
            failure
                .recovery_action
                .as_ref()
                .map(|action| (action.label.as_str(), action.command_id.as_str())),
            Some(("Create outlet stream", "run_panel.recover_failure"))
        );
        assert_eq!(
            failure
                .recovery_target
                .as_ref()
                .map(|target| (target.kind_label, target.target_id.as_str())),
            Some(("Unit", "feed-1"))
        );
        assert_eq!(
            failure
                .recovery_target
                .as_ref()
                .map(|target| target.action.command_id.as_str()),
            Some("inspector.focus_unit:feed-1")
        );
        assert!(failure.diagnostic_actions.iter().any(|action| {
            action.source_label == "Recovery"
                && action.target_label == "Run panel"
                && action.action.command_id == "run_panel.recover_failure"
        }));
        assert!(failure.diagnostic_actions.iter().any(|action| {
            action.source_label == "Recovery target"
                && action.target_label == "Unit"
                && action.action.command_id == "inspector.focus_unit:feed-1"
        }));
        assert!(failure.diagnostic_actions.iter().any(|action| {
            action.source_label == "Failure port"
                && action.target_label == "Port"
                && action.summary == "Unit feed-1 port outlet"
                && action.action.command_id == "inspector.focus_unit:feed-1"
        }));
        assert!(failure.latest_log_message.is_some());

        let focused_failure_unit = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "inspector.focus_unit:feed-1".to_string(),
            })
            .expect("expected failure target focus dispatch");
        let failure_detail = focused_failure_unit
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active failure unit inspector detail");
        assert!(failure_detail.unit_ports.iter().any(|port| {
            port.name == "outlet"
                && port.attention_summary.as_ref().is_some_and(|summary| {
                    summary.contains("port feed-1:outlet")
                        && summary.contains("solver.connection_validation.unbound_outlet_port")
                })
        }));

        let recovery = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.recover_failure".to_string(),
            })
            .expect("expected recovery dispatch");
        assert_eq!(
            recovery
                .window
                .runtime
                .active_inspector_target
                .as_ref()
                .map(|target| (target.kind_label, target.target_id.as_str())),
            Some(("Unit", "feed-1"))
        );
        let detail = recovery
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active unit inspector detail");
        assert_eq!(detail.target.target_id, "feed-1");
        assert!(
            detail
                .summary_rows
                .iter()
                .any(|row| row.label == "Kind" && row.value == "feed")
        );
        assert!(detail.unit_ports.iter().any(|port| port.name == "outlet"
            && port.stream_id.as_ref().is_some_and(|stream_id| {
                port.stream_action
                    .as_ref()
                    .map(|action| {
                        action.command_id == format!("inspector.focus_stream:{stream_id}")
                    })
                    .unwrap_or(false)
            })));
        assert!(
            detail
                .unit_ports
                .iter()
                .all(|port| port.attention_summary.is_none()),
            "expected stale failure attention to stay off the post-recovery unit port list"
        );
        let rerun = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.resume_workspace".to_string(),
            })
            .expect("expected successful rerun dispatch");

        assert_eq!(
            rerun.window.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(rerun.window.runtime.latest_failure, None);
        let snapshot = rerun
            .window
            .runtime
            .latest_solve_snapshot
            .expect("expected solve snapshot after recovery rerun");
        let inspector = snapshot.result_inspector(None);
        assert!(inspector.selected_stream.is_some());
        assert!(!inspector.has_stale_selection);
    }

    #[test]
    fn studio_gui_window_command_area_surfaces_palette_items_through_shared_model() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window = dispatch.snapshot.window_model();
        let palette_items = window.commands.palette_items("diagnostic");

        assert_eq!(
            palette_items
                .into_iter()
                .map(|item| (item.command_id, item.label, item.menu_path_text))
                .collect::<Vec<_>>(),
            vec![(
                "run_panel.recover_failure".to_string(),
                "Recover run panel failure (F8) [disabled]".to_string(),
                "Run > Recovery > Recover Run Panel Failure".to_string(),
            )]
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_command_area_surfaces_toolbar_sections_through_shared_model() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window = dispatch.snapshot.window_model();

        assert_eq!(
            window
                .commands
                .toolbar_sections
                .iter()
                .map(|section| section.title)
                .collect::<Vec<_>>(),
            vec![
                "File",
                "Edit",
                "Run Panel",
                "Recovery",
                "Entitlement",
                "Canvas"
            ]
        );
        assert_eq!(
            window.commands.toolbar_sections[2]
                .items
                .iter()
                .map(|item| item.command_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "run_panel.run_manual",
                "run_panel.resume_workspace",
                "run_panel.set_hold",
                "run_panel.set_active",
            ]
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_command_area_surfaces_command_list_sections_through_shared_model() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window = dispatch.snapshot.window_model();

        assert_eq!(
            window
                .commands
                .command_list_sections
                .iter()
                .map(|section| section.title)
                .collect::<Vec<_>>(),
            vec![
                "File",
                "Edit",
                "Run Panel",
                "Recovery",
                "Entitlement",
                "Canvas"
            ]
        );
        assert_eq!(
            window.commands.command_list_sections[2]
                .items
                .iter()
                .map(|item| item.command_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "run_panel.run_manual",
                "run_panel.resume_workspace",
                "run_panel.set_hold",
                "run_panel.set_active",
            ]
        );
        assert!(
            window.commands.command_list_sections[2].items[0]
                .label
                .contains("F5")
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_model_reports_parked_timer_after_last_window_closes() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer dispatch");
        let _ = driver
            .dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id })
            .expect("expected close dispatch");

        let window = driver.snapshot().window_model();

        assert_eq!(window.header.registered_window_count, 0);
        assert_eq!(window.header.foreground_window_id, None);
        assert_eq!(window.header.entitlement_timer_owner_window_id, None);
        assert!(window.header.has_parked_entitlement_timer);
        assert!(window.header.status_line.contains("timer owner: parked"));
        assert_eq!(
            window.layout_state.scope.kind,
            StudioGuiWindowLayoutScopeKind::EmptyWorkspace
        );
        assert_eq!(window.layout_state.scope.layout_key, "studio.window.empty");
        assert_eq!(window.drop_preview, None);
    }

    #[test]
    fn studio_gui_window_model_surfaces_preview_layout_presentation() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected preview dispatch");

        let window = driver.window_model_for_window(Some(window_id));
        let preview = window.drop_preview.expect("expected preview model");
        assert_eq!(preview.overlay.drag_area_id, StudioGuiWindowAreaId::Runtime);
        assert_eq!(
            preview.overlay.target_dock_region,
            StudioGuiWindowDockRegion::LeftSidebar
        );
        assert_eq!(preview.overlay.target_stack_group, 10);
        assert_eq!(
            preview.overlay.target_stack_area_ids,
            vec![StudioGuiWindowAreaId::Runtime]
        );
        assert_eq!(
            preview.changed_area_ids,
            vec![
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowAreaId::Runtime
            ]
        );
        assert_eq!(
            preview
                .preview_layout
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }
}
