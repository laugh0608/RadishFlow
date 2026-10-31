use super::*;

fn matrix_fixture() -> (SeparatorRatingTask, EngineeringBasis) {
    let (mut task, basis) = fixture();
    task.method = matrix_method();
    geometry(
        &mut task,
        SeparatorGeometryInput::InternalDiameter,
        (8.0 / std::f64::consts::PI).sqrt(),
    );
    geometry(&mut task, SeparatorGeometryInput::WorkingLiquidHeight, 1.0);
    task.cases[0].id = "gas".into();
    set(&mut task, SeparatorRatingInput::GasVolumeFlow, 1.5);
    set(&mut task, SeparatorRatingInput::LiquidVolumeFlow, 0.005);
    let mut second = task.cases[0].clone();
    second.id = "liquid".into();
    second.inputs.insert(
        SeparatorRatingInput::GasVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, 0.8, "operating"),
    );
    second.inputs.insert(
        SeparatorRatingInput::LiquidVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, 0.015, "operating"),
    );
    task.cases.push(second);
    (task, basis)
}
fn row(task: &SeparatorRatingTask, basis: &EngineeringBasis, id: &str) -> SeparatorRatingResult {
    calculate_case(task, &id.into(), basis).unwrap()
}

#[test]
fn r03_r04_control_different_criteria_on_the_same_geometry() {
    let (task, basis) = matrix_fixture();
    let gas = row(&task, &basis, "gas");
    let liquid = row(&task, &basis, "liquid");
    near(gas.performance().gas.load_fraction, 0.9375);
    near(gas.performance().liquid.load_fraction, 0.3);
    near(liquid.performance().gas.load_fraction, 0.5);
    near(liquid.performance().liquid.load_fraction, 0.9);
    near(liquid.performance().available_working_time_s, 400.0 / 3.0);
    assert!(gas.performance().tangent_to_tangent_height_m.is_none());
    let summary = summarize(&task, &[&gas, &liquid]).unwrap();
    assert_eq!(summary.coverage, MatrixCoverage::Complete);
    assert!(summary.all_cases_within_criteria());
    assert_eq!(summary.gas.as_ref().unwrap().case_ids, vec!["gas".into()]);
    assert_eq!(
        summary.liquid.as_ref().unwrap().case_ids,
        vec!["liquid".into()]
    );
    near(summary.gas.unwrap().maximum_load_fraction, 0.9375);
    near(summary.liquid.unwrap().maximum_load_fraction, 0.9);
}

#[test]
fn explicit_case_validation_preserves_versions_and_rejects_ambiguous_identities() {
    let (single, basis) = fixture();
    assert_eq!(
        calculate(&single, &basis).unwrap(),
        row(&single, &basis, "rating-case-1")
    );
    let mut one_v2 = single.clone();
    one_v2.method = matrix_method();
    assert!(check_case(&one_v2, &one_v2.cases[0].id, &basis).is_empty());
    assert!(calculate(&one_v2, &basis).is_err()); // Legacy API does not expand silently.
    let (mut task, basis) = matrix_fixture();
    assert!(check_case(&task, &"gas".into(), &basis).is_empty());
    assert!(
        calculate_case(&task, &"absent".into(), &basis)
            .unwrap_err()
            .issues
            .iter()
            .any(|i| i.target == RatingIssueTarget::Case("absent".into()))
    );
    task.method = method();
    assert!(
        check_case(&task, &"gas".into(), &basis)
            .iter()
            .any(|i| matches!(i.kind, RatingIssueKind::UnsupportedCaseCount { actual: 2 }))
    );
    task.method = matrix_method();
    task.cases[1].id = "gas".into();
    assert!(calculate_case(&task, &"gas".into(), &basis).is_err());
    task.cases.clear();
    assert!(summarize(&task, &[]).is_err());
    task = one_v2;
    task.method.version = 4;
    assert!(calculate_case(&task, &task.cases[0].id, &basis).is_err());
}

