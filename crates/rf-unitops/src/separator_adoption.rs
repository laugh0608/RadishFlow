//! Pure mapping of a successful preliminary size into a fixed design decision.
//! Application code owns eligibility, identities, publication and history.
use crate::separator_sizing::SeparatorCandidate;
use rf_model::{engineering::EngineeringBasis, equipment::*};

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionAdjustment {
    pub value_m: f64,
    pub reason: String,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeometryAdjustments {
    pub diameter: Option<DimensionAdjustment>,
    pub working_liquid_height: Option<DimensionAdjustment>,
}
fn dimension(
    original_m: f64,
    adjustment: Option<DimensionAdjustment>,
) -> Result<AdoptedDimension, GeometryAdoptionError> {
    let value = match adjustment {
        Some(a) => {
            if a.reason.trim().is_empty() {
                return Err(GeometryAdoptionError::MissingAdjustmentReason);
            }
            AdoptedDimension {
                original_m,
                adopted_m: a.value_m,
                reason: a.reason,
            }
        }
        None => AdoptedDimension {
            original_m,
            adopted_m: original_m,
            reason: String::new(),
        },
    };
    value.validate()?;
    Ok(value)
}
pub fn map_candidate(
    candidate: &SeparatorCandidate,
    document_id: String,
    adjustments: GeometryAdjustments,
) -> Result<SeparatorGeometryAdoption, GeometryAdoptionError> {
    let capture = candidate.capture();
    let result = SeparatorGeometryAdoption {
        source_document_id: document_id,
        source_task: capture.task.clone(),
        source_basis: EngineeringBasis {
            basis_entries: capture.basis_entries.values().cloned().collect(),
            sources: capture.sources.values().cloned().collect(),
        },
        diameter: dimension(
            candidate.dimensions().minimum_diameter_m,
            adjustments.diameter,
        )?,
        working_liquid_height: dimension(
            candidate.dimensions().working_liquid_height_m,
            adjustments.working_liquid_height,
        )?,
        axial_lengths_m: capture.axial_lengths_m.clone(),
    };
    result.validate()?;
    Ok(result)
}

/// The maximum allowed height is a sizing criterion, never a rating case input.
pub const fn sizing_field(field: SeparatorRatingInput) -> SeparatorInput {
    use SeparatorRatingInput::*;
    match field {
        OperatingTemperature => SeparatorInput::OperatingTemperature,
        OperatingPressure => SeparatorInput::OperatingPressure,
        GasVolumeFlow => SeparatorInput::GasVolumeFlow,
        LiquidVolumeFlow => SeparatorInput::LiquidVolumeFlow,
        GasDensity => SeparatorInput::GasDensity,
        LiquidDensity => SeparatorInput::LiquidDensity,
        EffectiveK => SeparatorInput::EffectiveK,
        GasVelocityUtilization => SeparatorInput::GasVelocityUtilization,
        RequiredWorkingTime => SeparatorInput::WorkingVolumeTime,
    }
}
