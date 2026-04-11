use rf_types::{DiagnosticPortTarget, StreamId, UnitId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelRecoveryMutation {
    DisconnectPort { unit_id: UnitId, port_name: String },
    DeleteStream { stream_id: StreamId },
    CreateAndBindOutletStream { unit_id: UnitId, port_name: String },
}

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
    pub target_stream_id: Option<StreamId>,
    pub target_port_name: Option<String>,
    pub mutation: Option<RunPanelRecoveryMutation>,
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
            target_stream_id: None,
            target_port_name: None,
            mutation: None,
        }
    }

    pub fn with_target_unit(mut self, target_unit_id: impl Into<UnitId>) -> Self {
        self.target_unit_id = Some(target_unit_id.into());
        self.target_stream_id = None;
        self.target_port_name = None;
        self
    }

    pub fn with_target_stream(mut self, target_stream_id: impl Into<StreamId>) -> Self {
        self.target_stream_id = Some(target_stream_id.into());
        self.target_unit_id = None;
        self.target_port_name = None;
        self
    }

    pub fn with_target_port(
        mut self,
        target_unit_id: impl Into<UnitId>,
        target_port_name: impl Into<String>,
    ) -> Self {
        self.target_unit_id = Some(target_unit_id.into());
        self.target_port_name = Some(target_port_name.into());
        self.target_stream_id = None;
        self
    }

    pub fn with_mutation(mut self, mutation: RunPanelRecoveryMutation) -> Self {
        self.mutation = Some(mutation);
        self
    }

    pub fn with_disconnect_port(
        self,
        unit_id: impl Into<UnitId>,
        port_name: impl Into<String>,
    ) -> Self {
        let unit_id = unit_id.into();
        let port_name = port_name.into();
        self.with_target_port(unit_id.clone(), port_name.clone())
            .with_mutation(RunPanelRecoveryMutation::DisconnectPort { unit_id, port_name })
    }

    pub fn with_delete_stream(self, stream_id: impl Into<StreamId>) -> Self {
        let stream_id = stream_id.into();
        self.with_target_stream(stream_id.clone())
            .with_mutation(RunPanelRecoveryMutation::DeleteStream { stream_id })
    }

    pub fn with_create_and_bind_outlet_stream(
        self,
        unit_id: impl Into<UnitId>,
        port_name: impl Into<String>,
    ) -> Self {
        let unit_id = unit_id.into();
        let port_name = port_name.into();
        self.with_target_port(unit_id.clone(), port_name.clone())
            .with_mutation(RunPanelRecoveryMutation::CreateAndBindOutletStream {
                unit_id,
                port_name,
            })
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
    if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unsupported_unit_kind",
    ) {
        "Unsupported unit kind"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.invalid_port_signature",
    ) {
        "Invalid port signature"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.missing_upstream_source",
    ) {
        "Missing upstream source"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.missing_stream_reference",
    ) {
        "Missing stream reference"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_upstream_source",
    ) {
        "Duplicate stream source"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_downstream_sink",
    ) {
        "Duplicate stream sink"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unbound_inlet_port",
    ) {
        "Unbound inlet port"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unbound_outlet_port",
    ) {
        "Unbound outlet port"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.orphan_stream",
    ) {
        "Orphan stream"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.topological_ordering.self_loop_cycle",
    ) {
        "Self loop detected"
    } else if diagnostic_code_matches(
        primary_code,
        "solver.topological_ordering.two_unit_cycle",
    ) {
        "Two-unit cycle detected"
    } else if diagnostic_code_in_family(primary_code, "solver.connection_validation") {
        "Connection validation failed"
    } else if diagnostic_code_in_family(primary_code, "solver.topological_ordering") {
        "Topological ordering failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.lookup") {
        "Unit lookup failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.spec") {
        "Unit specification failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.instantiation") {
        "Operation instantiation failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.inlet") {
        "Inlet resolution failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.materialization") {
        "Output materialization failed"
    } else if diagnostic_code_in_family(primary_code, "solver.step.execution") {
        "Unit execution failed"
    } else {
        "Run failed"
    }
}

