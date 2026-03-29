use std::collections::BTreeMap;
use std::time::SystemTime;

use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredLiquidPhaseModel,
    StoredPropertyPackageClassification, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageSource, StoredThermoComponent,
    StoredThermoMethod, StoredVaporPhaseModel, read_property_package_manifest,
    read_property_package_payload,
};
use rf_types::{ComponentId, PhaseLabel, RfError, RfResult};

#[derive(Debug, Clone, PartialEq)]
pub struct AntoineCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl AntoineCoefficients {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoComponent {
    pub id: ComponentId,
    pub name: String,
    pub antoine: Option<AntoineCoefficients>,
    pub liquid_heat_capacity_j_per_mol_k: Option<f64>,
    pub vapor_heat_capacity_j_per_mol_k: Option<f64>,
}

impl ThermoComponent {
    pub fn new(id: impl Into<ComponentId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            antoine: None,
            liquid_heat_capacity_j_per_mol_k: None,
            vapor_heat_capacity_j_per_mol_k: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidPhaseModel {
    IdealSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporPhaseModel {
    IdealGas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermoMethod {
    pub liquid_phase_model: LiquidPhaseModel,
    pub vapor_phase_model: VaporPhaseModel,
}

impl Default for ThermoMethod {
    fn default() -> Self {
        Self {
            liquid_phase_model: LiquidPhaseModel::IdealSolution,
            vapor_phase_model: VaporPhaseModel::IdealGas,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoSystem {
    pub components: Vec<ThermoComponent>,
    pub method: ThermoMethod,
}

impl ThermoSystem {
    pub fn new(components: Vec<ThermoComponent>) -> Self {
        Self {
            components,
            method: ThermoMethod::default(),
        }
    }

    pub fn binary(components: [ThermoComponent; 2]) -> Self {
        Self::new(Vec::from(components))
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .map(|component| component.id.clone())
            .collect()
    }

    pub fn validate_mole_fractions(&self, mole_fractions: &[f64]) -> RfResult<()> {
        if self.components.is_empty() {
            return Err(RfError::thermo(
                "thermo system must contain at least one component",
            ));
        }

        if mole_fractions.len() != self.component_count() {
            return Err(RfError::invalid_input(format!(
                "expected {} mole fractions, received {}",
                self.component_count(),
                mole_fractions.len()
            )));
        }

        if mole_fractions.iter().any(|value| *value < 0.0) {
            return Err(RfError::invalid_input(
                "mole fractions must be non-negative",
            ));
        }

        let sum = mole_fractions.iter().sum::<f64>();
        if sum <= 0.0 {
            return Err(RfError::invalid_input(
                "mole fractions must sum to a positive value",
            ));
        }

        Ok(())
    }

    pub fn validate_state(&self, state: &ThermoState) -> RfResult<()> {
        if state.temperature_k <= 0.0 {
            return Err(RfError::invalid_input(
                "temperature must be greater than zero kelvin",
            ));
        }

        if state.pressure_pa <= 0.0 {
            return Err(RfError::invalid_input(
                "pressure must be greater than zero pascal",
            ));
        }

        self.validate_mole_fractions(&state.overall_mole_fractions)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoState {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl ThermoState {
    pub fn new(temperature_k: f64, pressure_pa: f64, overall_mole_fractions: Vec<f64>) -> Self {
        Self {
            temperature_k,
            pressure_pa,
            overall_mole_fractions,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseThermoState {
    pub label: PhaseLabel,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub mole_fractions: Vec<f64>,
}

impl PhaseThermoState {
    pub fn new(
        label: PhaseLabel,
        temperature_k: f64,
        pressure_pa: f64,
        mole_fractions: Vec<f64>,
    ) -> Self {
        Self {
            label,
            temperature_k,
            pressure_pa,
            mole_fractions,
        }
    }
}

pub trait ThermoProvider {
    fn system(&self) -> &ThermoSystem;

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>>;

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64>;
}

#[derive(Debug, Clone)]
pub struct PlaceholderThermoProvider {
    system: ThermoSystem,
}

impl PlaceholderThermoProvider {
    pub fn new(system: ThermoSystem) -> Self {
        Self { system }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageSource {
    LocalBundled,
    RemoteDerivedPackage,
    RemoteEvaluationService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageClassification {
    Derived,
    RemoteOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageManifest {
    pub package_id: String,
    pub version: String,
    pub classification: PropertyPackageClassification,
    pub source: PropertyPackageSource,
    pub hash: String,
    pub size_bytes: u64,
    pub component_ids: Vec<ComponentId>,
    pub expires_at: Option<SystemTime>,
}

impl PropertyPackageManifest {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        source: PropertyPackageSource,
        component_ids: Vec<ComponentId>,
    ) -> Self {
        let classification = match source {
            PropertyPackageSource::RemoteEvaluationService => {
                PropertyPackageClassification::RemoteOnly
            }
            PropertyPackageSource::LocalBundled | PropertyPackageSource::RemoteDerivedPackage => {
                PropertyPackageClassification::Derived
            }
        };

        Self {
            package_id: package_id.into(),
            version: version.into(),
            classification,
            source,
            hash: String::new(),
            size_bytes: 0,
            component_ids,
            expires_at: None,
        }
    }
}

pub trait PropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest>;

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryPropertyPackageProvider {
    packages: BTreeMap<String, (PropertyPackageManifest, ThermoSystem)>,
}

impl InMemoryPropertyPackageProvider {
    pub fn new(entries: Vec<(PropertyPackageManifest, ThermoSystem)>) -> Self {
        let packages = entries
            .into_iter()
            .map(|(manifest, system)| (manifest.package_id.clone(), (manifest, system)))
            .collect();

        Self { packages }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CachedPropertyPackageProvider {
    packages: BTreeMap<String, (PropertyPackageManifest, ThermoSystem)>,
}

impl CachedPropertyPackageProvider {
    pub fn new(
        cache_root: impl AsRef<std::path::Path>,
        auth_cache_index: &StoredAuthCacheIndex,
    ) -> RfResult<Self> {
        Self::new_at(cache_root, auth_cache_index, SystemTime::now())
    }

    pub fn new_at(
        cache_root: impl AsRef<std::path::Path>,
        auth_cache_index: &StoredAuthCacheIndex,
        now: SystemTime,
    ) -> RfResult<Self> {
        auth_cache_index.validate()?;

        let cache_root = cache_root.as_ref();
        let mut packages = BTreeMap::new();

        for record in &auth_cache_index.property_packages {
            if record.is_expired_at(now) {
                continue;
            }

            if matches!(
                record.source,
                StoredPropertyPackageSource::RemoteEvaluationService
            ) {
                continue;
            }

            let manifest_path = record.manifest_path_under(cache_root);
            let stored_manifest = read_property_package_manifest(&manifest_path)?;
            stored_manifest.validate_against_record(record)?;

            let payload_path = record.payload_path_under(cache_root).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "stored property package `{}` is missing a local payload path",
                    record.package_id
                ))
            })?;
            let stored_payload = read_property_package_payload(&payload_path)?;
            stored_payload.validate_against_manifest(&stored_manifest)?;

            let runtime_manifest = property_package_manifest_from_stored(stored_manifest)?;
            let runtime_system = thermo_system_from_stored_payload(stored_payload);
            let package_id = runtime_manifest.package_id.clone();

            if packages
                .insert(package_id.clone(), (runtime_manifest, runtime_system))
                .is_some()
            {
                return Err(RfError::invalid_input(format!(
                    "duplicate cached property package `{package_id}` found in auth cache index"
                )));
            }
        }

        Ok(Self { packages })
    }
}

impl PropertyPackageProvider for InMemoryPropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|(manifest, _)| manifest.clone())
            .collect()
    }

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem> {
        self.packages
            .get(package_id)
            .map(|(_, system)| system.clone())
            .ok_or_else(|| RfError::missing_entity("property package", package_id))
    }
}

impl PropertyPackageProvider for CachedPropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|(manifest, _)| manifest.clone())
            .collect()
    }

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem> {
        self.packages
            .get(package_id)
            .map(|(_, system)| system.clone())
            .ok_or_else(|| RfError::missing_entity("property package", package_id))
    }
}

impl ThermoProvider for PlaceholderThermoProvider {
    fn system(&self) -> &ThermoSystem {
        &self.system
    }

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>> {
        self.system.validate_state(state)?;

        Err(RfError::not_implemented(
            "K-value estimation is not implemented yet",
        ))
    }

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64> {
        if state.temperature_k <= 0.0 {
            return Err(RfError::invalid_input(
                "temperature must be greater than zero kelvin",
            ));
        }

        if state.pressure_pa <= 0.0 {
            return Err(RfError::invalid_input(
                "pressure must be greater than zero pascal",
            ));
        }

        self.system.validate_mole_fractions(&state.mole_fractions)?;

        Err(RfError::not_implemented(
            "phase enthalpy evaluation is not implemented yet",
        ))
    }
}

fn property_package_manifest_from_stored(
    stored_manifest: StoredPropertyPackageManifest,
) -> RfResult<PropertyPackageManifest> {
    stored_manifest.validate()?;

    let source = property_package_source_from_stored(stored_manifest.source);
    let classification =
        property_package_classification_from_stored(stored_manifest.classification);

    let mut manifest = PropertyPackageManifest::new(
        stored_manifest.package_id,
        stored_manifest.version,
        source,
        stored_manifest.component_ids,
    );
    manifest.classification = classification;
    manifest.hash = stored_manifest.hash;
    manifest.size_bytes = stored_manifest.size_bytes;
    manifest.expires_at = stored_manifest.expires_at;
    Ok(manifest)
}

fn thermo_system_from_stored_payload(payload: StoredPropertyPackagePayload) -> ThermoSystem {
    let components = payload
        .components
        .into_iter()
        .map(thermo_component_from_stored)
        .collect();

    ThermoSystem {
        components,
        method: thermo_method_from_stored(payload.method),
    }
}

fn thermo_component_from_stored(component: StoredThermoComponent) -> ThermoComponent {
    ThermoComponent {
        id: component.id,
        name: component.name,
        antoine: component.antoine.map(antoine_coefficients_from_stored),
        liquid_heat_capacity_j_per_mol_k: component.liquid_heat_capacity_j_per_mol_k,
        vapor_heat_capacity_j_per_mol_k: component.vapor_heat_capacity_j_per_mol_k,
    }
}

fn antoine_coefficients_from_stored(
    coefficients: StoredAntoineCoefficients,
) -> AntoineCoefficients {
    AntoineCoefficients::new(coefficients.a, coefficients.b, coefficients.c)
}

fn thermo_method_from_stored(method: StoredThermoMethod) -> ThermoMethod {
    ThermoMethod {
        liquid_phase_model: liquid_phase_model_from_stored(method.liquid_phase_model),
        vapor_phase_model: vapor_phase_model_from_stored(method.vapor_phase_model),
    }
}

fn liquid_phase_model_from_stored(model: StoredLiquidPhaseModel) -> LiquidPhaseModel {
    match model {
        StoredLiquidPhaseModel::IdealSolution => LiquidPhaseModel::IdealSolution,
    }
}

fn vapor_phase_model_from_stored(model: StoredVaporPhaseModel) -> VaporPhaseModel {
    match model {
        StoredVaporPhaseModel::IdealGas => VaporPhaseModel::IdealGas,
    }
}

fn property_package_source_from_stored(
    source: StoredPropertyPackageSource,
) -> PropertyPackageSource {
    match source {
        StoredPropertyPackageSource::LocalBundled => PropertyPackageSource::LocalBundled,
        StoredPropertyPackageSource::RemoteDerivedPackage => {
            PropertyPackageSource::RemoteDerivedPackage
        }
        StoredPropertyPackageSource::RemoteEvaluationService => {
            PropertyPackageSource::RemoteEvaluationService
        }
    }
}

fn property_package_classification_from_stored(
    classification: StoredPropertyPackageClassification,
) -> PropertyPackageClassification {
    match classification {
        StoredPropertyPackageClassification::Derived => PropertyPackageClassification::Derived,
        StoredPropertyPackageClassification::RemoteOnly => {
            PropertyPackageClassification::RemoteOnly
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageManifest,
        StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
        StoredThermoComponent, read_property_package_manifest, read_property_package_payload,
        write_property_package_manifest, write_property_package_payload,
    };

    use super::{
        CachedPropertyPackageProvider, InMemoryPropertyPackageProvider,
        PropertyPackageClassification, PropertyPackageManifest, PropertyPackageProvider,
        PropertyPackageSource, ThermoComponent, ThermoSystem,
    };
    use rf_types::ComponentId;

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
        let provider = InMemoryPropertyPackageProvider::new(vec![(manifest, system.clone())]);

        let manifests = provider.list_manifests();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].package_id, "methane-basic-v1");

        let loaded = provider
            .load_system("methane-basic-v1")
            .expect("expected thermo system");
        assert_eq!(loaded, system);
    }

    #[test]
    fn package_provider_reports_missing_package() {
        let provider = InMemoryPropertyPackageProvider::default();
        let error = provider
            .load_system("missing-package")
            .expect_err("expected missing package error");

        assert_eq!(error.code().as_str(), "missing_entity");
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
            "sha256:pkg-1",
            512,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(800));
        let manifest = {
            let mut manifest = StoredPropertyPackageManifest::new(
                "methane-basic-v1",
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                vec![ComponentId::new("methane")],
            );
            manifest.hash = "sha256:pkg-1".to_string();
            manifest.size_bytes = 512;
            manifest.expires_at = Some(timestamp(800));
            manifest
        };
        let payload = StoredPropertyPackagePayload::new(
            "methane-basic-v1",
            "2026.03.1",
            vec![StoredThermoComponent::new(
                ComponentId::new("methane"),
                "Methane",
            )],
        );

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

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(700))
            .expect("expected cached provider");
        let manifests = provider.list_manifests();

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].package_id, "methane-basic-v1");
        assert_eq!(manifests[0].hash, "sha256:pkg-1");
        let system = provider
            .load_system("methane-basic-v1")
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

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(300))
            .expect("expected cached provider");

        assert!(provider.list_manifests().is_empty());
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
            256,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(500));
        let mut manifest = StoredPropertyPackageManifest::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![ComponentId::new("methane")],
        );
        manifest.hash = "sha256:actual".to_string();
        manifest.size_bytes = 256;
        manifest.expires_at = Some(timestamp(500));

        write_property_package_manifest(record.manifest_path_under(&root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(&root)
                .expect("expected payload path"),
            &StoredPropertyPackagePayload::new(
                "pkg-1",
                "2026.03.1",
                vec![StoredThermoComponent::new(
                    ComponentId::new("methane"),
                    "Methane",
                )],
            ),
        )
        .expect("expected payload write");
        index.property_packages.push(record);

        let error = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(300))
            .expect_err("expected manifest mismatch");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("does not match manifest hash"));
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

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(150))
            .expect("expected cached provider");
        let system = provider
            .load_system("binary-hydrocarbon-lite-v1")
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
}
