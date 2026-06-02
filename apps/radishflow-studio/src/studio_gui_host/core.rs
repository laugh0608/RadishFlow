use super::helpers::{
    dispatch_from_controller, global_event_from_controller, ui_commands_from_projection,
};
use super::*;
use crate::{
    StudioGuiDiagnosticStreamSnapshot, StudioGuiFailureDiagnosticContextSnapshot,
    StudioGuiFailureDiagnosticPortSnapshot, WorkspaceControlState,
    studio_stream_reconnect_presentation::{
        StudioStreamReconnectAvailability, StudioStreamReconnectMissingSide,
        stream_reconnect_already_connected, stream_reconnect_from_candidates,
        stream_reconnect_missing_endpoint, stream_reconnect_missing_stream,
    },
};
use std::collections::BTreeSet;

impl StudioGuiHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let controller = StudioAppHostController::new(config)?;
        let layout_state_overrides = match controller.document_path() {
            Some(project_path) => load_persisted_window_layouts(project_path)?,
            None => BTreeMap::new(),
        };
        let mut canvas_unit_positions = match controller.document_path() {
            Some(project_path) => load_persisted_canvas_unit_positions(project_path)?,
            None => BTreeMap::new(),
        };
        let flowsheet = &controller.document().flowsheet;
        canvas_unit_positions.retain(|unit_id, _| flowsheet.units.contains_key(unit_id));

        Ok(Self {
            controller,
            layout_state_overrides,
            canvas_unit_positions,
            window_drop_previews: BTreeMap::new(),
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.controller.state()
    }

    pub fn document(&self) -> &rf_ui::FlowsheetDocument {
        self.controller.document()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.state().ui_command_model()
    }

    pub fn canvas_state(&self) -> StudioGuiCanvasState {
        let canvas = self.controller.canvas_interaction();
        let control_state = self.controller.workspace_control_state();
        let latest_solve_snapshot = self.controller.latest_solve_snapshot();
        let active_unit_id = match self.controller.active_inspector_target() {
            Some(rf_ui::InspectorTarget::Unit(unit_id)) => Some(unit_id),
            _ => None,
        };
        let active_stream_id = match self.controller.active_inspector_target() {
            Some(rf_ui::InspectorTarget::Stream(stream_id)) => Some(stream_id),
            _ => None,
        };
        let flowsheet = &self.controller.document().flowsheet;
        let units = canvas_units_in_layout_order(flowsheet)
            .into_iter()
            .map(|unit| StudioGuiCanvasUnitState {
                unit_id: unit.id.clone(),
                name: unit.name.clone(),
                kind: unit.kind.clone(),
                layout_position: self.canvas_unit_positions.get(&unit.id).copied(),
                ports: unit
                    .ports
                    .iter()
                    .map(|port| StudioGuiCanvasUnitPortState {
                        name: port.name.clone(),
                        direction: port.direction,
                        kind: port.kind,
                        stream_id: port.stream_id.clone(),
                    })
                    .collect(),
                port_count: unit.ports.len(),
                connected_port_count: unit
                    .ports
                    .iter()
                    .filter(|port| port.stream_id.is_some())
                    .count(),
                is_active_inspector_target: active_unit_id.as_ref() == Some(&unit.id),
            })
            .collect();
        let stream_endpoints = canvas_material_stream_endpoints(flowsheet);
        let streams = flowsheet
            .streams
            .values()
            .flat_map(|stream| {
                let endpoint = stream_endpoints
                    .get(&stream.id)
                    .cloned()
                    .unwrap_or_default();
                let is_active_inspector_target = active_stream_id.as_ref() == Some(&stream.id);
                if endpoint.source.is_none() && endpoint.sinks.is_empty() {
                    return Vec::new();
                }
                if endpoint.sinks.is_empty() {
                    return vec![StudioGuiCanvasStreamState {
                        stream_id: stream.id.clone(),
                        name: stream.name.clone(),
                        source: endpoint.source,
                        sink: None,
                        is_active_inspector_target,
                    }];
                }
                endpoint
                    .sinks
                    .into_iter()
                    .map(|sink| StudioGuiCanvasStreamState {
                        stream_id: stream.id.clone(),
                        name: stream.name.clone(),
                        source: endpoint.source.clone(),
                        sink: Some(sink),
                        is_active_inspector_target,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let diagnostics = canvas_diagnostics_from_runtime(
            latest_solve_snapshot.as_ref(),
            control_state.latest_diagnostic.as_ref(),
            control_state.notice.as_ref(),
            self.controller.document().revision,
        );
        StudioGuiCanvasState {
            view_mode: canvas.view_mode,
            units,
            streams,
            run_status: Some(control_state.run_status),
            pending_reason: control_state.pending_reason,
            latest_snapshot_id: control_state.latest_snapshot_id,
            latest_snapshot_summary: control_state.latest_snapshot_summary,
            diagnostics,
            suggestions: canvas.suggestions,
            focused_suggestion_id: canvas.focused_suggestion_id,
            pending_edit: canvas.pending_edit,
        }
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        let latest_solve_snapshot = self.controller.latest_solve_snapshot();
        StudioGuiCommandRegistry::from_surfaces_with_results(
            &self.ui_commands(),
            &self.canvas_state(),
            self.preferred_target_window_id(),
            latest_solve_snapshot.as_ref(),
        )
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        let workspace_document = workspace_document_snapshot_from_controller(&self.controller);
        let control_state = self.controller.workspace_control_state();
        let run_panel = self.controller.run_panel_widget();
        let latest_solve_snapshot = self.controller.latest_solve_snapshot();
        let stale_solve_snapshot = self.controller.stale_solve_snapshot();
        let latest_failure_diagnostic_context = failure_diagnostic_context_from_controller(
            &self.controller,
            &control_state,
            workspace_document.revision,
        );
        let active_inspector_target = self.controller.active_inspector_target();
        let active_inspector_detail = active_inspector_detail_from_controller(&self.controller);
        let entitlement_host = self.controller.entitlement_host_output();
        let log_entries = self.controller.log_entries();
        let mut snapshot = StudioGuiSnapshot::new(
            self.state().clone(),
            self.ui_commands(),
            self.command_registry(),
            self.canvas_state().widget(),
            StudioGuiRuntimeSnapshot {
                workspace_document,
                example_projects: crate::studio_example_project_models(
                    self.controller.document_path(),
                ),
                control_state,
                run_panel,
                latest_solve_snapshot,
                stale_solve_snapshot,
                latest_failure_diagnostic_context,
                active_inspector_target,
                active_inspector_detail,
                entitlement_host,
                platform_notice: None,
                platform_timer_lines: Vec::new(),
                gui_activity_lines: Vec::new(),
                log_entries,
            },
            self.window_drop_previews.clone(),
        );
        snapshot.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, None);
        snapshot
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        let snapshot = self.snapshot();
        let mut window = snapshot.window_model_for_window(window_id);
        window.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        window
    }

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.controller.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.controller.replace_canvas_suggestions(suggestions);
    }

    pub fn execute_command(
        &mut self,
        command: StudioGuiHostCommand,
    ) -> RfResult<StudioGuiHostCommandOutcome> {
        match command {
            StudioGuiHostCommand::OpenWindow => self
                .open_window()
                .map(StudioGuiHostCommandOutcome::WindowOpened),
            StudioGuiHostCommand::DispatchWindowTrigger { window_id, trigger } => self
                .dispatch_window_trigger(window_id, trigger)
                .map(StudioGuiHostCommandOutcome::WindowDispatched),
            StudioGuiHostCommand::DispatchCanvasInteraction { action } => self
                .dispatch_canvas_interaction(action)
                .map(StudioGuiHostCommandOutcome::CanvasInteracted),
            StudioGuiHostCommand::MoveCanvasUnitLayout { unit_id, position } => self
                .move_canvas_unit_layout(unit_id, position)
                .map(StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved),
            StudioGuiHostCommand::DispatchLifecycleEvent { event } => self
                .dispatch_lifecycle_event(event)
                .map(StudioGuiHostCommandOutcome::LifecycleDispatched),
            StudioGuiHostCommand::DispatchUiCommand { command_id } => self
                .dispatch_ui_command(&command_id)
                .map(StudioGuiHostCommandOutcome::UiCommandDispatched),
            StudioGuiHostCommand::DispatchInspectorDraftUpdate {
                command_id,
                raw_value,
            } => self
                .dispatch_inspector_draft_update(&command_id, raw_value)
                .map(StudioGuiHostCommandOutcome::InspectorDraftUpdated),
            StudioGuiHostCommand::DispatchInspectorDraftCommit { command_id } => self
                .dispatch_inspector_draft_commit(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorDraftCommitted),
            StudioGuiHostCommand::DispatchInspectorDraftDiscard { command_id } => self
                .dispatch_inspector_draft_discard(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorDraftDiscarded),
            StudioGuiHostCommand::DispatchInspectorDraftBatchCommit { command_id } => self
                .dispatch_inspector_draft_batch_commit(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted),
            StudioGuiHostCommand::DispatchInspectorDraftBatchDiscard { command_id } => self
                .dispatch_inspector_draft_batch_discard(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded),
            StudioGuiHostCommand::DispatchInspectorCompositionNormalize { command_id } => self
                .dispatch_inspector_composition_normalize(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorCompositionNormalized),
            StudioGuiHostCommand::DispatchInspectorCompositionComponentAdd { command_id } => self
                .dispatch_inspector_composition_component_add(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded),
            StudioGuiHostCommand::DispatchInspectorCompositionComponentRemove { command_id } => {
                self.dispatch_inspector_composition_component_remove(&command_id)
                    .map(StudioGuiHostCommandOutcome::InspectorCompositionComponentRemoved)
            }
            StudioGuiHostCommand::QueryWindowDropTarget { window_id, query } => self
                .query_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetQueried),
            StudioGuiHostCommand::SetWindowDropTargetPreview { window_id, query } => self
                .set_window_drop_target_preview(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated),
            StudioGuiHostCommand::ClearWindowDropTargetPreview { window_id } => self
                .clear_window_drop_target_preview(window_id)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared),
            StudioGuiHostCommand::ApplyWindowDropTarget { window_id, query } => self
                .apply_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetApplied),
            StudioGuiHostCommand::CloseWindow { window_id } => self
                .close_window(window_id)
                .map(StudioGuiHostCommandOutcome::WindowClosed),
        }
    }

    pub fn open_window(&mut self) -> RfResult<StudioGuiHostWindowOpened> {
        let opened = self.controller.open_window()?;
        Ok(StudioGuiHostWindowOpened {
            ui_commands: ui_commands_from_projection(&opened.projection),
            canvas: self.canvas_state(),
            projection: opened.projection,
            registration: opened.registration,
            native_timers: StudioGuiNativeTimerEffects::from_driver(
                &opened.native_timer_transitions,
                &opened.native_timer_acks,
            ),
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self
            .controller
            .dispatch_window_trigger(window_id, trigger)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self.controller.focus_window(window_id)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioGuiHostGlobalEventDispatch> {
        let result = self.controller.dispatch_global_event(event)?;
        Ok(global_event_from_controller(result, self.canvas_state()))
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostCloseWindowResult> {
        if self.state().window(window_id).is_some() {
            let snapshot = self.snapshot();
            let layout_state =
                self.layout_state_for_window_from_snapshot(&snapshot, Some(window_id));
            self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
            if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
                self.clear_window_drop_preview_for_scope(&legacy_layout_key);
            }
        }
        let closed = self.controller.close_window(window_id)?;
        Ok(StudioGuiHostCloseWindowResult {
            ui_commands: ui_commands_from_projection(&closed.projection),
            canvas: self.canvas_state(),
            projection: closed.projection,
            native_timers: closed
                .close
                .as_ref()
                .map(|close| {
                    StudioGuiNativeTimerEffects::from_driver(
                        &close.native_timer_transitions,
                        &close.native_timer_acks,
                    )
                })
                .unwrap_or_default(),
            close: closed.close,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct CanvasMaterialStreamEndpoints {
    source: Option<StudioGuiCanvasStreamEndpointState>,
    sinks: Vec<StudioGuiCanvasStreamEndpointState>,
}

fn canvas_material_stream_endpoints(
    flowsheet: &rf_model::Flowsheet,
) -> BTreeMap<rf_types::StreamId, CanvasMaterialStreamEndpoints> {
    let mut endpoints = BTreeMap::<rf_types::StreamId, CanvasMaterialStreamEndpoints>::new();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.clone() else {
                continue;
            };
            let endpoint = StudioGuiCanvasStreamEndpointState {
                unit_id: unit.id.clone(),
                port_name: port.name.clone(),
            };
            let entry = endpoints.entry(stream_id).or_default();
            match port.direction {
                rf_types::PortDirection::Outlet => {
                    if entry.source.is_none() {
                        entry.source = Some(endpoint);
                    }
                }
                rf_types::PortDirection::Inlet => {
                    entry.sinks.push(endpoint);
                }
            }
        }
    }
    endpoints
}

fn canvas_units_in_layout_order(flowsheet: &rf_model::Flowsheet) -> Vec<&rf_model::UnitNode> {
    let mut source_unit_by_stream = BTreeMap::new();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material
                || port.direction != rf_types::PortDirection::Outlet
            {
                continue;
            }
            if let Some(stream_id) = port.stream_id.as_ref() {
                source_unit_by_stream
                    .entry(stream_id.clone())
                    .or_insert_with(|| unit.id.clone());
            }
        }
    }

    let mut dependencies = flowsheet
        .units
        .keys()
        .map(|unit_id| (unit_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material
                || port.direction != rf_types::PortDirection::Inlet
            {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            let Some(source_unit_id) = source_unit_by_stream.get(stream_id) else {
                continue;
            };
            if source_unit_id != &unit.id {
                dependencies
                    .entry(unit.id.clone())
                    .or_default()
                    .insert(source_unit_id.clone());
            }
        }
    }

    let mut ordered_unit_ids = Vec::new();
    while !dependencies.is_empty() {
        let ready = dependencies
            .iter()
            .find_map(|(unit_id, upstream)| upstream.is_empty().then(|| unit_id.clone()));
        let Some(unit_id) = ready else {
            ordered_unit_ids.extend(dependencies.keys().cloned());
            break;
        };

        dependencies.remove(&unit_id);
        for upstream in dependencies.values_mut() {
            upstream.remove(&unit_id);
        }
        ordered_unit_ids.push(unit_id);
    }

    ordered_unit_ids
        .into_iter()
        .filter_map(|unit_id| flowsheet.units.get(&unit_id))
        .collect()
}

fn canvas_diagnostics_from_runtime(
    latest_solve_snapshot: Option<&rf_ui::SolveSnapshot>,
    latest_diagnostic: Option<&rf_ui::DiagnosticSummary>,
    notice: Option<&rf_ui::RunPanelNotice>,
    document_revision: u64,
) -> Vec<StudioGuiCanvasDiagnosticState> {
    if let Some(snapshot) = latest_solve_snapshot {
        return snapshot
            .diagnostics
            .iter()
            .map(|diagnostic| StudioGuiCanvasDiagnosticState {
                severity: diagnostic.severity,
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                related_unit_ids: diagnostic.related_unit_ids.clone(),
                related_stream_ids: diagnostic.related_stream_ids.clone(),
                related_port_targets: diagnostic.related_port_targets.clone(),
            })
            .collect();
    }

    if let Some(diagnostic) =
        latest_diagnostic.filter(|diagnostic| diagnostic.document_revision == document_revision)
    {
        return vec![StudioGuiCanvasDiagnosticState {
            severity: diagnostic.highest_severity,
            code: diagnostic
                .primary_code
                .clone()
                .unwrap_or_else(|| "run_panel.diagnostic".to_string()),
            message: diagnostic.primary_message.clone(),
            related_unit_ids: diagnostic.related_unit_ids.clone(),
            related_stream_ids: diagnostic.related_stream_ids.clone(),
            related_port_targets: diagnostic.related_port_targets.clone(),
        }];
    }

    let Some(notice) = notice else {
        return Vec::new();
    };
    let severity = match notice.level {
        rf_ui::RunPanelNoticeLevel::Info => rf_ui::DiagnosticSeverity::Info,
        rf_ui::RunPanelNoticeLevel::Warning => rf_ui::DiagnosticSeverity::Warning,
        rf_ui::RunPanelNoticeLevel::Error => rf_ui::DiagnosticSeverity::Error,
    };
    let (related_unit_ids, related_stream_ids, related_port_targets) = notice
        .recovery_action
        .as_ref()
        .map(|action| {
            let related_unit_ids = action.target_unit_id.clone().into_iter().collect();
            let related_stream_ids = action.target_stream_id.clone().into_iter().collect();
            let related_port_targets = action
                .target_unit_id
                .as_ref()
                .zip(action.target_port_name.as_ref())
                .map(|(unit_id, port_name)| {
                    rf_types::DiagnosticPortTarget::new(unit_id.clone(), port_name.clone())
                })
                .into_iter()
                .collect();
            (related_unit_ids, related_stream_ids, related_port_targets)
        })
        .unwrap_or_default();

    vec![StudioGuiCanvasDiagnosticState {
        severity,
        code: "run_panel.notice".to_string(),
        message: format!("{}: {}", notice.title, notice.message),
        related_unit_ids,
        related_stream_ids,
        related_port_targets,
    }]
}

fn failure_diagnostic_context_from_controller(
    controller: &StudioAppHostController,
    control_state: &WorkspaceControlState,
    document_revision: u64,
) -> Option<StudioGuiFailureDiagnosticContextSnapshot> {
    let diagnostic = control_state.latest_diagnostic.as_ref()?;
    if diagnostic.document_revision != document_revision {
        return None;
    }

    let flowsheet = &controller.document().flowsheet;
    Some(StudioGuiFailureDiagnosticContextSnapshot {
        related_streams: diagnostic
            .related_stream_ids
            .iter()
            .filter_map(|stream_id| flowsheet.streams.get(stream_id))
            .map(diagnostic_stream_snapshot_from_model)
            .collect(),
        related_ports: diagnostic
            .related_port_targets
            .iter()
            .map(|target| {
                let stream = flowsheet
                    .units
                    .get(&target.unit_id)
                    .and_then(|unit| unit.ports.iter().find(|port| port.name == target.port_name))
                    .and_then(|port| port.stream_id.as_ref())
                    .and_then(|stream_id| flowsheet.streams.get(stream_id))
                    .map(diagnostic_stream_snapshot_from_model);
                StudioGuiFailureDiagnosticPortSnapshot {
                    unit_id: target.unit_id.as_str().to_string(),
                    port_name: target.port_name.clone(),
                    stream,
                }
            })
            .collect(),
    })
}

fn diagnostic_stream_snapshot_from_model(
    stream: &rf_model::MaterialStreamState,
) -> StudioGuiDiagnosticStreamSnapshot {
    StudioGuiDiagnosticStreamSnapshot {
        stream_id: stream.id.as_str().to_string(),
        temperature_k: stream.temperature_k,
        pressure_pa: stream.pressure_pa,
        total_molar_flow_mol_s: stream.total_molar_flow_mol_s,
        overall_mole_fractions: stream
            .overall_mole_fractions
            .iter()
            .map(|(component_id, fraction)| (component_id.as_str().to_string(), *fraction))
            .collect(),
    }
}

fn active_inspector_detail_from_controller(
    controller: &StudioAppHostController,
) -> Option<StudioGuiInspectorTargetDetailSnapshot> {
    let target = controller.active_inspector_target()?;
    let flowsheet = &controller.document().flowsheet;

    match &target {
        rf_ui::InspectorTarget::Unit(unit_id) => {
            let unit = flowsheet.units.get(unit_id)?;
            let property_fields =
                unit_property_fields(flowsheet, unit, controller.inspector_drafts());
            let property_notices = unit_property_notices(unit, &property_fields);
            Some(StudioGuiInspectorTargetDetailSnapshot {
                target,
                title: unit.name.clone(),
                summary_rows: vec![
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "Id".to_string(),
                        value: unit.id.as_str().to_string(),
                    },
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "Kind".to_string(),
                        value: unit.kind.clone(),
                    },
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "Ports".to_string(),
                        value: unit.ports.len().to_string(),
                    },
                ],
                property_fields,
                property_notices,
                property_composition_summary: None,
                property_batch_commit_command_id: None,
                property_batch_discard_command_id: None,
                property_composition_normalize_command_id: None,
                property_composition_component_actions: Vec::new(),
                connection_actions: Vec::new(),
                unit_ports: unit
                    .ports
                    .iter()
                    .map(|port| StudioGuiInspectorTargetPortSnapshot {
                        name: port.name.clone(),
                        direction: port.direction.as_str().to_string(),
                        kind: port.kind.as_str().to_string(),
                        stream_id: port
                            .stream_id
                            .as_ref()
                            .map(|stream_id| stream_id.as_str().to_string()),
                    })
                    .collect(),
            })
        }
        rf_ui::InspectorTarget::Stream(stream_id) => {
            let stream = flowsheet.streams.get(stream_id)?;
            let property_fields = stream_property_fields(stream, controller.inspector_drafts());
            let property_composition_summary =
                stream_property_composition_summary(stream, controller.inspector_drafts());
            let property_composition_normalize_command_id =
                stream_property_composition_normalize_command_id(
                    stream,
                    controller.inspector_drafts(),
                );
            let property_composition_component_actions =
                stream_property_composition_component_actions(stream, flowsheet);
            let property_notices =
                stream_property_notices(stream, controller.inspector_drafts(), &property_fields);
            Some(StudioGuiInspectorTargetDetailSnapshot {
                target,
                title: stream.name.clone(),
                summary_rows: vec![
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "Id".to_string(),
                        value: stream.id.as_str().to_string(),
                    },
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "T".to_string(),
                        value: format!("{:.2} K", stream.temperature_k),
                    },
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "P".to_string(),
                        value: format!("{:.0} Pa", stream.pressure_pa),
                    },
                    StudioGuiInspectorTargetSummaryRowSnapshot {
                        label: "F".to_string(),
                        value: format!("{:.6} mol/s", stream.total_molar_flow_mol_s),
                    },
                ],
                property_batch_commit_command_id: stream_property_batch_commit_command_id(
                    stream,
                    &property_fields,
                ),
                property_batch_discard_command_id: stream_property_batch_discard_command_id(
                    stream,
                    &property_fields,
                ),
                property_notices,
                property_composition_summary,
                property_composition_normalize_command_id,
                property_composition_component_actions,
                connection_actions: stream_connection_actions(flowsheet, &stream.id),
                property_fields,
                unit_ports: Vec::new(),
            })
        }
    }
}