pub fn run_panel_failure_recovery_action_for_diagnostic_code(
    primary_code: Option<&str>,
) -> Option<RunPanelRecoveryAction> {
    if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unsupported_unit_kind",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::CheckSupportedUnitKind,
            "Check supported unit kind",
            "检查单元 kind 是否属于当前内建 solver 支持范围，并确认是否误用了本阶段尚未支持的单元类型。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.invalid_port_signature",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectUnitSpec,
            "Inspect unit specs",
            "检查该单元的端口名称、方向、类型和数量是否与 canonical built-in spec 一致。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.missing_upstream_source",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectInletPath,
            "Inspect inlet path",
            "检查入口流股是否缺少上游 outlet source，并确认上游单元是否已绑定到同一 stream。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.missing_stream_reference",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Disconnect invalid stream reference",
            "断开当前端口上指向缺失 stream 的引用，避免无效 stream id 继续阻塞连接校验。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_upstream_source",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Disconnect conflicting source",
            "断开冲突的 outlet source 端口，让该流股只保留一个上游来源后再继续修复。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_downstream_sink",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Disconnect conflicting sink",
            "断开冲突的 inlet sink 端口，让该流股只保留一个下游去向后再继续修复。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unbound_inlet_port",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectInletPath,
            "Inspect inlet path",
            "检查该 inlet 端口应接入哪条上游流股，并确认是否遗漏了 stream 绑定。",
        ))
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unbound_outlet_port",
    ) {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Create outlet stream",
            "为当前未绑定 stream 的 outlet 端口创建一条占位流股，并立即写回连接。",
        ))
    } else if diagnostic_code_matches(primary_code, "solver.connection_validation.orphan_stream") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Delete orphan stream",
            "删除当前未连接到任何单元端口的孤立流股，避免它继续阻塞连接校验。",
        ))
    } else if diagnostic_code_matches(primary_code, "solver.topological_ordering.self_loop_cycle") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::BreakCycle,
            "Disconnect self-loop inlet",
            "断开当前单元引用自身 outlet stream 的 inlet 端口，先消除自环依赖，再继续检查剩余连接问题。",
        ))
    } else if diagnostic_code_matches(primary_code, "solver.topological_ordering.two_unit_cycle") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::BreakCycle,
            "Disconnect cycle inlet",
            "断开当前双单元回路中的一个 inlet 端口，先打破互相依赖，再继续检查剩余连接问题。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.connection_validation") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::FixConnections,
            "Fix connections",
            "检查流股连接、端口签名和缺失 stream 引用后再重试。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.topological_ordering") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::BreakCycle,
            "Break cycle",
            "消除自环或多单元回路后再重试，当前顺序模块法只支持无回路 flowsheet。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.lookup") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::VerifyUnitExists,
            "Verify unit exists",
            "确认当前 step 引用的 unit 仍存在于 flowsheet 中。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.spec") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectUnitSpec,
            "Inspect unit specs",
            "检查该单元的端口配置和必填规格是否完整。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.instantiation") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::CheckSupportedUnitKind,
            "Check supported unit kind",
            "检查单元 kind 与内建 solver 支持范围是否匹配。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.inlet") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectInletPath,
            "Inspect inlet path",
            "检查入口连接是否完整，以及上游流股是否应先于该单元求解。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.materialization") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectOutputMaterialization,
            "Inspect outlet materialization",
            "检查单元是否为每个预期 outlet 产出了对应流股。",
        ))
    } else if diagnostic_code_in_family(primary_code, "solver.step.execution") {
        Some(RunPanelRecoveryAction::new(
            RunPanelRecoveryActionKind::InspectExecutionInputs,
            "Inspect unit inputs",
            "检查单元规格、物性条件和入口状态是否满足执行前提。",
        ))
    } else {
        None
    }
}

