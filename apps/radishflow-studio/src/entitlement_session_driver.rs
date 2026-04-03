use std::time::SystemTime;

use rf_types::RfResult;
use rf_ui::{AppState, EntitlementActionId, EntitlementPanelWidgetEvent};

use crate::{
    EntitlementPanelDriverState, EntitlementPanelWidgetDispatchOutcome, EntitlementSessionPolicy,
    EntitlementSessionRuntime, EntitlementSessionSchedule, EntitlementSessionState,
    EntitlementSessionTickOutcome, RadishFlowControlPlaneClient, StudioAppCommandOutcome,
    StudioEntitlementActionOutcome, dispatch_entitlement_panel_widget_event_with_control_plane,
    dispatch_entitlement_session_tick_with_control_plane, record_entitlement_session_dispatch,
    snapshot_entitlement_panel_driver_state, snapshot_entitlement_session_schedule,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionDriverState {
    pub panel: EntitlementPanelDriverState,
    pub session_state: EntitlementSessionState,
    pub schedule: EntitlementSessionSchedule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionTickDriverOutcome {
    pub tick: EntitlementSessionTickOutcome,
    pub state: EntitlementSessionDriverState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionPanelDriverOutcome {
    pub dispatch: EntitlementPanelWidgetDispatchOutcome,
    pub state: EntitlementSessionDriverState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionEvent {
    SessionStarted,
    LoginCompleted,
    TimerElapsed,
    EntitlementCommandCompleted(StudioEntitlementActionOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionEventOutcome {
    Tick(Box<EntitlementSessionTickOutcome>),
    RecordedCommand {
        action: crate::StudioEntitlementAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionEventDriverOutcome {
    pub event: EntitlementSessionEvent,
    pub outcome: EntitlementSessionEventOutcome,
    pub state: EntitlementSessionDriverState,
}

pub fn snapshot_entitlement_session_driver_state(
    app_state: &AppState,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
    session_state: &EntitlementSessionState,
) -> EntitlementSessionDriverState {
    EntitlementSessionDriverState {
        panel: snapshot_entitlement_panel_driver_state(app_state),
        session_state: session_state.clone(),
        schedule: snapshot_entitlement_session_schedule(app_state, now, policy, session_state),
    }
}

pub fn dispatch_entitlement_session_tick_driver_with_control_plane<Client>(
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionTickDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let tick = dispatch_entitlement_session_tick_with_control_plane(runtime)?;
    let state = snapshot_runtime_state(runtime);

    Ok(EntitlementSessionTickDriverOutcome { tick, state })
}

pub fn dispatch_entitlement_session_event_with_control_plane<Client>(
    event: EntitlementSessionEvent,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionEventDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let outcome = match &event {
        EntitlementSessionEvent::SessionStarted
        | EntitlementSessionEvent::LoginCompleted
        | EntitlementSessionEvent::TimerElapsed => EntitlementSessionEventOutcome::Tick(Box::new(
            dispatch_entitlement_session_tick_with_control_plane(runtime)?,
        )),
        EntitlementSessionEvent::EntitlementCommandCompleted(dispatch) => {
            record_entitlement_session_dispatch(
                runtime.session_state,
                dispatch.action,
                &dispatch.outcome,
                runtime.now,
                runtime.policy,
            );
            EntitlementSessionEventOutcome::RecordedCommand {
                action: dispatch.action,
            }
        }
    };
    let state = snapshot_runtime_state(runtime);

    Ok(EntitlementSessionEventDriverOutcome {
        event,
        outcome,
        state,
    })
}

pub fn dispatch_entitlement_session_panel_widget_event_with_control_plane<Client>(
    event: &EntitlementPanelWidgetEvent,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionPanelDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let dispatch = dispatch_entitlement_panel_widget_event_with_control_plane(
        runtime.facade,
        runtime.app_state,
        runtime.context,
        runtime.control_plane_client,
        runtime.access_token,
        event,
    )?;
    if let EntitlementPanelWidgetDispatchOutcome::Executed(outcome) = &dispatch {
        record_panel_command_outcome(runtime.session_state, outcome, runtime.now, runtime.policy);
    }
    let state = snapshot_runtime_state(runtime);

    Ok(EntitlementSessionPanelDriverOutcome { dispatch, state })
}

pub fn dispatch_entitlement_session_panel_widget_action_with_control_plane<Client>(
    action_id: EntitlementActionId,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionPanelDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let widget = snapshot_entitlement_panel_driver_state(runtime.app_state).widget;
    dispatch_entitlement_session_panel_widget_event_with_control_plane(
        &widget.activate(action_id),
        runtime,
    )
}

pub fn dispatch_entitlement_session_panel_primary_action_with_control_plane<Client>(
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionPanelDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let widget = snapshot_entitlement_panel_driver_state(runtime.app_state).widget;
    dispatch_entitlement_session_panel_widget_event_with_control_plane(
        &widget.activate_primary(),
        runtime,
    )
}

fn snapshot_runtime_state<Client>(
    runtime: &EntitlementSessionRuntime<'_, '_, Client>,
) -> EntitlementSessionDriverState
where
    Client: RadishFlowControlPlaneClient,
{
    snapshot_entitlement_session_driver_state(
        runtime.app_state,
        runtime.now,
        runtime.policy,
        runtime.session_state,
    )
}

fn record_panel_command_outcome(
    session_state: &mut EntitlementSessionState,
    outcome: &StudioAppCommandOutcome,
    now: SystemTime,
    policy: &EntitlementSessionPolicy,
) {
    if let crate::StudioAppResultDispatch::Entitlement(dispatch) = &outcome.dispatch {
        record_entitlement_session_dispatch(
            session_state,
            dispatch.action,
            &dispatch.outcome,
            now,
            policy,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache};
    use rf_ui::{
        AppState, AuthenticatedUser, DocumentMetadata, EntitlementActionId, EntitlementSnapshot,
        FlowsheetDocument, OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse,
        PropertyPackageLeaseGrant, PropertyPackageLeaseRequest, PropertyPackageManifest,
        PropertyPackageManifestList, PropertyPackageSource, SecureCredentialHandle, TokenLease,
    };

    use super::{
        EntitlementSessionEvent, EntitlementSessionEventOutcome,
        dispatch_entitlement_session_event_with_control_plane,
        dispatch_entitlement_session_panel_primary_action_with_control_plane,
        dispatch_entitlement_session_tick_driver_with_control_plane,
        snapshot_entitlement_session_driver_state,
    };
    use crate::{
        EntitlementPanelWidgetDispatchOutcome, EntitlementPreflightAction,
        EntitlementSessionPolicy, EntitlementSessionRuntime, EntitlementSessionState,
        RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, StudioAppFacade,
        StudioAppMutableAuthCacheContext, StudioAppResultDispatch, StudioEntitlementAction,
        StudioEntitlementOutcome,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new(
            "doc-entitlement-session",
            "Entitlement Session",
            timestamp(10),
        );
        FlowsheetDocument::new(flowsheet, metadata)
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

    fn sample_manifest_list() -> PropertyPackageManifestList {
        PropertyPackageManifestList::new(timestamp(205), vec![sample_manifest()])
    }

    fn sample_offline_refresh_response() -> OfflineLeaseRefreshResponse {
        OfflineLeaseRefreshResponse {
            refreshed_at: timestamp(210),
            snapshot: sample_snapshot(900),
            manifest_list: sample_manifest_list(),
        }
    }

    fn sample_auth_cache_index() -> StoredAuthCacheIndex {
        let snapshot = sample_snapshot(210);
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
                sample_snapshot_with_expiry(2_000, 5_000),
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
                sample_manifest_list(),
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
                "session driver tests do not request leases",
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
    fn snapshot_entitlement_session_driver_state_includes_panel_and_schedule() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(2_000, 5_000),
            vec![sample_manifest()],
            timestamp(150),
        );

        let state = snapshot_entitlement_session_driver_state(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        assert_eq!(
            state.panel.widget.view().primary_action.label,
            "Refresh offline lease"
        );
        assert_eq!(state.schedule.next_check_at, Some(timestamp(1_100)));
    }

    #[test]
    fn session_tick_driver_updates_schedule_state() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        app_state.entitlement.clear();
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

        let outcome = dispatch_entitlement_session_tick_driver_with_control_plane(&mut runtime)
            .expect("expected session tick");

        let preflight = outcome.tick.preflight.expect("expected preflight");
        assert_eq!(
            preflight.decision.action,
            EntitlementPreflightAction::SyncEntitlement
        );
        assert_eq!(
            outcome.state.session_state.last_completed_at,
            Some(timestamp(200))
        );
        assert_eq!(outcome.state.schedule.next_check_at, Some(timestamp(1_100)));
    }

    #[test]
    fn session_started_event_runs_preflight_tick() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        app_state.entitlement.clear();
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

        let outcome = dispatch_entitlement_session_event_with_control_plane(
            EntitlementSessionEvent::SessionStarted,
            &mut runtime,
        )
        .expect("expected session started event dispatch");

        assert_eq!(outcome.event, EntitlementSessionEvent::SessionStarted);
        match outcome.outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick.preflight.expect("expected preflight");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::SyncEntitlement
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert_eq!(
            outcome.state.session_state.last_completed_at,
            Some(timestamp(200))
        );
    }

    #[test]
    fn entitlement_command_completed_event_records_session_state_without_tick() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot(210),
            vec![sample_manifest()],
            timestamp(150),
        );
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let mut session_state = EntitlementSessionState::default();
        let action_outcome = crate::StudioEntitlementActionOutcome {
            action: StudioEntitlementAction::RefreshOfflineLease,
            outcome: StudioEntitlementOutcome::OfflineLeaseRefreshed,
            entitlement_status: rf_ui::EntitlementStatus::Active,
            last_synced_at: Some(timestamp(210)),
            last_error: None,
            notice: None,
            latest_log_entry: None,
        };
        let policy = EntitlementSessionPolicy::default();
        let mut runtime = EntitlementSessionRuntime {
            facade: &facade,
            app_state: &mut app_state,
            context: &mut context,
            control_plane_client: &client,
            access_token: "access-token",
            now: timestamp(220),
            policy: &policy,
            session_state: &mut session_state,
        };

        let outcome = dispatch_entitlement_session_event_with_control_plane(
            EntitlementSessionEvent::EntitlementCommandCompleted(action_outcome),
            &mut runtime,
        )
        .expect("expected command completed event dispatch");

        match outcome.outcome {
            EntitlementSessionEventOutcome::RecordedCommand { action } => {
                assert_eq!(action, StudioEntitlementAction::RefreshOfflineLease);
            }
            other => panic!("expected recorded command outcome, got {other:?}"),
        }
        assert_eq!(
            outcome.state.session_state.last_completed_action,
            Some(EntitlementPreflightAction::RefreshOfflineLease)
        );
        assert_eq!(
            outcome.state.session_state.last_completed_at,
            Some(timestamp(220))
        );
    }

    #[test]
    fn session_panel_primary_action_records_manual_result_into_session_state() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot(210),
            vec![sample_manifest()],
            timestamp(150),
        );
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
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

        let outcome =
            dispatch_entitlement_session_panel_primary_action_with_control_plane(&mut runtime)
                .expect("expected panel primary action");

        match outcome.dispatch {
            EntitlementPanelWidgetDispatchOutcome::Executed(outcome) => match outcome.dispatch {
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
            },
            other => panic!("expected executed dispatch, got {other:?}"),
        }
        assert_eq!(session_state.last_completed_at, Some(timestamp(200)));
        assert_eq!(
            session_state.last_completed_action,
            Some(EntitlementPreflightAction::RefreshOfflineLease)
        );
        assert_eq!(
            outcome.state.panel.widget.view().primary_action.label,
            "Refresh offline lease"
        );
    }

    #[test]
    fn session_panel_disabled_action_keeps_schedule_snapshot() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
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

        let outcome = crate::dispatch_entitlement_session_panel_widget_action_with_control_plane(
            EntitlementActionId::SyncEntitlement,
            &mut runtime,
        )
        .expect("expected disabled session panel dispatch");

        assert_eq!(
            outcome.dispatch,
            EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled {
                action_id: EntitlementActionId::SyncEntitlement,
            }
        );
        assert_eq!(
            outcome.state.session_state,
            EntitlementSessionState::default()
        );
        assert_eq!(outcome.state.schedule.next_check_at, None);
    }
}
