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
pub fn build_binary_demo_provider() -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_binary_demo_provider()
}

#[doc(hidden)]
pub fn build_binary_hydrocarbon_lite_provider() -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_binary_hydrocarbon_lite_provider()
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

#[doc(hidden)]
pub fn build_binary_hydrocarbon_lite_stored_payload_for_components(
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
pub fn write_binary_hydrocarbon_lite_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
    component_specs: [(&str, &str); 2],
    downloaded_at: SystemTime,
    expires_at: Option<SystemTime>,
) {
    let payload =
        build_binary_hydrocarbon_lite_stored_payload_for_components(package_id, component_specs);
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
pub fn build_binary_hydrocarbon_lite_in_memory_provider_for_components(
    package_id: &str,
    component_specs: [(&str, &str); 2],
) -> InMemoryPropertyPackageProvider {
    let payload =
        build_binary_hydrocarbon_lite_stored_payload_for_components(package_id, component_specs);
    let first = build_thermo_component_from_stored_component(&payload.components[0]);
    let second = build_thermo_component_from_stored_component(&payload.components[1]);

    InMemoryPropertyPackageProvider::new(vec![(
        ThermoPropertyPackageManifest::new(
            package_id,
            &payload.version,
            ThermoPropertyPackageSource::LocalBundled,
            payload.component_ids(),
        ),
        ThermoSystem::binary([first, second]),
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
