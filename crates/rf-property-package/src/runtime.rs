use std::time::SystemTime;

use rf_flash::{PlaceholderTpFlashSolver, TpFlashSolver};
use rf_store::{
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, property_package_payload_integrity,
};
use rf_thermo::{PlaceholderThermoProvider, ThermoProvider, ThermoSystem};
use rf_types::RfError;

use crate::conversion::{
    manifest_to_stored, property_package_manifest_from_stored, system_to_stored,
    thermo_system_from_stored_payload,
};
use crate::{
    PackageDataOrigin, PackageFailureKind, PackageLoadError, PackageProvenance, PackageResult,
    PropertyPackageClassification, PropertyPackageDescriptor, PropertyPackageManifest,
    PropertyPackageSource,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedPropertyPackage {
    descriptor: PropertyPackageDescriptor,
    system: ThermoSystem,
    provenance: PackageProvenance,
}

impl LoadedPropertyPackage {
    pub fn from_stored(
        manifest: StoredPropertyPackageManifest,
        payload: StoredPropertyPackagePayload,
        origin: PackageDataOrigin,
        at: SystemTime,
    ) -> PackageResult<Self> {
        let id = manifest.package_id.as_str();
        payload
            .validate_against_manifest_detailed(&manifest)
            .map_err(|error| PackageLoadError::stored(id, error))?;
        let runtime_manifest =
            property_package_manifest_from_stored(manifest.clone()).map_err(|error| {
                PackageLoadError::new(PackageFailureKind::InvalidData, Some(id), error)
            })?;
        check_local_availability(&runtime_manifest, at)?;
        let integrity = property_package_payload_integrity(&payload).map_err(|error| {
            PackageLoadError::new(PackageFailureKind::InvalidData, Some(id), error)
        })?;
        let provenance = PackageProvenance {
            origin,
            manifest_schema_version: manifest.schema_version,
            payload_schema_version: payload.schema_version,
            payload_fingerprint: integrity.hash,
            canonical_payload_size_bytes: integrity.size_bytes,
            // Workspace crates use the same declared version; this does not identify a git build.
            thermo_implementation: concat!(
                "rf-thermo::PlaceholderThermoProvider@",
                env!("CARGO_PKG_VERSION")
            ),
            flash_implementation: concat!(
                "rf-flash::PlaceholderTpFlashSolver@",
                env!("CARGO_PKG_VERSION")
            ),
        };
        let system = thermo_system_from_stored_payload(payload);
        Ok(Self {
            descriptor: PropertyPackageDescriptor::new(runtime_manifest, &system),
            system,
            provenance,
        })
    }

    pub fn from_runtime(
        manifest: PropertyPackageManifest,
        system: ThermoSystem,
        at: SystemTime,
    ) -> PackageResult<Self> {
        check_local_availability(&manifest, at)?;
        let payload = system_to_stored(&manifest, &system);
        Self::from_stored(
            manifest_to_stored(&manifest),
            payload,
            PackageDataOrigin::InMemory,
            at,
        )
    }

    pub fn descriptor(&self) -> &PropertyPackageDescriptor {
        &self.descriptor
    }
    pub fn manifest(&self) -> &PropertyPackageManifest {
        &self.descriptor.manifest
    }
    pub fn system(&self) -> &ThermoSystem {
        &self.system
    }
    pub fn provenance(&self) -> &PackageProvenance {
        &self.provenance
    }

    pub(crate) fn check_available(&self, at: SystemTime) -> PackageResult<()> {
        check_local_availability(self.manifest(), at)
    }
}

pub(crate) fn check_local_availability(
    manifest: &PropertyPackageManifest,
    at: SystemTime,
) -> PackageResult<()> {
    let id = manifest.package_id.as_str();
    if manifest.expires_at.is_some_and(|expires| at >= expires) {
        return Err(PackageLoadError::new(
            PackageFailureKind::Expired,
            Some(id),
            RfError::invalid_input(format!("property package `{id}` has expired")),
        ));
    }
    if manifest.source == PropertyPackageSource::RemoteEvaluationService
        || manifest.classification == PropertyPackageClassification::RemoteOnly
    {
        return Err(PackageLoadError::new(
            PackageFailureKind::UnsupportedSource,
            Some(id),
            RfError::invalid_input(format!(
                "property package `{id}` requires remote evaluation and cannot be assembled locally"
            )),
        ));
    }
    Ok(())
}

/// Owns numerical data; contains no file handles, source callbacks or mutable registry.
pub struct LocalThermoContext {
    thermo: PlaceholderThermoProvider,
    tp_flash: PlaceholderTpFlashSolver,
    descriptor: PropertyPackageDescriptor,
    provenance: PackageProvenance,
}

impl LocalThermoContext {
    pub fn thermo(&self) -> &dyn ThermoProvider {
        &self.thermo
    }
    pub fn tp_flash(&self) -> &dyn TpFlashSolver {
        &self.tp_flash
    }
    pub fn descriptor(&self) -> &PropertyPackageDescriptor {
        &self.descriptor
    }
    pub fn provenance(&self) -> &PackageProvenance {
        &self.provenance
    }
}

pub fn prepare_local_context(package: LoadedPropertyPackage) -> PackageResult<LocalThermoContext> {
    // All supported stored methods currently resolve to this existing implementation.
    // Structural / source checks happen on load; data gaps retain execution-stage errors.
    Ok(LocalThermoContext {
        thermo: PlaceholderThermoProvider::new(package.system),
        tp_flash: PlaceholderTpFlashSolver,
        descriptor: package.descriptor,
        provenance: package.provenance,
    })
}
