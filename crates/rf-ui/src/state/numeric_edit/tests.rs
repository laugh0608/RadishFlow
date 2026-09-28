use super::*;
use crate::variable_browser::{
    ObjectId, VariableField, VariableId, VariableSection, VariableValue,
};
use crate::variable_commands::{VariableWriteError, VariableWriteRequest};
use crate::{DocumentMetadata, FlowsheetDocument};
use rf_types::units::{MeasurementUnit as U, QuantityKind as Q};
use std::time::UNIX_EPOCH;

fn app() -> AppState {
    let mut flowsheet = Flowsheet::new("numeric editing");
    let mut stream = MaterialStreamState::new("s", "Feed");
    stream.temperature_k = 300.1234567890123;
    stream.pressure_pa = 200_000.0;
    stream.total_molar_flow_mol_s = 1.23456789012345;
    flowsheet.insert_stream(stream).unwrap();
    let mut valve = UnitNode::new(
        "v",
        "Valve",
        "valve",
        vec![UnitPort::new(
            "inlet",
            PortDirection::Inlet,
            PortKind::Material,
            Some("s".into()),
        )],
    );
    valve.parameters.outlet_pressure_pa = Some(90_000.0);
    flowsheet.insert_unit(valve).unwrap();
    AppState::new(FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("d", "Demo", UNIX_EPOCH),
    ))
}
fn id(app: &AppState, field: VariableField) -> VariableId {
    VariableId {
        document: app.workspace.document.metadata.document_id.clone(),
        object: ObjectId::Stream("s".into()),
        section: VariableSection::Inputs,
        field,
    }
}
fn edit(app: &mut AppState, id: &VariableId, event: NumericEditEvent) -> u64 {
    let generation = app.workspace.numeric_edit(id).unwrap().generation();
    app.edit_numeric(id, generation, event).unwrap()
}
fn text(app: &mut AppState, id: &VariableId, raw: &str) -> u64 {
    edit(app, id, NumericEditEvent::ReplaceText(raw.into()))
}

#[test]
fn parser_distinguishes_prefixes_invalid_values_and_suffix_conflicts() {
    for raw in ["", " ", "+", "-", ".", "-.", "1e", "1e-", "+2.3E+"] {
        assert_eq!(
            parse_numeric_input(
                raw,
                Q::AbsoluteTemperature,
                U::Kelvin,
                InputUnitOrigin::Display
            ),
            NumericParseOutcome::Incomplete,
            "{raw}"
        );
    }
    for raw in [
        "NaN", "inf", "1,234", "1+2", "1e999", "300 barg", "300 bar", "300 foo",
    ] {
        assert!(
            matches!(
                parse_numeric_input(
                    raw,
                    Q::AbsoluteTemperature,
                    U::Kelvin,
                    InputUnitOrigin::Display
                ),
                NumericParseOutcome::Invalid(_)
            ),
            "{raw}"
        );
    }
    assert_eq!(
        parse_numeric_input(
            " 300 K ",
            Q::AbsoluteTemperature,
            U::Celsius,
            InputUnitOrigin::Display
        ),
        NumericParseOutcome::Value {
            si: 300.0,
            unit: U::Kelvin,
            origin: InputUnitOrigin::Suffix
        }
    );
    assert!(matches!(
        parse_numeric_input(
            "300 K",
            Q::AbsoluteTemperature,
            U::Celsius,
            InputUnitOrigin::Explicit
        ),
        NumericParseOutcome::Invalid(NumericParseError::UnitConflict { .. })
    ));
    assert!(matches!(
        parse_numeric_input(
            "1 bar",
            Q::AbsolutePressure,
            U::Pascal,
            InputUnitOrigin::Display
        ),
        NumericParseOutcome::Value { si: 100000.0, .. }
    ));
}

#[test]
fn unit_round_trips_preserve_original_si_and_real_tiny_edits_are_not_swallowed() {
    let mut app = app();
    for (field, units) in [
        (VariableField::Temperature, [U::Celsius, U::Kelvin]),
        (VariableField::Pressure, [U::Bar, U::Pascal]),
        (
            VariableField::MolarFlow,
            [U::KilomolePerHour, U::MolePerSecond],
        ),
    ] {
        let variable = id(&app, field);
        app.begin_numeric_edit(variable.clone(), None).unwrap();
        assert_eq!(app.workspace.drafts.pending_count(), 0);
        let original = app
            .workspace
            .numeric_edit(&variable)
            .unwrap()
            .candidate_si()
            .unwrap();
        for _ in 0..100 {
            for unit in units {
                edit(&mut app, &variable, NumericEditEvent::SelectInputUnit(unit));
            }
        }
        let session = app.workspace.numeric_edit(&variable).unwrap();
        assert_eq!(
            session.candidate_si().unwrap().to_bits(),
            original.to_bits()
        );
        let receipt = app
            .commit_numeric_edit(&variable, session.generation(), UNIX_EPOCH)
            .unwrap();
        assert!(receipt.command.is_none());
        assert_eq!(receipt.revision, 0);
    }
    let variable = id(&app, VariableField::Temperature);
    app.begin_numeric_edit(variable.clone(), None).unwrap();
    let generation = text(&mut app, &variable, "300.1234567890124");
    assert!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH)
            .unwrap()
            .command
            .is_some()
    );
    assert_eq!(
        app.workspace.document.flowsheet.streams[&StreamId::new("s")].temperature_k,
        300.1234567890124
    );
}

