use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::QuantityKind;
use rf_unitops::separator_rating::ConstraintStatus;

pub(super) fn session() -> EngineeringSession {
    let rf_store::project_v3::ProjectV3Read::Editable(p) = rf_store::project_v3::parse(
        include_str!("../../../rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!("rating fixture")
    };
    let e = p.engineering.unwrap();
    EngineeringSession::from_inputs(
        "rating-project".into(),
        EngineeringInputs {
            basis: e.basis,
            tasks: e.tasks,
        },
    )
    .unwrap()
}
pub(super) fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }
}
pub(super) fn task(state: &EngineeringSession) -> SeparatorRatingTask {
    state
        .inputs()
        .rating_task(&member().task_id)
        .unwrap()
        .clone()
}
pub(super) fn direct(quantity: QuantityKind, value_si: f64, source: &str) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id: source.into(),
    }
}
fn update_flow(state: &mut EngineeringSession, value: f64) {
    let mut c = task(state).cases.remove(0);
    c.inputs.insert(
        SeparatorRatingInput::GasVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, value, "source-test"),
    );
    state
        .apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(c),
        }])
        .unwrap();
}

#[test]
fn exceeding_rating_is_current_and_cannot_be_read_as_a_sizing_candidate() {
    let mut s = session();
    assert!(!s.is_dirty());
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert_eq!(
        s.calculate_rating(&member()).unwrap(),
        EquipmentResultStatus::Current
    );
    let r = s.current_rating(&member()).unwrap();
    assert_eq!(r.performance().conclusion(), ConstraintStatus::Exceeded);
    assert!(r.performance().gas.margin_fraction < 0.0);
    assert!(s.current_candidate(&member().task_id).is_none());
    assert!(!s.is_dirty());
    assert!(!s.can_undo());
}

#[test]
fn geometry_case_and_task_revisions_advance_independently_and_history_never_restores_eligibility() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let original = task(&s);
    let mut geometry = original.geometry.clone();
    geometry.inputs.insert(
        SeparatorGeometryInput::WorkingLiquidHeight,
        direct(QuantityKind::Length, 0.7, "drawing"),
    );
    let impact = s
        .apply(vec![EngineeringCommand::UpdateRatingGeometry {
            task: member().task_id,
            geometry: Box::new(geometry),
        }])
        .unwrap();
    assert_eq!(impact.len(), 1);
    assert_eq!(impact[0].case_id, member().case_id);
    assert!(impact[0].input_changed);
    let after_geometry = task(&s);
    assert!(after_geometry.revision > original.revision);
    assert!(after_geometry.geometry.revision > original.geometry.revision);
    assert_eq!(after_geometry.cases[0].revision, original.cases[0].revision);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    s.calculate_rating(&member()).unwrap();
    update_flow(&mut s, 0.7);
    let after_case = task(&s);
    assert!(after_case.cases[0].revision > after_geometry.cases[0].revision);
    assert_eq!(
        after_case.geometry.revision,
        after_geometry.geometry.revision
    );
    s.undo().unwrap();
    assert!(task(&s).cases[0].revision > after_case.cases[0].revision);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    s.undo().unwrap();
    assert!(!s.is_dirty());
    assert!(task(&s).geometry.revision > after_geometry.geometry.revision);
    assert!(s.current_rating(&member()).is_none());
    s.redo().unwrap();
    assert!(s.is_dirty());
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn labels_update_persisted_revisions_without_invalidating_current_rating() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let id = s.case_calculation_id(&member()).unwrap().clone();
    let generation = s.case_generation(&member()).unwrap();
    let original = task(&s);
    assert!(
        s.apply(vec![EngineeringCommand::UpdateRatingLabels {
            task: member().task_id,
            revision: original.revision,
            tag: "V-203".into(),
            name: "Renamed".into(),
        }])
        .unwrap()
        .is_empty()
    );
    let mut case = task(&s).cases.remove(0);
    case.name = "Renamed case".into();
    case.purpose = "Updated description".into();
    assert!(
        s.apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(case)
        }])
        .unwrap()
        .is_empty()
    );
    assert_eq!(s.case_generation(&member()), Some(generation));
    assert_eq!(s.case_calculation_id(&member()), Some(&id));
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    assert_eq!(
        s.current_rating(&member()).unwrap().capture().task,
        original
    );
    assert!(s.is_dirty());
}

