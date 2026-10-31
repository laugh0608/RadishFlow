use super::*;
use rf_model::engineering::{EngineeringBasis, SourceKind, SourceReview};

pub fn calculate(
    task: &SeparatorRatingTask,
    basis: &EngineeringBasis,
) -> Result<SeparatorRatingResult, RatingFailure> {
    let (capture, height_issues) = evidence::prepare(task, basis)?;
    calculate_capture(capture, height_issues)
}

/// Calculate only the explicitly requested member of a version 1 or 2 task.
pub fn calculate_case(
    task: &SeparatorRatingTask,
    case_id: &EngineeringCaseId,
    basis: &EngineeringBasis,
) -> Result<SeparatorRatingResult, RatingFailure> {
    let (capture, height_issues) = evidence::prepare_case(task, case_id, basis)?;
    calculate_capture(capture, height_issues)
}

fn calculate_capture(
    capture: RatingInputCapture,
    height_issues: Vec<RatingIssue>,
) -> Result<SeparatorRatingResult, RatingFailure> {
    use CalculationStep::*;
    use SeparatorRatingInput::*;
    let v = |field| capture.values_si[&field];
    let diameter = capture.geometry_values_m[&SeparatorGeometryInput::InternalDiameter];
    let height = capture.geometry_values_m[&SeparatorGeometryInput::WorkingLiquidHeight];
    let diameter_squared = positive(diameter * diameter, DiameterSquared)?;
    let area = positive(diameter_squared * std::f64::consts::FRAC_PI_4, GivenArea)?;
    let available_volume = positive(area * height, AvailableWorkingVolume)?;
    let velocities = crate::separator_hydraulics::gas_velocities(
        v(GasDensity),
        v(LiquidDensity),
        v(EffectiveK),
        v(GasVelocityUtilization),
    )
    .map_err(|step| {
        use crate::separator_hydraulics::VelocityStep;
        numeric_failure(match step {
            VelocityStep::DensityRatio => DensityRatio,
            VelocityStep::SoudersBrownVelocity => SoudersBrownVelocity,
            VelocityStep::AllowedVelocity => AllowedVelocity,
        })
    })?;
    let gas_limit = positive(area * velocities.allowed_m_s, GasFlowLimit)?;
    let required_volume = positive(
        v(LiquidVolumeFlow) * v(RequiredWorkingTime),
        RequiredWorkingVolume,
    )?;
    let available_time = positive(available_volume / v(LiquidVolumeFlow), AvailableWorkingTime)?;
    let liquid_limit = positive(available_volume / v(RequiredWorkingTime), LiquidFlowLimit)?;
    let gas_velocity = positive(v(GasVolumeFlow) / area, GasSuperficialVelocity)?;
    let gas_load = positive(v(GasVolumeFlow) / gas_limit, GasLoad)?;
    let liquid_load = positive(required_volume / available_volume, LiquidLoad)?;
    let missing_axial_sections: Vec<_> = SeparatorAxialSection::ALL
        .into_iter()
        .filter(|s| !capture.axial_lengths_m.contains_key(s))
        .collect();
    let total_height = if missing_axial_sections.is_empty() {
        let mut total = height;
        for length in capture.axial_lengths_m.values() {
            total = positive(total + length, TangentHeight)?;
        }
        Some(total)
    } else {
        None
    };
    let buffer_volume = capture
        .axial_lengths_m
        .get(&SeparatorAxialSection::NormalToHighLiquidLevel)
        .map(|length| {
            if *length == 0.0 {
                Ok(0.0)
            } else {
                positive(area * length, BufferVolume)
            }
        })
        .transpose()?;
    let assumed_sources = capture
        .sources
        .values()
        .filter(|s| s.kind == SourceKind::Assumed)
        .map(|s| s.id.clone())
        .collect();
    let pending_review_sources = capture
        .sources
        .values()
        .filter(|s| s.review != SourceReview::Reviewed)
        .map(|s| s.id.clone())
        .collect();
    Ok(SeparatorRatingResult {
        capture,
        performance: SeparatorPerformance {
            given_area_m2: area,
            available_working_volume_m3: available_volume,
            required_working_volume_m3: required_volume,
            souders_brown_velocity_m_s: velocities.souders_brown_m_s,
            allowed_gas_velocity_m_s: velocities.allowed_m_s,
            actual_gas_velocity_m_s: gas_velocity,
            gas_flow_limit_m3_s: gas_limit,
            liquid_flow_limit_m3_s: liquid_limit,
            available_working_time_s: available_time,
            gas: ConstraintAssessment::from_load(gas_load),
            liquid: ConstraintAssessment::from_load(liquid_load),
            tangent_to_tangent_height_m: total_height,
            buffer_volume_m3: buffer_volume,
        },
        assumed_sources,
        pending_review_sources,
        missing_axial_sections,
        height_issues,
    })
}

fn positive(value: f64, step: CalculationStep) -> Result<f64, RatingFailure> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(numeric_failure(step))
    }
}
fn numeric_failure(step: CalculationStep) -> RatingFailure {
    RatingFailure {
        issues: vec![RatingIssue {
            target: RatingIssueTarget::Calculation,
            kind: RatingIssueKind::NumericRange(step),
        }],
    }
}
