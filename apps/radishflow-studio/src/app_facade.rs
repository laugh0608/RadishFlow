use std::path::Path;

use rf_solver::SolveFailureContext;
use rf_store::StoredAuthCacheIndex;
use rf_types::{ErrorCode, RfError, RfResult};
use rf_ui::{
    AppLogEntry, AppLogLevel, AppState, DiagnosticSeverity, DiagnosticSummary, RunStatus,
    SimulationMode, SolvePendingReason, latest_snapshot, latest_snapshot_id,
};

use crate::{
    StudioSolveRequest, WorkspaceRunCommand, WorkspaceRunPackageSelection, WorkspaceSolveDispatch,
    WorkspaceSolveService, WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    resolve_workspace_run_package_id,
    solver_bridge::WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE,
    workspace_run_command::{
        WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING,
        WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH,
        WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED,
        WORKSPACE_RUN_DIAGNOSTIC_INVALID_SELECTION,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppCommand {
    RunWorkspace(WorkspaceRunCommand),
    ResumeWorkspace(WorkspaceRunPackageSelection),
    SetWorkspaceSimulationMode(SimulationMode),
    SyncEntitlement,
    RefreshOfflineLease,
}

impl StudioAppCommand {
    pub fn run_workspace(command: WorkspaceRunCommand) -> Self {
        Self::RunWorkspace(command)
    }

    pub fn resume_workspace(selection: WorkspaceRunPackageSelection) -> Self {
        Self::ResumeWorkspace(selection)
    }

    pub fn set_workspace_simulation_mode(mode: SimulationMode) -> Self {
        Self::SetWorkspaceSimulationMode(mode)
    }

    pub fn sync_entitlement() -> Self {
        Self::SyncEntitlement
    }

    pub fn refresh_offline_lease() -> Self {
        Self::RefreshOfflineLease
    }

    pub fn execution_boundary(&self) -> StudioAppExecutionBoundary {
        match self {
            Self::RunWorkspace(_) | Self::ResumeWorkspace(_) => {
                StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
            }
            Self::SetWorkspaceSimulationMode(_) => {
                StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceControl)
            }
            Self::SyncEntitlement | Self::RefreshOfflineLease => {
                StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppExecutionLane {
    WorkspaceSolve,
    WorkspaceControl,
    EntitlementControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppExecutionBoundary {
    Inline(StudioAppExecutionLane),
}

#[derive(Debug, Clone, Copy)]
pub struct StudioAppAuthCacheContext<'a> {
    pub cache_root: &'a Path,
    pub auth_cache_index: &'a StoredAuthCacheIndex,
}

impl<'a> StudioAppAuthCacheContext<'a> {
    pub fn new(cache_root: &'a Path, auth_cache_index: &'a StoredAuthCacheIndex) -> Self {
        Self {
            cache_root,
            auth_cache_index,
        }
    }
}

#[derive(Debug)]
pub struct StudioAppMutableAuthCacheContext<'a> {
    pub cache_root: &'a Path,
    pub auth_cache_index: &'a mut StoredAuthCacheIndex,
}

impl<'a> StudioAppMutableAuthCacheContext<'a> {
    pub fn new(cache_root: &'a Path, auth_cache_index: &'a mut StoredAuthCacheIndex) -> Self {
        Self {
            cache_root,
            auth_cache_index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWorkspaceRunDispatch {
    pub package_id: Option<String>,
    pub outcome: StudioWorkspaceRunOutcome,
    pub simulation_mode: SimulationMode,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub run_status: RunStatus,
    pub log_entry_count: usize,
    pub latest_log_entry: Option<AppLogEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioWorkspaceRunBlockedReason {
    CachedPackageMissing,
    ExplicitPackageSelectionRequired,
    EntitlementMismatch,
    InvalidSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWorkspaceRunBlocked {
    pub reason: StudioWorkspaceRunBlockedReason,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioWorkspaceRunFailedReason {
    LocalCacheUnavailable,
    SolveFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWorkspaceRunFailed {
    pub reason: StudioWorkspaceRunFailedReason,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWorkspaceRunOutcome {
    Started(StudioSolveRequest),
    Skipped(WorkspaceSolveSkipReason),
    Blocked(StudioWorkspaceRunBlocked),
    Failed(StudioWorkspaceRunFailed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWorkspaceModeDispatch {
    pub simulation_mode: SimulationMode,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub run_status: RunStatus,
    pub log_entry_count: usize,
    pub latest_log_entry: Option<AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppResultDispatch {
    WorkspaceRun(StudioWorkspaceRunDispatch),
    WorkspaceMode(StudioWorkspaceModeDispatch),
    Entitlement(crate::StudioEntitlementActionOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppCommandOutcome {
    pub boundary: StudioAppExecutionBoundary,
    pub dispatch: StudioAppResultDispatch,
}

#[derive(Debug, Clone, Default)]
pub struct StudioAppFacade {
    solve_service: WorkspaceSolveService,
}

impl StudioAppFacade {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_with_auth_cache(
        &self,
        app_state: &mut AppState,
        context: &StudioAppAuthCacheContext<'_>,
        command: &StudioAppCommand,
    ) -> RfResult<StudioAppCommandOutcome> {
        let boundary = command.execution_boundary();
        let dispatch = match command {
            StudioAppCommand::RunWorkspace(run_command) => StudioAppResultDispatch::WorkspaceRun(
                self.run_workspace_from_auth_cache(app_state, context, run_command)?,
            ),
            StudioAppCommand::ResumeWorkspace(selection) => StudioAppResultDispatch::WorkspaceRun(
                self.resume_workspace_from_auth_cache(app_state, context, selection)?,
            ),
            StudioAppCommand::SetWorkspaceSimulationMode(mode) => {
                StudioAppResultDispatch::WorkspaceMode(
                    self.set_workspace_simulation_mode(app_state, *mode),
                )
            }
            StudioAppCommand::SyncEntitlement | StudioAppCommand::RefreshOfflineLease => {
                return Err(RfError::invalid_input(
                    "control plane entitlement commands require mutable auth cache context and access token",
                ));
            }
        };

        Ok(StudioAppCommandOutcome { boundary, dispatch })
    }

    pub fn execute_with_control_plane<Client>(
        &self,
        app_state: &mut AppState,
        context: &mut StudioAppMutableAuthCacheContext<'_>,
        control_plane_client: &Client,
        access_token: &str,
        command: &StudioAppCommand,
    ) -> RfResult<StudioAppCommandOutcome>
    where
        Client: crate::RadishFlowControlPlaneClient,
    {
        let boundary = command.execution_boundary();
        let dispatch = match command {
            StudioAppCommand::RunWorkspace(run_command) => {
                let readonly_context =
                    StudioAppAuthCacheContext::new(context.cache_root, &*context.auth_cache_index);
                StudioAppResultDispatch::WorkspaceRun(self.run_workspace_from_auth_cache(
                    app_state,
                    &readonly_context,
                    run_command,
                )?)
            }
            StudioAppCommand::ResumeWorkspace(selection) => {
                let readonly_context =
                    StudioAppAuthCacheContext::new(context.cache_root, &*context.auth_cache_index);
                StudioAppResultDispatch::WorkspaceRun(self.resume_workspace_from_auth_cache(
                    app_state,
                    &readonly_context,
                    selection,
                )?)
            }
            StudioAppCommand::SetWorkspaceSimulationMode(mode) => {
                StudioAppResultDispatch::WorkspaceMode(
                    self.set_workspace_simulation_mode(app_state, *mode),
                )
            }
            StudioAppCommand::SyncEntitlement => {
                StudioAppResultDispatch::Entitlement(crate::sync_entitlement_with_control_plane(
                    control_plane_client,
                    app_state,
                    access_token,
                ))
            }
            StudioAppCommand::RefreshOfflineLease => StudioAppResultDispatch::Entitlement(
                crate::refresh_offline_lease_with_control_plane(
                    control_plane_client,
                    app_state,
                    context.cache_root,
                    context.auth_cache_index,
                    access_token,
                ),
            ),
        };

        Ok(StudioAppCommandOutcome { boundary, dispatch })
    }

    pub fn run_workspace_from_auth_cache(
        &self,
        app_state: &mut AppState,
        context: &StudioAppAuthCacheContext<'_>,
        command: &WorkspaceRunCommand,
    ) -> RfResult<StudioWorkspaceRunDispatch> {
        if let Some(skip_reason) = self.solve_service.skip_reason(app_state, command.trigger) {
            let outcome = StudioWorkspaceRunOutcome::Skipped(skip_reason);
            record_workspace_run_outcome(app_state, &outcome);
            return Ok(map_workspace_run_dispatch(app_state, None, outcome));
        }

        let package_id = match resolve_workspace_run_package_id(
            app_state,
            context.auth_cache_index,
            &command.package,
        ) {
            Ok(package_id) => package_id,
            Err(error) => {
                let blocked = map_workspace_run_blocked(&error);
                let outcome = StudioWorkspaceRunOutcome::Blocked(blocked);
                record_workspace_run_outcome(app_state, &outcome);
                return Ok(map_workspace_run_dispatch(app_state, None, outcome));
            }
        };

        match self.solve_service.dispatch_from_auth_cache(
            app_state,
            context.cache_root,
            context.auth_cache_index,
            package_id.clone(),
            command.trigger,
        ) {
            Ok(dispatch) => {
                let outcome = map_workspace_solve_dispatch(dispatch);
                record_workspace_run_outcome(app_state, &outcome);
                Ok(map_workspace_run_dispatch(
                    app_state,
                    Some(package_id),
                    outcome,
                ))
            }
            Err(error) => {
                let failed = map_workspace_run_failed(app_state, &package_id, &error);
                let outcome = StudioWorkspaceRunOutcome::Failed(failed);
                record_workspace_run_outcome(app_state, &outcome);
                Ok(map_workspace_run_dispatch(
                    app_state,
                    Some(package_id),
                    outcome,
                ))
            }
        }
    }

    pub fn resume_workspace_from_auth_cache(
        &self,
        app_state: &mut AppState,
        context: &StudioAppAuthCacheContext<'_>,
        selection: &WorkspaceRunPackageSelection,
    ) -> RfResult<StudioWorkspaceRunDispatch> {
        app_state.set_simulation_mode(SimulationMode::Active);
        app_state.push_log(AppLogLevel::Info, "Activated workspace simulation mode");

        self.run_workspace_from_auth_cache(
            app_state,
            context,
            &WorkspaceRunCommand::new(WorkspaceSolveTrigger::Automatic, selection.clone()),
        )
    }

    pub fn set_workspace_simulation_mode(
        &self,
        app_state: &mut AppState,
        mode: SimulationMode,
    ) -> StudioWorkspaceModeDispatch {
        app_state.set_simulation_mode(mode);
        app_state.push_log(
            AppLogLevel::Info,
            format!(
                "Set workspace simulation mode to {}",
                describe_simulation_mode(mode)
            ),
        );
        map_workspace_mode_dispatch(app_state)
    }
}

fn record_workspace_run_outcome(app_state: &mut AppState, outcome: &StudioWorkspaceRunOutcome) {
    match outcome {
        StudioWorkspaceRunOutcome::Started(_) => {}
        StudioWorkspaceRunOutcome::Skipped(reason) => {
            push_log_if_needed(
                app_state,
                AppLogLevel::Info,
                &format!(
                    "Skipped workspace run because {}",
                    describe_workspace_skip_reason(*reason)
                ),
            );
        }
        StudioWorkspaceRunOutcome::Blocked(blocked) => {
            push_log_if_needed(
                app_state,
                AppLogLevel::Warning,
                &format!("Blocked workspace run because {}", blocked.message),
            );
        }
        StudioWorkspaceRunOutcome::Failed(failed) => {
            if !matches!(app_state.workspace.solve_session.status, RunStatus::Error) {
                let revision = app_state.workspace.document.revision;
                let context = SolveFailureContext::from_message(&failed.message);
                let mut summary = DiagnosticSummary::new(
                    revision,
                    DiagnosticSeverity::Error,
                    failed.message.clone(),
                );
                if let Some(primary_code) = context.primary_code {
                    summary = summary.with_primary_code(primary_code);
                }
                if !context.related_unit_ids.is_empty() {
                    summary = summary.with_related_unit_ids(context.related_unit_ids);
                }
                if !context.related_stream_ids.is_empty() {
                    summary = summary.with_related_stream_ids(context.related_stream_ids);
                }
                if !context.related_port_targets.is_empty() {
                    summary = summary.with_related_port_targets(context.related_port_targets);
                }
                app_state.record_failure(revision, RunStatus::Error, summary);
            }
            push_log_if_needed(app_state, AppLogLevel::Error, &failed.message);
        }
    }
}

fn push_log_if_needed(app_state: &mut AppState, level: AppLogLevel, message: &str) {
    let duplicated = app_state
        .log_feed
        .entries
        .back()
        .map(|entry| entry.level == level && entry.message == message)
        .unwrap_or(false);
    if !duplicated {
        app_state.push_log(level, message.to_string());
    }
}

fn map_workspace_solve_dispatch(dispatch: WorkspaceSolveDispatch) -> StudioWorkspaceRunOutcome {
    match dispatch {
        WorkspaceSolveDispatch::Started(request) => StudioWorkspaceRunOutcome::Started(request),
        WorkspaceSolveDispatch::Skipped(reason) => StudioWorkspaceRunOutcome::Skipped(reason),
    }
}

fn map_workspace_run_blocked(error: &RfError) -> StudioWorkspaceRunBlocked {
    let reason = match error.context().diagnostic_code() {
        Some(WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING) => {
            StudioWorkspaceRunBlockedReason::CachedPackageMissing
        }
        Some(WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED) => {
            StudioWorkspaceRunBlockedReason::ExplicitPackageSelectionRequired
        }
        Some(WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH) => {
            StudioWorkspaceRunBlockedReason::EntitlementMismatch
        }
        Some(WORKSPACE_RUN_DIAGNOSTIC_INVALID_SELECTION) => {
            StudioWorkspaceRunBlockedReason::InvalidSelection
        }
        _ => match error.code() {
            ErrorCode::MissingEntity => StudioWorkspaceRunBlockedReason::CachedPackageMissing,
            _ => StudioWorkspaceRunBlockedReason::InvalidSelection,
        },
    };

    StudioWorkspaceRunBlocked {
        reason,
        message: error.message().to_string(),
    }
}

fn map_workspace_run_failed(
    app_state: &AppState,
    package_id: &str,
    error: &RfError,
) -> StudioWorkspaceRunFailed {
    let latest_solver_error = app_state
        .log_feed
        .entries
        .back()
        .filter(|entry| entry.level == AppLogLevel::Error)
        .map(|entry| entry.message.clone());

    let reason = match error.context().diagnostic_code() {
        Some(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE) => {
            StudioWorkspaceRunFailedReason::LocalCacheUnavailable
        }
        _ if matches!(app_state.workspace.solve_session.status, RunStatus::Error) => {
            StudioWorkspaceRunFailedReason::SolveFailed
        }
        _ => StudioWorkspaceRunFailedReason::LocalCacheUnavailable,
    };

    let message = match reason {
        StudioWorkspaceRunFailedReason::SolveFailed => latest_solver_error.unwrap_or_else(|| {
            format!(
                "workspace run with property package `{package_id}` failed: {}",
                error.message()
            )
        }),
        StudioWorkspaceRunFailedReason::LocalCacheUnavailable => format!(
            "failed to prepare local property package cache for `{package_id}`: {}",
            error.message()
        ),
    };

    StudioWorkspaceRunFailed { reason, message }
}

fn describe_workspace_skip_reason(reason: WorkspaceSolveSkipReason) -> &'static str {
    match reason {
        WorkspaceSolveSkipReason::HoldMode => "simulation mode is Hold",
        WorkspaceSolveSkipReason::NoPendingRequest => "there is no pending solve request",
    }
}

fn map_workspace_run_dispatch(
    app_state: &AppState,
    package_id: Option<String>,
    outcome: StudioWorkspaceRunOutcome,
) -> StudioWorkspaceRunDispatch {
    StudioWorkspaceRunDispatch {
        package_id,
        outcome,
        simulation_mode: app_state.workspace.solve_session.mode,
        pending_reason: app_state.workspace.solve_session.pending_reason,
        latest_snapshot_id: latest_snapshot_id(&app_state.workspace)
            .map(|snapshot_id| snapshot_id.as_str().to_string()),
        latest_snapshot_summary: latest_snapshot(&app_state.workspace)
            .map(|snapshot| snapshot.summary.primary_message.clone()),
        run_status: app_state.workspace.solve_session.status,
        log_entry_count: app_state.log_feed.entries.len(),
        latest_log_entry: app_state.log_feed.entries.back().cloned(),
    }
}

fn map_workspace_mode_dispatch(app_state: &AppState) -> StudioWorkspaceModeDispatch {
    StudioWorkspaceModeDispatch {
        simulation_mode: app_state.workspace.solve_session.mode,
        pending_reason: app_state.workspace.solve_session.pending_reason,
        latest_snapshot_id: latest_snapshot_id(&app_state.workspace)
            .map(|snapshot_id| snapshot_id.as_str().to_string()),
        latest_snapshot_summary: latest_snapshot(&app_state.workspace)
            .map(|snapshot| snapshot.summary.primary_message.clone()),
        run_status: app_state.workspace.solve_session.status,
        log_entry_count: app_state.log_feed.entries.len(),
        latest_log_entry: app_state.log_feed.entries.back().cloned(),
    }
}

fn describe_simulation_mode(mode: SimulationMode) -> &'static str {
    match mode {
        SimulationMode::Active => "Active",
        SimulationMode::Hold => "Hold",
    }
}

#[cfg(test)]
mod tests;
