use super::*;
use crate::UnitInspectorDraftField;
use crate::variable_browser::{
    BrowseError, ObjectId, VariableBrowser, VariableField, VariableId, VariableSection,
    VariableType, VariableValue,
};
use crate::variable_commands::{VariableWriteError, VariableWriteRequest};

fn input_app(kind: &str) -> AppState {
    let mut doc = sample_feed_flash_document();
    let mut inlet = doc.flowsheet.streams[&StreamId::new("stream-feed")].clone();
    inlet.pressure_pa = 100_000.;
    inlet
        .overall_mole_fractions
        .insert(ComponentId::new("component-a"), 0.5);
    inlet
        .overall_mole_fractions
        .insert(ComponentId::new("component-b"), 0.5);
    doc.flowsheet.streams.insert(inlet.id.clone(), inlet);
    doc.flowsheet
        .insert_stream(MaterialStreamState::new("out", "Outlet"))
        .unwrap();
    doc.flowsheet
        .insert_unit(UnitNode::new(
            "target",
            "Target",
            kind,
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
                    Some("out".into()),
                ),
            ],
        ))
        .unwrap();
    AppState::new(doc)
}

fn request(
    app: &AppState,
    object: ObjectId,
    field: VariableField,
    value: VariableValue,
) -> VariableWriteRequest {
    VariableWriteRequest {
        variable: VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object,
            section: VariableSection::Inputs,
            field,
        },
        expected_revision: app.workspace.document.revision,
        value,
    }
}

fn temperature(app: &AppState, value: f64) -> VariableWriteRequest {
    request(
        app,
        ObjectId::Unit(UnitId::new("target")),
        VariableField::OutletTemperature,
        VariableValue::Number(value),
    )
}

#[test]
fn variable_write_matches_inspector_commands_for_all_six_unit_types() {
    for kind in ["feed", "heater", "cooler", "mixer", "valve", "flash_drum"] {
        let mut direct = input_app(kind);
        let mut gui = direct.clone();
        gui.focus_inspector_target(InspectorTarget::Unit(UnitId::new("target")));
        let fields = if matches!(kind, "mixer" | "valve") {
            vec![(
                VariableField::OutletPressure,
                UnitInspectorDraftField::OutletPressurePa,
                90_000.,
            )]
        } else {
            vec![
                (
                    VariableField::OutletTemperature,
                    UnitInspectorDraftField::OutletTemperatureK,
                    280.,
                ),
                (
                    VariableField::OutletPressure,
                    UnitInspectorDraftField::OutletPressurePa,
                    90_000.,
                ),
            ]
        };
        for (variable, field, value) in fields {
            let request = request(
                &direct,
                ObjectId::Unit(UnitId::new("target")),
                variable,
                VariableValue::Number(value),
            );
            let receipt = direct.write_variable(request, timestamp(30)).unwrap();
            gui.update_unit_inspector_draft(
                &UnitId::new("target"),
                field.clone(),
                value.to_string(),
            )
            .unwrap();
            let commit = gui
                .commit_unit_inspector_draft(&UnitId::new("target"), field, timestamp(30))
                .unwrap()
                .unwrap();
            assert_eq!(receipt.command, Some(commit.command), "{kind}");
            assert_eq!(direct.workspace.document, gui.workspace.document);
            assert_eq!(
                direct.workspace.command_history,
                gui.workspace.command_history
            );
            assert_eq!(direct.workspace.solve_session, gui.workspace.solve_session);
        }
        assert_eq!(direct.workspace.selection, crate::SelectionState::default());
        assert!(direct.workspace.drafts.active_target.is_none());
    }
}

