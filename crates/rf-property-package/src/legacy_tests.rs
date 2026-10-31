use super::*;
use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    StoredThermoComponent, property_package_payload_integrity, read_property_package_manifest,
    read_property_package_payload, write_property_package_manifest, write_property_package_payload,
};
use rf_thermo::{ThermoComponent, ThermoSystem};
use rf_types::ComponentId;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn package_provider_returns_manifest_and_system_for_known_package() {
    let methane_id = ComponentId::new("methane");
    let system = ThermoSystem::new(vec![ThermoComponent::new(methane_id.clone(), "Methane")]);
    let manifest = PropertyPackageManifest::new(
        "methane-basic-v1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
        vec![methane_id],
    );
    let provider =
        InMemoryPropertyPackageProvider::try_new(vec![(manifest, system.clone())]).unwrap();

    let manifests = provider
        .list_packages(timestamp(700))
        .unwrap()
        .into_iter()
        .map(|entry| entry.availability.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].package_id, "methane-basic-v1");

    let loaded = provider
        .load_package("methane-basic-v1", timestamp(700))
        .map(|package| package.system().clone())
        .expect("expected thermo system");
    assert_eq!(loaded, system);
}

#[test]
fn package_provider_reports_missing_package() {
    let provider = InMemoryPropertyPackageProvider::default();
    let error = provider
        .load_package("missing-package", timestamp(700))
        .expect_err("expected missing package error");

    assert_eq!(error.source.code().as_str(), "missing_entity");
}

#[test]
fn remote_evaluation_manifest_defaults_to_remote_only() {
    let manifest = PropertyPackageManifest::new(
        "remote-eval-pkg",
        "2026.03.1",
        PropertyPackageSource::RemoteEvaluationService,
        vec![],
    );

    assert_eq!(
        manifest.classification,
        PropertyPackageClassification::RemoteOnly
    );
}

#[test]
fn cached_provider_loads_local_packages_from_store_cache_layout() {
    let root = unique_temp_path("cached-provider");
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    let mut record = StoredPropertyPackageRecord::new(
        "methane-basic-v1",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        "",
        0,
        timestamp(100),
    );
    record.expires_at = Some(timestamp(800));
    let payload = StoredPropertyPackagePayload::new(
        "methane-basic-v1",
        "2026.03.1",
        vec![StoredThermoComponent::new(
            ComponentId::new("methane"),
            "Methane",
        )],
    );
    let integrity =
        property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = {
        let mut manifest = StoredPropertyPackageManifest::new(
            "methane-basic-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![ComponentId::new("methane")],
        );
        manifest.hash = integrity.hash.clone();
        manifest.size_bytes = integrity.size_bytes;
        manifest.expires_at = Some(timestamp(800));
        manifest
    };
    record.hash = integrity.hash.clone();
    record.size_bytes = integrity.size_bytes;

    write_property_package_manifest(record.manifest_path_under(&root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(&root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    index.property_packages.push(record);

    let provider =
        CachedPropertyPackageProvider::new(&root, &index).expect("expected cached provider");
    let manifests = provider
        .list_packages(timestamp(700))
        .unwrap()
        .into_iter()
        .map(|entry| entry.availability.unwrap())
        .collect::<Vec<_>>();

    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].package_id, "methane-basic-v1");
    assert_eq!(manifests[0].hash, integrity.hash);
    let system = provider
        .load_package("methane-basic-v1", timestamp(700))
        .map(|package| package.system().clone())
        .expect("expected thermo system");
    assert_eq!(system.component_count(), 1);
    assert_eq!(system.components[0].name, "Methane");

    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

#[test]
fn cached_provider_skips_expired_records_before_touching_disk() {
    let root = unique_temp_path("cached-provider-expired");
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    let mut record = StoredPropertyPackageRecord::new(
        "expired-pkg",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        "sha256:expired",
        256,
        timestamp(100),
    );
    record.expires_at = Some(timestamp(200));
    index.property_packages.push(record);

    let provider =
        CachedPropertyPackageProvider::new(&root, &index).expect("expected cached provider");

    let listings = provider.list_packages(timestamp(300)).unwrap();
    assert_eq!(listings.len(), 1);
    assert_eq!(
        listings[0].availability.as_ref().unwrap_err().kind,
        PackageFailureKind::Expired
    );
    assert_eq!(
        provider
            .load_package("expired-pkg", timestamp(300))
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    fs::remove_dir_all(&root).ok();
}

#[test]
fn cached_provider_rejects_manifest_mismatch_against_index_record() {
    let root = unique_temp_path("cached-provider-mismatch");
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    let mut record = StoredPropertyPackageRecord::new(
        "pkg-1",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        "sha256:expected",
        0,
        timestamp(100),
    );
    record.expires_at = Some(timestamp(500));
    let mut manifest = StoredPropertyPackageManifest::new(
        "pkg-1",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![ComponentId::new("methane")],
    );
    let payload = StoredPropertyPackagePayload::new(
        "pkg-1",
        "2026.03.1",
        vec![StoredThermoComponent::new(
            ComponentId::new("methane"),
            "Methane",
        )],
    );
    let integrity =
        property_package_payload_integrity(&payload).expect("expected payload integrity");
    manifest.hash = integrity.hash;
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = Some(timestamp(500));
    record.size_bytes = integrity.size_bytes;

    write_property_package_manifest(record.manifest_path_under(&root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(&root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    index.property_packages.push(record);

    let error = CachedPropertyPackageProvider::new(&root, &index)
        .unwrap()
        .load_package("pkg-1", timestamp(300))
        .expect_err("expected manifest mismatch");

    assert_eq!(error.source.code().as_str(), "invalid_input");
    assert!(
        error
            .source
            .message()
            .contains("does not match manifest hash")
    );
    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

#[test]
fn cached_provider_loads_example_property_package_files() {
    let root = unique_temp_path("cached-provider-example");
    let example_manifest_path = example_package_path("manifest.json");
    let example_payload_path = example_package_path("payload.rfpkg");
    let manifest =
        read_property_package_manifest(&example_manifest_path).expect("expected manifest read");
    let payload =
        read_property_package_payload(&example_payload_path).expect("expected payload read");

    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        manifest.source,
        manifest.hash.clone(),
        manifest.size_bytes,
        timestamp(100),
    );
    record.expires_at = manifest.expires_at;

    write_property_package_manifest(record.manifest_path_under(&root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(&root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    index.property_packages.push(record);

    let provider =
        CachedPropertyPackageProvider::new(&root, &index).expect("expected cached provider");
    let system = provider
        .load_package("binary-hydrocarbon-lite-v1", timestamp(150))
        .map(|package| package.system().clone())
        .expect("expected thermo system");

    assert_eq!(system.component_count(), 2);
    assert_eq!(system.components[0].id.as_str(), "methane");
    assert_eq!(system.components[1].id.as_str(), "ethane");

    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn unique_temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
}

fn example_package_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1")
        .join(file_name)
}
