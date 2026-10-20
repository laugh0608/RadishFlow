use super::*;
use ConstraintStatus::*;
use SeparatorGeometryInput::*;
use SeparatorRatingInput::*;

#[test]
fn r01_given_geometry_matches_independent_boundary_values_without_mutation() {
    let (task, basis) = fixture();
    let before = (task.clone(), basis.clone());
    assert!(check(&task, &basis).is_empty());
    let result = calculate(&task, &basis).unwrap();
    let p = result.performance();
    for (actual, expected) in [
        (p.given_area_m2, 1.0),
        (p.available_working_volume_m3, 0.6),
        (p.required_working_volume_m3, 0.6),
        (p.souders_brown_velocity_m_s, 1.0),
        (p.allowed_gas_velocity_m_s, 0.8),
        (p.actual_gas_velocity_m_s, 0.8),
        (p.gas_flow_limit_m3_s, 0.8),
        (p.liquid_flow_limit_m3_s, 0.005),
        (p.available_working_time_s, 120.0),
        (p.gas.load_fraction, 1.0),
        (p.liquid.load_fraction, 1.0),
        (p.gas.margin_fraction, 0.0),
        (p.liquid.margin_fraction, 0.0),
    ] {
        near(actual, expected);
    }
    assert_eq!(p.gas.status, BoundaryReview);
    assert_eq!(p.liquid.status, BoundaryReview);
    assert_eq!(p.conclusion(), BoundaryReview);
    assert_eq!(p.tangent_to_tangent_height_m, None);
    assert_eq!(p.buffer_volume_m3, None);
    assert_eq!(result.missing_axial_sections().len(), 6);
    assert!(result.height_issues().is_empty());
    assert_eq!(result.unchecked_requirements().len(), 7);
    assert_eq!(result.assumed_sources(), &[SourceId::from("operating")]);
    assert_eq!(
        result.pending_review_sources(),
        &[SourceId::from("operating")]
    );
    assert_eq!((task, basis), before);
}

#[test]
fn r02_exceeded_checks_remain_valid_results_with_negative_margins() {
    let (mut task, basis) = fixture();
    set(&mut task, GasVolumeFlow, 1.0);
    set(&mut task, LiquidVolumeFlow, 0.006);
    let result = calculate(&task, &basis).unwrap();
    let p = result.performance();
    near(p.gas.load_fraction, 1.25);
    near(p.liquid.load_fraction, 1.2);
    near(p.gas.margin_fraction, -0.25);
    near(p.liquid.margin_fraction, -0.2);
    near(p.available_working_time_s, 100.0);
    near(p.available_working_volume_m3, 0.6);
    near(p.required_working_volume_m3, 0.72);
    assert_eq!(p.conclusion(), Exceeded);
    assert_eq!(
        result.capture().geometry_values_m[&WorkingLiquidHeight],
        0.6
    );
}

#[test]
fn r03_r04_two_separate_calls_use_identical_geometry_and_different_limiting_loads() {
    let (mut task, basis) = fixture();
    geometry(
        &mut task,
        InternalDiameter,
        (8.0 / std::f64::consts::PI).sqrt(),
    );
    geometry(&mut task, WorkingLiquidHeight, 1.0);
    let common_geometry = task.geometry.clone();
    for (qg, ql, gas_load, liquid_load, time) in [
        (1.5, 0.005, 0.9375, 0.3, 400.0),
        (0.8, 0.015, 0.5, 0.9, 400.0 / 3.0),
    ] {
        set(&mut task, GasVolumeFlow, qg);
        set(&mut task, LiquidVolumeFlow, ql);
        let result = calculate(&task, &basis).unwrap();
        let p = result.performance();
        near(p.given_area_m2, 2.0);
        near(p.available_working_volume_m3, 2.0);
        near(p.gas_flow_limit_m3_s, 1.6);
        near(p.liquid_flow_limit_m3_s, 1.0 / 60.0);
        near(p.gas.load_fraction, gas_load);
        near(p.liquid.load_fraction, liquid_load);
        near(p.available_working_time_s, time);
        assert_eq!(p.conclusion(), WithinCriteria);
        assert_eq!(task.geometry, common_geometry);
    }
}

