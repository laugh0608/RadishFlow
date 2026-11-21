//! Static discovery and construction of the existing built-in unit models.
//! No project mutation, service loading or execution takes place during construction.

use std::collections::BTreeMap;

use rf_model::{MaterialStreamState, UnitNode};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{RfError, RfResult, StreamId};

use crate::single_phase::{
    AdiabaticMixer, HeatDutyDomain, HeatSpecification, SinglePhaseFeed, SinglePhaseHeaterCooler,
    SinglePhaseSplitter,
};
use crate::specifications::{BuiltinInputSpecification, input_specification};
use crate::{
    BuiltinUnitKind, COOLER_SPEC, FEED_OUTLET_PORT, FEED_SPEC, FLASH_DRUM_LIQUID_PORT,
    FLASH_DRUM_SPEC, FLASH_DRUM_VAPOR_PORT, Feed, FlashDrum, HEATER_COOLER_OUTLET_PORT,
    HEATER_SPEC, HeaterCooler, MIXER_OUTLET_PORT, MIXER_SPEC, Mixer, StreamTarget, UnitOperation,
    UnitOperationSpec, VALVE_SPEC, Valve,
};

/// Whether absence of a service is supported by the current local execution model.
/// Optional services can enrich results; failures of supplied services still propagate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRequirement {
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinServiceRequirements {
    pub thermo: ServiceRequirement,
    /// Feed uses this service only when thermo is also supplied.
    pub tp_flash: ServiceRequirement,
}

type BuiltinConstructor =
    fn(&UnitNode, &BTreeMap<StreamId, MaterialStreamState>) -> RfResult<Box<dyn UnitOperation>>;

/// Describes the existing steady-state local `UnitOperation::run` contract.
/// This is not an external plugin ABI or a claim of EO / dynamic capability.
pub struct BuiltinModelDefinition {
    pub spec: &'static UnitOperationSpec,
    pub input_specification: BuiltinInputSpecification,
    pub services: BuiltinServiceRequirements,
    construct: BuiltinConstructor,
}

impl BuiltinModelDefinition {
    pub const fn component_equation_capability(
        &self,
    ) -> Option<crate::component_equations::ComponentEquationCapability> {
        use crate::component_equations::ComponentEquationCapability::*;
        match self.spec.kind {
            BuiltinUnitKind::Feed => Some(Feed),
            BuiltinUnitKind::Mixer => Some(Mixer),
            BuiltinUnitKind::Splitter => Some(Splitter),
            _ => None,
        }
    }

    /// Only the fixed-T/P component-flow mapping, not thermal or phase derivatives.
    pub const fn component_response_capability(
        &self,
    ) -> Option<crate::component_response::AffineComponentCapability> {
        use crate::component_response::AffineComponentCapability::*;
        match self.spec.kind {
            BuiltinUnitKind::Mixer => Some(TwoInletMixing),
            BuiltinUnitKind::Splitter => Some(NonselectiveSplitting),
            _ => None,
        }
    }

    /// A material-only capability, independent of the local run service contract.
    pub const fn component_balance_capability(
        &self,
    ) -> Option<crate::component_balance::ComponentBalanceCapability> {
        use crate::component_balance::ComponentBalanceCapability::*;
        match self.spec.kind {
            BuiltinUnitKind::Feed => Some(FixedFeedBoundary),
            BuiltinUnitKind::Mixer => Some(TwoInletMixing),
            _ => None,
        }
    }

    const fn new(
        spec: &'static UnitOperationSpec,
        thermo: ServiceRequirement,
        tp_flash: ServiceRequirement,
        construct: BuiltinConstructor,
    ) -> Self {
        Self {
            spec,
            input_specification: input_specification(spec.kind),
            services: BuiltinServiceRequirements { thermo, tp_flash },
            construct,
        }
    }
}

use ServiceRequirement::{Optional, Required};

