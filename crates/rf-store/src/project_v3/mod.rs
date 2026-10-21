//! Complete v3 equipment storage; legacy readers and CLI retain their v1/v2 contract.
mod adoption_wire;
mod engineering_wire;
mod equipment_wire;
mod file_io;
mod presentation_wire;
mod rating_wire;
mod task_wire;
pub use file_io::{read, write, write_upgraded};

use crate::{StoredProjectDocument, project::STORED_PROJECT_FILE_KIND};
use engineering_wire::EngineeringWire;
use presentation_wire::PresentationWire;
use rf_model::engineering::EngineeringBasis;
use rf_types::{RfError, RfResult, units::DisplayUnitSet};
use serde::{Deserialize, Deserializer, Serialize};

pub const VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoredEngineeringInputs {
    pub basis: EngineeringBasis,
    pub tasks: Vec<rf_model::equipment::EquipmentTask>,
}
impl StoredEngineeringInputs {
    pub fn has_rating(&self) -> bool {
        self.rating_method_version() != 0
    }
    pub fn rating_method_version(&self) -> u32 {
        self.tasks
            .iter()
            .filter_map(|t| t.as_rating().map(|r| r.method.version))
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredProjectV3 {
    pub document: StoredProjectDocument,
    pub display_units: DisplayUnitSet,
    /// None preserves an omitted engineering block. Some(empty) preserves an
    /// explicitly present block with three empty arrays, as required by v3.
    pub engineering: Option<StoredEngineeringInputs>,
}
impl StoredProjectV3 {
    pub fn rating_method_version(&self) -> u32 {
        self.engineering
            .as_ref()
            .map_or(0, StoredEngineeringInputs::rating_method_version)
    }
    pub fn has_rating(&self) -> bool {
        self.engineering
            .as_ref()
            .is_some_and(StoredEngineeringInputs::has_rating)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectV3Read {
    Editable(StoredProjectV3),
    ReadOnly(ReadOnlyProjectV3),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadOnlyProjectV3 {
    original_json: String,
    document: StoredProjectDocument,
    task_count: usize,
    display_units: DisplayUnitSet,
}
impl ReadOnlyProjectV3 {
    /// Original UTF-8 text, including unsupported fields and numeric lexemes.
    pub fn original_json(&self) -> &str {
        &self.original_json
    }
    pub fn document(&self) -> &StoredProjectDocument {
        &self.document
    }
    pub fn display_units(&self) -> &DisplayUnitSet {
        &self.display_units
    }
    pub fn task_count(&self) -> usize {
        self.task_count
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Header {
    kind: String,
    schema_version: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    bound(deserialize = "T: Deserialize<'de>")
)]
struct ProjectWire<T = engineering_wire::TaskProbe> {
    kind: String,
    schema_version: u32,
    #[serde(deserialize_with = "document")]
    document: StoredProjectDocument,
    #[serde(deserialize_with = "presentation")]
    presentation: PresentationWire,
    #[serde(
        default,
        deserialize_with = "engineering",
        bound(deserialize = "T: Deserialize<'de>"),
        skip_serializing_if = "Option::is_none"
    )]
    engineering: Option<EngineeringWire<T>>,
}

fn document<'de, D: Deserializer<'de>>(de: D) -> Result<StoredProjectDocument, D::Error> {
    StoredProjectDocument::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("document: {e}")))
}
fn presentation<'de, D: Deserializer<'de>>(de: D) -> Result<PresentationWire, D::Error> {
    PresentationWire::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("presentation: {e}")))
}
fn engineering<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    de: D,
) -> Result<Option<EngineeringWire<T>>, D::Error> {
    // A missing field is None; an explicit null cannot deserialize as EngineeringWire.
    EngineeringWire::deserialize(de)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(format!("engineering: {e}")))
}

/// Decode in stages from original text: the version probe never converts into Value,
/// so strict body decoding still sees duplicate keys and unknown-task numbers.
pub fn parse(contents: &str) -> RfResult<ProjectV3Read> {
    let header: Header = serde_json::from_str(contents).map_err(json_error)?;
    if header.kind != STORED_PROJECT_FILE_KIND {
        return Err(RfError::invalid_input(
            "kind: unsupported project file kind",
        ));
    }
    if header.schema_version != VERSION {
        return Err(RfError::invalid_input(format!(
            "schemaVersion: v3 codec requires version 3, got {}",
            header.schema_version
        )));
    }
    let wire: ProjectWire = serde_json::from_str(contents).map_err(json_error)?;
    wire.document.validate()?;
    let display_units = wire.presentation.into_display_units()?;
    let task_count = wire.engineering.as_ref().map_or(0, |e| e.tasks.len());
    let unsupported = wire
        .engineering
        .as_ref()
        .is_some_and(|e| e.tasks.iter().any(|t| !t.supported()));
    let engineering = wire
        .engineering
        .map(EngineeringWire::into_basis)
        .transpose()?;
    if unsupported || task_wire::legacy_payload(contents)? {
        return Ok(ProjectV3Read::ReadOnly(ReadOnlyProjectV3 {
            original_json: contents.to_string(),
            document: wire.document,
            task_count,
            display_units,
        }));
    }
    let tasks = task_wire::decode(contents)?;
    let engineering = engineering.map(|basis| StoredEngineeringInputs { basis, tasks });
    if let Some(inputs) = &engineering {
        validate_engineering(inputs)?;
    }
    Ok(ProjectV3Read::Editable(StoredProjectV3 {
        document: wire.document,
        display_units,
        engineering,
    }))
}

/// Reject read-only documents rather than reserializing a partially understood file.
/// Studio uses this API explicitly; legacy readers/writers remain separate.
pub fn to_pretty_json(project: &ProjectV3Read) -> RfResult<String> {
    let ProjectV3Read::Editable(project) = project else {
        return Err(RfError::invalid_input(
            "engineering.tasks: unsupported equipment capability; entire project is read-only",
        ));
    };
    project.document.validate()?;
    if let Some(inputs) = &project.engineering {
        validate_engineering(inputs)?;
    }
    let wire = ProjectWire {
        kind: STORED_PROJECT_FILE_KIND.into(),
        schema_version: VERSION,
        document: project.document.clone(),
        presentation: PresentationWire::from(&project.display_units),
        engineering: project.engineering.as_ref().map(|e| {
            EngineeringWire::with_tasks(
                &e.basis,
                e.tasks
                    .iter()
                    .map(equipment_wire::EquipmentWire::from)
                    .collect::<Vec<_>>(),
            )
        }),
    };
    serde_json::to_string_pretty(&wire).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> RfError {
    RfError::invalid_input(format!("project v3 JSON: {error}"))
}

#[cfg(test)]
mod tests;

fn validate_engineering(inputs: &StoredEngineeringInputs) -> RfResult<()> {
    equipment_wire::validate(inputs)
}

/// Version dispatch without a lossy JSON Value conversion. Body readers remain strict.
pub fn probe_version(contents: &str) -> RfResult<u32> {
    let header: Header = serde_json::from_str(contents).map_err(json_error)?;
    if header.kind != STORED_PROJECT_FILE_KIND {
        return Err(RfError::invalid_input(
            "kind: unsupported project file kind",
        ));
    }
    Ok(header.schema_version)
}
