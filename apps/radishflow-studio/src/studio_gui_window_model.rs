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
    pub recovery_title: Option<&'static str>,
    pub recovery_detail: Option<&'static str>,
    pub recovery_target: Option<StudioGuiWindowInspectorTargetModel>,
    pub latest_log_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowInspectorTargetModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub summary: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowStreamResultModel {
    pub stream_id: String,
    pub label: String,
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
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowCompositionResultModel {
    pub component_id: String,
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
    pub produced_streams: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowDiagnosticModel {
    pub severity_label: &'static str,
    pub code: String,
    pub message: String,
    pub related_unit_ids: Vec<String>,
    pub related_stream_ids: Vec<String>,
    pub target_candidates: Vec<StudioGuiWindowInspectorTargetModel>,
    pub related_units_text: Option<String>,
    pub related_streams_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowResultInspectorModel {
    pub snapshot_id: String,
    pub selected_stream_id: Option<String>,
    pub selected_stream: Option<StudioGuiWindowStreamResultModel>,
    pub stream_options: Vec<StudioGuiWindowResultInspectorStreamOptionModel>,
    pub related_steps: Vec<StudioGuiWindowSolveStepModel>,
    pub related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub has_stale_selection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowResultInspectorStreamOptionModel {
    pub stream_id: String,
    pub label: String,
    pub summary: String,
    pub is_selected: bool,
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
        let related_steps = selected_stream_id
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
        let related_diagnostics = selected_stream_id
            .as_deref()
            .map(|selected_id| {
                self.diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic
                            .related_stream_ids
                            .iter()
                            .any(|stream_id| stream_id == selected_id)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let stream_options = self
            .streams
            .iter()
            .map(|stream| StudioGuiWindowResultInspectorStreamOptionModel {
                stream_id: stream.stream_id.clone(),
                label: stream.label.clone(),
                summary: format!(
                    "{} | {} | {}",
                    stream.stream_id, stream.temperature_text, stream.molar_flow_text
                ),
                is_selected: selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id == stream.stream_id)
                    .unwrap_or(false),
            })
            .collect();

        StudioGuiWindowResultInspectorModel {
            snapshot_id: self.snapshot_id.clone(),
            selected_stream_id,
            selected_stream,
            stream_options,
            related_steps,
            related_diagnostics,
            has_stale_selection,
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

    Some(StudioGuiWindowFailureResultModel {
        status_label: run_status_label(control_state.run_status),
        title: notice.title.clone(),
        message: notice.message.clone(),
        recovery_title: notice.recovery_action.as_ref().map(|action| action.title),
        recovery_detail: notice.recovery_action.as_ref().map(|action| action.detail),
        recovery_target: notice
            .recovery_action
            .as_ref()
            .and_then(inspector_target_model_from_recovery_action),
        latest_log_message: control_state
            .latest_log_entry
            .as_ref()
            .map(|entry| entry.message.clone()),
    })
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
            StudioGuiWindowInspectorTargetModel {
                kind_label: "Unit",
                summary: format!("Unit {target_id}"),
                command_id: crate::inspector_target_command_id(target),
                target_id,
            }
        }
        rf_ui::InspectorTarget::Stream(stream_id) => {
            let target_id = stream_id.as_str().to_string();
            StudioGuiWindowInspectorTargetModel {
                kind_label: "Stream",
                summary: format!("Stream {target_id}"),
                command_id: crate::inspector_target_command_id(target),
                target_id,
            }
        }
    }
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
            .map(|step| StudioGuiWindowSolveStepModel {
                index: step.index,
                unit_id: step.unit_id.as_str().to_string(),
                summary: step.summary.clone(),
                produced_streams: step
                    .streams
                    .iter()
                    .map(|stream| stream.stream_id.as_str().to_string())
                    .collect(),
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

    StudioGuiWindowDiagnosticModel {
        severity_label: diagnostic_severity_label(diagnostic.severity),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        target_candidates: diagnostic_target_candidates_from_ui(diagnostic),
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

fn stream_result_model_from_ui(
    stream: &rf_ui::StreamStateSnapshot,
) -> StudioGuiWindowStreamResultModel {
    let temperature_text = format!("{:.2} K", stream.temperature_k);
    let pressure_text = format!("{:.0} Pa", stream.pressure_pa);
    let molar_flow_text = format!("{:.6} mol/s", stream.total_molar_flow_mol_s);
    StudioGuiWindowStreamResultModel {
        stream_id: stream.stream_id.as_str().to_string(),
        label: stream.label.clone(),
        temperature_text: temperature_text.clone(),
        pressure_text: pressure_text.clone(),
        molar_flow_text: molar_flow_text.clone(),
        summary_rows: vec![
            StudioGuiWindowStreamSummaryRowModel {
                label: "T",
                value: temperature_text,
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "P",
                value: pressure_text,
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "F",
                value: molar_flow_text,
            },
        ],
        composition_rows: stream
            .overall_mole_fractions
            .iter()
            .map(
                |(component_id, fraction)| StudioGuiWindowCompositionResultModel {
                    component_id: component_id.clone(),
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
            Some("Run Panel")
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
        assert_eq!(window.canvas.enabled_action_count, 4);
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
        assert!(
            heated_stream
                .summary_rows
                .iter()
                .any(|row| row.label == "T" && row.value == "345.00 K")
        );
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
                .any(|option| option.stream_id == "stream-heated" && option.is_selected)
        );
        assert!(
            inspector
                .related_steps
                .iter()
                .any(|step| step.unit_id == "heater-1")
        );
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

        let fallback_inspector = snapshot.result_inspector(Some("missing-stream"));
        assert!(fallback_inspector.has_stale_selection);
        assert_eq!(
            fallback_inspector.selected_stream_id.as_deref(),
            snapshot
                .streams
                .first()
                .map(|stream| stream.stream_id.as_str())
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
        assert_eq!(failure.recovery_title, Some("Create outlet stream"));
        assert!(failure.recovery_detail.is_some());
        assert_eq!(
            failure
                .recovery_target
                .as_ref()
                .map(|target| (target.kind_label, target.target_id.as_str())),
            Some(("Unit", "feed-1"))
        );
        assert!(failure.latest_log_message.is_some());

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
            vec!["Run Panel", "Recovery", "Entitlement", "Canvas"]
        );
        assert_eq!(
            window.commands.toolbar_sections[0]
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
            vec!["Run Panel", "Recovery", "Entitlement", "Canvas"]
        );
        assert_eq!(
            window.commands.command_list_sections[0]
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
            window.commands.command_list_sections[0].items[0]
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
