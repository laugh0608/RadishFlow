use super::*;
use crate::engineering::rating_tests::{direct, member, session, task};
use rf_model::equipment::*;
use rf_types::units::QuantityKind;

fn second() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: member().task_id,
        case_id: "second".into(),
    }
}
fn add(s: &mut EngineeringSession) {
    let mut case = task(s).cases[0].clone();
    case.id = second().case_id;
    s.apply(vec![
        EngineeringCommand::UpgradeRatingMethod(member().task_id),
        EngineeringCommand::AddRatingCase {
            task: member().task_id,
            case: Box::new(case),
        },
    ])
    .unwrap();
}
fn update(s: &mut EngineeringSession, key: &EquipmentCaseKey, value: f64) {
    let mut c = task(s)
        .cases
        .into_iter()
        .find(|c| c.id == key.case_id)
        .unwrap();
    c.inputs.insert(
        SeparatorRatingInput::GasVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, value, "source-test"),
    );
    s.apply(vec![EngineeringCommand::UpdateRatingCase {
        task: key.task_id.clone(),
        case: Box::new(c),
    }])
    .unwrap();
}
#[test]
fn membership_is_atomic_retired_and_preserves_unmodified_rows() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let late = s.begin_rating(&member()).unwrap().execute();
    add(&mut s);
    assert_eq!(
        s.complete_calculation(late),
        Err(EngineeringError::StaleCalculation)
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(
        s.case_result_status(&second()),
        EquipmentResultStatus::Missing
    );
    s.calculate_rating(&member()).unwrap();
    s.calculate_rating(&second()).unwrap();
    let first_id = s.case_calculation_id(&member()).unwrap().clone();
    let saved_case = task(&s).cases[1].clone();
    let before_delete = s.capture_rating_matrix(&member().task_id).unwrap();
    let late = s.begin_rating(&second()).unwrap().execute();
    s.apply(vec![EngineeringCommand::DeleteRatingCase(second())])
        .unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    assert_eq!(s.case_calculation_id(&member()), Some(&first_id));
    assert_eq!(task(&s).method.version, 2);
    assert!(!s.rating_matrix_is_current(&before_delete));
    assert!(
        s.apply(vec![EngineeringCommand::DeleteRatingCase(member())])
            .is_err()
    );
    assert!(matches!(
        s.apply(vec![EngineeringCommand::AddRatingCase {
            task: member().task_id,
            case: Box::new(saved_case)
        }]),
        Err(EngineeringError::IdentityUnavailable(_))
    ));
    s.undo().unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    assert_eq!(
        s.case_result_status(&second()),
        EquipmentResultStatus::Stale
    );
    assert!(!s.rating_matrix_is_current(&before_delete));
    assert_eq!(
        s.complete_calculation(late),
        Err(EngineeringError::StaleCalculation)
    );
    s.redo().unwrap();
    assert_eq!(task(&s).cases.len(), 1);
}
#[test]
fn explicit_rows_isolate_results_and_keep_all_missing_failed_stale_members() {
    let mut s = session();
    add(&mut s);
    let empty = s.capture_rating_matrix(&member().task_id).unwrap();
    assert_eq!(empty.rows().len(), 2);
    assert!(empty.summary().gas.is_none());
    s.calculate_rating(&member()).unwrap();
    assert!(!s.rating_matrix_is_current(&empty));
    let first = s.current_rating(&member()).unwrap().clone();
    update(&mut s, &second(), -1.0);
    assert_eq!(
        s.calculate_rating(&second()).unwrap(),
        EquipmentResultStatus::Failed
    );
    assert_eq!(s.current_rating(&member()), Some(&first));
    let partial = s.capture_rating_matrix(&member().task_id).unwrap();
    assert_eq!(partial.rows()[1].status, EquipmentResultStatus::Failed);
    assert!(partial.rows()[1].failure.is_some());
    assert_eq!(
        partial.summary().coverage,
        separator_rating::MatrixCoverage::Incomplete
    );
    assert!(!partial.summary().all_cases_within_criteria());
    update(&mut s, &second(), 0.01);
    let stale = s.capture_rating_matrix(&member().task_id).unwrap();
    assert_eq!(stale.rows()[1].status, EquipmentResultStatus::Stale);
    assert!(stale.rows()[1].result.is_none() && stale.rows()[1].failure.is_none());
    s.calculate_rating(&second()).unwrap();
    assert_eq!(
        s.capture_rating_matrix(&member().task_id)
            .unwrap()
            .summary()
            .coverage,
        separator_rating::MatrixCoverage::Complete
    );
}
#[test]
fn captured_matrix_tracks_every_execution_even_unchanged_failed_or_missing_rows() {
    let mut s = session();
    add(&mut s);
    s.calculate_rating(&member()).unwrap();
    let capture = s.capture_rating_matrix(&member().task_id).unwrap();
    let late = s.begin_rating(&second()).unwrap();
    assert!(!s.rating_matrix_is_current(&capture));
    let running = s.capture_rating_matrix(&member().task_id).unwrap();
    s.complete_calculation(late.execute()).unwrap();
    assert!(!s.rating_matrix_is_current(&running));
    let all = s.capture_rating_matrix(&member().task_id).unwrap();
    s.calculate_rating(&member()).unwrap();
    assert!(!s.rating_matrix_is_current(&all));
    update(&mut s, &second(), -1.0);
    s.calculate_rating(&second()).unwrap();
    let failed = s.capture_rating_matrix(&member().task_id).unwrap();
    s.calculate_rating(&second()).unwrap();
    assert!(!s.rating_matrix_is_current(&failed));
    let reopened =
        EngineeringSession::from_inputs("rating-project".into(), s.inputs().clone()).unwrap();
    assert!(!reopened.rating_matrix_is_current(&failed));
    assert_eq!(
        reopened.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
}
#[test]
fn metadata_order_and_unrelated_edits_do_not_revoke_matrix_or_pending_row() {
    let mut s = session();
    add(&mut s);
    s.calculate_rating(&member()).unwrap();
    let capture = s.capture_rating_matrix(&member().task_id).unwrap();
    s.apply(vec![EngineeringCommand::ReorderRatingCases {
        task: member().task_id,
        cases: vec![second().case_id, member().case_id],
    }])
    .unwrap();
    let mut c = task(&s).cases[0].clone();
    c.name = "renamed".into();
    c.purpose = "metadata".into();
    s.apply(vec![EngineeringCommand::UpdateRatingCase {
        task: member().task_id,
        case: Box::new(c),
    }])
    .unwrap();
    assert!(s.rating_matrix_is_current(&capture));
    assert_eq!(capture.task().cases[0].id, member().case_id);
    assert_eq!(
        s.capture_rating_matrix(&member().task_id)
            .unwrap()
            .task()
            .cases[0]
            .id,
        second().case_id
    );
    let before = s.clone();
    assert!(
        s.apply(vec![EngineeringCommand::ReorderRatingCases {
            task: member().task_id,
            cases: vec![second().case_id, second().case_id]
        }])
        .is_err()
    );
    assert_eq!(s, before);
    let pending = s.begin_rating(&member()).unwrap().execute();
    update(&mut s, &second(), 0.02);
    assert_eq!(
        s.complete_calculation(pending).unwrap(),
        EquipmentResultStatus::Current
    );
}
#[test]
fn shared_geometry_and_source_changes_invalidate_every_dependent_row() {
    let mut s = session();
    add(&mut s);
    s.calculate_rating(&member()).unwrap();
    s.calculate_rating(&second()).unwrap();
    let before = s.capture_rating_matrix(&member().task_id).unwrap();
    let mut g = task(&s).geometry;
    g.inputs.insert(
        SeparatorGeometryInput::WorkingLiquidHeight,
        direct(QuantityKind::Length, 1.0, "drawing"),
    );
    let impacts = s
        .apply(vec![EngineeringCommand::UpdateRatingGeometry {
            task: member().task_id,
            geometry: Box::new(g),
        }])
        .unwrap();
    assert_eq!(impacts.len(), 2);
    assert!(!s.rating_matrix_is_current(&before));
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(
        s.case_result_status(&second()),
        EquipmentResultStatus::Stale
    );
    s.calculate_rating(&member()).unwrap();
    s.calculate_rating(&second()).unwrap();
    let mut source = s
        .inputs()
        .basis
        .sources
        .iter()
        .find(|s| s.id.as_str() == "source-test")
        .unwrap()
        .clone();
    source.adoption_note.push_str(" changed evidence");
    assert_eq!(
        s.apply(vec![EngineeringCommand::UpdateSource(Box::new(source))])
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(
        s.case_result_status(&second()),
        EquipmentResultStatus::Stale
    );
}
