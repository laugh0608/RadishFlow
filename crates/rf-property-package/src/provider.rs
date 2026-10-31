use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rf_store::{
    StoredAuthCacheIndex, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    read_property_package_manifest_detailed, read_property_package_payload_detailed,
};
use rf_thermo::ThermoSystem;
use rf_types::RfError;

use crate::conversion::property_package_manifest_from_stored;
use crate::runtime::check_local_availability;
use crate::{
    LoadedPropertyPackage, PackageDataOrigin, PackageFailureKind, PackageLoadError, PackageResult,
    PropertyPackageManifest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageListing {
    pub package_id: String,
    /// Describes manifest availability only; payload validity is checked on load.
    pub availability: PackageResult<PropertyPackageManifest>,
}

pub trait PropertyPackageProvider {
    fn list_packages(&self, at: SystemTime) -> PackageResult<Vec<PackageListing>>;
    fn load_package(
        &self,
        package_id: &str,
        at: SystemTime,
    ) -> PackageResult<LoadedPropertyPackage>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryPropertyPackageProvider {
    packages: BTreeMap<String, LoadedPropertyPackage>,
}

impl InMemoryPropertyPackageProvider {
    pub fn try_new(entries: Vec<(PropertyPackageManifest, ThermoSystem)>) -> PackageResult<Self> {
        Self::try_new_at(entries, SystemTime::now())
    }

    pub fn try_new_at(
        entries: Vec<(PropertyPackageManifest, ThermoSystem)>,
        at: SystemTime,
    ) -> PackageResult<Self> {
        let mut packages = BTreeMap::new();
        for (manifest, system) in entries {
            let id = manifest.package_id.clone();
            if packages.contains_key(&id) {
                return Err(PackageLoadError::new(
                    PackageFailureKind::MetadataMismatch,
                    Some(&id),
                    RfError::duplicate_id("property package", &id),
                ));
            }
            packages.insert(
                id,
                LoadedPropertyPackage::from_runtime(manifest, system, at)?,
            );
        }
        Ok(Self { packages })
    }

    /// Explicit registration / replacement, unlike duplicate entries in a batch.
    /// Existing contexts own their data and are unaffected by replacement.
    pub fn insert(&mut self, package: LoadedPropertyPackage) {
        self.packages
            .insert(package.manifest().package_id.clone(), package);
    }

    /// Metadata of registered packages, including ones that have since expired.
    /// Kept separate from availability discovery for the existing FFI manifest protocol.
    pub fn registered_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|package| package.manifest().clone())
            .collect()
    }
}

impl PropertyPackageProvider for InMemoryPropertyPackageProvider {
    fn list_packages(&self, at: SystemTime) -> PackageResult<Vec<PackageListing>> {
        Ok(self
            .packages
            .iter()
            .map(|(id, package)| PackageListing {
                package_id: id.clone(),
                availability: package
                    .check_available(at)
                    .map(|()| package.manifest().clone()),
            })
            .collect())
    }

