use crate::{
    BuiltinUnitKind, FEED_OUTLET_PORT, MIXER_INLET_A_PORT, MIXER_INLET_B_PORT, MIXER_OUTLET_PORT,
    StreamTarget, UnitEnergyBalance, UnitOperation, UnitOperationInputs, UnitOperationOutputs,
    UnitOperationServices,
    component_balance::{mixer_component_residual, stream_component_molar_flows},
    mixed_composition, normalized_composition, validate_mixer_outlet_pressure, validated_pressure,
    validated_total_flow,
};
use rf_model::{MaterialStreamState, PhaseState};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{PhaseLabel, RfError, RfResult};
use std::collections::BTreeSet;

mod heat_transfer;
pub(crate) use heat_transfer::{HeatSpecification, SinglePhaseHeaterCooler};

const ENERGY_ABSOLUTE_TOLERANCE_W: f64 = 1e-6;
const ENERGY_SCALED_TOLERANCE: f64 = 1e-10;
const COMPONENT_FLOW_TOLERANCE_MOL_S: f64 = 1e-10;

pub(crate) struct SinglePhaseFeed<'a> {
    pub(crate) outlet: MaterialStreamState,
    pub(crate) model: &'a SensibleEnthalpyModel,
}

impl UnitOperation for SinglePhaseFeed<'_> {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Feed
    }

    fn run(
        &self,
        _: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;
        let stream = materialize_vapor(self.model, self.outlet.clone())
            .map_err(|error| error.with_related_stream_id(self.outlet.id.clone()))?;
        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(FEED_OUTLET_PORT, stream);
        Ok(outputs)
    }
}

pub(crate) struct AdiabaticMixer<'a> {
    pub(crate) outlet: StreamTarget,
    pub(crate) outlet_pressure_pa: Option<f64>,
    pub(crate) model: &'a SensibleEnthalpyModel,
}

impl UnitOperation for AdiabaticMixer<'_> {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Mixer
    }

    fn run(
        &self,
        _: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;
        let inlet_a = inputs.require_stream(MIXER_INLET_A_PORT)?;
        let inlet_b = inputs.require_stream(MIXER_INLET_B_PORT)?;
        let composition = mixed_composition([inlet_a, inlet_b]).map_err(|error| {
            error
                .with_related_stream_id(inlet_a.id.clone())
                .with_related_stream_id(inlet_b.id.clone())
        })?;
        let total_flow_mol_s = inlet_a.total_molar_flow_mol_s + inlet_b.total_molar_flow_mol_s;
        let inlet_pressure_limit = validated_pressure(inlet_a)?.min(validated_pressure(inlet_b)?);
        let pressure_pa = self.outlet_pressure_pa.unwrap_or(inlet_pressure_limit);
        validate_mixer_outlet_pressure(pressure_pa, inlet_pressure_limit)?;

        let enthalpy_a_w = enthalpy_flow_w(inlet_a)?;
        let enthalpy_b_w = enthalpy_flow_w(inlet_b)?;
        let enthalpy_j_per_mol = (enthalpy_a_w + enthalpy_b_w) / total_flow_mol_s;
        let temperature_k = self
            .model
            .temperature_for_molar_enthalpy(enthalpy_j_per_mol, pressure_pa, &composition)
            .map_err(|error| error.with_related_stream_id(self.outlet.id.clone()))?;
        let outlet = materialize_vapor(
            self.model,
            MaterialStreamState::from_tpzf(
                self.outlet.id.clone(),
                self.outlet.name.clone(),
                temperature_k,
                pressure_pa,
                total_flow_mol_s,
                composition,
            ),
        )
        .map_err(|error| error.with_related_stream_id(self.outlet.id.clone()))?;
        verify_component_balance(inlet_a, inlet_b, &outlet)?;
        let balance = verify_energy_balance(&[enthalpy_a_w, enthalpy_b_w], 0.0, &outlet)?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(MIXER_OUTLET_PORT, outlet);
        outputs.energy_balance = Some(balance);
        Ok(outputs)
    }
}

