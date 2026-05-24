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

pub(super) fn flash_drum_sink_only_reconnect_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-gui-host-sink-only-reconnect-{timestamp}.rfproj.json"
    ));
    let mut project = rf_store::parse_project_file_json(
        crate::test_support::official_heater_binary_hydrocarbon_project_json(),
    )
    .expect("expected official heater project");
    let heater = project
        .document
        .flowsheet
        .units
        .get_mut(&rf_types::UnitId::new("heater-1"))
        .expect("expected heater unit");
    let heater_outlet = heater
        .ports
        .iter_mut()
        .find(|port| port.name == "outlet")
        .expect("expected heater outlet");
    heater_outlet.stream_id = None;
    let project =
        rf_store::project_file_to_pretty_json(&project).expect("expected project serialization");
    fs::write(&project_path, project).expect("expected sink-only reconnect project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
    )
}

pub(super) fn cycle_reconnect_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-gui-host-cycle-reconnect-{timestamp}.rfproj.json"
    ));
    let mut flowsheet = rf_model::Flowsheet::new("reconnect-cycle");
    flowsheet
        .insert_stream(rf_model::MaterialStreamState::new("stream-feed", "Feed"))
        .expect("expected feed stream");
    flowsheet
        .insert_stream(rf_model::MaterialStreamState::new(
            "stream-heated",
            "Heated",
        ))
        .expect("expected heated stream");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "valve-1",
            "Valve",
            "valve",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-feed".into()),
                ),
            ],
        ))
        .expect("expected valve insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    Some("stream-feed".into()),
                ),
                rf_model::UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-heated".into()),
                ),
            ],
        ))
        .expect("expected heater insert");
    let project = rf_store::StoredProjectFile::new(
        flowsheet,
        rf_store::StoredDocumentMetadata::new(
            "doc-reconnect-cycle",
            "Reconnect Cycle",
            SystemTime::UNIX_EPOCH,
        ),
    );
    let project =
        rf_store::project_file_to_pretty_json(&project).expect("expected project serialization");
    fs::write(&project_path, project).expect("expected cycle reconnect project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
    )
}

pub(super) fn ambiguous_reconnect_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-gui-host-ambiguous-reconnect-{timestamp}.rfproj.json"
    ));
    let mut flowsheet = rf_model::Flowsheet::new("reconnect-ambiguous");
    flowsheet
        .insert_stream(rf_model::MaterialStreamState::new(
            "stream-heated",
            "Heated",
        ))
        .expect("expected heated stream");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-heated".into()),
                ),
            ],
        ))
        .expect("expected heater insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "flash-1",
            "Flash",
            "flash_drum",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "vapor",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "liquid",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
            ],
        ))
        .expect("expected flash insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "valve-1",
            "Valve",
            "valve",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
            ],
        ))
        .expect("expected valve insert");
    let project = rf_store::StoredProjectFile::new(
        flowsheet,
        rf_store::StoredDocumentMetadata::new(
            "doc-reconnect-ambiguous",
            "Reconnect Ambiguous",
            SystemTime::UNIX_EPOCH,
        ),
    );
    let project =
        rf_store::project_file_to_pretty_json(&project).expect("expected project serialization");
    fs::write(&project_path, project).expect("expected ambiguous reconnect project");

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
