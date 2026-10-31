use super::*;
use rf_model::engineering::*;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceObservationId, units::*};
mod adoption;

// Public synthetic arithmetic fixture S0-B01. No proprietary engineering data.
fn fixture() -> (SeparatorSizingTask, EngineeringBasis) {
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
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 + 1e-10 * expected.abs(),
        "{actual} != {expected}"
    );
}
fn rejects(task: &SeparatorSizingTask, basis: &EngineeringBasis, kind: SizingIssueKind) {
    let failure = calculate(task, basis).unwrap_err();
    assert!(failure.issues.iter().any(|i| i.kind == kind), "{failure:?}");
}

#[test]
fn b01_gas_control_matches_independent_analytic_values() {
    let (task, basis) = fixture();
    assert!(check(&task, &basis).is_empty());
    let candidate = calculate(&task, &basis).unwrap();
    let d = candidate.dimensions();
    for (actual, expected) in [
        (d.souders_brown_velocity_m_s, 1.0),
        (d.allowed_gas_velocity_m_s, 0.8),
        (d.gas_area_m2, 1.0),
        (d.liquid_area_m2, 0.6),
        (d.required_area_m2, 1.0),
        (d.working_volume_m3, 0.6),
        (d.minimum_diameter_m, std::f64::consts::FRAC_2_SQRT_PI),
        (d.working_liquid_height_m, 0.6),
        (d.gas_load_fraction, 1.0),
    ] {
        near(actual, expected);
    }
    assert_eq!(d.controlling_constraint, ControllingConstraint::GasArea);
    assert_eq!(d.tangent_to_tangent_height_m, None);
    assert_eq!(d.buffer_volume_m3, None);
    assert_eq!(candidate.missing_axial_sections().len(), 6);
    assert_eq!(candidate.assumed_sources(), &[SourceId::from("synthetic")]);
    assert_eq!(candidate.pending_review_sources().len(), 1);
    assert!(
        candidate
            .unchecked_requirements()
            .contains(&UncheckedRequirement::PhaseEquilibrium)
    );
}

#[test]
fn b02_liquid_control_and_joint_control_are_distinct() {
    let (mut task, basis) = fixture();
    set(&mut task, SeparatorInput::LiquidVolumeFlow, 0.02);
    let candidate = calculate(&task, &basis).unwrap();
    let d = candidate.dimensions();
    near(d.required_area_m2, 2.4);
    near(d.working_volume_m3, 2.4);
    near(d.minimum_diameter_m, 1.7480774889473265);
    near(d.working_liquid_height_m, 1.0);
    near(d.gas_load_fraction, 5.0 / 12.0);
    assert_eq!(
        d.controlling_constraint,
        ControllingConstraint::LiquidWorkingVolume
    );
    for (area, control) in [
        (1.0, ControllingConstraint::Both),
        (1.0 + 5e-11, ControllingConstraint::Both),
        (1.0 + 2e-10, ControllingConstraint::LiquidWorkingVolume),
    ] {
        set(&mut task, SeparatorInput::LiquidVolumeFlow, area / 120.0);
        assert_eq!(
            calculate(&task, &basis)
                .unwrap()
                .dimensions()
                .controlling_constraint,
            control
        );
    }
}

#[test]
fn b03_shared_units_preserve_canonical_inputs_and_outputs() {
    let (mut task, basis) = fixture();
    let original = calculate(&task, &basis).unwrap();
    for (field, value, unit) in [
        (
            SeparatorInput::GasVolumeFlow,
            2880.0,
            MeasurementUnit::CubicMetrePerHour,
        ),
        (
            SeparatorInput::LiquidVolumeFlow,
            18.0,
            MeasurementUnit::CubicMetrePerHour,
        ),
        (
            SeparatorInput::WorkingVolumeTime,
            2.0,
            MeasurementUnit::Minute,
        ),
        (
            SeparatorInput::MaximumWorkingLiquidHeight,
            1000.0,
            MeasurementUnit::Millimetre,
        ),
    ] {
        set(
            &mut task,
            field,
            to_canonical(value, field.quantity(), unit).unwrap(),
        );
    }
    assert_eq!(calculate(&task, &basis).unwrap(), original);
    assert!(
        to_canonical(
            2880.0,
            QuantityKind::ActualVolumeFlow,
            MeasurementUnit::StandardCubicMetrePerHour,
        )
        .is_err()
    );
}

