use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rf_ui::{
    GhostElement, GhostElementKind, StreamVisualKind, StreamVisualState, SuggestionSource,
};

use super::*;

pub(super) fn find_menu_command<'a>(
    nodes: &'a [crate::StudioGuiCommandMenuNode],
    command_id: &str,
) -> Option<&'a crate::StudioGuiCommandMenuCommandModel> {
    for node in nodes {
        if let Some(command) = node.command.as_ref() {
            if command.command_id == command_id {
                return Some(command);
            }
        }
        if let Some(command) = find_menu_command(&node.children, command_id) {
            return Some(command);
        }
    }
    None
}

pub(super) fn find_menu_command_by_label<'a>(
    nodes: &'a [crate::StudioGuiCommandMenuNode],
    label: &str,
) -> Option<&'a crate::StudioGuiCommandMenuCommandModel> {
    for node in nodes {
        if let Some(command) = node.command.as_ref() {
            if command.label == label {
                return Some(command);
            }
        }
        if let Some(command) = find_menu_command_by_label(&node.children, label) {
            return Some(command);
        }
    }
    None
}

pub(super) fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: crate::StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: crate::StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

pub(super) fn synced_workspace_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: crate::StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: crate::StudioRuntimeEntitlementSeed::Synced,
        ..StudioRuntimeConfig::default()
    }
}

pub(super) fn flash_drum_local_rules_synced_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-local-rules-synced-{timestamp}.rfproj.json"
    ));
    let project = include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
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
    fs::write(&project_path, project).expect("expected synced local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..synced_workspace_config()
        },
        project_path,
    )
}

pub(super) fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("failures")
            .join("unbound-outlet-port.rfproj.json"),
        entitlement_preflight: crate::StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: crate::StudioRuntimeEntitlementSeed::Synced,
        trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
    }
}

pub(super) fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-local-rules-{timestamp}.rfproj.json"
    ));
    let project = include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
        .replacen(
            "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
            "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
            1,
        )
        .replacen(
            "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
            "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
            1,
        )
        .replacen(
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
            1,
        );
    fs::write(&project_path, project).expect("expected local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
    )
}

pub(super) fn layout_persistence_config() -> (StudioRuntimeConfig, PathBuf, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-driver-layout-persistence-{timestamp}.rfproj.json"
    ));
    let project = include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json");
    fs::write(&project_path, project).expect("expected persistence project");
    let layout_path = rf_store::studio_layout_path_for_project(&project_path);

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
        layout_path,
    )
}

pub(super) fn sample_canvas_suggestion(id: &str, confidence: f32) -> rf_ui::CanvasSuggestion {
    rf_ui::CanvasSuggestion::new(
        rf_ui::CanvasSuggestionId::new(id),
        SuggestionSource::LocalRules,
        confidence,
        GhostElement {
            kind: GhostElementKind::Connection,
            target_unit_id: rf_types::UnitId::new("flash-1"),
            visual_kind: StreamVisualKind::Material,
            visual_state: StreamVisualState::Suggested,
        },
        "accept by tab",
    )
}

pub(super) fn assert_ignored_shortcut(
    dispatch: &StudioGuiDriverDispatch,
    shortcut: StudioGuiShortcut,
    reason: StudioGuiShortcutIgnoreReason,
) {
    assert_eq!(
        dispatch.outcome,
        StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason }
    );
}
