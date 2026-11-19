//! Constant-Cp sensible enthalpy within a declared single-vapor domain.
use super::{MOLE_FRACTION_SUM_TOLERANCE, REFERENCE_TEMPERATURE_K};
use rf_types::{ComponentId, RfError, RfResult};
use std::{collections::BTreeMap, ops::RangeInclusive};

#[derive(Debug)]
pub struct SensibleEnthalpyModel {
    heat_capacities_j_per_mol_k: BTreeMap<ComponentId, f64>,
    temperature_range_k: RangeInclusive<f64>,
    pressure_pa: f64,
}

impl SensibleEnthalpyModel {
    pub const REFERENCE_TEMPERATURE_K: f64 = REFERENCE_TEMPERATURE_K;

    pub fn new(
        heat_capacities_j_per_mol_k: BTreeMap<ComponentId, f64>,
        temperature_range_k: RangeInclusive<f64>,
        pressure_pa: f64,
    ) -> RfResult<Self> {
        let positive = |value: f64| value.is_finite() && value > 0.0;
        if !positive(*temperature_range_k.start())
            || !positive(*temperature_range_k.end())
            || temperature_range_k.start() >= temperature_range_k.end()
            || !positive(pressure_pa)
        {
            return Err(RfError::invalid_input(
                "sensible enthalpy requires an increasing positive finite temperature interval and a positive finite pressure",
            )
            .with_diagnostic_code("thermo.sensible_enthalpy.invalid_domain"));
        }
        for (component, &capacity) in &heat_capacities_j_per_mol_k {
            if !positive(capacity) {
                return Err(RfError::thermo(format!(
                    "component `{component}` requires a positive finite vapor heat capacity"
                ))
                .with_diagnostic_code("thermo.sensible_enthalpy.invalid_heat_capacity")
                .with_related_component_id(component.clone()));
            }
        }
        Ok(Self {
            heat_capacities_j_per_mol_k,
            temperature_range_k,
            pressure_pa,
        })
    }

    pub fn molar_enthalpy_j_per_mol(
        &self,
        temperature_k: f64,
        pressure_pa: f64,
        mole_fractions: &BTreeMap<ComponentId, f64>,
    ) -> RfResult<f64> {
        self.validate_domain(temperature_k, pressure_pa)?;
        let heat_capacity = self.molar_heat_capacity_j_per_mol_k(mole_fractions)?;
        let enthalpy = heat_capacity * (temperature_k - Self::REFERENCE_TEMPERATURE_K);
        if !enthalpy.is_finite() {
            return Err(RfError::thermo("single-vapor molar enthalpy is not finite")
                .with_diagnostic_code("thermo.sensible_enthalpy.non_finite"));
        }
        Ok(enthalpy)
    }

    /// The inclusive temperature interval declared by the caller.
    pub fn temperature_range_k(&self) -> RangeInclusive<f64> {
        self.temperature_range_k.clone()
    }

    pub fn temperature_for_molar_enthalpy(
        &self,
        enthalpy_j_per_mol: f64,
        pressure_pa: f64,
        mole_fractions: &BTreeMap<ComponentId, f64>,
    ) -> RfResult<f64> {
        if !enthalpy_j_per_mol.is_finite() {
            return Err(
                RfError::thermo("single-vapor target molar enthalpy is not finite")
                    .with_diagnostic_code("thermo.sensible_enthalpy.non_finite"),
            );
        }
        let heat_capacity = self.molar_heat_capacity_j_per_mol_k(mole_fractions)?;
        let temperature_k = enthalpy_j_per_mol / heat_capacity + Self::REFERENCE_TEMPERATURE_K;
        self.validate_domain(temperature_k, pressure_pa)?;
        Ok(temperature_k)
    }

    /// Check a state against the declared temperature interval and fixed pressure.
    pub fn validate_domain(&self, temperature_k: f64, pressure_pa: f64) -> RfResult<()> {
        if !self.temperature_range_k.contains(&temperature_k) || pressure_pa != self.pressure_pa {
            return Err(RfError::thermo(format!(
                "state at {temperature_k} K and {pressure_pa} Pa is outside the single-vapor domain of {:?} K at {} Pa",
                self.temperature_range_k, self.pressure_pa
            ))
            .with_diagnostic_code("thermo.sensible_enthalpy.outside_domain"));
        }
        Ok(())
    }

    /// Composition-weighted constant heat capacity of the vapor.
    pub fn molar_heat_capacity_j_per_mol_k(
        &self,
        mole_fractions: &BTreeMap<ComponentId, f64>,
    ) -> RfResult<f64> {
        let total: f64 = mole_fractions.values().sum();
        if !total.is_finite()
            || (total - 1.0).abs() > MOLE_FRACTION_SUM_TOLERANCE
            || mole_fractions
                .values()
                .any(|&fraction| !(0.0..=1.0).contains(&fraction))
        {
            return Err(RfError::invalid_input(
                "single-vapor composition requires finite mole fractions in [0, 1] summing to one",
            )
            .with_diagnostic_code("thermo.sensible_enthalpy.invalid_composition"));
        }
        let mut heat_capacity = 0.0;
        for (component, &fraction) in mole_fractions {
            if fraction == 0.0 {
                continue;
            }
            let capacity = self
                .heat_capacities_j_per_mol_k
                .get(component)
                .ok_or_else(|| {
                    RfError::thermo(format!(
                        "component `{component}` is missing vapor heat capacity"
                    ))
                    .with_diagnostic_code("thermo.sensible_enthalpy.missing_heat_capacity")
                    .with_related_component_id(component.clone())
                })?;
            heat_capacity += fraction / total * capacity;
        }
        if !heat_capacity.is_finite() || heat_capacity <= 0.0 {
            return Err(RfError::thermo(
                "single-vapor mixture heat capacity must be positive and finite",
            )
            .with_diagnostic_code("thermo.sensible_enthalpy.invalid_heat_capacity"));
        }
        Ok(heat_capacity)
    }
}
