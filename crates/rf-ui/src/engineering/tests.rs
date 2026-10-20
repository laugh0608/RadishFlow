use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::*;
use rf_unitops::separator_sizing::method;

// Public synthetic arithmetic fixture S0-B01. No proprietary engineering data.
pub(super) fn fixture() -> (SeparatorSizingTask, EngineeringBasis) {
    let source = SourceRecord {
        id: "synthetic".into(),
        revision: 1,
        kind: SourceKind::Assumed,
        title: "S0-B01 synthetic arithmetic input".into(),
        reference: String::new(),
        locator: String::new(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: "Explicit synthetic test assumption; no physical accuracy claim".into(),
        review: SourceReview::Adopted,
        applicability: SourceApplicability {
            fluid_system: "synthetic two-phase fluid".into(),
            phase: SourcePhase::Both,
            operating_conditions: Some(SourceOperatingConditions {
                temperature_k: 300.0,
                pressure_pa: 100000.0,
            }),
            scope: "Full cylinder, effective whole-vessel K, explicit working-volume criteria"
                .into(),
        },
        observations: Vec::new(),
    };
    let values = [
        300.0, 100000.0, 0.8, 0.005, 10.0, 1010.0, 0.1, 0.8, 120.0, 1.0,
    ];
    let task = SeparatorSizingTask {
        id: "task-1".into(),
        revision: 1,
        tag: "V-101".into(),
        name: "Synthetic sizing".into(),
        method: method(),
        case: SeparatorCase {
            id: "case-1".into(),
            name: "Single case".into(),
            purpose: "Arithmetic verification".into(),
            fluid_system: source.applicability.fluid_system.clone(),
            applicability_source_ids: vec![source.id.clone()],
            inputs: SeparatorInput::ALL
                .into_iter()
                .zip(values)
                .map(|(field, value)| (field, direct(field.quantity(), value)))
                .collect(),
            applicability: SeparatorApplicability {
                orientation: Some(SeparatorOrientation::Vertical),
                gas_direction: Some(SeparatorGasDirection::Upward),
                service: Some(SeparatorService::CleanNonFoamingTwoPhase),
                geometry: Some(SeparatorSectionGeometry::FullEqualDiameterCylinder),
                k_basis: Some(SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals),
                flow_basis: Some(SeparatorFlowBasis::ActualAtOperatingConditions),
                internals_description:
                    "No reduction of effective full-cylinder area; test assumption".into(),
            },
        },
        axial_sections: BTreeMap::new(),
    };
    (
        task,
        EngineeringBasis {
            sources: vec![source],
            basis_entries: Vec::new(),
        },
    )
}

fn direct(quantity: QuantityKind, value_si: f64) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id: "synthetic".into(),
    }
}
fn set(task: &mut SeparatorSizingTask, field: SeparatorInput, value: f64) {
    task.case
        .inputs
        .insert(field, direct(field.quantity(), value));
}

fn session() -> EngineeringSession {
    let (task, basis) = fixture();
    EngineeringSession::from_inputs(
        "project".into(),
        EngineeringInputs {
            basis,
            tasks: vec![task.into()],
        },
    )
    .unwrap()
}
fn id() -> EquipmentTaskId {
    "task-1".into()
}
fn task(state: &EngineeringSession) -> SeparatorSizingTask {
    state.inputs().task(&id()).unwrap().clone()
}
fn source(state: &EngineeringSession) -> SourceRecord {
    state.inputs().basis.sources[0].clone()
}
fn edit(state: &mut EngineeringSession, value: f64) {
    let mut next = task(state);
    set(&mut next, SeparatorInput::GasVolumeFlow, value);
    state
        .apply(vec![EngineeringCommand::UpdateTask(Box::new(next))])
        .unwrap();
}

