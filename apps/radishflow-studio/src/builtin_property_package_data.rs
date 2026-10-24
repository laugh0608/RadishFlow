//! Immutable authoring metadata derived from the same bundled payload used by bootstrap.
//! No filesystem access or numerical evaluation occurs when querying these descriptions.

use std::sync::LazyLock;
use std::time::UNIX_EPOCH;

use rf_property_package::{LoadedPropertyPackage, PackageDataOrigin, PropertyPackageDescriptor};
use rf_store::{
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageSource,
};

struct BundledPackageData {
    payload: StoredPropertyPackagePayload,
    package: LoadedPropertyPackage,
}

static BINARY_HYDROCARBON: LazyLock<BundledPackageData> = LazyLock::new(|| {
    let payload = crate::parse_property_package_download_json(include_str!(
        "../../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json"
    )).expect("valid bundled property package envelope")
        .to_stored_payload().expect("valid bundled property package payload");
    let manifest = StoredPropertyPackageManifest::new(
        &payload.package_id,
        &payload.version,
        StoredPropertyPackageSource::LocalBundled,
        payload.component_ids(),
    );
    let package = LoadedPropertyPackage::from_stored(
        manifest,
        payload.clone(),
        PackageDataOrigin::Bundled,
        UNIX_EPOCH,
    )
    .expect("consistent bundled property package");
    BundledPackageData { payload, package }
});

pub(crate) fn descriptor() -> &'static PropertyPackageDescriptor {
    BINARY_HYDROCARBON.package.descriptor()
}

pub(crate) fn payload() -> &'static StoredPropertyPackagePayload {
    &BINARY_HYDROCARBON.payload
}
