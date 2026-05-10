use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rf_store::studio_layout_path_for_project;
use rf_ui::RunPanelActionId;

use super::*;

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
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

pub(super) fn solver_failure_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-gui-host-failure-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::build_valve_solver_failure_project_json();
    fs::write(&project_path, project).expect("expected failure project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            ..lease_expiring_config()
        },
        project_path,
    )
}

pub(super) fn flash_drum_local_rules_synced_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-gui-host-local-rules-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::build_flash_drum_local_rules_synced_project_json();
    fs::write(&project_path, project).expect("expected synced local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..lease_expiring_config()
        },
        project_path,
    )
}

pub(super) fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-gui-host-local-rules-unsynced-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::build_flash_drum_local_rules_project_json();
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
        "radishflow-studio-layout-persistence-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::official_heater_binary_hydrocarbon_project_json();
    fs::write(&project_path, project).expect("expected persistence project");
    let layout_path = studio_layout_path_for_project(&project_path);

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
        layout_path,
    )
}
