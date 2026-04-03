use crate::run::{RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot};
use crate::state::AppLogEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPanelNoticeLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelNotice {
    pub level: RunPanelNoticeLevel,
    pub title: String,
    pub message: String,
}

impl RunPanelNotice {
    pub fn new(
        level: RunPanelNoticeLevel,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            title: title.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelState {
    pub simulation_mode: SimulationMode,
    pub run_status: RunStatus,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub latest_log_message: Option<String>,
    pub notice: Option<RunPanelNotice>,
    pub can_run_manual: bool,
    pub can_resume: bool,
    pub can_set_hold: bool,
    pub can_set_active: bool,
    pub commands: RunPanelCommandModel,
}

impl RunPanelState {
    pub fn from_runtime(
        solve_session: &SolveSessionState,
        latest_snapshot: Option<&SolveSnapshot>,
        latest_log_entry: Option<&AppLogEntry>,
    ) -> Self {
        let simulation_mode = solve_session.mode;
        let pending_reason = solve_session.pending_reason;
        let can_resume = matches!(simulation_mode, SimulationMode::Hold);
        let can_set_hold = !matches!(simulation_mode, SimulationMode::Hold);
        let can_set_active = !matches!(simulation_mode, SimulationMode::Active);

        let mut state = Self {
            simulation_mode,
            run_status: solve_session.status,
            pending_reason,
            latest_snapshot_id: latest_snapshot.map(|snapshot| snapshot.id.as_str().to_string()),
            latest_snapshot_summary: latest_snapshot
                .map(|snapshot| snapshot.summary.primary_message.clone()),
            latest_log_message: latest_log_entry.map(|entry| entry.message.clone()),
            notice: None,
            can_run_manual: true,
            can_resume,
            can_set_hold,
            can_set_active,
            commands: RunPanelCommandModel::default(),
        };
        state.commands = RunPanelCommandModel::from_state(&state);
        state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunPanelActionId {
    RunManual,
    Resume,
    SetHold,
    SetActive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelActionModel {
    pub id: RunPanelActionId,
    pub label: &'static str,
    pub intent: RunPanelIntent,
    pub enabled: bool,
    pub visible: bool,
}

impl RunPanelActionModel {
    fn new(
        id: RunPanelActionId,
        label: &'static str,
        intent: RunPanelIntent,
        enabled: bool,
        visible: bool,
    ) -> Self {
        Self {
            id,
            label,
            intent,
            enabled,
            visible,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelCommandModel {
    pub primary_action: RunPanelActionId,
    pub actions: Vec<RunPanelActionModel>,
}

impl Default for RunPanelCommandModel {
    fn default() -> Self {
        Self {
            primary_action: RunPanelActionId::RunManual,
            actions: Vec::new(),
        }
    }
}

impl RunPanelCommandModel {
    pub fn from_state(state: &RunPanelState) -> Self {
        let preferred_selection = RunPanelPackageSelection::preferred();
        let resume_enabled = state.can_resume && state.pending_reason.is_some();
        let primary_action = if resume_enabled {
            RunPanelActionId::Resume
        } else {
            RunPanelActionId::RunManual
        };

        Self {
            primary_action,
            actions: vec![
                RunPanelActionModel::new(
                    RunPanelActionId::RunManual,
                    "Run",
                    RunPanelIntent::run_manual(preferred_selection.clone()),
                    state.can_run_manual,
                    true,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::Resume,
                    "Resume",
                    RunPanelIntent::resume(preferred_selection),
                    resume_enabled,
                    state.can_resume,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::SetHold,
                    "Hold",
                    RunPanelIntent::set_mode(SimulationMode::Hold),
                    state.can_set_hold,
                    true,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::SetActive,
                    "Active",
                    RunPanelIntent::set_mode(SimulationMode::Active),
                    state.can_set_active,
                    true,
                ),
            ],
        }
    }

    pub fn action(&self, id: RunPanelActionId) -> Option<&RunPanelActionModel> {
        self.actions.iter().find(|action| action.id == id)
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
