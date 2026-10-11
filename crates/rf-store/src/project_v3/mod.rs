//! Foundation-only v3 codec, deliberately separate from the product project reader.
//! Studio, CLI and StoredProjectFile continue to accept only v1/v2. No equipment
//! method is registered here: every nonempty task array makes the entire file read-only.
mod engineering_wire;
mod file_io;
mod presentation_wire;
pub use file_io::{read, write};

use crate::{StoredProjectDocument, project::STORED_PROJECT_FILE_KIND};
use engineering_wire::EngineeringWire;
use presentation_wire::PresentationWire;
use rf_model::engineering::EngineeringBasis;
use rf_types::{RfError, RfResult, units::DisplayUnitSet};
use serde::{Deserialize, Deserializer, Serialize};

const VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredProjectV3Foundation {
    pub document: StoredProjectDocument,
    pub display_units: DisplayUnitSet,
    /// None preserves an omitted engineering block. Some(empty) preserves an
    /// explicitly present block with three empty arrays, as required by v3.
    pub engineering: Option<EngineeringBasis>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectV3Read {
    Foundation(StoredProjectV3Foundation),
    ReadOnly(ReadOnlyProjectV3),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadOnlyProjectV3 {
    original_json: String,
    document: StoredProjectDocument,
    task_count: usize,
}
impl ReadOnlyProjectV3 {
    /// Original UTF-8 text, including unsupported fields and numeric lexemes.
    pub fn original_json(&self) -> &str {
        &self.original_json
    }
    pub fn document(&self) -> &StoredProjectDocument {
        &self.document
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectWire {
    kind: String,
    schema_version: u32,
    #[serde(deserialize_with = "document")]
    document: StoredProjectDocument,
    #[serde(deserialize_with = "presentation")]
    presentation: PresentationWire,
    #[serde(
        default,
        deserialize_with = "engineering",
        skip_serializing_if = "Option::is_none"
    )]
    engineering: Option<EngineeringWire>,
}

fn document<'de, D: Deserializer<'de>>(de: D) -> Result<StoredProjectDocument, D::Error> {
    StoredProjectDocument::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("document: {e}")))
}
fn presentation<'de, D: Deserializer<'de>>(de: D) -> Result<PresentationWire, D::Error> {
    PresentationWire::deserialize(de)
        .map_err(|e| serde::de::Error::custom(format!("presentation: {e}")))
}
fn engineering<'de, D: Deserializer<'de>>(de: D) -> Result<Option<EngineeringWire>, D::Error> {
    // A missing field is None; an explicit null cannot deserialize as EngineeringWire.
    EngineeringWire::deserialize(de)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(format!("engineering: {e}")))
}

/// Parse original text twice: the version probe never converts JSON into Value,
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
            "schemaVersion: foundation v3 codec requires version 3, got {}",
            header.schema_version
        )));
    }
    let wire: ProjectWire = serde_json::from_str(contents).map_err(json_error)?;
    wire.document.validate()?;
    let display_units = wire.presentation.into_display_units()?;
    let task_count = wire.engineering.as_ref().map_or(0, |e| e.tasks.len());
    let engineering = wire
        .engineering
        .map(EngineeringWire::into_basis)
        .transpose()?;
    if task_count > 0 {
        return Ok(ProjectV3Read::ReadOnly(ReadOnlyProjectV3 {
            original_json: contents.to_string(),
            document: wire.document,
            task_count,
        }));
    }
    Ok(ProjectV3Read::Foundation(StoredProjectV3Foundation {
        document: wire.document,
        display_units,
        engineering,
    }))
}

/// Reject read-only documents rather than reserializing a partially understood file.
/// Only this scoped API emits v3; the product writer remains unchanged.
pub fn to_pretty_json(project: &ProjectV3Read) -> RfResult<String> {
    let ProjectV3Read::Foundation(project) = project else {
        return Err(RfError::invalid_input(
            "engineering.tasks: equipment capability is not implemented; entire project is read-only",
        ));
    };
    project.document.validate()?;
    if let Some(basis) = &project.engineering {
        basis
            .validate()
            .map_err(|e| RfError::invalid_input(format!("engineering.{e}")))?;
    }
    let wire = ProjectWire {
        kind: STORED_PROJECT_FILE_KIND.into(),
        schema_version: VERSION,
        document: project.document.clone(),
        presentation: PresentationWire::from(&project.display_units),
        engineering: project.engineering.as_ref().map(EngineeringWire::from),
    };
    serde_json::to_string_pretty(&wire).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> RfError {
    RfError::invalid_input(format!("project v3 JSON: {error}"))
}

#[cfg(test)]
mod tests;