static BUILTIN_MODELS: [BuiltinModelDefinition; 7] = [
    BuiltinModelDefinition::new(&FEED_SPEC, Optional, Optional, create_feed),
    BuiltinModelDefinition::new(&MIXER_SPEC, Required, Optional, create_mixer),
    BuiltinModelDefinition::new(&HEATER_SPEC, Required, Optional, create_heater),
    BuiltinModelDefinition::new(&COOLER_SPEC, Required, Optional, create_cooler),
    BuiltinModelDefinition::new(&VALVE_SPEC, Required, Optional, create_valve),
    BuiltinModelDefinition::new(&FLASH_DRUM_SPEC, Required, Required, create_flash_drum),
    BuiltinModelDefinition::new(&crate::SPLITTER_SPEC, Optional, Optional, create_splitter),
];

pub fn builtin_models() -> &'static [BuiltinModelDefinition] {
    &BUILTIN_MODELS
}

pub fn builtin_model(kind: BuiltinUnitKind) -> &'static BuiltinModelDefinition {
    match kind {
        BuiltinUnitKind::Feed => &BUILTIN_MODELS[0],
        BuiltinUnitKind::Mixer => &BUILTIN_MODELS[1],
        BuiltinUnitKind::Heater => &BUILTIN_MODELS[2],
        BuiltinUnitKind::Cooler => &BUILTIN_MODELS[3],
        BuiltinUnitKind::Valve => &BUILTIN_MODELS[4],
        BuiltinUnitKind::FlashDrum => &BUILTIN_MODELS[5],
        BuiltinUnitKind::Splitter => &BUILTIN_MODELS[6],
    }
}

/// Creates a fresh operation from project output templates and unit parameters.
/// Node / connection validation and execution-stage parameter checks remain the
/// caller's responsibility so existing failure ordering and defaults are preserved.
pub fn instantiate_builtin(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let kind = BuiltinUnitKind::parse(&unit.kind).ok_or_else(|| {
        RfError::invalid_input(format!(
            "{} uses unsupported model kind `{}`",
            unit_context(unit),
            unit.kind,
        ))
        .with_related_unit_id(unit.id.clone())
    })?;
    (builtin_model(kind).construct)(unit, streams)
}

/// Construct a local operation for the explicitly selected single-phase model.
pub fn instantiate_single_phase<'a>(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
    model: &'a SensibleEnthalpyModel,
) -> RfResult<Box<dyn UnitOperation + 'a>> {
    instantiate_single_phase_with_heat_duty(unit, streams, model, None)
}

/// Construct a local operation with an optional run-time heat duty supplied in watts.
pub fn instantiate_single_phase_with_heat_duty<'a>(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
    model: &'a SensibleEnthalpyModel,
    heat_duty_w: Option<f64>,
) -> RfResult<Box<dyn UnitOperation + 'a>> {
    single_phase_operation(unit, streams, model, heat_duty_w, HeatDutyDomain::Strict)
}

/// Validate the steady-state heat duty window on the actual fresh inflow before
/// allowing nonselective recycle trials. Both checks account for mixer roundoff.
pub fn instantiate_single_phase_recycle_heat_duty<'a>(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
    model: &'a SensibleEnthalpyModel,
    duty_w: f64,
    fresh: &MaterialStreamState,
) -> RfResult<Box<dyn UnitOperation + 'a>> {
    let operation = single_phase_operation(
        unit,
        streams,
        model,
        Some(duty_w),
        HeatDutyDomain::RecycleRoundoff,
    )?;
    model
        .validate_domain(fresh.temperature_k, fresh.pressure_pa)
        .and_then(|()| {
            HeatSpecification::HeatDutyW(duty_w)
                .resolve(model, fresh, HeatDutyDomain::RecycleRoundoff)
                .map(|_| ())
        })
        .map_err(|error| {
            error
                .with_related_unit_id(unit.id.clone())
                .with_related_stream_id(fresh.id.clone())
        })?;
    Ok(operation)
}

