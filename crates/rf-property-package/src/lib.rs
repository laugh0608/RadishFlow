//! Property package resources and local runtime assembly shared by application adapters.
//! Numerical crates do not depend on this layer or on storage / authorization caches.

mod conversion;
mod descriptor;
mod error;
mod manifest;
mod provider;
mod runtime;

pub use descriptor::{
    ComponentDataKind, ComponentDescriptor, PackageDataOrigin, PackageProvenance,
    PropertyPackageDescriptor, ThermoCapability,
};
pub use error::{PackageFailureKind, PackageLoadError, PackageResult};
pub use manifest::{PropertyPackageClassification, PropertyPackageManifest, PropertyPackageSource};
pub use provider::{
    CachedPropertyPackageProvider, InMemoryPropertyPackageProvider, PackageListing,
    PropertyPackageProvider, load_property_package_files,
};
pub use runtime::{LoadedPropertyPackage, LocalThermoContext, prepare_local_context};

#[cfg(test)]
mod legacy_tests;

#[cfg(test)]
mod tests;
