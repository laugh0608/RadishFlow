use crate::run::{RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot};
use crate::state::AppLogEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelState {
    pub simulation_mode: SimulationMode,
    pub run_status: RunStatus,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub latest_log_message: Option<String>,
    pub can_run_manual: bool,
    pub can_resume: bool,
    pub can_set_hold: bool,
    pub can_set_active: bool,
}

impl RunPanelState {
    pub fn from_runtime(
        solve_session: &SolveSessionState,
        latest_snapshot: Option<&SolveSnapshot>,
        latest_log_entry: Option<&AppLogEntry>,
    ) -> Self {
        let simulation_mode = solve_session.mode;

        Self {
            simulation_mode,
            run_status: solve_session.status,
            pending_reason: solve_session.pending_reason,
            latest_snapshot_id: latest_snapshot.map(|snapshot| snapshot.id.as_str().to_string()),
            latest_snapshot_summary: latest_snapshot
                .map(|snapshot| snapshot.summary.primary_message.clone()),
            latest_log_message: latest_log_entry.map(|entry| entry.message.clone()),
            can_run_manual: true,
            can_resume: matches!(simulation_mode, SimulationMode::Hold),
            can_set_hold: !matches!(simulation_mode, SimulationMode::Hold),
            can_set_active: !matches!(simulation_mode, SimulationMode::Active),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelPackageSelection {
    Explicit(String),
    Preferred,
}

impl RunPanelPackageSelection {
    pub fn explicit(package_id: impl Into<String>) -> Self {
        Self::Explicit(package_id.into())
    }

    pub fn preferred() -> Self {
        Self::Preferred
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelIntent {
    RunManual(RunPanelPackageSelection),
    Resume(RunPanelPackageSelection),
    SetMode(SimulationMode),
}

impl RunPanelIntent {
    pub fn run_manual(package: RunPanelPackageSelection) -> Self {
        Self::RunManual(package)
    }

    pub fn resume(package: RunPanelPackageSelection) -> Self {
        Self::Resume(package)
    }

    pub fn set_mode(mode: SimulationMode) -> Self {
        Self::SetMode(mode)
    }
}
