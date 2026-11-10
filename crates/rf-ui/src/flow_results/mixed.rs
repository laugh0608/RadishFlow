use super::*;
use rf_solver::mixed::MixedResult;

#[derive(Debug, Clone, PartialEq)]
pub struct MixedFlowSnapshot {
    pub common: FlowSnapshotHeader,
    pub result: Arc<MixedResult>,
    pub final_units: BTreeMap<UnitId, UnitResultSnapshot>,
}

impl MixedFlowSnapshot {
    pub fn new(
        id: impl Into<SolveSnapshotId>,
        revision: u64,
        sequence: u64,
        result: MixedResult,
    ) -> Self {
        let streams = result
            .streams
            .values()
            .map(crate::run::stream_state_snapshot_from_model)
            .collect();
        let final_units =
            final_unit_snapshots(result.blocks.iter().flat_map(|b| b.evidence.final_units()));
        let summary = DiagnosticSummary::new(
            revision,
            DiagnosticSeverity::Info,
            format!(
                "{} mixed blocks completed with verified global material closure",
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

impl Deref for MixedFlowSnapshot {
    type Target = FlowSnapshotHeader;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
impl DerefMut for MixedFlowSnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