#[test]
fn stale_geometry_and_case_updates_fail_atomically_without_changing_other_records() {
    let mut s = session();
    let stale = task(&s);
    update_flow(&mut s, 0.7);
    let before = s.clone();
    let mut geometry = task(&s).geometry;
    geometry.inputs.clear();
    assert!(matches!(
        s.apply(vec![
            EngineeringCommand::UpdateRatingGeometry {
                task: member().task_id.clone(),
                geometry: Box::new(geometry)
            },
            EngineeringCommand::UpdateRatingCase {
                task: member().task_id,
                case: Box::new(stale.cases[0].clone())
            },
        ]),
        Err(EngineeringError::RevisionConflict {
            record: EngineeringRecordId::Case(_),
            ..
        })
    ));
    assert_eq!(s, before);
    let mut invalid = task(&s).geometry;
    invalid.revision += 1;
    assert!(matches!(
        s.apply(vec![EngineeringCommand::UpdateRatingGeometry {
            task: member().task_id,
            geometry: Box::new(invalid)
        }]),
        Err(EngineeringError::RevisionConflict {
            record: EngineeringRecordId::Geometry(_),
            ..
        })
    ));
    assert_eq!(s, before);
    let prepared = s
        .prepare(vec![EngineeringCommand::DeleteTask(member().task_id)])
        .unwrap();
    update_flow(&mut s, 0.6);
    let before = s.clone();
    assert_eq!(s.commit(prepared), Err(EngineeringError::StalePreparation));
    assert_eq!(s, before);
}