    fn load_package(
        &self,
        package_id: &str,
        at: SystemTime,
    ) -> PackageResult<LoadedPropertyPackage> {
        let package = self
            .packages
            .get(package_id)
            .ok_or_else(|| missing_package(package_id))?;
        package.check_available(at)?;
        Ok(package.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CachedPropertyPackageProvider {
    cache_root: PathBuf,
    index: StoredAuthCacheIndex,
}

impl CachedPropertyPackageProvider {
    pub fn new(cache_root: impl AsRef<Path>, index: &StoredAuthCacheIndex) -> PackageResult<Self> {
        index
            .validate()
            .map_err(|error| PackageLoadError::new(PackageFailureKind::InvalidData, None, error))?;
        let mut ids = std::collections::BTreeSet::new();
        for record in &index.property_packages {
            if !ids.insert(&record.package_id) {
                return Err(PackageLoadError::new(
                    PackageFailureKind::MetadataMismatch,
                    Some(&record.package_id),
                    RfError::invalid_input(format!(
                        "duplicate cached property package `{}` found in auth cache index",
                        record.package_id,
                    )),
                ));
            }
        }
        Ok(Self {
            cache_root: cache_root.as_ref().to_path_buf(),
            index: index.clone(),
        })
    }

    fn load_manifest(
        &self,
        record: &StoredPropertyPackageRecord,
        at: SystemTime,
    ) -> PackageResult<rf_store::StoredPropertyPackageManifest> {
        let id = record.package_id.as_str();
        // Eligibility comes before any file access, including the manifest.
        if record.is_expired_at(at) {
            return Err(PackageLoadError::new(
                PackageFailureKind::Expired,
                Some(id),
                RfError::invalid_input(format!("property package `{id}` has expired")),
            ));
        }
        if record.source == StoredPropertyPackageSource::RemoteEvaluationService {
            return Err(PackageLoadError::new(
                PackageFailureKind::UnsupportedSource,
                Some(id),
                RfError::invalid_input(format!(
                    "property package `{id}` requires remote evaluation and cannot be assembled locally"
                )),
            ));
        }
        let manifest =
            read_property_package_manifest_detailed(record.manifest_path_under(&self.cache_root))
                .map_err(|error| PackageLoadError::stored(id, error))?;
        manifest
            .validate_against_record_detailed(record)
            .map_err(|error| PackageLoadError::stored(id, error))?;
        Ok(manifest)
    }
}

impl PropertyPackageProvider for CachedPropertyPackageProvider {
    fn list_packages(&self, at: SystemTime) -> PackageResult<Vec<PackageListing>> {
        Ok(self
            .index
            .property_packages
            .iter()
            .map(|record| PackageListing {
                package_id: record.package_id.clone(),
                availability: self.load_manifest(record, at).and_then(|manifest| {
                    property_package_manifest_from_stored(manifest).map_err(|error| {
                        PackageLoadError::new(
                            PackageFailureKind::InvalidData,
                            Some(&record.package_id),
                            error,
                        )
                    })
                }),
            })
            .collect())
    }

    fn load_package(
        &self,
        package_id: &str,
        at: SystemTime,
    ) -> PackageResult<LoadedPropertyPackage> {
        let record = self
            .index
            .property_packages
            .iter()
            .find(|record| record.package_id == package_id)
            .ok_or_else(|| missing_package(package_id))?;
        let manifest = self.load_manifest(record, at)?;
        let path = record.payload_path_under(&self.cache_root).ok_or_else(|| {
            PackageLoadError::new(
                PackageFailureKind::InvalidData,
                Some(package_id),
                RfError::invalid_input(format!(
                    "stored property package `{package_id}` is missing a local payload path"
                )),
            )
        })?;
        let payload = read_property_package_payload_detailed(path)
            .map_err(|error| PackageLoadError::stored(package_id, error))?;
        LoadedPropertyPackage::from_stored(manifest, payload, PackageDataOrigin::Cache, at)
    }
}

pub fn load_property_package_files(
    manifest_path: impl AsRef<Path>,
    payload_path: impl AsRef<Path>,
    at: SystemTime,
) -> PackageResult<LoadedPropertyPackage> {
    let manifest = read_property_package_manifest_detailed(manifest_path).map_err(|error| {
        // Identity is not known until the manifest has been read.
        let mut failure = PackageLoadError::stored("", error);
        failure.package_id = None;
        failure
    })?;
    let id = manifest.package_id.as_str();
    let runtime_manifest = property_package_manifest_from_stored(manifest.clone())
        .map_err(|error| PackageLoadError::new(PackageFailureKind::InvalidData, Some(id), error))?;
    check_local_availability(&runtime_manifest, at)?;
    let payload = read_property_package_payload_detailed(payload_path)
        .map_err(|error| PackageLoadError::stored(id, error))?;
    LoadedPropertyPackage::from_stored(manifest, payload, PackageDataOrigin::LocalFiles, at)
}

fn missing_package(id: &str) -> PackageLoadError {
    PackageLoadError::new(
        PackageFailureKind::Missing,
        Some(id),
        RfError::missing_entity("property package", id),
    )
}
