use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
    StoredPropertyPackageSource, StoredThermoComponent, property_package_payload_integrity,
    write_property_package_manifest, write_property_package_payload,
};
use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest, PropertyPackageSource, ThermoComponent, ThermoSystem,
};
use rf_types::ComponentId;

pub fn build_demo_antoine_coefficients(k_value: f64, pressure_pa: f64) -> AntoineCoefficients {
    const TEST_ANTOINE_BOUNDARY_SLOPE: f64 = 250.0;
    const TEST_REFERENCE_TEMPERATURE_K: f64 = 300.0;

    AntoineCoefficients::new(
        ((k_value * pressure_pa) / 1_000.0).ln()
            + TEST_ANTOINE_BOUNDARY_SLOPE / TEST_REFERENCE_TEMPERATURE_K,
        TEST_ANTOINE_BOUNDARY_SLOPE,
        0.0,
    )
}

pub fn build_binary_demo_provider() -> PlaceholderThermoProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(2.0, pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(0.5, pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

pub fn build_binary_demo_package_provider() -> InMemoryPropertyPackageProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(2.0, pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(0.5, pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
            vec!["component-a".into(), "component-b".into()],
        ),
        ThermoSystem::binary([first, second]),
    )])
}

pub fn unique_temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
}

pub fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

pub fn sample_auth_cache_index(package_ids: &[&str]) -> StoredAuthCacheIndex {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
    );
    index.property_packages = package_ids
        .iter()
        .map(|package_id| {
            let mut record = StoredPropertyPackageRecord::new(
                *package_id,
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                "sha256:test",
                128,
                timestamp(20),
            );
            record.expires_at = Some(timestamp(9_999_999_999));
            record
        })
        .collect();
    index
}

pub fn write_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
) {
    let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
    let first_antoine = build_demo_antoine_coefficients(2.0, 100_000.0);
    first.antoine = Some(StoredAntoineCoefficients::new(
        first_antoine.a,
        first_antoine.b,
        first_antoine.c,
    ));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);
    let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
    let second_antoine = build_demo_antoine_coefficients(0.5, 100_000.0);
    second.antoine = Some(StoredAntoineCoefficients::new(
        second_antoine.a,
        second_antoine.b,
        second_antoine.c,
    ));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    let payload = StoredPropertyPackagePayload::new(package_id, "2026.03.1", vec![first, second]);
    let integrity = property_package_payload_integrity(&payload).expect("expected payload hash");
    let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![
            ComponentId::new("component-a"),
            ComponentId::new("component-b"),
        ],
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
        timestamp(60),
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
