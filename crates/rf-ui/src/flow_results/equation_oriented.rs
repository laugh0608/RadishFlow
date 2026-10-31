use super::*;
use rf_solver::equation_oriented::EquationResult;

#[derive(Debug, Clone, PartialEq)]
pub struct EquationFlowSnapshot {
    pub common: FlowSnapshotHeader,
    pub result: Arc<EquationResult>,
    pub final_units: BTreeMap<UnitId, UnitResultSnapshot>,
}

impl EquationFlowSnapshot {
    pub fn new(
        id: impl Into<SolveSnapshotId>,
        revision: u64,
        sequence: u64,
        result: EquationResult,
    ) -> Self {
        let streams = result
            .streams
            .values()
            .map(crate::run::stream_state_snapshot_from_model)
            .collect();
        let final_units = final_unit_snapshots(result.evidence.final_units.iter());
        let summary = DiagnosticSummary::new(
            revision,
            DiagnosticSeverity::Info,
            format!(
                "{} variables completed with verified fixed-T/P material equation closure",
                result
                    .evidence
                    .definition
                    .as_ref()
                    .expect("successful EO definition")
                    .variables
                    .len()
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

impl Deref for EquationFlowSnapshot {
    type Target = FlowSnapshotHeader;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
impl DerefMut for EquationFlowSnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
