//! Versioned project configuration DTOs. They contain no solver workspace or UI identity.
use rf_types::{ComponentId, RfError, RfResult, StreamId, UnitId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;

pub const ACYCLIC_METHOD: &str = "radishflow.acyclic-sm";
pub const PARTITIONED_METHOD: &str = "radishflow.partitioned-sm";
#[derive(Debug, Clone, Default, PartialEq)]
pub enum StoredFlowPlan {
    #[default]
    Acyclic,
    Partitioned(StoredRecyclePlan),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredFixedBoundary {
    pub temperature_k: f64,
    pub pressure_pa: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredPortBinding {
    pub unit_id: UnitId,
    pub port: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredComponentValue {
    pub component_id: ComponentId,
    pub value: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredTearInitialization {
    ZeroComponentFlows,
    ComponentFlows {
        #[serde(rename = "valuesMolS")]
        values_mol_s: Vec<StoredComponentValue>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredResidualScaling {
    EvaluatedFreshTotal,
    ComponentValues {
        #[serde(rename = "valuesMolS")]
        values_mol_s: Vec<StoredComponentValue>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredTear {
    pub stream_id: StreamId,
    pub source: StoredPortBinding,
    pub sink: StoredPortBinding,
    pub initialization: StoredTearInitialization,
    pub residual_scaling: StoredResidualScaling,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StoredRecyclePlan {
    pub fixed_boundary: StoredFixedBoundary,
    pub max_iteration_evaluations: usize,
    pub tears: Vec<StoredTear>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredMethod {
    pub id: String,
    pub version: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanWire {
    method: StoredMethod,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    acceptance_contract_version: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    fixed_boundary: Option<StoredFixedBoundary>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    max_iteration_evaluations: Option<usize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    tears: Option<Vec<StoredTear>>,
}
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
impl StoredFlowPlan {
    pub fn method(&self) -> StoredMethod {
        StoredMethod {
            id: match self {
                Self::Acyclic => ACYCLIC_METHOD,
                Self::Partitioned(_) => PARTITIONED_METHOD,
            }
            .into(),
            version: 1,
        }
    }
    pub fn validate(&self) -> RfResult<()> {
        let Self::Partitioned(plan) = self else {
            return Ok(());
        };
        if !plan.fixed_boundary.temperature_k.is_finite()
            || plan.fixed_boundary.temperature_k <= 0.0
            || !plan.fixed_boundary.pressure_pa.is_finite()
            || plan.fixed_boundary.pressure_pa <= 0.0
            || !(1..=500).contains(&plan.max_iteration_evaluations)
        {
            return Err(RfError::invalid_input(
                "simulation: fixed T/P must be positive finite and iteration budget in 1..=500",
            ));
        }
        let mut streams = BTreeSet::new();
        for tear in &plan.tears {
            if tear.stream_id.as_str().trim().is_empty() || !streams.insert(&tear.stream_id) {
                return Err(RfError::invalid_input(
                    "simulation.tears: blank or duplicate stream identity",
                ));
            }
            for port in [&tear.source, &tear.sink] {
                if port.unit_id.as_str().trim().is_empty() || port.port.trim().is_empty() {
                    return Err(RfError::invalid_input(
                        "simulation.tears: blank unit or port binding",
                    ));
                }
            }
            if let StoredTearInitialization::ComponentFlows { values_mol_s } = &tear.initialization
            {
                validate_values(values_mol_s, false)?;
            }
            if let StoredResidualScaling::ComponentValues { values_mol_s } = &tear.residual_scaling
            {
                validate_values(values_mol_s, true)?;
            }
        }
        Ok(())
    }
}
fn validate_values(values: &[StoredComponentValue], positive: bool) -> RfResult<()> {
    let mut seen = BTreeSet::new();
    for item in values {
        if item.component_id.as_str().trim().is_empty()
            || !seen.insert(&item.component_id)
            || !item.value.is_finite()
            || item.value < 0.0
            || (positive && item.value == 0.0)
        {
            return Err(RfError::invalid_input("simulation: component values must be unique, finite and in the declared nonnegative/positive domain").with_related_component_id(item.component_id.clone()));
        }
    }
    Ok(())
}
impl Serialize for StoredFlowPlan {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.validate().map_err(serde::ser::Error::custom)?;
        let mut wire = PlanWire {
            method: self.method(),
            acceptance_contract_version: None,
            fixed_boundary: None,
            max_iteration_evaluations: None,
            tears: None,
        };
        if let Self::Partitioned(plan) = self {
            wire.acceptance_contract_version = Some(1);
            wire.fixed_boundary = Some(plan.fixed_boundary.clone());
            wire.max_iteration_evaluations = Some(plan.max_iteration_evaluations);
            wire.tears = Some(plan.tears.clone());
        }
        wire.serialize(s)
    }
}
impl<'de> Deserialize<'de> for StoredFlowPlan {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = PlanWire::deserialize(d)?;
        let missing = || {
            serde::de::Error::custom(
                "simulation: missing partitioned plan field or unsupported contract",
            )
        };
        let result = match (w.method.id.as_str(), w.method.version) {
            (ACYCLIC_METHOD, 1)
                if w.acceptance_contract_version.is_none()
                    && w.fixed_boundary.is_none()
                    && w.max_iteration_evaluations.is_none()
                    && w.tears.is_none() =>
            {
                Self::Acyclic
            }
            (PARTITIONED_METHOD, 1) if w.acceptance_contract_version == Some(1) => {
                Self::Partitioned(StoredRecyclePlan {
                    fixed_boundary: w.fixed_boundary.ok_or_else(missing)?,
                    max_iteration_evaluations: w.max_iteration_evaluations.ok_or_else(missing)?,
                    tears: w.tears.ok_or_else(missing)?,
                })
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "simulation: unsupported method, version or fields",
                ));
            }
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}