fn stream_connection_actions(
    flowsheet: &rf_model::Flowsheet,
    stream_id: &rf_types::StreamId,
) -> Vec<StudioGuiInspectorConnectionActionSnapshot> {
    let stream_id_label = stream_id.as_str();
    let mut actions = Vec::new();
    if has_material_stream_endpoint(flowsheet, stream_id) {
        actions.push(StudioGuiInspectorConnectionActionSnapshot {
            label: "Disconnect stream".to_string(),
            detail: format!(
                "Remove material port bindings for `{stream_id_label}` while keeping the stream specification."
            ),
            command_id: "canvas.disconnect_selected_stream".to_string(),
            enabled: true,
        });
    }
    if let Some(detail) = stream_endpoint_disconnect_detail(
        flowsheet,
        stream_id,
        rf_types::PortDirection::Outlet,
        "upstream source",
        "downstream bindings",
    ) {
        actions.push(StudioGuiInspectorConnectionActionSnapshot {
            label: "Disconnect source".to_string(),
            detail,
            command_id: "canvas.disconnect_selected_stream_source".to_string(),
            enabled: true,
        });
    }
    if let Some(detail) = stream_endpoint_disconnect_detail(
        flowsheet,
        stream_id,
        rf_types::PortDirection::Inlet,
        "downstream sink",
        "upstream bindings",
    ) {
        actions.push(StudioGuiInspectorConnectionActionSnapshot {
            label: "Disconnect sink".to_string(),
            detail,
            command_id: "canvas.disconnect_selected_stream_sink".to_string(),
            enabled: true,
        });
    }
    let reconnect = stream_reconnect_availability(flowsheet, stream_id);
    actions.push(StudioGuiInspectorConnectionActionSnapshot {
        label: "Reconnect stream".to_string(),
        detail: reconnect.action_detail(stream_id.as_str()),
        command_id: "canvas.reconnect_selected_stream".to_string(),
        enabled: reconnect.is_available(),
    });
    actions.extend([StudioGuiInspectorConnectionActionSnapshot {
        label: "Delete stream".to_string(),
        detail: format!(
            "Remove material port bindings for `{stream_id_label}` and delete the stream."
        ),
        command_id: "canvas.delete_selected_stream".to_string(),
        enabled: true,
    }]);
    actions
}

