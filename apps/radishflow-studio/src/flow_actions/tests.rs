use super::*;
use crate::modeling_actions::available_material_connections;
use rf_ui::variable_browser::{ObjectId, VariableField, VariableSection};
use rf_ui::{DocumentMetadata, FlowsheetDocument};
use std::time::UNIX_EPOCH;

fn app() -> AppState {
    AppState::new(FlowsheetDocument::new(
        rf_model::Flowsheet::new("Actions"),
        DocumentMetadata::new("same-document", "Actions", UNIX_EPOCH),
    ))
}
fn request(app: &AppState, action: FlowAction) -> FlowActionRequest {
    FlowActionRequest {
        expected: FlowEditContext::capture(&app.workspace),
        action,
    }
}
fn execute(app: &mut AppState, action: FlowAction) -> FlowActionReceipt {
    let request = request(app, action);
    execute_flow_action(app, FlowEditAccess::Editable, &request, UNIX_EPOCH).unwrap()
}
fn create(app: &mut AppState, kind: BuiltinUnitKind) -> UnitId {
    let FlowActionEffect::Created(id) = execute(app, FlowAction::CreateUnit(kind)).effect else {
        panic!("created identity")
    };
    id
}
fn write(app: &AppState, id: &UnitId, field: VariableField, value: VariableValue) -> FlowAction {
    FlowAction::Write {
        variable: VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object: ObjectId::Unit(id.clone()),
            section: VariableSection::Inputs,
            field,
        },
        value,
    }
}
fn renamed(app: &mut AppState, id: &UnitId, name: &str) {
    let action = write(
        app,
        id,
        VariableField::Name,
        VariableValue::Text(name.into()),
    );
    execute(app, action);
}
fn rejects(app: &mut AppState, request: &FlowActionRequest, expected: FlowActionError) {
    let before = app.clone();
    assert_eq!(
        preflight_flow_action(app, FlowEditAccess::Editable, request),
        Err(expected.clone())
    );
    assert_eq!(
        execute_flow_action(app, FlowEditAccess::Editable, request, UNIX_EPOCH),
        Err(expected)
    );
    assert_eq!(*app, before);
}
fn engineering(app: &mut AppState) {
    let rf_store::project_v3::ProjectV3Read::Editable(p) = rf_store::project_v3::parse(
        include_str!("../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!("fixture")
    };
    let e = p.engineering.unwrap();
    app.workspace.engineering = rf_ui::engineering::EngineeringSession::from_inputs(
        app.workspace.document.metadata.document_id.clone(),
        rf_ui::engineering::EngineeringInputs {
            basis: e.basis,
            tasks: e.tasks,
        },
    )
    .unwrap();
}

#[test]
fn all_unit_creations_match_existing_transactions_and_ids_are_not_reused() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::FlashDrum,
    ] {
        let mut app = app();
        let mut expected = app.clone();
        let legacy = expected.create_builtin_unit(kind, UNIX_EPOCH).unwrap();
        let receipt = execute(&mut app, FlowAction::CreateUnit(kind));
        assert_eq!(
            receipt.effect,
            FlowActionEffect::Created(legacy.unit_id.clone())
        );
        assert_eq!(app.workspace.document, expected.workspace.document);
        assert_eq!(
            app.workspace.command_history,
            expected.workspace.command_history
        );
        assert_eq!(receipt.outcome, FlowEditOutcome::Committed);
        assert_eq!(receipt.before.revision(), 0);
        assert_eq!(receipt.after.revision(), 1);
        app.delete_unit(&legacy.unit_id, UNIX_EPOCH).unwrap();
        assert_ne!(create(&mut app, kind), legacy.unit_id);
    }
}
#[test]
fn writes_preserve_legacy_validation_noop_and_history() {
    let mut app = app();
    let id = create(&mut app, BuiltinUnitKind::Heater);
    let action = write(
        &app,
        &id,
        VariableField::OutletTemperature,
        VariableValue::Number(330.0),
    );
    let receipt = execute(&mut app, action.clone());
    assert!(receipt.invalidates_flow_results());
    let before = app.clone();
    let receipt = execute(&mut app, action);
    assert_eq!(receipt.outcome, FlowEditOutcome::Unchanged);
    assert_eq!(receipt.command, None);
    assert!(!receipt.invalidates_flow_results());
    assert_eq!(receipt.before, receipt.after);
    assert_eq!(app, before);
    let req = request(
        &app,
        write(
            &app,
            &id,
            VariableField::OutletTemperature,
            VariableValue::Number(-1.0),
        ),
    );
    assert!(matches!(
        execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH),
        Err(FlowActionError::Write(VariableWriteError::Rejected(_)))
    ));
    assert_eq!(app, before);
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_ne!(
        app.workspace.document.flowsheet,
        before.workspace.document.flowsheet
    );
}
#[test]
fn input_guards_reject_foreign_session_revision_and_equal_revision_fork() {
    let mut left = app();
    let id = create(&mut left, BuiltinUnitKind::Heater);
    let original = request(
        &left,
        write(
            &left,
            &id,
            VariableField::Name,
            VariableValue::Text("same".into()),
        ),
    );
    let mut reopened = left.clone();
    reopened.workspace.reset_flow_run_session();
    rejects(
        &mut reopened,
        &original,
        FlowActionError::Context(FlowEditConflict::DifferentSession),
    );
    let mut foreign = left.clone();
    foreign.workspace.document.metadata.document_id = "other".into();
    rejects(
        &mut foreign,
        &original,
        FlowActionError::Context(FlowEditConflict::DifferentDocument),
    );
    let mut right = left.clone();
    renamed(&mut left, &id, "left");
    renamed(&mut right, &id, "right");
    rejects(
        &mut left,
        &original,
        FlowActionError::Context(FlowEditConflict::Revision {
            expected: 1,
            actual: 2,
        }),
    );
    let req = request(
        &left,
        write(
            &left,
            &id,
            VariableField::Name,
            VariableValue::Text("right".into()),
        ),
    );
    rejects(
        &mut right,
        &req,
        FlowActionError::Context(FlowEditConflict::InputChanged),
    );
}
#[test]
fn undo_never_revives_old_input_requests_even_when_values_return() {
    let mut app = app();
    let id = create(&mut app, BuiltinUnitKind::Heater);
    let req = request(&app, FlowAction::CreateUnit(BuiltinUnitKind::Feed));
    renamed(&mut app, &id, "new");
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    let revision = app.workspace.document.revision;
    rejects(
        &mut app,
        &req,
        FlowActionError::Context(FlowEditConflict::Revision {
            expected: 1,
            actual: revision,
        }),
    );
}
#[test]
fn independent_equipment_edit_and_cancel_do_not_revoke_flow_edit_context() {
    let mut app = app();
    engineering(&mut app);
    let req = request(&app, FlowAction::CreateUnit(BuiltinUnitKind::Heater));
    let mut source = app.workspace.engineering.inputs().basis.sources[0].clone();
    source.title = "changed".into();
    app.workspace
        .engineering
        .apply(vec![rf_ui::engineering::EngineeringCommand::UpdateSource(
            Box::new(source),
        )])
        .unwrap();
    app.workspace.revoke_flow_run_acceptance();
    let equipment = app.workspace.engineering.clone();
    execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH).unwrap();
    assert_eq!(app.workspace.engineering, equipment);
}
#[test]
fn preflight_does_not_reserve_identity_and_execution_rechecks_host_and_drafts() {
    let mut app = app();
    let req = request(&app, FlowAction::CreateUnit(BuiltinUnitKind::Feed));
    let before = app.clone();
    for _ in 0..3 {
        preflight_flow_action(&app, FlowEditAccess::Editable, &req).unwrap();
    }
    assert_eq!(app, before);
    assert_eq!(
        execute_flow_action(&mut app, FlowEditAccess::ReadOnly, &req, UNIX_EPOCH),
        Err(FlowActionError::ReadOnly)
    );
    assert_eq!(app, before);
    let expected = app
        .clone()
        .create_builtin_unit(BuiltinUnitKind::Feed, UNIX_EPOCH)
        .unwrap()
        .unit_id;
    let receipt =
        execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH).unwrap();
    assert_eq!(receipt.effect, FlowActionEffect::Created(expected.clone()));
    let req = request(&app, FlowAction::CreateUnit(BuiltinUnitKind::Heater));
    preflight_flow_action(&app, FlowEditAccess::Editable, &req).unwrap();
    app.focus_inspector_target(rf_ui::InspectorTarget::Unit(expected.clone()));
    app.update_unit_inspector_draft(&expected, rf_ui::UnitInspectorDraftField::Name, "pending");
    rejects(&mut app, &req, FlowActionError::PendingDrafts);
}
#[test]
fn pending_canvas_edit_is_preserved_and_blocks_formal_actions() {
    let mut app = app();
    let req = request(&app, FlowAction::CreateUnit(BuiltinUnitKind::Feed));
    app.begin_canvas_place_unit("heater");
    rejects(&mut app, &req, FlowActionError::PendingCanvasEdit);
}
#[test]
fn controlled_connection_and_exact_disconnect_preserve_stream_other_endpoint_and_selection() {
    let mut app = app();
    let feed = create(&mut app, BuiltinUnitKind::Feed);
    let outlet = available_material_connections(&app)
        .into_iter()
        .find(|c| c.source.unit_id == feed && c.sink.is_none())
        .unwrap();
    execute(&mut app, FlowAction::Connect(outlet));
    let heater = create(&mut app, BuiltinUnitKind::Heater);
    let target = available_material_connections(&app)
        .into_iter()
        .find(|c| c.source.unit_id == feed && c.sink.as_ref().is_some_and(|p| p.unit_id == heater))
        .unwrap();
    let receipt = execute(&mut app, FlowAction::Connect(target.clone()));
    assert_eq!(receipt.effect, FlowActionEffect::Connected(target.clone()));
    let stream = app.workspace.document.flowsheet.streams[&target.stream_id].clone();
    let selection = app.workspace.drafts.active_target.clone();
    let before_revision = app.workspace.document.revision;
    let req = request(&app, FlowAction::Disconnect(target.source.clone()));
    preflight_flow_action(&app, FlowEditAccess::Editable, &req).unwrap();
    let receipt =
        execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH).unwrap();
    assert_eq!(receipt.after.revision(), before_revision + 1);
    assert_eq!(
        app.workspace.document.flowsheet.streams[&target.stream_id],
        stream
    );
    assert_eq!(
        app.workspace.document.flowsheet.units[&heater].ports[0]
            .stream_id
            .as_ref(),
        Some(&target.stream_id)
    );
    assert_eq!(app.workspace.drafts.active_target, selection);
    let repeated = request(&app, FlowAction::Disconnect(target.source.clone()));
    rejects(
        &mut app,
        &repeated,
        FlowActionError::PortNotBound(target.source),
    );
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.units[&feed].ports[0]
            .stream_id
            .as_ref(),
        Some(&target.stream_id)
    );
}
#[test]
fn stream_disconnect_consumes_both_endpoints_in_one_transaction_and_repeated_call_is_unchanged() {
    let mut app = app();
    let feed = create(&mut app, BuiltinUnitKind::Feed);
    let outlet = available_material_connections(&app)
        .into_iter()
        .find(|c| c.source.unit_id == feed && c.sink.is_none())
        .unwrap();
    execute(&mut app, FlowAction::Connect(outlet));
    let heater = create(&mut app, BuiltinUnitKind::Heater);
    let target = available_material_connections(&app)
        .into_iter()
        .find(|c| c.source.unit_id == feed && c.sink.as_ref().is_some_and(|p| p.unit_id == heater))
        .unwrap();
    execute(&mut app, FlowAction::Connect(target.clone()));
    let before = app.workspace.document.revision;
    let receipt = execute(
        &mut app,
        FlowAction::DisconnectStream {
            stream: target.stream_id.clone(),
            direction: None,
        },
    );
    assert_eq!(receipt.after.revision(), before + 1);
    assert!(
        matches!(receipt.effect, FlowActionEffect::StreamDisconnected { ports, .. } if ports.len() == 2)
    );
    assert!(
        app.workspace
            .document
            .flowsheet
            .streams
            .contains_key(&target.stream_id)
    );
    let receipt = execute(
        &mut app,
        FlowAction::DisconnectStream {
            stream: target.stream_id.clone(),
            direction: None,
        },
    );
    assert_eq!(receipt.outcome, FlowEditOutcome::Unchanged);
    assert_eq!(receipt.before, receipt.after);
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.units[&feed].ports[0]
            .stream_id
            .as_ref(),
        Some(&target.stream_id)
    );
    assert_eq!(
        app.workspace.document.flowsheet.units[&heater].ports[0]
            .stream_id
            .as_ref(),
        Some(&target.stream_id)
    );
}
#[test]
fn invalid_connections_and_missing_ports_are_atomic_and_not_string_classified() {
    let mut app = app();
    let id = create(&mut app, BuiltinUnitKind::Feed);
    let target = MaterialConnectionTarget {
        stream_id: "fabricated".into(),
        source: MaterialPortTarget {
            unit_id: id.clone(),
            port: "outlet".into(),
        },
        sink: None,
    };
    let req = request(&app, FlowAction::Connect(target));
    rejects(
        &mut app,
        &req,
        FlowActionError::Connection(ModelingActionError::ConnectionUnavailable),
    );
    let req = request(
        &app,
        FlowAction::Disconnect(MaterialPortTarget {
            unit_id: id,
            port: "missing".into(),
        }),
    );
    let before = app.clone();
    assert!(matches!(
        execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH),
        Err(FlowActionError::Rejected(_))
    ));
    assert_eq!(app, before);
}
#[test]
fn write_targets_cannot_escape_context_or_mutate_results() {
    let mut app = app();
    let id = create(&mut app, BuiltinUnitKind::Heater);
    let FlowAction::Write {
        mut variable,
        value,
    } = write(
        &app,
        &id,
        VariableField::OutletTemperature,
        VariableValue::Number(330.0),
    )
    else {
        unreachable!()
    };
    variable.section = VariableSection::Results;
    let req = request(
        &app,
        FlowAction::Write {
            variable: variable.clone(),
            value: value.clone(),
        },
    );
    rejects(
        &mut app,
        &req,
        FlowActionError::Write(VariableWriteError::ReadOnly),
    );
    variable.document = "foreign".into();
    let req = request(&app, FlowAction::Write { variable, value });
    rejects(
        &mut app,
        &req,
        FlowActionError::Write(VariableWriteError::Lookup(
            rf_ui::variable_browser::BrowseError::DifferentDocument,
        )),
    );
}
#[test]
fn discovery_only_maps_supported_edits_and_excludes_run_and_equipment() {
    use rf_ui::project_query::{ProjectObjectId, ProjectQuery};
    use rf_ui::variable_browser::ActionExecution;
    let mut app = app();
    engineering(&mut app);
    let query = ProjectQuery::new(&app.workspace);
    let actions = query.actions(None, ActionExecution::Background).unwrap();
    assert_eq!(
        actions
            .iter()
            .filter_map(callable_flow_action)
            .collect::<Vec<_>>(),
        vec![
            FlowActionKind::CreateUnit,
            FlowActionKind::Connect,
            FlowActionKind::Disconnect
        ]
    );
    let target = query.reference(ProjectObjectId::Equipment("rating-1".into()));
    assert!(
        query
            .actions(Some(&target), ActionExecution::Inline)
            .unwrap()
            .iter()
            .all(|a| callable_flow_action(a).is_none())
    );
}

