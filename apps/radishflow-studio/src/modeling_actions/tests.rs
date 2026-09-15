use super::*;
use rf_store::StoredAuthCacheIndex;
use rf_ui::variable_browser::{
    ObjectId, VariableField, VariableId, VariableSection, VariableValue,
};
use rf_ui::variable_commands::VariableWriteRequest;
use rf_ui::{DocumentMetadata, FlowsheetDocument};
use std::time::{SystemTime, UNIX_EPOCH};

fn blank() -> AppState {
    AppState::new(FlowsheetDocument::new(
        rf_model::Flowsheet::new("Actions"),
        DocumentMetadata::new("action-doc", "Actions", UNIX_EPOCH),
    ))
}

fn request(app: &AppState, action: ModelingAction) -> ModelingActionRequest {
    ModelingActionRequest {
        document_id: app.workspace.document.metadata.document_id.clone(),
        expected_revision: app.workspace.document.revision,
        action,
    }
}

fn dispatch(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    action: ModelingAction,
) -> ModelingActionReceipt {
    let request = request(app, action);
    dispatch_modeling_action(&StudioAppFacade::new(), app, context, request, UNIX_EPOCH).unwrap()
}

fn create(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    kind: BuiltinUnitKind,
) -> UnitId {
    match dispatch(app, context, ModelingAction::CreateUnit(kind)).effect {
        ModelingActionEffect::Created { unit_id, .. } => unit_id,
        other => panic!("expected created unit, got {other:?}"),
    }
}

fn connect(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    source: &UnitId,
    sink: Option<&UnitId>,
    port: &str,
) -> StreamId {
    let matches = available_material_connections(app)
        .into_iter()
        .filter(|c| {
            &c.source.unit_id == source
                && c.source.port == port
                && c.sink.as_ref().map(|p| &p.unit_id) == sink
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected one connection for {source}.{port} -> {sink:?}"
    );
    let target = matches[0].clone();
    dispatch(app, context, ModelingAction::Connect(target.clone()));
    target.stream_id
}

fn write(app: &mut AppState, unit: &UnitId, field: VariableField, value: f64) {
    app.write_variable(
        VariableWriteRequest {
            variable: VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object: ObjectId::Unit(unit.clone()),
                section: VariableSection::Inputs,
                field,
            },
            expected_revision: app.workspace.document.revision,
            value: VariableValue::Number(value),
        },
        UNIX_EPOCH,
    )
    .unwrap();
}

fn cache() -> StoredAuthCacheIndex {
    StoredAuthCacheIndex::new(
        "https://example.invalid",
        "actions-test",
        rf_store::StoredCredentialReference::new("test", "actions-test"),
    )
}

#[test]
fn modeling_action_creation_matches_canvas_for_all_kinds_and_reserves_deleted_ids() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::FlashDrum,
    ] {
        let mut app = blank();
        let mut gui = app.clone();
        let unit = create(&mut app, &context, kind);
        gui.begin_canvas_place_unit(kind.as_str());
        gui.commit_canvas_pending_edit_at(rf_ui::CanvasPoint::new(0., 0.), UNIX_EPOCH)
            .unwrap()
            .unwrap();
        assert_eq!(app.workspace.document, gui.workspace.document);
        assert_eq!(app.workspace.command_history, gui.workspace.command_history);
        app.delete_unit(&unit, UNIX_EPOCH).unwrap();
        let next = create(&mut app, &context, kind);
        assert_ne!(unit, next);
    }
}

#[test]
fn modeling_actions_reject_stale_foreign_and_pending_edits_without_mutation() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    let mut app = blank();
    let mut req = request(&app, ModelingAction::CreateUnit(BuiltinUnitKind::Feed));
    let before = app.clone();
    req.document_id = DocumentId::new("foreign");
    assert_eq!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::DifferentDocument)
    );
    assert_eq!(app, before);
    let stale = request(&app, ModelingAction::CreateUnit(BuiltinUnitKind::Feed));
    let feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    let before = app.clone();
    assert!(matches!(
        dispatch_modeling_action(
            &StudioAppFacade::new(),
            &mut app,
            &context,
            stale,
            UNIX_EPOCH
        ),
        Err(ModelingActionError::RevisionConflict { .. })
    ));
    assert_eq!(app, before);
    app.focus_inspector_target(rf_ui::InspectorTarget::Unit(feed.clone()));
    app.update_unit_inspector_draft(&feed, rf_ui::UnitInspectorDraftField::Name, "draft");
    for action in [
        ModelingAction::CreateUnit(BuiltinUnitKind::Heater),
        ModelingAction::Run(WorkspaceRunPackageSelection::Preferred),
    ] {
        let req = request(&app, action);
        let before = app.clone();
        assert_eq!(
            dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
            Err(ModelingActionError::PendingDrafts)
        );
        assert_eq!(app, before);
    }
    app.discard_unit_inspector_draft(&feed, rf_ui::UnitInspectorDraftField::Name)
        .unwrap();
    app.begin_canvas_place_unit("heater");
    let req = request(&app, ModelingAction::CreateUnit(BuiltinUnitKind::Heater));
    let before = app.clone();
    assert_eq!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::PendingCanvasEdit)
    );
    assert_eq!(app, before);
}