fn has_material_stream_endpoint(
    flowsheet: &rf_model::Flowsheet,
    stream_id: &rf_types::StreamId,
) -> bool {
    flowsheet.units.values().any(|unit| {
        unit.ports.iter().any(|port| {
            port.kind == rf_types::PortKind::Material && port.stream_id.as_ref() == Some(stream_id)
        })
    })
}

fn stream_endpoint_disconnect_detail(
    flowsheet: &rf_model::Flowsheet,
    stream_id: &rf_types::StreamId,
    direction: rf_types::PortDirection,
    endpoint_name: &str,
    kept_bindings_name: &str,
) -> Option<String> {
    let endpoint = unique_material_stream_endpoint_label(flowsheet, stream_id, direction)?;
    Some(format!(
        "Disconnect {endpoint_name} `{endpoint}` from `{}` while keeping the stream specification and {kept_bindings_name}.",
        stream_id.as_str()
    ))
}

fn unique_material_stream_endpoint_label(
    flowsheet: &rf_model::Flowsheet,
    stream_id: &rf_types::StreamId,
    direction: rf_types::PortDirection,
) -> Option<String> {
    let mut endpoints = flowsheet
        .units
        .values()
        .flat_map(|unit| {
            unit.ports
                .iter()
                .filter(move |port| {
                    port.kind == rf_types::PortKind::Material
                        && port.direction == direction
                        && port.stream_id.as_ref() == Some(stream_id)
                })
                .map(|port| format!("{}:{}", unit.id, port.name))
        })
        .collect::<Vec<_>>();
    (endpoints.len() == 1).then(|| endpoints.remove(0))
}

