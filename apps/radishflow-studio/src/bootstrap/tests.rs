use std::path::PathBuf;

use rf_ui::{
    EntitlementActionId, RunPanelActionId, RunPanelIntent, RunPanelPackageSelection, RunStatus,
    SimulationMode,
};

use super::{
    BootstrapSession, StudioBootstrapConfig, StudioBootstrapDispatch,
    StudioBootstrapEntitlementSeed, StudioBootstrapEntitlementSessionEvent, StudioBootstrapTrigger,
    run_studio_bootstrap,
};
use crate::{
    EntitlementPreflightAction, EntitlementSessionEvent, EntitlementSessionEventOutcome,
    EntitlementSessionHostTimerEffect, StudioAppCommand, StudioAppExecutionBoundary,
    StudioAppExecutionLane, StudioAppResultDispatch, StudioEntitlementAction,
    StudioEntitlementOutcome, StudioRuntime, StudioWorkspaceRunOutcome, WorkspaceRunCommand,
    WorkspaceRunPackageSelection, WorkspaceSolveSkipReason,
};

#[test]
fn bootstrap_runs_sample_workspace_from_main_entry_boundary() {
    let report =
        run_studio_bootstrap(&StudioBootstrapConfig::default()).expect("expected bootstrap run");

    assert_eq!(
        app_command(&report).boundary,
        StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
    );
    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(
        report.control_state.simulation_mode,
        dispatch.simulation_mode
    );
    assert_eq!(report.control_state.run_status, dispatch.run_status);
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(
        dispatch.package_id.as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert!(matches!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Started(_)
    ));
    assert_eq!(
        dispatch.latest_snapshot_summary.as_deref(),
        Some("solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)")
    );
    assert_eq!(report.run_panel.view().mode_label, "Active");
    assert_eq!(report.run_panel.view().status_label, "Converged");
    assert_eq!(report.run_panel.view().primary_action.label, "Run");
    assert_eq!(report.run_panel.view().secondary_actions.len(), 2);
    assert_eq!(dispatch.log_entry_count, 2);
    assert_eq!(report.log_entries.len(), 2);
    assert_eq!(report.entitlement_preflight, None);
    assert_eq!(
        report
            .entitlement_host
            .snapshot
            .state
            .next_timer
            .as_ref()
            .map(|timer| timer.event),
        Some(crate::EntitlementSessionLifecycleEvent::TimerElapsed)
    );
    assert!(matches!(
        report.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::ArmTimer { .. })
    ));
    assert_eq!(
        report
            .entitlement_host
            .snapshot
            .state
            .host_notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Automatic check scheduled")
    );
    assert_eq!(
        report
            .entitlement_host
            .presentation
            .panel
            .view
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Automatic check scheduled")
    );
}

#[test]
fn bootstrap_runs_cooler_workspace_from_main_entry_boundary() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        project_path: example_project_path("feed-cooler-flash-binary-hydrocarbon.rfproj.json"),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected bootstrap run");

    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(
        dispatch.package_id.as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert!(matches!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Started(_)
    ));
    assert_eq!(
        dispatch.latest_snapshot_summary.as_deref(),
        Some("solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)")
    );
}