#[test]
fn modeling_connection_uses_fresh_rules_and_preserves_atomic_history() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    let mut app = blank();
    let feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    let target = available_material_connections(&app)[0].clone();
    let mut gui = app.clone();
    let suggestions = generate_local_canvas_suggestions(&gui);
    // A stale UI cache must neither authorize an invalid action nor supply its payload.
    let stale_suggestions = suggestions.clone();
    let suggestion_id = suggestions[0].id.clone();
    gui.replace_canvas_suggestions(suggestions);
    gui.accept_canvas_suggestion(&suggestion_id)
        .unwrap()
        .unwrap();
    dispatch(&mut app, &context, ModelingAction::Connect(target.clone()));
    assert_eq!(
        app.workspace.document.flowsheet,
        gui.workspace.document.flowsheet
    );
    assert_eq!(
        app.workspace
            .command_history
            .entries
            .last()
            .unwrap()
            .command,
        gui.workspace
            .command_history
            .entries
            .last()
            .unwrap()
            .command
    );
    assert_eq!(app.workspace.document.revision, 2);
    app.replace_canvas_suggestions(stale_suggestions);
    let req = request(&app, ModelingAction::Connect(target.clone()));
    let before = app.clone();
    assert_eq!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::ConnectionUnavailable)
    );
    assert_eq!(app, before);
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert!(app.workspace.document.flowsheet.streams.is_empty());
    assert_eq!(
        app.workspace.document.flowsheet.units[&feed].ports[0].stream_id,
        None
    );
    app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert!(
        app.workspace
            .document
            .flowsheet
            .streams
            .contains_key(&target.stream_id)
    );
    let mut invalid = target;
    invalid.source.port = "missing".into();
    let req = request(&app, ModelingAction::Connect(invalid));
    let before = app.clone();
    assert_eq!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::ConnectionUnavailable)
    );
    assert_eq!(app, before);
}

fn cached_package() -> (std::path::PathBuf, StoredAuthCacheIndex) {
    let root = std::env::temp_dir().join(format!(
        "radishflow-actions-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut index = cache();
    let package = crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID;
    crate::test_support::write_official_binary_hydrocarbon_cached_package(
        &root,
        &mut index,
        package,
        SystemTime::now(),
        None,
    );
    (root, index)
}

#[test]
fn modeling_actions_reject_ambiguous_upstream_choices() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    let mut app = blank();
    let feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    let stream = connect(&mut app, &context, &feed, None, "outlet");
    let second_feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    connect(&mut app, &context, &second_feed, None, "outlet");
    let heater = create(&mut app, &context, BuiltinUnitKind::Heater);
    assert!(
        available_material_connections(&app)
            .iter()
            .all(|c| c.sink.is_none())
    );
    let req = request(
        &app,
        ModelingAction::Connect(MaterialConnectionTarget {
            stream_id: stream,
            source: MaterialPortTarget {
                unit_id: feed,
                port: "outlet".into(),
            },
            sink: Some(MaterialPortTarget {
                unit_id: heater,
                port: "inlet".into(),
            }),
        }),
    );
    let before = app.clone();
    assert_eq!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::ConnectionUnavailable)
    );
    assert_eq!(app, before);
}

#[test]
fn modeling_actions_reject_invalid_generated_ports_without_inserting_streams() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    let mut app = blank();
    let feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    // Model an invalid imported endpoint: local suggestion generation alone is not validation.
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&feed)
        .unwrap()
        .ports[0]
        .direction = rf_types::PortDirection::Inlet;
    let target = available_material_connections(&app).remove(0);
    let req = request(&app, ModelingAction::Connect(target));
    let before = app.clone();
    assert!(matches!(
        dispatch_modeling_action(&StudioAppFacade::new(), &mut app, &context, req, UNIX_EPOCH),
        Err(ModelingActionError::Rejected(_))
    ));
    assert_eq!(app, before);
    assert!(app.workspace.document.flowsheet.streams.is_empty());
}

