use rf_types::RfResult;
use rf_ui::EntitlementActionId;

use crate::{
    EntitlementSessionDriverState, EntitlementSessionEvent,
    EntitlementSessionEventDriverOutcome, EntitlementSessionPanelDriverOutcome,
    EntitlementSessionRuntime, RadishFlowControlPlaneClient,
    dispatch_entitlement_session_event_with_control_plane,
    dispatch_entitlement_session_panel_primary_action_with_control_plane,
    dispatch_entitlement_session_panel_widget_action_with_control_plane,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementSessionLifecycleEvent {
    SessionStarted,
    LoginCompleted,
    TimerElapsed,
    NetworkRestored,
    WindowForegrounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionHostTrigger {
    LifecycleEvent(EntitlementSessionLifecycleEvent),
    EntitlementCommandCompleted(crate::StudioEntitlementActionOutcome),
    PanelPrimaryAction,
    PanelAction(EntitlementActionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionHostDispatch {
    Event(EntitlementSessionEventDriverOutcome),
    Panel(EntitlementSessionPanelDriverOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostOutcome {
    pub trigger: EntitlementSessionHostTrigger,
    pub dispatch: EntitlementSessionHostDispatch,
    pub state: EntitlementSessionDriverState,
}

pub fn dispatch_entitlement_session_host_trigger_with_control_plane<Client>(
    trigger: EntitlementSessionHostTrigger,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let dispatch = match &trigger {
        EntitlementSessionHostTrigger::LifecycleEvent(event) => EntitlementSessionHostDispatch::Event(
            dispatch_entitlement_session_event_with_control_plane(
                map_lifecycle_event_to_session_event(*event),
                runtime,
            )?,
        ),
        EntitlementSessionHostTrigger::EntitlementCommandCompleted(outcome) => {
            EntitlementSessionHostDispatch::Event(
                dispatch_entitlement_session_event_with_control_plane(
                    EntitlementSessionEvent::EntitlementCommandCompleted(outcome.clone()),
                    runtime,
                )?,
            )
        }
        EntitlementSessionHostTrigger::PanelPrimaryAction => EntitlementSessionHostDispatch::Panel(
            dispatch_entitlement_session_panel_primary_action_with_control_plane(runtime)?,
        ),
        EntitlementSessionHostTrigger::PanelAction(action_id) => EntitlementSessionHostDispatch::Panel(
            dispatch_entitlement_session_panel_widget_action_with_control_plane(
                *action_id,
                runtime,
            )?,
        ),
    };
    let state = match &dispatch {
        EntitlementSessionHostDispatch::Event(outcome) => outcome.state.clone(),
        EntitlementSessionHostDispatch::Panel(outcome) => outcome.state.clone(),
    };

    Ok(EntitlementSessionHostOutcome {
        trigger,
        dispatch,
        state,
    })
}

pub fn dispatch_entitlement_session_lifecycle_event_with_control_plane<Client>(
    event: EntitlementSessionLifecycleEvent,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    dispatch_entitlement_session_host_trigger_with_control_plane(
        EntitlementSessionHostTrigger::LifecycleEvent(event),
        runtime,
    )
}

fn map_lifecycle_event_to_session_event(event: EntitlementSessionLifecycleEvent) -> EntitlementSessionEvent {
    match event {
        EntitlementSessionLifecycleEvent::SessionStarted => EntitlementSessionEvent::SessionStarted,
        EntitlementSessionLifecycleEvent::LoginCompleted => {
            EntitlementSessionEvent::LoginCompleted
        }
        EntitlementSessionLifecycleEvent::TimerElapsed
        | EntitlementSessionLifecycleEvent::NetworkRestored
        | EntitlementSessionLifecycleEvent::WindowForegrounded => {
            EntitlementSessionEvent::TimerElapsed
        }
    }
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
        EntitlementSessionHostDispatch, EntitlementSessionHostTrigger,
        EntitlementSessionLifecycleEvent,
        dispatch_entitlement_session_host_trigger_with_control_plane,
        dispatch_entitlement_session_lifecycle_event_with_control_plane,
    };
    use crate::{
        EntitlementPreflightAction, EntitlementSessionEventOutcome, EntitlementSessionPolicy,
        EntitlementSessionRuntime, EntitlementSessionState, RadishFlowControlPlaneClient,
        RadishFlowControlPlaneClientError, RadishFlowControlPlaneClientErrorKind,
        RadishFlowControlPlaneResponse, StudioAppFacade, StudioAppMutableAuthCacheContext,
        StudioAppResultDispatch, StudioEntitlementAction, StudioEntitlementOutcome,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-session-host", "Session Host", timestamp(10));
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
                "session host tests do not request leases",
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
    fn host_dispatches_login_completed_as_session_tick() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.entitlement.clear();
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

        let outcome = dispatch_entitlement_session_host_trigger_with_control_plane(
            EntitlementSessionHostTrigger::LifecycleEvent(
                EntitlementSessionLifecycleEvent::LoginCompleted,
            ),
            &mut runtime,
        )
        .expect("expected session host login event");

        match outcome.dispatch {
            EntitlementSessionHostDispatch::Event(event) => match event.outcome {
                EntitlementSessionEventOutcome::Tick(tick) => {
                    let preflight = tick.preflight.expect("expected preflight");
                    assert_eq!(
                        preflight.decision.action,
                        EntitlementPreflightAction::SyncEntitlement
                    );
                }
                other => panic!("expected tick outcome, got {other:?}"),
            },
            other => panic!("expected event dispatch, got {other:?}"),
        }
    }

    #[test]
    fn host_dispatches_panel_primary_action_through_panel_driver() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(210), vec![sample_manifest()], timestamp(150));
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

        let outcome = dispatch_entitlement_session_host_trigger_with_control_plane(
            EntitlementSessionHostTrigger::PanelPrimaryAction,
            &mut runtime,
        )
        .expect("expected session host panel action");

        match outcome.dispatch {
            EntitlementSessionHostDispatch::Panel(panel) => match panel.dispatch {
                crate::EntitlementPanelWidgetDispatchOutcome::Executed(command) => {
                    match command.dispatch {
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
                other => panic!("expected executed panel dispatch, got {other:?}"),
            },
            other => panic!("expected panel dispatch, got {other:?}"),
        }
    }

    #[test]
    fn lifecycle_network_restored_reuses_timer_elapsed_path() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(210), vec![sample_manifest()], timestamp(150));
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

        let outcome = dispatch_entitlement_session_lifecycle_event_with_control_plane(
            EntitlementSessionLifecycleEvent::NetworkRestored,
            &mut runtime,
        )
        .expect("expected network restored lifecycle event");

        match outcome.dispatch {
            EntitlementSessionHostDispatch::Event(event) => match event.outcome {
                EntitlementSessionEventOutcome::Tick(tick) => {
                    let preflight = tick.preflight.expect("expected preflight");
                    assert_eq!(
                        preflight.decision.action,
                        EntitlementPreflightAction::RefreshOfflineLease
                    );
                }
                other => panic!("expected tick outcome, got {other:?}"),
            },
            other => panic!("expected event dispatch, got {other:?}"),
        }
    }
}
