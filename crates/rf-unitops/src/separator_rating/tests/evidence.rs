use super::*;

#[test]
fn version_one_rejects_unknown_methods_and_multiple_cases_without_dropping_rows() {
    let (mut task, basis) = fixture();
    task.method.version = 2;
    rejects(&task, &basis, RatingIssueKind::UnsupportedMethod);
    task.method = method();
    let mut second = task.cases[0].clone();
    second.id = "second".into();
    task.cases.push(second);
    rejects(
        &task,
        &basis,
        RatingIssueKind::UnsupportedCaseCount { actual: 2 },
    );
    task.cases.clear();
    rejects(
        &task,
        &basis,
        RatingIssueKind::UnsupportedCaseCount { actual: 0 },
    );
}

#[test]
fn required_fields_report_their_own_geometry_or_case_target() {
    let (task, basis) = fixture();
    for field in SeparatorRatingInput::ALL {
        let mut changed = task.clone();
        changed.cases[0].inputs.remove(&field);
        assert!(check(&changed, &basis).contains(&RatingIssue {
            target: RatingIssueTarget::Input {
                case_id: task.cases[0].id.clone(),
                field
            },
            kind: RatingIssueKind::MissingInput,
        }));
    }
    for field in SeparatorGeometryInput::ALL {
        let mut changed = task.clone();
        changed.geometry.inputs.remove(&field);
        assert!(check(&changed, &basis).contains(&RatingIssue {
            target: RatingIssueTarget::GeometryInput(field),
            kind: RatingIssueKind::MissingInput,
        }));
    }
}

#[test]
fn invalid_numbers_and_phase_or_criterion_domains_are_rejected() {
    let (task, basis) = fixture();
    for field in SeparatorRatingInput::ALL {
        for value in [-1.0, 0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut changed = task.clone();
            set(&mut changed, field, value);
            let issues = check(&changed, &basis);
            assert!(
                issues.iter().any(|i| i.target
                    == RatingIssueTarget::Input {
                        case_id: task.cases[0].id.clone(),
                        field
                    }),
                "{field:?}={value}: {issues:?}"
            );
            assert!(calculate(&changed, &basis).is_err());
        }
    }
    for field in SeparatorGeometryInput::ALL {
        for value in [-1.0, 0.0, f64::NAN, f64::INFINITY] {
            let mut changed = task.clone();
            geometry(&mut changed, field, value);
            assert!(
                check(&changed, &basis)
                    .iter()
                    .any(|i| i.target == RatingIssueTarget::GeometryInput(field))
            );
            assert!(calculate(&changed, &basis).is_err());
        }
    }
    let mut changed = task.clone();
    set(
        &mut changed,
        SeparatorRatingInput::GasVelocityUtilization,
        1.01,
    );
    rejects(&changed, &basis, RatingIssueKind::UtilizationAboveOne);
    for density in [10.0, 9.0] {
        set(&mut changed, SeparatorRatingInput::LiquidDensity, density);
        rejects(&changed, &basis, RatingIssueKind::InvalidDensityOrder);
    }
    for field in [
        SeparatorRatingInput::GasVolumeFlow,
        SeparatorRatingInput::LiquidVolumeFlow,
    ] {
        let mut changed = task.clone();
        set(&mut changed, field, 0.0);
        rejects(&changed, &basis, RatingIssueKind::AbsentPhase);
    }
}

#[test]
fn each_applicability_declaration_is_required_and_unsupported_values_are_rejected() {
    let (task, basis) = fixture();
    let mutations: [fn(&mut SeparatorRatingTask); 6] = [
        |t| t.geometry.applicability.orientation = Some(SeparatorOrientation::Horizontal),
        |t| t.geometry.applicability.gas_direction = Some(SeparatorGasDirection::Other),
        |t| {
            t.geometry.applicability.geometry = Some(SeparatorSectionGeometry::ReducedEffectiveArea)
        },
        |t| t.cases[0].applicability.service = Some(SeparatorService::ThreePhase),
        |t| t.cases[0].applicability.k_basis = Some(SeparatorKBasis::MistEliminatorOnly),
        |t| t.cases[0].applicability.flow_basis = Some(SeparatorFlowBasis::StandardConditions),
    ];
    for mutate in mutations {
        let mut changed = task.clone();
        mutate(&mut changed);
        rejects(&changed, &basis, RatingIssueKind::OutsideMethodScope);
    }
    let mut changed = task;
    changed.geometry.applicability = SeparatorGeometryApplicability::default();
    changed.cases[0].applicability = SeparatorRatingApplicability::default();
    changed.cases[0].fluid_system.clear();
    assert_eq!(
        check(&changed, &basis)
            .iter()
            .filter(|i| i.kind == RatingIssueKind::MissingInput)
            .count(),
        8
    );
}