#[test]
fn invalid_switch_is_atomic_and_history_restores_text_unit_and_precision() {
    let mut app = app();
    let variable = id(&app, VariableField::Temperature);
    app.begin_numeric_edit(variable.clone(), None).unwrap();
    text(&mut app, &variable, "300 K");
    edit(
        &mut app,
        &variable,
        NumericEditEvent::SelectInputUnit(U::Celsius),
    );
    let session = app.workspace.numeric_edit(&variable).unwrap();
    assert!((session.raw_text().parse::<f64>().unwrap() - 26.85).abs() < 1e-12);
    assert_eq!(session.candidate_si().unwrap(), 300.0);
    text(&mut app, &variable, "1e-");
    let before = app.clone();
    let generation = app.workspace.numeric_edit(&variable).unwrap().generation();
    assert_eq!(
        app.edit_numeric(
            &variable,
            generation,
            NumericEditEvent::SelectInputUnit(U::Kelvin)
        ),
        Err(NumericEditError::Incomplete)
    );
    assert_eq!(app, before);
    assert_eq!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH),
        Err(NumericEditError::Incomplete)
    );
    assert_eq!(app, before);
    edit(&mut app, &variable, NumericEditEvent::Undo);
    assert_eq!(
        app.workspace.numeric_edit(&variable).unwrap().input_unit(),
        U::Celsius
    );
    edit(&mut app, &variable, NumericEditEvent::Undo);
    let session = app.workspace.numeric_edit(&variable).unwrap();
    assert_eq!(session.raw_text(), "300 K");
    assert_eq!(session.input_unit(), U::Kelvin);
    edit(&mut app, &variable, NumericEditEvent::Redo);
    assert_eq!(
        app.workspace
            .numeric_edit(&variable)
            .unwrap()
            .candidate_si()
            .unwrap(),
        300.0
    );
}

#[test]
fn committing_one_field_preserves_other_text_and_revalidates_live_pressure_limit() {
    let mut app = app();
    let pressure = id(&app, VariableField::Pressure);
    let valve = VariableId {
        object: ObjectId::Unit("v".into()),
        field: VariableField::OutletPressure,
        ..pressure.clone()
    };
    app.begin_numeric_edit(valve.clone(), Some(U::Bar)).unwrap();
    text(&mut app, &valve, " 1.5 ");
    app.begin_numeric_edit(pressure.clone(), None).unwrap();
    let generation = text(&mut app, &pressure, "1 bar");
    app.commit_numeric_edit(&pressure, generation, UNIX_EPOCH)
        .unwrap();
    let session = app.workspace.numeric_edit(&valve).unwrap();
    assert_eq!(session.raw_text(), " 1.5 ");
    assert_eq!(session.input_unit(), U::Bar);
    assert!(matches!(
        session.validation(),
        Err(NumericEditError::Rejected(_))
    ));
    let generation = session.generation();
    let before = app.clone();
    assert!(
        app.commit_numeric_edit(&valve, generation, UNIX_EPOCH)
            .is_err()
    );
    assert_eq!(app, before);
    edit(&mut app, &valve, NumericEditEvent::Undo);
    assert_eq!(
        app.workspace
            .numeric_edit(&valve)
            .unwrap()
            .candidate_si()
            .unwrap(),
        90_000.0
    );
}

#[test]
fn changed_field_or_source_conflicts_and_stale_events_cannot_overwrite_new_sessions() {
    let mut app = app();
    let variable = id(&app, VariableField::Pressure);
    let first = app.begin_numeric_edit(variable.clone(), None).unwrap();
    let generation = text(&mut app, &variable, "120000");
    assert!(matches!(
        app.commit_numeric_edit(&variable, first, UNIX_EPOCH),
        Err(NumericEditError::StaleGeneration { .. })
    ));
    // A separate engineering operation modifies the same field. Its retained draft must conflict.
    let mut next = app.workspace.document.flowsheet.clone();
    next.streams
        .get_mut(&StreamId::new("s"))
        .unwrap()
        .pressure_pa = 180_000.0;
    app.commit_document_change(
        DocumentCommand::SetStreamSpecification {
            stream_id: "s".into(),
            field: "pressure_pa".into(),
            value: CommandValue::Number(180_000.0),
        },
        next,
        UNIX_EPOCH,
    );
    assert_eq!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH),
        Err(NumericEditError::FieldConflict)
    );
    let generation = edit(&mut app, &variable, NumericEditEvent::Rebase);
    assert!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH)
            .is_ok()
    );
    let new_generation = app.begin_numeric_edit(variable.clone(), None).unwrap();
    assert_ne!(new_generation, first);
    assert!(app.cancel_numeric_edit(&variable, first).is_err());
}