fn stream_reconnect_availability(
    flowsheet: &rf_model::Flowsheet,
    stream_id: &rf_types::StreamId,
) -> StudioStreamReconnectAvailability {
    if !flowsheet.streams.contains_key(stream_id) {
        return stream_reconnect_missing_stream();
    }

    let mut source_unit_id = None;
    let mut sink_unit_id = None;
    let mut latest_sink_binding = None;
    let mut source_count = 0usize;
    let mut sink_count = 0usize;

    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material
                || port.stream_id.as_ref() != Some(stream_id)
            {
                continue;
            }

            match port.direction {
                rf_types::PortDirection::Outlet => {
                    source_count += 1;
                    source_unit_id = Some(unit.id.clone());
                }
                rf_types::PortDirection::Inlet => {
                    sink_count += 1;
                    sink_unit_id = Some(unit.id.clone());
                    latest_sink_binding = Some(format!("{}:{}", unit.id, port.name));
                }
            }
        }
    }

    match (source_count, sink_count) {
        (1, 0) => {
            let Some(source_unit_id) = source_unit_id else {
                return stream_reconnect_missing_endpoint();
            };
            let candidates = material_reconnect_sink_candidates(flowsheet, &source_unit_id);
            stream_reconnect_from_host_candidates(
                stream_id.as_str(),
                StudioStreamReconnectMissingSide::Sink,
                candidates,
            )
        }
        (0, 1) => {
            let Some(sink_unit_id) = sink_unit_id else {
                return stream_reconnect_missing_endpoint();
            };
            let candidates = material_reconnect_source_candidates(flowsheet, &sink_unit_id);
            stream_reconnect_from_host_candidates(
                stream_id.as_str(),
                StudioStreamReconnectMissingSide::Source,
                candidates,
            )
        }
        (1, 1) => stream_reconnect_already_connected(
            latest_sink_binding.unwrap_or_else(|| "connected sink".to_string()),
        ),
        _ => stream_reconnect_missing_endpoint(),
    }
}