#[test]
fn bootstrap_resumes_workspace_from_hold_via_run_panel_intent() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::resume(
            RunPanelPackageSelection::preferred(),
        )),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected bootstrap resume");

    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(
        report.control_state.simulation_mode,
        dispatch.simulation_mode
    );
    assert_eq!(report.control_state.run_status, dispatch.run_status);
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert!(matches!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Started(_)
    ));
    assert_eq!(dispatch.log_entry_count, 2);
    assert_eq!(report.log_entries.len(), 2);
    assert_eq!(
        report.log_entries[0].message,
        "Activated workspace simulation mode"
    );
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_accepts_preferred_package_selection_when_single_cached_package_exists() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::run_manual(
            RunPanelPackageSelection::preferred(),
        )),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected preferred package bootstrap run");

    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(
        report.control_state.simulation_mode,
        dispatch.simulation_mode
    );
    assert_eq!(
        dispatch.package_id.as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert_eq!(dispatch.log_entry_count, 1);
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_can_switch_workspace_mode_from_run_panel_intent() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::set_mode(SimulationMode::Active)),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected mode intent bootstrap run");

    match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceMode(dispatch) => {
            assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            assert_eq!(dispatch.run_status, RunStatus::Idle);
        }
        StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected workspace mode dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace mode dispatch"),
    }
    assert_eq!(report.control_state.simulation_mode, SimulationMode::Active);
    assert_eq!(report.run_panel.view().mode_label, "Active");
    assert_eq!(report.run_panel.view().primary_action.label, "Run");
    assert_eq!(report.log_entries.len(), 1);
    assert_eq!(
        report.log_entries[0].message,
        "Set workspace simulation mode to Active"
    );
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_runtime_can_dispatch_automatic_run_after_mode_activation() {
    let config = StudioBootstrapConfig::default();
    let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

    let mode = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::AppCommand(
            StudioAppCommand::set_workspace_simulation_mode(SimulationMode::Active),
        ))
        .expect("expected mode activation");
    match &app_command(&mode).dispatch {
        StudioAppResultDispatch::WorkspaceMode(dispatch) => {
            assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            assert_eq!(
                dispatch.pending_reason,
                Some(rf_ui::SolvePendingReason::ModeActivated)
            );
        }
        other => panic!("expected workspace mode dispatch, got {other:?}"),
    }

    let automatic = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::AppCommand(
            StudioAppCommand::run_workspace(WorkspaceRunCommand::automatic_preferred()),
        ))
        .expect("expected automatic run");
    let dispatch = match &app_command(&automatic).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        other => panic!("expected workspace run dispatch, got {other:?}"),
    };
    assert_eq!(
        dispatch.package_id.as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert!(matches!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Started(_)
    ));
    assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
    assert_eq!(dispatch.pending_reason, None);
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(automatic.control_state.run_status, RunStatus::Converged);
    assert_eq!(automatic.run_panel.view().status_label, "Converged");
}

#[test]
fn bootstrap_runtime_skips_automatic_run_when_no_pending_request_after_success() {
    let config = StudioBootstrapConfig::default();
    let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

    let first = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::AppCommand(
            StudioAppCommand::resume_workspace(WorkspaceRunPackageSelection::Preferred),
        ))
        .expect("expected successful resume");
    let first_dispatch = match &app_command(&first).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        other => panic!("expected workspace run dispatch, got {other:?}"),
    };
    assert!(matches!(
        first_dispatch.outcome,
        StudioWorkspaceRunOutcome::Started(_)
    ));

    let automatic = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::AppCommand(
            StudioAppCommand::run_workspace(WorkspaceRunCommand::automatic_preferred()),
        ))
        .expect("expected automatic skip");
    let dispatch = match &app_command(&automatic).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        other => panic!("expected workspace run dispatch, got {other:?}"),
    };
    assert_eq!(dispatch.package_id, None);
    assert!(matches!(
        dispatch.outcome,
        StudioWorkspaceRunOutcome::Skipped(WorkspaceSolveSkipReason::NoPendingRequest)
    ));
    assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
    assert_eq!(dispatch.pending_reason, None);
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(
        dispatch.latest_snapshot_id.as_deref(),
        Some("example-feed-heater-flash-rev-0-seq-1")
    );
    assert_eq!(automatic.control_state.run_status, RunStatus::Converged);
}