#[test]
fn no_defaults_for_any_missing_required_field_or_declaration() {
    let (task, basis) = fixture();
    for field in SeparatorInput::ALL {
        let mut incomplete = task.clone();
        incomplete.case.inputs.remove(&field);
        let issues = check(&incomplete, &basis);
        assert!(issues.contains(&SizingIssue {
            target: IssueTarget::Input(field),
            kind: SizingIssueKind::MissingInput
        }));
        assert!(calculate(&incomplete, &basis).is_err());
    }
    let mut incomplete = task;
    incomplete.case.inputs.clear();
    incomplete.case.applicability = SeparatorApplicability::default();
    assert_eq!(
        check(&incomplete, &basis)
            .iter()
            .filter(|i| i.kind == SizingIssueKind::MissingInput)
            .count(),
        17
    );
}

#[test]
fn invalid_finite_nonfinite_and_single_phase_inputs_return_structured_errors() {
    let (task, basis) = fixture();
    for field in SeparatorInput::ALL {
        for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut invalid = task.clone();
            set(&mut invalid, field, value);
            let failure = calculate(&invalid, &basis).unwrap_err();
            assert!(
                failure
                    .issues
                    .iter()
                    .any(|i| i.target == IssueTarget::Input(field))
            );
        }
        let mut zero = task.clone();
        set(&mut zero, field, 0.0);
        let phase = matches!(
            field,
            SeparatorInput::GasVolumeFlow | SeparatorInput::LiquidVolumeFlow
        );
        rejects(
            &zero,
            &basis,
            if phase {
                SizingIssueKind::AbsentPhase
            } else {
                SizingIssueKind::NonPositiveInput
            },
        );
    }
    let mut invalid = task;
    set(&mut invalid, SeparatorInput::GasVelocityUtilization, 1.01);
    rejects(&invalid, &basis, SizingIssueKind::UtilizationAboveOne);
    for rho in [10.0, 9.0] {
        set(&mut invalid, SeparatorInput::LiquidDensity, rho);
        rejects(&invalid, &basis, SizingIssueKind::InvalidDensityOrder);
    }
}

#[test]
fn overflow_and_underflow_cannot_produce_candidates() {
    let (task, basis) = fixture();
    for (field, value, step) in [
        (
            SeparatorInput::WorkingVolumeTime,
            f64::from_bits(1),
            CalculationStep::WorkingVolume,
        ),
        (
            SeparatorInput::LiquidVolumeFlow,
            f64::MAX,
            CalculationStep::WorkingVolume,
        ),
        (
            SeparatorInput::GasDensity,
            f64::MIN_POSITIVE,
            CalculationStep::DensityRatio,
        ),
        (
            SeparatorInput::EffectiveK,
            f64::MAX,
            CalculationStep::SoudersBrownVelocity,
        ),
        (
            SeparatorInput::GasVolumeFlow,
            f64::MAX,
            CalculationStep::GasArea,
        ),
        (
            SeparatorInput::MaximumWorkingLiquidHeight,
            1e-308,
            CalculationStep::DiameterSquared,
        ),
    ] {
        let mut invalid = task.clone();
        set(&mut invalid, field, value);
        rejects(&invalid, &basis, SizingIssueKind::NumericRange(step));
    }
}

#[test]
fn physical_response_and_constraint_switch_are_monotonic() {
    let (task, basis) = fixture();
    let before = calculate(&task, &basis).unwrap();
    for (field, value) in [
        (SeparatorInput::GasVolumeFlow, 1.6),
        (SeparatorInput::LiquidVolumeFlow, 0.02),
        (SeparatorInput::GasVelocityUtilization, 0.4),
        (SeparatorInput::WorkingVolumeTime, 480.0),
        (SeparatorInput::MaximumWorkingLiquidHeight, 0.25),
    ] {
        let mut changed = task.clone();
        set(&mut changed, field, value);
        let candidate = calculate(&changed, &basis).unwrap();
        assert!(candidate.dimensions().minimum_diameter_m > before.dimensions().minimum_diameter_m);
        assert_eq!(candidate.dimensions().tangent_to_tangent_height_m, None);
    }
    let mut changed = task;
    set(&mut changed, SeparatorInput::GasVelocityUtilization, 0.4);
    near(
        calculate(&changed, &basis)
            .unwrap()
            .dimensions()
            .minimum_diameter_m,
        before.dimensions().minimum_diameter_m * 2.0_f64.sqrt(),
    );
}

