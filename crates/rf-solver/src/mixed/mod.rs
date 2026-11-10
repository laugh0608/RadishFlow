//! Explicit DAG composition of real local units and forward material equation blocks.
mod execution;
mod layout;
mod plan;
pub use layout::EquationBlockLayout;
mod report;
#[cfg(test)]
mod tests;
use crate::equation_oriented::{EquationOptions, ExplicitEquationScales};
use crate::partitioned::FixedRecycleBoundary;
pub use plan::{MixedPlan, assemble};
pub use report::*;
use rf_model::Flowsheet;
use rf_types::{RfError, RfResult, UnitId};
use std::collections::{BTreeMap, BTreeSet};

pub const METHOD: &str = "radishflow.fixed-tp-mixed-material/1";
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
}
impl MixedOptions {
    pub fn canonicalized(&self) -> Self {
        let mut result = self.clone();
        result.blocks.sort_by(|a, b| a.id.cmp(&b.id));
        result
    }
    pub fn validate_shape(&self) -> RfResult<()> {
        EquationOptions::new(self.boundary)
            .validate_shape()
            .map_err(|e| e.error)?;
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
            .collect()
    }
    pub fn referenced_stream_ids(&self) -> BTreeSet<rf_types::StreamId> {
        self.blocks
            .iter()
            .filter_map(|b| b.equation_options(self.boundary))
            .flat_map(|o| o.referenced_stream_ids())
            .collect()
    }
}
fn failure(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("solver.mixed.{code}"))
}