#[test]
fn bootstrap_can_dispatch_run_via_widget_action() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::WidgetAction(RunPanelActionId::RunManual),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected widget action bootstrap run");

    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(report.run_panel.view().primary_action.label, "Run");
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_can_dispatch_run_panel_recovery_action() {
    let error = run_studio_bootstrap(&StudioBootstrapConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("feed-valve-flash.rfproj.json"),
        trigger: StudioBootstrapTrigger::WidgetRecoveryAction,
        ..StudioBootstrapConfig::default()
    })
    .expect_err("expected recovery action to be unavailable before failure");

    assert_eq!(
        error.code().as_str(),
        "invalid_input",
        "expected bootstrap trigger to reject recovery before solver failure"
    );

    let config = StudioBootstrapConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("feed-valve-flash.rfproj.json"),
        trigger: StudioBootstrapTrigger::WidgetAction(RunPanelActionId::RunManual),
        ..StudioBootstrapConfig::default()
    };
    let mut session = BootstrapSession::new(&config).expect("expected bootstrap session");
    session
        .app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&rf_types::StreamId::new("stream-throttled"))
        .expect("expected throttled stream")
        .pressure_pa = 130_000.0;

    let report = session
        .run_trigger(&StudioBootstrapTrigger::WidgetAction(
            RunPanelActionId::RunManual,
        ))
        .expect("expected failed run dispatch");
    match &report.dispatch {
        StudioBootstrapDispatch::AppCommand(outcome) => match &outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Failed(_)
                ));
            }
            other => panic!("expected workspace run dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }

    let recovery_report = session
        .run_trigger(&StudioBootstrapTrigger::WidgetRecoveryAction)
        .expect("expected recovery trigger dispatch");

    match &recovery_report.dispatch {
        StudioBootstrapDispatch::RunPanelRecovery(outcome) => {
            assert_eq!(outcome.action.title, "Inspect unit inputs");
            assert_eq!(
                outcome.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "valve-1"
                )))
            );
        }
        other => panic!("expected run panel recovery dispatch, got {other:?}"),
    }
    assert_eq!(
        recovery_report
            .run_panel
            .text()
            .lines
            .iter()
            .find(|line| line.as_str() == "Suggested target: unit valve-1"),
        Some(&"Suggested target: unit valve-1".to_string())
    );
    assert!(recovery_report.control_state.notice.is_some());
}

#[test]
fn bootstrap_default_trigger_runs_via_primary_widget_action() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig::default())
        .expect("expected primary widget bootstrap run");

    let dispatch = match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
    };
    assert_eq!(dispatch.run_status, RunStatus::Converged);
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_can_sync_entitlement_via_control_plane_trigger() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::EntitlementWidgetAction(
            EntitlementActionId::SyncEntitlement,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected entitlement sync bootstrap run");

    assert_eq!(
        app_command(&report).boundary,
        StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
    );
    match &app_command(&report).dispatch {
        StudioAppResultDispatch::Entitlement(dispatch) => {
            assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
            assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
            assert_eq!(
                dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
                Some("Entitlement synced")
            );
        }
        StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected entitlement dispatch"),
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected entitlement dispatch"),
    }
    assert_eq!(report.control_state.run_status, RunStatus::Idle);
    assert_eq!(report.run_panel.view().primary_action.label, "Resume");
    assert_eq!(
        report
            .entitlement_host
            .presentation
            .panel
            .view
            .primary_action
            .label,
        "Refresh offline lease"
    );
    assert_eq!(report.log_entries.len(), 1);
    assert_eq!(
        report.log_entries[0].message,
        "Synced entitlement snapshot and property package manifests from control plane"
    );
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_can_refresh_offline_lease_via_control_plane_trigger() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        trigger: StudioBootstrapTrigger::EntitlementWidgetPrimaryAction,
        ..StudioBootstrapConfig::default()
    })
    .expect("expected offline refresh bootstrap run");

    assert_eq!(
        app_command(&report).boundary,
        StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
    );
    match &app_command(&report).dispatch {
        StudioAppResultDispatch::Entitlement(dispatch) => {
            assert_eq!(
                dispatch.action,
                StudioEntitlementAction::RefreshOfflineLease
            );
            assert_eq!(
                dispatch.outcome,
                StudioEntitlementOutcome::OfflineLeaseRefreshed
            );
            assert_eq!(
                dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
                Some("Offline lease refreshed")
            );
        }
        StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected entitlement dispatch"),
        StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected entitlement dispatch"),
    }
    assert_eq!(report.control_state.run_status, RunStatus::Idle);
    assert_eq!(report.run_panel.view().primary_action.label, "Resume");
    assert_eq!(
        report
            .entitlement_host
            .presentation
            .panel
            .view
            .primary_action
            .label,
        "Refresh offline lease"
    );
    assert_eq!(report.log_entries.len(), 1);
    assert_eq!(
        report.log_entries[0].message,
        "Refreshed offline lease state from control plane"
    );
    assert_eq!(report.entitlement_preflight, None);
}

