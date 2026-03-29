use std::collections::BTreeSet;
use std::path::Path;
use std::time::SystemTime;

use rf_store::{
    StoredAntoineCoefficients, StoredLiquidPhaseModel, StoredPropertyPackagePayload,
    StoredThermoComponent, StoredThermoMethod, StoredVaporPhaseModel,
};
use rf_types::{ComponentId, RfError, RfResult};
use serde::Deserialize;

use crate::persist_downloaded_package_to_cache;
use rf_store::StoredAuthCacheIndex;
use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest};

pub const PROPERTY_PACKAGE_DOWNLOAD_KIND: &str = "radishflow.property-package-download";
pub const PROPERTY_PACKAGE_DOWNLOAD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyPackageDownload {
    pub kind: String,
    pub schema_version: u32,
    pub package_id: String,
    pub version: String,
    pub components: Vec<PropertyPackageDownloadComponent>,
    pub method: PropertyPackageDownloadMethod,
}

impl PropertyPackageDownload {
    pub fn validate(&self) -> RfResult<()> {
        if self.kind != PROPERTY_PACKAGE_DOWNLOAD_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported property package download kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != PROPERTY_PACKAGE_DOWNLOAD_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported property package download schema version `{}`",
                self.schema_version
            )));
        }

        if self.package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "property package download must contain a non-empty package_id",
            ));
        }

        if self.version.trim().is_empty() {
            return Err(RfError::invalid_input(
                "property package download must contain a non-empty version",
            ));
        }

        if self.components.is_empty() {
            return Err(RfError::invalid_input(
                "property package download must contain at least one component",
            ));
        }

        let mut seen_component_ids = BTreeSet::new();
        for component in &self.components {
            component.validate()?;
            if !seen_component_ids.insert(component.id.clone()) {
                return Err(RfError::invalid_input(format!(
                    "property package download contains duplicate component `{}`",
                    component.id.as_str()
                )));
            }
        }

        Ok(())
    }

    pub fn to_stored_payload(&self) -> RfResult<StoredPropertyPackagePayload> {
        self.validate()?;

        let payload = StoredPropertyPackagePayload {
            kind: rf_store::STORED_PROPERTY_PACKAGE_PAYLOAD_KIND.to_string(),
            schema_version: rf_store::STORED_PROPERTY_PACKAGE_SCHEMA_VERSION,
            package_id: self.package_id.clone(),
            version: self.version.clone(),
            components: self
                .components
                .iter()
                .cloned()
                .map(StoredThermoComponent::from)
                .collect(),
            method: self.method.into(),
        };
        payload.validate()?;
        Ok(payload)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyPackageDownloadComponent {
    pub id: ComponentId,
    pub name: String,
    pub antoine: Option<PropertyPackageDownloadAntoineCoefficients>,
    pub liquid_heat_capacity_j_per_mol_k: Option<f64>,
    pub vapor_heat_capacity_j_per_mol_k: Option<f64>,
}

impl PropertyPackageDownloadComponent {
    fn validate(&self) -> RfResult<()> {
        if self.name.trim().is_empty() {
            return Err(RfError::invalid_input(
                "property package download component must contain a non-empty name",
            ));
        }

        if let Some(antoine) = &self.antoine {
            antoine.validate()?;
        }

        validate_optional_positive_finite(
            self.liquid_heat_capacity_j_per_mol_k,
            "download liquid heat capacity",
        )?;
        validate_optional_positive_finite(
            self.vapor_heat_capacity_j_per_mol_k,
            "download vapor heat capacity",
        )?;

        Ok(())
    }
}

impl From<PropertyPackageDownloadComponent> for StoredThermoComponent {
    fn from(value: PropertyPackageDownloadComponent) -> Self {
        Self {
            id: value.id,
            name: value.name,
            antoine: value.antoine.map(StoredAntoineCoefficients::from),
            liquid_heat_capacity_j_per_mol_k: value.liquid_heat_capacity_j_per_mol_k,
            vapor_heat_capacity_j_per_mol_k: value.vapor_heat_capacity_j_per_mol_k,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyPackageDownloadAntoineCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl PropertyPackageDownloadAntoineCoefficients {
    fn validate(&self) -> RfResult<()> {
        if !self.a.is_finite() || !self.b.is_finite() || !self.c.is_finite() {
            return Err(RfError::invalid_input(
                "property package download Antoine coefficients must be finite numbers",
            ));
        }

        Ok(())
    }
}

impl From<PropertyPackageDownloadAntoineCoefficients> for StoredAntoineCoefficients {
    fn from(value: PropertyPackageDownloadAntoineCoefficients) -> Self {
        Self::new(value.a, value.b, value.c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyPackageDownloadLiquidPhaseModel {
    IdealSolution,
}

impl From<PropertyPackageDownloadLiquidPhaseModel> for StoredLiquidPhaseModel {
    fn from(value: PropertyPackageDownloadLiquidPhaseModel) -> Self {
        match value {
            PropertyPackageDownloadLiquidPhaseModel::IdealSolution => Self::IdealSolution,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyPackageDownloadVaporPhaseModel {
    IdealGas,
}

impl From<PropertyPackageDownloadVaporPhaseModel> for StoredVaporPhaseModel {
    fn from(value: PropertyPackageDownloadVaporPhaseModel) -> Self {
        match value {
            PropertyPackageDownloadVaporPhaseModel::IdealGas => Self::IdealGas,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyPackageDownloadMethod {
    pub liquid_phase_model: PropertyPackageDownloadLiquidPhaseModel,
    pub vapor_phase_model: PropertyPackageDownloadVaporPhaseModel,
}

impl From<PropertyPackageDownloadMethod> for StoredThermoMethod {
    fn from(value: PropertyPackageDownloadMethod) -> Self {
        Self {
            liquid_phase_model: value.liquid_phase_model.into(),
            vapor_phase_model: value.vapor_phase_model.into(),
        }
    }
}

pub fn parse_property_package_download_json(contents: &str) -> RfResult<PropertyPackageDownload> {
    let download: PropertyPackageDownload = serde_json::from_str(contents).map_err(|error| {
        RfError::invalid_input(format!("deserialize property package download: {error}"))
    })?;
    download.validate()?;
    Ok(download)
}

pub fn persist_downloaded_package_response_to_cache(
    cache_root: impl AsRef<Path>,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    download_contents: &str,
    downloaded_at: SystemTime,
) -> RfResult<()> {
    let download = parse_property_package_download_json(download_contents)?;
    let payload = download.to_stored_payload()?;
    persist_downloaded_package_to_cache(
        cache_root,
        index,
        manifest,
        lease_grant,
        &payload,
        downloaded_at,
    )
}

fn validate_optional_positive_finite(value: Option<f64>, label: &str) -> RfResult<()> {
    if let Some(value) = value {
        if !value.is_finite() || value <= 0.0 {
            return Err(RfError::invalid_input(format!(
                "{label} must be a finite number greater than zero"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        StoredLiquidPhaseModel, property_package_payload_integrity, read_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest, PropertyPackageSource};

    use crate::{
        PROPERTY_PACKAGE_DOWNLOAD_KIND, parse_property_package_download_json,
        persist_downloaded_package_response_to_cache,
    };

    #[test]
    fn parse_download_json_maps_to_stored_payload_shape() {
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected download parse");
        let payload = download
            .to_stored_payload()
            .expect("expected payload mapping");

        assert_eq!(payload.package_id, "binary-hydrocarbon-lite-v1");
        assert_eq!(payload.version, "2026.03.1");
        assert_eq!(payload.components.len(), 2);
        assert_eq!(payload.components[0].id.as_str(), "methane");
        assert_eq!(payload.components[1].id.as_str(), "ethane");
    }

    #[test]
    fn parse_rejects_wrong_download_kind() {
        let json = sample_download_json().replace(PROPERTY_PACKAGE_DOWNLOAD_KIND, "wrong-kind");
        let error =
            parse_property_package_download_json(&json).expect_err("expected wrong kind error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("unsupported property package download kind")
        );
    }

    #[test]
    fn persist_download_response_to_cache_maps_and_writes_payload() {
        let root = unique_temp_path("download-response-cache");
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        index.entitlement = Some(StoredEntitlementCache {
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            synced_at: timestamp(100),
            issued_at: timestamp(90),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            feature_keys: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: BTreeSet::from(["binary-hydrocarbon-lite-v1".to_string()]),
        });
        let payload = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download")
            .to_stored_payload()
            .expect("expected stored payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let mut manifest = PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.hash = integrity.hash.clone();
        manifest.size_bytes = integrity.size_bytes;
        manifest.component_ids = vec![ComponentId::new("methane"), ComponentId::new("ethane")];
        let lease_grant = PropertyPackageLeaseGrant {
            package_id: "binary-hydrocarbon-lite-v1".to_string(),
            version: "2026.03.1".to_string(),
            lease_id: "lease-1".to_string(),
            download_url: "https://assets.radish.local/lease-1".to_string(),
            hash: integrity.hash,
            size_bytes: integrity.size_bytes,
            expires_at: timestamp(210),
        };

        persist_downloaded_package_response_to_cache(
            &root,
            &mut index,
            &manifest,
            &lease_grant,
            &sample_download_json(),
            timestamp(200),
        )
        .expect("expected download response persistence");

        let payload = read_property_package_payload(
            index.property_packages[0]
                .payload_path_under(&root)
                .expect("expected payload path"),
        )
        .expect("expected payload read");

        assert_eq!(payload.package_id, "binary-hydrocarbon-lite-v1");
        assert_eq!(payload.components.len(), 2);
        assert_eq!(
            payload.method.liquid_phase_model,
            StoredLiquidPhaseModel::IdealSolution
        );

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    fn sample_download_json() -> String {
        fs::read_to_string(sample_download_path()).expect("expected sample download json")
    }

    fn sample_download_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json")
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
}
