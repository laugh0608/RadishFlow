//! The engineering and presentation sections are shared by known project envelopes.
//! This module never changes a file's outer version or invokes a legacy flow guard.
use super::*;

pub(crate) use super::presentation_wire::PresentationWire;
#[derive(Serialize)]
pub(crate) struct EngineeringSection(EngineeringWire<equipment_wire::EquipmentWire>);
pub(crate) enum EngineeringRead {
    Supported(Option<StoredEngineeringInputs>),
    Unsupported(usize),
}

pub(crate) fn decode(contents: &str) -> RfResult<EngineeringRead> {
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default, deserialize_with = "engineering")]
        engineering: Option<EngineeringWire>,
    }
    let envelope: Envelope = serde_json::from_str(contents).map_err(json_error)?;
    let count = envelope.engineering.as_ref().map_or(0, |e| e.tasks.len());
    let unsupported = envelope
        .engineering
        .as_ref()
        .is_some_and(|e| e.tasks.iter().any(|t| !t.supported()));
    if unsupported || task_wire::legacy_payload(contents)? {
        return Ok(EngineeringRead::Unsupported(count));
    }
    let tasks = task_wire::decode(contents)?;
    let inputs = envelope
        .engineering
        .map(|e| e.into_basis())
        .transpose()?
        .map(|basis| StoredEngineeringInputs { basis, tasks });
    if let Some(inputs) = &inputs {
        validate_engineering(inputs)?;
    }
    Ok(EngineeringRead::Supported(inputs))
}
pub(crate) fn encode(
    inputs: Option<&StoredEngineeringInputs>,
) -> RfResult<Option<EngineeringSection>> {
    inputs
        .map(|inputs| {
            validate_engineering(inputs)?;
            Ok(EngineeringSection(EngineeringWire::with_tasks(
                &inputs.basis,
                inputs
                    .tasks
                    .iter()
                    .map(equipment_wire::EquipmentWire::from)
                    .collect(),
            )))
        })
        .transpose()
}
