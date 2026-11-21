//! Independent, adiabatic countercurrent heat exchange between two single-vapor streams.
use crate::{
    StreamTarget, UnitEnergyBalance,
    component_balance::stream_component_molar_flows,
    single_phase::{
        COMPONENT_FLOW_TOLERANCE_MOL_S, enthalpy_flow_w, materialize_vapor, verify_energy_balance,
    },
};
use rf_model::MaterialStreamState;
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{RfError, RfResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatExchangerSpecification {
    /// Positive heat transfer from the hot side to the cold side.
    HeatDutyW(f64),
    HotOutletTemperatureK(f64),
    ColdOutletTemperatureK(f64),
}

impl HeatExchangerSpecification {
    /// Check the selection when the caller provides individual optional inputs.
    pub fn from_inputs(
        heat_duty_w: Option<f64>,
        hot_outlet_temperature_k: Option<f64>,
        cold_outlet_temperature_k: Option<f64>,
    ) -> RfResult<Self> {
        match (
            heat_duty_w,
            hot_outlet_temperature_k,
            cold_outlet_temperature_k,
        ) {
            (Some(value), None, None) => Ok(Self::HeatDutyW(value)),
            (None, Some(value), None) => Ok(Self::HotOutletTemperatureK(value)),
            (None, None, Some(value)) => Ok(Self::ColdOutletTemperatureK(value)),
            (None, None, None) => Err(error(
                "missing_specification",
                "heat exchanger requires a heat duty or one outlet temperature",
            )),
            _ => Err(error(
                "conflicting_specifications",
                "heat duty and the two outlet temperature specifications are mutually exclusive",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeatExchangerSide<'a> {
    pub inlet: &'a MaterialStreamState,
    pub outlet: &'a StreamTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeatExchangerSideResult {
    pub outlet: MaterialStreamState,
    pub energy_balance: UnitEnergyBalance,
    pub max_component_flow_residual_mol_s: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeatExchangerResult {
    pub hot: HeatExchangerSideResult,
    pub cold: HeatExchangerSideResult,
    pub heat_duty_w: f64,
    /// Hot inlet temperature minus cold outlet temperature.
    pub hot_inlet_approach_k: f64,
    /// Hot outlet temperature minus cold inlet temperature.
    pub cold_inlet_approach_k: f64,
    pub pinch_heat_duty_w: f64,
    pub overall_energy_balance: UnitEnergyBalance,
}

/// Calculate a steady countercurrent exchanger within the supplied constant-Cp model.
/// Both sides retain their material, flow rate and pressure.
pub fn calculate_countercurrent(
    model: &SensibleEnthalpyModel,
    hot: HeatExchangerSide<'_>,
    cold: HeatExchangerSide<'_>,
    specification: HeatExchangerSpecification,
) -> RfResult<HeatExchangerResult> {
    let (hot_inlet, hot_capacity) = prepare_inlet(model, hot.inlet)?;
    let (cold_inlet, cold_capacity) = prepare_inlet(model, cold.inlet)?;
    let hot_in_k = hot_inlet.temperature_k;
    let cold_in_k = cold_inlet.temperature_k;
    let boundary_error = |code, message| {
        error(code, message)
            .with_related_stream_ids(vec![hot_inlet.id.clone(), cold_inlet.id.clone()])
    };
    if hot_in_k < cold_in_k {
        return Err(boundary_error(
            "reversed_inlets",
            "hot inlet temperature must be at least the cold inlet temperature",
        ));
    }

    let (heat_duty_w, specified_hot_outlet_k, specified_cold_outlet_k) = match specification {
        HeatExchangerSpecification::HeatDutyW(duty_w) => (duty_w, None, None),
        HeatExchangerSpecification::HotOutletTemperatureK(temperature_k) => (
            duty_from_outlet_temperature(model, &hot_inlet, temperature_k, -hot_capacity)?,
            Some(temperature_k),
            None,
        ),
        HeatExchangerSpecification::ColdOutletTemperatureK(temperature_k) => (
            duty_from_outlet_temperature(model, &cold_inlet, temperature_k, cold_capacity)?,
            None,
            Some(temperature_k),
        ),
    };
    if !heat_duty_w.is_finite() {
        return Err(error(
            "non_finite_specification",
            "heat duty must be finite",
        ));
    }
    if heat_duty_w < 0.0 {
        return Err(error(
            "reverse_heat_transfer",
            "heat duty must transfer heat from hot to cold",
        ));
    }
    let pinch_heat_duty_w = hot_capacity.min(cold_capacity) * (hot_in_k - cold_in_k);
    if !pinch_heat_duty_w.is_finite() {
        return Err(boundary_error(
            "numeric_range",
            "countercurrent pinch duty is not finite",
        ));
    }
    if heat_duty_w > pinch_heat_duty_w {
        return Err(boundary_error(
            "above_pinch",
            "heat duty exceeds the countercurrent pinch limit",
        ));
    }

    let hot_out_k = specified_hot_outlet_k.unwrap_or_else(|| hot_in_k - heat_duty_w / hot_capacity);
    let cold_out_k =
        specified_cold_outlet_k.unwrap_or_else(|| cold_in_k + heat_duty_w / cold_capacity);
    let hot_inlet_approach_k = hot_in_k - cold_out_k;
    let cold_inlet_approach_k = hot_out_k - cold_in_k;
    if heat_duty_w > 0.0
        && (heat_duty_w == pinch_heat_duty_w
            || hot_inlet_approach_k <= 0.0
            || cold_inlet_approach_k <= 0.0)
    {
        return Err(boundary_error(
            "pinch_limit",
            "positive heat transfer requires a positive temperature difference at each end",
        ));
    }

    let hot = evaluate_side(model, &hot_inlet, hot.outlet, hot_out_k, -heat_duty_w)?;
    let cold = evaluate_side(model, &cold_inlet, cold.outlet, cold_out_k, heat_duty_w)?;
    let overall_energy_balance = verify_energy_balance(
        &[
            hot.energy_balance.inlet_enthalpy_flow_w,
            cold.energy_balance.inlet_enthalpy_flow_w,
        ],
        0.0,
        &[&hot.outlet, &cold.outlet],
    )?;
    Ok(HeatExchangerResult {
        hot,
        cold,
        heat_duty_w,
        hot_inlet_approach_k,
        cold_inlet_approach_k,
        pinch_heat_duty_w,
        overall_energy_balance,
    })
}

fn duty_from_outlet_temperature(
    model: &SensibleEnthalpyModel,
    inlet: &MaterialStreamState,
    outlet_temperature_k: f64,
    signed_capacity_w_per_k: f64,
) -> RfResult<f64> {
    let at_inlet = |error: RfError| error.with_related_stream_id(inlet.id.clone());
    if !outlet_temperature_k.is_finite() {
        return Err(at_inlet(error(
            "non_finite_specification",
            "outlet temperature must be finite",
        )));
    }
    model
        .validate_domain(outlet_temperature_k, inlet.pressure_pa)
        .map_err(at_inlet)?;
    let temperature_change_k = outlet_temperature_k - inlet.temperature_k;
    if temperature_change_k == 0.0 {
        return Ok(0.0);
    }
    let duty_w = signed_capacity_w_per_k * temperature_change_k;
    if duty_w == 0.0 || !duty_w.is_finite() {
        return Err(at_inlet(error(
            "numeric_range",
            "outlet temperature implies a heat duty outside the numeric range",
        )));
    }
    Ok(duty_w)
}

fn prepare_inlet(
    model: &SensibleEnthalpyModel,
    inlet: &MaterialStreamState,
) -> RfResult<(MaterialStreamState, f64)> {
    if !inlet.total_molar_flow_mol_s.is_finite() || inlet.total_molar_flow_mol_s <= 0.0 {
        return Err(error(
            "invalid_flow",
            "each inlet requires positive finite molar flow",
        )
        .with_related_stream_id(inlet.id.clone()));
    }
    let prepared = materialize_vapor(model, inlet.clone())
        .map_err(|error| error.with_related_stream_id(inlet.id.clone()))?;
    let capacity = model
        .molar_heat_capacity_j_per_mol_k(&prepared.overall_mole_fractions)
        .map_err(|error| error.with_related_stream_id(inlet.id.clone()))?
        * prepared.total_molar_flow_mol_s;
    if !capacity.is_finite() || capacity <= 0.0 {
        return Err(error(
            "numeric_range",
            "heat capacity rate must be positive and finite",
        )
        .with_related_stream_id(inlet.id.clone()));
    }
    Ok((prepared, capacity))
}

fn evaluate_side(
    model: &SensibleEnthalpyModel,
    inlet: &MaterialStreamState,
    target: &StreamTarget,
    outlet_temperature_k: f64,
    heat_duty_w: f64,
) -> RfResult<HeatExchangerSideResult> {
    let outlet = materialize_vapor(
        model,
        MaterialStreamState::from_tpzf(
            target.id.clone(),
            target.name.clone(),
            outlet_temperature_k,
            inlet.pressure_pa,
            inlet.total_molar_flow_mol_s,
            inlet.overall_mole_fractions.clone(),
        ),
    )
    .map_err(|error| error.with_related_stream_ids(vec![inlet.id.clone(), target.id.clone()]))?;
    let outgoing = stream_component_molar_flows(&outlet)?;
    let mut max_component_flow_residual_mol_s: f64 = 0.0;
    for (id, incoming) in stream_component_molar_flows(inlet)? {
        let residual = outgoing[&id] - incoming;
        if !residual.is_finite() || residual.abs() > COMPONENT_FLOW_TOLERANCE_MOL_S {
            return Err(RfError::thermo(format!(
                "heat exchanger component residual is {residual} mol/s"
            ))
            .with_diagnostic_code("unit.heat_exchanger.component_balance")
            .with_related_component_id(id)
            .with_related_stream_ids(vec![inlet.id.clone(), outlet.id.clone()]));
        }
        max_component_flow_residual_mol_s = max_component_flow_residual_mol_s.max(residual.abs());
    }
    let energy_balance =
        verify_energy_balance(&[enthalpy_flow_w(inlet)?], heat_duty_w, &[&outlet])?;
    Ok(HeatExchangerSideResult {
        outlet,
        energy_balance,
        max_component_flow_residual_mol_s,
    })
}

fn error(code: &str, message: &str) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("unit.heat_exchanger.{code}"))
}
