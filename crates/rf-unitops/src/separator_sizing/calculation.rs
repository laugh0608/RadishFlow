use super::*;
use rf_model::engineering::{EngineeringBasis, SourceKind, SourceReview};

pub fn calculate(
    task: &SeparatorSizingTask,
    basis: &EngineeringBasis,
) -> Result<SeparatorCandidate, SizingFailure> {
    let (capture, height_issues) = evidence::prepare(task, basis)?;
    use CalculationStep::*;
    use SeparatorInput::*;
    let v = |field| capture.values_si[&field];
    let velocities = crate::separator_hydraulics::gas_velocities(
        v(GasDensity),
        v(LiquidDensity),
        v(EffectiveK),
        v(GasVelocityUtilization),
    )
    .map_err(|step| {
        use crate::separator_hydraulics::VelocityStep;
        let step = match step {
            VelocityStep::DensityRatio => DensityRatio,
            VelocityStep::SoudersBrownVelocity => SoudersBrownVelocity,
            VelocityStep::AllowedVelocity => AllowedVelocity,
        };
        SizingFailure {
            issues: vec![SizingIssue {
                target: IssueTarget::Calculation,
                kind: SizingIssueKind::NumericRange(step),
            }],
        }
    })?;
    let u_sb = velocities.souders_brown_m_s;
    let u_allow = velocities.allowed_m_s;
    let gas_area = positive(v(GasVolumeFlow) / u_allow, GasArea)?;
    let volume = positive(v(LiquidVolumeFlow) * v(WorkingVolumeTime), WorkingVolume)?;
    let liquid_area = positive(volume / v(MaximumWorkingLiquidHeight), LiquidArea)?;
    let area = gas_area.max(liquid_area);
    let diameter_squared = positive(
        positive(4.0 * area, DiameterSquared)? / std::f64::consts::PI,
        DiameterSquared,
    )?;
    let diameter = positive(diameter_squared.sqrt(), Diameter)?;
    let height = positive(volume / area, WorkingHeight)?;
    let gas_velocity = positive(v(GasVolumeFlow) / area, GasSuperficialVelocity)?;
    let gas_load = positive(gas_velocity / u_allow, GasLoad)?;
    let control = if evidence::numerically_equal(gas_area, liquid_area) {
        ControllingConstraint::Both
    } else if gas_area > liquid_area {
        ControllingConstraint::GasArea
    } else {
        ControllingConstraint::LiquidWorkingVolume
    };
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
    Ok(SeparatorCandidate {
        capture,
        dimensions: SeparatorDimensions {
            souders_brown_velocity_m_s: u_sb,
            allowed_gas_velocity_m_s: u_allow,
            gas_area_m2: gas_area,
            liquid_area_m2: liquid_area,
            required_area_m2: area,
            working_volume_m3: volume,
            minimum_diameter_m: diameter,
            working_liquid_height_m: height,
            gas_load_fraction: gas_load,
            controlling_constraint: control,
            tangent_to_tangent_height_m: total_height,
            buffer_volume_m3: buffer_volume,
        },
        assumed_sources,
        pending_review_sources,
        missing_axial_sections,
        height_issues,
    })
}

fn positive(value: f64, step: CalculationStep) -> Result<f64, SizingFailure> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(SizingFailure {
            issues: vec![SizingIssue {
                target: IssueTarget::Calculation,
                kind: SizingIssueKind::NumericRange(step),
            }],
        })
    }
}