#[test]
fn selected_row_capture_and_errors_do_not_include_unrelated_members() {
    let (mut task, mut basis) = matrix_fixture();
    let before = row(&task, &basis, "gas");
    let mut source = basis.sources[0].clone();
    source.id = "other-source".into();
    basis.sources.push(source);
    task.cases[1].inputs.insert(
        SeparatorRatingInput::GasVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, 0.0, "other-source"),
    );
    task.cases[1]
        .inputs
        .remove(&SeparatorRatingInput::LiquidDensity);
    task.cases[1].applicability.service = Some(SeparatorService::Foaming);
    let after = row(&task, &basis, "gas");
    assert_eq!(before, after);
    assert!(!after.capture().sources.contains_key(&"other-source".into()));
    assert_eq!(after.capture().input.case.id.as_str(), "gas");
    let failure = calculate_case(&task, &"liquid".into(), &basis).unwrap_err();
    assert!(
        failure
            .issues
            .iter()
            .any(|i| i.kind == RatingIssueKind::AbsentPhase)
    );
    assert!(
        failure
            .issues
            .iter()
            .any(|i| i.kind == RatingIssueKind::OutsideMethodScope)
    );
    assert!(
        failure
            .issues
            .iter()
            .any(|i| i.kind == RatingIssueKind::MissingInput)
    );
    let summary = summarize(&task, &[&after]).unwrap();
    assert_eq!(summary.required_case_ids.len(), 2);
    assert_eq!(summary.coverage, MatrixCoverage::Incomplete);
    assert!(!summary.all_cases_within_criteria());
}

#[test]
fn incomplete_coverage_exceedance_boundary_and_no_results_remain_distinct() {
    let (mut task, basis) = matrix_fixture();
    let empty = summarize(&task, &[]).unwrap();
    assert_eq!(empty.required_case_ids.len(), 2);
    assert_eq!(empty.coverage, MatrixCoverage::Incomplete);
    assert_eq!(empty.subset_conclusion, None);
    assert_eq!(empty.gas, None);
    assert_eq!(empty.liquid, None);
    for (flow, expected) in [
        (1.6, ConstraintStatus::BoundaryReview),
        (2.0, ConstraintStatus::Exceeded),
    ] {
        set(&mut task, SeparatorRatingInput::GasVolumeFlow, flow);
        let gas = row(&task, &basis, "gas");
        let partial = summarize(&task, &[&gas]).unwrap();
        assert_eq!(partial.coverage, MatrixCoverage::Incomplete);
        assert_eq!(partial.subset_conclusion, Some(expected));
        assert!(!partial.all_cases_within_criteria());
        let liquid = row(&task, &basis, "liquid");
        let complete = summarize(&task, &[&gas, &liquid]).unwrap();
        assert_eq!(complete.coverage, MatrixCoverage::Complete);
        assert_eq!(complete.subset_conclusion, Some(expected));
        assert!(!complete.all_cases_within_criteria());
    }
}

#[test]
fn ties_are_anchored_to_raw_maximum_without_chaining_or_order_dependence() {
    let (mut task, basis) = matrix_fixture();
    let template = task.cases[0].clone();
    task.cases.clear();
    // Adjacent differences are within tolerance, but the low end is not tied to max.
    for (id, delta) in [
        ("maximum", 0.0),
        ("near", -0.75e-10),
        ("chain", -1.5e-10),
        ("equal", 0.0),
    ] {
        let mut case = template.clone();
        case.id = id.into();
        case.inputs.insert(
            SeparatorRatingInput::GasVolumeFlow,
            direct(
                QuantityKind::ActualVolumeFlow,
                1.6 * (0.9 + delta),
                "operating",
            ),
        );
        task.cases.push(case);
    }
    let mut results: Vec<_> = task
        .cases
        .iter()
        .map(|c| calculate_case(&task, &c.id, &basis).unwrap())
        .collect();
    let first = summarize(&task, &results.iter().collect::<Vec<_>>()).unwrap();
    let control = first.gas.as_ref().unwrap();
    near(control.maximum_load_fraction, 0.9);
    assert_eq!(
        control.case_ids,
        vec!["equal".into(), "maximum".into(), "near".into()]
    );
    task.cases.reverse();
    results.reverse();
    let reversed = summarize(&task, &results.iter().collect::<Vec<_>>()).unwrap();
    assert_eq!(first.gas, reversed.gas);
    assert_eq!(first.liquid, reversed.liquid);
    assert_eq!(first.subset_conclusion, reversed.subset_conclusion);
}