fn stream_reconnect_from_host_candidates(
    stream_id: &str,
    missing_side: StudioStreamReconnectMissingSide,
    candidates: Vec<(String, bool)>,
) -> StudioStreamReconnectAvailability {
    let candidate_count = candidates.len();
    let cycle_blocked_count = candidates
        .iter()
        .filter(|(_, would_create_cycle)| *would_create_cycle)
        .count();
    let available_targets = candidates
        .into_iter()
        .filter_map(|(label, would_create_cycle)| (!would_create_cycle).then_some(label))
        .collect::<Vec<_>>();
    stream_reconnect_from_candidates(
        stream_id,
        missing_side,
        candidate_count,
        cycle_blocked_count,
        available_targets,
    )
}

fn material_reconnect_sink_candidates(
    flowsheet: &rf_model::Flowsheet,
    source_unit_id: &rf_types::UnitId,
) -> Vec<(String, bool)> {
    flowsheet
        .units
        .values()
        .filter(|unit| &unit.id != source_unit_id)
        .flat_map(|unit| {
            unit.ports
                .iter()
                .filter(move |port| {
                    port.kind == rf_types::PortKind::Material
                        && port.direction == rf_types::PortDirection::Inlet
                        && port.stream_id.is_none()
                })
                .map(|port| {
                    (
                        format!("{}:{}", unit.id, port.name),
                        would_create_unit_dependency_cycle(flowsheet, source_unit_id, &unit.id),
                    )
                })
        })
        .collect()
}

fn material_reconnect_source_candidates(
    flowsheet: &rf_model::Flowsheet,
    sink_unit_id: &rf_types::UnitId,
) -> Vec<(String, bool)> {
    flowsheet
        .units
        .values()
        .filter(|unit| &unit.id != sink_unit_id)
        .flat_map(|unit| {
            unit.ports
                .iter()
                .filter(move |port| {
                    port.kind == rf_types::PortKind::Material
                        && port.direction == rf_types::PortDirection::Outlet
                        && port.stream_id.is_none()
                })
                .map(|port| {
                    (
                        format!("{}:{}", unit.id, port.name),
                        would_create_unit_dependency_cycle(flowsheet, &unit.id, sink_unit_id),
                    )
                })
        })
        .collect()
}

fn would_create_unit_dependency_cycle(
    flowsheet: &rf_model::Flowsheet,
    source_unit_id: &rf_types::UnitId,
    sink_unit_id: &rf_types::UnitId,
) -> bool {
    if source_unit_id == sink_unit_id {
        return true;
    }

    let mut source_by_stream = BTreeMap::<rf_types::StreamId, rf_types::UnitId>::new();
    let mut sinks_by_stream = BTreeMap::<rf_types::StreamId, Vec<rf_types::UnitId>>::new();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            match port.direction {
                rf_types::PortDirection::Outlet => {
                    source_by_stream
                        .entry(stream_id.clone())
                        .or_insert_with(|| unit.id.clone());
                }
                rf_types::PortDirection::Inlet => {
                    sinks_by_stream
                        .entry(stream_id.clone())
                        .or_default()
                        .push(unit.id.clone());
                }
            }
        }
    }

    let mut downstream_units = BTreeMap::<rf_types::UnitId, Vec<rf_types::UnitId>>::new();
    for (stream_id, source) in source_by_stream {
        if let Some(sinks) = sinks_by_stream.get(&stream_id) {
            downstream_units
                .entry(source)
                .or_default()
                .extend(sinks.clone());
        }
    }

    let mut stack = vec![sink_unit_id.clone()];
    let mut visited = BTreeSet::new();
    while let Some(unit_id) = stack.pop() {
        if !visited.insert(unit_id.clone()) {
            continue;
        }
        if &unit_id == source_unit_id {
            return true;
        }
        if let Some(children) = downstream_units.get(&unit_id) {
            stack.extend(children.iter().cloned());
        }
    }

    false
}

fn unit_property_fields(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
) -> Vec<StudioGuiInspectorTargetFieldSnapshot> {
    match unit.kind.as_str() {
        "feed" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Source temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Source pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        "heater" | "cooler" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Outlet temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Outlet pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        "valve" => unit_number_property_field(
            flowsheet,
            unit,
            drafts,
            rf_ui::UnitInspectorDraftField::OutletPressurePa,
            "Outlet pressure (Pa)",
        )
        .into_iter()
        .collect(),
        "mixer" => unit_number_property_field(
            flowsheet,
            unit,
            drafts,
            rf_ui::UnitInspectorDraftField::OutletPressurePa,
            "Outlet pressure (Pa)",
        )
        .into_iter()
        .collect(),
        "flash_drum" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Flash temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Flash pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        _ => Vec::new(),
    }
}

fn unit_number_property_field(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
    field: rf_ui::UnitInspectorDraftField,
    label: &str,
) -> Option<StudioGuiInspectorTargetFieldSnapshot> {
    let original = rf_ui::unit_inspector_parameter_value(flowsheet, &unit.id, &field)?;
    let key = rf_ui::unit_inspector_draft_key(&unit.id, &field);
    let mut property_field = inspector_number_field(drafts, key.clone(), label, original);
    if unit_parameter_display_value_needs_explicit_commit(flowsheet, unit, drafts, &field, original)
    {
        property_field.is_dirty = true;
        property_field.validation = StudioGuiInspectorTargetFieldValidationSnapshot::Valid;
        property_field.commit_command_id = Some(crate::inspector_draft_commit_command_id(&key));
        property_field.discard_command_id = None;
    }
    property_field.constraint_text = Some(unit_parameter_constraint_text(flowsheet, unit, &field));
    Some(property_field)
}

fn unit_parameter_display_value_needs_explicit_commit(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
    field: &rf_ui::UnitInspectorDraftField,
    value: f64,
) -> bool {
    let key = rf_ui::unit_inspector_draft_key(&unit.id, field);
    if drafts.fields.contains_key(&key) || rf_ui::unit_inspector_parameter_is_explicit(unit, field)
    {
        return false;
    }
    if !value.is_finite() || value <= 0.0 {
        return false;
    }
    if matches!(field, rf_ui::UnitInspectorDraftField::OutletPressurePa)
        && unit_outlet_pressure_cannot_exceed_inlet(unit)
    {
        return connected_inlet_pressure_limit(flowsheet, unit)
            .map(|pressure_pa| value <= pressure_pa)
            .unwrap_or(true);
    }
    true
}

