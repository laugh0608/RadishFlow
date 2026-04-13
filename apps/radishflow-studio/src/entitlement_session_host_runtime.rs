use rf_types::RfResult;

use crate::{
    EntitlementSessionHostContext, EntitlementSessionHostDispatch, EntitlementSessionHostOutcome,
    EntitlementSessionHostPresentation, EntitlementSessionHostSnapshot,
    EntitlementSessionHostTrigger, EntitlementSessionPolicy, EntitlementSessionRuntime,
    EntitlementSessionState, EntitlementSessionTimerArm, EntitlementSessionTimerCommand,
    RadishFlowControlPlaneClient,
    dispatch_entitlement_session_host_trigger_with_context_and_control_plane,
    snapshot_entitlement_session_host_with_context,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionHostTimerEffect {
    KeepTimer {
        timer: EntitlementSessionTimerArm,
    },
    ArmTimer {
        timer: EntitlementSessionTimerArm,
    },
    RearmTimer {
        previous: EntitlementSessionTimerArm,
        next: EntitlementSessionTimerArm,
    },
    ClearTimer {
        previous: EntitlementSessionTimerArm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostRuntimeOutput {
    pub snapshot: EntitlementSessionHostSnapshot,
    pub presentation: EntitlementSessionHostPresentation,
    pub timer_effect: Option<EntitlementSessionHostTimerEffect>,
}

impl EntitlementSessionHostRuntimeOutput {
    pub fn from_snapshot(snapshot: EntitlementSessionHostSnapshot) -> Self {
        let presentation = snapshot.presentation();
        let timer_effect = snapshot.timer_effect();
        Self {
            snapshot,
            presentation,
            timer_effect,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostRuntimeDispatchOutcome {
    pub trigger: EntitlementSessionHostTrigger,
    pub dispatch: EntitlementSessionHostDispatch,
    pub output: EntitlementSessionHostRuntimeOutput,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntitlementSessionHostRuntime {
    context: EntitlementSessionHostContext,
}

impl EntitlementSessionHostRuntime {
    pub fn current_timer(&self) -> Option<&EntitlementSessionTimerArm> {
        self.context.current_timer()
    }

    pub fn last_snapshot(&self) -> Option<&EntitlementSessionHostSnapshot> {
        self.context.last_snapshot()
    }

    pub fn last_output(&self) -> Option<EntitlementSessionHostRuntimeOutput> {
        self.last_snapshot()
            .cloned()
            .map(EntitlementSessionHostRuntimeOutput::from_snapshot)
    }

    pub fn snapshot(
        &mut self,
        app_state: &rf_ui::AppState,
        now: std::time::SystemTime,
        policy: &EntitlementSessionPolicy,
        session_state: &EntitlementSessionState,
    ) -> EntitlementSessionHostRuntimeOutput {
        EntitlementSessionHostRuntimeOutput::from_snapshot(
            snapshot_entitlement_session_host_with_context(
                app_state,
                now,
                policy,
                session_state,
                &mut self.context,
            ),
        )
    }

    pub fn dispatch_trigger_with_control_plane<Client>(
        &mut self,
        trigger: EntitlementSessionHostTrigger,
        runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
    ) -> RfResult<EntitlementSessionHostRuntimeDispatchOutcome>
    where
        Client: RadishFlowControlPlaneClient,
    {
        let outcome = dispatch_entitlement_session_host_trigger_with_context_and_control_plane(
            trigger,
            &mut self.context,
            runtime,
        )?;
        Ok(EntitlementSessionHostRuntimeDispatchOutcome::from_host_outcome(outcome))
    }
}

impl EntitlementSessionHostRuntimeDispatchOutcome {
    fn from_host_outcome(outcome: EntitlementSessionHostOutcome) -> Self {
        let EntitlementSessionHostOutcome {
            trigger,
            dispatch,
            snapshot,
        } = outcome;

        Self {
            trigger,
            dispatch,
            output: EntitlementSessionHostRuntimeOutput::from_snapshot(snapshot),
        }
    }
}

impl EntitlementSessionHostSnapshot {
    pub fn timer_effect(&self) -> Option<EntitlementSessionHostTimerEffect> {
        self.timer_command.as_ref().map(timer_effect_from_command)
    }
}

fn timer_effect_from_command(
    command: &EntitlementSessionTimerCommand,
) -> EntitlementSessionHostTimerEffect {
    match command {
        EntitlementSessionTimerCommand::Keep { timer } => {
            EntitlementSessionHostTimerEffect::KeepTimer {
                timer: timer.clone(),
            }
        }
        EntitlementSessionTimerCommand::Schedule { timer } => {
            EntitlementSessionHostTimerEffect::ArmTimer {
                timer: timer.clone(),
            }
        }
        EntitlementSessionTimerCommand::Reschedule { previous, next } => {
            EntitlementSessionHostTimerEffect::RearmTimer {
                previous: previous.clone(),
                next: next.clone(),
            }
        }
        EntitlementSessionTimerCommand::Clear { previous } => {
            EntitlementSessionHostTimerEffect::ClearTimer {
                previous: previous.clone(),
            }
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

    use crate::{
        EntitlementPreflightAction, EntitlementSessionHostRuntime,
        EntitlementSessionHostTimerEffect, EntitlementSessionLifecycleEvent,
        EntitlementSessionPolicy, EntitlementSessionRuntime, EntitlementSessionState,
        EntitlementSessionTimerReason, RadishFlowControlPlaneClient,
        RadishFlowControlPlaneClientError, RadishFlowControlPlaneClientErrorKind,
        RadishFlowControlPlaneResponse, StudioAppFacade, StudioAppMutableAuthCacheContext,
        StudioEntitlementFailureReason,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new(
            "doc-session-host-runtime",
            "Session Host Runtime",
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
                "session host runtime tests do not request leases",
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
    fn runtime_snapshot_emits_arm_timer_effect() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(5_000, 9_000),
            vec![sample_manifest()],
            timestamp(150),
        );
        let mut runtime = EntitlementSessionHostRuntime::default();

        let output = runtime.snapshot(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        match output.timer_effect {
            Some(EntitlementSessionHostTimerEffect::ArmTimer { timer }) => {
                assert_eq!(timer.reason, EntitlementSessionTimerReason::ScheduledCheck);
            }
            other => panic!("expected arm timer effect, got {other:?}"),
        }
        assert_eq!(
            runtime.current_timer().map(|timer| timer.reason),
            Some(EntitlementSessionTimerReason::ScheduledCheck)
        );
    }

    #[test]
    fn runtime_dispatch_emits_keep_timer_effect_when_schedule_is_unchanged() {
        let facade = StudioAppFacade::new();
        let client = ScriptedControlPlaneClient;
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(5_000, 9_000),
            vec![sample_manifest()],
            timestamp(150),
        );
        let mut host_runtime = EntitlementSessionHostRuntime::default();
        let _ = host_runtime.snapshot(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );
        let cache_root = PathBuf::from("D:\\cache-root");
        let mut auth_cache_index = sample_auth_cache_index();
        let mut auth_context =
            StudioAppMutableAuthCacheContext::new(&cache_root, &mut auth_cache_index);
        let mut session_state = EntitlementSessionState::default();
        let policy = EntitlementSessionPolicy::default();
        let mut runtime = EntitlementSessionRuntime {
            facade: &facade,
            app_state: &mut app_state,
            context: &mut auth_context,
            control_plane_client: &client,
            access_token: "access-token",
            now: timestamp(200),
            policy: &policy,
            session_state: &mut session_state,
        };

        let output = host_runtime
            .dispatch_trigger_with_control_plane(
                crate::EntitlementSessionHostTrigger::LifecycleEvent(
                    EntitlementSessionLifecycleEvent::WindowForegrounded,
                ),
                &mut runtime,
            )
            .expect("expected runtime dispatch");

        match output.output.timer_effect {
            Some(EntitlementSessionHostTimerEffect::KeepTimer { timer }) => {
                assert_eq!(timer.reason, EntitlementSessionTimerReason::ScheduledCheck);
            }
            other => panic!("expected keep timer effect, got {other:?}"),
        }
    }

    #[test]
    fn runtime_snapshot_emits_rearm_timer_effect_for_backoff_transition() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot(210),
            vec![sample_manifest()],
            timestamp(150),
        );
        let mut host_runtime = EntitlementSessionHostRuntime::default();
        let _ = host_runtime.snapshot(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );
        let session_state = EntitlementSessionState {
            backoff: Some(crate::EntitlementSessionBackoff {
                action: EntitlementPreflightAction::RefreshOfflineLease,
                failure_reason: StudioEntitlementFailureReason::ConnectionUnavailable,
                consecutive_failures: 1,
                retry_not_before: timestamp(260),
            }),
            ..EntitlementSessionState::default()
        };

        let output = host_runtime.snapshot(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &session_state,
        );

        match output.timer_effect {
            Some(EntitlementSessionHostTimerEffect::RearmTimer { previous, next }) => {
                assert_eq!(
                    previous.reason,
                    EntitlementSessionTimerReason::ImmediateCheck
                );
                assert_eq!(next.reason, EntitlementSessionTimerReason::BackoffRetry);
            }
            other => panic!("expected rearm timer effect, got {other:?}"),
        }
    }
}
