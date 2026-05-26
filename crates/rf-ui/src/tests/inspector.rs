use super::*;

#[test]
fn applying_run_panel_recovery_action_selects_unit_and_opens_inspector() {
    let mut document = sample_document();
    document
        .flowsheet
        .insert_unit(UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![
                UnitPort::new("inlet", PortDirection::Inlet, PortKind::Material, None),
                UnitPort::new("outlet", PortDirection::Outlet, PortKind::Material, None),
            ],
        ))
        .expect("expected heater insert");
    let mut app_state = AppState::new(document);
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.step.spec: solver step 1 unit spec validation failed",
    )
    .with_primary_code("solver.step.spec")
    .with_related_unit_ids(vec![UnitId::new("heater-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("heater-1"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn focusing_inspector_target_selects_unit_without_document_mutation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);

    let applied_target =
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("feed-1"))
    );
    assert!(app_state.workspace.selection.selected_streams.is_empty());
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn focusing_inspector_target_selects_stream_and_clears_previous_unit() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let applied_target = app_state
        .focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
    );
    assert!(app_state.workspace.selection.selected_units.is_empty());
    assert!(
        app_state
            .workspace
            .selection
            .selected_streams
            .contains(&StreamId::new("stream-feed"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
    );
}

#[test]
fn adding_flowsheet_component_records_document_command() {
    let document = sample_document();
    let mut app_state = AppState::new(document);
    let component = rf_model::Component::new("component-c", "Component C").with_formula("C");

    let revision = app_state
        .add_flowsheet_component(component, timestamp(42))
        .expect("expected component add")
        .expect("expected applied component add");

    assert_eq!(revision, 1);
    assert!(
        app_state
            .workspace
            .document
            .flowsheet
            .components
            .contains_key(&ComponentId::new("component-c"))
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::AddComponent {
            component_id: ComponentId::new("component-c"),
            name: "Component C".to_string(),
            formula: Some("C".to_string()),
        })
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn removing_flowsheet_component_rejects_stream_composition_references() {
    let mut document = inspector_focus_document();
    document
        .flowsheet
        .insert_component(rf_model::Component::new("component-a", "Component A"))
        .expect("expected component-a insert");
    let mut app_state = AppState::new(document);

    let error = app_state
        .remove_flowsheet_component(ComponentId::new("component-a"), timestamp(42))
        .expect_err("expected referenced component removal to fail");

    assert!(
        error
            .to_string()
            .contains("component `component-a` is still referenced by stream `stream-feed`")
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
}

#[test]
fn removing_unused_flowsheet_component_records_document_command() {
    let mut document = sample_document();
    document
        .flowsheet
        .insert_component(rf_model::Component::new("component-c", "Component C"))
        .expect("expected component-c insert");
    let mut app_state = AppState::new(document);

    let revision = app_state
        .remove_flowsheet_component(ComponentId::new("component-c"), timestamp(42))
        .expect("expected component remove")
        .expect("expected applied component remove");

    assert_eq!(revision, 1);
    assert!(
        !app_state
            .workspace
            .document
            .flowsheet
            .components
            .contains_key(&ComponentId::new("component-c"))
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::RemoveComponent {
            component_id: ComponentId::new("component-c"),
        })
    );
}

#[test]
fn focusing_missing_inspector_target_keeps_current_focus() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let applied_target =
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("missing-unit")));

    assert_eq!(applied_target, None);
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("feed-1"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
}

fn unit_parameter_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed",
            "Feed",
            300.0,
            120_000.0,
            5.0,
            Default::default(),
        ))
        .expect("expected feed stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-heated",
            "Heated Outlet",
            345.0,
            95_000.0,
            0.0,
            Default::default(),
        ))
        .expect("expected outlet stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![
                UnitPort::new(
                    "inlet",
                    PortDirection::Inlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                ),
                UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-heated".into()),
                ),
            ],
        ))
        .expect("expected heater insert");

    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-unit-parameter", "Unit Parameter Demo", timestamp(10)),
    )
}

fn feed_parameter_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("feed-parameter-demo");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed",
            "Feed",
            300.0,
            120_000.0,
            5.0,
            Default::default(),
        ))
        .expect("expected feed stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![UnitPort::new(
                "outlet",
                PortDirection::Outlet,
                PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");

    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-feed-parameter", "Feed Parameter Demo", timestamp(10)),
    )
}