fn enthalpy_flow_w(stream: &MaterialStreamState) -> RfResult<f64> {
    if stream.total_molar_flow_mol_s == 0.0 {
        return Ok(0.0);
    }
    let enthalpy = stream
        .phases
        .iter()
        .find(|phase| phase.label == PhaseLabel::Overall)
        .and_then(|phase| phase.molar_enthalpy_j_per_mol)
        .ok_or_else(|| {
            RfError::thermo(format!(
                "stream `{}` has no overall molar enthalpy",
                stream.id
            ))
            .with_diagnostic_code("solver.single_phase.missing_enthalpy")
            .with_related_stream_id(stream.id.clone())
        })?;
    let rate = stream.total_molar_flow_mol_s * enthalpy;
    if !rate.is_finite() {
        return Err(RfError::thermo(format!(
            "stream `{}` enthalpy flow is not finite",
            stream.id
        ))
        .with_diagnostic_code("thermo.sensible_enthalpy.non_finite")
        .with_related_stream_id(stream.id.clone()));
    }
    Ok(rate)
}

fn verify_component_balance(
    inlet_a: &MaterialStreamState,
    inlet_b: &MaterialStreamState,
    outlet: &MaterialStreamState,
) -> RfResult<()> {
    let a = stream_component_molar_flows(inlet_a)?;
    let b = stream_component_molar_flows(inlet_b)?;
    let out = stream_component_molar_flows(outlet)?;
    let components: BTreeSet<_> = a.keys().chain(b.keys()).chain(out.keys()).collect();
    for component in components {
        let residual = mixer_component_residual(
            a.get(component).copied().unwrap_or(0.0),
            b.get(component).copied().unwrap_or(0.0),
            out.get(component).copied().unwrap_or(0.0),
        );
        if !residual.is_finite() || residual.abs() > COMPONENT_FLOW_TOLERANCE_MOL_S {
            return Err(RfError::thermo(format!(
                "adiabatic mixer component `{component}` residual is {residual} mol/s"
            ))
            .with_diagnostic_code("solver.single_phase.component_balance")
            .with_related_component_id(component.clone())
            .with_related_stream_id(outlet.id.clone()));
        }
    }
    Ok(())
}

fn verify_energy_balance(
    inlet_enthalpy_flows_w: &[f64],
    heat_duty_w: f64,
    outlet: &MaterialStreamState,
) -> RfResult<UnitEnergyBalance> {
    let inlet_enthalpy_flow_w = inlet_enthalpy_flows_w.iter().sum::<f64>();
    let outlet_enthalpy_flow_w = enthalpy_flow_w(outlet)?;
    let residual_w = (outlet_enthalpy_flow_w - inlet_enthalpy_flow_w) - heat_duty_w;
    let residual_scale_w = (inlet_enthalpy_flows_w
        .iter()
        .map(|value| value.abs())
        .sum::<f64>()
        + outlet_enthalpy_flow_w.abs()
        + heat_duty_w.abs())
    .max(1.0);
    let scaled_residual = residual_w / residual_scale_w;
    if !residual_w.is_finite()
        || !residual_scale_w.is_finite()
        || residual_w.abs() > ENERGY_ABSOLUTE_TOLERANCE_W
        || scaled_residual.abs() > ENERGY_SCALED_TOLERANCE
    {
        return Err(RfError::thermo(format!(
            "unit energy residual is {residual_w} W (scale {residual_scale_w} W)"
        ))
        .with_diagnostic_code("solver.single_phase.energy_balance")
        .with_related_stream_id(outlet.id.clone()));
    }
    Ok(UnitEnergyBalance {
        inlet_enthalpy_flow_w,
        outlet_enthalpy_flow_w,
        heat_duty_w,
        residual_w,
        residual_scale_w,
        scaled_residual,
    })
}

fn materialize_vapor(
    model: &SensibleEnthalpyModel,
    mut stream: MaterialStreamState,
) -> RfResult<MaterialStreamState> {
    let flow = validated_total_flow(&stream)?;
    let enthalpy = model.molar_enthalpy_j_per_mol(
        stream.temperature_k,
        stream.pressure_pa,
        &stream.overall_mole_fractions,
    )?;
    if !(flow * enthalpy).is_finite() {
        return Err(RfError::thermo("single-vapor enthalpy flow is not finite")
            .with_diagnostic_code("thermo.sensible_enthalpy.non_finite"));
    }
    stream.overall_mole_fractions = normalized_composition(&stream)?;
    stream.phases.clear();
    stream.bubble_dew_window = None;
    if flow > 0.0 {
        for label in [PhaseLabel::Overall, PhaseLabel::Vapor] {
            let mut phase = PhaseState::new(label, 1.0, stream.overall_mole_fractions.clone());
            phase.molar_enthalpy_j_per_mol = Some(enthalpy);
            stream.phases.push(phase);
        }
    }
    Ok(stream)
}
