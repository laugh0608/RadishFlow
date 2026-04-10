use rf_types::UnitId;

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
    pub recovery_action: Option<RunPanelRecoveryAction>,
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
            recovery_action: None,
        }
    }

    pub fn with_recovery_action(mut self, recovery_action: RunPanelRecoveryAction) -> Self {
        self.recovery_action = Some(recovery_action);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPanelRecoveryActionKind {
    FixConnections,
    BreakCycle,
    VerifyUnitExists,
    InspectUnitSpec,
    CheckSupportedUnitKind,
    InspectInletPath,
    InspectOutputMaterialization,
    InspectExecutionInputs,
    RepairLocalCache,
    InspectFailureDetails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelRecoveryAction {
    pub kind: RunPanelRecoveryActionKind,
    pub title: &'static str,
    pub detail: &'static str,
    pub target_unit_id: Option<UnitId>,
}

impl RunPanelRecoveryAction {
    pub fn new(
        kind: RunPanelRecoveryActionKind,
        title: &'static str,
        detail: &'static str,
    ) -> Self {
        Self {
            kind,
            title,
            detail,
            target_unit_id: None,
        }
    }

    pub fn with_target_unit(mut self, target_unit_id: impl Into<UnitId>) -> Self {
        self.target_unit_id = Some(target_unit_id.into());
        self
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
            notice: runtime_notice_from_solve_session(solve_session),
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

pub fn run_panel_failure_title_for_diagnostic_code(primary_code: Option<&str>) -> &'static str {
    match primary_code {
        Some("solver.connection_validation") => "Connection validation failed",
        Some("solver.topological_ordering") => "Topological ordering failed",
        Some("solver.step.lookup") => "Unit lookup failed",
        Some("solver.step.spec") => "Unit specification failed",
        Some("solver.step.instantiation") => "Operation instantiation failed",
        Some("solver.step.inlet") => "Inlet resolution failed",
        Some("solver.step.materialization") => "Output materialization failed",
        Some("solver.step.execution") => "Unit execution failed",
        _ => "Run failed",
    }
}

pub fn run_panel_failure_recovery_action_for_diagnostic_code(
    primary_code: Option<&str>,
) -> Option<RunPanelRecoveryAction> {
    match primary_code {
        Some("solver.connection_validation") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Fix connections",
            "检查流股连接、端口签名和缺失 stream 引用后再重试。",
        )),
        Some("solver.topological_ordering") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::BreakCycle,
            "Break cycle",
            "消除自环或多单元回路后再重试，当前顺序模块法只支持无回路 flowsheet。",
        )),
        Some("solver.step.lookup") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::VerifyUnitExists,
            "Verify unit exists",
            "确认当前 step 引用的 unit 仍存在于 flowsheet 中。",
        )),
        Some("solver.step.spec") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectUnitSpec,
            "Inspect unit specs",
            "检查该单元的端口配置和必填规格是否完整。",
        )),
        Some("solver.step.instantiation") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::CheckSupportedUnitKind,
            "Check supported unit kind",
            "检查单元 kind 与内建 solver 支持范围是否匹配。",
        )),
        Some("solver.step.inlet") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectInletPath,
            "Inspect inlet path",
            "检查入口连接是否完整，以及上游流股是否应先于该单元求解。",
        )),
        Some("solver.step.materialization") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectOutputMaterialization,
            "Inspect outlet materialization",
            "检查单元是否为每个预期 outlet 产出了对应流股。",
        )),
        Some("solver.step.execution") => Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectExecutionInputs,
            "Inspect unit inputs",
            "检查单元规格、物性条件和入口状态是否满足执行前提。",
        )),
        _ => None,
    }
}

pub fn run_panel_failure_notice(
    message: impl Into<String>,
    primary_code: Option<&str>,
    target_unit_id: Option<&UnitId>,
) -> RunPanelNotice {
    let mut notice = RunPanelNotice::new(
        RunPanelNoticeLevel::Error,
        run_panel_failure_title_for_diagnostic_code(primary_code),
        message,
    );
    if let Some(mut recovery_action) =
        run_panel_failure_recovery_action_for_diagnostic_code(primary_code)
    {
        if let Some(unit_id) = target_unit_id {
            recovery_action = recovery_action.with_target_unit(unit_id.clone());
        }
        notice = notice.with_recovery_action(recovery_action);
    }
    notice
}

fn runtime_notice_from_solve_session(solve_session: &SolveSessionState) -> Option<RunPanelNotice> {
    if !matches!(solve_session.status, RunStatus::Error) {
        return None;
    }

    let summary = solve_session.latest_diagnostic.as_ref()?;
    Some(run_panel_failure_notice(
        summary.primary_message.clone(),
        summary.primary_code.as_deref(),
        summary.related_unit_ids.first(),
    ))
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
    pub detail: &'static str,
    pub intent: RunPanelIntent,
    pub enabled: bool,
    pub visible: bool,
}

impl RunPanelActionModel {
    fn new(
        id: RunPanelActionId,
        label: &'static str,
        detail: &'static str,
        intent: RunPanelIntent,
        enabled: bool,
        visible: bool,
    ) -> Self {
        Self {
            id,
            label,
            detail,
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
                    run_manual_detail(state),
                    RunPanelIntent::run_manual(preferred_selection.clone()),
                    state.can_run_manual,
                    true,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::Resume,
                    "Resume",
                    resume_detail(state),
                    RunPanelIntent::resume(preferred_selection),
                    resume_enabled,
                    state.can_resume,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::SetHold,
                    "Hold",
                    set_hold_detail(state),
                    RunPanelIntent::set_mode(SimulationMode::Hold),
                    state.can_set_hold,
                    true,
                ),
                RunPanelActionModel::new(
                    RunPanelActionId::SetActive,
                    "Active",
                    set_active_detail(state),
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

fn run_manual_detail(state: &RunPanelState) -> &'static str {
    if state.can_run_manual {
        "Run the current workspace once"
    } else {
        "Manual run is unavailable in the current workspace state"
    }
}

fn resume_detail(state: &RunPanelState) -> &'static str {
    if state.can_resume && state.pending_reason.is_some() {
        "Resume pending work while the workspace stays in Hold mode"
    } else if !state.can_resume {
        "Switch the workspace to Hold mode before resuming"
    } else if state.pending_reason.is_none() {
        "No pending work is waiting to resume"
    } else {
        "Resume is unavailable in the current workspace state"
    }
}

fn set_hold_detail(state: &RunPanelState) -> &'static str {
    if state.can_set_hold {
        "Pause automatic solving and keep the workspace in Hold mode"
    } else {
        "Workspace is already in Hold mode"
    }
}

fn set_active_detail(state: &RunPanelState) -> &'static str {
    if state.can_set_active {
        "Enable automatic solving for future pending work"
    } else {
        "Workspace is already in Active mode"
    }
}