fn valve_parameter_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("valve-demo");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed",
            "Feed",
            315.0,
            120_000.0,
            5.0,
            Default::default(),
        ))
        .expect("expected feed stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-throttled",
            "Valve Outlet",
            315.0,
            90_000.0,
            0.0,
            Default::default(),
        ))
        .expect("expected outlet stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "valve-1",
            "Valve",
            "valve",
            vec![
                UnitPort::new(
                    "inlet",
                    PortDirection::Inlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                ),
                UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-throttled".into()),
                ),
            ],
        ))
        .expect("expected valve insert");

    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-valve-parameter", "Valve Parameter Demo", timestamp(10)),
    )
}

fn mixer_parameter_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("mixer-demo");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed-a",
            "Feed A",
            315.0,
            120_000.0,
            2.0,
            Default::default(),
        ))
        .expect("expected feed a stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed-b",
            "Feed B",
            325.0,
            100_000.0,
            3.0,
            Default::default(),
        ))
        .expect("expected feed b stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-mixed",
            "Mixer Outlet",
            321.0,
            100_000.0,
            0.0,
            Default::default(),
        ))
        .expect("expected mixer outlet stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "mixer-1",
            "Mixer",
            "mixer",
            vec![
                UnitPort::new(
                    "inlet_a",
                    PortDirection::Inlet,
                    PortKind::Material,
                    Some("stream-feed-a".into()),
                ),
                UnitPort::new(
                    "inlet_b",
                    PortDirection::Inlet,
                    PortKind::Material,
                    Some("stream-feed-b".into()),
                ),
                UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-mixed".into()),
                ),
            ],
        ))
        .expect("expected mixer insert");

    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-mixer-parameter", "Mixer Parameter Demo", timestamp(10)),
    )
}

fn flash_parameter_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("flash-demo");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed",
            "Feed",
            345.0,
            95_000.0,
            5.0,
            Default::default(),
        ))
        .expect("expected feed stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-liquid",
            "Liquid Outlet",
            345.0,
            95_000.0,
            0.0,
            Default::default(),
        ))
        .expect("expected liquid stream insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-vapor",
            "Vapor Outlet",
            345.0,
            95_000.0,
            0.0,
            Default::default(),
        ))
        .expect("expected vapor stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            vec![
                UnitPort::new(
                    "inlet",
                    PortDirection::Inlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                ),
                UnitPort::new(
                    "liquid",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-liquid".into()),
                ),
                UnitPort::new(
                    "vapor",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-vapor".into()),
                ),
            ],
        ))
        .expect("expected flash drum insert");

    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-flash-parameter", "Flash Parameter Demo", timestamp(10)),
    )
}

#[test]
fn updating_unit_inspector_draft_keeps_document_unchanged() {
    let mut app_state = AppState::new(unit_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("heater-1")));

    let outcome = app_state
        .update_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            "360.0",
        )
        .expect("expected unit draft update");

    assert_eq!(outcome.key, "unit:heater-1:outlet_temperature_k");
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Valid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("heater-1")]
            .parameters
            .outlet_temperature_k,
        None
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-heated")]
            .temperature_k,
        345.0
    );
}