fn single_phase_operation<'a>(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
    model: &'a SensibleEnthalpyModel,
    heat_duty_w: Option<f64>,
    heat_duty_domain: HeatDutyDomain,
) -> RfResult<Box<dyn UnitOperation + 'a>> {
    let kind = BuiltinUnitKind::parse(&unit.kind);
    if heat_duty_w.is_some()
        && !matches!(
            kind,
            Some(BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler)
        )
    {
        return Err(RfError::invalid_input(format!(
            "{} does not accept a heat duty specification",
            unit_context(unit),
        ))
        .with_diagnostic_code("solver.single_phase.unsupported_heat_duty_target")
        .with_related_unit_id(unit.id.clone()));
    }
    match kind {
        Some(BuiltinUnitKind::Feed) => Ok(Box::new(SinglePhaseFeed {
            outlet: feed_outlet_template(unit, streams)?,
            model,
        })),
        Some(BuiltinUnitKind::Mixer) => Ok(Box::new(AdiabaticMixer {
            outlet: stream_target_for_port(unit, MIXER_OUTLET_PORT, streams)?,
            outlet_pressure_pa: unit.parameters.outlet_pressure_pa,
            model,
        })),
        Some(BuiltinUnitKind::Splitter) => Ok(Box::new(SinglePhaseSplitter {
            splitter: splitter_from_node(unit, streams)?,
            model,
        })),
        Some(kind @ (BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler)) => {
            let outlet = stream_target_for_port(unit, HEATER_COOLER_OUTLET_PORT, streams)?;
            let specification =
                HeatSpecification::from_inputs(unit.parameters.outlet_temperature_k, heat_duty_w)
                    .map_err(|error| {
                    error
                        .with_related_unit_id(unit.id.clone())
                        .with_related_stream_id(outlet.id.clone())
                })?;
            Ok(Box::new(SinglePhaseHeaterCooler {
                kind,
                outlet,
                outlet_pressure_pa: unit.parameters.outlet_pressure_pa,
                specification,
                heat_duty_domain,
                model,
            }))
        }
        _ => Err(RfError::invalid_input(format!(
            "{} has no single-phase sensible enthalpy model",
            unit_context(unit),
        ))
        .with_diagnostic_code("solver.single_phase.unsupported_unit")
        .with_related_unit_id(unit.id.clone())),
    }
}

fn create_feed(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let outlet = feed_outlet_template(unit, streams)?;
    Ok(Box::new(Feed::new(outlet)))
}

fn create_mixer(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let outlet = stream_target_for_port(unit, MIXER_OUTLET_PORT, streams)?;
    let mut operation = Mixer::new(outlet);
    if let Some(pressure_pa) = unit.parameters.outlet_pressure_pa {
        operation = operation.with_outlet_pressure_pa(pressure_pa);
    }
    Ok(Box::new(operation))
}

fn create_heater(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let outlet = heater_cooler_outlet_template(unit, streams)?;
    Ok(Box::new(HeaterCooler::new(
        BuiltinUnitKind::Heater,
        outlet,
    )?))
}

fn create_cooler(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let outlet = heater_cooler_outlet_template(unit, streams)?;
    Ok(Box::new(HeaterCooler::new(
        BuiltinUnitKind::Cooler,
        outlet,
    )?))
}

fn create_valve(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let outlet = valve_outlet_template(unit, streams)?;
    Ok(Box::new(Valve::new(outlet)))
}

fn create_flash_drum(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    let liquid = stream_target_for_port(unit, FLASH_DRUM_LIQUID_PORT, streams)?;
    let vapor = stream_target_for_port(unit, FLASH_DRUM_VAPOR_PORT, streams)?;
    let mut operation = FlashDrum::new(liquid, vapor);
    if let Some(temperature_k) = unit.parameters.outlet_temperature_k {
        operation = operation.with_outlet_temperature_k(temperature_k);
    }
    if let Some(pressure_pa) = unit.parameters.outlet_pressure_pa {
        operation = operation.with_outlet_pressure_pa(pressure_pa);
    }
    Ok(Box::new(operation))
}

