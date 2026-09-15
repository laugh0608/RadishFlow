use super::*;
use crate::{
    DiagnosticSeverity, DiagnosticSummary, DocumentMetadata, PhaseStateSnapshot, RunStatus,
    StreamStateSnapshot,
};
use rf_model::{Component, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_types::{ComponentId, PortDirection, PortKind};

fn document() -> FlowsheetDocument {
    let mut flow = Flowsheet::new("browser");
    flow.insert_component(Component::new("a", "A")).unwrap();
    let mut stream = MaterialStreamState::new("s", "进料");
    stream.temperature_k = 300.;
    stream.pressure_pa = 100_000.;
    stream.total_molar_flow_mol_s = 1.;
    stream
        .overall_mole_fractions
        .insert(ComponentId::new("a"), 1.);
    flow.insert_stream(stream).unwrap();
    for kind in [
        "feed",
        "heater",
        "cooler",
        "mixer",
        "valve",
        "flash_drum",
        "unsupported",
    ] {
        flow.insert_unit(UnitNode::new(
            kind,
            kind,
            kind,
            vec![UnitPort::new(
                "inlet",
                PortDirection::Inlet,
                PortKind::Material,
                Some(StreamId::new("s")),
            )],
        ))
        .unwrap();
    }
    FlowsheetDocument::new(
        flow,
        DocumentMetadata::new("doc-browser", "Browser", std::time::UNIX_EPOCH),
    )
}

fn id(
    document: &FlowsheetDocument,
    object: ObjectId,
    section: VariableSection,
    field: VariableField,
) -> VariableId {
    VariableId {
        document: document.metadata.document_id.clone(),
        object,
        section,
        field,
    }
}

fn result(document: &FlowsheetDocument) -> SolveSnapshot {
    let mut snapshot = SolveSnapshot::new(
        "solve-1",
        document.revision,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(document.revision, DiagnosticSeverity::Info, "converged"),
    );
    snapshot.streams.push(StreamStateSnapshot {
        stream_id: StreamId::new("s"),
        label: "进料".into(),
        temperature_k: 310.,
        pressure_pa: 90_000.,
        total_molar_flow_mol_s: 1.,
        overall_mole_fractions: vec![("a".into(), 1.)],
        phases: vec![PhaseStateSnapshot {
            label: "vapor".into(),
            phase_fraction: 1.,
            composition: vec![("a".into(), 1.)],
            molar_enthalpy_j_per_mol: Some(-200.),
        }],
        bubble_dew_window: None,
    });
    snapshot
}

#[test]
fn variable_browser_reports_supported_fields_without_inventing_committed_defaults() {
    let document = document();
    let browser = VariableBrowser::new(&document, None, None);
    for kind in [
        "feed",
        "heater",
        "cooler",
        "flash_drum",
        "mixer",
        "valve",
        "unsupported",
    ] {
        let rows = browser
            .variables(&ObjectId::Unit(UnitId::new(kind)))
            .unwrap();
        let count = match kind {
            "mixer" | "valve" => 2,
            "unsupported" => 1,
            _ => 3,
        };
        assert_eq!(rows.len(), count, "{kind}");
        for row in &rows[1..] {
            assert_eq!(row.state, ValueState::Unspecified);
            assert_eq!(row.value, None);
            assert_eq!(row.write_via, Some(ActionKind::SetUnitParameter));
        }
    }
}

#[test]
fn variable_browser_uses_unit_pressure_constraints_and_distinguishes_templates() {
    let mut document = document();
    let unit = document
        .flowsheet
        .units
        .get_mut(&UnitId::new("cooler"))
        .unwrap();
    unit.parameters.outlet_pressure_pa = Some(110_000.);
    unit.parameters.outlet_temperature_k = Some(f64::NAN);
    unit.ports.push(UnitPort::new(
        "outlet",
        PortDirection::Outlet,
        PortKind::Material,
        Some(StreamId::new("s")),
    ));
    let browser = VariableBrowser::new(&document, None, None);
    let rows = browser
        .variables(&ObjectId::Unit(UnitId::new("cooler")))
        .unwrap();
    assert!(rows[1..].iter().all(|r| r.state == ValueState::Invalid));
    let pressure = rows
        .iter()
        .find(|r| r.id.field == VariableField::OutletPressure)
        .unwrap();
    let constraint = pressure.constraint.unwrap();
    assert_eq!(constraint.maximum, Some(100_000.));
    assert!(constraint.accepts(100_000.));
    assert!(!constraint.accepts(0.));
    assert!(!constraint.accepts(f64::INFINITY));
    let stream = browser
        .variables(&ObjectId::Stream(StreamId::new("s")))
        .unwrap();
    assert_eq!(stream[1].source, ValueSource::StreamTemplate);
    assert_eq!(
        browser
            .stream_references(&ObjectId::Unit(UnitId::new("cooler")))
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn variable_browser_identity_survives_rename_and_rejects_deleted_or_foreign_objects() {
    let mut document = document();
    let id = id(
        &document,
        ObjectId::Unit(UnitId::new("heater")),
        VariableSection::Inputs,
        VariableField::Name,
    );
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater"))
        .unwrap()
        .name = "主加热器".into();
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("cooler"))
        .unwrap()
        .name = "主加热器".into();
    let browser = VariableBrowser::new(&document, None, None);
    assert_eq!(
        browser
            .objects()
            .iter()
            .filter(|o| o.name == "主加热器")
            .count(),
        2
    );
    assert_eq!(
        browser.read(&id).unwrap().value,
        Some(VariableValue::Text("主加热器".into()))
    );
    assert!(browser.search("主加热器").iter().any(|r| r.id == id));
    assert!(browser.search("Pa").iter().all(|r| r.unit == "Pa"));
    let mut foreign = id.clone();
    foreign.document = DocumentId::new("other");
    assert_eq!(browser.read(&foreign), Err(BrowseError::DifferentDocument));
    let mut missing = id.clone();
    missing.field = VariableField::MolarFlow;
    assert_eq!(
        browser.read(&missing),
        Err(BrowseError::VariableMissing(missing.clone()))
    );
    document.flowsheet.units.remove(&UnitId::new("heater"));
    assert_eq!(
        VariableBrowser::new(&document, None, None).read(&id),
        Err(BrowseError::ObjectMissing(id.object))
    );
}

#[test]
fn variable_browser_keeps_current_missing_and_stale_results_separate() {
    let mut document = document();
    let snapshot = result(&document);
    let id = id(
        &document,
        ObjectId::Stream(StreamId::new("s")),
        VariableSection::Results,
        VariableField::Temperature,
    );
    let missing = VariableBrowser::new(&document, None, None)
        .read(&id)
        .unwrap();
    assert_eq!(missing.state, ValueState::Missing);
    assert_eq!(missing.source, ValueSource::NoResult);
    let browser = VariableBrowser::new(&document, Some(&snapshot), None);
    let current = browser.read(&id).unwrap();
    assert_eq!(current.state, ValueState::Valid);
    assert_eq!(current.value, Some(VariableValue::Number(310.)));
    assert_eq!(
        current.source,
        ValueSource::SolveResult {
            snapshot: snapshot.id.clone(),
            revision: document.revision,
            sequence: 1
        }
    );
    assert_eq!(current.write_via, None);
    let mut enthalpy = id.clone();
    enthalpy.field = VariableField::PhaseMolarEnthalpy("vapor".into());
    assert_eq!(
        browser.read(&enthalpy).unwrap().value,
        Some(VariableValue::Number(-200.))
    );
    document.revision += 1;
    for browser in [
        VariableBrowser::new(&document, None, Some(&snapshot)),
        VariableBrowser::new(&document, Some(&snapshot), None),
    ] {
        let stale = browser.read(&id).unwrap();
        assert_eq!(stale.state, ValueState::Stale);
        assert_eq!(stale.value, None);
        assert_eq!(browser.read(&enthalpy).unwrap().value, None);
        assert_eq!(
            browser.variables(&id.object).unwrap()[1].value,
            Some(VariableValue::Number(300.))
        );
    }
}

#[test]
fn variable_browser_describes_existing_actions_without_promising_task_execution() {
    let document = document();
    let browser = VariableBrowser::new(&document, None, None);
    let actions = browser.actions(None).unwrap();
    let run = actions.iter().find(|a| a.kind == ActionKind::Run).unwrap();
    assert!(!run.modifies_document && !run.undoable && !run.asynchronous);
    let valve = browser
        .actions(Some(&ObjectId::Unit(UnitId::new("valve"))))
        .unwrap();
    assert_eq!(valve[1].fields, vec![VariableField::OutletPressure]);
    let unknown = browser
        .actions(Some(&ObjectId::Unit(UnitId::new("unsupported"))))
        .unwrap();
    assert_eq!(unknown.len(), 1);
    let stream = browser
        .actions(Some(&ObjectId::Stream(StreamId::new("s"))))
        .unwrap();
    assert!(
        !stream[0]
            .fields
            .iter()
            .any(|f| matches!(f, VariableField::PhaseFraction(_)))
    );
    assert!(
        browser
            .actions(Some(&ObjectId::Stream(StreamId::new("deleted"))))
            .is_err()
    );
}