#[test]
fn tied_rows_keep_their_own_boundary_and_exceeded_classification() {
    let (mut task, basis) = matrix_fixture();
    set(
        &mut task,
        SeparatorRatingInput::GasVolumeFlow,
        1.6 * (1.0 + 0.75e-10),
    );
    task.cases[1].inputs.insert(
        SeparatorRatingInput::GasVolumeFlow,
        direct(
            QuantityKind::ActualVolumeFlow,
            1.6 * (1.0 + 1.25e-10),
            "operating",
        ),
    );
    let gas = row(&task, &basis, "gas");
    let liquid = row(&task, &basis, "liquid");
    assert_eq!(
        gas.performance().gas.status,
        ConstraintStatus::BoundaryReview
    );
    assert_eq!(liquid.performance().gas.status, ConstraintStatus::Exceeded);
    assert!(liquid.performance().gas.margin_fraction < 0.0);
    let summary = summarize(&task, &[&gas, &liquid]).unwrap();
    assert_eq!(summary.gas.unwrap().case_ids.len(), 2);
    assert_eq!(summary.subset_conclusion, Some(ConstraintStatus::Exceeded));
}

#[test]
fn matrix_rejects_foreign_duplicate_unknown_and_mismatched_captures() {
    let (task, basis) = matrix_fixture();
    let gas = row(&task, &basis, "gas");
    assert!(matches!(
        summarize(&task, &[&gas, &gas]),
        Err(RatingMatrixError::DuplicateCase(_))
    ));
    let mut other = task.clone();
    other.id = "other-task".into();
    assert!(matches!(
        summarize(&other, &[&gas]),
        Err(RatingMatrixError::ForeignTask(_))
    ));
    other = task.clone();
    other.cases.remove(0);
    assert!(matches!(
        summarize(&other, &[&gas]),
        Err(RatingMatrixError::UnknownCase(_))
    ));
    other = task.clone();
    other.geometry.revision += 1;
    assert!(matches!(
        summarize(&other, &[&gas]),
        Err(RatingMatrixError::GeometryMismatch(_))
    ));
    other = task.clone();
    set(&mut other, SeparatorRatingInput::EffectiveK, 0.2);
    assert!(matches!(
        summarize(&other, &[&gas]),
        Err(RatingMatrixError::CaseInputMismatch(_))
    ));
    other = task.clone();
    other.cases.truncate(1);
    other.method = method();
    assert!(matches!(
        summarize(&other, &[&gas]),
        Err(RatingMatrixError::MethodMismatch(_))
    ));
    // Presentation-only edits keep the original immutable evidence usable.
    other = task.clone();
    other.revision += 1;
    other.tag = "V-102".into();
    other.name = "Renamed".into();
    other.cases[0].revision += 1;
    other.cases[0].name = "Renamed case".into();
    other.cases[0].purpose = "Different label".into();
    assert!(summarize(&other, &[&gas]).is_ok());
}

#[test]
fn selected_rows_use_their_own_conditions_sources_and_criteria() {
    let (mut task, mut basis) = matrix_fixture();
    let mut source = basis.sources[0].clone();
    source.id = "hot-operation".into();
    source.applicability.fluid_system = "Other synthetic fluid".into();
    source.applicability.operating_conditions = Some(SourceOperatingConditions {
        temperature_k: 350.0,
        pressure_pa: 200000.0,
    });
    basis.sources.push(source.clone());
    let case = &mut task.cases[1];
    case.fluid_system = source.applicability.fluid_system.clone();
    case.applicability_source_ids = vec![source.id.clone()];
    for binding in case.inputs.values_mut() {
        if let InputBinding::Direct { source_id, .. } = binding {
            *source_id = source.id.clone();
        }
    }
    for (field, value) in [
        (SeparatorRatingInput::OperatingTemperature, 350.0),
        (SeparatorRatingInput::OperatingPressure, 200000.0),
        (SeparatorRatingInput::EffectiveK, 0.2),
        (SeparatorRatingInput::GasVelocityUtilization, 0.5),
        (SeparatorRatingInput::RequiredWorkingTime, 60.0),
    ] {
        case.inputs
            .insert(field, direct(field.quantity(), value, "hot-operation"));
    }
    let gas = row(&task, &basis, "gas");
    let liquid = row(&task, &basis, "liquid");
    near(gas.performance().gas_flow_limit_m3_s, 1.6);
    near(liquid.performance().gas_flow_limit_m3_s, 2.0);
    near(liquid.performance().liquid.load_fraction, 0.45);
    assert!(!liquid.capture().sources.contains_key(&"operating".into()));
    assert!(liquid.capture().sources.contains_key(&source.id));
    assert!(
        summarize(&task, &[&gas, &liquid])
            .unwrap()
            .all_cases_within_criteria()
    );
    task.cases[1].inputs.insert(
        SeparatorRatingInput::OperatingPressure,
        direct(QuantityKind::AbsolutePressure, 300000.0, "hot-operation"),
    );
    assert!(
        check_case(&task, &"liquid".into(), &basis)
            .iter()
            .any(|i| i.kind == RatingIssueKind::SourceConditionMismatch)
    );
    assert_eq!(gas, row(&task, &basis, "gas"));
}