#[test]
fn unsupported_services_geometry_and_method_are_not_confirmable() {
    let (task, basis) = fixture();
    let mut variants = Vec::new();
    for service in [
        SeparatorService::SinglePhase,
        SeparatorService::Foaming,
        SeparatorService::Solids,
        SeparatorService::Slugging,
        SeparatorService::ThreePhase,
    ] {
        let mut changed = task.clone();
        changed.case.applicability.service = Some(service);
        variants.push(changed);
    }
    let mut changed = task.clone();
    changed.case.applicability.orientation = Some(SeparatorOrientation::Horizontal);
    variants.push(changed);
    let mut changed = task.clone();
    changed.case.applicability.gas_direction = Some(SeparatorGasDirection::Other);
    variants.push(changed);
    for geometry in [
        SeparatorSectionGeometry::ReducedEffectiveArea,
        SeparatorSectionGeometry::NonCylindrical,
    ] {
        let mut changed = task.clone();
        changed.case.applicability.geometry = Some(geometry);
        variants.push(changed);
    }
    for k_basis in [
        SeparatorKBasis::MistEliminatorOnly,
        SeparatorKBasis::UnmatchedInternals,
    ] {
        let mut changed = task.clone();
        changed.case.applicability.k_basis = Some(k_basis);
        variants.push(changed);
    }
    let mut changed = task.clone();
    changed.case.applicability.flow_basis = Some(SeparatorFlowBasis::StandardConditions);
    variants.push(changed);
    for changed in variants {
        rejects(&changed, &basis, SizingIssueKind::OutsideMethodScope);
    }
    let mut changed = task.clone();
    changed.method.version += 1;
    rejects(&changed, &basis, SizingIssueKind::UnsupportedMethod);
    changed.method = method();
    changed.method.id = "unknown".into();
    rejects(&changed, &basis, SizingIssueKind::UnsupportedMethod);
    assert_eq!(changed.method.id, "unknown");
}

#[test]
fn provenance_requires_explicit_adoption_and_retains_assumptions() {
    let (task, basis) = fixture();
    for kind in [
        SourceKind::Unknown,
        SourceKind::Literature,
        SourceKind::Vendor,
        SourceKind::Measured,
        SourceKind::Estimated,
    ] {
        let mut changed = basis.clone();
        changed.sources[0].kind = kind;
        rejects(&task, &changed, SizingIssueKind::MissingSourceEvidence);
    }
    let mut changed = basis.clone();
    changed.sources[0].adoption_note.clear();
    rejects(&task, &changed, SizingIssueKind::MissingSourceEvidence);
    let mut changed = basis.clone();
    changed.sources[0].applicability.scope.clear();
    rejects(&task, &changed, SizingIssueKind::MissingSourceEvidence);
    let mut changed = basis.clone();
    changed.sources[0].review = SourceReview::Unreviewed;
    rejects(&task, &changed, SizingIssueKind::SourceNotAdopted);
    changed.sources[0].review = SourceReview::Reviewed;
    let candidate = calculate(&task, &changed).unwrap();
    assert_eq!(candidate.assumed_sources().len(), 1); // Review never converts an assumption to measured data.
    assert!(candidate.pending_review_sources().is_empty());
    changed.sources[0].kind = SourceKind::Literature;
    changed.sources[0].reference = "public synthetic fixture".into();
    changed.sources[0].locator = "S0-B01".into();
    assert!(calculate(&task, &changed).is_ok());
}

#[test]
fn mismatched_or_missing_fluid_phase_and_operating_conditions_are_rejected() {
    let (task, basis) = fixture();
    let mut variants = Vec::new();
    let mut b = basis.clone();
    b.sources[0].applicability.fluid_system.clear();
    variants.push(b);
    let mut b = basis.clone();
    b.sources[0].applicability.fluid_system = "different fluid".into();
    variants.push(b);
    let mut b = basis.clone();
    b.sources[0].applicability.operating_conditions = None;
    variants.push(b);
    let mut b = basis.clone();
    b.sources[0].applicability.phase = SourcePhase::NotApplicable;
    variants.push(b);
    let mut b = basis.clone();
    b.sources[0]
        .applicability
        .operating_conditions
        .as_mut()
        .unwrap()
        .temperature_k += 1.0;
    variants.push(b);
    let mut b = basis.clone();
    b.sources[0]
        .applicability
        .operating_conditions
        .as_mut()
        .unwrap()
        .pressure_pa += 1.0;
    variants.push(b);
    for b in variants {
        rejects(&task, &b, SizingIssueKind::SourceConditionMismatch);
    }
    let mut changed = task;
    set(&mut changed, SeparatorInput::OperatingTemperature, 301.0);
    rejects(&changed, &basis, SizingIssueKind::SourceConditionMismatch);
}