#[test]
fn load_boundary_is_numerical_review_not_automatic_pass_or_hidden_margin() {
    let samples = [
        (1.0 - 2e-10, WithinCriteria),
        (1.0 - 5e-11, BoundaryReview),
        (1.0, BoundaryReview),
        (1.0 + 5e-11, BoundaryReview),
        (1.0 + 2e-10, Exceeded),
    ];
    for (load, expected) in samples {
        let (mut task, basis) = fixture();
        set(&mut task, GasVolumeFlow, 0.8 * load);
        set(&mut task, LiquidVolumeFlow, 0.005 * load);
        let result = calculate(&task, &basis).unwrap();
        for assessment in [&result.performance().gas, &result.performance().liquid] {
            near(assessment.load_fraction, load);
            assert_eq!(assessment.status, expected);
            if load != 1.0 {
                assert_eq!(assessment.margin_fraction.is_sign_positive(), load < 1.0);
            }
        }
        assert_eq!(result.performance().conclusion(), expected);
    }
    // Adjacent representable values on each side of the literal tolerance.
    let upper = 1.0 + LOAD_BOUNDARY_TOLERANCE;
    assert_eq!(
        ConstraintAssessment::from_load(upper.next_down()).status,
        BoundaryReview
    );
    assert_eq!(
        ConstraintAssessment::from_load(upper.next_up()).status,
        Exceeded
    );
    let lower = 1.0 - LOAD_BOUNDARY_TOLERANCE;
    assert_eq!(
        ConstraintAssessment::from_load(lower.next_up()).status,
        BoundaryReview
    );
    assert_eq!(
        ConstraintAssessment::from_load(lower.next_down()).status,
        WithinCriteria
    );
    let (mut task, basis) = fixture();
    set(&mut task, GasVolumeFlow, 1.0); // Exceeded dominates liquid boundary.
    assert_eq!(
        calculate(&task, &basis).unwrap().performance().conclusion(),
        Exceeded
    );
}

#[test]
fn non_si_inputs_are_converted_once_and_rating_preserves_canonical_precision() {
    let (mut task, basis) = fixture();
    for (field, value, unit) in [
        (OperatingTemperature, 26.85, MeasurementUnit::Celsius),
        (OperatingPressure, 1.0, MeasurementUnit::Bar),
        (GasVolumeFlow, 2880.0, MeasurementUnit::CubicMetrePerHour),
        (LiquidVolumeFlow, 18.0, MeasurementUnit::CubicMetrePerHour),
        (RequiredWorkingTime, 2.0, MeasurementUnit::Minute),
    ] {
        set(
            &mut task,
            field,
            to_canonical(value, field.quantity(), unit).unwrap(),
        );
    }
    geometry(
        &mut task,
        WorkingLiquidHeight,
        to_canonical(600.0, QuantityKind::Length, MeasurementUnit::Millimetre).unwrap(),
    );
    let result = calculate(&task, &basis).unwrap();
    near(result.performance().gas_flow_limit_m3_s, 0.8);
    near(result.performance().available_working_time_s, 120.0);
    assert_eq!(result.performance().conclusion(), BoundaryReview);
    near(result.capture().values_si[&OperatingTemperature], 300.0);
}

#[test]
fn given_geometry_rating_agrees_with_s1_relations_without_implicit_adoption() {
    use crate::separator_sizing as sizing;
    let (mut task, basis) = fixture();
    let mappings = [
        (OperatingTemperature, SeparatorInput::OperatingTemperature),
        (OperatingPressure, SeparatorInput::OperatingPressure),
        (GasVolumeFlow, SeparatorInput::GasVolumeFlow),
        (LiquidVolumeFlow, SeparatorInput::LiquidVolumeFlow),
        (GasDensity, SeparatorInput::GasDensity),
        (LiquidDensity, SeparatorInput::LiquidDensity),
        (EffectiveK, SeparatorInput::EffectiveK),
        (
            GasVelocityUtilization,
            SeparatorInput::GasVelocityUtilization,
        ),
        (RequiredWorkingTime, SeparatorInput::WorkingVolumeTime),
    ];
    for liquid_flow in [0.005, 0.02] {
        set(&mut task, LiquidVolumeFlow, liquid_flow);
        let case = &task.cases[0];
        let g = &task.geometry.applicability;
        let mut s = SeparatorSizingTask {
            id: "sizing".into(),
            revision: 1,
            tag: "V-102".into(),
            name: "S1 comparison".into(),
            method: sizing::method(),
            case: SeparatorCase {
                id: "sizing-case".into(),
                name: case.name.clone(),
                purpose: case.purpose.clone(),
                fluid_system: case.fluid_system.clone(),
                applicability_source_ids: case.applicability_source_ids.clone(),
                inputs: mappings
                    .iter()
                    .map(|(r, s)| (*s, case.inputs[r].clone()))
                    .collect(),
                applicability: SeparatorApplicability {
                    orientation: g.orientation,
                    gas_direction: g.gas_direction,
                    geometry: g.geometry,
                    internals_description: g.internals_description.clone(),
                    service: case.applicability.service,
                    k_basis: case.applicability.k_basis,
                    flow_basis: case.applicability.flow_basis,
                },
            },
            axial_sections: BTreeMap::new(),
        };
        s.case.inputs.insert(
            SeparatorInput::MaximumWorkingLiquidHeight,
            direct(QuantityKind::Length, 1.0, "operating"),
        );
        let sized = sizing::calculate(&s, &basis).unwrap();
        geometry(
            &mut task,
            InternalDiameter,
            sized.dimensions().minimum_diameter_m,
        );
        geometry(
            &mut task,
            WorkingLiquidHeight,
            sized.dimensions().working_liquid_height_m,
        );
        let rated = calculate(&task, &basis).unwrap();
        near(
            rated.performance().given_area_m2,
            sized.dimensions().required_area_m2,
        );
        near(
            rated.performance().allowed_gas_velocity_m_s,
            sized.dimensions().allowed_gas_velocity_m_s,
        );
        near(
            rated.performance().gas.load_fraction,
            sized.dimensions().gas_load_fraction,
        );
        near(rated.performance().liquid.load_fraction, 1.0);
    }
}