#[test]
fn variable_write_matches_inspector_for_stream_scalars_composition_and_names() {
    let mut direct = input_app("heater");
    let mut gui = direct.clone();
    let stream = StreamId::new("stream-feed");
    gui.focus_inspector_target(InspectorTarget::Stream(stream.clone()));
    for (variable, field, value, raw) in [
        (
            VariableField::Name,
            crate::StreamInspectorDraftField::Name,
            VariableValue::Text("主进料".into()),
            "主进料",
        ),
        (
            VariableField::Temperature,
            crate::StreamInspectorDraftField::TemperatureK,
            VariableValue::Number(320.),
            "320",
        ),
        (
            VariableField::Pressure,
            crate::StreamInspectorDraftField::PressurePa,
            VariableValue::Number(80_000.),
            "80000",
        ),
        (
            VariableField::MolarFlow,
            crate::StreamInspectorDraftField::TotalMolarFlowMolS,
            VariableValue::Number(2.),
            "2",
        ),
        (
            VariableField::MoleFraction(ComponentId::new("component-a")),
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new("component-a")),
            VariableValue::Number(0.25),
            "0.25",
        ),
    ] {
        direct
            .write_variable(
                request(&direct, ObjectId::Stream(stream.clone()), variable, value),
                timestamp(30),
            )
            .unwrap();
        gui.update_stream_inspector_draft(&stream, field.clone(), raw)
            .unwrap();
        gui.commit_stream_inspector_draft(&stream, field, timestamp(30))
            .unwrap()
            .unwrap();
        assert_eq!(direct.workspace.document, gui.workspace.document);
        assert_eq!(
            direct.workspace.command_history,
            gui.workspace.command_history
        );
    }
    let name = request(
        &direct,
        ObjectId::Unit(UnitId::new("target")),
        VariableField::Name,
        VariableValue::Text("主加热器".into()),
    );
    direct.write_variable(name.clone(), timestamp(40)).unwrap();
    assert_eq!(
        VariableBrowser::new(&direct.workspace.document, None, None)
            .read(&name.variable)
            .unwrap()
            .value,
        Some(name.value)
    );
    assert!(matches!(
        direct
            .workspace
            .command_history
            .entries
            .last()
            .unwrap()
            .command,
        DocumentCommand::RenameUnit { .. }
    ));
}

#[test]
fn variable_write_rejections_are_atomic_and_revision_guard_includes_noops() {
    let mut app = input_app("cooler");
    for value in [0., -1., f64::NAN, f64::INFINITY] {
        let before = app.workspace.clone();
        assert!(matches!(
            app.write_variable(temperature(&app, value), timestamp(30)),
            Err(VariableWriteError::Rejected(_))
        ));
        assert_eq!(app.workspace, before);
    }
    let mut req = temperature(&app, 280.);
    req.value = VariableValue::Text("280".into());
    assert_eq!(
        app.write_variable(req, timestamp(30)),
        Err(VariableWriteError::TypeMismatch {
            expected: VariableType::Number
        })
    );
    let mut req = temperature(&app, 280.);
    req.variable.document = crate::DocumentId::new("foreign");
    assert_eq!(
        app.write_variable(req, timestamp(30)),
        Err(VariableWriteError::Lookup(BrowseError::DifferentDocument))
    );
    let mut stale = temperature(&app, 280.);
    app.write_variable(stale.clone(), timestamp(30)).unwrap();
    let before = app.workspace.clone();
    assert_eq!(
        app.write_variable(stale.clone(), timestamp(40)),
        Err(VariableWriteError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    );
    stale.expected_revision = 1;
    stale.variable.object = ObjectId::Unit(UnitId::new("deleted"));
    assert!(matches!(
        app.write_variable(stale, timestamp(40)),
        Err(VariableWriteError::Lookup(BrowseError::ObjectMissing(_)))
    ));
    let mut result = temperature(&app, 280.);
    result.variable.section = VariableSection::Results;
    assert_eq!(
        app.write_variable(result, timestamp(40)),
        Err(VariableWriteError::ReadOnly)
    );
    assert_eq!(app.workspace, before);
}

#[test]
fn variable_write_preserves_drafts_and_rechecks_live_pressure_limits() {
    let mut app = input_app("cooler");
    app.focus_inspector_target(InspectorTarget::Unit(UnitId::new("target")));
    app.update_unit_inspector_draft(
        &UnitId::new("target"),
        UnitInspectorDraftField::OutletTemperatureK,
        "invalid",
    );
    let before = app.workspace.clone();
    assert_eq!(
        app.write_variable(temperature(&app, 280.), timestamp(30)),
        Err(VariableWriteError::PendingDrafts)
    );
    assert_eq!(app.workspace, before);
    app.discard_unit_inspector_draft(
        &UnitId::new("target"),
        UnitInspectorDraftField::OutletTemperatureK,
    )
    .unwrap();
    let high_pressure = request(
        &app,
        ObjectId::Unit(UnitId::new("target")),
        VariableField::OutletPressure,
        VariableValue::Number(110_000.),
    );
    let before = app.workspace.clone();
    assert!(matches!(
        app.write_variable(high_pressure, timestamp(30)),
        Err(VariableWriteError::Rejected(_))
    ));
    assert_eq!(app.workspace, before);
}

#[test]
fn variable_write_noop_preserves_current_results_and_changes_support_undo_redo() {
    let mut app = input_app("heater");
    app.write_variable(temperature(&app, 280.), timestamp(30))
        .unwrap();
    app.store_snapshot(SolveSnapshot::new(
        "current",
        1,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(1, DiagnosticSeverity::Info, "converged"),
    ));
    let before = app.workspace.clone();
    let noop = app
        .write_variable(temperature(&app, 280.), timestamp(40))
        .unwrap();
    assert!(noop.command.is_none());
    assert_eq!(app.workspace, before);
    app.write_variable(temperature(&app, 275.), timestamp(40))
        .unwrap();
    assert!(latest_snapshot(&app.workspace).is_none());
    assert!(crate::stale_snapshot(&app.workspace).is_some());
    assert_eq!(
        app.workspace.document.flowsheet.streams[&StreamId::new("out")].temperature_k,
        275.
    );
    app.undo_document_command(timestamp(50)).unwrap().unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.streams[&StreamId::new("out")].temperature_k,
        280.
    );
    app.redo_document_command(timestamp(60)).unwrap().unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.streams[&StreamId::new("out")].temperature_k,
        275.
    );
    assert_eq!(app.workspace.command_history.len(), 2);
    assert_eq!(app.workspace.document.revision, 4);
}