pub fn run_panel_failure_notice(
    message: impl Into<String>,
    primary_code: Option<&str>,
    target_unit_id: Option<&UnitId>,
    target_stream_id: Option<&StreamId>,
    related_port_targets: &[DiagnosticPortTarget],
) -> RunPanelNotice {
    let mut notice = RunPanelNotice::new(
        RunPanelNoticeLevel::Error,
        run_panel_failure_title_for_diagnostic_code(primary_code),
        message,
    );
    if let Some(mut recovery_action) =
        run_panel_failure_recovery_action_for_diagnostic_code(primary_code)
    {
        if let Some(port_target) = preferred_recovery_port_target(primary_code, related_port_targets)
        {
            recovery_action =
                configure_recovery_action_for_port_target(recovery_action, primary_code, port_target);
        } else if prefers_stream_recovery_target(primary_code) {
            if let Some(stream_id) = target_stream_id {
                recovery_action =
                    configure_recovery_action_for_stream_target(recovery_action, primary_code, stream_id);
            } else if let Some(unit_id) = target_unit_id {
                recovery_action = recovery_action.with_target_unit(unit_id.clone());
            }
        } else if let Some(unit_id) = target_unit_id {
            recovery_action = recovery_action.with_target_unit(unit_id.clone());
        } else if let Some(stream_id) = target_stream_id {
            recovery_action = recovery_action.with_target_stream(stream_id.clone());
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
        summary.related_stream_ids.first(),
        &summary.related_port_targets,
    ))
}

fn diagnostic_code_matches(primary_code: Option<&str>, expected: &str) -> bool {
    matches!(primary_code, Some(code) if code == expected)
}

fn diagnostic_code_in_family(primary_code: Option<&str>, family: &str) -> bool {
    matches!(primary_code, Some(code) if code == family)
        || matches!(
            primary_code,
            Some(code)
                if code
                    .strip_prefix(family)
                    .map(|suffix| suffix.starts_with('.'))
                    .unwrap_or(false)
        )
}

fn prefers_stream_recovery_target(primary_code: Option<&str>) -> bool {
    diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_upstream_source",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_downstream_sink",
    ) || diagnostic_code_matches(primary_code, "solver.connection_validation.orphan_stream")
}

fn preferred_recovery_port_target<'a>(
    primary_code: Option<&str>,
    related_port_targets: &'a [DiagnosticPortTarget],
) -> Option<&'a DiagnosticPortTarget> {
    if related_port_targets.is_empty() {
        return None;
    }

    if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_upstream_source",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_downstream_sink",
    ) {
        return related_port_targets.last();
    }

    if diagnostic_code_matches(primary_code, "solver.topological_ordering.two_unit_cycle") {
        return related_port_targets.last();
    }

    related_port_targets.first()
}

fn configure_recovery_action_for_port_target(
    recovery_action: RunPanelRecoveryAction,
    primary_code: Option<&str>,
    port_target: &DiagnosticPortTarget,
) -> RunPanelRecoveryAction {
    if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.missing_stream_reference",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_upstream_source",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.duplicate_downstream_sink",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.topological_ordering.self_loop_cycle",
    ) || diagnostic_code_matches(
        primary_code,
        "solver.topological_ordering.two_unit_cycle",
    ) {
        recovery_action
            .with_disconnect_port(port_target.unit_id.clone(), port_target.port_name.clone())
    } else if diagnostic_code_matches(
        primary_code,
        "solver.connection_validation.unbound_outlet_port",
    ) {
        recovery_action.with_create_and_bind_outlet_stream(
            port_target.unit_id.clone(),
            port_target.port_name.clone(),
        )
    } else {
        recovery_action.with_target_port(port_target.unit_id.clone(), port_target.port_name.clone())
    }
}

fn configure_recovery_action_for_stream_target(
    recovery_action: RunPanelRecoveryAction,
    primary_code: Option<&str>,
    stream_id: &StreamId,
) -> RunPanelRecoveryAction {
    if diagnostic_code_matches(primary_code, "solver.connection_validation.orphan_stream") {
        recovery_action.with_delete_stream(stream_id.clone())
    } else {
        recovery_action.with_target_stream(stream_id.clone())
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
