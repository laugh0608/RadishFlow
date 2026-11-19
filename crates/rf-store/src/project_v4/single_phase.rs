use rf_model::Flowsheet;
use rf_types::{ComponentId, RfError, RfResult, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SINGLE_PHASE_METHOD: &str = "radishflow.acyclic-single-vapor-sensible";
pub const SINGLE_PHASE_CAPABILITY: &str = "radishflow.solver.single-vapor-sensible";

/// Inputs for the version 1 constant-Cp vapor model with the fixed 298.15 K reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredSinglePhasePlan {
    pub min_temperature_k: f64,
    pub max_temperature_k: f64,
    pub pressure_pa: f64,
    pub heat_capacities_j_per_mol_k: BTreeMap<ComponentId, f64>,
    pub heat_duties_w: BTreeMap<UnitId, f64>,
}

impl StoredSinglePhasePlan {
    pub fn validate(&self) -> RfResult<()> {
        let positive = |v: f64| v.is_finite() && v > 0.0;
        if !positive(self.min_temperature_k)
            || !positive(self.max_temperature_k)
            || self.min_temperature_k >= self.max_temperature_k
            || !positive(self.pressure_pa)
        {
            return Err(RfError::invalid_input(
                "singlePhase: expected an increasing positive finite temperature interval and positive finite pressure",
            ));
        }
        for (id, &capacity) in &self.heat_capacities_j_per_mol_k {
            if id.as_str().trim().is_empty() || !positive(capacity) {
                return Err(RfError::invalid_input(
                    "singlePhase: invalid component or heat capacity",
                )
                .with_related_component_id(id.clone()));
            }
        }
        for (id, &duty) in &self.heat_duties_w {
            if id.as_str().trim().is_empty() || !duty.is_finite() {
                return Err(
                    RfError::invalid_input("singlePhase: invalid unit or heat duty")
                        .with_related_unit_id(id.clone()),
                );
            }
        }
        Ok(())
    }

    pub(super) fn validate_references(&self, flowsheet: &Flowsheet) -> RfResult<()> {
        for id in self.heat_capacities_j_per_mol_k.keys() {
            if !flowsheet.components.contains_key(id) {
                return Err(RfError::invalid_input(
                    "singlePhase: heat capacity references an unknown component",
                )
                .with_related_component_id(id.clone()));
            }
        }
        for id in self.heat_duties_w.keys() {
            let unit = flowsheet.unit(id)?;
            if !matches!(unit.kind.as_str(), "heater" | "cooler") {
                return Err(RfError::invalid_input(
                    "singlePhase: heat duty requires a Heater/Cooler",
                )
                .with_related_unit_id(id.clone()));
            }
            if unit.parameters.outlet_temperature_k.is_some() {
                return Err(RfError::invalid_input(
                    "singlePhase: outlet temperature and heat duty are mutually exclusive",
                )
                .with_diagnostic_code("solver.single_phase.conflicting_heat_specifications")
                .with_related_unit_id(id.clone()));
            }
        }
        Ok(())
    }
}
