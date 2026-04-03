use rf_store::StoredAuthCacheIndex;
use rf_types::RfResult;
use rf_ui::{
    AppLogEntry, AppLogLevel, AppState, EntitlementNotice, EntitlementNoticeLevel,
    EntitlementSnapshot, EntitlementStatus, OfflineLeaseRefreshResponse,
    PropertyPackageManifestList,
};

use crate::{
    RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
    RadishFlowControlPlaneClientErrorKind, apply_offline_refresh_to_auth_cache,
    build_offline_refresh_request,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioEntitlementAction {
    SyncEntitlement,
    RefreshOfflineLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioEntitlementFailureReason {
    ConnectionUnavailable,
    ServiceUnavailable,
    AuthenticationRequired,
    AccessDenied,
    InvalidResponse,
    LocalStateInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioEntitlementFailure {
    pub reason: StudioEntitlementFailureReason,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioEntitlementOutcome {
    Synced,
    OfflineLeaseRefreshed,
    Failed(StudioEntitlementFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioEntitlementActionOutcome {
    pub action: StudioEntitlementAction,
    pub outcome: StudioEntitlementOutcome,
    pub entitlement_status: EntitlementStatus,
    pub last_synced_at: Option<std::time::SystemTime>,
    pub last_error: Option<String>,
    pub notice: Option<EntitlementNotice>,
    pub latest_log_entry: Option<AppLogEntry>,
}

pub fn sync_entitlement_with_control_plane<Client>(
    client: &Client,
    app_state: &mut AppState,
    access_token: &str,
) -> StudioEntitlementActionOutcome
where
    Client: RadishFlowControlPlaneClient,
{
    let previous_status = app_state.entitlement.status;
    let had_snapshot = app_state.entitlement.snapshot.is_some();
    app_state.entitlement.begin_sync();

    let outcome = match fetch_entitlement_sync_payload(client, access_token) {
        Ok((snapshot, manifests, synced_at)) => {
            app_state.update_entitlement(snapshot, manifests.packages, synced_at);
            let notice = EntitlementNotice::new(
                EntitlementNoticeLevel::Info,
                "Entitlement synced",
                "control plane entitlement snapshot and manifest list were refreshed",
            );
            app_state.entitlement.set_notice(notice.clone());
            push_log_if_needed(
                app_state,
                AppLogLevel::Info,
                "Synced entitlement snapshot and property package manifests from control plane",
            );
            StudioEntitlementOutcome::Synced
        }
        Err(error) => {
            let failure = map_client_error_to_failure(error);
            apply_entitlement_failure(
                app_state,
                previous_status,
                had_snapshot,
                &failure,
                StudioEntitlementAction::SyncEntitlement,
            );
            StudioEntitlementOutcome::Failed(failure)
        }
    };

    snapshot_entitlement_action_outcome(
        app_state,
        StudioEntitlementAction::SyncEntitlement,
        outcome,
    )
}

pub fn refresh_offline_lease_with_control_plane<Client>(
    client: &Client,
    app_state: &mut AppState,
    auth_cache_index: &mut StoredAuthCacheIndex,
    access_token: &str,
) -> StudioEntitlementActionOutcome
where
    Client: RadishFlowControlPlaneClient,
{
    let previous_status = app_state.entitlement.status;
    let had_snapshot = app_state.entitlement.snapshot.is_some();

    let outcome = match build_offline_refresh_request(auth_cache_index) {
        Ok(request) => match client.refresh_offline_leases(access_token, &request) {
            Ok(response) => match apply_offline_refresh_success(
                app_state,
                auth_cache_index,
                response.value,
            ) {
                Ok(()) => {
                    let notice = EntitlementNotice::new(
                        EntitlementNoticeLevel::Info,
                        "Offline lease refreshed",
                        "offline lease state and cached package permissions were refreshed",
                    );
                    app_state.entitlement.set_notice(notice);
                    push_log_if_needed(
                        app_state,
                        AppLogLevel::Info,
                        "Refreshed offline lease state from control plane",
                    );
                    StudioEntitlementOutcome::OfflineLeaseRefreshed
                }
                Err(error) => {
                    let failure = StudioEntitlementFailure {
                        reason: StudioEntitlementFailureReason::LocalStateInvalid,
                        message: error.message().to_string(),
                    };
                    apply_entitlement_failure(
                        app_state,
                        previous_status,
                        had_snapshot,
                        &failure,
                        StudioEntitlementAction::RefreshOfflineLease,
                    );
                    StudioEntitlementOutcome::Failed(failure)
                }
            },
            Err(error) => {
                let failure = map_client_error_to_failure(error);
                apply_entitlement_failure(
                    app_state,
                    previous_status,
                    had_snapshot,
                    &failure,
                    StudioEntitlementAction::RefreshOfflineLease,
                );
                StudioEntitlementOutcome::Failed(failure)
            }
        },
        Err(error) => {
            let failure = StudioEntitlementFailure {
                reason: StudioEntitlementFailureReason::LocalStateInvalid,
                message: error.message().to_string(),
            };
            apply_entitlement_failure(
                app_state,
                previous_status,
                had_snapshot,
                &failure,
                StudioEntitlementAction::RefreshOfflineLease,
            );
            StudioEntitlementOutcome::Failed(failure)
        }
    };

    snapshot_entitlement_action_outcome(
        app_state,
        StudioEntitlementAction::RefreshOfflineLease,
        outcome,
    )
}

fn fetch_entitlement_sync_payload<Client>(
    client: &Client,
    access_token: &str,
) -> Result<
    (
        EntitlementSnapshot,
        PropertyPackageManifestList,
        std::time::SystemTime,
    ),
    RadishFlowControlPlaneClientError,
>
where
    Client: RadishFlowControlPlaneClient,
{
    let snapshot = client.fetch_entitlement_snapshot(access_token)?;
    let manifest_list = client.fetch_property_package_manifest_list(access_token)?;

    validate_manifest_sync_consistency(&snapshot.value, &manifest_list.value)?;
    let synced_at = if snapshot.received_at >= manifest_list.received_at {
        snapshot.received_at
    } else {
        manifest_list.received_at
    };

    Ok((snapshot.value, manifest_list.value, synced_at))
}

fn validate_manifest_sync_consistency(
    snapshot: &EntitlementSnapshot,
    manifest_list: &PropertyPackageManifestList,
) -> Result<(), RadishFlowControlPlaneClientError> {
    let mut seen = std::collections::BTreeSet::new();
    for manifest in &manifest_list.packages {
        if !seen.insert(manifest.package_id.clone()) {
            return Err(RadishFlowControlPlaneClientError::invalid_response(format!(
                "control plane manifest list contains duplicate package `{}`",
                manifest.package_id
            )));
        }
        if !snapshot.allowed_package_ids.contains(&manifest.package_id) {
            return Err(RadishFlowControlPlaneClientError::invalid_response(format!(
                "control plane manifest list returned package `{}` outside allowedPackageIds",
                manifest.package_id
            )));
        }
    }

    Ok(())
}

fn apply_offline_refresh_success(
    app_state: &mut AppState,
    auth_cache_index: &mut StoredAuthCacheIndex,
    response: OfflineLeaseRefreshResponse,
) -> RfResult<()> {
    apply_offline_refresh_to_auth_cache(auth_cache_index, &response)?;
    app_state.entitlement.apply_offline_refresh(response);
    Ok(())
}

fn map_client_error_to_failure(
    error: RadishFlowControlPlaneClientError,
) -> StudioEntitlementFailure {
    let reason = match error.kind {
        RadishFlowControlPlaneClientErrorKind::Timeout
        | RadishFlowControlPlaneClientErrorKind::ConnectionUnavailable => {
            StudioEntitlementFailureReason::ConnectionUnavailable
        }
        RadishFlowControlPlaneClientErrorKind::RateLimited
        | RadishFlowControlPlaneClientErrorKind::ServiceUnavailable
        | RadishFlowControlPlaneClientErrorKind::OtherTransient => {
            StudioEntitlementFailureReason::ServiceUnavailable
        }
        RadishFlowControlPlaneClientErrorKind::Unauthorized => {
            StudioEntitlementFailureReason::AuthenticationRequired
        }
        RadishFlowControlPlaneClientErrorKind::Forbidden => {
            StudioEntitlementFailureReason::AccessDenied
        }
        RadishFlowControlPlaneClientErrorKind::NotFound
        | RadishFlowControlPlaneClientErrorKind::InvalidResponse
        | RadishFlowControlPlaneClientErrorKind::OtherPermanent => {
            StudioEntitlementFailureReason::InvalidResponse
        }
    };

    StudioEntitlementFailure {
        reason,
        message: error.message,
    }
}

fn apply_entitlement_failure(
    app_state: &mut AppState,
    previous_status: EntitlementStatus,
    had_snapshot: bool,
    failure: &StudioEntitlementFailure,
    action: StudioEntitlementAction,
) {
    let notice = notice_for_failure(failure);
    let (level, log_message) = log_payload_for_failure(action, failure);

    if had_snapshot {
        app_state.entitlement.status = previous_status;
        app_state
            .entitlement
            .record_nonblocking_error(log_message.clone());
    } else {
        app_state.entitlement.record_error(log_message.clone());
    }

    app_state.entitlement.set_notice(notice);

    if matches!(
        failure.reason,
        StudioEntitlementFailureReason::AuthenticationRequired
    ) {
        app_state.auth_session.record_error(log_message.clone());
    }

    push_log_if_needed(app_state, level, &log_message);
}

fn notice_for_failure(failure: &StudioEntitlementFailure) -> EntitlementNotice {
    match failure.reason {
        StudioEntitlementFailureReason::ConnectionUnavailable => EntitlementNotice::new(
            EntitlementNoticeLevel::Warning,
            "Connection unavailable",
            failure.message.clone(),
        ),
        StudioEntitlementFailureReason::ServiceUnavailable => EntitlementNotice::new(
            EntitlementNoticeLevel::Warning,
            "Control plane unavailable",
            failure.message.clone(),
        ),
        StudioEntitlementFailureReason::AuthenticationRequired => EntitlementNotice::new(
            EntitlementNoticeLevel::Error,
            "Login required",
            failure.message.clone(),
        ),
        StudioEntitlementFailureReason::AccessDenied => EntitlementNotice::new(
            EntitlementNoticeLevel::Error,
            "Access denied",
            failure.message.clone(),
        ),
        StudioEntitlementFailureReason::InvalidResponse => EntitlementNotice::new(
            EntitlementNoticeLevel::Error,
            "Control plane response invalid",
            failure.message.clone(),
        ),
        StudioEntitlementFailureReason::LocalStateInvalid => EntitlementNotice::new(
            EntitlementNoticeLevel::Error,
            "Local auth cache invalid",
            failure.message.clone(),
        ),
    }
}

fn log_payload_for_failure(
    action: StudioEntitlementAction,
    failure: &StudioEntitlementFailure,
) -> (AppLogLevel, String) {
    let action_label = match action {
        StudioEntitlementAction::SyncEntitlement => "sync entitlement",
        StudioEntitlementAction::RefreshOfflineLease => "refresh offline lease",
    };
    let level = match failure.reason {
        StudioEntitlementFailureReason::ConnectionUnavailable
        | StudioEntitlementFailureReason::ServiceUnavailable => AppLogLevel::Warning,
        StudioEntitlementFailureReason::AuthenticationRequired
        | StudioEntitlementFailureReason::AccessDenied
        | StudioEntitlementFailureReason::InvalidResponse
        | StudioEntitlementFailureReason::LocalStateInvalid => AppLogLevel::Error,
    };

    (
        level,
        format!("{action_label} failed: {}", failure.message),
    )
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

fn snapshot_entitlement_action_outcome(
    app_state: &AppState,
    action: StudioEntitlementAction,
    outcome: StudioEntitlementOutcome,
) -> StudioEntitlementActionOutcome {
    StudioEntitlementActionOutcome {
        action,
        outcome,
        entitlement_status: app_state.entitlement.status,
        last_synced_at: app_state.entitlement.last_synced_at,
        last_error: app_state.entitlement.last_error.clone(),
        notice: app_state.entitlement.notice.clone(),
        latest_log_entry: app_state.log_feed.entries.back().cloned(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        StoredPropertyPackageRecord, StoredPropertyPackageSource,
    };
    use rf_ui::{
        AuthenticatedUser, DocumentMetadata, EntitlementSnapshot, FlowsheetDocument,
        OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant,
        PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
        PropertyPackageSource, SecureCredentialHandle, TokenLease,
    };

    use super::{
        StudioEntitlementAction, StudioEntitlementFailureReason, StudioEntitlementOutcome,
        refresh_offline_lease_with_control_plane, sync_entitlement_with_control_plane,
    };
    use crate::{
        RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse,
    };

    const SAMPLE_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";

    #[test]
    fn sync_entitlement_updates_runtime_state_and_notice() {
        let client = ScriptedControlPlaneClient::success();
        let mut app_state = sample_app_state();

        let dispatch = sync_entitlement_with_control_plane(&client, &mut app_state, "access");

        assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
        assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
        assert_eq!(dispatch.entitlement_status, rf_ui::EntitlementStatus::Active);
        assert_eq!(
            dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Entitlement synced")
        );
        assert!(app_state.entitlement.is_package_allowed(SAMPLE_PACKAGE_ID));
        assert_eq!(
            dispatch
                .latest_log_entry
                .as_ref()
                .map(|entry| entry.message.as_str()),
            Some("Synced entitlement snapshot and property package manifests from control plane")
        );
    }

    #[test]
    fn sync_entitlement_preserves_existing_state_when_connection_is_unavailable() {
        let client = ScriptedControlPlaneClient::entitlement_failure(
            RadishFlowControlPlaneClientError::connection_unavailable("offline"),
        );
        let mut app_state = sample_app_state();
        app_state.update_entitlement(
            sample_snapshot(),
            vec![sample_manifest()],
            timestamp(150),
        );

        let dispatch = sync_entitlement_with_control_plane(&client, &mut app_state, "access");

        match dispatch.outcome {
            StudioEntitlementOutcome::Failed(failure) => {
                assert_eq!(
                    failure.reason,
                    StudioEntitlementFailureReason::ConnectionUnavailable
                );
            }
            other => panic!("expected failure dispatch, got {other:?}"),
        }
        assert_eq!(dispatch.entitlement_status, rf_ui::EntitlementStatus::Active);
        assert_eq!(
            dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Connection unavailable")
        );
        assert_eq!(
            app_state.entitlement.last_error.as_deref(),
            Some("sync entitlement failed: offline")
        );
    }

    #[test]
    fn refresh_offline_lease_updates_auth_cache_and_notice() {
        let client = ScriptedControlPlaneClient::success();
        let mut app_state = sample_app_state();
        app_state.update_entitlement(
            sample_snapshot(),
            vec![sample_manifest()],
            timestamp(150),
        );
        let mut auth_cache_index = sample_auth_cache_index();

        let dispatch = refresh_offline_lease_with_control_plane(
            &client,
            &mut app_state,
            &mut auth_cache_index,
            "access",
        );

        assert_eq!(
            dispatch.action,
            StudioEntitlementAction::RefreshOfflineLease
        );
        assert_eq!(dispatch.outcome, StudioEntitlementOutcome::OfflineLeaseRefreshed);
        assert_eq!(
            dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Offline lease refreshed")
        );
        assert_eq!(auth_cache_index.last_synced_at, Some(timestamp(210)));
        assert_eq!(dispatch.last_synced_at, Some(timestamp(210)));
    }

    #[test]
    fn refresh_offline_lease_marks_auth_session_error_when_login_is_required() {
        let client = ScriptedControlPlaneClient::offline_refresh_failure(
            RadishFlowControlPlaneClientError::unauthorized("token expired"),
        );
        let mut app_state = sample_app_state();
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot(),
            vec![sample_manifest()],
            timestamp(150),
        );
        let mut auth_cache_index = sample_auth_cache_index();

        let dispatch = refresh_offline_lease_with_control_plane(
            &client,
            &mut app_state,
            &mut auth_cache_index,
            "access",
        );

        match dispatch.outcome {
            StudioEntitlementOutcome::Failed(failure) => {
                assert_eq!(
                    failure.reason,
                    StudioEntitlementFailureReason::AuthenticationRequired
                );
            }
            other => panic!("expected failure dispatch, got {other:?}"),
        }
        assert_eq!(app_state.auth_session.status, rf_ui::AuthSessionStatus::Error);
        assert_eq!(
            dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Login required")
        );
    }

    fn sample_app_state() -> rf_ui::AppState {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-entitlement", "Entitlement Demo", timestamp(10));
        rf_ui::AppState::new(FlowsheetDocument::new(flowsheet, metadata))
    }

    fn complete_login(app_state: &mut rf_ui::AppState) {
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

    fn sample_snapshot() -> EntitlementSnapshot {
        EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            features: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: BTreeSet::from([SAMPLE_PACKAGE_ID.to_string()]),
        }
    }

    fn sample_manifest() -> PropertyPackageManifest {
        let mut manifest = PropertyPackageManifest::new(
            SAMPLE_PACKAGE_ID,
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

    fn sample_auth_cache_index() -> StoredAuthCacheIndex {
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        index.entitlement = Some(StoredEntitlementCache {
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            synced_at: timestamp(100),
            issued_at: timestamp(90),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(700)),
            feature_keys: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: BTreeSet::from([SAMPLE_PACKAGE_ID.to_string()]),
        });
        let mut record = StoredPropertyPackageRecord::new(
            SAMPLE_PACKAGE_ID,
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:pkg-1",
            1024,
            timestamp(110),
        );
        record.expires_at = Some(timestamp(900));
        index.property_packages.push(record);
        index
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
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

        fn entitlement_failure(error: RadishFlowControlPlaneClientError) -> Self {
            Self {
                entitlement_response: Err(error),
                ..Self::success()
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
                "lease request is not used in entitlement control tests",
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
