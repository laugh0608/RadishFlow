use std::time::{Duration, SystemTime};

use rf_types::RfResult;
use rf_ui::{AppState, AuthSessionStatus, EntitlementIntent, EntitlementStatus};

use crate::{
    RadishFlowControlPlaneClient, StudioAppCommandOutcome, StudioAppFacade,
    StudioAppMutableAuthCacheContext, dispatch_entitlement_panel_intent_with_control_plane,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementPreflightAction {
    SyncEntitlement,
    RefreshOfflineLease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPreflightDecision {
    pub action: EntitlementPreflightAction,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPreflightOutcome {
    pub decision: EntitlementPreflightDecision,
    pub outcome: StudioAppCommandOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPreflightPolicy {
    pub offline_refresh_window: Duration,
}

impl Default for EntitlementPreflightPolicy {
    fn default() -> Self {
        Self {
            offline_refresh_window: Duration::from_secs(30 * 60),
        }
    }
}

pub fn decide_entitlement_preflight_action(
    app_state: &AppState,
    now: SystemTime,
    policy: &EntitlementPreflightPolicy,
) -> Option<EntitlementPreflightDecision> {
    if !has_authenticated_session(app_state) {
        return None;
    }

    let entitlement = &app_state.entitlement;
    if entitlement.snapshot.is_none()
        || entitlement.last_synced_at.is_none()
        || matches!(
            entitlement.status,
            EntitlementStatus::Unknown | EntitlementStatus::Error
        )
    {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::SyncEntitlement,
            reason: "authenticated session is present but entitlement snapshot is unavailable",
        });
    }

    if matches!(entitlement.status, EntitlementStatus::LeaseExpired) {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::RefreshOfflineLease,
            reason: "offline lease is already expired and must be refreshed",
        });
    }

    let Some(offline_lease_expires_at) = entitlement
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.offline_lease_expires_at)
    else {
        return None;
    };

    if now >= offline_lease_expires_at {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::RefreshOfflineLease,
            reason: "offline lease deadline has passed",
        });
    }

    let remaining = offline_lease_expires_at
        .duration_since(now)
        .unwrap_or(Duration::ZERO);
    if remaining <= policy.offline_refresh_window {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::RefreshOfflineLease,
            reason: "offline lease is approaching expiration and should be refreshed",
        });
    }

    None
}

pub fn dispatch_entitlement_preflight_with_control_plane<Client>(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &mut StudioAppMutableAuthCacheContext<'_>,
    control_plane_client: &Client,
    access_token: &str,
    now: SystemTime,
    policy: &EntitlementPreflightPolicy,
) -> RfResult<Option<EntitlementPreflightOutcome>>
where
    Client: RadishFlowControlPlaneClient,
{
    let Some(decision) = decide_entitlement_preflight_action(app_state, now, policy) else {
        return Ok(None);
    };

    let intent = match decision.action {
        EntitlementPreflightAction::SyncEntitlement => EntitlementIntent::sync_entitlement(),
        EntitlementPreflightAction::RefreshOfflineLease => {
            EntitlementIntent::refresh_offline_lease()
        }
    };
    let outcome = dispatch_entitlement_panel_intent_with_control_plane(
        facade,
        app_state,
        context,
        control_plane_client,
        access_token,
        &intent,
    )?;

    Ok(Some(EntitlementPreflightOutcome { decision, outcome }))
}