#[test]
fn bootstrap_auto_preflight_syncs_when_snapshot_is_missing() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_seed: StudioBootstrapEntitlementSeed::MissingSnapshot,
        ..StudioBootstrapConfig::default()
    })
    .expect("expected bootstrap run with entitlement preflight sync");

    let preflight = report
        .entitlement_preflight
        .as_ref()
        .expect("expected preflight outcome");
    assert_eq!(
        preflight.decision.action,
        EntitlementPreflightAction::SyncEntitlement
    );
    match &preflight.outcome.dispatch {
        StudioAppResultDispatch::Entitlement(dispatch) => {
            assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
            assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
        }
        other => panic!("expected entitlement preflight dispatch, got {other:?}"),
    }
    match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert_eq!(dispatch.run_status, RunStatus::Converged);
        }
        other => panic!("expected workspace run after preflight, got {other:?}"),
    }
}

#[test]
fn bootstrap_auto_preflight_refreshes_when_lease_is_expiring() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        ..StudioBootstrapConfig::default()
    })
    .expect("expected bootstrap run with entitlement preflight refresh");

    let preflight = report
        .entitlement_preflight
        .as_ref()
        .expect("expected preflight outcome");
    assert_eq!(
        preflight.decision.action,
        EntitlementPreflightAction::RefreshOfflineLease
    );
    match &preflight.outcome.dispatch {
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
        other => panic!("expected entitlement preflight dispatch, got {other:?}"),
    }
    match &app_command(&report).dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert_eq!(dispatch.run_status, RunStatus::Converged);
        }
        other => panic!("expected workspace run after preflight, got {other:?}"),
    }
}

#[test]
fn bootstrap_can_dispatch_login_completed_session_event() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::MissingSnapshot,
        trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::LoginCompleted,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected login completed session event");

    let outcome = session_event(&report);
    assert_eq!(outcome.event, EntitlementSessionEvent::LoginCompleted);
    match &outcome.outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            let preflight = tick.preflight.as_ref().expect("expected sync preflight");
            assert_eq!(
                preflight.decision.action,
                EntitlementPreflightAction::SyncEntitlement
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
    assert_eq!(report.entitlement_preflight, None);
    assert_eq!(
        report
            .entitlement_host
            .presentation
            .panel
            .view
            .primary_action
            .label,
        "Refresh offline lease"
    );
}

#[test]
fn bootstrap_can_dispatch_timer_elapsed_session_event() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::TimerElapsed,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected timer elapsed session event");

    let outcome = session_event(&report);
    assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
    match &outcome.outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            let preflight = tick
                .preflight
                .as_ref()
                .expect("expected offline refresh preflight");
            assert_eq!(
                preflight.decision.action,
                EntitlementPreflightAction::RefreshOfflineLease
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
    assert_eq!(report.entitlement_preflight, None);
    assert_eq!(report.control_state.run_status, RunStatus::Idle);
    assert_eq!(
        report
            .entitlement_host
            .snapshot
            .state
            .next_timer
            .as_ref()
            .map(|timer| timer.event),
        Some(crate::EntitlementSessionLifecycleEvent::TimerElapsed)
    );
    assert!(matches!(
        report.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
    ));
    assert_eq!(
        report
            .entitlement_host
            .snapshot
            .state
            .host_notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Automatic check scheduled")
    );
}

#[test]
fn bootstrap_entitlement_host_report_exposes_runtime_presentation_and_effect() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::TimerElapsed,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected timer elapsed session event");

    assert_eq!(
        report.entitlement_host.presentation.text.title,
        "Entitlement host"
    );
    assert!(
        report
            .entitlement_host
            .presentation
            .text
            .lines
            .iter()
            .any(|line| line.starts_with("Timer effect: Rearm timer"))
    );
    assert!(matches!(
        report.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
    ));
}

