//! Version 1 rating payload. Missing input bindings remain incomplete records;
//! unknown fields, duplicate bindings and missing declarations are rejected.
use super::task_wire::{
    AxialKey, BindingWire, FlowBasis, GasDirection, Geometry, KBasis, MethodWire, Orientation,
    Service, bindings, declaration, wire_enum,
};
use rf_model::equipment::*;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const TASK_KIND: &str = "verticalGasLiquidSeparatorRating";
pub(super) const METHOD_ID: &str = "radishflow.separator.vertical-sb-working-volume-rating";
pub(super) const METHOD_VERSION: u32 = 1;
pub(super) const MATRIX_METHOD_VERSION: u32 = 2;

wire_enum!(GeometryKey, SeparatorGeometryInput, {
    InternalDiameter => "internalDiameter", WorkingLiquidHeight => "workingLiquidHeight"
});
wire_enum!(RatingKey, SeparatorRatingInput, {
    OperatingTemperature => "operatingTemperature", OperatingPressure => "operatingPressure",
    GasVolumeFlow => "gasActualVolumeFlow", LiquidVolumeFlow => "liquidActualVolumeFlow",
    GasDensity => "gasMassDensity", LiquidDensity => "liquidMassDensity", EffectiveK => "effectiveK",
    GasVelocityUtilization => "gasUtilization", RequiredWorkingTime => "workingDuration"
});

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RatingWire {
    id: EquipmentTaskId,
    revision: u64,
    tag: String,
    name: String,
    method: MethodWire,
    geometry: GeometryWire,
    cases: Vec<CaseWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GeometryWire {
    revision: u64,
    applicability: GeometryApplicabilityWire,
    applicability_source_ids: Vec<SourceId>,
    #[serde(deserialize_with = "bindings")]
    inputs: BTreeMap<GeometryKey, BindingWire>,
    #[serde(
        default,
        deserialize_with = "bindings",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    axial_sections: BTreeMap<AxialKey, BindingWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GeometryApplicabilityWire {
    #[serde(deserialize_with = "declaration")]
    orientation: Option<Orientation>,
    #[serde(deserialize_with = "declaration")]
    gas_direction: Option<GasDirection>,
    #[serde(deserialize_with = "declaration")]
    geometry: Option<Geometry>,
    internals_description: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseWire {
    id: EngineeringCaseId,
    revision: u64,
    name: String,
    purpose: String,
    fluid_system: String,
    applicability: CaseApplicabilityWire,
    applicability_source_ids: Vec<SourceId>,
    #[serde(deserialize_with = "bindings")]
    inputs: BTreeMap<RatingKey, BindingWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseApplicabilityWire {
    #[serde(deserialize_with = "declaration")]
    service: Option<Service>,
    #[serde(deserialize_with = "declaration")]
    k_basis: Option<KBasis>,
    #[serde(deserialize_with = "declaration")]
    flow_basis: Option<FlowBasis>,
}
impl From<RatingWire> for SeparatorRatingTask {
    fn from(t: RatingWire) -> Self {
        let a = t.geometry.applicability;
        Self {
            id: t.id,
            revision: t.revision,
            tag: t.tag,
            name: t.name,
            method: EquipmentMethod {
                id: t.method.id,
                version: t.method.version,
            },
            geometry: SeparatorGivenGeometry {
                revision: t.geometry.revision,
                applicability: SeparatorGeometryApplicability {
                    orientation: a.orientation.map(Into::into),
                    gas_direction: a.gas_direction.map(Into::into),
                    geometry: a.geometry.map(Into::into),
                    internals_description: a.internals_description,
                },
                applicability_source_ids: t.geometry.applicability_source_ids,
                inputs: t
                    .geometry
                    .inputs
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
                axial_sections: t
                    .geometry
                    .axial_sections
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            cases: t
                .cases
                .into_iter()
                .map(|c| SeparatorRatingCase {
                    id: c.id,
                    revision: c.revision,
                    name: c.name,
                    purpose: c.purpose,
                    fluid_system: c.fluid_system,
                    applicability: SeparatorRatingApplicability {
                        service: c.applicability.service.map(Into::into),
                        k_basis: c.applicability.k_basis.map(Into::into),
                        flow_basis: c.applicability.flow_basis.map(Into::into),
                    },
                    applicability_source_ids: c.applicability_source_ids,
                    inputs: c
                        .inputs
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                })
                .collect(),
        }
    }
}
impl From<&SeparatorRatingTask> for RatingWire {
    fn from(t: &SeparatorRatingTask) -> Self {
        let a = &t.geometry.applicability;
        Self {
            id: t.id.clone(),
            revision: t.revision,
            tag: t.tag.clone(),
            name: t.name.clone(),
            method: MethodWire {
                id: t.method.id.clone(),
                version: t.method.version,
            },
            geometry: GeometryWire {
                revision: t.geometry.revision,
                applicability: GeometryApplicabilityWire {
                    orientation: a.orientation.map(Into::into),
                    gas_direction: a.gas_direction.map(Into::into),
                    geometry: a.geometry.map(Into::into),
                    internals_description: a.internals_description.clone(),
                },
                applicability_source_ids: t.geometry.applicability_source_ids.clone(),
                inputs: t
                    .geometry
                    .inputs
                    .iter()
                    .map(|(k, v)| ((*k).into(), v.into()))
                    .collect(),
                axial_sections: t
                    .geometry
                    .axial_sections
                    .iter()
                    .map(|(k, v)| ((*k).into(), v.into()))
                    .collect(),
            },
            cases: t
                .cases
                .iter()
                .map(|c| CaseWire {
                    id: c.id.clone(),
                    revision: c.revision,
                    name: c.name.clone(),
                    purpose: c.purpose.clone(),
                    fluid_system: c.fluid_system.clone(),
                    applicability: CaseApplicabilityWire {
                        service: c.applicability.service.map(Into::into),
                        k_basis: c.applicability.k_basis.map(Into::into),
                        flow_basis: c.applicability.flow_basis.map(Into::into),
                    },
                    applicability_source_ids: c.applicability_source_ids.clone(),
                    inputs: c
                        .inputs
                        .iter()
                        .map(|(k, v)| ((*k).into(), v.into()))
                        .collect(),
                })
                .collect(),
        }
    }
}