fn heater_cooler_outlet_template(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<MaterialStreamState> {
    let mut outlet = stream_for_port(unit, HEATER_COOLER_OUTLET_PORT, streams)?.clone();
    if let Some(temperature_k) = unit.parameters.outlet_temperature_k {
        outlet.temperature_k = temperature_k;
    }
    if let Some(pressure_pa) = unit.parameters.outlet_pressure_pa {
        outlet.pressure_pa = pressure_pa;
    }
    Ok(outlet)
}

/// Resolve the same effective Feed boundary used by local execution, without
/// loading services or evaluating thermal properties.
pub fn effective_feed_boundary(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<MaterialStreamState> {
    if crate::validate_unit_node(unit)?.kind != BuiltinUnitKind::Feed {
        return Err(RfError::invalid_input(
            "feed boundary requires a canonical Feed",
        ));
    }
    feed_outlet_template(unit, streams)
}
fn feed_outlet_template(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<MaterialStreamState> {
    use crate::specifications::{FeedInputField, effective_feed_input};
    let mut outlet = stream_for_port(unit, FEED_OUTLET_PORT, streams)?.clone();
    outlet.temperature_k =
        effective_feed_input(&unit.parameters, &outlet, FeedInputField::Temperature).value;
    outlet.pressure_pa =
        effective_feed_input(&unit.parameters, &outlet, FeedInputField::Pressure).value;
    Ok(outlet)
}

fn valve_outlet_template(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<MaterialStreamState> {
    let mut outlet = stream_for_port(unit, HEATER_COOLER_OUTLET_PORT, streams)?.clone();
    if let Some(pressure_pa) = unit.parameters.outlet_pressure_pa {
        outlet.pressure_pa = pressure_pa;
    }
    Ok(outlet)
}

fn stream_for_port<'a>(
    unit: &UnitNode,
    port_name: &str,
    streams: &'a BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<&'a MaterialStreamState> {
    let port = unit
        .ports
        .iter()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "{} does not define expected port `{port_name}`",
                unit_context(unit),
            ))
        })?;
    let stream_id = port.stream_id.as_ref().ok_or_else(|| {
        RfError::invalid_input(format!(
            "{} port `{port_name}` is missing a connected stream id",
            unit_context(unit),
        ))
    })?;
    streams.get(stream_id).ok_or_else(|| {
        let error = RfError::missing_entity("stream", stream_id);
        RfError::new(
            error.code(),
            format!(
                "{} port `{port_name}` references missing stream `{stream_id}`: {}",
                unit_context(unit),
                error.message(),
            ),
        )
    })
}

fn stream_target_for_port(
    unit: &UnitNode,
    port_name: &str,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<StreamTarget> {
    let stream = stream_for_port(unit, port_name, streams)?;
    Ok(StreamTarget::new(stream.id.clone(), stream.name.clone()))
}

fn unit_context(unit: &UnitNode) -> String {
    format!("unit `{}` (`{}`)", unit.id, unit.kind)
}

#[cfg(test)]
mod tests;

fn create_splitter(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<Box<dyn UnitOperation>> {
    Ok(Box::new(splitter_from_node(unit, streams)?))
}

fn splitter_from_node(
    unit: &UnitNode,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<crate::Splitter> {
    let fraction = unit.parameters.outlet_a_split_fraction.ok_or_else(|| {
        RfError::invalid_input("splitter requires outlet_a_split_fraction")
            .with_diagnostic_code("unitops.splitter.missing_fraction")
            .with_related_unit_id(unit.id.clone())
    })?;
    Ok(crate::Splitter::new(
        stream_target_for_port(unit, crate::SPLITTER_OUTLET_A_PORT, streams)?,
        stream_target_for_port(unit, crate::SPLITTER_OUTLET_B_PORT, streams)?,
        fraction,
    ))
}
