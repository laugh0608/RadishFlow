use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    StudioAppHost, StudioAppHostCommand, StudioAppHostCommandOutcome, StudioAppHostController,
    StudioAppHostEntitlementTimerEffect, StudioAppHostEntitlementTimerState, StudioAppHostStore,
    StudioAppHostTimerSlotChange, StudioAppHostUiAction, StudioAppHostUiActionAvailability,
    StudioAppHostUiActionDisabledReason, StudioAppHostUiActionModel, StudioAppHostUiActionState,
    StudioAppHostUiCommandDispatchResult, StudioAppHostUiCommandGroup, StudioAppHostWindowChange,
    StudioAppHostWindowSelectionChange, StudioAppWindowHostGlobalEvent,
    StudioCanvasInteractionAction, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRetirement,
    StudioWindowHostRole,
};
use rf_ui::RunPanelActionId;

fn lease_expiring_config() -> crate::StudioRuntimeConfig {
    crate::StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..crate::StudioRuntimeConfig::default()
    }
}

fn solver_failure_config() -> (crate::StudioRuntimeConfig, PathBuf) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    let project_path =
        std::env::temp_dir().join(format!("radishflow-app-host-recovery-{unique}.rfproj.json"));
    let project_json = crate::test_support::build_valve_solver_failure_project_json();
    fs::write(&project_path, project_json).expect("expected temporary failure project");

    (
        crate::StudioRuntimeConfig {
            project_path: project_path.clone(),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            trigger: crate::StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        },
        project_path,
    )
}

fn flash_drum_local_rules_synced_config() -> (crate::StudioRuntimeConfig, PathBuf) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-app-host-local-rules-{unique}.rfproj.json"
    ));
    let project_json = crate::test_support::build_flash_drum_local_rules_synced_project_json();
    fs::write(&project_path, project_json).expect("expected local rules project");

    (
        crate::StudioRuntimeConfig {
            project_path: project_path.clone(),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..crate::StudioRuntimeConfig::default()
        },
        project_path,
    )
}

fn registration_from_opened_window(
    opened: impl std::borrow::Borrow<crate::StudioAppWindowHostOpenWindow>,
) -> crate::StudioWindowHostRegistration {
    super::registration_from_opened_window(opened)
}

mod commands;
mod controller;
mod snapshot;
mod store;
