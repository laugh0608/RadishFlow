use rf_store::{StoredPackageError, StoredPackageFailureStage};
use rf_types::RfError;

pub type PackageResult<T> = Result<T, PackageLoadError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFailureKind {
    Missing,
    Expired,
    UnsupportedSource,
    MetadataMismatch,
    IntegrityMismatch,
    ResourceUnavailable,
    InvalidData,
}

impl PackageFailureKind {
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::Missing => "property_package.missing",
            Self::Expired => "property_package.expired",
            Self::UnsupportedSource => "property_package.unsupported_source",
            Self::MetadataMismatch => "property_package.metadata_mismatch",
            Self::IntegrityMismatch => "property_package.integrity_mismatch",
            Self::ResourceUnavailable => "property_package.resource_unavailable",
            Self::InvalidData => "property_package.invalid_data",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageLoadError {
    pub kind: PackageFailureKind,
    pub package_id: Option<String>,
    pub source: RfError,
}

impl PackageLoadError {
    pub(crate) fn new(kind: PackageFailureKind, package_id: Option<&str>, source: RfError) -> Self {
        Self {
            kind,
            package_id: package_id.map(str::to_owned),
            source,
        }
    }

    pub(crate) fn stored(package_id: &str, error: StoredPackageError) -> Self {
        let kind = match error.stage {
            StoredPackageFailureStage::Resource => PackageFailureKind::ResourceUnavailable,
            StoredPackageFailureStage::InvalidData => PackageFailureKind::InvalidData,
            StoredPackageFailureStage::Metadata => PackageFailureKind::MetadataMismatch,
            StoredPackageFailureStage::Integrity => PackageFailureKind::IntegrityMismatch,
        };
        Self::new(kind, Some(package_id), error.error)
    }

    pub fn into_rf_error(self) -> RfError {
        self.source
            .with_diagnostic_code(self.kind.diagnostic_code())
    }
}

impl std::fmt::Display for PackageLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for PackageLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
