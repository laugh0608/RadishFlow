use super::*;

#[test]
fn revocation_is_exact_and_cannot_erase_a_newer_or_foreign_request() {
    let mut session = super::rating_tests::session();
    let member = super::rating_tests::member();
    let old = session.begin_rating(&member).unwrap();
    let old_key = old.qualification();
    let current = session.begin_rating(&member).unwrap();
    let current_key = current.qualification();
    assert!(!session.revoke_calculation(&old_key));
    assert!(session.calculation_is_current(&current_key));
    assert!(matches!(
        session.complete_calculation(old.execute()),
        Err(EngineeringError::StaleCalculation)
    ));
    let mut reopened = super::rating_tests::session();
    let foreign = reopened.begin_rating(&member).unwrap();
    assert!(!reopened.calculation_belongs_to_session(&current_key));
    assert!(!reopened.revoke_calculation(&current_key));
    assert!(reopened.calculation_is_current(&foreign.qualification()));
    assert!(session.revoke_calculation(&current_key));
    assert!(!session.calculation_is_current(&current_key));
    assert!(matches!(
        session.complete_calculation(current.execute()),
        Err(EngineeringError::StaleCalculation)
    ));
}

#[test]
fn cancelling_sizing_keeps_previous_candidate_without_restoring_rating_eligibility() {
    let (task, basis) = super::tests::fixture();
    let id = task.id.clone();
    let mut sizing = EngineeringSession::from_inputs(
        "sizing".into(),
        EngineeringInputs {
            basis,
            tasks: vec![task.into()],
        },
    )
    .unwrap();
    sizing.calculate(&id).unwrap();
    let previous = sizing.current_candidate(&id).cloned();
    let request = sizing.begin_calculation(&id).unwrap();
    assert_eq!(sizing.current_candidate(&id), previous.as_ref());
    assert!(sizing.revoke_calculation(&request.qualification()));
    assert!(matches!(
        sizing.complete_calculation(request.execute()),
        Err(EngineeringError::StaleCalculation)
    ));
    assert_eq!(sizing.current_candidate(&id), previous.as_ref());
    let mut rating = super::rating_tests::session();
    let member = super::rating_tests::member();
    rating.calculate_rating(&member).unwrap();
    let request = rating.begin_rating(&member).unwrap();
    assert_eq!(
        rating.case_result_status(&member),
        EquipmentResultStatus::Stale
    );
    assert!(rating.revoke_calculation(&request.qualification()));
    assert!(matches!(
        rating.complete_calculation(request.execute()),
        Err(EngineeringError::StaleCalculation)
    ));
    assert_eq!(
        rating.case_result_status(&member),
        EquipmentResultStatus::Stale
    );
}