#[test]
fn atomic_batch_validates_final_graph_and_rejects_partial_updates() {
    let mut state = session();
    let before = state.clone();
    let mut next = task(&state);
    next.name = "changed".into();
    let mut invalid = source(&state);
    invalid.observations.push(SourceObservation {
        id: "bad".into(),
        quantity: QuantityKind::Length,
        original_value: f64::NAN,
        original_unit: "m".into(),
        canonical_value: 1.0,
        conversion_note: String::new(),
        depends_on: vec![],
    });
    assert!(
        state
            .apply(vec![
                EngineeringCommand::UpdateTask(Box::new(next)),
                EngineeringCommand::UpdateSource(Box::new(invalid))
            ])
            .is_err()
    );
    assert_eq!(state, before);
    let mut next = task(&state);
    next.case.inputs.remove(&SeparatorInput::GasDensity);
    state
        .apply(vec![EngineeringCommand::UpdateTask(Box::new(next))])
        .unwrap();
    assert_eq!(
        state.calculate(&id()).unwrap(),
        EquipmentResultStatus::Failed
    );
    assert!(state.is_dirty()); // Incomplete formal data is permitted; calculation is blocked.
}

#[test]
fn referenced_deletion_reports_locations_and_explicit_unbind_is_one_history_step() {
    let mut state = session();
    let before = state.clone();
    let error = state
        .apply(vec![EngineeringCommand::DeleteSource("synthetic".into())])
        .unwrap_err();
    let EngineeringError::Referenced { references } = error else {
        panic!("expected referrers");
    };
    assert!(references.contains(&EngineeringReference::TaskApplicability(id())));
    assert!(references.contains(&EngineeringReference::TaskInput {
        task: id(),
        field: SeparatorInput::EffectiveK
    }));
    assert_eq!(state, before);
    let mut next = task(&state);
    next.case.inputs.clear();
    next.case.applicability_source_ids.clear();
    state
        .apply(vec![
            EngineeringCommand::DeleteSource("synthetic".into()),
            EngineeringCommand::UpdateTask(Box::new(next)),
        ])
        .unwrap();
    assert!(state.inputs().basis.sources.is_empty());
    state.undo().unwrap();
    assert!(!state.is_dirty());
    assert!(!state.can_undo());
    assert_eq!(state.inputs().basis.sources.len(), 1);
    state.redo().unwrap();
    assert_eq!(
        state.apply(vec![EngineeringCommand::CreateSource(Box::new(source(
            &before
        )))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Source("synthetic".into())
        ))
    );
}

