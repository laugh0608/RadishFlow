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
pub struct PhaseStateSnapshot {
    pub label: String,
    pub phase_fraction: f64,
    pub composition: Vec<(String, f64)>,
    pub molar_enthalpy_j_per_mol: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamStateSnapshot {
    pub stream_id: StreamId,
    pub label: String,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub total_molar_flow_mol_s: f64,
    pub overall_mole_fractions: Vec<(String, f64)>,
    pub phases: Vec<PhaseStateSnapshot>,
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
    pub streams: Vec<StreamStateSnapshot>,
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
            streams: Vec::new(),
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
                related_stream_ids: snapshot.summary.related_stream_ids.clone(),
                related_port_targets: Vec::new(),
            },
            diagnostics: snapshot
                .diagnostics
                .iter()
                .map(|diagnostic| DiagnosticSnapshot {
                    severity: map_solver_severity(diagnostic.severity),
                    code: diagnostic.code.clone(),
                    message: diagnostic.message.clone(),
                    related_unit_ids: diagnostic.related_unit_ids.clone(),
                    related_stream_ids: diagnostic.related_stream_ids.clone(),
                    related_port_targets: Vec::new(),
                })
                .collect(),
            streams: snapshot
                .streams
                .values()
                .map(stream_state_snapshot_from_model)
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
                        .map(|stream_id| {
                            snapshot
                                .streams
                                .get(stream_id)
                                .map(stream_state_snapshot_from_model)
                                .unwrap_or_else(|| StreamStateSnapshot {
                                    stream_id: stream_id.clone(),
                                    label: stream_id.as_str().to_string(),
                                    temperature_k: 0.0,
                                    pressure_pa: 0.0,
                                    total_molar_flow_mol_s: 0.0,
                                    overall_mole_fractions: Vec::new(),
                                    phases: Vec::new(),
                                })
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

fn stream_state_snapshot_from_model(stream: &rf_model::MaterialStreamState) -> StreamStateSnapshot {
    StreamStateSnapshot {
        stream_id: stream.id.clone(),
        label: stream.name.clone(),
        temperature_k: stream.temperature_k,
        pressure_pa: stream.pressure_pa,
        total_molar_flow_mol_s: stream.total_molar_flow_mol_s,
        overall_mole_fractions: stream
            .overall_mole_fractions
            .iter()
            .map(|(component_id, fraction)| (component_id.as_str().to_string(), *fraction))
            .collect(),
        phases: stream
            .phases
            .iter()
            .map(|phase| PhaseStateSnapshot {
                label: phase.label.as_str().to_string(),
                phase_fraction: phase.phase_fraction,
                composition: phase
                    .mole_fractions
                    .iter()
                    .map(|(component_id, fraction)| (component_id.as_str().to_string(), *fraction))
                    .collect(),
                molar_enthalpy_j_per_mol: phase.molar_enthalpy_j_per_mol,
            })
            .collect(),
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
        self.latest_snapshot = None;
        self.latest_diagnostic = None;
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
        self.latest_snapshot = None;
        self.latest_diagnostic = Some(summary);
        self.pending_reason = None;
        self.mode = SimulationMode::Hold;
    }
}