fn stream_property_fields(
    stream: &rf_model::MaterialStreamState,
    drafts: &rf_ui::InspectorDraftState,
) -> Vec<StudioGuiInspectorTargetFieldSnapshot> {
    vec![
        inspector_text_field(
            drafts,
            rf_ui::stream_inspector_draft_key(&stream.id, &rf_ui::StreamInspectorDraftField::Name),
            "Name",
            stream.name.clone(),
        ),
        inspector_number_field(
            drafts,
            rf_ui::stream_inspector_draft_key(
                &stream.id,
                &rf_ui::StreamInspectorDraftField::TemperatureK,
            ),
            "Temperature (K)",
            stream.temperature_k,
        ),
        inspector_number_field(
            drafts,
            rf_ui::stream_inspector_draft_key(
                &stream.id,
                &rf_ui::StreamInspectorDraftField::PressurePa,
            ),
            "Pressure (Pa)",
            stream.pressure_pa,
        ),
        inspector_number_field(
            drafts,
            rf_ui::stream_inspector_draft_key(
                &stream.id,
                &rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS,
            ),
            "Molar flow (mol/s)",
            stream.total_molar_flow_mol_s,
        ),
    ]
    .into_iter()
    .chain(
        stream
            .overall_mole_fractions
            .iter()
            .map(|(component_id, fraction)| {
                let mut field = inspector_number_field(
                    drafts,
                    rf_ui::stream_inspector_draft_key(
                        &stream.id,
                        &rf_ui::StreamInspectorDraftField::OverallMoleFraction(
                            component_id.clone(),
                        ),
                    ),
                    &format!("Overall mole fraction ({})", component_id.as_str()),
                    *fraction,
                );
                field.remove_command_id = (stream.overall_mole_fractions.len() > 1).then(|| {
                    crate::inspector_composition_component_remove_command_id(
                        stream.id.as_str(),
                        component_id.as_str(),
                    )
                });
                field
            }),
    )
    .collect()
}

fn stream_property_composition_component_actions(
    stream: &rf_model::MaterialStreamState,
    flowsheet: &rf_model::Flowsheet,
) -> Vec<crate::StudioGuiInspectorCompositionComponentActionSnapshot> {
    flowsheet
        .components
        .values()
        .filter(|component| !stream.overall_mole_fractions.contains_key(&component.id))
        .map(
            |component| crate::StudioGuiInspectorCompositionComponentActionSnapshot {
                component_id: component.id.as_str().to_string(),
                component_name: component.name.clone(),
                command_id: crate::inspector_composition_component_add_command_id(
                    stream.id.as_str(),
                    component.id.as_str(),
                ),
            },
        )
        .collect()
}

fn inspector_text_field(
    drafts: &rf_ui::InspectorDraftState,
    key: String,
    label: &str,
    original: String,
) -> StudioGuiInspectorTargetFieldSnapshot {
    match drafts.fields.get(&key) {
        Some(rf_ui::DraftValue::Text(draft)) => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            constraint_text: None,
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Text,
            original_value: draft.original.clone(),
            current_value: draft.current.clone(),
            is_dirty: draft.is_dirty,
            validation: inspector_validation_from_ui(draft.validation),
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: inspector_commit_command_id_for_field(
                &key,
                draft.is_dirty,
                draft.validation,
            ),
            discard_command_id: inspector_discard_command_id_for_field(
                &key,
                draft.is_dirty,
                draft.validation,
            ),
            remove_command_id: None,
        },
        _ => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            constraint_text: None,
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Text,
            original_value: original.clone(),
            current_value: original,
            is_dirty: false,
            validation: StudioGuiInspectorTargetFieldValidationSnapshot::Unknown,
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: None,
            discard_command_id: None,
            remove_command_id: None,
        },
    }
}

fn inspector_number_field(
    drafts: &rf_ui::InspectorDraftState,
    key: String,
    label: &str,
    original: f64,
) -> StudioGuiInspectorTargetFieldSnapshot {
    match drafts.fields.get(&key) {
        Some(rf_ui::DraftValue::Number(draft)) => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            constraint_text: None,
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Number,
            original_value: draft.original.clone(),
            current_value: draft.current.clone(),
            is_dirty: draft.is_dirty,
            validation: inspector_validation_from_ui(draft.validation),
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: inspector_commit_command_id_for_field(
                &key,
                draft.is_dirty,
                draft.validation,
            ),
            discard_command_id: inspector_discard_command_id_for_field(
                &key,
                draft.is_dirty,
                draft.validation,
            ),
            remove_command_id: None,
        },
        _ => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            constraint_text: None,
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Number,
            original_value: format_field_number(original),
            current_value: format_field_number(original),
            is_dirty: false,
            validation: StudioGuiInspectorTargetFieldValidationSnapshot::Unknown,
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: None,
            discard_command_id: None,
            remove_command_id: None,
        },
    }
}

fn inspector_validation_from_ui(
    validation: rf_ui::DraftValidationState,
) -> StudioGuiInspectorTargetFieldValidationSnapshot {
    match validation {
        rf_ui::DraftValidationState::Unknown => {
            StudioGuiInspectorTargetFieldValidationSnapshot::Unknown
        }
        rf_ui::DraftValidationState::Valid => {
            StudioGuiInspectorTargetFieldValidationSnapshot::Valid
        }
        rf_ui::DraftValidationState::Invalid => {
            StudioGuiInspectorTargetFieldValidationSnapshot::Invalid
        }
    }
}

fn inspector_commit_command_id_for_field(
    key: &str,
    is_dirty: bool,
    validation: rf_ui::DraftValidationState,
) -> Option<String> {
    (is_dirty && validation == rf_ui::DraftValidationState::Valid)
        .then(|| crate::inspector_draft_commit_command_id(key))
}

fn inspector_discard_command_id_for_field(
    key: &str,
    is_dirty: bool,
    validation: rf_ui::DraftValidationState,
) -> Option<String> {
    (is_dirty || validation == rf_ui::DraftValidationState::Invalid)
        .then(|| crate::inspector_draft_discard_command_id(key))
}

fn stream_property_batch_commit_command_id(
    stream: &rf_model::MaterialStreamState,
    fields: &[StudioGuiInspectorTargetFieldSnapshot],
) -> Option<String> {
    let committable_field_count = fields
        .iter()
        .filter(|field| field.commit_command_id.is_some())
        .count();
    (committable_field_count > 1)
        .then(|| crate::inspector_draft_batch_commit_command_id(stream.id.as_str()))
}

fn stream_property_batch_discard_command_id(
    stream: &rf_model::MaterialStreamState,
    fields: &[StudioGuiInspectorTargetFieldSnapshot],
) -> Option<String> {
    fields
        .iter()
        .any(|field| field.discard_command_id.is_some())
        .then(|| crate::inspector_draft_batch_discard_command_id(stream.id.as_str()))
}

