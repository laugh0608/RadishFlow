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