fn has_authenticated_session(app_state: &AppState) -> bool {
    matches!(
        app_state.auth_session.status,
        AuthSessionStatus::Authenticated
    ) && app_state.auth_session.authority_url.is_some()
        && app_state.auth_session.current_user.is_some()
        && app_state.auth_session.token_lease.is_some()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache};
    use rf_ui::{
        AppState, AuthenticatedUser, DocumentMetadata, EntitlementSnapshot, FlowsheetDocument,
        OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant,
        PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
        PropertyPackageSource, SecureCredentialHandle, TokenLease,
    };

    use super::{
        EntitlementPreflightAction, EntitlementPreflightPolicy,
        decide_entitlement_preflight_action, dispatch_entitlement_preflight_with_control_plane,
    };
    use crate::{
        RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, StudioAppFacade,
        StudioAppMutableAuthCacheContext, StudioAppResultDispatch, StudioEntitlementAction,
        StudioEntitlementOutcome,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        FlowsheetDocument::new(
            Flowsheet::new("demo"),
            DocumentMetadata::new(
                "doc-entitlement-preflight",
                "Entitlement Preflight",
                timestamp(10),
            ),
        )
    }

    fn complete_login(app_state: &mut AppState) {
        app_state.complete_login(
            "https://id.radish.local",
            AuthenticatedUser::new("user-123", "luobo"),
            TokenLease::new(
                timestamp(500),
                SecureCredentialHandle::new("radishflow-studio", "user-123-primary"),
            ),
            timestamp(120),
        );
    }

    fn sample_snapshot(offline_lease_expires_at: u64) -> EntitlementSnapshot {
        EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(400),
            offline_lease_expires_at: Some(timestamp(offline_lease_expires_at)),
            features: ["desktop-login".to_string()].into_iter().collect(),
            allowed_package_ids: ["binary-hydrocarbon-lite-v1".to_string()]
                .into_iter()
                .collect(),
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
        manifest.expires_at = Some(timestamp(700));
        manifest
    }

    fn sample_auth_cache_index(snapshot: &EntitlementSnapshot) -> StoredAuthCacheIndex {
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        index.entitlement = Some(StoredEntitlementCache {
            subject_id: snapshot.subject_id.clone(),
            tenant_id: snapshot.tenant_id.clone(),
            synced_at: timestamp(150),
            issued_at: snapshot.issued_at,
            expires_at: snapshot.expires_at,
            offline_lease_expires_at: snapshot.offline_lease_expires_at,
            feature_keys: snapshot.features.clone(),
            allowed_package_ids: snapshot.allowed_package_ids.clone(),
        });
        index.last_synced_at = Some(timestamp(150));
        index
    }

    fn sample_offline_refresh_response() -> OfflineLeaseRefreshResponse {
        OfflineLeaseRefreshResponse {
            refreshed_at: timestamp(210),
            snapshot: sample_snapshot(900),
            manifest_list: PropertyPackageManifestList::new(
                timestamp(205),
                vec![sample_manifest()],
            ),
        }
    }

    #[derive(Debug, Clone)]
    struct ScriptedControlPlaneClient;

    impl RadishFlowControlPlaneClient for ScriptedControlPlaneClient {
        fn fetch_entitlement_snapshot(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        > {
            Ok(RadishFlowControlPlaneResponse::new(
                sample_snapshot(900),
                timestamp(200),
            ))
        }

        fn fetch_property_package_manifest_list(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        > {
            Ok(RadishFlowControlPlaneResponse::new(
                PropertyPackageManifestList::new(timestamp(205), vec![sample_manifest()]),
                timestamp(205),
            ))
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
                "entitlement preflight tests do not request leases",
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
            Ok(RadishFlowControlPlaneResponse::new(
                sample_offline_refresh_response(),
                timestamp(210),
            ))
        }
    }

    #[test]
    fn preflight_requires_authenticated_session() {
        let app_state = AppState::new(sample_document());

        let decision = decide_entitlement_preflight_action(
            &app_state,
            timestamp(200),
            &EntitlementPreflightPolicy::default(),
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn preflight_syncs_when_snapshot_is_missing() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);

        let decision = decide_entitlement_preflight_action(
            &app_state,
            timestamp(200),
            &EntitlementPreflightPolicy::default(),
        )
        .expect("expected preflight decision");

        assert_eq!(decision.action, EntitlementPreflightAction::SyncEntitlement);
    }

    #[test]
    fn preflight_refreshes_when_offline_lease_is_near_expiry() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot(210),
            vec![sample_manifest()],
            timestamp(150),
        );

        let decision = decide_entitlement_preflight_action(
            &app_state,
            timestamp(200),
            &EntitlementPreflightPolicy::default(),
        )
        .expect("expected preflight decision");

        assert_eq!(
            decision.action,
            EntitlementPreflightAction::RefreshOfflineLease
        );
    }

    #[test]
    fn preflight_dispatch_returns_none_when_entitlement_is_fresh() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let snapshot = sample_snapshot(5_000);
        app_state.update_entitlement(snapshot.clone(), vec![sample_manifest()], timestamp(150));
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index(&snapshot);
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);

        let outcome = dispatch_entitlement_preflight_with_control_plane(
            &facade,
            &mut app_state,
            &mut context,
            &client,
            "access-token",
            timestamp(200),
            &EntitlementPreflightPolicy::default(),
        )
        .expect("expected preflight dispatch");

        assert_eq!(outcome, None);
    }

    #[test]
    fn preflight_dispatch_executes_refresh_when_needed() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let snapshot = sample_snapshot(210);
        app_state.update_entitlement(snapshot.clone(), vec![sample_manifest()], timestamp(150));
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index(&snapshot);
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);

        let outcome = dispatch_entitlement_preflight_with_control_plane(
            &facade,
            &mut app_state,
            &mut context,
            &client,
            "access-token",
            timestamp(200),
            &EntitlementPreflightPolicy::default(),
        )
        .expect("expected preflight dispatch")
        .expect("expected executed preflight");

        assert_eq!(
            outcome.decision.action,
            EntitlementPreflightAction::RefreshOfflineLease
        );
        match outcome.outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(
                    dispatch.action,
                    StudioEntitlementAction::RefreshOfflineLease
                );
                assert_eq!(
                    dispatch.outcome,
                    StudioEntitlementOutcome::OfflineLeaseRefreshed
                );
            }
            other => panic!("expected entitlement dispatch, got {other:?}"),
        }
    }
}
