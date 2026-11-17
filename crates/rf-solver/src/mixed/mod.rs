//! Explicit composition of real local units and forward material equation blocks.
mod execution;
mod layout;
mod plan;
mod recycle;
pub use layout::EquationBlockLayout;
mod report;
#[cfg(test)]
mod tests;
use crate::equation_oriented::{EquationOptions, ExplicitEquationScales};
use crate::partitioned::{FixedRecycleBoundary, MAX_ITERATION_EVALUATIONS, TearSpecification};
pub use plan::{MixedPlan, assemble};
pub use report::*;
use rf_model::Flowsheet;
use rf_types::{RfError, RfResult, UnitId};
use std::collections::{BTreeMap, BTreeSet};

pub const METHOD: &str = "radishflow.fixed-tp-mixed-material/1";
pub const RECYCLE_METHOD: &str = "radishflow.fixed-tp-mixed-recycle/1";
pub const ACCEPTANCE_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MixedBlockId(String);
impl MixedBlockId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<&str> for MixedBlockId {
    fn from(v: &str) -> Self {
        Self::new(v)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum MixedBlockMethod {
    Local,
    EquationOriented {
        scales: Option<ExplicitEquationScales>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedBlock {
    pub id: MixedBlockId,
    pub members: BTreeSet<UnitId>,
    pub method: MixedBlockMethod,
}
impl MixedBlock {
    pub fn equation_options(&self, boundary: FixedRecycleBoundary) -> Option<EquationOptions> {
        match &self.method {
            MixedBlockMethod::Local => None,
            MixedBlockMethod::EquationOriented { scales } => {
                let mut o = EquationOptions::new(boundary);
                o.scales = scales.clone();
                Some(o)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedOptions {
    pub boundary: FixedRecycleBoundary,
    pub blocks: Vec<MixedBlock>,
    /// In-memory single cross-block recycle. The DAG project contract excludes it.
    pub recycle: Option<Box<MixedRecycleOptions>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixedRecycleOptions {
    pub tear: TearSpecification,
    pub max_iteration_evaluations: usize,
}

impl MixedRecycleOptions {
    pub fn new(tear: TearSpecification) -> Self {
        Self {
            tear,
            max_iteration_evaluations: MAX_ITERATION_EVALUATIONS,
        }
    }
}

impl MixedOptions {
    pub fn method(&self) -> &'static str {
        if self.recycle.is_some() {
            RECYCLE_METHOD
        } else {
            METHOD
        }
    }

    pub fn canonicalized(&self) -> Self {
        let mut result = self.clone();
        result.blocks.sort_by(|a, b| a.id.cmp(&b.id));
        result
    }
    pub fn validate_shape(&self) -> RfResult<()> {
        EquationOptions::new(self.boundary)
            .validate_shape()
            .map_err(|e| e.error)?;
        if let Some(recycle) = &self.recycle {
            let tear = &recycle.tear;
            if !(1..=MAX_ITERATION_EVALUATIONS).contains(&recycle.max_iteration_evaluations)
                || tear.stream_id.as_str().trim().is_empty()
                || [&tear.source, &tear.sink]
                    .iter()
                    .any(|p| p.unit_id.as_str().trim().is_empty() || p.port_name.trim().is_empty())
                || tear.component_scales_mol_s.is_some()
            {
                return Err(recycle::tear_error(
                    tear,
                    "recycle_options",
                    "the tear requires complete port bindings, the fresh-total scale and a budget in 1..=500",
                ));
            }
            if let Some(values) = &tear.initial_component_flows_mol_s {
                crate::fixed_tp_material::validate_vector(
                    &values.keys().cloned().collect::<Vec<_>>(),
                    values,
                    false,
                )
                .map_err(|e| e.with_related_stream_id(tear.stream_id.clone()))?;
            }
        }
        let mut ids = BTreeSet::new();
        let mut members = BTreeSet::new();
        for block in &self.blocks {
            if block.id.as_str().trim().is_empty()
                || !ids.insert(&block.id)
                || block.members.is_empty()
                || block.members.iter().any(|u| u.as_str().trim().is_empty())
                || matches!(block.method, MixedBlockMethod::Local) && block.members.len() != 1
            {
                return Err(failure(
                    "configuration",
                    "unique nonblank block IDs and nonempty members; Local requires one unit",
                ));
            }
            if block.members.iter().any(|u| !members.insert(u)) {
                return Err(failure("overlap", "unit belongs to multiple blocks"));
            }
            if let Some(o) = block.equation_options(self.boundary) {
                o.validate_shape().map_err(|e| e.error)?;
            }
        }
        Ok(())
    }
    pub fn referenced_unit_ids(&self) -> BTreeSet<UnitId> {
        self.blocks
            .iter()
            .flat_map(|b| {
                b.members.iter().cloned().chain(
                    b.equation_options(self.boundary)
                        .into_iter()
                        .flat_map(|o| o.referenced_unit_ids()),
                )
            })
            .chain(self.recycle.iter().flat_map(|recycle| {
                [
                    recycle.tear.source.unit_id.clone(),
                    recycle.tear.sink.unit_id.clone(),
                ]
            }))
            .collect()
    }
    pub fn referenced_stream_ids(&self) -> BTreeSet<rf_types::StreamId> {
        self.blocks
            .iter()
            .filter_map(|b| b.equation_options(self.boundary))
            .flat_map(|o| o.referenced_stream_ids())
            .chain(self.recycle.iter().map(|r| r.tear.stream_id.clone()))
            .collect()
    }
}
fn failure(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("solver.mixed.{code}"))
}
