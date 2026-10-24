use super::*;
use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    property_package_payload_integrity, read_property_package_payload,
    write_property_package_manifest, write_property_package_payload,
};
use rf_thermo::{PhaseThermoState, ThermoState};
use rf_types::PhaseLabel;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

struct Cache {
    root: PathBuf,
    index: StoredAuthCacheIndex,
    manifest: StoredPropertyPackageManifest,
    payload: StoredPropertyPackagePayload,
}

impl Cache {
    fn new() -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rf-package-test-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let payload = read_property_package_payload(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/payload.rfpkg")).unwrap();
        let manifest = StoredPropertyPackageManifest::new(
            &payload.package_id,
            &payload.version,
            StoredPropertyPackageSource::LocalBundled,
            payload.component_ids(),
        );
        let record = StoredPropertyPackageRecord::new(
            &payload.package_id,
            &payload.version,
            manifest.source,
            "",
            0,
            at(1),
        );
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "test-user",
            StoredCredentialReference::new("test", "test-user"),
        );
        index.property_packages.push(record);
        let cache = Self {
            root,
            index,
            manifest,
            payload,
        };
        cache.write();
        cache
    }
    fn write(&self) {
        write_property_package_manifest(self.manifest_path(), &self.manifest).unwrap();
        write_property_package_payload(self.payload_path(), &self.payload).unwrap();
    }
    fn manifest_path(&self) -> PathBuf {
        self.index.property_packages[0].manifest_path_under(&self.root)
    }
    fn payload_path(&self) -> PathBuf {
        self.index.property_packages[0]
            .payload_path_under(&self.root)
            .unwrap()
    }
    fn provider(&self) -> CachedPropertyPackageProvider {
        CachedPropertyPackageProvider::new(&self.root, &self.index).unwrap()
    }
    fn load(&self) -> LoadedPropertyPackage {
        self.provider()
            .load_package(&self.payload.package_id, at(10))
            .unwrap()
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn liquid_enthalpy(context: &LocalThermoContext) -> f64 {
    context
        .thermo()
        .phase_molar_enthalpy(&PhaseThermoState::new(
            PhaseLabel::Liquid,
            300.0,
            100_000.0,
            vec![0.5, 0.5],
        ))
        .unwrap()
}

#[test]
fn selected_package_isolated_from_corrupt_manifest_and_payload() {
    let mut cache = Cache::new();
    let mut other = cache.index.property_packages[0].clone();
    other.package_id = "broken-package".into();
    // Use the record constructor so each package has its own storage paths.
    other = StoredPropertyPackageRecord::new(
        &other.package_id,
        &other.version,
        other.source,
        "",
        0,
        at(1),
    );
    let mut other_manifest = cache.manifest.clone();
    other_manifest.package_id = other.package_id.clone();
    write_property_package_manifest(other.manifest_path_under(&cache.root), &other_manifest)
        .unwrap();
    fs::write(
        other.payload_path_under(&cache.root).unwrap(),
        "broken payload",
    )
    .unwrap();
    cache.index.property_packages.push(other.clone());
    let provider = cache.provider();
    assert_eq!(provider.list_packages(at(10)).unwrap().len(), 2);
    assert!(
        provider.list_packages(at(10)).unwrap()[1]
            .availability
            .is_ok()
    );
    assert_eq!(
        provider
            .load_package(&other.package_id, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::InvalidData
    );
    assert!(
        provider
            .load_package(&cache.payload.package_id, at(10))
            .is_ok()
    );
    fs::write(other.manifest_path_under(&cache.root), "broken manifest").unwrap();
    assert_eq!(
        provider.list_packages(at(10)).unwrap()[1]
            .availability
            .as_ref()
            .unwrap_err()
            .kind,
        PackageFailureKind::InvalidData
    );
    assert!(
        provider
            .load_package(&cache.payload.package_id, at(10))
            .is_ok()
    );
}

#[test]
fn malformed_or_duplicate_index_is_not_hidden_by_selected_loading() {
    let mut cache = Cache::new();
    cache
        .index
        .property_packages
        .push(cache.index.property_packages[0].clone());
    assert!(CachedPropertyPackageProvider::new(&cache.root, &cache.index).is_err());
    cache.index.schema_version = 999;
    let error = CachedPropertyPackageProvider::new(&cache.root, &cache.index).unwrap_err();
    assert_eq!(error.kind, PackageFailureKind::InvalidData);
    assert_eq!(error.package_id, None);
}

#[test]
fn cached_expiration_and_remote_source_precede_file_access() {
    let mut cache = Cache::new();
    fs::remove_dir_all(&cache.root).unwrap();
    cache.index.property_packages[0].expires_at = Some(at(10));
    assert_eq!(
        cache
            .provider()
            .load_package(&cache.payload.package_id, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    cache.index.property_packages[0].expires_at = None;
    cache.index.property_packages[0] = StoredPropertyPackageRecord::new(
        &cache.payload.package_id,
        &cache.payload.version,
        StoredPropertyPackageSource::RemoteEvaluationService,
        "",
        0,
        at(1),
    );
    assert_eq!(
        cache
            .provider()
            .load_package(&cache.payload.package_id, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::UnsupportedSource
    );
}

#[test]
fn local_files_reject_expired_and_remote_before_payload_read() {
    let mut cache = Cache::new();
    fs::remove_file(cache.payload_path()).unwrap();
    cache.manifest.expires_at = Some(at(10));
    write_property_package_manifest(cache.manifest_path(), &cache.manifest).unwrap();
    assert_eq!(
        load_property_package_files(cache.manifest_path(), cache.payload_path(), at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    cache.manifest = StoredPropertyPackageManifest::new(
        &cache.payload.package_id,
        &cache.payload.version,
        StoredPropertyPackageSource::RemoteEvaluationService,
        cache.payload.component_ids(),
    );
    write_property_package_manifest(cache.manifest_path(), &cache.manifest).unwrap();
    assert_eq!(
        load_property_package_files(cache.manifest_path(), cache.payload_path(), at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::UnsupportedSource
    );
}

#[test]
fn memory_registration_rejects_batch_duplicates_and_component_mismatch() {
    let cache = Cache::new();
    let loaded = cache.load();
    let pair = (loaded.manifest().clone(), loaded.system().clone());
    assert_eq!(
        InMemoryPropertyPackageProvider::try_new_at(vec![pair.clone(), pair.clone()], at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::MetadataMismatch
    );
    let (mut manifest, system) = pair;
    manifest.component_ids.reverse();
    assert_eq!(
        InMemoryPropertyPackageProvider::try_new_at(vec![(manifest, system)], at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::MetadataMismatch
    );
}

#[test]
fn memory_registration_checks_expiration_source_and_remote_only_classification() {
    let cache = Cache::new();
    let loaded = cache.load();
    let system = loaded.system().clone();
    let mut manifest = loaded.manifest().clone();
    manifest.expires_at = Some(at(10));
    assert_eq!(
        LoadedPropertyPackage::from_runtime(manifest.clone(), system.clone(), at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    manifest.expires_at = None;
    manifest.classification = PropertyPackageClassification::RemoteOnly;
    assert_eq!(
        LoadedPropertyPackage::from_runtime(manifest.clone(), system.clone(), at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::UnsupportedSource
    );
    manifest.source = PropertyPackageSource::RemoteEvaluationService;
    assert_eq!(
        LoadedPropertyPackage::from_runtime(manifest, system, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::UnsupportedSource
    );
}

#[test]
fn frozen_context_survives_expiration_but_next_load_rejects_it() {
    let cache = Cache::new();
    let loaded = cache.load();
    let mut manifest = loaded.manifest().clone();
    manifest.expires_at = Some(at(11));
    let provider = InMemoryPropertyPackageProvider::try_new_at(
        vec![(manifest, loaded.system().clone())],
        at(10),
    )
    .unwrap();
    let context = prepare_local_context(
        provider
            .load_package(&cache.payload.package_id, at(10))
            .unwrap(),
    )
    .unwrap();
    let before = liquid_enthalpy(&context);
    assert_eq!(
        provider
            .load_package(&cache.payload.package_id, at(11))
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    assert_eq!(
        provider.list_packages(at(11)).unwrap()[0]
            .availability
            .as_ref()
            .unwrap_err()
            .kind,
        PackageFailureKind::Expired
    );
    assert_eq!(provider.registered_manifests().len(), 1);
    assert_eq!(liquid_enthalpy(&context), before);
}

#[test]
fn equivalent_sources_have_same_descriptor_fingerprint_and_numerical_results() {
    let cache = Cache::new();
    let cached = cache.load();
    let files =
        load_property_package_files(cache.manifest_path(), cache.payload_path(), at(10)).unwrap();
    let memory = LoadedPropertyPackage::from_runtime(
        cached.manifest().clone(),
        cached.system().clone(),
        at(10),
    )
    .unwrap();
    assert_eq!(cached.descriptor(), files.descriptor());
    assert_eq!(cached.descriptor(), memory.descriptor());
    assert_eq!(
        cached.provenance().payload_fingerprint,
        files.provenance().payload_fingerprint
    );
    assert_eq!(
        cached.provenance().payload_fingerprint,
        memory.provenance().payload_fingerprint
    );
    assert_eq!(cached.provenance().origin, PackageDataOrigin::Cache);
    assert_eq!(files.provenance().origin, PackageDataOrigin::LocalFiles);
    assert_eq!(memory.provenance().origin, PackageDataOrigin::InMemory);
    assert!(cached.manifest().hash.is_empty());
    let values = [cached, files, memory]
        .map(|loaded| liquid_enthalpy(&prepare_local_context(loaded).unwrap()));
    assert_eq!(values, [values[0]; 3]);
}

#[test]
fn frozen_context_is_independent_of_file_replacement_and_deletion() {
    let mut cache = Cache::new();
    let provider = cache.provider();
    let old = prepare_local_context(cache.load()).unwrap();
    let before = liquid_enthalpy(&old);
    cache.payload.components[0].liquid_heat_capacity_j_per_mol_k = Some(350.0);
    cache.write();
    let new = prepare_local_context(
        provider
            .load_package(&cache.payload.package_id, at(10))
            .unwrap(),
    )
    .unwrap();
    assert_ne!(liquid_enthalpy(&new), before);
    assert_ne!(
        old.provenance().payload_fingerprint,
        new.provenance().payload_fingerprint
    );
    fs::remove_dir_all(&cache.root).unwrap();
    assert_eq!(liquid_enthalpy(&old), before);
    assert_eq!(
        provider
            .load_package(&cache.payload.package_id, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::ResourceUnavailable
    );
}

#[test]
fn explicit_memory_replacement_preserves_existing_context() {
    let cache = Cache::new();
    let loaded = cache.load();
    let mut registry = InMemoryPropertyPackageProvider::default();
    registry.insert(loaded.clone());
    let old = prepare_local_context(
        registry
            .load_package(&cache.payload.package_id, at(10))
            .unwrap(),
    )
    .unwrap();
    let mut changed_system = loaded.system().clone();
    changed_system.components[0].liquid_heat_capacity_j_per_mol_k = Some(350.0);
    registry.insert(
        LoadedPropertyPackage::from_runtime(loaded.manifest().clone(), changed_system, at(10))
            .unwrap(),
    );
    let new = prepare_local_context(
        registry
            .load_package(&cache.payload.package_id, at(10))
            .unwrap(),
    )
    .unwrap();
    assert_ne!(liquid_enthalpy(&old), liquid_enthalpy(&new));
    assert_eq!(registry.registered_manifests().len(), 1);
    for _ in 0..3 {
        assert_eq!(
            liquid_enthalpy(&old),
            liquid_enthalpy(&prepare_local_context(loaded.clone()).unwrap())
        );
    }
}

#[test]
fn typed_validation_stages_preserve_store_errors_and_first_failure() {
    let cache = Cache::new();
    let check =
        |manifest: StoredPropertyPackageManifest, payload: StoredPropertyPackagePayload, kind| {
            let legacy = payload.validate_against_manifest(&manifest).unwrap_err();
            let detailed = payload
                .validate_against_manifest_detailed(&manifest)
                .unwrap_err();
            assert_eq!(legacy, detailed.error);
            let loaded = LoadedPropertyPackage::from_stored(
                manifest,
                payload,
                PackageDataOrigin::LocalFiles,
                at(10),
            )
            .unwrap_err();
            assert_eq!(loaded.kind, kind);
            assert_eq!(loaded.source, legacy);
        };
    let mut payload = cache.payload.clone();
    payload.package_id = "other".into();
    check(
        cache.manifest.clone(),
        payload,
        PackageFailureKind::MetadataMismatch,
    );
    let mut payload = cache.payload.clone();
    payload.version = "other".into();
    check(
        cache.manifest.clone(),
        payload,
        PackageFailureKind::MetadataMismatch,
    );
    let mut payload = cache.payload.clone();
    payload.components.reverse();
    check(
        cache.manifest.clone(),
        payload,
        PackageFailureKind::MetadataMismatch,
    );
    let mut payload = cache.payload.clone();
    payload.components[1] = payload.components[0].clone();
    check(
        cache.manifest.clone(),
        payload,
        PackageFailureKind::InvalidData,
    );
    let mut payload = cache.payload.clone();
    payload.schema_version = 999;
    payload.package_id = "other".into();
    check(
        cache.manifest.clone(),
        payload,
        PackageFailureKind::InvalidData,
    );
    let mut manifest = cache.manifest.clone();
    manifest.hash = "sha256:wrong".into();
    check(
        manifest,
        cache.payload.clone(),
        PackageFailureKind::IntegrityMismatch,
    );
    let mut manifest = cache.manifest.clone();
    manifest.size_bytes = 1;
    check(
        manifest,
        cache.payload.clone(),
        PackageFailureKind::IntegrityMismatch,
    );
}

#[test]
fn canonical_fingerprint_ignores_file_whitespace_and_checks_declared_hash() {
    let mut cache = Cache::new();
    let integrity = property_package_payload_integrity(&cache.payload).unwrap();
    cache.manifest.hash = integrity.hash.clone();
    cache.manifest.size_bytes = integrity.size_bytes;
    cache.index.property_packages[0].hash = integrity.hash.clone();
    cache.index.property_packages[0].size_bytes = integrity.size_bytes;
    cache.write();
    let json = fs::read_to_string(cache.payload_path()).unwrap();
    fs::write(cache.payload_path(), format!("\n\n{json}\n ")).unwrap();
    assert_eq!(
        cache.load().provenance().payload_fingerprint,
        integrity.hash
    );
    cache.payload.components[0].name = "Changed".into();
    write_property_package_payload(cache.payload_path(), &cache.payload).unwrap();
    assert_eq!(
        cache
            .provider()
            .load_package(&cache.payload.package_id, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::IntegrityMismatch
    );
}

#[test]
fn descriptor_reports_gaps_without_eager_numerical_evaluation() {
    let mut cache = Cache::new();
    cache.payload.components[0].antoine = None;
    cache.payload.components[0].liquid_heat_capacity_j_per_mol_k = None;
    cache.write();
    let loaded = cache.load();
    assert_eq!(
        loaded.descriptor().components[0].missing_data,
        vec![
            ComponentDataKind::Antoine,
            ComponentDataKind::LiquidHeatCapacity
        ]
    );
    assert!(loaded.descriptor().components[1].missing_data.is_empty());
    let context = prepare_local_context(loaded).unwrap();
    assert!(
        context
            .thermo()
            .phase_molar_enthalpy(&PhaseThermoState::new(
                PhaseLabel::Vapor,
                300.0,
                100_000.0,
                vec![0.5, 0.5]
            ))
            .is_ok()
    );
    assert!(
        context
            .thermo()
            .phase_molar_enthalpy(&PhaseThermoState::new(
                PhaseLabel::Liquid,
                300.0,
                100_000.0,
                vec![0.5, 0.5]
            ))
            .is_err()
    );
    assert!(
        context
            .thermo()
            .estimate_k_values(&ThermoState::new(300.0, 100_000.0, vec![0.5, 0.5]))
            .is_err()
    );
}

#[test]
fn empty_components_are_rejected_and_single_component_descriptor_preserves_identity() {
    let cache = Cache::new();
    let mut payload = cache.payload.clone();
    payload.components.clear();
    let mut manifest = cache.manifest.clone();
    manifest.component_ids.clear();
    assert_eq!(
        LoadedPropertyPackage::from_stored(manifest, payload, PackageDataOrigin::InMemory, at(10))
            .unwrap_err()
            .kind,
        PackageFailureKind::InvalidData
    );
    let mut payload = cache.payload.clone();
    payload.components.truncate(1);
    let mut manifest = cache.manifest.clone();
    manifest.component_ids.truncate(1);
    let loaded =
        LoadedPropertyPackage::from_stored(manifest, payload, PackageDataOrigin::InMemory, at(10))
            .unwrap();
    assert_eq!(loaded.descriptor().components.len(), 1);
    assert_eq!(loaded.descriptor().components[0].id.as_str(), "methane");
}
