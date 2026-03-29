use std::collections::BTreeSet;
use std::time::SystemTime;

use rf_types::{ComponentId, RfError, RfResult};
use serde::{Deserialize, Serialize};

use crate::auth_cache::StoredPropertyPackageSource;

pub const STORED_PROPERTY_PACKAGE_MANIFEST_KIND: &str = "radishflow.property-package-manifest";
pub const STORED_PROPERTY_PACKAGE_PAYLOAD_KIND: &str = "radishflow.property-package-payload";
pub const STORED_PROPERTY_PACKAGE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoredPropertyPackageClassification {
    Derived,
    RemoteOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPropertyPackageManifest {
    pub kind: String,
    pub schema_version: u32,
    pub package_id: String,
    pub version: String,
    pub classification: StoredPropertyPackageClassification,
    pub source: StoredPropertyPackageSource,
    pub hash: String,
    pub size_bytes: u64,
    pub component_ids: Vec<ComponentId>,
    pub lease_required: bool,
    #[serde(with = "crate::json::option_time_format")]
    pub expires_at: Option<SystemTime>,
}

impl StoredPropertyPackageManifest {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        source: StoredPropertyPackageSource,
        component_ids: Vec<ComponentId>,
    ) -> Self {
        let (classification, lease_required) = manifest_defaults_for_source(source);

        Self {
            kind: STORED_PROPERTY_PACKAGE_MANIFEST_KIND.to_string(),
            schema_version: STORED_PROPERTY_PACKAGE_SCHEMA_VERSION,
            package_id: package_id.into(),
            version: version.into(),
            classification,
            source,
            hash: String::new(),
            size_bytes: 0,
            component_ids,
            lease_required,
            expires_at: None,
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_PROPERTY_PACKAGE_MANIFEST_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored property package manifest kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != STORED_PROPERTY_PACKAGE_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored property package manifest schema version `{}`",
                self.schema_version
            )));
        }

        if self.package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package manifest must contain a non-empty package_id",
            ));
        }

        if self.version.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package manifest must contain a non-empty version",
            ));
        }

        let (expected_classification, expected_lease_required) =
            manifest_defaults_for_source(self.source);
        if self.classification != expected_classification {
            return Err(RfError::invalid_input(format!(
                "stored property package manifest classification `{:?}` does not match source `{:?}`",
                self.classification, self.source
            )));
        }

        if self.lease_required != expected_lease_required {
            return Err(RfError::invalid_input(format!(
                "stored property package manifest lease_required `{}` does not match source `{:?}`",
                self.lease_required, self.source
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoredLiquidPhaseModel {
    IdealSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoredVaporPhaseModel {
    IdealGas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredThermoMethod {
    pub liquid_phase_model: StoredLiquidPhaseModel,
    pub vapor_phase_model: StoredVaporPhaseModel,
}

impl Default for StoredThermoMethod {
    fn default() -> Self {
        Self {
            liquid_phase_model: StoredLiquidPhaseModel::IdealSolution,
            vapor_phase_model: StoredVaporPhaseModel::IdealGas,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAntoineCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl StoredAntoineCoefficients {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }

    fn validate(&self) -> RfResult<()> {
        if !self.a.is_finite() || !self.b.is_finite() || !self.c.is_finite() {
            return Err(RfError::invalid_input(
                "stored Antoine coefficients must be finite numbers",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredThermoComponent {
    pub id: ComponentId,
    pub name: String,
    pub antoine: Option<StoredAntoineCoefficients>,
    pub liquid_heat_capacity_j_per_mol_k: Option<f64>,
    pub vapor_heat_capacity_j_per_mol_k: Option<f64>,
}

impl StoredThermoComponent {
    pub fn new(id: impl Into<ComponentId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            antoine: None,
            liquid_heat_capacity_j_per_mol_k: None,
            vapor_heat_capacity_j_per_mol_k: None,
        }
    }

    fn validate(&self) -> RfResult<()> {
        if self.name.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored thermo component must contain a non-empty name",
            ));
        }

        if let Some(antoine) = &self.antoine {
            antoine.validate()?;
        }

        validate_optional_positive_finite(
            self.liquid_heat_capacity_j_per_mol_k,
            "stored liquid heat capacity",
        )?;
        validate_optional_positive_finite(
            self.vapor_heat_capacity_j_per_mol_k,
            "stored vapor heat capacity",
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPropertyPackagePayload {
    pub kind: String,
    pub schema_version: u32,
    pub package_id: String,
    pub version: String,
    pub components: Vec<StoredThermoComponent>,
    pub method: StoredThermoMethod,
}

impl StoredPropertyPackagePayload {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        components: Vec<StoredThermoComponent>,
    ) -> Self {
        Self {
            kind: STORED_PROPERTY_PACKAGE_PAYLOAD_KIND.to_string(),
            schema_version: STORED_PROPERTY_PACKAGE_SCHEMA_VERSION,
            package_id: package_id.into(),
            version: version.into(),
            components,
            method: StoredThermoMethod::default(),
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_PROPERTY_PACKAGE_PAYLOAD_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored property package payload kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != STORED_PROPERTY_PACKAGE_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored property package payload schema version `{}`",
                self.schema_version
            )));
        }

        if self.package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package payload must contain a non-empty package_id",
            ));
        }

        if self.version.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package payload must contain a non-empty version",
            ));
        }

        if self.components.is_empty() {
            return Err(RfError::invalid_input(
                "stored property package payload must contain at least one component",
            ));
        }

        let mut seen_component_ids = BTreeSet::new();
        for component in &self.components {
            component.validate()?;
            if !seen_component_ids.insert(component.id.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored property package payload contains duplicate component `{}`",
                    component.id.as_str()
                )));
            }
        }

        Ok(())
    }
}

fn manifest_defaults_for_source(
    source: StoredPropertyPackageSource,
) -> (StoredPropertyPackageClassification, bool) {
    match source {
        StoredPropertyPackageSource::LocalBundled => {
            (StoredPropertyPackageClassification::Derived, false)
        }
        StoredPropertyPackageSource::RemoteDerivedPackage => {
            (StoredPropertyPackageClassification::Derived, true)
        }
        StoredPropertyPackageSource::RemoteEvaluationService => {
            (StoredPropertyPackageClassification::RemoteOnly, false)
        }
    }
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
