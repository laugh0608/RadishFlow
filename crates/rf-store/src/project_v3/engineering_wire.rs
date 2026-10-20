use rf_model::engineering::*;
use rf_types::{
    BasisEntryId, RfError, RfResult, SourceId, SourceObservationId,
    units::{ALL_QUANTITIES, QuantityKind},
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt, marker::PhantomData};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EngineeringWire<T = TaskProbe> {
    #[serde(deserialize_with = "basis_entries")]
    basis_entries: Vec<BasisWire>,
    #[serde(deserialize_with = "sources")]
    sources: Vec<SourceWire>,
    #[serde(deserialize_with = "tasks", bound(deserialize = "T: Deserialize<'de>"))]
    pub(super) tasks: Vec<T>,
}

// Only the stable identity envelope is inspected. No device case/input/method
// payload is decoded during the capability probe.
// Unknown fields remain in the read-only original text, never in a lossy Value.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TaskProbe {
    id: String,
    kind: String,
    method: MethodProbe,
}
impl TaskProbe {
    pub(super) fn supported(&self) -> bool {
        self.kind == super::task_wire::TASK_KIND
            && self.method.id == super::task_wire::METHOD_ID
            && self.method.version == u64::from(super::task_wire::METHOD_VERSION)
    }
}
#[derive(Serialize, Deserialize)]
struct MethodProbe {
    id: String,
    version: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BasisWire {
    id: BasisEntryId,
    revision: u64,
    name: String,
    #[serde(with = "quantity_id")]
    quantity_id: QuantityKind,
    value_si: f64,
    source_id: SourceId,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceWire {
    id: SourceId,
    revision: u64,
    kind: SourceKind,
    title: String,
    reference: String,
    locator: String,
    publication_version: String,
    accessed_on: String,
    adoption_note: String,
    review: SourceReview,
    #[serde(deserialize_with = "applicability")]
    applicability: ApplicabilityWire,
    #[serde(deserialize_with = "observations")]
    observations: Vec<ObservationWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplicabilityWire {
    fluid_system: String,
    phase: SourcePhase,
    #[serde(deserialize_with = "operating_conditions")]
    operating_conditions: ConditionsWire,
    scope: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConditionsWire {
    #[serde(
        default,
        deserialize_with = "present_number",
        skip_serializing_if = "Option::is_none"
    )]
    temperature_k: Option<f64>,
    #[serde(
        default,
        deserialize_with = "present_number",
        skip_serializing_if = "Option::is_none"
    )]
    pressure_pa: Option<f64>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ObservationWire {
    id: SourceObservationId,
    #[serde(with = "quantity_id")]
    quantity_id: QuantityKind,
    original_value: f64,
    original_unit: String,
    canonical_value: f64,
    conversion_note: String,
    #[serde(deserialize_with = "dependencies")]
    depends_on: Vec<ObservationRefWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ObservationRefWire {
    source_id: SourceId,
    observation_id: SourceObservationId,
}

impl EngineeringWire {
    pub(super) fn into_basis(self) -> RfResult<EngineeringBasis> {
        let mut ids = BTreeSet::new();
        for (i, task) in self.tasks.iter().enumerate() {
            if task.id.trim().is_empty()
                || task.kind.trim().is_empty()
                || task.method.id.trim().is_empty()
            {
                return Err(RfError::invalid_input(format!(
                    "engineering.tasks[{i}]: id, kind and method.id must be nonempty"
                )));
            }
            if !ids.insert(&task.id) {
                return Err(RfError::invalid_input(format!(
                    "engineering.tasks[{i}].id: duplicate ID"
                )));
            }
        }
        let basis_entries = self
            .basis_entries
            .into_iter()
            .map(|b| BasisEntry {
                id: b.id,
                revision: b.revision,
                name: b.name,
                value: EngineeringValue {
                    quantity: b.quantity_id,
                    value_si: b.value_si,
                },
                source_id: b.source_id,
            })
            .collect();
        let sources = self.sources.into_iter().enumerate().map(|(i, s)| {
            let path = format!("engineering.sources[{i}]");
            let accessed_on = if s.accessed_on.is_empty() { None } else {
                Some(s.accessed_on.parse().map_err(|e| RfError::invalid_input(format!("{path}.{e}")))?)
            };
            let conditions = s.applicability.operating_conditions;
            let operating_conditions = match (conditions.temperature_k, conditions.pressure_pa) {
                (None, None) => None,
                (Some(temperature_k), Some(pressure_pa)) => Some(SourceOperatingConditions { temperature_k, pressure_pa }),
                _ => return Err(RfError::invalid_input(format!("{path}.applicability.operatingConditions: temperatureK and pressurePa must occur together"))),
            };
            Ok(SourceRecord {
                id: s.id, revision: s.revision, kind: s.kind, title: s.title, reference: s.reference,
                locator: s.locator, publication_version: s.publication_version, accessed_on,
                adoption_note: s.adoption_note, review: s.review,
                applicability: SourceApplicability { fluid_system: s.applicability.fluid_system, phase: s.applicability.phase, operating_conditions, scope: s.applicability.scope },
                observations: s.observations.into_iter().map(|o| SourceObservation {
                    id: o.id, quantity: o.quantity_id, original_value: o.original_value,
                    original_unit: o.original_unit, canonical_value: o.canonical_value,
                    conversion_note: o.conversion_note,
                    depends_on: o.depends_on.into_iter().map(|d| SourceObservationRef { source_id: d.source_id, observation_id: d.observation_id }).collect(),
                }).collect(),
            })
        }).collect::<RfResult<Vec<_>>>()?;
        let basis = EngineeringBasis {
            basis_entries,
            sources,
        };
        basis
            .validate()
            .map_err(|e| RfError::invalid_input(format!("engineering.{e}")))?;
        Ok(basis)
    }
}

impl<T> EngineeringWire<T> {
    pub(super) fn with_tasks(basis: &EngineeringBasis, tasks: Vec<T>) -> Self {
        let wire = EngineeringWire::from(basis);
        Self {
            basis_entries: wire.basis_entries,
            sources: wire.sources,
            tasks,
        }
    }
}

impl From<&EngineeringBasis> for EngineeringWire {
    fn from(basis: &EngineeringBasis) -> Self {
        Self {
            tasks: vec![],
            basis_entries: basis
                .basis_entries
                .iter()
                .map(|b| BasisWire {
                    id: b.id.clone(),
                    revision: b.revision,
                    name: b.name.clone(),
                    quantity_id: b.value.quantity,
                    value_si: b.value.value_si,
                    source_id: b.source_id.clone(),
                })
                .collect(),
            sources: basis
                .sources
                .iter()
                .map(|s| SourceWire {
                    id: s.id.clone(),
                    revision: s.revision,
                    kind: s.kind,
                    title: s.title.clone(),
                    reference: s.reference.clone(),
                    locator: s.locator.clone(),
                    publication_version: s.publication_version.clone(),
                    accessed_on: s
                        .accessed_on
                        .map_or_else(String::new, |date| date.to_string()),
                    adoption_note: s.adoption_note.clone(),
                    review: s.review,
                    applicability: ApplicabilityWire {
                        fluid_system: s.applicability.fluid_system.clone(),
                        phase: s.applicability.phase,
                        operating_conditions: ConditionsWire {
                            temperature_k: s
                                .applicability
                                .operating_conditions
                                .map(|c| c.temperature_k),
                            pressure_pa: s
                                .applicability
                                .operating_conditions
                                .map(|c| c.pressure_pa),
                        },
                        scope: s.applicability.scope.clone(),
                    },
                    observations: s
                        .observations
                        .iter()
                        .map(|o| ObservationWire {
                            id: o.id.clone(),
                            quantity_id: o.quantity,
                            original_value: o.original_value,
                            original_unit: o.original_unit.clone(),
                            canonical_value: o.canonical_value,
                            conversion_note: o.conversion_note.clone(),
                            depends_on: o
                                .depends_on
                                .iter()
                                .map(|d| ObservationRefWire {
                                    source_id: d.source_id.clone(),
                                    observation_id: d.observation_id.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

pub(super) mod quantity_id {
    use super::*;
    pub(in crate::project_v3) fn serialize<S: Serializer>(
        quantity: &QuantityKind,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(quantity.definition().id)
    }
    pub(in crate::project_v3) fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
    ) -> Result<QuantityKind, D::Error> {
        let id = String::deserialize(de)?;
        ALL_QUANTITIES
            .iter()
            .copied()
            .find(|q| q.definition().id == id)
            .ok_or_else(|| serde::de::Error::custom(format!("quantityId: unknown quantity {id}")))
    }
}

fn present_number<'de, D: Deserializer<'de>>(de: D) -> Result<Option<f64>, D::Error> {
    f64::deserialize(de).map(Some)
}
fn applicability<'de, D: Deserializer<'de>>(de: D) -> Result<ApplicabilityWire, D::Error> {
    ApplicabilityWire::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("applicability: {e}")))
}
fn operating_conditions<'de, D: Deserializer<'de>>(de: D) -> Result<ConditionsWire, D::Error> {
    ConditionsWire::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("operatingConditions: {e}")))
}

// Contextualize array failures without adding a second JSON parser or losing duplicate keys.
pub(super) fn indexed<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    de: D,
    field: &'static str,
) -> Result<Vec<T>, D::Error> {
    struct Indexed<T> {
        field: &'static str,
        marker: PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Indexed<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} array", self.field)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            loop {
                let value = seq.next_element().map_err(|e| {
                    serde::de::Error::custom(format!("{}[{}]: {e}", self.field, values.len()))
                })?;
                match value {
                    Some(value) => values.push(value),
                    None => return Ok(values),
                }
            }
        }
    }
    de.deserialize_seq(Indexed {
        field,
        marker: PhantomData,
    })
}
fn basis_entries<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<BasisWire>, D::Error> {
    indexed(de, "basisEntries")
}
fn sources<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<SourceWire>, D::Error> {
    indexed(de, "sources")
}
fn tasks<'de, D: Deserializer<'de>, T: Deserialize<'de>>(de: D) -> Result<Vec<T>, D::Error> {
    indexed(de, "tasks")
}
fn observations<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<ObservationWire>, D::Error> {
    indexed(de, "observations")
}
fn dependencies<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<ObservationRefWire>, D::Error> {
    indexed(de, "dependsOn")
}
