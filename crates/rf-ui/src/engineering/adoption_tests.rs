use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::QuantityKind;
use rf_unitops::separator_rating::ConstraintStatus;

fn session() -> EngineeringSession {
    let (task, basis) = tests::fixture();
    let mut session = EngineeringSession::from_inputs(
        "doc".into(),
        EngineeringInputs {
            basis,
            tasks: vec![task.into()],
        },
    )
    .unwrap();
    session.calculate(&"task-1".into()).unwrap();
    session
}
fn request(copy_case: bool) -> GeometryAdoptionRequest {
    GeometryAdoptionRequest {
        source_task: "task-1".into(),
        target: GeometryAdoptionTarget::New {
            task_id: "adopted".into(),
            case_id: "adopted-case".into(),
            tag: "V-102".into(),
            name: "Adopted geometry".into(),
            copy_case,
        },
        adjustments: GeometryAdjustments::default(),
    }
}
fn adopt(session: &mut EngineeringSession, copy: bool) {
    let prepared = session.prepare_geometry_adoption(request(copy)).unwrap();
    session.commit_geometry_adoption(prepared).unwrap();
}
fn rating(session: &EngineeringSession) -> &SeparatorRatingTask {
    session.inputs().rating_task(&"adopted".into()).unwrap()
}
fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "adopted".into(),
        case_id: "adopted-case".into(),
    }
}

#[test]
fn adoption_is_one_atomic_history_step_and_boundary_rating_is_expected() {
    let mut s = session();
    let before = s.inputs().clone();
    let prepared = s.prepare_geometry_adoption(request(true)).unwrap();
    assert_eq!(s.inputs(), &before); // Dropping a preview is a complete cancellation.
    assert!(prepared.previous_geometry().is_none());
    s.commit_geometry_adoption(prepared).unwrap();
    assert_eq!(rating(&s).method.version, 3);
    assert_eq!(rating(&s).cases[0].inputs.len(), 9);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert!(s.is_dirty());
    s.calculate_rating(&member()).unwrap();
    assert_eq!(
        s.current_rating(&member())
            .unwrap()
            .performance()
            .conclusion(),
        ConstraintStatus::BoundaryReview
    );
    s.undo().unwrap();
    assert!(s.inputs().rating_task(&member().task_id).is_none());
    assert_eq!(s.inputs().basis.sources.len(), before.basis.sources.len());
    s.redo().unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert!(rating(&s).geometry.adoption.is_some());
}

#[test]
fn blank_case_is_savable_and_new_identity_collision_has_no_partial_sources() {
    let mut s = session();
    adopt(&mut s, false);
    assert!(rating(&s).cases[0].inputs.is_empty());
    assert!(rating(&s).cases[0].applicability_source_ids.is_empty());
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    let before = s.clone();
    assert!(s.prepare_geometry_adoption(request(true)).is_err());
    assert_eq!(s, before);
}

#[test]
fn pending_identical_rerun_foreign_session_and_changed_target_revoke_previews() {
    let mut s = session();
    let prepared = s.prepare_geometry_adoption(request(true)).unwrap();
    let rerun = s.begin_calculation(&"task-1".into()).unwrap();
    assert!(s.prepare_geometry_adoption(request(true)).is_err());
    assert!(s.commit_geometry_adoption(prepared.clone()).is_err());
    s.complete_calculation(rerun.execute()).unwrap();
    assert!(s.commit_geometry_adoption(prepared).is_err());
    let prepared = s.prepare_geometry_adoption(request(true)).unwrap();
    let mut foreign = session();
    assert!(foreign.commit_geometry_adoption(prepared.clone()).is_err());
    let mut source = s.inputs().basis.sources[0].clone();
    source.title.push_str(" changed");
    s.apply(vec![EngineeringCommand::UpdateSource(Box::new(source))])
        .unwrap();
    assert!(s.commit_geometry_adoption(prepared).is_err());
    assert!(s.inputs().rating_task(&"adopted".into()).is_none());
}

