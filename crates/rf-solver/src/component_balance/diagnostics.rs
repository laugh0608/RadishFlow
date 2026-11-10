//! Preserve equation/variable domain identity in the existing diagnostic boundary.
use super::*;
use rf_types::{DiagnosticPortTarget, RfError};

fn binding(error: RfError, value: &ComponentFlowBinding) -> RfError {
    let mut error = error
        .with_related_unit_id(value.unit_id.clone())
        .with_related_stream_id(value.stream_id.clone());
    let port = match value.port {
        ComponentBalancePort::FeedOutlet => rf_unitops::FEED_OUTLET_PORT,
        ComponentBalancePort::Mixer(port) => port.name(),
    };
    error = error.with_related_port_target(DiagnosticPortTarget::new(value.unit_id.clone(), port));
    error
}
fn equation(error: RfError, value: &ComponentBalanceEquation) -> RfError {
    match value {
        ComponentBalanceEquation::Connection { source, target } => {
            binding(binding(error, source), target)
        }
        ComponentBalanceEquation::MixerConservation { unit_id, .. } => {
            error.with_related_unit_id(unit_id.clone())
        }
    }
}
impl ComponentBalanceError {
    pub fn diagnostic(&self) -> RfError {
        if let Self::Model(error) = self {
            return error.clone();
        }
        let code = match self {
            Self::Model(_) => unreachable!(),
            Self::UnsupportedModel { .. } => "material.unsupported_model",
            Self::UnsupportedTopology { .. } => "material.unsupported_topology",
            Self::UnsupportedSpecification { .. } => "material.unsupported_specification",
            Self::UnsupportedThermalBoundary { .. } => "material.unsupported_thermal_boundary",
            Self::NoComponents => "material.no_components",
            Self::UnknownComponent { .. } => "material.unknown_component",
            Self::InvalidScale { .. } => "material.invalid_scale",
            Self::InvalidOutletSpecification => "material.invalid_outlet_specification",
            Self::RedundantOutletSpecification { .. } => "material.redundant_specification",
            Self::ConflictingOutletSpecification { .. } => "material.conflicting_specification",
            Self::InvalidOutletTotal { .. } => "material.invalid_outlet_total",
            Self::PointSize { .. } => "material.point_size",
            Self::InvalidTrialValue { .. } => "material.invalid_trial_value",
            Self::NonFiniteEquation { .. } => "material.non_finite_equation",
            Self::NonFiniteScaling { .. } => "material.non_finite_scaling",
            Self::UnsatisfiedEquation { .. } => "material.unsatisfied_equation",
        };
        let error = RfError::invalid_input(self.to_string()).with_diagnostic_code(code);
        match self {
            Self::UnsupportedModel { unit_id, .. }
            | Self::UnsupportedSpecification { unit_id, .. } => {
                error.with_related_unit_id(unit_id.clone())
            }
            Self::UnsupportedTopology { mixer_id }
            | Self::RedundantOutletSpecification { mixer_id, .. }
            | Self::ConflictingOutletSpecification { mixer_id, .. }
            | Self::InvalidOutletTotal { mixer_id } => error.with_related_unit_id(mixer_id.clone()),
            Self::UnsupportedThermalBoundary { stream_id }
            | Self::UnknownComponent { stream_id, .. } => {
                error.with_related_stream_id(stream_id.clone())
            }
            Self::InvalidTrialValue { binding: value, .. } => binding(error, value),
            Self::NonFiniteEquation { equation: value }
            | Self::NonFiniteScaling { equation: value }
            | Self::UnsatisfiedEquation {
                equation: value, ..
            } => equation(error, value),
            _ => error,
        }
    }
}
impl MaterialBalanceSolveError {
    pub fn diagnostic(&self) -> RfError {
        let error = RfError::invalid_input(self.to_string());
        match self {
            Self::InvalidTolerance { .. } => {
                error.with_diagnostic_code("material.invalid_tolerance")
            }
            Self::UnsupportedStructure { equation: value } => equation(
                error.with_diagnostic_code("material.unsupported_structure"),
                value,
            ),
            Self::NonFiniteSolution { binding: value } => binding(
                error.with_diagnostic_code("material.non_finite_solution"),
                value,
            ),
            Self::Problem(error) => error.diagnostic(),
        }
    }
}
