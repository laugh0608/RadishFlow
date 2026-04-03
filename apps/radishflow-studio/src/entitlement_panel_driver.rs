use rf_types::RfResult;
use rf_ui::{
    AppState, EntitlementActionId, EntitlementIntent, EntitlementPanelState,
    EntitlementPanelWidgetEvent, EntitlementPanelWidgetModel,
};

use crate::{
    RadishFlowControlPlaneClient, StudioAppCommand, StudioAppCommandOutcome, StudioAppFacade,
    StudioAppMutableAuthCacheContext,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelDriverState {
    pub widget: EntitlementPanelWidgetModel,
    pub panel_state: EntitlementPanelState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelDriverOutcome {
    pub dispatch: EntitlementPanelWidgetDispatchOutcome,
    pub state: EntitlementPanelDriverState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementPanelWidgetDispatchOutcome {
    Executed(StudioAppCommandOutcome),
    IgnoredDisabled { action_id: EntitlementActionId },
    IgnoredMissing { action_id: EntitlementActionId },
}

pub fn snapshot_entitlement_panel_driver_state(
    app_state: &AppState,
) -> EntitlementPanelDriverState {
    let panel_state =
        EntitlementPanelState::from_runtime(&app_state.auth_session, &app_state.entitlement);
    let widget = EntitlementPanelWidgetModel::from_state(&panel_state);

    EntitlementPanelDriverState {
        widget,
        panel_state,
    }
}

pub fn map_entitlement_intent_to_app_command(intent: &EntitlementIntent) -> StudioAppCommand {
    match intent {
        EntitlementIntent::SyncEntitlement => StudioAppCommand::sync_entitlement(),
        EntitlementIntent::RefreshOfflineLease => StudioAppCommand::refresh_offline_lease(),
    }
}

pub fn dispatch_entitlement_panel_intent_with_control_plane<Client>(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &mut StudioAppMutableAuthCacheContext<'_>,
    control_plane_client: &Client,
    access_token: &str,
    intent: &EntitlementIntent,
) -> RfResult<StudioAppCommandOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let command = map_entitlement_intent_to_app_command(intent);
    facade.execute_with_control_plane(
        app_state,
        context,
        control_plane_client,
        access_token,
        &command,
    )
}

pub fn dispatch_entitlement_panel_widget_event_with_control_plane<Client>(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &mut StudioAppMutableAuthCacheContext<'_>,
    control_plane_client: &Client,
    access_token: &str,
    event: &EntitlementPanelWidgetEvent,
) -> RfResult<EntitlementPanelWidgetDispatchOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    match event {
        EntitlementPanelWidgetEvent::Dispatched { intent, .. } => {
            dispatch_entitlement_panel_intent_with_control_plane(
                facade,
                app_state,
                context,
                control_plane_client,
                access_token,
                intent,
            )
            .map(EntitlementPanelWidgetDispatchOutcome::Executed)
        }
        EntitlementPanelWidgetEvent::Disabled { action_id } => {
            Ok(EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled {
                action_id: *action_id,
            })
        }
        EntitlementPanelWidgetEvent::Missing { action_id } => {
            Ok(EntitlementPanelWidgetDispatchOutcome::IgnoredMissing {
                action_id: *action_id,
            })
        }
    }
}

pub fn dispatch_entitlement_panel_widget_action_with_control_plane<Client>(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &mut StudioAppMutableAuthCacheContext<'_>,
    control_plane_client: &Client,
    access_token: &str,
    action_id: EntitlementActionId,
) -> RfResult<EntitlementPanelDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let widget = EntitlementPanelWidgetModel::from_state(&EntitlementPanelState::from_runtime(
        &app_state.auth_session,
        &app_state.entitlement,
    ));
    let dispatch = dispatch_entitlement_panel_widget_event_with_control_plane(
        facade,
        app_state,
        context,
        control_plane_client,
        access_token,
        &widget.activate(action_id),
    )?;
    let state = snapshot_entitlement_panel_driver_state(app_state);

    Ok(EntitlementPanelDriverOutcome { dispatch, state })
}

pub fn dispatch_entitlement_panel_primary_action_with_control_plane<Client>(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &mut StudioAppMutableAuthCacheContext<'_>,
    control_plane_client: &Client,
    access_token: &str,
) -> RfResult<EntitlementPanelDriverOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let widget = EntitlementPanelWidgetModel::from_state(&EntitlementPanelState::from_runtime(
        &app_state.auth_session,
        &app_state.entitlement,
    ));
    let dispatch = dispatch_entitlement_panel_widget_event_with_control_plane(
        facade,
        app_state,
        context,
        control_plane_client,
        access_token,
        &widget.activate_primary(),
    )?;
    let state = snapshot_entitlement_panel_driver_state(app_state);

    Ok(EntitlementPanelDriverOutcome { dispatch, state })
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
        EntitlementPanelDriverOutcome, EntitlementPanelWidgetDispatchOutcome,
        dispatch_entitlement_panel_primary_action_with_control_plane,
        dispatch_entitlement_panel_widget_action_with_control_plane,
        snapshot_entitlement_panel_driver_state,
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
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new(
            "doc-entitlement-driver",
            "Entitlement Driver",
            timestamp(10),
        );
        FlowsheetDocument::new(flowsheet, metadata)
    }

    fn sample_snapshot() -> EntitlementSnapshot {
        EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(400),
            offline_lease_expires_at: Some(timestamp(700)),
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
            snapshot: sample_snapshot(),
            manifest_list: sample_manifest_list(),
        }
    }

    fn sample_auth_cache_index() -> StoredAuthCacheIndex {
        let snapshot = sample_snapshot();
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
                sample_snapshot(),
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
                "entitlement panel driver tests do not request leases",
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
    fn snapshot_entitlement_panel_driver_state_builds_widget_and_panel_state() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(), vec![sample_manifest()], timestamp(150));

        let state = snapshot_entitlement_panel_driver_state(&app_state);

        assert_eq!(
            state.widget.view().primary_action.label,
            "Refresh offline lease"
        );
        assert_eq!(state.panel_state.allowed_package_count, 1);
    }

    #[test]
    fn dispatching_primary_action_through_driver_executes_offline_refresh() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(), vec![sample_manifest()], timestamp(150));
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);

        let outcome = dispatch_entitlement_panel_primary_action_with_control_plane(
            &facade,
            &mut app_state,
            &mut context,
            &client,
            "access-token",
        )
        .expect("expected primary entitlement dispatch");

        match outcome {
            EntitlementPanelDriverOutcome {
                dispatch: EntitlementPanelWidgetDispatchOutcome::Executed(outcome),
                state,
            } => {
                match outcome.dispatch {
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
                assert_eq!(
                    state.widget.view().primary_action.label,
                    "Refresh offline lease"
                );
            }
            other => panic!("expected executed entitlement driver outcome, got {other:?}"),
        }
    }

    #[test]
    fn dispatching_disabled_action_through_driver_returns_ignored_state() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        let mut context = StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);

        let outcome = dispatch_entitlement_panel_widget_action_with_control_plane(
            &facade,
            &mut app_state,
            &mut context,
            &client,
            "access-token",
            EntitlementActionId::SyncEntitlement,
        )
        .expect("expected disabled entitlement driver outcome");

        match outcome {
            EntitlementPanelDriverOutcome {
                dispatch:
                    EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled {
                        action_id: EntitlementActionId::SyncEntitlement,
                    },
                ..
            } => {}
            other => panic!("expected ignored disabled outcome, got {other:?}"),
        }
    }
}