#[test]
fn rerun_edit_reopen_clear_and_divergent_session_reject_late_calculations() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let old = s.begin_rating(&member()).unwrap();
    assert!(s.current_rating(&member()).is_none());
    let current = s.begin_rating(&member()).unwrap();
    assert_eq!(
        s.complete_calculation(old.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    s.complete_calculation(current.execute()).unwrap();
    let late = s.begin_rating(&member()).unwrap();
    update_flow(&mut s, 0.7);
    assert_eq!(
        s.complete_calculation(late.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    let late = s.begin_rating(&member()).unwrap();
    let mut reopened =
        EngineeringSession::from_inputs("rating-project".into(), s.inputs().clone()).unwrap();
    assert_eq!(
        reopened.complete_calculation(late.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    assert_eq!(
        reopened.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    let late = s.begin_rating(&member()).unwrap();
    s.clear_results();
    assert_eq!(
        s.complete_calculation(late.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    let late = s.begin_rating(&member()).unwrap();
    let mut divergent = s.clone();
    update_flow(&mut divergent, 0.9);
    assert_eq!(
        divergent.complete_calculation(late.execute()),
        Err(EngineeringError::StaleCalculation)
    );
}

#[test]
fn failure_replaces_success_and_history_does_not_resurrect_it() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    update_flow(&mut s, 0.0);
    assert_eq!(
        s.calculate_rating(&member()).unwrap(),
        EquipmentResultStatus::Failed
    );
    assert!(s.current_rating(&member()).is_none());
    assert!(s.current_rating_failure(&member()).is_some());
    s.undo().unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert!(s.current_rating(&member()).is_none());
    assert_eq!(
        s.calculate_rating(&member()).unwrap(),
        EquipmentResultStatus::Current
    );
}

#[test]
fn related_evidence_invalidates_all_consumers_while_local_inputs_only_affect_their_case() {
    let mut s = session();
    let mut second = task(&s);
    second.id = "rating-2".into();
    second.tag = "V-103".into();
    second.cases[0].id = "rating-case-2".into();
    let other = EquipmentCaseKey {
        task_id: second.id.clone(),
        case_id: second.cases[0].id.clone(),
    };
    s.apply(vec![EngineeringCommand::CreateRatingTask(Box::new(second))])
        .unwrap();
    s.calculate_rating(&member()).unwrap();
    s.calculate_rating(&other).unwrap();
    update_flow(&mut s, 0.7);
    assert_eq!(s.case_result_status(&other), EquipmentResultStatus::Current);
    let mut drawing = s
        .inputs()
        .basis
        .sources
        .iter()
        .find(|r| r.id.as_str() == "drawing")
        .unwrap()
        .clone();
    drawing.locator = "Updated drawing evidence".into();
    let impacts = s
        .apply(vec![EngineeringCommand::UpdateSource(Box::new(drawing))])
        .unwrap();
    assert_eq!(impacts.len(), 2);
    assert!(
        impacts
            .iter()
            .all(|i| i.evidence_changed && !i.input_changed)
    );
    assert_eq!(s.case_result_status(&other), EquipmentResultStatus::Stale);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn override_original_basis_and_transitive_observation_dependencies_remain_tracked() {
    let mut s = session();
    let mut drawing = s
        .inputs()
        .basis
        .sources
        .iter()
        .find(|r| r.id.as_str() == "drawing")
        .unwrap()
        .clone();
    drawing.observations.push(SourceObservation {
        id: "derived-length".into(),
        quantity: QuantityKind::Length,
        original_value: 600.,
        original_unit: "mm".into(),
        canonical_value: 0.6,
        conversion_note: "Synthetic provenance dependency only".into(),
        depends_on: vec![SourceObservationRef {
            source_id: "source-test".into(),
            observation_id: "raw-flow".into(),
        }],
    });
    s.apply(vec![EngineeringCommand::UpdateSource(Box::new(drawing))])
        .unwrap();
    s.calculate_rating(&member()).unwrap();
    let mut original = s
        .inputs()
        .basis
        .basis_entries
        .iter()
        .find(|r| r.id.as_str() == "basis-diameter")
        .unwrap()
        .clone();
    original.value.value_si = 0.75; // Overridden, but original evidence remains part of captured provenance.
    assert!(
        !s.apply(vec![EngineeringCommand::UpdateBasis(original)])
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    s.calculate_rating(&member()).unwrap();
    let mut source = s.inputs().basis.sources[0].clone();
    source.observations[0].conversion_note = "Revised source note".into();
    assert!(
        !s.apply(vec![EngineeringCommand::UpdateSource(Box::new(source))])
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn referenced_sources_and_basis_cannot_be_deleted_and_missing_bindings_are_structured() {
    let mut s = session();
    let before = s.clone();
    let error = s
        .apply(vec![EngineeringCommand::DeleteSource("drawing".into())])
        .unwrap_err();
    let EngineeringError::Referenced { references } = error else {
        panic!("references expected")
    };
    assert!(
        references.contains(&EngineeringReference::RatingGeometryApplicability(
            member().task_id.clone()
        ))
    );
    assert!(
        references.contains(&EngineeringReference::RatingGeometryInput {
            task: member().task_id.clone(),
            field: SeparatorGeometryInput::WorkingLiquidHeight
        })
    );
    assert_eq!(s, before);
    assert!(matches!(
        s.apply(vec![EngineeringCommand::DeleteBasis("basis-time".into())]),
        Err(EngineeringError::Referenced { .. })
    ));
    let mut case = task(&s).cases.remove(0);
    case.inputs.insert(
        SeparatorRatingInput::RequiredWorkingTime,
        direct(QuantityKind::Length, 1.0, "source-test"),
    );
    assert!(matches!(
        s.apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(case)
        }]),
        Err(EngineeringError::InvalidBinding {
            reference: EngineeringReference::RatingCaseInput { .. },
            ..
        })
    ));
    assert_eq!(s, before);
}

#[test]
fn removed_task_and_case_identities_stay_retired_and_restored_records_receive_new_revisions() {
    let mut s = session();
    let original = task(&s);
    s.calculate_rating(&member()).unwrap();
    s.apply(vec![EngineeringCommand::DeleteTask(member().task_id)])
        .unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    let mut replacement = original.clone();
    replacement.id = "other-task".into();
    assert!(matches!(
        s.apply(vec![EngineeringCommand::CreateRatingTask(Box::new(
            replacement
        ))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Case(_)
        ))
    ));
    s.undo().unwrap();
    assert!(task(&s).revision > original.revision);
    assert!(task(&s).geometry.revision > original.geometry.revision);
    assert!(task(&s).cases[0].revision > original.cases[0].revision);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn unsupported_case_membership_and_method_are_rejected_before_formal_publication() {
    let s = session();
    for count in [0, 2] {
        let mut inputs = s.inputs().clone();
        let EquipmentTask::SeparatorRating(t) = &mut inputs.tasks[0] else {
            unreachable!()
        };
        if count == 0 {
            t.cases.clear();
        } else {
            let mut second = t.cases[0].clone();
            second.id = "another-case".into();
            t.cases.push(second);
        }
        assert!(EngineeringSession::from_inputs("test".into(), inputs).is_err());
    }
    let mut inputs = s.inputs().clone();
    let EquipmentTask::SeparatorRating(t) = &mut inputs.tasks[0] else {
        unreachable!()
    };
    t.method.version = 2;
    assert!(matches!(
        EngineeringSession::from_inputs("test".into(), inputs),
        Err(EngineeringError::UnsupportedRatingMethod(_))
    ));
    let mut s = s;
    let before = s.clone();
    let mut case = task(&s).cases.remove(0);
    case.id = "replacement-case".into();
    assert!(
        s.apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(case)
        }])
        .is_err()
    );
    assert_eq!(s, before);
}

#[test]
fn successful_save_acknowledges_only_captured_inputs_and_preserves_history_and_current_results() {
    let mut s = session();
    update_flow(&mut s, 0.7);
    s.calculate_rating(&member()).unwrap();
    let saved = s.capture_save();
    update_flow(&mut s, 0.6);
    s.record_saved(saved).unwrap();
    assert!(s.is_dirty());
    assert!(s.can_undo());
    s.undo().unwrap();
    assert!(!s.is_dirty());
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    s.calculate_rating(&member()).unwrap();
    s.record_saved(s.capture_save()).unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    let mut reopened =
        EngineeringSession::from_inputs("rating-project".into(), s.inputs().clone()).unwrap();
    assert!(reopened.record_saved(s.capture_save()).is_err());
    assert!(!reopened.can_undo());
    assert_eq!(
        reopened.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
}

#[test]
fn mixed_sizing_and_rating_share_identity_rules_but_keep_results_and_impacts_separate() {
    let mut s = session();
    let (sizing, basis) = super::tests::fixture();
    let sizing_id = sizing.id.clone();
    s.apply(vec![
        EngineeringCommand::CreateSource(Box::new(basis.sources[0].clone())),
        EngineeringCommand::CreateTask(Box::new(sizing.clone())),
    ])
    .unwrap();
    s.calculate(&sizing_id).unwrap();
    s.calculate_rating(&member()).unwrap();
    update_flow(&mut s, 0.7);
    assert_eq!(s.result_status(&sizing_id), EquipmentResultStatus::Current);
    assert!(s.current_candidate(&sizing_id).is_some());
    let before = s.clone();
    let mut duplicate = sizing;
    duplicate.id = "different-task".into();
    duplicate.tag = "V-999".into();
    duplicate.case.id = member().case_id;
    assert!(matches!(
        s.apply(vec![EngineeringCommand::CreateTask(Box::new(duplicate))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Case(_)
        ))
    ));
    assert_eq!(s, before);
}

#[test]
fn geometry_and_case_revision_overflow_do_not_partially_publish_inputs_or_history() {
    for geometry in [true, false] {
        let mut s = session();
        s.calculate_rating(&member()).unwrap();
        let id = if geometry {
            EngineeringRecordId::Geometry(member().task_id)
        } else {
            EngineeringRecordId::Case(member().case_id)
        };
        s.revisions.insert(id, u64::MAX);
        let before = s.clone();
        let t = task(&s);
        let command = if geometry {
            let mut g = t.geometry;
            g.inputs.clear();
            EngineeringCommand::UpdateRatingGeometry {
                task: member().task_id,
                geometry: Box::new(g),
            }
        } else {
            let mut c = t.cases[0].clone();
            c.purpose = "Changed".into();
            EngineeringCommand::UpdateRatingCase {
                task: member().task_id,
                case: Box::new(c),
            }
        };
        assert_eq!(
            s.apply(vec![command]),
            Err(EngineeringError::CounterExhausted)
        );
        assert_eq!(s, before);
    }
}