#[test]
fn committed_edits_revoke_flow_results_and_pending_receipts_but_preserve_equipment_results() {
    use rf_ui::engineering::{EquipmentCaseKey, EquipmentResultStatus};
    use rf_ui::result_query::FlowResultView;
    use rf_ui::{
        DiagnosticSeverity, DiagnosticSummary, FlowRunDiscardReason, FlowRunDisposition, RunStatus,
        SolveSnapshot,
    };
    let mut app = app();
    let id = create(&mut app, BuiltinUnitKind::Heater);
    engineering(&mut app);
    let member = EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    };
    app.workspace.engineering.calculate_rating(&member).unwrap();
    let equipment = app.workspace.engineering.clone();
    let revision = app.workspace.document.revision;
    app.store_snapshot(SolveSnapshot::new(
        "accepted",
        revision,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(revision, DiagnosticSeverity::Info, "current"),
    ));
    let mut pending = app.prepare_flow_run().into_receipt();
    let req = request(
        &app,
        write(
            &app,
            &id,
            VariableField::OutletTemperature,
            VariableValue::Number(330.0),
        ),
    );
    let before = app.clone();
    preflight_flow_action(&app, FlowEditAccess::Editable, &req).unwrap();
    assert_eq!(app, before);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Current { .. }
    ));
    execute_flow_action(&mut app, FlowEditAccess::Editable, &req, UNIX_EPOCH).unwrap();
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
    assert_eq!(
        app.accept_flow_run(&mut pending),
        FlowRunDisposition::Discarded(FlowRunDiscardReason::InputChanged)
    );
    assert_eq!(app.workspace.engineering, equipment);
    assert_eq!(
        app.workspace
            .engineering
            .query_case_result(&member)
            .status(),
        EquipmentResultStatus::Current
    );
}
