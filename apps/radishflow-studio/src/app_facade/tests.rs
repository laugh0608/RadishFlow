use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_model::Flowsheet;
use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageRecord, StoredPropertyPackageSource, parse_project_file_json,
};
use rf_ui::{
    AppState, AuthenticatedUser, DiagnosticSeverity, DiagnosticSummary, DocumentMetadata,
    EntitlementSnapshot, FlowsheetDocument, OfflineLeaseRefreshRequest,
    OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
    PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource, RunStatus,
    SecureCredentialHandle, SimulationMode, SolvePendingReason, TokenLease,
};

use super::{
    StudioAppAuthCacheContext, StudioAppCommand, StudioAppExecutionBoundary,
    StudioAppExecutionLane, StudioAppFacade, StudioAppMutableAuthCacheContext,
    StudioAppResultDispatch, StudioWorkspaceRunBlocked, StudioWorkspaceRunBlockedReason,
    StudioWorkspaceRunFailedReason, StudioWorkspaceRunOutcome,
};
use crate::{
    RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
    RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, StudioEntitlementAction,
    StudioEntitlementFailureReason, StudioEntitlementOutcome, WorkspaceRunCommand,
    WorkspaceRunPackageSelection,
    test_support::write_official_binary_hydrocarbon_cached_package as write_shared_official_binary_hydrocarbon_cached_package,
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

fn write_official_binary_hydrocarbon_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
) {
    write_shared_official_binary_hydrocarbon_cached_package(
        cache_root,
        auth_cache_index,
        package_id,
        timestamp(60),
        Some(SystemTime::now() + Duration::from_secs(3_600)),
    );
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
    write_official_binary_hydrocarbon_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let project = parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("doc-app-facade", "App Facade Demo", timestamp(70)),
    ));
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
    let command =
        StudioAppCommand::run_workspace(WorkspaceRunCommand::manual("binary-hydrocarbon-lite-v1"));

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
        Some("solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)")
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
fn facade_mode_dispatch_hides_stale_snapshot_after_document_revision_advances() {
    let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
    let facade = StudioAppFacade::new();
    let mut app_state = AppState::new(sample_document());
    app_state.store_snapshot(rf_ui::SolveSnapshot::new(
        "snapshot-stale",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
    ));
    app_state.commit_document_change(
        rf_ui::DocumentCommand::MoveUnit {
            unit_id: "heater-1".into(),
            position: rf_ui::CanvasPoint::new(80.0, 40.0),
        },
        Flowsheet::new("demo-updated"),
        timestamp(31),
    );
    let cache_root = PathBuf::from("D:\\cache-root");
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
    let command = StudioAppCommand::set_workspace_simulation_mode(SimulationMode::Active);

    let outcome = facade
        .execute_with_auth_cache(&mut app_state, &context, &command)
        .expect("expected mode dispatch");

    let dispatch = match outcome.dispatch {
        StudioAppResultDispatch::WorkspaceMode(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected workspace mode dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace mode dispatch"),
    };

    assert_eq!(dispatch.latest_snapshot_id, None);
    assert_eq!(dispatch.latest_snapshot_summary, None);
    assert_eq!(dispatch.run_status, RunStatus::Dirty);
    assert_eq!(
        dispatch.pending_reason,
        Some(SolvePendingReason::ModeActivated)
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
    write_official_binary_hydrocarbon_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let project = parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
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
fn facade_returns_blocked_dispatch_when_entitlement_mismatch_is_structured() {
    let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
    let facade = StudioAppFacade::new();
    let mut app_state = AppState::new(sample_document());
    app_state.update_entitlement(sample_snapshot(), vec![sample_manifest()], timestamp(140));
    let cache_root = PathBuf::from("D:\\cache-root");
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    let dispatch = facade
        .run_workspace_from_auth_cache(
            &mut app_state,
            &context,
            &WorkspaceRunCommand::manual("pkg-2"),
        )
        .expect("expected blocked dispatch");

    assert_eq!(dispatch.package_id, None);
    assert_eq!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Blocked(StudioWorkspaceRunBlocked {
            reason: StudioWorkspaceRunBlockedReason::EntitlementMismatch,
            message: "workspace run package `pkg-2` is not present in entitlement manifests"
                .to_string(),
        })
    );
    assert_eq!(
        dispatch
            .latest_log_entry
            .as_ref()
            .map(|entry| (entry.level, entry.message.as_str())),
        Some((
            rf_ui::AppLogLevel::Warning,
            "Blocked workspace run because workspace run package `pkg-2` is not present in entitlement manifests",
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
    let mut auth_cache_index = sample_entitled_auth_cache_index(&["binary-hydrocarbon-lite-v1"]);
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
    let mut auth_cache_index = sample_entitled_auth_cache_index(&["binary-hydrocarbon-lite-v1"]);
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