fn run_with_gui_comparison(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
) -> WorkspaceControlActionOutcome {
    let document = app.workspace.document.clone();
    let history = app.workspace.command_history.clone();
    let mut gui = app.clone();
    let ModelingActionEffect::Run(outcome) = dispatch(
        app,
        context,
        ModelingAction::Run(WorkspaceRunPackageSelection::Preferred),
    )
    .effect
    else {
        panic!("expected run outcome");
    };
    let gui_outcome = dispatch_workspace_control_action_with_auth_cache(
        &StudioAppFacade::new(),
        &mut gui,
        context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Preferred),
    )
    .unwrap();
    assert_eq!(*outcome, gui_outcome);
    assert_eq!(app.workspace.document, document);
    assert_eq!(app.workspace.command_history, history);
    *outcome
}

#[test]
fn modeling_actions_preserve_blocked_run_feedback() {
    let index = cache();
    let context = StudioAppAuthCacheContext::new(std::path::Path::new("unused"), &index);
    let mut app = blank();
    let outcome = run_with_gui_comparison(&mut app, &context);
    let crate::StudioAppResultDispatch::WorkspaceRun(run) = outcome.dispatch else {
        panic!("expected workspace run");
    };
    assert!(matches!(
        run.outcome,
        crate::StudioWorkspaceRunOutcome::Blocked(_)
    ));
    assert!(outcome.control_state.notice.is_some());
    assert_ne!(
        app.workspace.solve_session.status,
        rf_ui::RunStatus::Converged
    );
}

#[test]
fn modeling_actions_preserve_solver_failure_and_diagnostics() {
    let (root, index) = cached_package();
    let context = StudioAppAuthCacheContext::new(&root, &index);
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    let mut app = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("actions-failure", "Actions failure", UNIX_EPOCH),
    ));
    app.workspace
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("stream-throttled"))
        .unwrap()
        .pressure_pa = 730_000.;
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("valve-1"))
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(730_000.);
    let outcome = run_with_gui_comparison(&mut app, &context);
    let crate::StudioAppResultDispatch::WorkspaceRun(run) = outcome.dispatch else {
        panic!("expected workspace run");
    };
    assert!(matches!(
        run.outcome,
        crate::StudioWorkspaceRunOutcome::Failed(_)
    ));
    assert_eq!(outcome.control_state.run_status, rf_ui::RunStatus::Error);
    assert_eq!(
        outcome.control_state.notice.unwrap().title,
        "Unit parameter invalid"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn modeling_actions_build_write_connect_and_run_a_blank_feed_heater_flash() {
    let (root, index) = cached_package();
    let package = crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID;
    let context = StudioAppAuthCacheContext::new(&root, &index);
    let mut app = blank();
    app.set_flowsheet_property_package_id(Some(package.into()), UNIX_EPOCH)
        .unwrap();
    for (id, name) in crate::test_support::OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS {
        app.add_flowsheet_component(rf_model::Component::new(id, name), UNIX_EPOCH)
            .unwrap();
    }
    let feed = create(&mut app, &context, BuiltinUnitKind::Feed);
    connect(&mut app, &context, &feed, None, "outlet");
    write(&mut app, &feed, VariableField::OutletTemperature, 310.);
    write(&mut app, &feed, VariableField::OutletPressure, 120_000.);
    let heater = create(&mut app, &context, BuiltinUnitKind::Heater);
    connect(&mut app, &context, &feed, Some(&heater), "outlet");
    let heated = connect(&mut app, &context, &heater, None, "outlet");
    write(&mut app, &heater, VariableField::OutletTemperature, 330.);
    write(&mut app, &heater, VariableField::OutletPressure, 90_000.);
    let flash = create(&mut app, &context, BuiltinUnitKind::FlashDrum);
    connect(&mut app, &context, &heater, Some(&flash), "outlet");
    connect(&mut app, &context, &flash, None, "liquid");
    connect(&mut app, &context, &flash, None, "vapor");
    write(&mut app, &flash, VariableField::OutletTemperature, 280.);
    write(&mut app, &flash, VariableField::OutletPressure, 80_000.);
    let revision = app.workspace.document.revision;
    run_with_gui_comparison(&mut app, &context);
    assert_eq!(
        app.workspace.solve_session.status,
        rf_ui::RunStatus::Converged
    );
    let snapshot = rf_ui::latest_snapshot(&app.workspace).unwrap();
    let stream = snapshot
        .streams
        .iter()
        .find(|s| s.stream_id == heated)
        .unwrap();
    assert_eq!(stream.temperature_k, 330.);
    assert_eq!(stream.pressure_pa, 90_000.);
    assert_eq!(stream.total_molar_flow_mol_s, 1.);
    assert_eq!(snapshot.document_revision, revision);
    std::fs::remove_dir_all(root).unwrap();
}
