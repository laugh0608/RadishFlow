use std::time::{Duration, SystemTime};

use rf_types::RfResult;
use rf_ui::{EntitlementActionId, EntitlementNotice, EntitlementNoticeLevel};

use crate::{
    EntitlementPanelDriverState,
    EntitlementSessionDriverState, EntitlementSessionEvent,
    EntitlementSessionEventDriverOutcome, EntitlementSessionPanelDriverOutcome,
    EntitlementSessionRuntime, RadishFlowControlPlaneClient,
    snapshot_entitlement_panel_driver_state,
    snapshot_entitlement_session_driver_state,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementSessionTimerReason {
    ImmediateCheck,
    ScheduledCheck,
    BackoffRetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionTimerArm {
    pub event: EntitlementSessionLifecycleEvent,
    pub due_at: SystemTime,
    pub delay: Duration,
    pub reason: EntitlementSessionTimerReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionTimerCommand {
    Keep {
        timer: EntitlementSessionTimerArm,
    },
    Schedule {
        timer: EntitlementSessionTimerArm,
    },
    Reschedule {
        previous: EntitlementSessionTimerArm,
        next: EntitlementSessionTimerArm,
    },
    Clear {
        previous: EntitlementSessionTimerArm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostState {
    pub driver: EntitlementSessionDriverState,
    pub next_timer: Option<EntitlementSessionTimerArm>,
    pub host_notice: Option<EntitlementNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostOutcome {
    pub trigger: EntitlementSessionHostTrigger,
    pub dispatch: EntitlementSessionHostDispatch,
    pub state: EntitlementSessionHostState,
}

pub fn snapshot_entitlement_session_host_state(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
) -> EntitlementSessionHostState {
    let driver = snapshot_entitlement_session_driver_state(app_state, now, policy, session_state);
    let next_timer = timer_arm_from_schedule(&driver.schedule, now);
    let host_notice = host_notice_from_schedule(&driver.schedule, next_timer.as_ref());

    EntitlementSessionHostState {
        driver,
        next_timer,
        host_notice,
    }
}

pub fn plan_entitlement_session_timer_command(
    current: Option<&EntitlementSessionTimerArm>,
    next: Option<&EntitlementSessionTimerArm>,
) -> Option<EntitlementSessionTimerCommand> {
    match (current, next) {
        (Some(current), Some(next)) if current == next => Some(EntitlementSessionTimerCommand::Keep {
            timer: current.clone(),
        }),
        (None, Some(next)) => Some(EntitlementSessionTimerCommand::Schedule {
            timer: next.clone(),
        }),
        (Some(current), Some(next)) => Some(EntitlementSessionTimerCommand::Reschedule {
            previous: current.clone(),
            next: next.clone(),
        }),
        (Some(current), None) => Some(EntitlementSessionTimerCommand::Clear {
            previous: current.clone(),
        }),
        (None, None) => None,
    }
}

pub fn snapshot_entitlement_session_panel_driver_state_with_host_notice(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
) -> EntitlementPanelDriverState {
    let host_state = snapshot_entitlement_session_host_state(app_state, now, policy, session_state);
    let mut panel = snapshot_entitlement_panel_driver_state(app_state);
    if panel.panel_state.notice.is_none() {
        panel.panel_state.notice = host_state.host_notice;
        panel.widget = rf_ui::EntitlementPanelWidgetModel::from_state(&panel.panel_state);
    }
    panel
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
    let state = snapshot_entitlement_session_host_state(
        runtime.app_state,
        runtime.now,
        runtime.policy,
        runtime.session_state,
    );

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

fn timer_arm_from_schedule(
    schedule: &crate::EntitlementSessionSchedule,
    now: SystemTime,
) -> Option<EntitlementSessionTimerArm> {
    let due_at = schedule.next_check_at?;
    let delay = due_at.duration_since(now).unwrap_or(Duration::ZERO);
    let reason = if schedule.blocked_by_backoff {
        EntitlementSessionTimerReason::BackoffRetry
    } else if schedule.recommended_action.is_some() && delay == Duration::ZERO {
        EntitlementSessionTimerReason::ImmediateCheck
    } else {
        EntitlementSessionTimerReason::ScheduledCheck
    };

    Some(EntitlementSessionTimerArm {
        event: EntitlementSessionLifecycleEvent::TimerElapsed,
        due_at,
        delay,
        reason,
    })
}

fn host_notice_from_schedule(
    schedule: &crate::EntitlementSessionSchedule,
    timer: Option<&EntitlementSessionTimerArm>,
) -> Option<EntitlementNotice> {
    if schedule.blocked_by_backoff {
        let backoff = schedule.backoff.as_ref()?;
        return Some(EntitlementNotice::new(
            EntitlementNoticeLevel::Warning,
            "Automatic retry scheduled",
            format!(
                "entitlement session will retry {:?} after backoff at {:?}",
                backoff.action, backoff.retry_not_before
            ),
        ));
    }

    let timer = timer?;
    if timer.reason == EntitlementSessionTimerReason::ScheduledCheck {
        return Some(EntitlementNotice::new(
            EntitlementNoticeLevel::Info,
            "Automatic check scheduled",
            format!(
                "entitlement session will trigger {:?} at {:?}",
                timer.event, timer.due_at
            ),
        ));
    }

    None
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
        EntitlementSessionTimerArm, EntitlementSessionTimerCommand,
        EntitlementSessionTimerReason,
        EntitlementSessionLifecycleEvent,
        dispatch_entitlement_session_host_trigger_with_control_plane,
        dispatch_entitlement_session_lifecycle_event_with_control_plane,
        plan_entitlement_session_timer_command,
        snapshot_entitlement_session_panel_driver_state_with_host_notice,
        snapshot_entitlement_session_host_state,
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
        assert_eq!(
            outcome.state.next_timer.as_ref().map(|timer| timer.reason),
            Some(EntitlementSessionTimerReason::ImmediateCheck)
        );
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
        assert_eq!(
            outcome.state.next_timer.as_ref().map(|timer| timer.reason),
            Some(EntitlementSessionTimerReason::ImmediateCheck)
        );
    }

    #[test]
    fn snapshot_host_state_marks_immediate_timer_when_check_is_due_now() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.entitlement.clear();

        let state = snapshot_entitlement_session_host_state(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        let timer = state.next_timer.expect("expected timer arm");
        assert_eq!(timer.delay, Duration::ZERO);
        assert_eq!(timer.reason, EntitlementSessionTimerReason::ImmediateCheck);
        assert_eq!(timer.event, EntitlementSessionLifecycleEvent::TimerElapsed);
        assert!(state.host_notice.is_none());
    }

    #[test]
    fn snapshot_host_state_marks_backoff_retry_when_scheduler_is_blocked() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(sample_snapshot(210), vec![sample_manifest()], timestamp(150));
        let policy = EntitlementSessionPolicy::default();
        let session_state = EntitlementSessionState {
            backoff: Some(crate::EntitlementSessionBackoff {
                action: EntitlementPreflightAction::RefreshOfflineLease,
                failure_reason: crate::StudioEntitlementFailureReason::ConnectionUnavailable,
                consecutive_failures: 1,
                retry_not_before: timestamp(260),
            }),
            ..EntitlementSessionState::default()
        };

        let state = snapshot_entitlement_session_host_state(
            &app_state,
            timestamp(200),
            &policy,
            &session_state,
        );

        let timer = state.next_timer.expect("expected timer arm");
        assert_eq!(timer.delay, Duration::from_secs(60));
        assert_eq!(timer.reason, EntitlementSessionTimerReason::BackoffRetry);
        assert_eq!(
            state.host_notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Automatic retry scheduled")
        );
    }

    #[test]
    fn snapshot_host_state_exposes_scheduled_check_notice() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(5_000, 9_000),
            vec![sample_manifest()],
            timestamp(150),
        );

        let state = snapshot_entitlement_session_host_state(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        let timer = state.next_timer.expect("expected timer arm");
        assert_eq!(timer.reason, EntitlementSessionTimerReason::ScheduledCheck);
        assert_eq!(
            state.host_notice.as_ref().map(|notice| notice.title.as_str()),
            Some("Automatic check scheduled")
        );
    }

    #[test]
    fn panel_driver_uses_host_notice_when_runtime_notice_is_absent() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(5_000, 9_000),
            vec![sample_manifest()],
            timestamp(150),
        );

        let panel = snapshot_entitlement_session_panel_driver_state_with_host_notice(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        assert_eq!(
            panel
                .panel_state
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Automatic check scheduled")
        );
    }

    #[test]
    fn panel_driver_preserves_runtime_notice_over_host_notice() {
        let mut app_state = AppState::new(sample_document());
        complete_login(&mut app_state);
        app_state.update_entitlement(
            sample_snapshot_with_expiry(5_000, 9_000),
            vec![sample_manifest()],
            timestamp(150),
        );
        app_state.entitlement.set_notice(rf_ui::EntitlementNotice::new(
            rf_ui::EntitlementNoticeLevel::Info,
            "Runtime notice",
            "runtime notice should win",
        ));

        let panel = snapshot_entitlement_session_panel_driver_state_with_host_notice(
            &app_state,
            timestamp(200),
            &EntitlementSessionPolicy::default(),
            &EntitlementSessionState::default(),
        );

        assert_eq!(
            panel
                .panel_state
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Runtime notice")
        );
    }

    #[test]
    fn timer_command_schedules_when_previous_timer_is_missing() {
        let next = EntitlementSessionTimerArm {
            event: EntitlementSessionLifecycleEvent::TimerElapsed,
            due_at: timestamp(260),
            delay: Duration::from_secs(60),
            reason: EntitlementSessionTimerReason::ScheduledCheck,
        };

        let command = plan_entitlement_session_timer_command(None, Some(&next));

        assert_eq!(
            command,
            Some(EntitlementSessionTimerCommand::Schedule { timer: next })
        );
    }

    #[test]
    fn timer_command_keeps_when_timer_is_unchanged() {
        let timer = EntitlementSessionTimerArm {
            event: EntitlementSessionLifecycleEvent::TimerElapsed,
            due_at: timestamp(260),
            delay: Duration::from_secs(60),
            reason: EntitlementSessionTimerReason::ScheduledCheck,
        };

        let command = plan_entitlement_session_timer_command(Some(&timer), Some(&timer));

        assert_eq!(
            command,
            Some(EntitlementSessionTimerCommand::Keep { timer })
        );
    }

    #[test]
    fn timer_command_reschedules_when_due_time_changes() {
        let previous = EntitlementSessionTimerArm {
            event: EntitlementSessionLifecycleEvent::TimerElapsed,
            due_at: timestamp(260),
            delay: Duration::from_secs(60),
            reason: EntitlementSessionTimerReason::ScheduledCheck,
        };
        let next = EntitlementSessionTimerArm {
            event: EntitlementSessionLifecycleEvent::TimerElapsed,
            due_at: timestamp(800),
            delay: Duration::from_secs(600),
            reason: EntitlementSessionTimerReason::BackoffRetry,
        };

        let command = plan_entitlement_session_timer_command(Some(&previous), Some(&next));

        assert_eq!(
            command,
            Some(EntitlementSessionTimerCommand::Reschedule { previous, next })
        );
    }

    #[test]
    fn timer_command_clears_when_timer_is_no_longer_needed() {
        let previous = EntitlementSessionTimerArm {
            event: EntitlementSessionLifecycleEvent::TimerElapsed,
            due_at: timestamp(260),
            delay: Duration::from_secs(60),
            reason: EntitlementSessionTimerReason::ScheduledCheck,
        };

        let command = plan_entitlement_session_timer_command(Some(&previous), None);

        assert_eq!(
            command,
            Some(EntitlementSessionTimerCommand::Clear { previous })
        );
    }
}