#[test]
fn variable_write_rejects_unsupported_fields_and_can_set_missing_project_composition() {
    let mut app = input_app("valve");
    let before = app.workspace.clone();
    assert!(matches!(
        app.write_variable(temperature(&app, 280.), timestamp(30)),
        Err(VariableWriteError::Lookup(BrowseError::VariableMissing(_)))
    ));
    assert_eq!(app.workspace, before);
    let missing_fraction = request(
        &app,
        ObjectId::Stream(StreamId::new("out")),
        VariableField::MoleFraction(ComponentId::new("component-a")),
        VariableValue::Number(1.),
    );
    let variable = missing_fraction.variable.clone();
    app.write_variable(missing_fraction, timestamp(30)).unwrap();
    assert_eq!(
        VariableBrowser::new(&app.workspace.document, None, None)
            .read(&variable)
            .unwrap()
            .value,
        Some(VariableValue::Number(1.))
    );
    let before = app.workspace.clone();
    let invalid = request(
        &app,
        variable.object,
        variable.field,
        VariableValue::Number(0.),
    );
    assert!(matches!(
        app.write_variable(invalid, timestamp(40)),
        Err(VariableWriteError::Rejected(_))
    ));
    assert_eq!(app.workspace, before);
}

#[test]
fn variable_write_identical_value_still_validates_composition_and_persisted_inputs() {
    let mut app = input_app("heater");
    let stream = app
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("stream-feed"))
        .unwrap();
    for value in stream.overall_mole_fractions.values_mut() {
        *value = 0.;
    }
    let req = request(
        &app,
        ObjectId::Stream(StreamId::new("stream-feed")),
        VariableField::MoleFraction(ComponentId::new("component-a")),
        VariableValue::Number(0.),
    );
    let before = app.workspace.clone();
    assert!(matches!(
        app.write_variable(req, timestamp(30)),
        Err(VariableWriteError::Rejected(_))
    ));
    assert_eq!(app.workspace, before);
}

#[test]
fn input_command_batch_rolls_back_earlier_values_when_later_validation_fails() {
    let mut app = input_app("heater");
    let before = app.workspace.clone();
    let command = DocumentCommand::SetStreamSpecifications {
        stream_id: StreamId::new("stream-feed"),
        values: vec![
            crate::StreamSpecificationValue {
                field: "temperature_k".into(),
                value: CommandValue::Number(350.),
            },
            crate::StreamSpecificationValue {
                field: "pressure_pa".into(),
                value: CommandValue::Number(-1.),
            },
        ],
    };
    assert!(
        app.workspace
            .commit_input_document_command(command, timestamp(30))
            .is_err()
    );
    assert_eq!(app.workspace, before);
}
