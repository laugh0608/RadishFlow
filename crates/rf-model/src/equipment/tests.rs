use super::*;

fn rating(id: &str, case: &str, tag: &str) -> SeparatorRatingTask {
    SeparatorRatingTask {
        id: id.into(),
        revision: 1,
        tag: tag.into(),
        name: "Incomplete rating record".into(),
        method: EquipmentMethod {
            id: "test-method".into(),
            version: 1,
        },
        geometry: SeparatorGivenGeometry {
            revision: 1,
            applicability: SeparatorGeometryApplicability::default(),
            applicability_source_ids: vec![],
            inputs: BTreeMap::new(),
            axial_sections: BTreeMap::new(),
        },
        cases: vec![SeparatorRatingCase {
            id: case.into(),
            revision: 1,
            name: "Case label".into(),
            purpose: String::new(),
            fluid_system: String::new(),
            applicability: SeparatorRatingApplicability::default(),
            applicability_source_ids: vec![],
            inputs: BTreeMap::new(),
        }],
    }
}
fn sizing(id: &str, case: &str, tag: &str) -> SeparatorSizingTask {
    SeparatorSizingTask {
        id: id.into(),
        revision: 1,
        tag: tag.into(),
        name: "Incomplete sizing record".into(),
        method: EquipmentMethod {
            id: "test-method".into(),
            version: 1,
        },
        case: SeparatorCase {
            id: case.into(),
            name: "Case label".into(),
            purpose: String::new(),
            fluid_system: String::new(),
            applicability_source_ids: vec![],
            inputs: BTreeMap::new(),
            applicability: SeparatorApplicability::default(),
        },
        axial_sections: BTreeMap::new(),
    }
}

#[test]
fn rating_fields_have_distinct_geometry_and_operating_quantities() {
    use QuantityKind::*;
    assert_eq!(SeparatorRatingInput::ALL.len(), 9);
    assert_eq!(
        SeparatorRatingInput::ALL.map(|field| field.quantity()),
        [
            AbsoluteTemperature,
            AbsolutePressure,
            ActualVolumeFlow,
            ActualVolumeFlow,
            MassDensity,
            MassDensity,
            Velocity,
            DimensionlessRatio,
            Duration,
        ]
    );
    assert_eq!(
        SeparatorGeometryInput::ALL.map(|f| f.quantity()),
        [Length; 2]
    );
    assert_eq!(SeparatorInput::ALL.len(), 10);
    // Incomplete formal records are representable without inventing defaults.
    let task = rating("rating", "case", "");
    assert!(task.geometry.inputs.is_empty());
    assert!(task.cases[0].inputs.is_empty());
    assert!(validate_separator_rating_identities(&[task]).is_ok());
}

#[test]
fn mixed_tasks_share_id_case_and_tag_uniqueness() {
    use EquipmentIdentityErrorKind::*;
    let first = sizing("sizing", "sizing-case", "V-101");
    for (id, case, tag, expected) in [
        ("sizing", "rating-case", "V-102", DuplicateTaskId),
        ("rating", "sizing-case", "V-102", DuplicateCaseId),
        ("rating", "rating-case", " v-101 ", DuplicateTag),
        (" ", "rating-case", "", EmptyTaskId),
        ("rating", " ", "", EmptyCaseId),
    ] {
        assert_eq!(
            validate_equipment_task_identities(&[
                EquipmentTask::SeparatorSizing(first.clone()),
                EquipmentTask::SeparatorRating(rating(id, case, tag)),
            ]),
            Err(EquipmentIdentityError {
                task_index: 1,
                kind: expected,
            })
        );
    }
    assert!(
        validate_equipment_task_identities(&[
            EquipmentTask::SeparatorRating(rating("rating", "rating-case", "V-102")),
            EquipmentTask::SeparatorSizing(first),
        ])
        .is_ok()
    );
}

#[test]
fn rating_case_membership_is_nonempty_and_unique_before_method_checks() {
    let mut task = rating("rating", "case", "");
    task.cases.clear();
    assert_eq!(
        validate_separator_rating_identities(&[task.clone()])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::EmptyCaseCollection
    );
    task = rating("rating", "case", "");
    task.cases.push(task.cases[0].clone());
    assert_eq!(
        validate_separator_rating_identities(&[task.clone()])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::DuplicateCaseId
    );
    task.cases[1].id = "case-2".into();
    // Method version 1 rejects multiple cases in rf-unitops, not in this object model.
    assert!(validate_separator_rating_identities(&[task]).is_ok());
}

#[test]
fn rating_identities_are_case_sensitive_and_labels_are_not_keys() {
    let mut first = rating("rating", "case", "");
    let mut second = rating("Rating", "Case", " ");
    first.name = "same".into();
    second.name = first.name.clone();
    assert!(validate_separator_rating_identities(&[first.clone(), second.clone()]).is_ok());
    second.cases[0].id = first.cases[0].id.clone();
    assert_eq!(
        validate_separator_rating_identities(&[first, second])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::DuplicateCaseId
    );
}