#[test]
fn common_geometry_cannot_mix_different_resolved_basis_values() {
    let (mut task, mut basis) = matrix_fixture();
    basis.basis_entries.push(BasisEntry {
        id: "diameter".into(),
        revision: 1,
        name: "Shared diameter".into(),
        value: EngineeringValue {
            quantity: QuantityKind::Length,
            value_si: (8.0 / std::f64::consts::PI).sqrt(),
        },
        source_id: "drawing".into(),
    });
    task.geometry.inputs.insert(
        SeparatorGeometryInput::InternalDiameter,
        InputBinding::BasisRef {
            basis_entry_id: "diameter".into(),
        },
    );
    let gas = row(&task, &basis, "gas");
    basis.basis_entries[0].value.value_si = 2.0;
    basis.basis_entries[0].revision += 1;
    let liquid = row(&task, &basis, "liquid");
    assert!(matches!(
        summarize(&task, &[&gas, &liquid]),
        Err(RatingMatrixError::GeometryMismatch(_))
    ));
    assert!(matches!(
        summarize(&task, &[&liquid, &gas]),
        Err(RatingMatrixError::GeometryMismatch(_))
    ));
}

#[test]
fn matrix_non_si_equivalence_and_arithmetic_failure_are_row_local() {
    let (mut task, basis) = matrix_fixture();
    let expected = row(&task, &basis, "gas");
    for (field, value, unit) in [
        (
            SeparatorRatingInput::OperatingTemperature,
            26.85,
            MeasurementUnit::Celsius,
        ),
        (
            SeparatorRatingInput::OperatingPressure,
            1.0,
            MeasurementUnit::Bar,
        ),
        (
            SeparatorRatingInput::GasVolumeFlow,
            5400.0,
            MeasurementUnit::CubicMetrePerHour,
        ),
        (
            SeparatorRatingInput::LiquidVolumeFlow,
            18.0,
            MeasurementUnit::CubicMetrePerHour,
        ),
        (
            SeparatorRatingInput::RequiredWorkingTime,
            2.0,
            MeasurementUnit::Minute,
        ),
    ] {
        set(
            &mut task,
            field,
            to_canonical(value, field.quantity(), unit).unwrap(),
        );
    }
    geometry(
        &mut task,
        SeparatorGeometryInput::WorkingLiquidHeight,
        to_canonical(1000.0, QuantityKind::Length, MeasurementUnit::Millimetre).unwrap(),
    );
    let converted = row(&task, &basis, "gas");
    assert_eq!(converted.performance(), expected.performance());
    task.cases[1].inputs.insert(
        SeparatorRatingInput::RequiredWorkingTime,
        direct(QuantityKind::Duration, f64::MAX, "operating"),
    );
    task.cases[1].inputs.insert(
        SeparatorRatingInput::LiquidVolumeFlow,
        direct(QuantityKind::ActualVolumeFlow, 2.0, "operating"),
    );
    assert!(
        calculate_case(&task, &"liquid".into(), &basis)
            .unwrap_err()
            .issues
            .iter()
            .any(
                |i| i.kind == RatingIssueKind::NumericRange(CalculationStep::RequiredWorkingVolume)
            )
    );
    assert_eq!(row(&task, &basis, "gas"), converted);
    let summary = summarize(&task, &[&converted]).unwrap();
    assert_eq!(summary.coverage, MatrixCoverage::Incomplete);
    assert_eq!(summary.evaluated_case_ids, vec!["gas".into()]);
}