#[test]
fn committing_unit_inspector_draft_sets_parameter_and_syncs_outlet_template() {
    let mut app_state = AppState::new(unit_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("heater-1")));
    app_state
        .update_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            "360.0",
        )
        .expect("expected unit draft update");

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            timestamp(42),
        )
        .expect("expected unit draft commit")
        .expect("expected committed unit draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("heater-1"),
            parameter: "outlet_temperature_k".to_string(),
            value: CommandValue::Number(360.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("heater-1")]
            .parameters
            .outlet_temperature_k,
        Some(360.0)
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-heated")]
            .temperature_k,
        360.0
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn committing_heater_pressure_parameter_syncs_outlet_template() {
    let mut app_state = AppState::new(unit_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("heater-1")));
    app_state
        .update_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "90000",
        )
        .expect("expected heater pressure draft update");

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected heater pressure draft commit")
        .expect("expected committed heater pressure draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("heater-1"),
            parameter: "outlet_pressure_pa".to_string(),
            value: CommandValue::Number(90_000.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("heater-1")]
            .parameters
            .outlet_pressure_pa,
        Some(90_000.0)
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-heated")].pressure_pa,
        90_000.0
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn committing_mixer_pressure_parameter_syncs_outlet_template() {
    let mut app_state = AppState::new(mixer_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("mixer-1")));
    app_state
        .update_unit_inspector_draft(
            &UnitId::new("mixer-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "95000",
        )
        .expect("expected mixer pressure draft update");

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("mixer-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected mixer pressure draft commit")
        .expect("expected committed mixer pressure draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("mixer-1"),
            parameter: "outlet_pressure_pa".to_string(),
            value: CommandValue::Number(95_000.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("mixer-1")]
            .parameters
            .outlet_pressure_pa,
        Some(95_000.0)
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-mixed")].pressure_pa,
        95_000.0
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn updating_mixer_pressure_above_lowest_inlet_marks_draft_invalid() {
    let mut app_state = AppState::new(mixer_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("mixer-1")));

    let outcome = app_state
        .update_unit_inspector_draft(
            &UnitId::new("mixer-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "105000",
        )
        .expect("expected mixer pressure draft update");

    assert_eq!(outcome.key, "unit:mixer-1:outlet_pressure_pa");
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("mixer-1")]
            .parameters
            .outlet_pressure_pa,
        None
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-mixed")].pressure_pa,
        100_000.0
    );

    let ignored = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("mixer-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected invalid commit to be ignored");

    assert_eq!(ignored, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.drafts.fields.contains_key(&outcome.key));
}

#[test]
fn updating_heater_pressure_above_inlet_marks_draft_invalid() {
    let mut app_state = AppState::new(unit_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("heater-1")));

    let outcome = app_state
        .update_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "130000",
        )
        .expect("expected heater pressure draft update");

    assert_eq!(outcome.key, "unit:heater-1:outlet_pressure_pa");
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("heater-1")]
            .parameters
            .outlet_pressure_pa,
        None
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-heated")].pressure_pa,
        95_000.0
    );

    let ignored = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("heater-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected invalid commit to be ignored");

    assert_eq!(ignored, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.drafts.fields.contains_key(&outcome.key));
}

#[test]
fn committing_flash_pressure_parameter_syncs_both_outlet_templates() {
    let mut app_state = AppState::new(flash_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("flash-1")));
    app_state
        .update_unit_inspector_draft(
            &UnitId::new("flash-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "88000",
        )
        .expect("expected flash pressure draft update");

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("flash-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected flash pressure draft commit")
        .expect("expected committed flash pressure draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("flash-1"),
            parameter: "outlet_pressure_pa".to_string(),
            value: CommandValue::Number(88_000.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("flash-1")]
            .parameters
            .outlet_pressure_pa,
        Some(88_000.0)
    );
    for stream_id in ["stream-liquid", "stream-vapor"] {
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new(stream_id)].pressure_pa,
            88_000.0
        );
    }
}