#[test]
fn legacy_si_writes_and_engineering_history_do_not_discard_pending_numeric_edits() {
    let mut app = app();
    let variable = id(&app, VariableField::Pressure);
    app.begin_numeric_edit(variable.clone(), None).unwrap();
    text(&mut app, &variable, "-1");
    let before = app.clone();
    assert_eq!(
        app.write_variable(
            VariableWriteRequest {
                variable: variable.clone(),
                expected_revision: 0,
                value: VariableValue::Number(100000.0)
            },
            UNIX_EPOCH
        ),
        Err(VariableWriteError::PendingDrafts)
    );
    assert!(app.undo_document_command(UNIX_EPOCH).is_err());
    assert!(
        app.delete_stream_and_connections(&StreamId::new("s"), UNIX_EPOCH)
            .is_err()
    );
    assert_eq!(app, before);
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(
            rf_types::units::DisplayUnitSet::engineering(),
        ));
    let session = app.workspace.numeric_edit(&variable).unwrap();
    assert_eq!(session.raw_text(), "-1");
    assert_eq!(session.input_unit(), U::Pascal);
}

#[test]
fn adopting_an_inherited_value_is_a_real_transaction_even_when_value_is_identical() {
    let mut app = app();
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("v"))
        .unwrap()
        .parameters
        .outlet_pressure_pa = None;
    let variable = VariableId {
        object: ObjectId::Unit("v".into()),
        field: VariableField::OutletPressure,
        ..id(&app, VariableField::Pressure)
    };
    let generation = app.begin_numeric_edit(variable.clone(), None).unwrap();
    let value = app
        .workspace
        .numeric_edit(&variable)
        .unwrap()
        .candidate_si()
        .unwrap();
    let receipt = app
        .commit_numeric_edit(&variable, generation, UNIX_EPOCH)
        .unwrap();
    assert!(receipt.command.is_some());
    assert_eq!(
        app.workspace.document.flowsheet.units[&UnitId::new("v")]
            .parameters
            .outlet_pressure_pa,
        Some(value)
    );
}

#[test]
fn inherited_source_changes_conflict_even_with_an_identical_number() {
    let mut app = app();
    let value = app.workspace.document.flowsheet.streams[&StreamId::new("s")].pressure_pa;
    let valve = app
        .workspace
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("v"))
        .unwrap();
    valve.parameters.outlet_pressure_pa = None;
    valve.ports.push(UnitPort::new(
        "outlet",
        PortDirection::Outlet,
        PortKind::Material,
        Some("s".into()),
    ));
    let variable = VariableId {
        object: ObjectId::Unit("v".into()),
        field: VariableField::OutletPressure,
        ..id(&app, VariableField::Pressure)
    };
    app.begin_numeric_edit(variable.clone(), None).unwrap();
    let generation = text(&mut app, &variable, "100000");
    // Equal numbers are not equal source semantics: inherited -> specified still conflicts.
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("v"))
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(value);
    assert_eq!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH),
        Err(NumericEditError::FieldConflict)
    );
    assert_eq!(
        app.workspace.numeric_edit(&variable).unwrap().raw_text(),
        "100000"
    );
}

#[test]
fn readonly_foreign_and_removed_fields_never_commit_or_drop_the_original_session() {
    let mut app = app();
    let variable = id(&app, VariableField::Pressure);
    let mut foreign = variable.clone();
    foreign.document = "another-project".into();
    assert!(matches!(
        app.begin_numeric_edit(foreign, None),
        Err(NumericEditError::Lookup(_))
    ));
    let mut readonly = variable.clone();
    readonly.section = VariableSection::Results;
    assert_eq!(
        app.begin_numeric_edit(readonly, None),
        Err(NumericEditError::UnsupportedField)
    );
    app.begin_numeric_edit(variable.clone(), None).unwrap();
    let generation = text(&mut app, &variable, "1 bar");
    app.workspace
        .document
        .flowsheet
        .streams
        .remove(&StreamId::new("s"));
    let before = app.clone();
    assert!(matches!(
        app.commit_numeric_edit(&variable, generation, UNIX_EPOCH),
        Err(NumericEditError::Lookup(_))
    ));
    assert_eq!(app, before);
    app.cancel_numeric_edit(&variable, generation).unwrap();
    assert_eq!(app.workspace.drafts.pending_count(), 0);
}

