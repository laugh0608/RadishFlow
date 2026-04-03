use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_types::{RfError, RfResult};
use rf_ui::{AppState, AuthSessionStatus, EntitlementIntent, EntitlementStatus};

use crate::{
    RadishFlowControlPlaneClient, StudioAppCommandOutcome, StudioAppFacade,
    StudioAppMutableAuthCacheContext, StudioAppResultDispatch, StudioEntitlementAction,
    StudioEntitlementFailureReason, StudioEntitlementOutcome,
    dispatch_entitlement_panel_intent_with_control_plane,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionPolicy {
    pub preflight: EntitlementPreflightPolicy,
    pub entitlement_sync_window: Duration,
    pub transient_failure_backoff_base: Duration,
    pub permanent_failure_backoff_base: Duration,
    pub failure_backoff_max: Duration,
}

impl Default for EntitlementSessionPolicy {
    fn default() -> Self {
        Self {
            preflight: EntitlementPreflightPolicy::default(),
            entitlement_sync_window: Duration::from_secs(15 * 60),
            transient_failure_backoff_base: Duration::from_secs(60),
            permanent_failure_backoff_base: Duration::from_secs(10 * 60),
            failure_backoff_max: Duration::from_secs(60 * 60),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionBackoff {
    pub action: EntitlementPreflightAction,
    pub failure_reason: StudioEntitlementFailureReason,
    pub consecutive_failures: u32,
    pub retry_not_before: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EntitlementSessionState {
    pub last_attempted_action: Option<EntitlementPreflightAction>,
    pub last_attempted_at: Option<SystemTime>,
    pub last_completed_action: Option<EntitlementPreflightAction>,
    pub last_completed_at: Option<SystemTime>,
    pub backoff: Option<EntitlementSessionBackoff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionSchedule {
    pub recommended_action: Option<EntitlementPreflightAction>,
    pub recommended_reason: Option<&'static str>,
    pub recommended_at: Option<SystemTime>,
    pub next_sync_at: Option<SystemTime>,
    pub next_offline_refresh_at: Option<SystemTime>,
    pub next_check_at: Option<SystemTime>,
    pub blocked_by_backoff: bool,
    pub backoff: Option<EntitlementSessionBackoff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionTickOutcome {
    pub preflight: Option<EntitlementPreflightOutcome>,
    pub schedule: EntitlementSessionSchedule,
}

pub struct EntitlementSessionRuntime<'a, 'cache, Client>
where
    Client: RadishFlowControlPlaneClient,
{
    pub facade: &'a StudioAppFacade,
    pub app_state: &'a mut AppState,
    pub context: &'a mut StudioAppMutableAuthCacheContext<'cache>,
    pub control_plane_client: &'a Client,
    pub access_token: &'a str,
    pub now: SystemTime,
    pub policy: &'a EntitlementSessionPolicy,
    pub session_state: &'a mut EntitlementSessionState,
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

    let offline_lease_expires_at = entitlement
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.offline_lease_expires_at)?;

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

pub fn snapshot_entitlement_session_schedule(
    app_state: &AppState,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
    session_state: &EntitlementSessionState,
) -> EntitlementSessionSchedule {
    let immediate =
        decide_entitlement_session_action(app_state, now, policy).map(|decision| (decision, now));
    let next_sync_at = recommended_sync_at(app_state, policy);
    let next_offline_refresh_at = recommended_offline_refresh_at(app_state, policy);
    let next_scheduled_check_at = earliest_system_time(next_sync_at, next_offline_refresh_at);

    let (recommended_action, recommended_reason, recommended_at) = immediate
        .as_ref()
        .map(|(decision, recommended_at)| {
            (
                Some(decision.action),
                Some(decision.reason),
                Some(*recommended_at),
            )
        })
        .unwrap_or((None, None, None));

    let applicable_backoff = immediate
        .as_ref()
        .and_then(|(decision, _)| active_backoff_for_action(session_state, decision.action, now));
    let blocked_by_backoff = applicable_backoff.is_some();
    let next_check_at = if let Some(backoff) = applicable_backoff.as_ref() {
        Some(backoff.retry_not_before)
    } else if immediate.is_some() {
        Some(now)
    } else {
        next_scheduled_check_at
    };

    EntitlementSessionSchedule {
        recommended_action,
        recommended_reason,
        recommended_at,
        next_sync_at,
        next_offline_refresh_at,
        next_check_at,
        blocked_by_backoff,
        backoff: applicable_backoff,
    }
}

pub fn dispatch_entitlement_session_tick_with_control_plane<Client>(
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionTickOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let immediate =
        decide_entitlement_session_action(runtime.app_state, runtime.now, runtime.policy);
    let preflight = match immediate {
        Some(decision)
            if active_backoff_for_action(runtime.session_state, decision.action, runtime.now)
                .is_none() =>
        {
            let intent = match decision.action {
                EntitlementPreflightAction::SyncEntitlement => {
                    EntitlementIntent::sync_entitlement()
                }
                EntitlementPreflightAction::RefreshOfflineLease => {
                    EntitlementIntent::refresh_offline_lease()
                }
            };
            let outcome = dispatch_entitlement_panel_intent_with_control_plane(
                runtime.facade,
                runtime.app_state,
                runtime.context,
                runtime.control_plane_client,
                runtime.access_token,
                &intent,
            )?;
            let preflight = EntitlementPreflightOutcome { decision, outcome };
            record_entitlement_session_outcome(
                runtime.session_state,
                &preflight,
                runtime.now,
                runtime.policy,
            )?;
            Some(preflight)
        }
        _ => None,
    };

    let schedule = snapshot_entitlement_session_schedule(
        runtime.app_state,
        runtime.now,
        runtime.policy,
        runtime.session_state,
    );
    Ok(EntitlementSessionTickOutcome {
        preflight,
        schedule,
    })
}

pub fn record_entitlement_session_outcome(
    session_state: &mut EntitlementSessionState,
    outcome: &EntitlementPreflightOutcome,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
) -> RfResult<()> {
    let entitlement_outcome = outcome
        .outcome
        .dispatch
        .as_entitlement_outcome()
        .ok_or_else(|| {
            RfError::invalid_input(
                "entitlement session scheduler expected entitlement dispatch outcome",
            )
        })?;
    apply_entitlement_session_outcome(
        session_state,
        outcome.decision.action,
        entitlement_outcome,
        now,
        policy,
    );
    Ok(())
}

pub fn record_entitlement_session_dispatch(
    session_state: &mut EntitlementSessionState,
    action: StudioEntitlementAction,
    outcome: &StudioEntitlementOutcome,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
) {
    let action = match action {
        StudioEntitlementAction::SyncEntitlement => EntitlementPreflightAction::SyncEntitlement,
        StudioEntitlementAction::RefreshOfflineLease => {
            EntitlementPreflightAction::RefreshOfflineLease
        }
    };
    apply_entitlement_session_outcome(session_state, action, outcome, now, policy);
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

fn decide_entitlement_session_action(
    app_state: &AppState,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
) -> Option<EntitlementPreflightDecision> {
    if let Some(decision) = decide_entitlement_preflight_action(app_state, now, &policy.preflight) {
        return Some(decision);
    }

    if !has_authenticated_session(app_state) {
        return None;
    }

    let entitlement = &app_state.entitlement;
    let snapshot = entitlement.snapshot.as_ref()?;

    if now >= snapshot.expires_at {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::SyncEntitlement,
            reason: "entitlement snapshot has expired and should be synchronized",
        });
    }

    let remaining = snapshot
        .expires_at
        .duration_since(now)
        .unwrap_or(Duration::ZERO);
    if remaining <= policy.entitlement_sync_window {
        return Some(EntitlementPreflightDecision {
            action: EntitlementPreflightAction::SyncEntitlement,
            reason: "entitlement snapshot is approaching expiration and should be synchronized",
        });
    }

    None
}

fn recommended_sync_at(
    app_state: &AppState,
    policy: &EntitlementSessionPolicy,
) -> Option<SystemTime> {
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
        return entitlement
            .last_synced_at
            .or(app_state.auth_session.last_authenticated_at);
    }

    let snapshot = entitlement.snapshot.as_ref()?;
    Some(saturating_sub_system_time(
        snapshot.expires_at,
        policy.entitlement_sync_window,
    ))
}

fn recommended_offline_refresh_at(
    app_state: &AppState,
    policy: &EntitlementSessionPolicy,
) -> Option<SystemTime> {
    if !has_authenticated_session(app_state) {
        return None;
    }

    let entitlement = &app_state.entitlement;
    if matches!(entitlement.status, EntitlementStatus::LeaseExpired) {
        return entitlement
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.offline_lease_expires_at)
            .or(entitlement.last_synced_at)
            .or(app_state.auth_session.last_authenticated_at);
    }

    let offline_lease_expires_at = entitlement
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.offline_lease_expires_at)?;
    Some(saturating_sub_system_time(
        offline_lease_expires_at,
        policy.preflight.offline_refresh_window,
    ))
}

fn earliest_system_time(lhs: Option<SystemTime>, rhs: Option<SystemTime>) -> Option<SystemTime> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => Some(if lhs <= rhs { lhs } else { rhs }),
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn active_backoff_for_action(
    session_state: &EntitlementSessionState,
    action: EntitlementPreflightAction,
    now: SystemTime,
) -> Option<EntitlementSessionBackoff> {
    session_state
        .backoff
        .as_ref()
        .filter(|backoff| backoff.action == action && now < backoff.retry_not_before)
        .cloned()
}

fn apply_entitlement_session_outcome(
    session_state: &mut EntitlementSessionState,
    action: EntitlementPreflightAction,
    outcome: &StudioEntitlementOutcome,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
) {
    session_state.last_attempted_action = Some(action);
    session_state.last_attempted_at = Some(now);

    match outcome {
        StudioEntitlementOutcome::Synced | StudioEntitlementOutcome::OfflineLeaseRefreshed => {
            session_state.last_completed_action = Some(action);
            session_state.last_completed_at = Some(now);
            session_state.backoff = None;
        }
        StudioEntitlementOutcome::Failed(failure) => {
            let consecutive_failures = session_state
                .backoff
                .as_ref()
                .filter(|backoff| backoff.action == action)
                .map(|backoff| backoff.consecutive_failures.saturating_add(1))
                .unwrap_or(1);
            let backoff_duration =
                failure_backoff_duration(failure.reason, consecutive_failures, policy);
            let retry_not_before = now.checked_add(backoff_duration).unwrap_or(now);
            session_state.backoff = Some(EntitlementSessionBackoff {
                action,
                failure_reason: failure.reason,
                consecutive_failures,
                retry_not_before,
            });
        }
    }
}

fn failure_backoff_duration(
    reason: StudioEntitlementFailureReason,
    consecutive_failures: u32,
    policy: &EntitlementSessionPolicy,
) -> Duration {
    let base = match reason {
        StudioEntitlementFailureReason::ConnectionUnavailable
        | StudioEntitlementFailureReason::ServiceUnavailable => {
            policy.transient_failure_backoff_base
        }
        StudioEntitlementFailureReason::AuthenticationRequired
        | StudioEntitlementFailureReason::AccessDenied
        | StudioEntitlementFailureReason::InvalidResponse
        | StudioEntitlementFailureReason::LocalStateInvalid => {
            policy.permanent_failure_backoff_base
        }
    };
    let factor = 1_u32 << consecutive_failures.saturating_sub(1).min(30);
    let scaled = base
        .checked_mul(factor)
        .unwrap_or(policy.failure_backoff_max);
    if scaled > policy.failure_backoff_max {
        policy.failure_backoff_max
    } else {
        scaled
    }
}

fn saturating_sub_system_time(time: SystemTime, duration: Duration) -> SystemTime {
    time.checked_sub(duration).unwrap_or(UNIX_EPOCH)
}

trait StudioAppResultDispatchExt {
    fn as_entitlement_outcome(&self) -> Option<&StudioEntitlementOutcome>;
}

impl StudioAppResultDispatchExt for StudioAppResultDispatch {
    fn as_entitlement_outcome(&self) -> Option<&StudioEntitlementOutcome> {
        match self {
            StudioAppResultDispatch::Entitlement(dispatch) => Some(&dispatch.outcome),
            StudioAppResultDispatch::WorkspaceRun(_)
            | StudioAppResultDispatch::WorkspaceMode(_) => None,
        }
    }
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
        EntitlementPreflightAction, EntitlementPreflightPolicy, EntitlementSessionPolicy,
        EntitlementSessionRuntime, EntitlementSessionState, decide_entitlement_preflight_action,
        dispatch_entitlement_preflight_with_control_plane,
        dispatch_entitlement_session_tick_with_control_plane, record_entitlement_session_dispatch,
        snapshot_entitlement_session_schedule,
    };
    use crate::{
        RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, StudioAppFacade,
        StudioAppMutableAuthCacheContext, StudioAppResultDispatch, StudioEntitlementAction,
        StudioEntitlementFailure, StudioEntitlementFailureReason, StudioEntitlementOutcome,
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

    fn sample_snapshot_with_expiry(
        expires_at: u64,
        offline_lease_expires_at: u64,
    ) -> EntitlementSnapshot {
        EntitlementSnapshot {
            expires_at: timestamp(expires_at),
            ..sample_snapshot(offline_lease_expires_at)
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

    #[derive(Debug, Clone)]
    struct FailingSyncControlPlaneClient;

    impl RadishFlowControlPlaneClient for FailingSyncControlPlaneClient {
        fn fetch_entitlement_snapshot(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        > {
            Err(RadishFlowControlPlaneClientError::connection_unavailable(
                "offline",
            ))
        }

        fn fetch_property_package_manifest_list(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        > {
            Err(RadishFlowControlPlaneClientError::connection_unavailable(
                "offline",
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
                "session scheduler tests do not request leases",
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
            Err(RadishFlowControlPlaneClientError::connection_unavailable(
                "offline",
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

    #[test]
    fn session_schedule_recommends_sync_when_snapshot_is_near_expiry() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(260, 5_000),
            vec![sample_manifest()],
            timestamp(150),
        );

        let schedule = snapshot_entitlement_session_schedule(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        assert_eq!(
            schedule.recommended_action,
            Some(EntitlementPreflightAction::SyncEntitlement)
        );
        assert_eq!(
            schedule.recommended_reason,
            Some("entitlement snapshot is approaching expiration and should be synchronized")
        );
        assert_eq!(schedule.next_check_at, Some(timestamp(200)));
        assert!(!schedule.blocked_by_backoff);
    }

    #[test]
    fn session_tick_records_backoff_after_sync_failure() {
        let facade = StudioAppFacade::new();
        let client = FailingSyncControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index(&sample_snapshot(900));
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let mut session_state = EntitlementSessionState::default();

        let policy = EntitlementSessionPolicy::default();
        let mut runtime = EntitlementSessionRuntime {
            facade: &facade,
            app_state: &mut app_state,
            context: &mut context,
            control_plane_client: &client,
            access_token: "access-token",
            now: timestamp(200),
            policy: &policy,
            session_state: &mut session_state,
        };
        let tick = dispatch_entitlement_session_tick_with_control_plane(&mut runtime)
            .expect("expected scheduler tick");

        let preflight = tick.preflight.expect("expected executed session tick");
        assert_eq!(
            preflight.decision.action,
            EntitlementPreflightAction::SyncEntitlement
        );
        assert!(tick.schedule.blocked_by_backoff);
        assert_eq!(tick.schedule.next_check_at, Some(timestamp(260)));
        assert_eq!(
            tick.schedule
                .backoff
                .as_ref()
                .map(|backoff| backoff.failure_reason),
            Some(StudioEntitlementFailureReason::ConnectionUnavailable)
        );
        assert_eq!(
            session_state
                .backoff
                .as_ref()
                .map(|backoff| backoff.consecutive_failures),
            Some(1)
        );
    }

    #[test]
    fn session_schedule_uses_backoff_for_manual_entitlement_failures() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(260, 5_000),
            vec![sample_manifest()],
            timestamp(150),
        );
        let policy = EntitlementSessionPolicy::default();
        let mut session_state = EntitlementSessionState::default();
        record_entitlement_session_dispatch(
            &mut session_state,
            StudioEntitlementAction::SyncEntitlement,
            &StudioEntitlementOutcome::Failed(StudioEntitlementFailure {
                reason: StudioEntitlementFailureReason::AuthenticationRequired,
                message: "token expired".to_string(),
            }),
            timestamp(200),
            &policy,
        );

        let schedule = snapshot_entitlement_session_schedule(
            &app_state,
            timestamp(205),
            &policy,
            &session_state,
        );

        assert_eq!(
            schedule.recommended_action,
            Some(EntitlementPreflightAction::SyncEntitlement)
        );
        assert!(schedule.blocked_by_backoff);
        assert_eq!(schedule.next_check_at, Some(timestamp(800)));
        assert_eq!(
            schedule
                .backoff
                .as_ref()
                .map(|backoff| backoff.failure_reason),
            Some(StudioEntitlementFailureReason::AuthenticationRequired)
        );
    }

    #[test]
    fn session_tick_clears_backoff_after_successful_refresh() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let snapshot = sample_snapshot(210);
        app_state.update_entitlement(snapshot.clone(), vec![sample_manifest()], timestamp(150));
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index(&snapshot);
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let policy = EntitlementSessionPolicy::default();
        let mut session_state = EntitlementSessionState {
            backoff: Some(super::EntitlementSessionBackoff {
                action: EntitlementPreflightAction::RefreshOfflineLease,
                failure_reason: StudioEntitlementFailureReason::ConnectionUnavailable,
                consecutive_failures: 2,
                retry_not_before: timestamp(190),
            }),
            ..EntitlementSessionState::default()
        };

        let mut runtime = EntitlementSessionRuntime {
            facade: &facade,
            app_state: &mut app_state,
            context: &mut context,
            control_plane_client: &client,
            access_token: "access-token",
            now: timestamp(200),
            policy: &policy,
            session_state: &mut session_state,
        };
        let tick = dispatch_entitlement_session_tick_with_control_plane(&mut runtime)
            .expect("expected scheduler tick");

        let preflight = tick.preflight.expect("expected executed session tick");
        assert_eq!(
            preflight.decision.action,
            EntitlementPreflightAction::RefreshOfflineLease
        );
        assert_eq!(session_state.backoff, None);
        assert!(!tick.schedule.blocked_by_backoff);
        assert_eq!(session_state.last_completed_at, Some(timestamp(200)));
    }
}