#[test]
fn bindings_resolve_once_into_owned_evidence_without_mutating_shared_basis() {
    let (mut task, mut basis) = fixture();
    let entry = BasisEntry {
        id: "shared-time".into(),
        revision: 7,
        name: "Working time".into(),
        value: EngineeringValue {
            quantity: QuantityKind::Duration,
            value_si: 120.0,
        },
        source_id: "synthetic".into(),
    };
    basis.basis_entries.push(entry.clone());
    task.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::BasisRef {
            basis_entry_id: entry.id.clone(),
        },
    );
    let shared = calculate(&task, &basis).unwrap();
    task.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::Override {
            basis_entry_id: entry.id.clone(),
            value: EngineeringValue {
                quantity: QuantityKind::Duration,
                value_si: 480.0,
            },
            source_id: "synthetic".into(),
            reason: "Explicit scenario assumption".into(),
        },
    );
    let overridden = calculate(&task, &basis).unwrap();
    assert_eq!(basis.basis_entries[0], entry);
    assert_eq!(
        shared.dimensions().controlling_constraint,
        ControllingConstraint::GasArea
    );
    assert_eq!(
        overridden.dimensions().controlling_constraint,
        ControllingConstraint::LiquidWorkingVolume
    );
    assert_eq!(overridden.capture().basis_entries[&entry.id].revision, 7);
    basis.sources[0].adoption_note = "Changed after calculation".into();
    basis.basis_entries[0].value.value_si = 240.0;
    assert_ne!(
        overridden.capture().sources[&SourceId::from("synthetic")].adoption_note,
        basis.sources[0].adoption_note
    );
    assert_eq!(
        shared.capture().values_si[&SeparatorInput::WorkingVolumeTime],
        120.0
    );
    assert_eq!(
        overridden.capture().values_si[&SeparatorInput::WorkingVolumeTime],
        480.0
    );
}

#[test]
fn missing_references_quantity_mismatches_and_invalid_overrides_are_structured() {
    let (task, basis) = fixture();
    let mut variants = Vec::new();
    let mut changed = task.clone();
    changed.case.inputs.insert(
        SeparatorInput::GasVolumeFlow,
        direct(QuantityKind::MolarFlow, 0.8),
    );
    variants.push(changed);
    let mut changed = task.clone();
    changed.case.inputs.insert(
        SeparatorInput::EffectiveK,
        InputBinding::BasisRef {
            basis_entry_id: "missing".into(),
        },
    );
    variants.push(changed);
    let mut changed = task;
    changed.case.inputs.insert(
        SeparatorInput::EffectiveK,
        InputBinding::Direct {
            value: EngineeringValue {
                quantity: QuantityKind::Velocity,
                value_si: 0.1,
            },
            source_id: "missing".into(),
        },
    );
    variants.push(changed);
    for changed in variants {
        assert!(
            calculate(&changed, &basis)
                .unwrap_err()
                .issues
                .iter()
                .any(|i| matches!(i.kind, SizingIssueKind::InvalidBinding(_)))
        );
    }
}

#[test]
fn transitive_source_dependencies_are_checked_and_captured() {
    let (task, mut basis) = fixture();
    let mut dependency = basis.sources[0].clone();
    dependency.id = "dependency".into();
    dependency.review = SourceReview::Unreviewed;
    dependency.observations.push(SourceObservation {
        id: "original".into(),
        quantity: QuantityKind::Duration,
        original_value: 2.0,
        original_unit: "min".into(),
        canonical_value: 120.0,
        conversion_note: "minutes to seconds".into(),
        depends_on: vec![],
    });
    basis.sources[0].observations.push(SourceObservation {
        id: "derived".into(),
        quantity: QuantityKind::Duration,
        original_value: 120.0,
        original_unit: "s".into(),
        canonical_value: 120.0,
        conversion_note: "explicit derivation".into(),
        depends_on: vec![SourceObservationRef {
            source_id: dependency.id.clone(),
            observation_id: SourceObservationId::from("original"),
        }],
    });
    basis.sources.push(dependency);
    rejects(&task, &basis, SizingIssueKind::SourceNotAdopted);
    basis.sources[1].review = SourceReview::Adopted;
    assert_eq!(calculate(&task, &basis).unwrap().capture().sources.len(), 2);
}