fn inspector_property_notices(
    fields: &[StudioGuiInspectorTargetFieldSnapshot],
) -> Vec<crate::StudioGuiInspectorPropertyNoticeSnapshot> {
    if fields
        .iter()
        .any(|field| field.validation == StudioGuiInspectorTargetFieldValidationSnapshot::Invalid)
    {
        return vec![crate::StudioGuiInspectorPropertyNoticeSnapshot {
            status_label: "Invalid",
            message:
                "Fix invalid property drafts before applying changes; invalid drafts are preserved and are not committed."
                    .to_string(),
        }];
    }

    Vec::new()
}

fn unit_property_notices(
    unit: &rf_model::UnitNode,
    fields: &[StudioGuiInspectorTargetFieldSnapshot],
) -> Vec<crate::StudioGuiInspectorPropertyNoticeSnapshot> {
    let notices: Vec<_> = fields
        .iter()
        .filter(|field| {
            field.validation == StudioGuiInspectorTargetFieldValidationSnapshot::Invalid
        })
        .map(|field| crate::StudioGuiInspectorPropertyNoticeSnapshot {
            status_label: "Invalid",
            message: unit_parameter_invalid_notice(unit, field),
        })
        .collect();

    if notices.is_empty() {
        inspector_property_notices(fields)
    } else {
        notices
    }
}

fn unit_parameter_invalid_notice(
    unit: &rf_model::UnitNode,
    field: &StudioGuiInspectorTargetFieldSnapshot,
) -> String {
    if field.key.ends_with(":outlet_temperature_k") {
        return format!(
            "{} must be a positive finite outlet temperature in K.",
            field.label
        );
    }

    if field.key.ends_with(":outlet_pressure_pa") {
        if unit_outlet_pressure_cannot_exceed_inlet(unit) {
            return format!(
                "{} must be a positive finite outlet pressure in Pa and cannot exceed connected inlet pressure.",
                field.label
            );
        }
        return format!(
            "{} must be a positive finite outlet pressure in Pa.",
            field.label
        );
    }

    "Fix invalid property drafts before applying changes; invalid drafts are preserved and are not committed.".to_string()
}

fn unit_parameter_constraint_text(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    field: &rf_ui::UnitInspectorDraftField,
) -> String {
    match field {
        rf_ui::UnitInspectorDraftField::OutletTemperatureK => {
            if unit.kind == "feed" {
                return "Unit K; positive finite source outlet temperature; commit syncs the Feed outlet template.".to_string();
            }
            if unit.kind == "flash_drum" {
                return "Unit K; positive finite flash temperature; commit syncs liquid/vapor outlet templates.".to_string();
            }
            "Unit K; positive finite outlet temperature; commit syncs the outlet stream template."
                .to_string()
        }
        rf_ui::UnitInspectorDraftField::OutletPressurePa => {
            if unit_outlet_pressure_cannot_exceed_inlet(unit) {
                let inlet_limit = connected_inlet_pressure_limit(flowsheet, unit)
                    .map(|pressure_pa| format!(" Inlet limit: {pressure_pa:.0} Pa."));
                return format!(
                    "Unit Pa; positive finite outlet pressure; cannot exceed connected inlet pressure.{}",
                    inlet_limit.unwrap_or_default()
                );
            }
            if unit.kind == "feed" {
                return "Unit Pa; positive finite source outlet pressure; commit syncs the Feed outlet template.".to_string();
            }
            "Unit Pa; positive finite flash pressure; commit syncs liquid/vapor outlet templates."
                .to_string()
        }
    }
}

fn unit_outlet_pressure_cannot_exceed_inlet(unit: &rf_model::UnitNode) -> bool {
    matches!(unit.kind.as_str(), "mixer" | "heater" | "cooler" | "valve")
}

fn connected_inlet_pressure_limit(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
) -> Option<f64> {
    unit.ports
        .iter()
        .filter(|port| {
            port.direction == rf_types::PortDirection::Inlet
                && port.kind == rf_types::PortKind::Material
        })
        .filter_map(|port| port.stream_id.as_ref())
        .filter_map(|stream_id| flowsheet.streams.get(stream_id))
        .map(|stream| stream.pressure_pa)
        .reduce(f64::min)
}

fn stream_property_notices(
    stream: &rf_model::MaterialStreamState,
    drafts: &rf_ui::InspectorDraftState,
    fields: &[StudioGuiInspectorTargetFieldSnapshot],
) -> Vec<crate::StudioGuiInspectorPropertyNoticeSnapshot> {
    let mut notices = if fields
        .iter()
        .any(|field| field.validation == StudioGuiInspectorTargetFieldValidationSnapshot::Invalid)
    {
        vec![crate::StudioGuiInspectorPropertyNoticeSnapshot {
            status_label: "Invalid",
            message:
                "Fix invalid stream property drafts before applying changes; invalid drafts are preserved and are not committed."
                    .to_string(),
        }]
    } else {
        Vec::new()
    };

    if let Some(sum) = stream_property_composition_sum(stream, drafts) {
        if !sum.is_finite() || sum <= 0.0 {
            notices.push(crate::StudioGuiInspectorPropertyNoticeSnapshot {
                status_label: "Invalid",
                message: "Overall mole fraction sum must be positive and finite before it can be normalized.".to_string(),
            });
        } else if (sum - 1.0).abs() > 1e-9 {
            let status_label = if fields
                .iter()
                .any(|field| field.key.contains(":overall_mole_fraction:") && field.is_dirty)
            {
                "Draft"
            } else {
                "Unnormalized"
            };
            notices.push(crate::StudioGuiInspectorPropertyNoticeSnapshot {
                status_label,
                message: format!(
                    "Overall mole fraction sum is {sum:.6}, not 1.000000. Use Normalize composition or adjust the draft values explicitly; no automatic compensation is applied."
                ),
            });
        }
    }

    notices
}

fn stream_property_composition_normalize_command_id(
    stream: &rf_model::MaterialStreamState,
    drafts: &rf_ui::InspectorDraftState,
) -> Option<String> {
    if stream.overall_mole_fractions.is_empty() {
        return None;
    }

    let mut has_dirty_composition = false;
    let mut sum = 0.0;
    for (component_id, original_value) in &stream.overall_mole_fractions {
        let key = rf_ui::stream_inspector_draft_key(
            &stream.id,
            &rf_ui::StreamInspectorDraftField::OverallMoleFraction(component_id.clone()),
        );
        let value = match drafts.fields.get(&key) {
            Some(rf_ui::DraftValue::Number(draft))
                if draft.is_dirty && draft.validation == rf_ui::DraftValidationState::Valid =>
            {
                has_dirty_composition = true;
                match draft.current.trim().parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => return None,
                }
            }
            Some(rf_ui::DraftValue::Number(draft))
                if draft.validation == rf_ui::DraftValidationState::Invalid =>
            {
                return None;
            }
            Some(_) => return None,
            None => *original_value,
        };
        if !value.is_finite() || value < 0.0 {
            return None;
        }
        sum += value;
    }

    (sum.is_finite() && sum > 0.0 && (has_dirty_composition || (sum - 1.0).abs() > 1e-9))
        .then(|| crate::inspector_composition_normalize_command_id(stream.id.as_str()))
}