#[test]
fn bootstrap_session_replays_entitlement_host_event_sequence_with_stable_timer_effects() {
    let config = StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        ..StudioBootstrapConfig::default()
    };
    let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

    let timer_elapsed = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::TimerElapsed,
        ))
        .expect("expected timer elapsed event");
    let network_restored = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::NetworkRestored,
        ))
        .expect("expected network restored event");
    let window_foregrounded = runtime
        .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::WindowForegrounded,
        ))
        .expect("expected window foregrounded event");

    match &session_event(&timer_elapsed).outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            let preflight = tick
                .preflight
                .as_ref()
                .expect("expected refresh preflight on timer elapsed");
            assert_eq!(
                preflight.decision.action,
                EntitlementPreflightAction::RefreshOfflineLease
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
    assert!(matches!(
        timer_elapsed.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
    ));

    match &session_event(&network_restored).outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            assert!(
                tick.preflight.is_none(),
                "expected no preflight after refresh, got {:?}",
                tick.preflight
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
    assert!(matches!(
        network_restored.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
    ));

    match &session_event(&window_foregrounded).outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            assert!(
                tick.preflight.is_none(),
                "expected no preflight after refresh, got {:?}",
                tick.preflight
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
    assert!(matches!(
        window_foregrounded.entitlement_host.timer_effect,
        Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
    ));
    assert_eq!(
        network_restored.entitlement_host.snapshot.state.next_timer,
        window_foregrounded
            .entitlement_host
            .snapshot
            .state
            .next_timer
    );
}

#[test]
fn bootstrap_can_dispatch_network_restored_session_event() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::NetworkRestored,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected network restored session event");

    let outcome = session_event(&report);
    assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
    match &outcome.outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            let preflight = tick
                .preflight
                .as_ref()
                .expect("expected offline refresh preflight");
            assert_eq!(
                preflight.decision.action,
                EntitlementPreflightAction::RefreshOfflineLease
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
}

#[test]
fn bootstrap_can_dispatch_window_foregrounded_session_event() {
    let report = run_studio_bootstrap(&StudioBootstrapConfig {
        entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
        entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
        trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
            StudioBootstrapEntitlementSessionEvent::WindowForegrounded,
        ),
        ..StudioBootstrapConfig::default()
    })
    .expect("expected window foregrounded session event");

    let outcome = session_event(&report);
    assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
    match &outcome.outcome {
        EntitlementSessionEventOutcome::Tick(tick) => {
            let preflight = tick
                .preflight
                .as_ref()
                .expect("expected offline refresh preflight");
            assert_eq!(
                preflight.decision.action,
                EntitlementPreflightAction::RefreshOfflineLease
            );
        }
        other => panic!("expected tick outcome, got {other:?}"),
    }
}

fn app_command(report: &super::StudioBootstrapReport) -> &crate::StudioAppCommandOutcome {
    match &report.dispatch {
        StudioBootstrapDispatch::AppCommand(outcome) => outcome,
        StudioBootstrapDispatch::RunPanelRecovery(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::DocumentLifecycle(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorTarget(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftUpdate(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftCommit(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftDiscard(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftBatchCommit(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftBatchDiscard(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionNormalize(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionComponentAdd(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionComponentRemove(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::DocumentHistory(_) => {
            panic!("expected app command dispatch")
        }
        StudioBootstrapDispatch::EntitlementSessionEvent(_) => {
            panic!("expected app command dispatch")
        }
    }
}

fn session_event(
    report: &super::StudioBootstrapReport,
) -> &crate::EntitlementSessionEventDriverOutcome {
    match &report.dispatch {
        StudioBootstrapDispatch::EntitlementSessionEvent(outcome) => outcome,
        StudioBootstrapDispatch::AppCommand(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::RunPanelRecovery(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::DocumentLifecycle(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorTarget(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftUpdate(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftCommit(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftDiscard(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftBatchCommit(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorDraftBatchDiscard(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionNormalize(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionComponentAdd(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::InspectorCompositionComponentRemove(_) => {
            panic!("expected entitlement session event dispatch")
        }
        StudioBootstrapDispatch::DocumentHistory(_) => {
            panic!("expected entitlement session event dispatch")
        }
    }
}

fn example_project_path(project_file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("flowsheets")
        .join(project_file_name)
}