#[test]
fn source_basis_edits_deletion_and_recalculation_do_not_mutate_adopted_inputs_or_evidence() {
    let mut s = session();
    let mut source = s.inputs().task(&"task-1".into()).unwrap().clone();
    s.apply(vec![EngineeringCommand::CreateBasis(BasisEntry {
        id: "flow-basis".into(),
        revision: 0,
        name: "Gas flow".into(),
        value: EngineeringValue {
            quantity: QuantityKind::ActualVolumeFlow,
            value_si: 0.8,
        },
        source_id: "synthetic".into(),
    })])
    .unwrap();
    source.case.inputs.insert(
        SeparatorInput::GasVolumeFlow,
        InputBinding::BasisRef {
            basis_entry_id: "flow-basis".into(),
        },
    );
    s.apply(vec![EngineeringCommand::UpdateTask(Box::new(source))])
        .unwrap();
    s.calculate(&"task-1".into()).unwrap();
    adopt(&mut s, true);
    let fixed = rating(&s).clone();
    s.calculate_rating(&member()).unwrap();
    let mut basis = s.inputs().basis.basis_entries[0].clone();
    basis.value.value_si = 2.0;
    s.apply(vec![EngineeringCommand::UpdateBasis(basis)])
        .unwrap();
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    s.calculate(&"task-1".into()).unwrap();
    s.apply(vec![
        EngineeringCommand::DeleteTask("task-1".into()),
        EngineeringCommand::DeleteBasis("flow-basis".into()),
        EngineeringCommand::DeleteSource("synthetic".into()),
    ])
    .unwrap();
    assert_eq!(rating(&s), &fixed);
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    assert!(matches!(
        fixed
            .geometry
            .adoption
            .as_ref()
            .unwrap()
            .source_task
            .case
            .inputs[&SeparatorInput::GasVolumeFlow],
        InputBinding::BasisRef { .. }
    ));
}

#[test]
fn existing_matrix_preserves_cases_and_previews_removed_sections_then_stales_every_row() {
    let mut s = session();
    adopt(&mut s, true);
    let mut geometry = rating(&s).geometry.clone();
    geometry.axial_sections.insert(
        SeparatorAxialSection::NormalToHighLiquidLevel,
        geometry.inputs[&SeparatorGeometryInput::WorkingLiquidHeight].clone(),
    );
    let mut second = rating(&s).cases[0].clone();
    second.id = "second".into();
    s.apply(vec![
        EngineeringCommand::UpdateRatingGeometry {
            task: "adopted".into(),
            geometry: Box::new(geometry),
        },
        EngineeringCommand::AddRatingCase {
            task: "adopted".into(),
            case: Box::new(second),
        },
    ])
    .unwrap();
    let members = [
        member(),
        EquipmentCaseKey {
            task_id: "adopted".into(),
            case_id: "second".into(),
        },
    ];
    for member in &members {
        s.calculate_rating(member).unwrap();
    }
    let cases = rating(&s).cases.clone();
    let mut update = request(false);
    update.target = GeometryAdoptionTarget::Existing("adopted".into());
    update.adjustments.diameter = Some(DimensionAdjustment {
        value_m: 1.2,
        reason: "Standard diameter".into(),
    });
    let preview = s.prepare_geometry_adoption(update.clone()).unwrap();
    assert_eq!(preview.previous_geometry().unwrap().axial_sections.len(), 1);
    assert!(
        preview
            .inputs()
            .rating_task(&"adopted".into())
            .unwrap()
            .geometry
            .axial_sections
            .is_empty()
    );
    assert_eq!(preview.impacts().len(), 2);
    // An intervening target label edit also invalidates the atomic proposal.
    let old = rating(&s);
    s.apply(vec![EngineeringCommand::UpdateRatingLabels {
        task: old.id.clone(),
        revision: old.revision,
        tag: old.tag.clone(),
        name: "Changed".into(),
    }])
    .unwrap();
    assert!(s.commit_geometry_adoption(preview).is_err());
    let preview = s.prepare_geometry_adoption(update).unwrap();
    s.commit_geometry_adoption(preview).unwrap();
    assert_eq!(rating(&s).cases, cases);
    for member in &members {
        assert_eq!(s.case_result_status(member), EquipmentResultStatus::Stale);
    }
    let historical = rating(&s).geometry.adoption.clone();
    let mut edited = rating(&s).geometry.clone();
    edited.adoption = None; // Ordinary edits cannot erase the design history.
    edited
        .inputs
        .remove(&SeparatorGeometryInput::WorkingLiquidHeight);
    s.apply(vec![
        EngineeringCommand::UpdateRatingGeometry {
            task: "adopted".into(),
            geometry: Box::new(edited),
        },
        EngineeringCommand::DeleteRatingCase(members[1].clone()),
        EngineeringCommand::UpgradeRatingMethod("adopted".into()),
    ])
    .unwrap();
    assert_eq!(rating(&s).geometry.adoption, historical);
    assert_eq!(rating(&s).method.version, 3);
}

