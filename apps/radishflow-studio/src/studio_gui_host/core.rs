use super::helpers::{
    dispatch_from_controller, global_event_from_controller, ui_commands_from_projection,
};
use super::*;

impl StudioGuiHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let controller = StudioAppHostController::new(config)?;
        let layout_state_overrides = match controller.document_path() {
            Some(project_path) => load_persisted_window_layouts(project_path)?,
            None => BTreeMap::new(),
        };
        let canvas_unit_positions = match controller.document_path() {
            Some(project_path) => load_persisted_canvas_unit_positions(project_path)?,
            None => BTreeMap::new(),
        };

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
        let units = self
            .controller
            .document()
            .flowsheet
            .units
            .values()
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
            control_state.notice.as_ref(),
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
        StudioGuiCommandRegistry::from_surfaces(
            &self.ui_commands(),
            &self.canvas_state(),
            self.preferred_target_window_id(),
        )
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        let mut snapshot = StudioGuiSnapshot::new(
            self.state().clone(),
            self.ui_commands(),
            self.command_registry(),
            self.canvas_state().widget(),
            StudioGuiRuntimeSnapshot {
                workspace_document: workspace_document_snapshot_from_controller(&self.controller),
                example_projects: crate::studio_example_project_models(
                    self.controller.document_path(),
                ),
                control_state: self.controller.workspace_control_state(),
                run_panel: self.controller.run_panel_widget(),
                latest_solve_snapshot: self.controller.latest_solve_snapshot(),
                active_inspector_target: self.controller.active_inspector_target(),
                active_inspector_detail: active_inspector_detail_from_controller(&self.controller),
                entitlement_host: self.controller.entitlement_host_output(),
                platform_notice: None,
                platform_timer_lines: Vec::new(),
                gui_activity_lines: Vec::new(),
                log_entries: self.controller.log_entries(),
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
            StudioGuiHostCommand::DispatchInspectorDraftBatchCommit { command_id } => self
                .dispatch_inspector_draft_batch_commit(&command_id)
                .map(StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted),
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

fn canvas_diagnostics_from_runtime(
    latest_solve_snapshot: Option<&rf_ui::SolveSnapshot>,
    notice: Option<&rf_ui::RunPanelNotice>,
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

fn active_inspector_detail_from_controller(
    controller: &StudioAppHostController,
) -> Option<StudioGuiInspectorTargetDetailSnapshot> {
    let target = controller.active_inspector_target()?;
    let flowsheet = &controller.document().flowsheet;

    match &target {
        rf_ui::InspectorTarget::Unit(unit_id) => {
            let unit = flowsheet.units.get(unit_id)?;
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
                property_fields: Vec::new(),
                property_batch_commit_command_id: None,
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
                property_fields,
                unit_ports: Vec::new(),
            })
        }
    }
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
                inspector_number_field(
                    drafts,
                    rf_ui::stream_inspector_draft_key(
                        &stream.id,
                        &rf_ui::StreamInspectorDraftField::OverallMoleFraction(
                            component_id.clone(),
                        ),
                    ),
                    &format!("Overall mole fraction ({})", component_id.as_str()),
                    *fraction,
                )
            }),
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
        },
        _ => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Text,
            original_value: original.clone(),
            current_value: original,
            is_dirty: false,
            validation: StudioGuiInspectorTargetFieldValidationSnapshot::Unknown,
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: None,
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
        },
        _ => StudioGuiInspectorTargetFieldSnapshot {
            key: key.clone(),
            label: label.to_string(),
            value_kind: StudioGuiInspectorTargetFieldValueKindSnapshot::Number,
            original_value: format_field_number(original),
            current_value: format_field_number(original),
            is_dirty: false,
            validation: StudioGuiInspectorTargetFieldValidationSnapshot::Unknown,
            draft_update_command_id: crate::inspector_draft_update_command_id(&key),
            commit_command_id: None,
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

fn format_field_number(value: f64) -> String {
    value.to_string()
}

fn workspace_document_snapshot_from_controller(
    controller: &crate::StudioAppHostController,
) -> crate::StudioGuiWorkspaceDocumentSnapshot {
    let document = controller.document();
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
        unit_count: document.flowsheet.units.len(),
        stream_count: document.flowsheet.streams.len(),
        snapshot_history_count: controller.snapshot_history_count(),
    }
}
