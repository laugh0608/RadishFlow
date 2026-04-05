use rf_types::{StreamId, UnitId};

use crate::diagnostics::{DiagnosticSnapshot, DiagnosticSummary};
use crate::ids::SolveSnapshotId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationMode {
    Active,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunStatus {
    Idle,
    Dirty,
    Checking,
    Runnable,
    Solving,
    Converged,
    UnderSpecified,
    OverSpecified,
    Unconverged,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolvePendingReason {
    DocumentRevisionAdvanced,
    ModeActivated,
    ManualRunRequested,
    SnapshotMissing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamStateSnapshot {
    pub stream_id: StreamId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitExecutionSnapshot {
    pub unit_id: UnitId,
    pub status: RunStatus,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepSnapshot {
    pub index: usize,
    pub unit_id: UnitId,
    pub summary: String,
    pub execution: UnitExecutionSnapshot,
    pub streams: Vec<StreamStateSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveSnapshot {
    pub id: SolveSnapshotId,
    pub document_revision: u64,
    pub sequence: u64,
    pub status: RunStatus,
    pub summary: DiagnosticSummary,
    pub diagnostics: Vec<DiagnosticSnapshot>,
    pub steps: Vec<StepSnapshot>,
}

impl SolveSnapshot {
    pub fn new(
        id: impl Into<SolveSnapshotId>,
        document_revision: u64,
        sequence: u64,
        status: RunStatus,
        summary: DiagnosticSummary,
    ) -> Self {
        Self {
            id: id.into(),
            document_revision,
            sequence,
            status,
            summary,
            diagnostics: Vec::new(),
            steps: Vec::new(),
        }
    }

    pub fn from_solver_snapshot(
        id: impl Into<SolveSnapshotId>,
        document_revision: u64,
        sequence: u64,
        snapshot: &rf_solver::SolveSnapshot,
    ) -> Self {
        Self {
            id: id.into(),
            document_revision,
            sequence,
            status: map_solver_status(snapshot.status),
            summary: DiagnosticSummary {
                document_revision,
                highest_severity: map_solver_severity(snapshot.summary.highest_severity),
                primary_code: snapshot
                    .diagnostics
                    .first()
                    .map(|diagnostic| diagnostic.code.clone()),
                primary_message: snapshot.summary.primary_message.clone(),
                diagnostic_count: snapshot.summary.diagnostic_count,
                related_unit_ids: snapshot.summary.related_unit_ids.clone(),
            },
            diagnostics: snapshot
                .diagnostics
                .iter()
                .map(|diagnostic| DiagnosticSnapshot {
                    severity: map_solver_severity(diagnostic.severity),
                    code: diagnostic.code.clone(),
                    message: diagnostic.message.clone(),
                    related_unit_ids: diagnostic.related_unit_ids.clone(),
                })
                .collect(),
            steps: snapshot
                .steps
                .iter()
                .map(|step| StepSnapshot {
                    index: step.index,
                    unit_id: step.unit_id.clone(),
                    summary: step.summary.clone(),
                    execution: UnitExecutionSnapshot {
                        unit_id: step.unit_id.clone(),
                        status: RunStatus::Converged,
                        summary: step.summary.clone(),
                    },
                    streams: step
                        .produced_stream_ids
                        .iter()
                        .map(|stream_id| StreamStateSnapshot {
                            stream_id: stream_id.clone(),
                            label: stream_id.as_str().to_string(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

fn map_solver_status(status: rf_solver::SolveStatus) -> RunStatus {
    match status {
        rf_solver::SolveStatus::Converged => RunStatus::Converged,
    }
}

fn map_solver_severity(severity: rf_solver::SolveDiagnosticSeverity) -> crate::DiagnosticSeverity {
    match severity {
        rf_solver::SolveDiagnosticSeverity::Info => crate::DiagnosticSeverity::Info,
        rf_solver::SolveDiagnosticSeverity::Warning => crate::DiagnosticSeverity::Warning,
        rf_solver::SolveDiagnosticSeverity::Error => crate::DiagnosticSeverity::Error,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveSessionState {
    pub mode: SimulationMode,
    pub status: RunStatus,
    pub observed_revision: u64,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot: Option<SolveSnapshotId>,
    pub latest_diagnostic: Option<DiagnosticSummary>,
}

impl SolveSessionState {
    pub fn new(observed_revision: u64) -> Self {
        Self {
            mode: SimulationMode::Hold,
            status: RunStatus::Idle,
            observed_revision,
            pending_reason: Some(SolvePendingReason::SnapshotMissing),
            latest_snapshot: None,
            latest_diagnostic: None,
        }
    }

    pub fn mark_document_revision_advanced(&mut self, revision: u64) {
        self.observed_revision = revision;
        self.status = RunStatus::Dirty;
        self.pending_reason = Some(SolvePendingReason::DocumentRevisionAdvanced);
    }

    pub fn activate(&mut self) {
        self.mode = SimulationMode::Active;
        self.pending_reason = Some(SolvePendingReason::ModeActivated);
    }

    pub fn request_manual_run(&mut self) {
        self.pending_reason = Some(SolvePendingReason::ManualRunRequested);
        if matches!(self.status, RunStatus::Idle | RunStatus::Converged) {
            self.status = RunStatus::Dirty;
        }
    }

    pub fn begin_checking(&mut self, revision: u64) {
        self.observed_revision = revision;
        self.status = RunStatus::Checking;
    }

    pub fn mark_runnable(&mut self) {
        self.status = RunStatus::Runnable;
    }

    pub fn begin_solving(&mut self) {
        self.status = RunStatus::Solving;
    }

    pub fn complete_with_snapshot(&mut self, snapshot: &SolveSnapshot) {
        self.observed_revision = snapshot.document_revision;
        self.latest_snapshot = Some(snapshot.id.clone());
        self.latest_diagnostic = Some(snapshot.summary.clone());
        self.status = snapshot.status;
        self.pending_reason = None;
    }

    pub fn hold_with_failure(
        &mut self,
        revision: u64,
        status: RunStatus,
        summary: DiagnosticSummary,
    ) {
        self.observed_revision = revision;
        self.status = status;
        self.latest_diagnostic = Some(summary);
        self.pending_reason = None;
        self.mode = SimulationMode::Hold;
    }
}
