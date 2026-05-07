use super::*;

#[test]
fn committing_invalid_stream_inspector_draft_is_ignored_without_document_mutation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    let update = app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected draft update");

    let outcome = app_state
        .commit_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            timestamp(42),
        )
        .expect("expected ignored invalid commit");

    assert_eq!(outcome, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].pressure_pa,
        101_325.0
    );
    assert!(app_state.workspace.drafts.fields.contains_key(&update.key));
}

#[test]
fn applying_run_panel_recovery_action_disconnects_port_and_opens_unit_inspector() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/missing-stream-reference.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-missing-stream-recovery",
            "Missing Stream Recovery Demo",
            timestamp(40),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.missing_stream_reference: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.missing_stream_reference")
    .with_related_unit_ids(vec![UnitId::new("heater-1")])
    .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
        "heater-1",
        "outlet",
    )]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(crate::RunPanelRecoveryMutation::DisconnectPort {
            unit_id: UnitId::new("heater-1"),
            port_name: "outlet".to_string(),
        })
    );

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
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: UnitId::new("heater-1"),
            port: "outlet".to_string(),
        })
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("heater-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "outlet"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        None
    );
}

#[test]
fn applying_run_panel_recovery_action_deletes_orphan_stream_without_selecting_missing_target() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/orphan-stream.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-orphan-stream-recovery",
            "Orphan Stream Recovery Demo",
            timestamp(41),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.orphan_stream: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.orphan_stream")
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-orphan")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(crate::RunPanelRecoveryMutation::DeleteStream {
            stream_id: rf_types::StreamId::new("stream-orphan"),
        })
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(applied_target, None);
    assert!(app_state.workspace.selection.selected_units.is_empty());
    assert!(app_state.workspace.selection.selected_streams.is_empty());
    assert_eq!(app_state.workspace.drafts.active_target, None);
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DeleteStream {
            stream_id: rf_types::StreamId::new("stream-orphan"),
        })
    );
    assert!(
        !app_state
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(&rf_types::StreamId::new("stream-orphan"))
    );
}

#[test]
fn applying_run_panel_recovery_action_creates_stream_for_unbound_outlet_and_opens_unit_inspector() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-unbound-outlet-recovery",
            "Unbound Outlet Recovery Demo",
            timestamp(44),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.unbound_outlet_port: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.unbound_outlet_port")
    .with_related_unit_ids(vec![UnitId::new("feed-1")])
    .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
        "feed-1", "outlet",
    )]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(crate::RunPanelRecoveryMutation::CreateAndBindOutletStream {
            unit_id: UnitId::new("feed-1"),
            port_name: "outlet".to_string(),
        })
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::ConnectPorts {
            stream_id: rf_types::StreamId::new("stream-feed-1-outlet"),
            from_unit_id: UnitId::new("feed-1"),
            from_port: "outlet".to_string(),
            to_unit_id: None,
            to_port: None,
        })
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("feed-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "outlet"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        Some("stream-feed-1-outlet")
    );
    assert!(
        app_state
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(&rf_types::StreamId::new("stream-feed-1-outlet"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn applying_run_panel_recovery_action_disconnects_missing_upstream_source_and_deletes_stream() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-missing-upstream-recovery",
            "Missing Upstream Recovery Demo",
            timestamp(45),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.missing_upstream_source: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.missing_upstream_source")
    .with_related_unit_ids(vec![UnitId::new("mixer-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-feed-a")])
    .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
        "mixer-1", "inlet_a",
    )]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(
            crate::RunPanelRecoveryMutation::DisconnectPortAndDeleteStream {
                unit_id: UnitId::new("mixer-1"),
                port_name: "inlet_a".to_string(),
                stream_id: rf_types::StreamId::new("stream-feed-a"),
            }
        )
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("mixer-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPortAndDeleteStream {
            unit_id: UnitId::new("mixer-1"),
            port: "inlet_a".to_string(),
            stream_id: rf_types::StreamId::new("stream-feed-a"),
        })
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("mixer-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet_a"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        None
    );
    assert!(
        !app_state
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(&rf_types::StreamId::new("stream-feed-a"))
    );
}

#[test]
fn applying_run_panel_recovery_action_disconnects_self_loop_inlet_and_opens_unit_inspector() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-self-loop-recovery",
            "Self Loop Recovery Demo",
            timestamp(42),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.topological_ordering.self_loop_cycle: solver topological ordering failed",
    )
    .with_primary_code("solver.topological_ordering.self_loop_cycle")
    .with_related_unit_ids(vec![UnitId::new("flash-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-loop")])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
        rf_types::DiagnosticPortTarget::new("flash-1", "liquid"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(crate::RunPanelRecoveryMutation::DisconnectPort {
            unit_id: UnitId::new("flash-1"),
            port_name: "inlet".to_string(),
        })
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: UnitId::new("flash-1"),
            port: "inlet".to_string(),
        })
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("flash-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        None
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn applying_run_panel_recovery_action_disconnects_two_unit_cycle_inlet_and_opens_unit_inspector() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-two-unit-cycle-recovery",
            "Two Unit Cycle Recovery Demo",
            timestamp(43),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.topological_ordering.two_unit_cycle: solver topological ordering failed",
    )
    .with_primary_code("solver.topological_ordering.two_unit_cycle")
    .with_related_unit_ids(vec![UnitId::new("heater-1"), UnitId::new("valve-1")])
    .with_related_stream_ids(vec![
        rf_types::StreamId::new("stream-a"),
        rf_types::StreamId::new("stream-b"),
    ])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("valve-1", "inlet"),
        rf_types::DiagnosticPortTarget::new("heater-1", "inlet"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(crate::RunPanelRecoveryMutation::DisconnectPort {
            unit_id: UnitId::new("heater-1"),
            port_name: "inlet".to_string(),
        })
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: UnitId::new("heater-1"),
            port: "inlet".to_string(),
        })
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("heater-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        None
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}