fn stream_property_composition_sum(
    stream: &rf_model::MaterialStreamState,
    drafts: &rf_ui::InspectorDraftState,
) -> Option<f64> {
    if stream.overall_mole_fractions.is_empty() {
        return None;
    }

    let mut sum = 0.0;
    for (component_id, original_value) in &stream.overall_mole_fractions {
        let key = rf_ui::stream_inspector_draft_key(
            &stream.id,
            &rf_ui::StreamInspectorDraftField::OverallMoleFraction(component_id.clone()),
        );
        let value = match drafts.fields.get(&key) {
            Some(rf_ui::DraftValue::Number(draft))
                if draft.validation == rf_ui::DraftValidationState::Valid =>
            {
                draft.current.trim().parse::<f64>().ok()?
            }
            Some(rf_ui::DraftValue::Number(draft))
                if draft.validation == rf_ui::DraftValidationState::Unknown =>
            {
                *original_value
            }
            Some(_) => return None,
            None => *original_value,
        };
        sum += value;
    }

    Some(sum)
}

fn stream_property_composition_summary(
    stream: &rf_model::MaterialStreamState,
    drafts: &rf_ui::InspectorDraftState,
) -> Option<StudioGuiInspectorCompositionSummarySnapshot> {
    if stream.overall_mole_fractions.is_empty() {
        return None;
    }

    let mut values = Vec::new();
    let mut has_dirty_composition = false;
    let mut has_invalid_composition = false;
    for (component_id, original_value) in &stream.overall_mole_fractions {
        let key = rf_ui::stream_inspector_draft_key(
            &stream.id,
            &rf_ui::StreamInspectorDraftField::OverallMoleFraction(component_id.clone()),
        );
        let value = match drafts.fields.get(&key) {
            Some(rf_ui::DraftValue::Number(draft)) => match draft.validation {
                rf_ui::DraftValidationState::Valid => {
                    has_dirty_composition |= draft.is_dirty;
                    draft.current.trim().parse::<f64>().ok()
                }
                rf_ui::DraftValidationState::Invalid => {
                    has_invalid_composition = true;
                    None
                }
                rf_ui::DraftValidationState::Unknown => Some(*original_value),
            },
            Some(_) => {
                has_invalid_composition = true;
                None
            }
            None => Some(*original_value),
        };
        if let Some(value) = value {
            values.push((component_id.as_str().to_string(), value));
        }
    }

    if has_invalid_composition {
        return Some(StudioGuiInspectorCompositionSummarySnapshot {
            current_sum_text: "-".to_string(),
            normalized_preview_text: "Fix invalid composition drafts before normalizing."
                .to_string(),
            status_label: "Invalid",
        });
    }

    let sum = values.iter().map(|(_, value)| value).sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return Some(StudioGuiInspectorCompositionSummarySnapshot {
            current_sum_text: format_field_number(sum),
            normalized_preview_text: "Composition sum must be positive and finite.".to_string(),
            status_label: "Invalid",
        });
    }

    let normalized_preview_text = values
        .iter()
        .map(|(component_id, value)| format!("{component_id}={:.6}", value / sum))
        .collect::<Vec<_>>()
        .join(", ");
    let status_label = if has_dirty_composition {
        "Draft"
    } else if (sum - 1.0).abs() > 1e-9 {
        "Unnormalized"
    } else {
        "Synced"
    };

    Some(StudioGuiInspectorCompositionSummarySnapshot {
        current_sum_text: format!("{sum:.6}"),
        normalized_preview_text,
        status_label,
    })
}

fn format_field_number(value: f64) -> String {
    value.to_string()
}

fn workspace_document_snapshot_from_controller(
    controller: &crate::StudioAppHostController,
) -> crate::StudioGuiWorkspaceDocumentSnapshot {
    let document = controller.document();
    let selected_property_package_id = document.flowsheet.property_package_id();
    crate::StudioGuiWorkspaceDocumentSnapshot {
        document_id: document.metadata.document_id.as_str().to_string(),
        title: document.metadata.title.clone(),
        flowsheet_name: document.flowsheet.name.clone(),
        revision: document.revision,
        last_saved_revision: controller.document_last_saved_revision(),
        has_unsaved_changes: controller.document_has_unsaved_changes(),
        project_path: controller
            .document_path()
            .map(|path| path.display().to_string()),
        property_package_id: selected_property_package_id.map(|package_id| package_id.to_string()),
        property_package_choices: crate::STUDIO_BUILTIN_PROPERTY_PACKAGES
            .iter()
            .map(|package| crate::StudioGuiPropertyPackageChoiceSnapshot {
                package_id: package.package_id.to_string(),
                label: package.label.to_string(),
                detail: package.detail.to_string(),
                component_summary: package.component_summary.to_string(),
                command_id: crate::property_package_select_command_id(package.package_id),
                selected: selected_property_package_id == Some(package.package_id),
                enabled: true,
            })
            .collect(),
        project_component_choices: crate::STUDIO_BUILTIN_PROJECT_COMPONENTS
            .iter()
            .map(|component| {
                let component_id = rf_types::ComponentId::new(component.component_id);
                let selected = document.flowsheet.components.contains_key(&component_id);
                let referenced = flowsheet_component_is_referenced(&document.flowsheet, &component_id);
                crate::StudioGuiProjectComponentChoiceSnapshot {
                    component_id: component.component_id.to_string(),
                    name: component.name.to_string(),
                    formula: Some(component.formula.to_string()),
                    selected,
                    select_command_id: crate::project_component_select_command_id(
                        component.component_id,
                    ),
                    remove_command_id: crate::project_component_remove_command_id(
                        component.component_id,
                    ),
                    remove_enabled: selected && !referenced,
                    remove_detail: if referenced {
                        "Remove this component from stream compositions before removing it from the project."
                            .to_string()
                    } else {
                        "Remove this unused component from the current flowsheet.".to_string()
                    },
                }
            })
            .collect(),
        unit_count: document.flowsheet.units.len(),
        stream_count: document.flowsheet.streams.len(),
        snapshot_history_count: controller.snapshot_history_count(),
    }
}

fn flowsheet_component_is_referenced(
    flowsheet: &rf_model::Flowsheet,
    component_id: &rf_types::ComponentId,
) -> bool {
    flowsheet
        .streams
        .values()
        .any(|stream| stream.overall_mole_fractions.contains_key(component_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_units_default_to_material_process_order_for_heater_flash_example() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected example project parse");

        let unit_order = canvas_units_in_layout_order(&project.document.flowsheet)
            .into_iter()
            .map(|unit| unit.id.as_str().to_string())
            .collect::<Vec<_>>();

        assert_eq!(unit_order, ["feed-1", "heater-1", "flash-1"]);
    }
}
