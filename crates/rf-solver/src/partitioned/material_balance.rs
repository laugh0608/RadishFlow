//! The material boundary actually rejected by the fixed-T/P partitioned solver.
use super::{ComponentId, RecycleTolerances, StreamId, UnitId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialBalanceScope {
    Unit {
        block_index: usize,
        unit_id: UnitId,
    },
    RecycleRegion {
        block_index: usize,
        tear_stream_id: StreamId,
    },
    Flowsheet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowDifference {
    pub input_mol_s: f64,
    pub output_mol_s: f64,
    /// Output minus input, with the sign and nonfinite values from this check.
    pub difference_mol_s: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentBalanceExcess {
    pub component_id: ComponentId,
    pub flow: FlowDifference,
    pub scale_mol_s: f64,
    pub scaled_absolute: f64,
    pub absolute_exceeded: bool,
    pub scaled_exceeded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceReadings {
    pub inputs: Vec<StreamId>,
    pub outputs: Vec<StreamId>,
    /// Only components that exceeded a threshold, in frozen registry order.
    pub exceeded_components: Vec<ComponentBalanceExcess>,
    pub tolerances: RecycleTolerances,
    pub total: FlowDifference,
    pub total_exceeded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceFailure {
    pub scope: MaterialBalanceScope,
    pub readings: Box<MaterialBalanceReadings>,
}
