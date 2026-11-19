//! Formal steady-state plan values. Persistence and workspace identity are
//! supplied by callers. Configuration checks allow an editable graph that
//! is not ready to execute.
use crate::partitioned::{RecycleSolveOptions, assemble_partitioned_plan};
use rf_model::Flowsheet;
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{ComponentId, RfError, RfResult, StreamId, UnitId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseOptions {
    pub model: SensibleEnthalpyModel,
    pub heat_duties_w: BTreeMap<UnitId, f64>,
}

impl SinglePhaseOptions {
    pub fn new(
        capacities: BTreeMap<ComponentId, f64>,
        temperature_range_k: std::ops::RangeInclusive<f64>,
        pressure_pa: f64,
        heat_duties_w: BTreeMap<UnitId, f64>,
    ) -> RfResult<Self> {
        let result = Self {
            model: SensibleEnthalpyModel::new(capacities, temperature_range_k, pressure_pa)?,
            heat_duties_w,
        };
        result.validate_configuration()?;
        Ok(result)
    }

    fn validate_configuration(&self) -> RfResult<()> {
        for (id, duty) in &self.heat_duties_w {
            if id.as_str().trim().is_empty() || !duty.is_finite() {
                return Err(RfError::invalid_input(
                    "heat duties require non-blank unit IDs and finite values",
                )
                .with_related_unit_id(id.clone()));
            }
        }
        Ok(())
    }
    fn validate_for(&self, flowsheet: &Flowsheet) -> RfResult<()> {
        for id in self.model.heat_capacities_j_per_mol_k().keys() {
            if !flowsheet.components.contains_key(id) {
                return Err(RfError::invalid_input(
                    "heat capacity references an unknown component",
                )
                .with_related_component_id(id.clone()));
            }
        }
        for id in self.heat_duties_w.keys() {
            flowsheet.unit(id)?;
        }
        for unit in flowsheet.units.values() {
            rf_unitops::catalog::instantiate_single_phase_with_heat_duty(
                unit,
                &flowsheet.streams,
                &self.model,
                self.heat_duties_w.get(&unit.id).copied(),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum FlowPlanConfiguration {
    #[default]
    Acyclic,
    Partitioned(RecycleSolveOptions),
    Simultaneous(crate::simultaneous::ModularOptions),
    EquationOriented(crate::equation_oriented::EquationOptions),
    Mixed(crate::mixed::MixedOptions),
    SinglePhase(SinglePhaseOptions),
}
impl FlowPlanConfiguration {
    /// Validate the configuration without requiring a ready flowsheet.
    pub fn validate_configuration(&self) -> RfResult<()> {
        match self {
            Self::Acyclic => Ok(()),
            Self::SinglePhase(o) => o.validate_configuration(),
            Self::Mixed(o) => o.validate_shape(),
            Self::Partitioned(o) => o.validate_configuration(),
            Self::Simultaneous(o) => o.validate_configuration(),
            Self::EquationOriented(o) => o.validate_shape().map_err(|e| e.error),
        }
    }

    /// Check the selected plan against the current graph without evaluating properties.
    pub fn validate_for(&self, flowsheet: &Flowsheet) -> RfResult<()> {
        match self {
            Self::Mixed(o) => crate::mixed::assemble(flowsheet, o).map(|_| ()),
            Self::EquationOriented(o) => crate::equation_oriented::assemble(flowsheet, o)
                .and_then(|p| p.require_ready())
                .map_err(|e| e.error),
            Self::Acyclic => crate::sequential_execution_order(flowsheet).map(|_| ()),
            Self::SinglePhase(o) => {
                crate::sequential_execution_order(flowsheet)?;
                o.validate_for(flowsheet)
            }
            Self::Partitioned(o) => assemble_partitioned_plan(flowsheet, o).map(|_| ()),
            Self::Simultaneous(o) => crate::simultaneous::assemble(flowsheet, o).map(|_| ()),
        }
    }
    pub fn referenced_unit_ids(&self) -> BTreeSet<UnitId> {
        match self {
            Self::Acyclic => BTreeSet::new(),
            Self::SinglePhase(o) => o.heat_duties_w.keys().cloned().collect(),
            Self::EquationOriented(o) => o.referenced_unit_ids(),
            Self::Mixed(o) => o.referenced_unit_ids(),
            Self::Simultaneous(o) => o
                .scales
                .iter()
                .flat_map(|s| s.keys())
                .flat_map(|b| [b.source.unit_id.clone(), b.sink.unit_id.clone()])
                .collect(),
            Self::Partitioned(o) => o
                .tears
                .iter()
                .flat_map(|t| [t.source.unit_id.clone(), t.sink.unit_id.clone()])
                .collect(),
        }
    }
    pub fn referenced_stream_ids(&self) -> BTreeSet<StreamId> {
        match self {
            Self::Acyclic | Self::SinglePhase(_) => BTreeSet::new(),
            Self::EquationOriented(o) => o.referenced_stream_ids(),
            Self::Mixed(o) => o.referenced_stream_ids(),
            Self::Simultaneous(o) => o
                .scales
                .iter()
                .flat_map(|s| s.keys())
                .map(|b| b.stream.clone())
                .collect(),
            Self::Partitioned(o) => o.tears.iter().map(|t| t.stream_id.clone()).collect(),
        }
    }
}
