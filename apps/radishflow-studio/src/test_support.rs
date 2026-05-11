use std::path::Path;
use std::time::SystemTime;

use rf_store::{
    StoredAuthCacheIndex, StoredProjectFile, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    property_package_payload_integrity, write_property_package_manifest,
    write_property_package_payload,
};
use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest as ThermoPropertyPackageManifest,
    PropertyPackageSource as ThermoPropertyPackageSource, ThermoComponent, ThermoSystem,
};
use rf_types::ComponentId;

#[doc(hidden)]
pub const OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS: [(&str, &str); 2] =
    [("methane", "Methane"), ("ethane", "Ethane")];
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_A_ID: &str = "synthetic-component-a";
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_B_ID: &str = "synthetic-component-b";
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_C_ID: &str = "synthetic-component-c";
#[doc(hidden)]
pub const SYNTHETIC_BINARY_COMPONENT_SPECS: [(&str, &str); 2] = [
    (SYNTHETIC_COMPONENT_A_ID, "Synthetic Component A"),
    (SYNTHETIC_COMPONENT_B_ID, "Synthetic Component B"),
];
#[doc(hidden)]
pub const OFFICIAL_HEATER_BINARY_HYDROCARBON_PROJECT_FILE_NAME: &str =
    "feed-heater-flash-binary-hydrocarbon.rfproj.json";
#[doc(hidden)]
pub const OFFICIAL_VALVE_BINARY_HYDROCARBON_PROJECT_FILE_NAME: &str =
    "feed-valve-flash-binary-hydrocarbon.rfproj.json";
#[doc(hidden)]
pub const OFFICIAL_HEATER_BINARY_HYDROCARBON_AUTORUN_SNAPSHOT_ID: &str =
    "example-feed-heater-flash-binary-hydrocarbon-rev-1-seq-1";

#[doc(hidden)]
pub fn official_heater_binary_hydrocarbon_project_json() -> &'static str {
    include_str!("../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json")
}

#[doc(hidden)]
pub fn official_valve_binary_hydrocarbon_project_json() -> &'static str {
    include_str!("../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json")
}

#[doc(hidden)]
pub fn build_flash_drum_local_rules_project_json() -> String {
    official_heater_binary_hydrocarbon_project_json()
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
        )
}

#[doc(hidden)]
pub fn build_flash_drum_local_rules_synced_project_json() -> String {
    official_heater_binary_hydrocarbon_project_json()
        .replacen(
            ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"methane\": 0.5,\n            \"ethane\": 0.5\n          },\n          \"phases\": []\n        }",
            "",
            1,
        )
        .replacen(
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
            1,
        )
}

#[doc(hidden)]
pub fn build_valve_solver_failure_project_json() -> String {
    official_valve_binary_hydrocarbon_project_json().replacen(
        "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 650000.0,",
        "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 730000.0,",
        1,
    )
}

#[doc(hidden)]
pub fn build_official_binary_hydrocarbon_provider() -> PlaceholderThermoProvider {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        "binary-hydrocarbon-lite-v1",
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let [first, second] = binary_hydrocarbon_lite_components(&payload);

    PlaceholderThermoProvider::new(ThermoSystem::binary([
        build_thermo_component_from_stored_component(first),
        build_thermo_component_from_stored_component(second),
    ]))
}

fn build_thermo_component_from_stored_component(
    component: &rf_store::StoredThermoComponent,
) -> ThermoComponent {
    let mut thermo_component = ThermoComponent::new(component.id.clone(), component.name.clone());
    thermo_component.antoine = component.antoine.as_ref().map(|coefficients| {
        AntoineCoefficients::new(coefficients.a, coefficients.b, coefficients.c)
    });
    thermo_component.liquid_heat_capacity_j_per_mol_k = component.liquid_heat_capacity_j_per_mol_k;
    thermo_component.vapor_heat_capacity_j_per_mol_k = component.vapor_heat_capacity_j_per_mol_k;
    thermo_component
}

fn binary_hydrocarbon_lite_components(
    payload: &StoredPropertyPackagePayload,
) -> [&rf_store::StoredThermoComponent; 2] {
    match payload.components.as_slice() {
        [first, second] => [first, second],
        _ => panic!("expected property package payload to stay binary"),
    }
}

#[doc(hidden)]
fn build_stored_payload_from_official_binary_hydrocarbon_sample(
    package_id: &str,
    component_specs: [(&str, &str); 2],
) -> StoredPropertyPackagePayload {
    let mut payload = crate::parse_property_package_download_json(include_str!(
        "../../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json"
    ))
    .expect("expected official sample property package download")
    .to_stored_payload()
    .expect("expected stored payload");
    payload.package_id = package_id.to_string();
    for (component, (component_id, component_name)) in
        payload.components.iter_mut().zip(component_specs)
    {
        component.id = ComponentId::new(component_id);
        component.name = component_name.to_string();
    }
    payload
        .validate()
        .expect("expected mapped stored payload to stay valid");
    payload
}

#[doc(hidden)]
pub fn write_official_binary_hydrocarbon_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
    downloaded_at: SystemTime,
    expires_at: Option<SystemTime>,
) {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        package_id,
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let integrity =
        property_package_payload_integrity(&payload).expect("expected payload integrity");
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        &payload.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        payload.component_ids(),
    );
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = expires_at;
    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        manifest.hash.clone(),
        manifest.size_bytes,
        downloaded_at,
    );
    record.expires_at = expires_at;

    write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(cache_root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    auth_cache_index.property_packages.push(record);
}

#[doc(hidden)]
pub fn build_official_binary_hydrocarbon_in_memory_provider(
    package_id: &str,
) -> InMemoryPropertyPackageProvider {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        package_id,
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let [first, second] = binary_hydrocarbon_lite_components(&payload);

    InMemoryPropertyPackageProvider::new(vec![(
        ThermoPropertyPackageManifest::new(
            package_id,
            &payload.version,
            ThermoPropertyPackageSource::LocalBundled,
            payload.component_ids(),
        ),
        ThermoSystem::binary([
            build_thermo_component_from_stored_component(first),
            build_thermo_component_from_stored_component(second),
        ]),
    )])
}

#[doc(hidden)]
pub fn build_synthetic_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_synthetic_provider(k_values, pressure_pa)
}

#[doc(hidden)]
pub fn apply_stream_state_and_composition(
    project: &mut StoredProjectFile,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
    temperature_k: f64,
    pressure_pa: f64,
) {
    crate::studio_gui_window_model::test_support::apply_stream_state_and_composition(
        project,
        stream_id,
        overall_mole_fractions,
        temperature_k,
        pressure_pa,
    )
}

#[doc(hidden)]
pub fn solve_snapshot_model_from_project_with_provider_and_edit<F>(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
    edit_project: F,
) -> crate::StudioGuiWindowSolveSnapshotModel
where
    F: FnOnce(&mut StoredProjectFile),
{
    crate::studio_gui_window_model::test_support::solve_snapshot_model_from_project_with_provider_and_edit(
        project_json,
        provider,
        edit_project,
    )
}

#[doc(hidden)]
pub fn stream_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::stream_target_detail_model(
        snapshot, stream_id, title,
    )
}

#[doc(hidden)]
pub fn unit_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    unit_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::unit_target_detail_model(snapshot, unit_id, title)
}

#[doc(hidden)]
pub fn stream_target_detail_snapshot(
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiInspectorTargetDetailSnapshot {
    crate::studio_gui_window_model::test_support::stream_target_detail_snapshot(stream_id, title)
}
