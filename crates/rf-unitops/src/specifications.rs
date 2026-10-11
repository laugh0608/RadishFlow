//! Local contracts for the existing steady-state built-ins. These checks neither solve
//! a unit nor decide application readiness; explicit specification is a caller policy.

use rf_model::{Composition, MaterialStreamState, UnitOperationParameters};
use rf_types::{ComponentId, StreamId};

use crate::{BuiltinUnitKind, HEATER_COOLER_INLET_PORT, MIXER_INLET_A_PORT, MIXER_INLET_B_PORT};

pub const SPECIFICATION_CONTRACT_VERSION: u32 = 1;
pub const COMPOSITION_SUM_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitParameter {
    OutletTemperature,
    OutletPressure,
}

impl UnitParameter {
    pub const fn key(self) -> &'static str {
        match self {
            Self::OutletTemperature => "outlet_temperature_k",
            Self::OutletPressure => "outlet_pressure_pa",
        }
    }

    pub fn value(self, parameters: &UnitOperationParameters) -> Option<f64> {
        match self {
            Self::OutletTemperature => parameters.outlet_temperature_k,
            Self::OutletPressure => parameters.outlet_pressure_pa,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutletPressureRule {
    Unrestricted,
    AtMostInlet,
    AtMostLowestInlet,
}

impl OutletPressureRule {
    pub const fn inlet_ports(self) -> &'static [&'static str] {
        match self {
            Self::Unrestricted => &[],
            Self::AtMostInlet => &[HEATER_COOLER_INLET_PORT],
            Self::AtMostLowestInlet => &[MIXER_INLET_A_PORT, MIXER_INLET_B_PORT],
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Unrestricted => "no inlet pressure constraint",
            Self::AtMostInlet => "inlet pressure",
            Self::AtMostLowestInlet => "lowest inlet pressure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinInputSpecification {
    pub parameters: &'static [UnitParameter],
    /// Required by explicit authoring, not by the lower-level execution defaults.
    pub required_explicit_parameters: &'static [UnitParameter],
    pub outlet_pressure_rule: OutletPressureRule,
}

pub const fn input_specification(kind: BuiltinUnitKind) -> BuiltinInputSpecification {
    use OutletPressureRule::*;
    use UnitParameter::*;
    let temperature_pressure = &[OutletTemperature, OutletPressure];
    let pressure = &[OutletPressure];
    match kind {
        BuiltinUnitKind::Feed => BuiltinInputSpecification {
            parameters: temperature_pressure,
            required_explicit_parameters: &[],
            outlet_pressure_rule: Unrestricted,
        },
        BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler => BuiltinInputSpecification {
            parameters: temperature_pressure,
            required_explicit_parameters: temperature_pressure,
            outlet_pressure_rule: AtMostInlet,
        },
        BuiltinUnitKind::FlashDrum => BuiltinInputSpecification {
            parameters: temperature_pressure,
            required_explicit_parameters: temperature_pressure,
            outlet_pressure_rule: Unrestricted,
        },
        BuiltinUnitKind::Mixer => BuiltinInputSpecification {
            parameters: pressure,
            required_explicit_parameters: pressure,
            outlet_pressure_rule: AtMostLowestInlet,
        },
        BuiltinUnitKind::Valve => BuiltinInputSpecification {
            parameters: pressure,
            required_explicit_parameters: pressure,
            outlet_pressure_rule: AtMostInlet,
        },
    }
}

pub fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterViolation {
    Unsupported,
    NotPositiveFinite,
    AboveInletPressure,
}

/// A missing limit means the inlet relationship has not been checked. Callers must
/// retain that coverage separately; local acceptance is not a readiness conclusion.
pub fn check_parameter_value(
    kind: BuiltinUnitKind,
    parameter: UnitParameter,
    value: f64,
    pressure_limit: Option<&InletPressureLimit>,
) -> Result<(), ParameterViolation> {
    let spec = input_specification(kind);
    if !spec.parameters.contains(&parameter) {
        return Err(ParameterViolation::Unsupported);
    }
    if !is_positive_finite(value) {
        return Err(ParameterViolation::NotPositiveFinite);
    }
    if parameter == UnitParameter::OutletPressure
        && spec.outlet_pressure_rule != OutletPressureRule::Unrestricted
        && pressure_limit.is_some_and(|limit| value > limit.pressure_pa)
    {
        return Err(ParameterViolation::AboveInletPressure);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct InletPressure {
    pub port: &'static str,
    pub stream_id: StreamId,
    pub pressure_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InletPressureLimit {
    pub pressure_pa: f64,
    pub limiting_stream_id: StreamId,
}

/// Returns a limit only for a complete set of positive, finite canonical inlets.
/// Equal mixer pressures deterministically select inlet_a as the limiting source.
pub fn inlet_pressure_limit(
    rule: OutletPressureRule,
    inlets: &[InletPressure],
) -> Option<InletPressureLimit> {
    let mut limit: Option<InletPressureLimit> = None;
    for port in rule.inlet_ports() {
        let mut matches = inlets.iter().filter(|input| input.port == *port);
        let input = matches.next()?;
        if matches.next().is_some() || !is_positive_finite(input.pressure_pa) {
            return None;
        }
        if limit
            .as_ref()
            .is_none_or(|current| input.pressure_pa < current.pressure_pa)
        {
            limit = Some(InletPressureLimit {
                pressure_pa: input.pressure_pa,
                limiting_stream_id: input.stream_id.clone(),
            });
        }
    }
    limit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedInputField {
    Temperature,
    Pressure,
    MolarFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedInputSource {
    StreamInput,
    UnitParameter(UnitParameter),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeedInputValue {
    pub value: f64,
    pub source: FeedInputSource,
}

/// The same precedence used to instantiate Feed. A present invalid override is
/// still the effective input; it must never silently fall back to the stream.
pub fn effective_feed_input(
    parameters: &UnitOperationParameters,
    stream: &MaterialStreamState,
    field: FeedInputField,
) -> FeedInputValue {
    let (parameter, stream_value) = match field {
        FeedInputField::Temperature => {
            (Some(UnitParameter::OutletTemperature), stream.temperature_k)
        }
        FeedInputField::Pressure => (Some(UnitParameter::OutletPressure), stream.pressure_pa),
        FeedInputField::MolarFlow => (None, stream.total_molar_flow_mol_s),
    };
    if let Some(parameter) = parameter
        && let Some(value) = parameter.value(parameters)
    {
        return FeedInputValue {
            value,
            source: FeedInputSource::UnitParameter(parameter),
        };
    }
    FeedInputValue {
        value: stream_value,
        source: FeedInputSource::StreamInput,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionPolicy {
    /// Application authoring: normalized mole fractions, never implicit normalization.
    SpecifiedMoleFractions,
    /// Existing execution API: nonnegative weights with a positive finite sum.
    NonnegativeWeights,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositionCheck {
    Empty,
    InvalidFractions(Vec<ComponentId>),
    InvalidTotal { sum: f64 },
    Unnormalized { sum: f64 },
    Valid { sum: f64 },
}

pub fn check_composition(composition: &Composition, policy: CompositionPolicy) -> CompositionCheck {
    if composition.is_empty() {
        return CompositionCheck::Empty;
    }
    let invalid = composition
        .iter()
        .filter(|(_, value)| {
            !value.is_finite()
                || **value < 0.0
                || (policy == CompositionPolicy::SpecifiedMoleFractions && **value > 1.0)
        })
        .map(|(component, _)| component.clone())
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        return CompositionCheck::InvalidFractions(invalid);
    }
    let sum = composition.values().sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return CompositionCheck::InvalidTotal { sum };
    }
    if policy == CompositionPolicy::SpecifiedMoleFractions
        && (sum - 1.0).abs() > COMPOSITION_SUM_TOLERANCE
    {
        return CompositionCheck::Unnormalized { sum };
    }
    CompositionCheck::Valid { sum }
}

#[cfg(test)]
mod tests;
