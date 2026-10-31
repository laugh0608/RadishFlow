//! Typed application results. Shared header access never turns a block execution
//! into an acyclic step sequence.
use crate::{
    DiagnosticSeverity, DiagnosticSummary, RunStatus, SolveSnapshot, SolveSnapshotId,
    StreamStateSnapshot,
};
use rf_solver::partitioned::PartitionedSolveResult;
use rf_types::UnitId;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FlowSnapshotHeader {
    pub origin: Option<crate::result_query::FlowResultOrigin>,
    pub id: SolveSnapshotId,
    pub document_revision: u64,
    pub sequence: u64,
    pub status: RunStatus,
    pub summary: DiagnosticSummary,
    pub diagnostics: Vec<crate::DiagnosticSnapshot>,
    pub streams: Vec<StreamStateSnapshot>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct UnitResultSnapshot {
    pub consumed_streams: Vec<StreamStateSnapshot>,
    pub produced_streams: Vec<StreamStateSnapshot>,
}
#[derive(Debug, Clone, Copy)]
pub struct UnitResultView<'a> {
    pub consumed_streams: &'a [StreamStateSnapshot],
    pub produced_streams: &'a [StreamStateSnapshot],
}
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedFlowSnapshot {
    pub common: FlowSnapshotHeader,
    pub result: Arc<PartitionedSolveResult>,
    pub final_units: BTreeMap<UnitId, UnitResultSnapshot>,
}
impl PartitionedFlowSnapshot {
    pub fn new(
        id: impl Into<SolveSnapshotId>,
        revision: u64,
        sequence: u64,
        result: PartitionedSolveResult,
    ) -> Self {
        let streams = result
            .streams
            .values()
            .map(crate::run::stream_state_snapshot_from_model)
            .collect();
        let final_units = result
            .blocks
            .iter()
            .flat_map(|b| &b.final_units)
            .map(|u| {
                (
                    u.unit_id.clone(),
                    UnitResultSnapshot {
                        consumed_streams: u
                            .consumed_streams
                            .iter()
                            .map(crate::run::stream_state_snapshot_from_model)
                            .collect(),
                        produced_streams: u
                            .produced_streams
                            .iter()
                            .map(crate::run::stream_state_snapshot_from_model)
                            .collect(),
                    },
                )
            })
            .collect();
        let summary = DiagnosticSummary::new(
            revision,
            DiagnosticSeverity::Info,
            format!(
                "{} dependency blocks completed with verified recycle closure",
                result.blocks.len()
            ),
        );
        Self {
            common: FlowSnapshotHeader {
                origin: None,
                id: id.into(),
                document_revision: revision,
                sequence,
                status: RunStatus::Converged,
                summary,
                diagnostics: Vec::new(),
                streams,
            },
            result: Arc::new(result),
            final_units,
        }
    }
}
impl Deref for PartitionedFlowSnapshot {
    type Target = FlowSnapshotHeader;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
impl DerefMut for PartitionedFlowSnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FlowResultRecord {
    Acyclic(SolveSnapshot),
    Partitioned(PartitionedFlowSnapshot),
}
impl From<SolveSnapshot> for FlowResultRecord {
    fn from(v: SolveSnapshot) -> Self {
        Self::Acyclic(v)
    }
}
impl From<PartitionedFlowSnapshot> for FlowResultRecord {
    fn from(v: PartitionedFlowSnapshot) -> Self {
        Self::Partitioned(v)
    }
}
impl Deref for FlowResultRecord {
    type Target = FlowSnapshotHeader;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Acyclic(s) => &s.common,
            Self::Partitioned(s) => &s.common,
        }
    }
}
impl DerefMut for FlowResultRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Acyclic(s) => &mut s.common,
            Self::Partitioned(s) => &mut s.common,
        }
    }
}
impl FlowResultRecord {
    pub fn matches_plan(&self, plan: &crate::flow_plan::FlowPlanConfiguration) -> bool {
        matches!(
            (self, plan),
            (
                Self::Acyclic(_),
                crate::flow_plan::FlowPlanConfiguration::Acyclic
            ) | (
                Self::Partitioned(_),
                crate::flow_plan::FlowPlanConfiguration::Partitioned(_)
            )
        )
    }
    pub fn unit_result(&self, id: &UnitId) -> Option<UnitResultView<'_>> {
        match self {
            Self::Acyclic(s) => s
                .steps
                .iter()
                .find(|u| &u.unit_id == id)
                .map(|u| UnitResultView {
                    consumed_streams: &u.consumed_streams,
                    produced_streams: &u.streams,
                }),
            Self::Partitioned(s) => s.final_units.get(id).map(|u| UnitResultView {
                consumed_streams: &u.consumed_streams,
                produced_streams: &u.produced_streams,
            }),
        }
    }
    pub fn acyclic(&self) -> Option<&SolveSnapshot> {
        match self {
            Self::Acyclic(s) => Some(s),
            Self::Partitioned(_) => None,
        }
    }
}
