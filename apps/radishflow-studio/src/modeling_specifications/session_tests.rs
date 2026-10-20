use super::*;
use rf_ui::DocumentMetadata;

fn document() -> FlowsheetDocument {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("session-test", "Example", std::time::UNIX_EPOCH),
    )
}

fn missing_temperature(document: &mut FlowsheetDocument) {
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .parameters
        .outlet_temperature_k = None;
    document.revision += 1;
}

#[test]
fn unknown_stale_and_failed_reports_never_publish_zero_current_problems() {
    let mut document = document();
    missing_temperature(&mut document);
    let mut session = ModelingCheckSession::new(&document);
    assert_eq!(session.current_count(), None);
    session.refresh(&document);
    assert_eq!(session.current_count(), Some(1));
    let old = session.report().unwrap().clone();
    document.revision += 1;
    session.observe_document(&document);
    assert_eq!(session.phase(), &ModelingCheckPhase::Stale);
    assert_eq!(session.current_count(), None);
    let ticket = session.begin(&document);
    session.finish(&ticket, Err(ModelingCheckFailure::Panicked), &document);
    assert_eq!(session.current_count(), None);
    assert_eq!(session.report(), Some(&old));
    assert_eq!(session.entries().len(), 1);
    session.refresh(&document);
    assert!(
        matches!(session.phase(), ModelingCheckPhase::Failed(_)),
        "a failed check awaits explicit retry"
    );
    session.recheck(&document);
    assert_eq!(session.current_count(), Some(1));
}

#[test]
fn late_revision_generation_and_reopened_session_results_are_rejected() {
    let mut document = document();
    let mut session = ModelingCheckSession::new(&document);
    let first = session.begin(&document);
    let first_report = check_modeling_specifications(&document, first.identity.session);
    let next = session.begin(&document);
    assert!(!session.finish(&first, Ok(first_report.clone()), &document));
    document.revision += 1;
    assert!(!session.finish(&next, Ok(first_report.clone()), &document));
    let reopened = session.begin(&document);
    let reopened_report = check_modeling_specifications(&document, reopened.identity.session);
    session.replace_document(&document);
    assert_ne!(session.identity().session, reopened.identity.session);
    assert!(!session.finish(&reopened, Ok(reopened_report), &document));
    session.refresh(&document);
    assert_eq!(session.current_count(), Some(0));
}

#[test]
fn wrong_report_identity_is_failure_and_preserves_previous_facts() {
    let mut document = document();
    missing_temperature(&mut document);
    let mut session = ModelingCheckSession::new(&document);
    session.refresh(&document);
    let before = session.entries().to_vec();
    let ticket = session.begin(&document);
    let wrong = check_modeling_specifications(&document, ModelingCheckSessionId(999));
    assert!(session.finish(&ticket, Ok(wrong), &document));
    assert_eq!(
        session.phase(),
        &ModelingCheckPhase::Failed(ModelingCheckFailure::RejectedReportIdentity)
    );
    assert_eq!(session.entries(), before);
}

#[test]
fn pressure_fact_identity_ignores_values_names_and_revision() {
    let mut document = document();
    let heater = document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap();
    heater.parameters.outlet_pressure_pa = Some(1e9);
    let mut session = ModelingCheckSession::new(&document);
    session.refresh(&document);
    let key = session
        .entries()
        .iter()
        .find(|e| e.key.rule == SpecificationRule::PressureRelationship)
        .unwrap()
        .key
        .clone();
    let heater = document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap();
    heater.name = "新名称".into();
    heater.parameters.outlet_pressure_pa = Some(2e9);
    document.revision += 1;
    session.refresh(&document);
    assert!(session.entries().iter().any(|e| e.key == key));
    assert!(session.changes().is_empty());
}

#[test]
fn interrupted_scope_stays_unconfirmed_across_rechecks_until_evaluated() {
    let mut document = document();
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(1e9);
    let mut session = ModelingCheckSession::new(&document);
    session.refresh(&document);
    let original_ports = document.flowsheet.units[&UnitId::new("heater-1")]
        .ports
        .clone();
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet")
        .unwrap()
        .stream_id = None;
    document.revision += 1;
    session.refresh(&document);
    for _ in 0..2 {
        assert!(
            session
                .entries()
                .iter()
                .any(|e| e.key.rule == SpecificationRule::PressureRelationship
                    && e.state == SpecificationEntryState::Unconfirmed)
        );
        assert!(
            !session
                .changes()
                .iter()
                .any(|e| e.key.rule == SpecificationRule::PressureRelationship)
        );
        session.recheck(&document);
    }
    let heater = document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap();
    heater.ports = original_ports;
    heater.parameters.outlet_pressure_pa = Some(1.0);
    document.revision += 1;
    session.refresh(&document);
    assert!(
        session
            .changes()
            .iter()
            .any(|e| e.key.rule == SpecificationRule::PressureRelationship
                && e.state == SpecificationEntryState::Resolved)
    );
}

#[test]
fn removed_target_is_not_resolved_or_rebound_to_same_name() {
    let mut document = document();
    missing_temperature(&mut document);
    let mut session = ModelingCheckSession::new(&document);
    session.refresh(&document);
    let original = session.entries()[0].key.clone();
    let mut replacement = document
        .flowsheet
        .units
        .remove(&UnitId::new("heater-1"))
        .unwrap();
    replacement.id = UnitId::new("different-id");
    document.flowsheet.insert_unit(replacement).unwrap();
    document.revision += 1;
    session.refresh(&document);
    assert!(
        session
            .changes()
            .iter()
            .any(|e| e.key == original && e.state == SpecificationEntryState::TargetRemoved)
    );
    assert!(!original.targets[0].exists(&document));
    assert!(
        session.entries().iter().any(|e| e.key != original
            && matches!(e.problem, SpecificationProblem::MissingParameter { .. }))
    );
}

#[test]
fn repeated_refresh_and_localized_presentation_do_not_recheck_or_mutate_document() {
    let mut document = document();
    missing_temperature(&mut document);
    let mut session = ModelingCheckSession::new(&document);
    session.refresh(&document);
    let original_session = session.clone();
    let original_document = document.clone();
    for _ in 0..4 {
        session.refresh(&document);
    }
    assert_eq!(session, original_session);
    assert_eq!(document, original_document);
}
