use super::*;
use rf_solver::simultaneous::ModularResult;

#[derive(Debug, Clone, PartialEq)]
pub struct SimultaneousFlowSnapshot {
    pub common: FlowSnapshotHeader,
    pub result: Arc<ModularResult>,
    pub final_units: BTreeMap<UnitId, UnitResultSnapshot>,
}

impl SimultaneousFlowSnapshot {
    pub fn new(
        id: impl Into<SolveSnapshotId>,
        revision: u64,
        sequence: u64,
        result: ModularResult,
    ) -> Self {
        let streams = result
            .streams
            .values()
            .map(crate::run::stream_state_snapshot_from_model)
            .collect();
        let final_units = final_unit_snapshots(result.blocks.iter().flat_map(|b| &b.final_units));
        let summary = DiagnosticSummary::new(
            revision,
            DiagnosticSeverity::Info,
            format!(
                "{} dependency blocks completed with verified fixed-T/P affine material closure",
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

impl Deref for SimultaneousFlowSnapshot {
    type Target = FlowSnapshotHeader;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
impl DerefMut for SimultaneousFlowSnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
