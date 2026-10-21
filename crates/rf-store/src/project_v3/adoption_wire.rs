//! Strict embedded evidence. This payload is decoded only after capability probing.
use super::{
    engineering_wire::EngineeringWire,
    task_wire::{AxialKey, TaskWire},
};
use rf_model::equipment::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

pub(super) struct AdoptionWire(pub SeparatorGeometryAdoption);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvidenceWire {
    source_document_id: String,
    source_task: TaskWire,
    source_basis: EngineeringWire,
    diameter: DimensionWire,
    working_liquid_height: DimensionWire,
    axial_sections: Vec<AxialWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DimensionWire {
    original_m: f64,
    adopted_m: f64,
    reason: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AxialWire {
    section: AxialKey,
    value_m: f64,
}

impl From<&AdoptedDimension> for DimensionWire {
    fn from(value: &AdoptedDimension) -> Self {
        Self {
            original_m: value.original_m,
            adopted_m: value.adopted_m,
            reason: value.reason.clone(),
        }
    }
}
impl From<DimensionWire> for AdoptedDimension {
    fn from(value: DimensionWire) -> Self {
        Self {
            original_m: value.original_m,
            adopted_m: value.adopted_m,
            reason: value.reason,
        }
    }
}
impl Serialize for AdoptionWire {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let a = &self.0;
        a.validate().map_err(serde::ser::Error::custom)?;
        EvidenceWire {
            source_document_id: a.source_document_id.clone(),
            source_task: (&a.source_task).into(),
            source_basis: (&a.source_basis).into(),
            diameter: (&a.diameter).into(),
            working_liquid_height: (&a.working_liquid_height).into(),
            axial_sections: a
                .axial_lengths_m
                .iter()
                .map(|(key, value)| AxialWire {
                    section: (*key).into(),
                    value_m: *value,
                })
                .collect(),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for AdoptionWire {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = EvidenceWire::deserialize(deserializer)?;
        if !wire.source_basis.tasks.is_empty() {
            return Err(serde::de::Error::custom(
                "adoption sourceBasis cannot contain tasks",
            ));
        }
        let source_task: SeparatorSizingTask = wire.source_task.into();
        if source_task.method.id != super::task_wire::METHOD_ID
            || source_task.method.version != super::task_wire::METHOD_VERSION
        {
            return Err(serde::de::Error::custom(
                "unsupported adoption source method",
            ));
        }
        let mut axial_lengths_m = BTreeMap::new();
        for axial in wire.axial_sections {
            if axial_lengths_m
                .insert(axial.section.into(), axial.value_m)
                .is_some()
            {
                return Err(serde::de::Error::custom("duplicate adoption axial section"));
            }
        }
        let result = SeparatorGeometryAdoption {
            source_document_id: wire.source_document_id,
            source_task,
            source_basis: wire
                .source_basis
                .into_basis()
                .map_err(serde::de::Error::custom)?,
            diameter: wire.diameter.into(),
            working_liquid_height: wire.working_liquid_height.into(),
            axial_lengths_m,
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(Self(result))
    }
}
