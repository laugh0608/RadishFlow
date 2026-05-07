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
    let project_json = include_str!("../../../../examples/flowsheets/feed-valve-flash.rfproj.json")
        .replacen(
            "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 90000.0,",
            "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 130000.0,",
            1,
        );
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
    let project_json =
        include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
            .replacen(
                ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"component-a\": 0.5,\n            \"component-b\": 0.5\n          },\n          \"phases\": []\n        }",
                "",
                1,
            )
            .replacen(
                "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                1,
            );
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