#[test]
fn editor_rejects_source_target_and_evidence_drafts_without_discarding_or_partial_commit() {
    use editor::{
        EquipmentDraft, EquipmentEditError, EquipmentEditorCommand as C, EquipmentEditorState,
        EquipmentEditorTarget as T,
    };
    use rf_types::units::DisplayUnitSet;
    let mut baseline = session();
    adopt(&mut baseline, true);
    for target in [
        T::Task("task-1".into()),
        T::RatingCase(member()),
        T::RatingGeometry("adopted".into()),
        T::Source("synthetic".into()),
    ] {
        let mut s = baseline.clone();
        let mut e = EquipmentEditorState::default();
        let mut update = request(false);
        update.target = GeometryAdoptionTarget::Existing("adopted".into());
        let proposal = e.prepare_geometry_adoption(&s, update.clone()).unwrap();
        e.dispatch(&mut s, &DisplayUnitSet::si(), C::Select(target.clone()))
            .unwrap();
        let draft = e.draft(&target).unwrap();
        let generation = draft.generation();
        let mut value = draft.value().clone();
        match &mut value {
            EquipmentDraft::Task(t) => t.name.push_str(" draft"),
            EquipmentDraft::RatingCase(c) => c.name.push_str(" draft"),
            EquipmentDraft::RatingGeometry(g) => {
                g.applicability.internals_description.push_str(" draft")
            }
            EquipmentDraft::Source(source) => source.title.push_str(" draft"),
            _ => unreachable!(),
        }
        e.dispatch(
            &mut s,
            &DisplayUnitSet::si(),
            C::Change {
                target: target.clone(),
                generation,
                value: Box::new(value.clone()),
            },
        )
        .unwrap();
        let before = s.clone();
        assert!(
            e.commit_geometry_adoption(&mut s, &DisplayUnitSet::si(), proposal)
                .is_err()
        );
        assert_eq!(s, before);
        assert_eq!(e.draft(&target).unwrap().value(), &value);
        assert_eq!(
            e.prepare_geometry_adoption(&s, update),
            Err(EquipmentEditError::PendingDrafts)
        );
    }
}