#[test]
fn complete_axial_intervals_add_height_without_double_counting_work_or_buffer() {
    let (mut task, basis) = fixture();
    for section in SeparatorAxialSection::ALL {
        task.axial_sections
            .insert(section, direct(QuantityKind::Length, 0.2));
    }
    let candidate = calculate(&task, &basis).unwrap();
    near(
        candidate.dimensions().tangent_to_tangent_height_m.unwrap(),
        1.8,
    );
    near(candidate.dimensions().buffer_volume_m3.unwrap(), 0.2);
    near(candidate.dimensions().working_volume_m3, 0.6);
    assert!(candidate.missing_axial_sections().is_empty());
    task.axial_sections
        .remove(&SeparatorAxialSection::LowerTangentToLowLiquidLevel);
    assert_eq!(
        calculate(&task, &basis)
            .unwrap()
            .dimensions()
            .tangent_to_tangent_height_m,
        None
    );
    task.axial_sections.insert(
        SeparatorAxialSection::NormalToHighLiquidLevel,
        direct(QuantityKind::Length, 0.0),
    );
    assert_eq!(
        calculate(&task, &basis)
            .unwrap()
            .dimensions()
            .buffer_volume_m3,
        Some(0.0)
    );
    task.axial_sections.insert(
        SeparatorAxialSection::InletToInternals,
        direct(QuantityKind::Length, 0.0),
    );
    rejects(&task, &basis, SizingIssueKind::NonPositiveInput);
}

#[test]
fn invalid_optional_intervals_cannot_be_silently_dropped() {
    let (task, basis) = fixture();
    for value in [-1.0, f64::NAN, f64::INFINITY] {
        let mut changed = task.clone();
        changed.axial_sections.insert(
            SeparatorAxialSection::NormalToHighLiquidLevel,
            direct(QuantityKind::Length, value),
        );
        assert!(calculate(&changed, &basis).is_err());
    }
    let mut changed = task;
    for section in SeparatorAxialSection::ALL {
        changed
            .axial_sections
            .insert(section, direct(QuantityKind::Length, f64::MAX));
    }
    rejects(
        &changed,
        &basis,
        SizingIssueKind::NumericRange(CalculationStep::TangentHeight),
    );
}

#[test]
fn equipment_identity_is_independent_of_names_and_normalizes_only_tags() {
    let (task, basis) = fixture();
    let mut second = task.clone();
    second.id = EquipmentTaskId::from("task-2");
    second.case.id = EngineeringCaseId::from("case-2");
    second.tag = " v-101 ".into();
    assert_eq!(
        validate_equipment_identities(&[task.clone(), second.clone()])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::DuplicateTag
    );
    second.tag = "V-102".into();
    assert!(validate_equipment_identities(&[task.clone(), second.clone()]).is_ok());
    second.case.id = task.case.id.clone();
    assert_eq!(
        validate_equipment_identities(&[task.clone(), second.clone()])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::DuplicateCaseId
    );
    second.id = task.id.clone();
    assert_eq!(
        validate_equipment_identities(&[task.clone(), second.clone()])
            .unwrap_err()
            .kind,
        EquipmentIdentityErrorKind::DuplicateTaskId
    );
    second.id = " ".into();
    rejects(
        &second,
        &basis,
        SizingIssueKind::InvalidIdentity(EquipmentIdentityErrorKind::EmptyTaskId),
    );
}

