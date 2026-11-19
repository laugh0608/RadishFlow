use super::{enthalpy_flow_w, materialize_vapor, verify_energy_balance};
use crate::{
    BuiltinUnitKind, HEATER_COOLER_INLET_PORT, HEATER_COOLER_OUTLET_PORT, StreamTarget,
    UnitOperation, UnitOperationInputs, UnitOperationOutputs, UnitOperationServices,
};
use rf_model::MaterialStreamState;
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{RfError, RfResult};

pub(crate) enum HeatSpecification {
    OutletTemperatureK(f64),
    HeatDutyW(f64),
}

impl HeatSpecification {
    pub(crate) fn from_inputs(
        outlet_temperature_k: Option<f64>,
        heat_duty_w: Option<f64>,
    ) -> RfResult<Self> {
        match (outlet_temperature_k, heat_duty_w) {
            (Some(temperature_k), None) => Ok(Self::OutletTemperatureK(temperature_k)),
            (None, Some(duty_w)) if duty_w.is_finite() => Ok(Self::HeatDutyW(duty_w)),
            (None, Some(_)) => Err(RfError::invalid_input("heat duty must be finite")
                .with_diagnostic_code("solver.single_phase.non_finite_heat_duty")),
            (Some(_), Some(_)) => Err(RfError::invalid_input(
                "single-phase Heater/Cooler received both outlet temperature and heat duty",
            )
            .with_diagnostic_code("solver.single_phase.conflicting_heat_specifications")),
            (None, None) => Err(RfError::invalid_input(
                "single-phase Heater/Cooler requires either outlet temperature or heat duty",
            )
            .with_diagnostic_code("solver.single_phase.missing_heat_specification")),
        }
    }

    fn resolve(
        &self,
        model: &SensibleEnthalpyModel,
        inlet: &MaterialStreamState,
    ) -> RfResult<(f64, f64)> {
        let heat_capacity_rate_w_per_k = inlet.total_molar_flow_mol_s
            * model.molar_heat_capacity_j_per_mol_k(&inlet.overall_mole_fractions)?;
        let duty_at =
            |temperature_k| heat_capacity_rate_w_per_k * (temperature_k - inlet.temperature_k);
        match *self {
            Self::OutletTemperatureK(temperature_k) => {
                model.validate_domain(temperature_k, inlet.pressure_pa)?;
                let duty_w = duty_at(temperature_k);
                if !duty_w.is_finite() {
                    return Err(RfError::thermo("single-vapor heat duty is not finite")
                        .with_diagnostic_code("solver.single_phase.non_finite_heat_duty"));
                }
                Ok((temperature_k, duty_w))
            }
            Self::HeatDutyW(duty_w) => {
                if inlet.total_molar_flow_mol_s <= 0.0 {
                    return Err(RfError::invalid_input(
                        "specified heat duty requires positive inlet molar flow",
                    )
                    .with_diagnostic_code("solver.single_phase.heat_duty_requires_flow")
                    .with_related_stream_id(inlet.id.clone()));
                }
                if !heat_capacity_rate_w_per_k.is_finite() || heat_capacity_rate_w_per_k == 0.0 {
                    return Err(RfError::thermo(
                        "single-vapor heat capacity rate must be positive and finite",
                    )
                    .with_diagnostic_code("thermo.sensible_enthalpy.invalid_heat_capacity"));
                }
                let range_k = model.temperature_range_k();
                let (min_k, max_k) = range_k.into_inner();
                let (min_w, max_w) = (duty_at(min_k), duty_at(max_k));
                if !(min_w..=max_w).contains(&duty_w) {
                    return Err(RfError::thermo(format!(
                        "heat duty {duty_w} W is outside the single-vapor domain of {min_w}..={max_w} W for this inlet"
                    ))
                    .with_diagnostic_code("thermo.sensible_enthalpy.outside_domain"));
                }
                // The duty check enforces the domain before rounding the inverse temperature.
                let temperature_k =
                    (inlet.temperature_k + duty_w / heat_capacity_rate_w_per_k).clamp(min_k, max_k);
                Ok((temperature_k, duty_w))
            }
        }
    }
}

pub(crate) struct SinglePhaseHeaterCooler<'a> {
    pub(crate) kind: BuiltinUnitKind,
    pub(crate) outlet: StreamTarget,
    pub(crate) outlet_pressure_pa: Option<f64>,
    pub(crate) specification: HeatSpecification,
    pub(crate) model: &'a SensibleEnthalpyModel,
}

impl UnitOperation for SinglePhaseHeaterCooler<'_> {
    fn kind(&self) -> BuiltinUnitKind {
        self.kind
    }

    fn run(
        &self,
        _: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;
        let inlet = inputs.require_stream(HEATER_COOLER_INLET_PORT)?;
        self.model
            .validate_domain(inlet.temperature_k, inlet.pressure_pa)
            .map_err(|error| error.with_related_stream_id(inlet.id.clone()))?;
        let pressure_pa = self.outlet_pressure_pa.unwrap_or(inlet.pressure_pa);
        let (temperature_k, heat_duty_w) = self
            .specification
            .resolve(self.model, inlet)
            .map_err(|error| error.with_related_stream_id(self.outlet.id.clone()))?;
        let outlet = materialize_vapor(
            self.model,
            MaterialStreamState::from_tpzf(
                self.outlet.id.clone(),
                self.outlet.name.clone(),
                temperature_k,
                pressure_pa,
                inlet.total_molar_flow_mol_s,
                inlet.overall_mole_fractions.clone(),
            ),
        )
        .map_err(|error| error.with_related_stream_id(self.outlet.id.clone()))?;
        let balance = verify_energy_balance(&[enthalpy_flow_w(inlet)?], heat_duty_w, &outlet)?;
        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(HEATER_COOLER_OUTLET_PORT, outlet);
        outputs.energy_balance = Some(balance);
        Ok(outputs)
    }
}
