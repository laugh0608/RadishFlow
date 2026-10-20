use super::*;

#[test]
fn override_preserves_original_basis_without_adopting_its_inactive_evidence() {
    let (mut task, mut basis) = fixture();
    let mut original = basis.sources[1].clone();
    original.id = "old-drawing".into();
    original.kind = SourceKind::Unknown;
    original.review = SourceReview::Unreviewed;
    basis.sources.push(original.clone());
    let entry = BasisEntry {
        id: "diameter".into(),
        revision: 11,
        name: "Previous diameter".into(),
        value: EngineeringValue {
            quantity: QuantityKind::Length,
            value_si: 0.5,
        },
        source_id: original.id.clone(),
    };
    basis.basis_entries.push(entry.clone());
    let field = SeparatorGeometryInput::InternalDiameter;
    task.geometry.inputs.insert(
        field,
        InputBinding::BasisRef {
            basis_entry_id: entry.id.clone(),
        },
    );
    rejects(&task, &basis, RatingIssueKind::MissingSourceEvidence);
    rejects(&task, &basis, RatingIssueKind::SourceNotAdopted);
    let binding = InputBinding::Override {
        basis_entry_id: entry.id.clone(),
        value: EngineeringValue {
            quantity: QuantityKind::Length,
            value_si: std::f64::consts::FRAC_2_SQRT_PI,
        },
        source_id: "drawing".into(),
        reason: "Explicit replacement from revised geometry drawing".into(),
    };
    task.geometry.inputs.insert(field, binding.clone());
    let result = calculate(&task, &basis).unwrap();
    near(result.performance().given_area_m2, 1.0);
    assert_eq!(result.capture().task.geometry.inputs[&field], binding);
    assert_eq!(result.capture().basis_entries[&entry.id], entry);
    assert_eq!(result.capture().sources[&original.id], original);
    assert!(result.pending_review_sources().contains(&original.id));
    if let InputBinding::Override { reason, .. } = task.geometry.inputs.get_mut(&field).unwrap() {
        reason.clear();
    }
    assert!(check(&task, &basis).iter().any(|i| matches!(&i.kind,
        RatingIssueKind::InvalidBinding(error) if error.kind == EngineeringDataErrorKind::MissingOverrideReason)));
}

#[test]
fn transitive_observation_evidence_is_captured_without_relabeling_its_conditions() {
    let (task, mut basis) = fixture();
    let observation = SourceObservation {
        id: SourceObservationId::from("raw-flow"),
        quantity: QuantityKind::ActualVolumeFlow,
        original_value: 2880.0,
        original_unit: "m3/h".into(),
        canonical_value: 0.8,
        conversion_note: "Synthetic conversion; divide by 3600".into(),
        depends_on: vec![],
    };
    let mut ancestor = basis.sources[0].clone();
    ancestor.id = "upstream-observation".into();
    ancestor.publication_version = "Fixture edition 2".into();
    ancestor.accessed_on = Some("2026-10-20".parse().unwrap());
    // Ancestor context is provenance, not the adopted derived input's context.
    ancestor
        .applicability
        .operating_conditions
        .as_mut()
        .unwrap()
        .temperature_k = 280.0;
    ancestor.observations.push(observation.clone());
    basis.sources[0].observations.push(SourceObservation {
        id: "derived-flow".into(),
        depends_on: vec![SourceObservationRef {
            source_id: ancestor.id.clone(),
            observation_id: observation.id,
        }],
        ..observation
    });
    basis.sources.push(ancestor.clone());
    let result = calculate(&task, &basis).unwrap();
    assert_eq!(result.capture().sources[&ancestor.id], ancestor);
    assert!(result.assumed_sources().contains(&ancestor.id));
    basis.sources[2].review = SourceReview::Unreviewed;
    rejects(&task, &basis, RatingIssueKind::SourceNotAdopted);
    basis.sources[2].review = SourceReview::Adopted;
    basis.sources[2].kind = SourceKind::Unknown;
    rejects(&task, &basis, RatingIssueKind::MissingSourceEvidence);
}

#[test]
fn result_is_an_owned_snapshot_of_referenced_values_bindings_and_revisions() {
    let (mut task, mut basis) = fixture();
    let entry = BasisEntry {
        id: "required-time".into(),
        revision: 7,
        name: "Required working time".into(),
        value: EngineeringValue {
            quantity: QuantityKind::Duration,
            value_si: 120.0,
        },
        source_id: "operating".into(),
    };
    task.cases[0].inputs.insert(
        SeparatorRatingInput::RequiredWorkingTime,
        InputBinding::BasisRef {
            basis_entry_id: entry.id.clone(),
        },
    );
    basis.basis_entries.push(entry.clone());
    let mut unrelated = basis.sources[1].clone();
    unrelated.id = "unreferenced".into();
    unrelated.review = SourceReview::Unreviewed;
    unrelated.kind = SourceKind::Unknown;
    basis.sources.push(unrelated);
    let result = calculate(&task, &basis).unwrap();
    assert_eq!(result.capture().task, task);
    assert_eq!(result.capture().basis_entries[&entry.id], entry);
    assert_eq!(result.capture().sources.len(), 2);
    assert_eq!(result.assumed_sources(), &["operating".into()]);
    assert_eq!(result.pending_review_sources(), &["operating".into()]);
    assert_eq!(result.unchecked_requirements().len(), 7);
    let snapshot = result.clone();
    task.geometry.revision += 1;
    geometry(&mut task, SeparatorGeometryInput::InternalDiameter, 2.0);
    task.cases[0].revision += 1;
    basis.basis_entries[0].value.value_si = 180.0;
    basis.sources[0].revision += 1;
    basis.sources[0].adoption_note = "Changed after calculation".into();
    assert_eq!(result, snapshot);
    assert_ne!(calculate(&task, &basis).unwrap(), result);
}