#[test]
fn arithmetic_overflow_and_underflow_reject_at_the_specific_step() {
    use CalculationStep::*;
    // Each fixture isolates a finite-input arithmetic failure. No Inf/NaN result
    // or silent zero is permitted, even when the remaining checks would pass.
    for (geometry_changes, inputs, expected) in [
        (vec![(InternalDiameter, f64::MAX)], vec![], DiameterSquared),
        (vec![(InternalDiameter, 1e-200)], vec![], DiameterSquared),
        (
            vec![(InternalDiameter, 1e154), (WorkingLiquidHeight, 10.0)],
            vec![],
            AvailableWorkingVolume,
        ),
        (vec![], vec![(GasDensity, 1e-308)], DensityRatio),
        (vec![], vec![(EffectiveK, 1e308)], SoudersBrownVelocity),
        (
            vec![],
            vec![(EffectiveK, 1e-308), (GasVelocityUtilization, 1e-100)],
            AllowedVelocity,
        ),
        (
            vec![(InternalDiameter, 1e153)],
            vec![(EffectiveK, 1e3)],
            GasFlowLimit,
        ),
        (
            vec![],
            vec![(LiquidVolumeFlow, f64::MAX)],
            RequiredWorkingVolume,
        ),
        (
            vec![],
            vec![
                (LiquidVolumeFlow, f64::from_bits(1)),
                (RequiredWorkingTime, 1e-100),
            ],
            RequiredWorkingVolume,
        ),
        (
            vec![],
            vec![(LiquidVolumeFlow, f64::from_bits(1))],
            AvailableWorkingTime,
        ),
        (vec![], vec![(RequiredWorkingTime, 1e-310)], LiquidFlowLimit),
        (
            vec![(InternalDiameter, 1.0)],
            vec![(GasVolumeFlow, f64::MAX)],
            GasSuperficialVelocity,
        ),
        (
            vec![],
            vec![(EffectiveK, 1e-300), (GasVolumeFlow, 1e20)],
            GasLoad,
        ),
        (
            vec![(InternalDiameter, 1e-100), (WorkingLiquidHeight, 1e-100)],
            vec![(LiquidVolumeFlow, 1e15)],
            LiquidLoad,
        ),
    ] {
        let (mut task, basis) = fixture();
        for (field, value) in geometry_changes {
            geometry(&mut task, field, value);
        }
        for (field, value) in inputs {
            set(&mut task, field, value);
        }
        rejects(&task, &basis, RatingIssueKind::NumericRange(expected));
    }
}

#[test]
fn axial_height_uses_given_working_layer_and_separate_buffer_without_double_counting() {
    let (mut task, basis) = fixture();
    for section in SeparatorAxialSection::ALL {
        task.geometry
            .axial_sections
            .insert(section, direct(QuantityKind::Length, 0.2, "drawing"));
    }
    let result = calculate(&task, &basis).unwrap();
    near(
        result.performance().tangent_to_tangent_height_m.unwrap(),
        1.8,
    );
    near(result.performance().buffer_volume_m3.unwrap(), 0.2);
    near(result.performance().available_working_volume_m3, 0.6);
    assert!(result.missing_axial_sections().is_empty());
    task.geometry
        .axial_sections
        .remove(&SeparatorAxialSection::LowerTangentToLowLiquidLevel);
    task.geometry.axial_sections.insert(
        SeparatorAxialSection::NormalToHighLiquidLevel,
        direct(QuantityKind::Length, 0.0, "drawing"),
    );
    let partial = calculate(&task, &basis).unwrap();
    assert_eq!(partial.performance().tangent_to_tangent_height_m, None);
    assert_eq!(partial.performance().buffer_volume_m3, Some(0.0));
    task.geometry.axial_sections.insert(
        SeparatorAxialSection::InletToInternals,
        direct(QuantityKind::Length, 0.0, "drawing"),
    );
    rejects(&task, &basis, RatingIssueKind::NonPositiveInput);
    for section in SeparatorAxialSection::ALL {
        task.geometry
            .axial_sections
            .insert(section, direct(QuantityKind::Length, 1e308, "drawing"));
    }
    rejects(
        &task,
        &basis,
        RatingIssueKind::NumericRange(CalculationStep::TangentHeight),
    );
    task.geometry.axial_sections.clear();
    geometry(&mut task, InternalDiameter, 1e153);
    task.geometry.axial_sections.insert(
        SeparatorAxialSection::NormalToHighLiquidLevel,
        direct(QuantityKind::Length, 1e4, "drawing"),
    );
    rejects(
        &task,
        &basis,
        RatingIssueKind::NumericRange(CalculationStep::BufferVolume),
    );
}