#[test]
fn committing_flash_temperature_parameter_syncs_both_outlet_templates() {
    let mut app_state = AppState::new(flash_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("flash-1")));
    let update = app_state
        .update_unit_inspector_draft(
            &UnitId::new("flash-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            "335",
        )
        .expect("expected flash temperature draft update");

    assert_eq!(update.key, "unit:flash-1:outlet_temperature_k");
    assert!(update.is_dirty);
    assert_eq!(update.validation, crate::DraftValidationState::Valid);

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("flash-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            timestamp(42),
        )
        .expect("expected flash temperature draft commit")
        .expect("expected committed flash temperature draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("flash-1"),
            parameter: "outlet_temperature_k".to_string(),
            value: CommandValue::Number(335.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("flash-1")]
            .parameters
            .outlet_temperature_k,
        Some(335.0)
    );
    for stream_id in ["stream-liquid", "stream-vapor"] {
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new(stream_id)].temperature_k,
            335.0
        );
    }
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn committing_feed_temperature_parameter_syncs_source_stream_template() {
    let mut app_state = AppState::new(feed_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));
    let update = app_state
        .update_unit_inspector_draft(
            &UnitId::new("feed-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            "310",
        )
        .expect("expected feed temperature draft update");

    assert_eq!(update.key, "unit:feed-1:outlet_temperature_k");
    assert!(update.is_dirty);
    assert_eq!(update.validation, crate::DraftValidationState::Valid);

    let outcome = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("feed-1"),
            crate::UnitInspectorDraftField::OutletTemperatureK,
            timestamp(42),
        )
        .expect("expected feed temperature draft commit")
        .expect("expected committed feed temperature draft");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetUnitParameter {
            unit_id: UnitId::new("feed-1"),
            parameter: "outlet_temperature_k".to_string(),
            value: CommandValue::Number(310.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("feed-1")]
            .parameters
            .outlet_temperature_k,
        Some(310.0)
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")].temperature_k,
        310.0
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn updating_valve_parameter_above_inlet_pressure_marks_draft_invalid() {
    let mut app_state = AppState::new(valve_parameter_document());
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("valve-1")));

    let outcome = app_state
        .update_unit_inspector_draft(
            &UnitId::new("valve-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            "130000",
        )
        .expect("expected valve draft update");

    assert_eq!(outcome.key, "unit:valve-1:outlet_pressure_pa");
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.units[&UnitId::new("valve-1")]
            .parameters
            .outlet_pressure_pa,
        None
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-throttled")]
            .pressure_pa,
        90_000.0
    );

    let ignored = app_state
        .commit_unit_inspector_draft(
            &UnitId::new("valve-1"),
            crate::UnitInspectorDraftField::OutletPressurePa,
            timestamp(42),
        )
        .expect("expected invalid commit to be ignored");

    assert_eq!(ignored, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.drafts.fields.contains_key(&outcome.key));
}

#[test]
fn updating_stream_inspector_draft_keeps_document_unchanged() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    let outcome = app_state
        .update_stream_inspector_draft(
            &StreamId::new("stream-feed"),
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");

    assert_eq!(
        outcome.key,
        crate::stream_inspector_draft_key(
            &StreamId::new("stream-feed"),
            &crate::StreamInspectorDraftField::TemperatureK,
        )
    );
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Valid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")].temperature_k,
        298.15
    );
    assert_eq!(
        app_state.workspace.drafts.fields.get(&outcome.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "298.15".to_string(),
            current: "333.5".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Valid,
        }))
    );
}

#[test]
fn updating_stream_inspector_draft_preserves_invalid_raw_number() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    let outcome = app_state
        .update_stream_inspector_draft(
            &StreamId::new("stream-feed"),
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected draft update");

    assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
    assert_eq!(
        app_state.workspace.drafts.fields.get(&outcome.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "101325".to_string(),
            current: "not-a-pressure".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Invalid,
        }))
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
}

#[test]
fn updating_stream_inspector_draft_requires_active_stream_target() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let outcome = app_state.update_stream_inspector_draft(
        &StreamId::new("stream-feed"),
        crate::StreamInspectorDraftField::Name,
        "Edited stream",
    );

    assert_eq!(outcome, None);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(app_state.workspace.document.revision, 0);
}

#[test]
fn committing_stream_inspector_draft_writes_document_command_and_preserves_focus() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");

    let outcome = app_state
        .commit_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            timestamp(42),
        )
        .expect("expected draft commit")
        .expect("expected applied draft commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: "temperature_k".to_string(),
            value: CommandValue::Number(333.5),
        }
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&outcome.command)
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id.clone()))
    );
    assert!(!app_state.workspace.drafts.fields.contains_key(&outcome.key));
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Dirty);
}

#[test]
fn committing_stream_inspector_composition_draft_updates_overall_mole_fraction() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    let component_id = ComponentId::new("component-a");
    let field = crate::StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let update = app_state
        .update_stream_inspector_draft(&stream_id, field.clone(), "0.25")
        .expect("expected composition draft update");

    assert_eq!(
        update.key,
        "stream:stream-feed:overall_mole_fraction:component-a"
    );
    assert!(update.is_dirty);
    assert_eq!(update.validation, crate::DraftValidationState::Valid);
    assert_eq!(
        app_state.workspace.drafts.fields.get(&update.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "0.4".to_string(),
            current: "0.25".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Valid,
        }))
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
            [&component_id],
        0.4
    );

    let outcome = app_state
        .commit_stream_inspector_draft(&stream_id, field, timestamp(42))
        .expect("expected draft commit")
        .expect("expected applied composition draft commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: "overall_mole_fraction:component-a".to_string(),
            value: CommandValue::Number(0.25),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
            [&component_id],
        0.25
    );
    assert!(!app_state.workspace.drafts.fields.contains_key(&update.key));
}