#[test]
fn typing_groups_keep_units_and_precision_in_one_undo_owner() {
    let mut app = app();
    let id = id(&app, VariableField::Temperature);
    app.begin_numeric_edit(id.clone(), None).unwrap();
    for raw in ["3", "31", "315"] {
        edit(
            &mut app,
            &id,
            NumericEditEvent::ReplaceTextGrouped {
                raw: raw.into(),
                group: (10, 1),
            },
        );
    }
    edit(&mut app, &id, NumericEditEvent::SelectInputUnit(U::Celsius));
    edit(&mut app, &id, NumericEditEvent::Undo);
    let session = app.workspace.numeric_edit(&id).unwrap();
    assert_eq!(session.raw_text(), "315");
    assert_eq!(session.input_unit(), U::Kelvin);
    edit(&mut app, &id, NumericEditEvent::Undo);
    let session = app.workspace.numeric_edit(&id).unwrap();
    assert!(!session.is_pending());
    assert_eq!(
        session.precision_source(),
        NumericPrecisionSource::OriginalSi
    );
    assert_eq!(session.candidate_si().unwrap(), 300.1234567890123);
    edit(&mut app, &id, NumericEditEvent::Redo);
    assert_eq!(app.workspace.numeric_edit(&id).unwrap().raw_text(), "315");
}

#[test]
fn ime_preedit_cannot_commit_or_switch_and_confirmation_is_one_edit() {
    let mut app = app();
    let id = id(&app, VariableField::Temperature);
    app.begin_numeric_edit(id.clone(), None).unwrap();
    for raw in ["3", "31", "315"] {
        let generation = edit(
            &mut app,
            &id,
            NumericEditEvent::PreviewComposition(raw.into()),
        );
        assert_eq!(
            app.commit_numeric_edit(&id, generation, UNIX_EPOCH),
            Err(NumericEditError::Incomplete)
        );
        assert_eq!(
            app.edit_numeric(
                &id,
                generation,
                NumericEditEvent::SelectInputUnit(U::Celsius)
            ),
            Err(NumericEditError::Incomplete)
        );
        assert!(
            app.workspace
                .numeric_field_presentation(&id)
                .unwrap()
                .composing
        );
        assert_eq!(app.workspace.drafts.pending_count(), 1);
    }
    edit(
        &mut app,
        &id,
        NumericEditEvent::CommitComposition("315".into()),
    );
    assert!(
        !app.workspace
            .numeric_field_presentation(&id)
            .unwrap()
            .composing
    );
    assert_eq!(app.workspace.document.revision, 0);
    edit(&mut app, &id, NumericEditEvent::Undo);
    assert!(!app.workspace.numeric_edit(&id).unwrap().is_pending());
    let original = app
        .workspace
        .numeric_edit(&id)
        .unwrap()
        .raw_text()
        .to_owned();
    edit(
        &mut app,
        &id,
        NumericEditEvent::PreviewComposition("pending".into()),
    );
    edit(&mut app, &id, NumericEditEvent::CommitComposition(original));
    assert!(
        !app.workspace
            .numeric_field_presentation(&id)
            .unwrap()
            .composing
    );
    text(&mut app, &id, "320");
    edit(
        &mut app,
        &id,
        NumericEditEvent::PreviewComposition("pending".into()),
    );
    edit(&mut app, &id, NumericEditEvent::CancelComposition);
    assert_eq!(app.workspace.numeric_edit(&id).unwrap().raw_text(), "320");
}

#[test]
fn display_projection_keeps_active_input_and_engineering_history_independent() {
    let mut app = app();
    let id = id(&app, VariableField::Temperature);
    let revision = app.workspace.document.revision;
    app.begin_numeric_edit(id.clone(), None).unwrap();
    text(&mut app, &id, "1e-");
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(
            rf_types::units::DisplayUnitSet::engineering(),
        ));
    let field = app.workspace.numeric_field_presentation(&id).unwrap();
    assert_eq!(field.input_unit, U::Kelvin);
    assert_eq!(field.display_unit, U::Celsius);
    assert_eq!(field.text, "1e-");
    assert_eq!(field.issue, Some(NumericFieldIssue::Incomplete));
    assert!(!field.can_apply);
    app.cancel_numeric_edit(&id, field.generation.unwrap())
        .unwrap();
    let field = app.workspace.numeric_field_presentation(&id).unwrap();
    assert_eq!(field.input_unit, U::Celsius);
    assert!(field.text.starts_with("26.973456"));
    assert_eq!(app.workspace.document.revision, revision);
}
