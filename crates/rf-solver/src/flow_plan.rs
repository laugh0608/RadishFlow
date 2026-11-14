//! Formal steady-state plan values. Persistence and workspace identity are
//! supplied by callers. Configuration checks allow an editable graph that
//! is not ready to execute.
use crate::partitioned::{RecycleSolveOptions, assemble_partitioned_plan};
use rf_model::Flowsheet;
use rf_types::{RfResult, StreamId, UnitId};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum FlowPlanConfiguration {
    #[default]
    Acyclic,
    Partitioned(RecycleSolveOptions),
    Simultaneous(crate::simultaneous::ModularOptions),
    EquationOriented(crate::equation_oriented::EquationOptions),
    Mixed(crate::mixed::MixedOptions),
}
impl FlowPlanConfiguration {
    /// Validate the configuration without requiring a ready flowsheet.
    pub fn validate_configuration(&self) -> RfResult<()> {
        match self {
            Self::Acyclic => Ok(()),
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
            Self::Partitioned(o) => assemble_partitioned_plan(flowsheet, o).map(|_| ()),
            Self::Simultaneous(o) => crate::simultaneous::assemble(flowsheet, o).map(|_| ()),
        }
    }
    pub fn referenced_unit_ids(&self) -> BTreeSet<UnitId> {
        match self {
            Self::Acyclic => BTreeSet::new(),
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
            Self::Acyclic => BTreeSet::new(),
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
