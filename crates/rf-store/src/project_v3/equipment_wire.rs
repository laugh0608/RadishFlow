use super::{StoredEngineeringInputs, rating_wire, task_wire};
use rf_model::equipment::*;
use rf_types::{RfError, RfResult, units::QuantityKind};
use serde::{Deserialize, Serialize};

// Internally tagged decoding retains map entries for the strict variant decoder;
// it never passes through serde_json::Value or retries another task shape.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub(super) enum EquipmentWire {
    #[serde(rename = "verticalGasLiquidSeparatorSizing")]
    Sizing(task_wire::TaskWire),
    #[serde(rename = "verticalGasLiquidSeparatorRating")]
    Rating(rating_wire::RatingWire),
}
impl From<EquipmentWire> for EquipmentTask {
    fn from(value: EquipmentWire) -> Self {
        match value {
            EquipmentWire::Sizing(t) => Self::SeparatorSizing(t.into()),
            EquipmentWire::Rating(t) => Self::SeparatorRating(t.into()),
        }
    }
}
impl From<&EquipmentTask> for EquipmentWire {
    fn from(value: &EquipmentTask) -> Self {
        match value {
            EquipmentTask::SeparatorSizing(t) => Self::Sizing(t.into()),
            EquipmentTask::SeparatorRating(t) => Self::Rating(t.into()),
        }
    }
}

pub(super) fn validate(inputs: &StoredEngineeringInputs) -> RfResult<()> {
    inputs
        .basis
        .validate()
        .map_err(|e| RfError::invalid_input(format!("engineering.{e}")))?;
    validate_equipment_task_identities(&inputs.tasks).map_err(|e| {
        RfError::invalid_input(format!("engineering.tasks[{}]: {:?}", e.task_index, e.kind))
    })?;
    for (index, task) in inputs.tasks.iter().enumerate() {
        let error = |path: &str, message: String| {
            RfError::invalid_input(format!("engineering.tasks[{index}].{path}: {message}"))
        };
        let binding = |path: &str, quantity, value| {
            inputs
                .basis
                .resolve(quantity, value)
                .map(|_| ())
                .map_err(|e| error(path, e.to_string()))
        };
        let sources = |path: &str, ids: &[rf_types::SourceId]| {
            for id in ids {
                if !inputs.basis.sources.iter().any(|s| s.id == *id) {
                    return Err(error(path, format!("missing source {id}")));
                }
            }
            Ok(())
        };
        let (method_id, version) = match task {
            EquipmentTask::SeparatorSizing(_) => (task_wire::METHOD_ID, task_wire::METHOD_VERSION),
            EquipmentTask::SeparatorRating(_) => (
                rating_wire::METHOD_ID,
                if task.method().version == rating_wire::ADOPTION_METHOD_VERSION {
                    rating_wire::ADOPTION_METHOD_VERSION
                } else if task.method().version == rating_wire::MATRIX_METHOD_VERSION {
                    rating_wire::MATRIX_METHOD_VERSION
                } else {
                    rating_wire::METHOD_VERSION
                },
            ),
        };
        if task.method().id != method_id || task.method().version != version {
            return Err(error(
                "method",
                "unsupported method cannot be written".into(),
            ));
        }
        match task {
            EquipmentTask::SeparatorSizing(t) => {
                for (field, value) in &t.case.inputs {
                    binding(&format!("inputs.{field:?}"), field.quantity(), value)?;
                }
                for (field, value) in &t.axial_sections {
                    binding(
                        &format!("axialSections.{field:?}"),
                        QuantityKind::Length,
                        value,
                    )?;
                }
                sources(
                    "case.applicabilitySourceIds",
                    &t.case.applicability_source_ids,
                )?;
            }
            EquipmentTask::SeparatorRating(t) => {
                if let Some(adoption) = &t.geometry.adoption {
                    if t.method.version != rating_wire::ADOPTION_METHOD_VERSION {
                        return Err(error(
                            "geometry.adoption",
                            "requires rating method version 3".into(),
                        ));
                    }
                    if adoption.source_task.method.id != task_wire::METHOD_ID
                        || adoption.source_task.method.version != task_wire::METHOD_VERSION
                    {
                        return Err(error(
                            "geometry.adoption",
                            "unsupported source method".into(),
                        ));
                    }
                    adoption
                        .validate()
                        .map_err(|e| error("geometry.adoption", e.to_string()))?;
                }
                if t.method.version == rating_wire::METHOD_VERSION && t.cases.len() != 1 {
                    return Err(error(
                        "cases",
                        "method version 1 requires exactly one case".into(),
                    ));
                }
                for (field, value) in &t.geometry.inputs {
                    binding(
                        &format!("geometry.inputs.{field:?}"),
                        field.quantity(),
                        value,
                    )?;
                }
                for (field, value) in &t.geometry.axial_sections {
                    binding(
                        &format!("geometry.axialSections.{field:?}"),
                        QuantityKind::Length,
                        value,
                    )?;
                }
                sources(
                    "geometry.applicabilitySourceIds",
                    &t.geometry.applicability_source_ids,
                )?;
                for (case, c) in t.cases.iter().enumerate() {
                    for (field, value) in &c.inputs {
                        binding(
                            &format!("cases[{case}].inputs.{field:?}"),
                            field.quantity(),
                            value,
                        )?;
                    }
                    sources(
                        &format!("cases[{case}].applicabilitySourceIds"),
                        &c.applicability_source_ids,
                    )?;
                }
            }
        }
    }
    Ok(())
}