#[test]
fn adopted_inputs_round_trip_through_real_writer_and_reopen_missing() {
    let mut s = session();
    adopt(&mut s, true);
    s.calculate_rating(&member()).unwrap();
    let rf_store::project_v3::ProjectV3Read::Editable(mut project) = rf_store::project_v3::parse(
        include_str!("../../../rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!()
    };
    project.engineering = Some(rf_store::project_v3::StoredEngineeringInputs {
        basis: s.inputs().basis.clone(),
        tasks: s.inputs().tasks.clone(),
    });
    let encoded = rf_store::project_v3::to_pretty_json(
        &rf_store::project_v3::ProjectV3Read::Editable(project),
    )
    .unwrap();
    for forbidden in ["eligibility", "calculationId", "checkpoint", "loadFraction"] {
        assert!(!encoded.contains(forbidden));
    }
    let rf_store::project_v3::ProjectV3Read::Editable(decoded) =
        rf_store::project_v3::parse(&encoded).unwrap()
    else {
        panic!()
    };
    let inputs = decoded.engineering.unwrap();
    assert_eq!(inputs.tasks, s.inputs().tasks);
    assert_eq!(inputs.basis, s.inputs().basis);
    let reopened = EngineeringSession::from_inputs(
        "doc".into(),
        EngineeringInputs {
            basis: inputs.basis,
            tasks: inputs.tasks,
        },
    )
    .unwrap();
    assert_eq!(
        reopened.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert!(!reopened.can_undo());
    assert!(!reopened.is_dirty());
}

#[test]
fn copying_effective_override_values_freezes_source_dependency_graph_without_promoting_reviews() {
    let mut s = session();
    let mut original = s.inputs().basis.sources[0].clone();
    original.observations.push(SourceObservation {
        id: "flow".into(),
        quantity: QuantityKind::ActualVolumeFlow,
        original_value: 0.6,
        original_unit: "m3/s".into(),
        canonical_value: 0.6,
        conversion_note: String::new(),
        depends_on: vec![],
    });
    let mut adopted_source = original.clone();
    adopted_source.id = "adjusted-source".into();
    adopted_source.observations[0]
        .depends_on
        .push(SourceObservationRef {
            source_id: original.id.clone(),
            observation_id: "flow".into(),
        });
    adopted_source.adoption_note = "Explicit effective-value adjustment".into();
    let mut task = s.inputs().task(&"task-1".into()).unwrap().clone();
    task.case.inputs.insert(
        SeparatorInput::GasVolumeFlow,
        InputBinding::Override {
            basis_entry_id: "original-flow".into(),
            value: EngineeringValue {
                quantity: QuantityKind::ActualVolumeFlow,
                value_si: 0.8,
            },
            source_id: adopted_source.id.clone(),
            reason: "Selected independent operating case".into(),
        },
    );
    s.apply(vec![
        EngineeringCommand::UpdateSource(Box::new(original)),
        EngineeringCommand::CreateSource(Box::new(adopted_source)),
        EngineeringCommand::CreateBasis(BasisEntry {
            id: "original-flow".into(),
            revision: 0,
            name: "Original gas flow".into(),
            value: EngineeringValue {
                quantity: QuantityKind::ActualVolumeFlow,
                value_si: 0.6,
            },
            source_id: "synthetic".into(),
        }),
        EngineeringCommand::UpdateTask(Box::new(task)),
    ])
    .unwrap();
    s.calculate(&"task-1".into()).unwrap();
    adopt(&mut s, true);
    let copied = &rating(&s).cases[0].inputs[&SeparatorRatingInput::GasVolumeFlow];
    let resolved = s
        .inputs()
        .basis
        .resolve(QuantityKind::ActualVolumeFlow, copied)
        .unwrap();
    assert_eq!(resolved.value.value_si, 0.8);
    assert!(matches!(resolved.origin, BindingOrigin::Direct));
    assert_ne!(resolved.source.id.as_str(), "adjusted-source");
    assert_eq!(resolved.source.kind, SourceKind::Assumed);
    assert_eq!(resolved.source.review, SourceReview::Adopted);
    let dependency = &resolved.source.observations[0].depends_on[0];
    assert_ne!(dependency.source_id.as_str(), "synthetic");
    let archived = rating(&s).geometry.adoption.as_ref().unwrap();
    assert!(matches!(
        archived.source_task.case.inputs[&SeparatorInput::GasVolumeFlow],
        InputBinding::Override { .. }
    ));
    assert!(
        archived
            .source_basis
            .sources
            .iter()
            .any(|source| source.id.as_str() == "synthetic")
    );
    s.apply(vec![
        EngineeringCommand::DeleteTask("task-1".into()),
        EngineeringCommand::DeleteBasis("original-flow".into()),
        EngineeringCommand::DeleteSource("adjusted-source".into()),
        EngineeringCommand::DeleteSource("synthetic".into()),
    ])
    .unwrap();
    assert_eq!(
        s.calculate_rating(&member()).unwrap(),
        EquipmentResultStatus::Current
    );
}

#[test]
fn adopting_into_legacy_v1_and_matrix_v2_upgrades_only_the_selected_target() {
    for version in [1, 2] {
        let mut s = session();
        let legacy = rating_tests::session();
        let mut target = rating_tests::task(&legacy);
        target.method.version = version;
        if version == 2 {
            let mut extra = target.cases[0].clone();
            extra.id = "matrix-extra".into();
            target.cases.insert(0, extra);
        }
        let mut commands: Vec<_> = legacy
            .inputs()
            .basis
            .sources
            .iter()
            .cloned()
            .map(|source| EngineeringCommand::CreateSource(Box::new(source)))
            .collect();
        commands.extend(
            legacy
                .inputs()
                .basis
                .basis_entries
                .iter()
                .cloned()
                .map(EngineeringCommand::CreateBasis),
        );
        commands.push(EngineeringCommand::CreateRatingTask(Box::new(target)));
        s.apply(commands).unwrap();
        let before = s.inputs().rating_task(&"rating-1".into()).unwrap().clone();
        for case in &before.cases {
            s.calculate_rating(&EquipmentCaseKey {
                task_id: before.id.clone(),
                case_id: case.id.clone(),
            })
            .unwrap();
        }
        let mut update = request(false);
        update.target = GeometryAdoptionTarget::Existing(before.id.clone());
        let proposal = s.prepare_geometry_adoption(update).unwrap();
        assert_eq!(
            s.inputs().rating_task(&before.id).unwrap().method.version,
            version
        );
        s.commit_geometry_adoption(proposal).unwrap();
        let after = s.inputs().rating_task(&before.id).unwrap();
        assert_eq!(after.method.version, 3);
        assert_eq!(after.cases, before.cases);
        assert_eq!((&after.tag, &after.name), (&before.tag, &before.name));
        assert_eq!(s.inputs().task(&"task-1".into()).unwrap().method.version, 1);
        for case in &after.cases {
            assert_eq!(
                s.case_result_status(&EquipmentCaseKey {
                    task_id: after.id.clone(),
                    case_id: case.id.clone()
                }),
                EquipmentResultStatus::Stale
            );
        }
    }
}

#[test]
fn editor_commit_refreshes_clean_geometry_and_preserves_unrelated_pending_drafts() {
    use editor::{
        EquipmentDraft, EquipmentEditorCommand as C, EquipmentEditorState,
        EquipmentEditorTarget as T, RatingGeometryField,
    };
    use rf_types::units::DisplayUnitSet;
    let mut s = session();
    adopt(&mut s, true);
    let mut unrelated = s.inputs().basis.sources[0].clone();
    unrelated.id = "unrelated".into();
    s.apply(vec![EngineeringCommand::CreateSource(Box::new(unrelated))])
        .unwrap();
    let mut e = EquipmentEditorState::default();
    let source = T::Source("unrelated".into());
    e.dispatch(&mut s, &DisplayUnitSet::si(), C::Select(source.clone()))
        .unwrap();
    let record = e.draft(&source).unwrap();
    let generation = record.generation();
    let mut draft = record.value().clone();
    let EquipmentDraft::Source(ref mut value) = draft else {
        panic!()
    };
    value.title = "Unsaved unrelated source".into();
    e.dispatch(
        &mut s,
        &DisplayUnitSet::si(),
        C::Change {
            target: source.clone(),
            generation,
            value: Box::new(draft.clone()),
        },
    )
    .unwrap();
    let geometry = T::RatingGeometry("adopted".into());
    e.dispatch(&mut s, &DisplayUnitSet::si(), C::Select(geometry.clone()))
        .unwrap();
    let mut update = request(false);
    update.target = GeometryAdoptionTarget::Existing("adopted".into());
    update.adjustments.diameter = Some(DimensionAdjustment {
        value_m: 1.3,
        reason: "Selected alternative".into(),
    });
    let proposal = e.prepare_geometry_adoption(&s, update).unwrap();
    e.commit_geometry_adoption(&mut s, &DisplayUnitSet::si(), proposal)
        .unwrap();
    assert_eq!(e.active(), Some(&geometry));
    assert_eq!(e.draft(&source).unwrap().value(), &draft);
    assert_eq!(e.pending_count(), 1);
    let EquipmentDraft::RatingGeometry(value) = e.draft(&geometry).unwrap().value() else {
        panic!()
    };
    assert_eq!(
        value.fields[&RatingGeometryField::Input(SeparatorGeometryInput::InternalDiameter)]
            .value
            .value(QuantityKind::Length)
            .unwrap(),
        1.3
    );
    assert!(!e.draft(&geometry).unwrap().is_pending());
}