#[test]
fn normalizing_stream_inspector_composition_drafts_commits_all_mole_fractions() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new("component-a")),
            "0.25",
        )
        .expect("expected component-a draft update");

    let outcome = app_state
        .normalize_stream_inspector_composition_drafts(&stream_id, timestamp(42))
        .expect("expected composition normalize")
        .expect("expected applied composition normalize");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:overall_mole_fraction:component-a".to_string(),
            "stream:stream-feed:overall_mole_fraction:component-b".to_string(),
        ]
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values: vec![
                crate::StreamSpecificationValue {
                    field: "overall_mole_fraction:component-a".to_string(),
                    value: CommandValue::Number(0.25 / 0.85),
                },
                crate::StreamSpecificationValue {
                    field: "overall_mole_fraction:component-b".to_string(),
                    value: CommandValue::Number(0.6 / 0.85),
                },
            ],
        }
    );
    let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
    let component_a = stream.overall_mole_fractions[&ComponentId::new("component-a")];
    let component_b = stream.overall_mole_fractions[&ComponentId::new("component-b")];
    assert_eq!(component_a, 0.25 / 0.85);
    assert_eq!(component_b, 0.6 / 0.85);
    assert!((component_a + component_b - 1.0).abs() <= 1e-12);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn normalizing_stream_inspector_composition_drafts_preserves_invalid_drafts() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    let update = app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new("component-a")),
            "not-a-fraction",
        )
        .expect("expected invalid component draft update");

    let outcome = app_state
        .normalize_stream_inspector_composition_drafts(&stream_id, timestamp(42))
        .expect("expected ignored composition normalize");

    assert_eq!(outcome, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert!(app_state.workspace.drafts.fields.contains_key(&update.key));
}

#[test]
fn adding_stream_inspector_composition_component_uses_flowsheet_component_list() {
    let mut document = inspector_focus_document();
    document
        .flowsheet
        .insert_component(rf_model::Component::new("component-c", "Component C"))
        .expect("expected component-c insert");
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    let component_id = ComponentId::new("component-c");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let outcome = app_state
        .add_stream_inspector_composition_component(&stream_id, component_id.clone(), timestamp(42))
        .expect("expected component add")
        .expect("expected applied component add");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.key,
        "stream:stream-feed:overall_mole_fraction:component-c"
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: "overall_mole_fraction:component-c".to_string(),
            value: CommandValue::Number(0.0),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
            [&component_id],
        0.0
    );
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn adding_stream_inspector_composition_component_rejects_unknown_or_existing_component() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let unknown = app_state
        .add_stream_inspector_composition_component(
            &stream_id,
            ComponentId::new("missing-component"),
            timestamp(42),
        )
        .expect("expected ignored unknown component");
    let existing = app_state
        .add_stream_inspector_composition_component(
            &stream_id,
            ComponentId::new("component-a"),
            timestamp(43),
        )
        .expect("expected ignored existing component");

    assert_eq!(unknown, None);
    assert_eq!(existing, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
}