#[test]
fn optional_height_without_evidence_preserves_partial_size_and_reports_reason() {
    let (mut task, mut basis) = fixture();
    let mut axial_source = basis.sources[0].clone();
    axial_source.id = "axial".into();
    axial_source.kind = SourceKind::Unknown;
    basis.sources.push(axial_source);
    for section in SeparatorAxialSection::ALL {
        task.axial_sections.insert(
            section,
            InputBinding::Direct {
                value: EngineeringValue {
                    quantity: QuantityKind::Length,
                    value_si: 0.2,
                },
                source_id: "axial".into(),
            },
        );
    }
    let partial = calculate(&task, &basis).unwrap();
    assert_eq!(partial.dimensions().tangent_to_tangent_height_m, None);
    assert_eq!(partial.missing_axial_sections().len(), 6);
    assert_eq!(partial.height_issues().len(), 6);
    near(
        partial.dimensions().minimum_diameter_m,
        std::f64::consts::FRAC_2_SQRT_PI,
    );
    basis.sources[1].kind = SourceKind::Assumed;
    basis.sources[1]
        .applicability
        .operating_conditions
        .as_mut()
        .unwrap()
        .pressure_pa *= 2.0;
    let partial = calculate(&task, &basis).unwrap();
    assert!(
        partial
            .height_issues()
            .iter()
            .all(|i| i.kind == SizingIssueKind::SourceConditionMismatch)
    );
    assert_eq!(partial.dimensions().tangent_to_tangent_height_m, None);
    basis.sources[1].applicability.operating_conditions = None;
    let complete = calculate(&task, &basis).unwrap();
    assert!(complete.height_issues().is_empty());
    near(
        complete.dimensions().tangent_to_tangent_height_m.unwrap(),
        1.8,
    );
}

#[test]
fn source_phase_and_override_evidence_follow_the_effective_binding() {
    let (mut task, mut basis) = fixture();
    basis.basis_entries.push(BasisEntry {
        id: "old-basis".into(),
        revision: 1,
        name: "Original time".into(),
        value: EngineeringValue {
            quantity: QuantityKind::Duration,
            value_si: 100.0,
        },
        source_id: "old-source".into(),
    });
    let mut old = basis.sources[0].clone();
    old.id = "old-source".into();
    old.kind = SourceKind::Unknown;
    basis.sources.push(old);
    task.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::Override {
            basis_entry_id: "old-basis".into(),
            value: EngineeringValue {
                quantity: QuantityKind::Duration,
                value_si: 120.0,
            },
            source_id: "synthetic".into(),
            reason: "Explicit sourced override".into(),
        },
    );
    assert!(calculate(&task, &basis).is_ok());
    if let InputBinding::Override { reason, .. } = task
        .case
        .inputs
        .get_mut(&SeparatorInput::WorkingVolumeTime)
        .unwrap()
    {
        reason.clear();
    }
    assert!(calculate(&task, &basis).is_err());
    task.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::BasisRef {
            basis_entry_id: "old-basis".into(),
        },
    );
    rejects(&task, &basis, SizingIssueKind::MissingSourceEvidence);
    set(&mut task, SeparatorInput::WorkingVolumeTime, 120.0);
    basis.sources[0].applicability.phase = SourcePhase::Gas;
    let issues = check(&task, &basis);
    assert!(
        issues
            .iter()
            .any(|i| i.target == IssueTarget::Input(SeparatorInput::LiquidDensity))
    );
    assert!(
        !issues
            .iter()
            .any(|i| i.target == IssueTarget::Input(SeparatorInput::GasDensity))
    );
}

#[test]
fn applicability_needs_explicit_resolvable_evidence_not_just_configuration_labels() {
    let (mut task, mut basis) = fixture();
    task.case.applicability_source_ids.clear();
    rejects(&task, &basis, SizingIssueKind::MissingSourceEvidence);
    task.case.applicability_source_ids.push("missing".into());
    rejects(
        &task,
        &basis,
        SizingIssueKind::MissingSource(SourceId::from("missing")),
    );
    let mut evidence = basis.sources[0].clone();
    evidence.id = "scope-evidence".into();
    evidence.review = SourceReview::Unreviewed;
    task.case.applicability_source_ids = vec![evidence.id.clone()];
    basis.sources.push(evidence);
    rejects(&task, &basis, SizingIssueKind::SourceNotAdopted);
    basis.sources[1].review = SourceReview::Adopted;
    let candidate = calculate(&task, &basis).unwrap();
    assert!(
        candidate
            .capture()
            .sources
            .contains_key(&SourceId::from("scope-evidence"))
    );
    basis.sources[1].applicability.fluid_system = "different fluid".into();
    rejects(&task, &basis, SizingIssueKind::SourceConditionMismatch);
}
