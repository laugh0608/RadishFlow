//! Dedicated SI input records. Identity, method versions and fixed evidence are
//! read-only metadata, never accepted inside an ordinary input update.
use super::{enums::*, protocol::IdentityRef, strict::optional};
use rf_model::{engineering as domain, equipment};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringValue {
    pub quantity: Quantity,
    pub value_si: f64,
}
impl From<domain::EngineeringValue> for EngineeringValue {
    fn from(v: domain::EngineeringValue) -> Self {
        Self {
            quantity: v.quantity.into(),
            value_si: v.value_si,
        }
    }
}
impl From<EngineeringValue> for domain::EngineeringValue {
    fn from(v: EngineeringValue) -> Self {
        Self {
            quantity: v.quantity.into(),
            value_si: v.value_si,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Binding {
    Direct {
        value: EngineeringValue,
        source: IdentityRef,
    },
    Basis {
        basis: IdentityRef,
    },
    Override {
        basis: IdentityRef,
        value: EngineeringValue,
        source: IdentityRef,
        reason: String,
    },
}
impl From<&domain::InputBinding> for Binding {
    fn from(v: &domain::InputBinding) -> Self {
        match v {
            domain::InputBinding::Direct { value, source_id } => Self::Direct {
                value: (*value).into(),
                source: IdentityRef::id(source_id),
            },
            domain::InputBinding::BasisRef { basis_entry_id } => Self::Basis {
                basis: IdentityRef::id(basis_entry_id),
            },
            domain::InputBinding::Override {
                basis_entry_id,
                value,
                source_id,
                reason,
            } => Self::Override {
                basis: IdentityRef::id(basis_entry_id),
                value: (*value).into(),
                source: IdentityRef::id(source_id),
                reason: reason.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatingConditions {
    pub temperature_k: f64,
    pub pressure_pa: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceApplicability {
    pub fluid_system: String,
    pub phase: SourcePhase,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub operating_conditions: Option<OperatingConditions>,
    pub scope: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationRef {
    pub source: IdentityRef,
    pub observation_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: String,
    pub quantity: Quantity,
    pub original_value: f64,
    pub original_unit: String,
    pub canonical_value: f64,
    pub conversion_note: String,
    pub depends_on: Vec<ObservationRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceData {
    pub kind: SourceKind,
    pub title: String,
    pub reference: String,
    pub locator: String,
    pub publication_version: String,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub accessed_on: Option<String>,
    pub adoption_note: String,
    pub review: SourceReview,
    pub applicability: SourceApplicability,
    pub observations: Vec<Observation>,
}
impl From<&domain::SourceRecord> for SourceData {
    fn from(v: &domain::SourceRecord) -> Self {
        Self {
            kind: v.kind.into(),
            title: v.title.clone(),
            reference: v.reference.clone(),
            locator: v.locator.clone(),
            publication_version: v.publication_version.clone(),
            accessed_on: v.accessed_on.map(|v| v.to_string()),
            adoption_note: v.adoption_note.clone(),
            review: v.review.into(),
            applicability: SourceApplicability {
                fluid_system: v.applicability.fluid_system.clone(),
                phase: v.applicability.phase.into(),
                operating_conditions: v.applicability.operating_conditions.map(|c| {
                    OperatingConditions {
                        temperature_k: c.temperature_k,
                        pressure_pa: c.pressure_pa,
                    }
                }),
                scope: v.applicability.scope.clone(),
            },
            observations: v
                .observations
                .iter()
                .map(|o| Observation {
                    id: o.id.to_string(),
                    quantity: o.quantity.into(),
                    original_value: o.original_value,
                    original_unit: o.original_unit.clone(),
                    canonical_value: o.canonical_value,
                    conversion_note: o.conversion_note.clone(),
                    depends_on: o
                        .depends_on
                        .iter()
                        .map(|r| ObservationRef {
                            source: IdentityRef::id(&r.source_id),
                            observation_id: r.observation_id.to_string(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisData {
    pub name: String,
    pub value: EngineeringValue,
    pub source: IdentityRef,
}
impl From<&domain::BasisEntry> for BasisData {
    fn from(v: &domain::BasisEntry) -> Self {
        Self {
            name: v.name.clone(),
            value: v.value.into(),
            source: IdentityRef::id(&v.source_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryApplicability {
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub orientation: Option<Orientation>,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub gas_direction: Option<GasDirection>,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub geometry: Option<SectionGeometry>,
    pub internals_description: String,
}
impl From<&equipment::SeparatorGeometryApplicability> for GeometryApplicability {
    fn from(v: &equipment::SeparatorGeometryApplicability) -> Self {
        Self {
            orientation: v.orientation.map(Into::into),
            gas_direction: v.gas_direction.map(Into::into),
            geometry: v.geometry.map(Into::into),
            internals_description: v.internals_description.clone(),
        }
    }
}
impl From<GeometryApplicability> for equipment::SeparatorGeometryApplicability {
    fn from(v: GeometryApplicability) -> Self {
        Self {
            orientation: v.orientation.map(Into::into),
            gas_direction: v.gas_direction.map(Into::into),
            geometry: v.geometry.map(Into::into),
            internals_description: v.internals_description,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseApplicability {
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub service: Option<Service>,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub k_basis: Option<KBasis>,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub flow_basis: Option<FlowBasis>,
}
impl From<&equipment::SeparatorRatingApplicability> for CaseApplicability {
    fn from(v: &equipment::SeparatorRatingApplicability) -> Self {
        Self {
            service: v.service.map(Into::into),
            k_basis: v.k_basis.map(Into::into),
            flow_basis: v.flow_basis.map(Into::into),
        }
    }
}
impl From<CaseApplicability> for equipment::SeparatorRatingApplicability {
    fn from(v: CaseApplicability) -> Self {
        Self {
            service: v.service.map(Into::into),
            k_basis: v.k_basis.map(Into::into),
            flow_basis: v.flow_basis.map(Into::into),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizingCaseData {
    pub name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub geometry_applicability: GeometryApplicability,
    pub case_applicability: CaseApplicability,
    pub applicability_sources: Vec<IdentityRef>,
    pub inputs: BTreeMap<SizingField, Binding>,
}
impl From<&equipment::SeparatorCase> for SizingCaseData {
    fn from(v: &equipment::SeparatorCase) -> Self {
        Self {
            name: v.name.clone(),
            purpose: v.purpose.clone(),
            fluid_system: v.fluid_system.clone(),
            geometry_applicability: GeometryApplicability {
                orientation: v.applicability.orientation.map(Into::into),
                gas_direction: v.applicability.gas_direction.map(Into::into),
                geometry: v.applicability.geometry.map(Into::into),
                internals_description: v.applicability.internals_description.clone(),
            },
            case_applicability: CaseApplicability {
                service: v.applicability.service.map(Into::into),
                k_basis: v.applicability.k_basis.map(Into::into),
                flow_basis: v.applicability.flow_basis.map(Into::into),
            },
            applicability_sources: v
                .applicability_source_ids
                .iter()
                .map(IdentityRef::id)
                .collect(),
            inputs: v
                .inputs
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizingData {
    pub tag: String,
    pub name: String,
    pub case: SizingCaseData,
    pub axial_sections: BTreeMap<AxialSection, Binding>,
}
impl From<&equipment::SeparatorSizingTask> for SizingData {
    fn from(v: &equipment::SeparatorSizingTask) -> Self {
        Self {
            tag: v.tag.clone(),
            name: v.name.clone(),
            case: (&v.case).into(),
            axial_sections: v
                .axial_sections
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Labels {
    pub tag: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryData {
    pub applicability: GeometryApplicability,
    pub applicability_sources: Vec<IdentityRef>,
    pub inputs: BTreeMap<GeometryField, Binding>,
    pub axial_sections: BTreeMap<AxialSection, Binding>,
}
impl From<&equipment::SeparatorGivenGeometry> for GeometryData {
    fn from(v: &equipment::SeparatorGivenGeometry) -> Self {
        Self {
            applicability: (&v.applicability).into(),
            applicability_sources: v
                .applicability_source_ids
                .iter()
                .map(IdentityRef::id)
                .collect(),
            inputs: v
                .inputs
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
            axial_sections: v
                .axial_sections
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RatingCaseData {
    pub name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub applicability: CaseApplicability,
    pub applicability_sources: Vec<IdentityRef>,
    pub inputs: BTreeMap<RatingField, Binding>,
}
impl From<&equipment::SeparatorRatingCase> for RatingCaseData {
    fn from(v: &equipment::SeparatorRatingCase) -> Self {
        Self {
            name: v.name.clone(),
            purpose: v.purpose.clone(),
            fluid_system: v.fluid_system.clone(),
            applicability: (&v.applicability).into(),
            applicability_sources: v
                .applicability_source_ids
                .iter()
                .map(IdentityRef::id)
                .collect(),
            inputs: v
                .inputs
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
        }
    }
}

pub fn source(v: &domain::SourceRecord) -> Value {
    json!({"id": v.id.as_str(), "revision": v.revision, "data": SourceData::from(v)})
}
pub fn basis(v: &domain::BasisEntry) -> Value {
    json!({"id": v.id.as_str(), "revision": v.revision, "data": BasisData::from(v)})
}
pub fn method(v: &equipment::EquipmentMethod) -> Value {
    json!({"id":v.id,"version":v.version})
}
pub fn sizing(v: &equipment::SeparatorSizingTask) -> Value {
    json!({"kind":"sizing", "id":v.id.as_str(), "revision":v.revision, "case_id":v.case.id.as_str(), "method": method(&v.method), "data":SizingData::from(v)})
}
pub fn geometry(v: &equipment::SeparatorGivenGeometry) -> Value {
    json!({"revision":v.revision,"data":GeometryData::from(v),"adoption":v.adoption.as_deref().map(adoption)})
}
pub fn rating_case(v: &equipment::SeparatorRatingCase) -> Value {
    json!({"id":v.id.as_str(),"revision":v.revision,"data":RatingCaseData::from(v)})
}
pub fn task(v: &equipment::EquipmentTask) -> Value {
    match v {
        equipment::EquipmentTask::SeparatorSizing(v) => sizing(v),
        equipment::EquipmentTask::SeparatorRating(v) => {
            json!({
                "kind": "rating",
                "id": v.id.as_str(),
                "revision": v.revision,
                "method": method(&v.method),
                "data": Labels {
                    tag:v.tag.clone(),
                    name:v.name.clone()
                },
                "geometry": geometry(&v.geometry),
                "cases": v.cases.iter().map(rating_case).collect::<Vec<_>>()
            })
        }
    }
}
pub fn adoption(v: &equipment::SeparatorGeometryAdoption) -> Value {
    let dimension = |v: &equipment::AdoptedDimension| json!({"original_m":v.original_m,"adopted_m":v.adopted_m,"reason":v.reason});
    json!({
        "source_document_id": v.source_document_id,
        "source_task": sizing(&v.source_task),
        "source_basis": {
            "sources": v.source_basis.sources.iter().map(source).collect::<Vec<_>>(),
            "basis": v.source_basis.basis_entries.iter().map(basis).collect::<Vec<_>>()
        },
        "diameter": dimension(&v.diameter),
        "working_liquid_height": dimension(&v.working_liquid_height),
        "axial_lengths_m": v.axial_lengths_m.iter().map(|(k,
        v)|(AxialSection::from(*k),
        *v)).collect::<BTreeMap<_,
        _>>()
    })
}