#[test]
fn geometry_needs_documentary_evidence_but_no_invented_fluid_or_operating_conditions() {
    let (task, basis) = fixture();
    assert!(calculate(&task, &basis).is_ok());
    let mutations: [fn(&mut SourceRecord); 4] = [
        |s| s.locator.clear(),
        |s| s.reference.clear(),
        |s| s.applicability.scope.clear(),
        |s| s.kind = SourceKind::Unknown,
    ];
    for mutate in mutations {
        let mut changed = basis.clone();
        mutate(&mut changed.sources[1]);
        rejects(&task, &changed, RatingIssueKind::MissingSourceEvidence);
    }
    let mut changed = basis.clone();
    changed.sources[1].review = SourceReview::Unreviewed;
    rejects(&task, &changed, RatingIssueKind::SourceNotAdopted);
    changed = basis.clone();
    changed.sources[1].applicability.fluid_system = "Different restricted fluid".into();
    rejects(&task, &changed, RatingIssueKind::SourceConditionMismatch);
    changed = basis;
    changed.sources[1].applicability.operating_conditions = Some(SourceOperatingConditions {
        temperature_k: 301.0,
        pressure_pa: 100000.0,
    });
    rejects(&task, &changed, RatingIssueKind::SourceConditionMismatch);
}

#[test]
fn operating_sources_require_adoption_and_matching_phase_fluid_and_conditions() {
    let (task, basis) = fixture();
    let mismatches: [fn(&mut SourceRecord); 5] = [
        |s| s.applicability.fluid_system.clear(),
        |s| s.applicability.phase = SourcePhase::NotApplicable,
        |s| s.applicability.operating_conditions = None,
        |s| {
            s.applicability
                .operating_conditions
                .as_mut()
                .unwrap()
                .temperature_k = 301.0
        },
        |s| {
            s.applicability
                .operating_conditions
                .as_mut()
                .unwrap()
                .pressure_pa = 110000.0
        },
    ];
    for mutate in mismatches {
        let mut changed = basis.clone();
        mutate(&mut changed.sources[0]);
        rejects(&task, &changed, RatingIssueKind::SourceConditionMismatch);
    }
    let mut changed = basis.clone();
    changed.sources[0].adoption_note.clear();
    rejects(&task, &changed, RatingIssueKind::MissingSourceEvidence);
    changed = basis.clone();
    changed.sources[0].review = SourceReview::Unreviewed;
    rejects(&task, &changed, RatingIssueKind::SourceNotAdopted);
    let mut changed = task.clone();
    changed.geometry.applicability_source_ids.clear();
    changed.cases[0].applicability_source_ids.clear();
    let issues = check(&changed, &basis);
    for target in [
        RatingIssueTarget::Geometry,
        RatingIssueTarget::Case(task.cases[0].id.clone()),
    ] {
        assert!(issues.contains(&RatingIssue {
            target,
            kind: RatingIssueKind::MissingSourceEvidence
        }));
    }
    changed
        .geometry
        .applicability_source_ids
        .push("absent".into());
    rejects(
        &changed,
        &basis,
        RatingIssueKind::MissingSource("absent".into()),
    );
}

#[test]
fn malformed_bindings_preserve_structured_errors() {
    let (task, basis) = fixture();
    let field = SeparatorRatingInput::RequiredWorkingTime;
    let cases = [
        (
            direct(QuantityKind::Length, 120.0, "operating"),
            EngineeringDataErrorKind::QuantityMismatch {
                expected: QuantityKind::Duration,
                actual: QuantityKind::Length,
            },
        ),
        (
            direct(QuantityKind::Duration, 120.0, "absent"),
            EngineeringDataErrorKind::MissingSource("absent".into()),
        ),
        (
            InputBinding::BasisRef {
                basis_entry_id: "absent".into(),
            },
            EngineeringDataErrorKind::MissingBasis("absent".into()),
        ),
    ];
    for (binding, expected) in cases {
        let mut changed = task.clone();
        changed.cases[0].inputs.insert(field, binding);
        assert!(check(&changed, &basis).iter().any(
            |i| matches!(&i.kind, RatingIssueKind::InvalidBinding(error) if error.kind == expected)
        ));
    }
}

#[test]
fn optional_height_gaps_stay_unknown_but_invalid_dimensions_block_calculation() {
    let (mut task, mut basis) = fixture();
    let section = SeparatorAxialSection::ALL[0];
    let mut incomplete = basis.sources[1].clone();
    incomplete.id = "incomplete".into();
    incomplete.kind = SourceKind::Unknown;
    basis.sources.push(incomplete);
    task.geometry
        .axial_sections
        .insert(section, direct(QuantityKind::Length, 0.1, "incomplete"));
    let result = calculate(&task, &basis).unwrap();
    assert!(result.performance().tangent_to_tangent_height_m.is_none());
    assert!(result.missing_axial_sections().contains(&section));
    assert!(!result.height_issues().is_empty());
    task.geometry
        .axial_sections
        .insert(section, direct(QuantityKind::Length, -0.1, "incomplete"));
    rejects(&task, &basis, RatingIssueKind::NonPositiveInput);
    task.geometry
        .axial_sections
        .insert(section, direct(QuantityKind::Duration, 1.0, "incomplete"));
    assert!(matches!(
        &calculate(&task, &basis).unwrap_err().issues[0].kind,
        RatingIssueKind::InvalidBinding(_)
    ));
}
