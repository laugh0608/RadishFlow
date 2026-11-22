use super::StoredPortBinding;
use rf_model::Flowsheet;
use rf_types::{
    ComponentId, DiagnosticPortTarget, PortDirection, PortKind, RfError, RfResult, StreamId, UnitId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SINGLE_PHASE_METHOD: &str = "radishflow.acyclic-single-vapor-sensible";
pub const SINGLE_PHASE_CAPABILITY: &str = "radishflow.solver.single-vapor-sensible";
pub const SINGLE_PHASE_RECYCLE_METHOD: &str = "radishflow.single-vapor-sensible-recycle";
pub const SINGLE_PHASE_RECYCLE_CAPABILITY: &str = "radishflow.solver.single-vapor-sensible-recycle";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredSinglePhaseRecyclePlan {
    pub single_phase: StoredSinglePhasePlan,
    pub tear: StoredSinglePhaseTear,
    pub max_iteration_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredSinglePhaseTear {
    pub stream_id: StreamId,
    pub source: StoredPortBinding,
    pub sink: StoredPortBinding,
    pub initialization: StoredSinglePhaseTearInitialization,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoredSinglePhaseTearInitialization {
    ZeroFlowsAndEnthalpy,
    ComponentFlowsAndEnthalpy {
        component_flows_mol_s: BTreeMap<ComponentId, f64>,
        enthalpy_flow_w: f64,
    },
}

impl StoredSinglePhaseRecyclePlan {
    pub fn validate(&self) -> RfResult<()> {
        self.single_phase.validate()?;
        if !(1..=500).contains(&self.max_iteration_evaluations) {
            return Err(RfError::invalid_input(
                "singlePhaseRecycle: iteration budget must be in 1..=500",
            )
            .with_diagnostic_code("solver.single_phase_recycle.invalid_budget"));
        }
        let tear = &self.tear;
        if tear.stream_id.as_str().trim().is_empty()
            || tear.source.unit_id.as_str().trim().is_empty()
            || tear.sink.unit_id.as_str().trim().is_empty()
            || tear.source.unit_id == tear.sink.unit_id
            || tear.source.port != "outlet_a"
            || !matches!(tear.sink.port.as_str(), "inlet_a" | "inlet_b")
        {
            return Err(tear.binding_error());
        }
        if let StoredSinglePhaseTearInitialization::ComponentFlowsAndEnthalpy {
            component_flows_mol_s,
            enthalpy_flow_w,
        } = &tear.initialization
        {
            let total = component_flows_mol_s.values().sum::<f64>();
            if component_flows_mol_s.is_empty()
                || component_flows_mol_s
                    .iter()
                    .any(|(id, f)| id.as_str().trim().is_empty() || !f.is_finite() || *f < 0.0)
                || !total.is_finite()
                || !enthalpy_flow_w.is_finite()
                || (total == 0.0 && *enthalpy_flow_w != 0.0)
            {
                return Err(tear.initialization_error());
            }
        }
        Ok(())
    }

    pub fn validate_for(&self, flowsheet: &Flowsheet) -> RfResult<()> {
        self.validate()?;
        self.single_phase.validate_references(flowsheet)?;
        let tear = &self.tear;
        let matching_ports = flowsheet.units.iter().flat_map(|(id, unit)| {
            unit.ports.iter().filter_map(move |port| {
                (port.kind == PortKind::Material
                    && port.stream_id.as_ref() == Some(&tear.stream_id))
                .then_some((id, unit, port))
            })
        });
        let (mut sources, mut sinks) = (0, 0);
        for (id, unit, port) in matching_ports {
            let (binding, kind, count) = match port.direction {
                PortDirection::Outlet => (&tear.source, "splitter", &mut sources),
                PortDirection::Inlet => (&tear.sink, "mixer", &mut sinks),
            };
            if id != &binding.unit_id
                || &unit.id != id
                || unit.kind != kind
                || port.name != binding.port
            {
                return Err(tear.binding_error());
            }
            *count += 1;
        }
        if !flowsheet.streams.contains_key(&tear.stream_id) || sources != 1 || sinks != 1 {
            return Err(tear.binding_error());
        }
        if let StoredSinglePhaseTearInitialization::ComponentFlowsAndEnthalpy {
            component_flows_mol_s,
            ..
        } = &tear.initialization
            && !component_flows_mol_s.keys().eq(flowsheet.components.keys())
        {
            return Err(tear.initialization_error());
        }
        Ok(())
    }
}

impl StoredSinglePhaseTear {
    fn binding_error(&self) -> RfError {
        RfError::invalid_input(
            "singlePhaseRecycle: tear must bind the actual Splitter.outlet_a to a Mixer inlet",
        )
        .with_diagnostic_code("solver.single_phase_recycle.invalid_tear")
        .with_related_stream_id(self.stream_id.clone())
        .with_related_port_targets(
            [&self.source, &self.sink]
                .map(|port| DiagnosticPortTarget::new(port.unit_id.clone(), port.port.clone()))
                .to_vec(),
        )
    }

    fn initialization_error(&self) -> RfError {
        RfError::invalid_input("singlePhaseRecycle: initial flows must match the registered components and be finite and nonnegative, with finite enthalpy flow and zero enthalpy at zero flow")
            .with_diagnostic_code("solver.single_phase_recycle.invalid_initial_state")
            .with_related_stream_id(self.stream_id.clone())
    }
}

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
