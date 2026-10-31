//! Limited component material relations; no thermal, equilibrium or dynamic claim.
use rf_model::{Composition, MaterialStreamState};
use rf_types::{RfError, RfResult};

/// Only these built-ins provide the material subproblem used by QS0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentBalanceCapability {
    FixedFeedBoundary,
    TwoInletMixing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MixerBalancePort {
    InletA,
    InletB,
    Outlet,
}
impl MixerBalancePort {
    pub const ALL: [Self; 3] = [Self::InletA, Self::InletB, Self::Outlet];

    pub const fn name(self) -> &'static str {
        match self {
            Self::InletA => crate::MIXER_INLET_A_PORT,
            Self::InletB => crate::MIXER_INLET_B_PORT,
            Self::Outlet => crate::MIXER_OUTLET_PORT,
        }
    }

    /// Derivative of outlet minus the two inlet component molar flows.
    pub const fn residual_derivative(self) -> f64 {
        match self {
            Self::InletA | Self::InletB => -1.0,
            Self::Outlet => 1.0,
        }
    }
}

/// Shared additive material relation. Callers validate their own input domain.
pub fn mixer_component_outlet(inlet_a_mol_s: f64, inlet_b_mol_s: f64) -> f64 {
    inlet_a_mol_s + inlet_b_mol_s
}

/// Nonselective extensive-flow relation; callers validate flow and fraction.
pub fn splitter_component_outlet(inlet_mol_s: f64, fraction: f64) -> f64 {
    inlet_mol_s * fraction
}

pub fn mixer_component_residual(inlet_a_mol_s: f64, inlet_b_mol_s: f64, outlet_mol_s: f64) -> f64 {
    outlet_mol_s - mixer_component_outlet(inlet_a_mol_s, inlet_b_mol_s)
}

/// Uses the same flow and nonnegative-weight normalization policy as local SM.
pub fn stream_component_molar_flows(stream: &MaterialStreamState) -> RfResult<Composition> {
    let flow = validated_total_flow(stream)?;
    Ok(normalized_composition(stream)?
        .into_iter()
        .map(|(component, fraction)| (component, flow * fraction))
        .collect())
}

pub(crate) fn validated_total_flow(stream: &MaterialStreamState) -> RfResult<f64> {
    if !stream.total_molar_flow_mol_s.is_finite() || stream.total_molar_flow_mol_s < 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` total molar flow must be a finite non-negative value",
            stream.id
        )));
    }

    Ok(stream.total_molar_flow_mol_s)
}

pub(crate) fn normalized_composition(stream: &MaterialStreamState) -> RfResult<Composition> {
    use crate::specifications::{CompositionCheck, CompositionPolicy, check_composition};
    let check = check_composition(
        &stream.overall_mole_fractions,
        CompositionPolicy::NonnegativeWeights,
    );
    let message = match check {
        CompositionCheck::Empty => Some("must define at least one overall mole fraction entry"),
        CompositionCheck::InvalidFractions(_) => {
            Some("overall mole fractions must be finite non-negative values")
        }
        CompositionCheck::InvalidTotal { .. } | CompositionCheck::Unnormalized { .. } => {
            Some("overall mole fractions must sum to a positive finite value")
        }
        CompositionCheck::Valid { .. } => None,
    };
    if let Some(message) = message {
        return Err(RfError::invalid_input(format!(
            "stream `{}` {message}",
            stream.id
        )));
    }
    let CompositionCheck::Valid { sum } = check else {
        unreachable!("invalid composition returned its error");
    };
    Ok(stream
        .overall_mole_fractions
        .iter()
        .map(|(component_id, value)| (component_id.clone(), value / sum))
        .collect())
}

pub(crate) fn mixed_composition(streams: [&MaterialStreamState; 2]) -> RfResult<Composition> {
    let mut component_molar_flows = Composition::new();
    let mut total_flow = 0.0;

    for stream in streams {
        let flow = validated_total_flow(stream)?;
        let normalized = normalized_composition(stream)?;
        total_flow += flow;

        for (component_id, fraction) in normalized {
            let accumulated = component_molar_flows.entry(component_id).or_insert(0.0);
            *accumulated = mixer_component_outlet(*accumulated, flow * fraction);
        }
    }

    if !total_flow.is_finite() || total_flow <= 0.0 {
        return Err(RfError::invalid_input(
            "mixed stream total molar flow must be a positive finite value",
        ));
    }

    Ok(component_molar_flows
        .into_iter()
        .map(|(component_id, molar_flow)| (component_id, molar_flow / total_flow))
        .collect())
}