#[test]
fn removing_stream_inspector_composition_component_deletes_explicit_entry_without_compensation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    let component_id = ComponentId::new("component-b");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let outcome = app_state
        .remove_stream_inspector_composition_component(
            &stream_id,
            component_id.clone(),
            timestamp(42),
        )
        .expect("expected component remove")
        .expect("expected applied component remove");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.key,
        "stream:stream-feed:overall_mole_fraction:component-b"
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::RemoveStreamCompositionComponent {
            stream_id: stream_id.clone(),
            component_id: component_id.clone(),
        }
    );
    let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
    assert!(!stream.overall_mole_fractions.contains_key(&component_id));
    assert_eq!(
        stream.overall_mole_fractions[&ComponentId::new("component-a")],
        0.4
    );
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn removing_stream_inspector_composition_component_rejects_last_or_missing_component() {
    let mut document = inspector_focus_document();
    let stream_id = StreamId::new("stream-feed");
    document
        .flowsheet
        .streams
        .get_mut(&stream_id)
        .expect("expected feed stream")
        .overall_mole_fractions
        .remove(&ComponentId::new("component-b"));
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let last = app_state
        .remove_stream_inspector_composition_component(
            &stream_id,
            ComponentId::new("component-a"),
            timestamp(42),
        )
        .expect("expected ignored last component");
    let missing = app_state
        .remove_stream_inspector_composition_component(
            &stream_id,
            ComponentId::new("missing-component"),
            timestamp(43),
        )
        .expect("expected ignored missing component");

    assert_eq!(last, None);
    assert_eq!(missing, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
}

#[test]
fn updating_stream_inspector_composition_draft_rejects_unknown_component() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let outcome = app_state.update_stream_inspector_draft(
        &stream_id,
        crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new(
            "missing-component",
        )),
        "0.25",
    );

    assert_eq!(outcome, None);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(app_state.workspace.document.revision, 0);
}

#[test]
fn committing_stream_inspector_drafts_records_one_batch_history_entry() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "202650",
        )
        .expect("expected pressure draft update");

    let outcome = app_state
        .commit_stream_inspector_drafts(&stream_id, timestamp(42))
        .expect("expected batch commit")
        .expect("expected applied batch commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:temperature_k".to_string(),
            "stream:stream-feed:pressure_pa".to_string()
        ]
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values: vec![
                crate::StreamSpecificationValue {
                    field: "temperature_k".to_string(),
                    value: CommandValue::Number(333.5),
                },
                crate::StreamSpecificationValue {
                    field: "pressure_pa".to_string(),
                    value: CommandValue::Number(202650.0),
                },
            ],
        }
    );
    let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
    assert_eq!(stream.temperature_k, 333.5);
    assert_eq!(stream.pressure_pa, 202650.0);
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&outcome.command)
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn batch_commit_preserves_invalid_stream_inspector_drafts() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected invalid pressure draft update");

    let outcome = app_state
        .commit_stream_inspector_drafts(&stream_id, timestamp(42))
        .expect("expected batch commit")
        .expect("expected applied batch commit");

    assert_eq!(outcome.keys, vec!["stream:stream-feed:temperature_k"]);
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].pressure_pa,
        101_325.0
    );
    assert!(
        app_state
            .workspace
            .drafts
            .fields
            .contains_key("stream:stream-feed:pressure_pa")
    );
}

#[test]
fn discarding_stream_inspector_draft_removes_field_without_document_mutation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    let update = app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");

    let outcome = app_state
        .discard_stream_inspector_draft(&stream_id, crate::StreamInspectorDraftField::TemperatureK)
        .expect("expected discarded draft");

    assert_eq!(outcome.key, update.key);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        298.15
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn discarding_stream_inspector_drafts_removes_valid_and_invalid_fields() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected invalid pressure draft update");

    let outcome = app_state
        .discard_stream_inspector_drafts(&stream_id)
        .expect("expected discarded drafts");

    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:temperature_k".to_string(),
            "stream:stream-feed:pressure_pa".to_string()
        ]
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn undo_redo_replays_stream_inspector_document_snapshots() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");
    app_state
        .commit_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            timestamp(42),
        )
        .expect("expected draft commit")
        .expect("expected applied draft commit");

    let undo = app_state
        .undo_document_command(timestamp(43))
        .expect("expected undo")
        .expect("expected undo result");

    assert_eq!(undo.direction, crate::DocumentHistoryDirection::Undo);
    assert_eq!(undo.revision, 2);
    assert_eq!(app_state.workspace.command_history.cursor, 0);
    assert!(app_state.workspace.command_history.can_redo());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        298.15
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id.clone()))
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );

    let redo = app_state
        .redo_document_command(timestamp(44))
        .expect("expected redo")
        .expect("expected redo result");

    assert_eq!(redo.direction, crate::DocumentHistoryDirection::Redo);
    assert_eq!(redo.revision, 3);
    assert_eq!(app_state.workspace.command_history.cursor, 1);
    assert!(!app_state.workspace.command_history.can_redo());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id))
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}
