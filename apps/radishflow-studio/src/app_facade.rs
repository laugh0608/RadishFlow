use std::path::Path;

use rf_solver::SolveFailureContext;
use rf_store::StoredAuthCacheIndex;
use rf_types::{ErrorCode, RfError, RfResult};
use rf_ui::{
    AppLogEntry, AppLogLevel, AppState, DiagnosticSeverity, DiagnosticSummary, RunStatus,
    SimulationMode, SolvePendingReason, latest_snapshot_id,
};

use crate::{
    StudioSolveRequest, WorkspaceRunCommand, WorkspaceRunPackageSelection, WorkspaceSolveDispatch,
    WorkspaceSolveService, WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    resolve_workspace_run_package_id,
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
    let reason = match error.code() {
        ErrorCode::MissingEntity => StudioWorkspaceRunBlockedReason::CachedPackageMissing,
        ErrorCode::InvalidInput => {
            if error.message().contains("explicit package selection") {
                StudioWorkspaceRunBlockedReason::ExplicitPackageSelectionRequired
            } else if error.message().contains("entitlement manifests")
                || error
                    .message()
                    .contains("matches current entitlement manifests")
            {
                StudioWorkspaceRunBlockedReason::EntitlementMismatch
            } else {
                StudioWorkspaceRunBlockedReason::InvalidSelection
            }
        }
        _ => StudioWorkspaceRunBlockedReason::InvalidSelection,
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

    if matches!(app_state.workspace.solve_session.status, RunStatus::Error) {
        return StudioWorkspaceRunFailed {
            reason: StudioWorkspaceRunFailedReason::SolveFailed,
            message: latest_solver_error.unwrap_or_else(|| {
                format!(
                    "workspace run with property package `{package_id}` failed: {}",
                    error.message()
                )
            }),
        };
    }

    StudioWorkspaceRunFailed {
        reason: StudioWorkspaceRunFailedReason::LocalCacheUnavailable,
        message: format!(
            "failed to prepare local property package cache for `{package_id}`: {}",
            error.message()
        ),
    }
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
        latest_snapshot_summary: app_state
            .workspace
            .snapshot_history
            .back()
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
        latest_snapshot_summary: app_state
            .workspace
            .snapshot_history
            .back()
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
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{
        StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
        StoredEntitlementCache, StoredPropertyPackageManifest, StoredPropertyPackagePayload,
        StoredPropertyPackageRecord, StoredPropertyPackageSource, StoredThermoComponent,
        parse_project_file_json, property_package_payload_integrity,
        write_property_package_manifest, write_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{
        AppState, AuthenticatedUser, DocumentMetadata, EntitlementSnapshot, FlowsheetDocument,
        OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant,
        PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
        PropertyPackageSource, RunStatus, SecureCredentialHandle, SimulationMode,
        SolvePendingReason, TokenLease,
    };

    use super::{
        StudioAppAuthCacheContext, StudioAppCommand, StudioAppExecutionBoundary,
        StudioAppExecutionLane, StudioAppFacade, StudioAppMutableAuthCacheContext,
        StudioAppResultDispatch, StudioWorkspaceRunBlocked, StudioWorkspaceRunBlockedReason,
        StudioWorkspaceRunFailedReason, StudioWorkspaceRunOutcome,
    };
    use crate::{
        RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse,
        StudioEntitlementAction, StudioEntitlementFailureReason, StudioEntitlementOutcome,
        WorkspaceRunCommand, WorkspaceRunPackageSelection,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
        FlowsheetDocument::new(flowsheet, metadata)
    }

    fn sample_auth_cache_index(package_ids: &[&str]) -> StoredAuthCacheIndex {
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        index.property_packages = package_ids
            .iter()
            .map(|package_id| {
                let mut record = StoredPropertyPackageRecord::new(
                    *package_id,
                    "2026.03.1",
                    StoredPropertyPackageSource::RemoteDerivedPackage,
                    "sha256:test",
                    128,
                    timestamp(20),
                );
                record.expires_at = Some(timestamp(9_999_999_999));
                record
            })
            .collect();
        index
    }

    fn sample_entitled_auth_cache_index(package_ids: &[&str]) -> StoredAuthCacheIndex {
        let mut index = sample_auth_cache_index(package_ids);
        index.entitlement = Some(StoredEntitlementCache {
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            synced_at: timestamp(100),
            issued_at: timestamp(90),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(700)),
            feature_keys: std::collections::BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: package_ids.iter().map(|item| item.to_string()).collect(),
        });
        index
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    fn write_cached_package(
        cache_root: &Path,
        auth_cache_index: &mut StoredAuthCacheIndex,
        package_id: &str,
    ) {
        let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(StoredAntoineCoefficients::new(
            ((2.0_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));
        let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(StoredAntoineCoefficients::new(
            ((0.5_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let payload =
            StoredPropertyPackagePayload::new(package_id, "2026.03.1", vec![first, second]);
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
        let mut manifest = StoredPropertyPackageManifest::new(
            package_id,
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![
                ComponentId::new("component-a"),
                ComponentId::new("component-b"),
            ],
        );
        manifest.hash = integrity.hash.clone();
        manifest.size_bytes = integrity.size_bytes;
        manifest.expires_at = expires_at;
        let mut record = StoredPropertyPackageRecord::new(
            &manifest.package_id,
            &manifest.version,
            StoredPropertyPackageSource::RemoteDerivedPackage,
            manifest.hash.clone(),
            manifest.size_bytes,
            timestamp(60),
        );
        record.expires_at = expires_at;

        write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(cache_root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        auth_cache_index.property_packages.push(record);
    }

    fn sample_snapshot() -> EntitlementSnapshot {
        EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            features: std::collections::BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: std::collections::BTreeSet::from([
                "binary-hydrocarbon-lite-v1".to_string()
            ]),
        }
    }

    fn sample_manifest() -> PropertyPackageManifest {
        let mut manifest = PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.hash = "sha256:pkg-1".to_string();
        manifest.size_bytes = 1024;
        manifest.expires_at = Some(timestamp(900));
        manifest
    }

    fn sample_manifest_list() -> PropertyPackageManifestList {
        PropertyPackageManifestList::new(timestamp(205), vec![sample_manifest()])
    }

    fn sample_offline_refresh_response() -> OfflineLeaseRefreshResponse {
        OfflineLeaseRefreshResponse {
            refreshed_at: timestamp(210),
            snapshot: sample_snapshot(),
            manifest_list: sample_manifest_list(),
        }
    }

    fn complete_login(app_state: &mut AppState) {
        app_state.complete_login(
            "https://id.radish.local",
            AuthenticatedUser::new("user-123", "luobo"),
            TokenLease::new(
                timestamp(400),
                SecureCredentialHandle::new("radishflow-studio", "user-123-primary"),
            ),
            timestamp(120),
        );
    }

    #[test]
    fn facade_runs_workspace_command_from_auth_cache() {
        let cache_root = unique_temp_path("app-facade-run");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        write_cached_package(
            &cache_root,
            &mut auth_cache_index,
            "binary-hydrocarbon-lite-v1",
        );
        let facade = StudioAppFacade::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-app-facade", "App Facade Demo", timestamp(70)),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let command = StudioAppCommand::run_workspace(WorkspaceRunCommand::manual(
            "binary-hydrocarbon-lite-v1",
        ));

        let outcome = facade
            .execute_with_auth_cache(&mut app_state, &context, &command)
            .expect("expected app facade run");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
        );
        let dispatch = match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            dispatch.package_id,
            Some("binary-hydrocarbon-lite-v1".to_string())
        );
        assert_eq!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Started(crate::StudioSolveRequest::new(
                "binary-hydrocarbon-lite-v1",
                "doc-app-facade-rev-0-seq-1",
                1,
            ))
        );
        assert_eq!(
            dispatch.latest_snapshot_id.as_deref(),
            Some("doc-app-facade-rev-0-seq-1")
        );
        assert_eq!(dispatch.simulation_mode, SimulationMode::Hold);
        assert_eq!(dispatch.pending_reason, None);
        assert_eq!(
            dispatch.latest_snapshot_summary.as_deref(),
            Some(
                "solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)"
            )
        );
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| entry.message.as_str()),
            Some(
                "Solved document revision 0 with property package `binary-hydrocarbon-lite-v1` into snapshot `doc-app-facade-rev-0-seq-1`"
            )
        );
        assert_eq!(app_state.log_feed.entries.len(), 1);

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn facade_skips_automatic_workspace_command_before_package_resolution() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let command = StudioAppCommand::run_workspace(WorkspaceRunCommand::new(
            crate::WorkspaceSolveTrigger::Automatic,
            WorkspaceRunPackageSelection::Preferred,
        ));

        let outcome = facade
            .execute_with_auth_cache(&mut app_state, &context, &command)
            .expect("expected skip outcome");

        let dispatch = match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(dispatch.package_id, None);
        assert_eq!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Skipped(crate::WorkspaceSolveSkipReason::HoldMode)
        );
        assert_eq!(dispatch.simulation_mode, SimulationMode::Hold);
        assert_eq!(
            dispatch.pending_reason,
            Some(SolvePendingReason::SnapshotMissing)
        );
        assert_eq!(dispatch.latest_snapshot_id, None);
        assert_eq!(dispatch.latest_snapshot_summary, None);
        assert_eq!(dispatch.run_status, RunStatus::Idle);
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| entry.message.as_str()),
            Some("Skipped workspace run because simulation mode is Hold")
        );
        assert_eq!(app_state.log_feed.entries.len(), 1);
        assert_eq!(
            app_state.log_feed.entries[0].message,
            "Skipped workspace run because simulation mode is Hold"
        );
    }

    #[test]
    fn facade_sets_workspace_simulation_mode_without_running_solver() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let command = StudioAppCommand::set_workspace_simulation_mode(SimulationMode::Active);

        let outcome = facade
            .execute_with_auth_cache(&mut app_state, &context, &command)
            .expect("expected mode dispatch");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceControl)
        );
        let dispatch = match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceMode(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected workspace mode dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace mode dispatch"),
        };
        assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
        assert_eq!(
            dispatch.pending_reason,
            Some(SolvePendingReason::ModeActivated)
        );
        assert_eq!(dispatch.run_status, RunStatus::Idle);
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| entry.message.as_str()),
            Some("Set workspace simulation mode to Active")
        );
    }

    #[test]
    fn facade_resumes_workspace_from_hold_and_runs_automatic_dispatch() {
        let cache_root = unique_temp_path("app-facade-resume");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        write_cached_package(
            &cache_root,
            &mut auth_cache_index,
            "binary-hydrocarbon-lite-v1",
        );
        let facade = StudioAppFacade::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-app-resume", "App Resume Demo", timestamp(70)),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let command = StudioAppCommand::resume_workspace(WorkspaceRunPackageSelection::Preferred);

        let outcome = facade
            .execute_with_auth_cache(&mut app_state, &context, &command)
            .expect("expected app facade resume");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
        );
        let dispatch = match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Started(crate::StudioSolveRequest::new(
                "binary-hydrocarbon-lite-v1",
                "doc-app-resume-rev-0-seq-1",
                1,
            ))
        );
        assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
        assert_eq!(dispatch.pending_reason, None);
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(dispatch.log_entry_count, 2);
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| entry.message.as_str()),
            Some(
                "Solved document revision 0 with property package `binary-hydrocarbon-lite-v1` into snapshot `doc-app-resume-rev-0-seq-1`"
            )
        );

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn facade_returns_blocked_dispatch_when_preferred_package_is_ambiguous() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let dispatch = facade
            .run_workspace_from_auth_cache(
                &mut app_state,
                &context,
                &WorkspaceRunCommand::new(
                    crate::WorkspaceSolveTrigger::Manual,
                    WorkspaceRunPackageSelection::Preferred,
                ),
            )
            .expect("expected blocked dispatch");

        assert_eq!(dispatch.package_id, None);
        assert_eq!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Blocked(StudioWorkspaceRunBlocked {
                reason: StudioWorkspaceRunBlockedReason::ExplicitPackageSelectionRequired,
                message:
                    "multiple cached property packages are available; explicit package selection is required"
                        .to_string(),
            })
        );
        assert_eq!(dispatch.run_status, RunStatus::Idle);
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| (entry.level, entry.message.as_str())),
            Some((
                rf_ui::AppLogLevel::Warning,
                "Blocked workspace run because multiple cached property packages are available; explicit package selection is required",
            ))
        );
    }

    #[test]
    fn facade_returns_failed_dispatch_when_local_cache_files_are_unavailable() {
        let cache_root = unique_temp_path("app-facade-failed");
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let dispatch = facade
            .run_workspace_from_auth_cache(
                &mut app_state,
                &context,
                &WorkspaceRunCommand::manual("pkg-1"),
            )
            .expect("expected failed dispatch");

        assert_eq!(dispatch.package_id, Some("pkg-1".to_string()));
        match dispatch.outcome {
            StudioWorkspaceRunOutcome::Failed(failed) => {
                assert_eq!(
                    failed.reason,
                    StudioWorkspaceRunFailedReason::LocalCacheUnavailable
                );
                assert!(
                    failed
                        .message
                        .contains("failed to prepare local property package cache")
                );
            }
            other => panic!("expected failed dispatch, got {other:?}"),
        }
        assert_eq!(dispatch.run_status, RunStatus::Error);
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .and_then(|summary| summary.primary_code.as_deref()),
            None
        );
        assert_eq!(
            dispatch.latest_log_entry.as_ref().map(|entry| entry.level),
            Some(rf_ui::AppLogLevel::Error)
        );

        std::fs::remove_dir_all(cache_root).ok();
    }

    #[test]
    fn facade_executes_entitlement_sync_through_control_plane_context() {
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index =
            sample_entitled_auth_cache_index(&["binary-hydrocarbon-lite-v1"]);
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let client = ScriptedControlPlaneClient::success();

        let outcome = facade
            .execute_with_control_plane(
                &mut app_state,
                &mut context,
                &client,
                "access-token",
                &StudioAppCommand::sync_entitlement(),
            )
            .expect("expected entitlement sync dispatch");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
        );
        match outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
                assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
            }
            other => panic!("expected entitlement dispatch, got {other:?}"),
        }
        assert!(
            app_state
                .entitlement
                .is_package_allowed("binary-hydrocarbon-lite-v1")
        );
    }

    #[test]
    fn facade_executes_offline_refresh_through_control_plane_context() {
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(), vec![sample_manifest()], timestamp(150));
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index =
            sample_entitled_auth_cache_index(&["binary-hydrocarbon-lite-v1"]);
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let client = ScriptedControlPlaneClient::offline_refresh_failure(
            RadishFlowControlPlaneClientError::unauthorized("token expired"),
        );

        let outcome = facade
            .execute_with_control_plane(
                &mut app_state,
                &mut context,
                &client,
                "access-token",
                &StudioAppCommand::refresh_offline_lease(),
            )
            .expect("expected offline refresh dispatch");

        match outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => match dispatch.outcome {
                StudioEntitlementOutcome::Failed(failure) => {
                    assert_eq!(
                        failure.reason,
                        StudioEntitlementFailureReason::AuthenticationRequired
                    );
                }
                other => panic!("expected failed entitlement outcome, got {other:?}"),
            },
            other => panic!("expected entitlement dispatch, got {other:?}"),
        }
        assert_eq!(
            app_state.auth_session.status,
            rf_ui::AuthSessionStatus::Error
        );
    }

    #[derive(Debug, Clone)]
    struct ScriptedControlPlaneClient {
        entitlement_response: Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        >,
        manifest_response: Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        >,
        refresh_response: Result<
            RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
            RadishFlowControlPlaneClientError,
        >,
    }

    impl ScriptedControlPlaneClient {
        fn success() -> Self {
            Self {
                entitlement_response: Ok(RadishFlowControlPlaneResponse::new(
                    sample_snapshot(),
                    timestamp(200),
                )),
                manifest_response: Ok(RadishFlowControlPlaneResponse::new(
                    sample_manifest_list(),
                    timestamp(210),
                )),
                refresh_response: Ok(RadishFlowControlPlaneResponse::new(
                    sample_offline_refresh_response(),
                    timestamp(220),
                )),
            }
        }

        fn offline_refresh_failure(error: RadishFlowControlPlaneClientError) -> Self {
            Self {
                refresh_response: Err(error),
                ..Self::success()
            }
        }
    }

    impl RadishFlowControlPlaneClient for ScriptedControlPlaneClient {
        fn fetch_entitlement_snapshot(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        > {
            self.entitlement_response.clone()
        }

        fn fetch_property_package_manifest_list(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        > {
            self.manifest_response.clone()
        }

        fn request_property_package_lease(
            &self,
            _access_token: &str,
            _package_id: &str,
            _request: &PropertyPackageLeaseRequest,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
            RadishFlowControlPlaneClientError,
        > {
            Err(RadishFlowControlPlaneClientError::new(
                RadishFlowControlPlaneClientErrorKind::OtherPermanent,
                "lease request is not used in app facade tests",
            ))
        }

        fn refresh_offline_leases(
            &self,
            _access_token: &str,
            _request: &OfflineLeaseRefreshRequest,
        ) -> Result<
            RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
            RadishFlowControlPlaneClientError,
        > {
            self.refresh_response.clone()
        }
    }
}