#[test]
fn shared_basis_invalidates_all_referrers_including_override_but_not_other_task() {
    let mut state = session();
    let mut a = task(&state);
    a.case.inputs.insert(
        SeparatorInput::GasVolumeFlow,
        InputBinding::BasisRef {
            basis_entry_id: "flow".into(),
        },
    );
    let mut b = a.clone();
    b.id = "task-2".into();
    b.case.id = "case-2".into();
    b.tag = "V-102".into();
    b.case.inputs.insert(
        SeparatorInput::GasVolumeFlow,
        InputBinding::Override {
            basis_entry_id: "flow".into(),
            value: EngineeringValue {
                quantity: QuantityKind::ActualVolumeFlow,
                value_si: 0.8,
            },
            source_id: "synthetic".into(),
            reason: "explicit override".into(),
        },
    );
    let mut c = task(&state);
    c.id = "task-3".into();
    c.case.id = "case-3".into();
    c.tag = "V-103".into();
    state
        .apply(vec![
            EngineeringCommand::CreateBasis(BasisEntry {
                id: "flow".into(),
                revision: 0,
                name: "flow".into(),
                value: EngineeringValue {
                    quantity: QuantityKind::ActualVolumeFlow,
                    value_si: 0.8,
                },
                source_id: "synthetic".into(),
            }),
            EngineeringCommand::UpdateTask(Box::new(a)),
            EngineeringCommand::CreateTask(Box::new(b)),
            EngineeringCommand::CreateTask(Box::new(c)),
        ])
        .unwrap();
    for task in [id(), "task-2".into(), "task-3".into()] {
        state.calculate(&task).unwrap();
    }
    let mut basis = state.inputs().basis.basis_entries[0].clone();
    basis.value.value_si = 1.0;
    let prepared = state
        .prepare(vec![EngineeringCommand::UpdateBasis(basis)])
        .unwrap();
    assert_eq!(
        prepared
            .impacts()
            .iter()
            .map(|i| i.task_id.clone())
            .collect::<Vec<_>>(),
        vec![id(), "task-2".into()]
    );
    assert!(
        prepared
            .impacts()
            .iter()
            .all(|i| i.input_changed && !i.evidence_changed)
    );
    state.commit(prepared).unwrap();
    assert_eq!(state.result_status(&id()), EquipmentResultStatus::Stale);
    assert_eq!(
        state.result_status(&"task-2".into()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(
        state.result_status(&"task-3".into()),
        EquipmentResultStatus::Current
    );
    assert!(matches!(
        state.apply(vec![EngineeringCommand::DeleteBasis("flow".into())]),
        Err(EngineeringError::Referenced { .. })
    ));
}

#[test]
fn evidence_and_premises_advance_separate_generations_labels_do_not() {
    let mut state = session();
    state.calculate(&id()).unwrap();
    let generation = state.generation(&id()).unwrap();
    let mut next = task(&state);
    next.tag = "V-999".into();
    next.name = "Label".into();
    next.case.name = "Label".into();
    state
        .apply(vec![EngineeringCommand::UpdateTask(Box::new(next))])
        .unwrap();
    let mut src = source(&state);
    src.title = "Spelling correction".into();
    state
        .apply(vec![EngineeringCommand::UpdateSource(Box::new(src))])
        .unwrap();
    assert!(state.is_dirty());
    assert_eq!(state.generation(&id()), Some(generation));
    assert!(state.current_candidate(&id()).is_some());
    let mut src = source(&state);
    src.review = SourceReview::Reviewed;
    let impacts = state
        .apply(vec![EngineeringCommand::UpdateSource(Box::new(src))])
        .unwrap();
    assert!(!impacts[0].input_changed);
    assert!(impacts[0].evidence_changed);
    assert!(state.current_candidate(&id()).is_none());
    assert_eq!(
        state.staleness(&id()),
        Some(ResultStaleness {
            input_changed: false,
            evidence_changed: true,
            task_missing: false,
        })
    );
    let mut src = source(&state);
    src.applicability.fluid_system = "other premise".into();
    let impacts = state
        .apply(vec![EngineeringCommand::UpdateSource(Box::new(src))])
        .unwrap();
    assert!(impacts[0].input_changed && impacts[0].evidence_changed);
    assert_eq!(
        state.generation(&id()),
        Some(ResultGeneration {
            input: generation.input + 1,
            evidence: generation.evidence + 2
        })
    );
}

#[test]
fn transitive_source_evidence_is_tracked_and_referenced_source_cannot_be_deleted() {
    let mut state = session();
    let observation = |name: &str| SourceObservation {
        id: name.into(),
        quantity: QuantityKind::Velocity,
        original_value: 0.1,
        original_unit: "m/s".into(),
        canonical_value: 0.1,
        conversion_note: String::new(),
        depends_on: vec![],
    };
    let mut upstream = source(&state);
    upstream.id = "upstream".into();
    upstream.observations = vec![observation("raw")];
    let mut derived = source(&state);
    derived.observations = vec![observation("derived")];
    derived.observations[0].depends_on = vec![SourceObservationRef {
        source_id: upstream.id.clone(),
        observation_id: "raw".into(),
    }];
    state
        .apply(vec![
            EngineeringCommand::UpdateSource(Box::new(derived)),
            EngineeringCommand::CreateSource(Box::new(upstream)),
        ])
        .unwrap();
    state.calculate(&id()).unwrap();
    let error = state
        .apply(vec![EngineeringCommand::DeleteSource("upstream".into())])
        .unwrap_err();
    assert!(
        matches!(error, EngineeringError::Referenced { references } if references.contains(&EngineeringReference::SourceObservation { source: "synthetic".into(), observation: "derived".into() }))
    );
    let mut upstream = state.inputs().basis.sources[1].clone();
    upstream.locator = "page 2".into();
    let impacts = state
        .apply(vec![EngineeringCommand::UpdateSource(Box::new(upstream))])
        .unwrap();
    assert_eq!(
        impacts,
        vec![TaskImpact {
            task_id: id(),
            case_id: "case-1".into(),
            input_changed: false,
            evidence_changed: true
        }]
    );
    assert!(state.current_candidate(&id()).is_none());
}

#[test]
fn undo_redo_restore_inputs_but_never_revive_results_or_reuse_generations() {
    let mut state = session();
    state.calculate(&id()).unwrap();
    let old_request = state.begin_calculation(&id()).unwrap();
    let initial_revision = task(&state).revision;
    edit(&mut state, 1.0);
    let edited = state.generation(&id()).unwrap();
    assert_eq!(
        state.staleness(&id()),
        Some(ResultStaleness {
            input_changed: true,
            evidence_changed: false,
            task_missing: false,
        })
    );
    state.undo().unwrap();
    let undone = state.generation(&id()).unwrap();
    assert!(!state.is_dirty());
    assert!(undone.input > edited.input);
    assert!(task(&state).revision > initial_revision);
    assert_eq!(state.result_status(&id()), EquipmentResultStatus::Stale);
    assert_eq!(
        state.complete_calculation(old_request.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    let unchanged = task(&state);
    state
        .apply(vec![EngineeringCommand::UpdateTask(Box::new(unchanged))])
        .unwrap();
    assert!(state.can_redo());
    state.redo().unwrap();
    assert!(state.generation(&id()).unwrap().input > undone.input);
    state.undo().unwrap();
    edit(&mut state, 1.2);
    assert!(!state.can_redo());
    assert!(state.current_candidate(&id()).is_none());
}

#[test]
fn retired_ids_and_case_ids_survive_delete_undo_and_history_branching() {
    let mut state = session();
    let original = task(&state);
    state
        .apply(vec![EngineeringCommand::DeleteTask(id())])
        .unwrap();
    assert_eq!(
        state.apply(vec![EngineeringCommand::CreateTask(Box::new(
            original.clone()
        ))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Task(id())
        ))
    );
    let mut replacement = original.clone();
    replacement.id = "replacement".into();
    assert_eq!(
        state.apply(vec![EngineeringCommand::CreateTask(Box::new(replacement))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Case("case-1".into())
        ))
    );
    state.undo().unwrap();
    let mut new = original;
    new.id = "new".into();
    new.case.id = "new-case".into();
    new.tag = "NEW".into();
    state
        .apply(vec![EngineeringCommand::CreateTask(Box::new(new.clone()))])
        .unwrap();
    state.undo().unwrap();
    edit(&mut state, 1.5);
    assert_eq!(
        state.apply(vec![EngineeringCommand::CreateTask(Box::new(new))]),
        Err(EngineeringError::IdentityUnavailable(
            EngineeringRecordId::Task("new".into())
        ))
    );
}

#[test]
fn preparation_rejects_stale_foreign_and_divergent_sessions() {
    let mut state = session();
    let mut next = task(&state);
    next.name = "new name".into();
    let prepare = |s: &EngineeringSession| {
        s.prepare(vec![EngineeringCommand::UpdateTask(Box::new(next.clone()))])
            .unwrap()
    };
    let prepared = prepare(&state);
    let foreign = prepare(&state);
    assert_eq!(
        session().commit(foreign),
        Err(EngineeringError::StalePreparation)
    );
    let mut branch = state.clone();
    edit(&mut branch, 1.3);
    assert_eq!(
        branch.commit(prepare(&state)),
        Err(EngineeringError::StalePreparation)
    );
    edit(&mut state, 1.2);
    state.undo().unwrap();
    assert_eq!(
        state.commit(prepared),
        Err(EngineeringError::StalePreparation)
    );
}

#[test]
fn calculations_reject_late_out_of_order_cleared_and_reopened_completions() {
    let mut state = session();
    let first = state.begin_calculation(&id()).unwrap();
    let second = state.begin_calculation(&id()).unwrap();
    assert_eq!(
        state.complete_calculation(first.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    assert_eq!(
        state.complete_calculation(second.execute()).unwrap(),
        EquipmentResultStatus::Current
    );
    let old_id = state.calculation_id(&id()).unwrap().clone();
    state.calculate(&id()).unwrap();
    assert_ne!(state.calculation_id(&id()), Some(&old_id));
    let late = state.begin_calculation(&id()).unwrap();
    edit(&mut state, 1.1);
    assert_eq!(
        state.complete_calculation(late.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    let cleared = state.begin_calculation(&id()).unwrap();
    let before = state.checkpoint();
    state.clear_results();
    assert_eq!(state.checkpoint(), before);
    assert_eq!(
        state.complete_calculation(cleared.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    let old = state.begin_calculation(&id()).unwrap();
    let mut reopened =
        EngineeringSession::from_inputs("project".into(), state.inputs().clone()).unwrap();
    assert_eq!(
        reopened.result_status(&id()),
        EquipmentResultStatus::Missing
    );
    assert_eq!(
        reopened.complete_calculation(old.execute()),
        Err(EngineeringError::StaleCalculation)
    );
}

#[test]
fn failure_replaces_previous_candidate_and_undo_requires_explicit_recalculation() {
    let mut state = session();
    state.calculate(&id()).unwrap();
    edit(&mut state, -1.0);
    assert!(state.is_dirty()); // Formal finite input, calculation error.
    assert_eq!(
        state.calculate(&id()).unwrap(),
        EquipmentResultStatus::Failed
    );
    assert!(state.current_candidate(&id()).is_none());
    assert!(state.current_failure(&id()).is_some());
    state.undo().unwrap();
    assert_eq!(state.result_status(&id()), EquipmentResultStatus::Stale);
    assert!(state.current_candidate(&id()).is_none());
    assert_eq!(
        state.calculate(&id()).unwrap(),
        EquipmentResultStatus::Current
    );
}

#[test]
fn counter_overflow_is_atomic_and_revision_conflicts_and_duplicate_tags_are_rejected() {
    let mut state = session();
    state
        .generations
        .get_mut(&EquipmentCaseKey {
            task_id: id(),
            case_id: task(&state).case.id,
        })
        .unwrap()
        .input = u64::MAX;
    let before = state.clone();
    let mut next = task(&state);
    set(&mut next, SeparatorInput::GasVolumeFlow, 1.1);
    assert_eq!(
        state.apply(vec![EngineeringCommand::UpdateTask(Box::new(next))]),
        Err(EngineeringError::CounterExhausted)
    );
    assert_eq!(state, before);
    let mut next = task(&state);
    next.revision += 1;
    assert_eq!(
        state.apply(vec![EngineeringCommand::UpdateTask(Box::new(next))]),
        Err(EngineeringError::RevisionConflict {
            record: EngineeringRecordId::Task(id()),
            expected: task(&state).revision,
            supplied: task(&state).revision + 1
        })
    );
    let mut next = task(&state);
    next.case.id = "changed".into();
    assert_eq!(
        state.apply(vec![EngineeringCommand::UpdateTask(Box::new(next))]),
        Err(EngineeringError::CaseIdentityChanged(id()))
    );
    let mut duplicate = task(&state);
    duplicate.id = "duplicate".into();
    duplicate.case.id = "duplicate-case".into();
    duplicate.tag = " v-101 ".into();
    assert!(matches!(
        state.apply(vec![EngineeringCommand::CreateTask(Box::new(duplicate))]),
        Err(EngineeringError::InvalidIdentity(_))
    ));
    assert_eq!(state, before);
}

#[test]
fn record_revision_overflow_is_atomic() {
    let (mut task, basis) = fixture();
    task.revision = u64::MAX;
    let mut state = EngineeringSession::from_inputs(
        "project".into(),
        EngineeringInputs {
            basis,
            tasks: vec![task.into()],
        },
    )
    .unwrap();
    let before = state.clone();
    let mut next = super::tests::task(&state);
    next.name = "label".into();
    assert_eq!(
        state.apply(vec![EngineeringCommand::UpdateTask(Box::new(next))]),
        Err(EngineeringError::CounterExhausted)
    );
    assert_eq!(state, before);
}

#[test]
fn equipment_history_and_calculation_are_isolated_from_flow_and_presentation() {
    use crate::{
        AppState, CanvasPoint, DiagnosticSeverity, DiagnosticSummary, DocumentCommand,
        DocumentMetadata, FlowsheetDocument, ProjectPresentationCommand, RunStatus, SolveSnapshot,
    };
    let document = FlowsheetDocument::new(
        rf_model::Flowsheet::new("flow"),
        DocumentMetadata::new("project", "Project", std::time::UNIX_EPOCH),
    );
    let mut app = AppState::new(document);
    app.workspace.engineering = session();
    app.mark_saved("memory.rfproj.json");
    app.store_snapshot(SolveSnapshot::new(
        "snapshot",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "ok"),
    ));
    let flow_before = app.workspace.solve_session.clone();
    let history_before = app.workspace.command_history.clone();
    edit(&mut app.workspace.engineering, 1.1);
    app.workspace.engineering.calculate(&id()).unwrap();
    assert_eq!(app.workspace.solve_session, flow_before);
    assert_eq!(app.workspace.command_history, history_before);
    assert_eq!(app.workspace.document.revision, 0);
    assert!(crate::latest_snapshot(&app.workspace).is_some());
    let save = app.workspace.project_save_state();
    assert!(save.engineering_dirty && save.needs_close_confirmation());
    assert!(!save.document_dirty && !save.presentation_dirty);
    app.mark_saved("memory.rfproj.json");
    assert!(app.workspace.project_save_state().engineering_dirty);
    let engine_before = app.workspace.engineering.clone();
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ));
    app.commit_document_change(
        DocumentCommand::MoveUnit {
            unit_id: "unrelated".into(),
            position: CanvasPoint::new(1.0, 2.0),
        },
        rf_model::Flowsheet::new("changed"),
        std::time::SystemTime::now(),
    );
    app.store_snapshot(SolveSnapshot::new(
        "new-snapshot",
        1,
        2,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "ok"),
    ));
    assert_eq!(app.workspace.engineering, engine_before);
    assert!(app.workspace.engineering.current_candidate(&id()).is_some());
}

#[test]
fn divergent_clones_with_equal_generations_cannot_accept_each_others_completion() {
    let mut a = session();
    let mut b = a.clone();
    edit(&mut a, 1.1);
    edit(&mut b, 1.2);
    assert_eq!(a.generation(&id()), b.generation(&id()));
    let request = a.begin_calculation(&id()).unwrap();
    b.begin_calculation(&id()).unwrap();
    assert_eq!(
        b.complete_calculation(request.execute()),
        Err(EngineeringError::StaleCalculation)
    );
}

#[test]
fn method_change_and_delete_restore_do_not_allow_previous_execution() {
    let mut state = session();
    state.calculate(&id()).unwrap();
    let request = state.begin_calculation(&id()).unwrap();
    let mut changed = task(&state);
    changed.method.version += 1;
    state
        .apply(vec![EngineeringCommand::UpdateTask(Box::new(changed))])
        .unwrap();
    assert_eq!(
        state.complete_calculation(request.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    assert_eq!(
        state.calculate(&id()).unwrap(),
        EquipmentResultStatus::Failed
    );
    state.undo().unwrap();
    state.calculate(&id()).unwrap();
    let request = state.begin_calculation(&id()).unwrap();
    state
        .apply(vec![EngineeringCommand::DeleteTask(id())])
        .unwrap();
    assert!(state.current_candidate(&id()).is_none());
    state.undo().unwrap();
    assert_eq!(
        state.complete_calculation(request.execute()),
        Err(EngineeringError::StaleCalculation)
    );
    assert_eq!(state.result_status(&id()), EquipmentResultStatus::Stale);
}

#[test]
fn save_acknowledges_captured_inputs_and_rejects_foreign_session_without_losing_history() {
    let mut state = session();
    edit(&mut state, 1.1);
    let written = state.capture_save();
    edit(&mut state, 1.2);
    state.record_saved(written).unwrap();
    assert!(state.is_dirty());
    state.undo().unwrap();
    assert!(!state.is_dirty());
    state.redo().unwrap();
    assert!(state.is_dirty());
    let foreign = state.capture_save();
    let mut reopened = session();
    let before = reopened.clone();
    assert_eq!(
        reopened.record_saved(foreign),
        Err(EngineeringError::StalePreparation)
    );
    assert_eq!(reopened, before);
}
