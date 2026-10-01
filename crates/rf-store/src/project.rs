use std::time::SystemTime;

use rf_model::Flowsheet;
use rf_types::{RfError, RfResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

use crate::StoredProjectPresentation;

pub type DateTimeUtc = SystemTime;
pub const STORED_PROJECT_FILE_KIND: &str = "radishflow.project-file";
pub const STORED_PROJECT_FILE_SCHEMA_VERSION: u32 = 2;
pub const STORED_DOCUMENT_SCHEMA_VERSION: u32 = 1;
pub const STORED_PROJECT_FILE_EXTENSION: &str = ".rfproj.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredDocumentMetadata {
    pub document_id: String,
    pub title: String,
    pub schema_version: u32,
    #[serde(with = "crate::json::time_format")]
    pub created_at: DateTimeUtc,
    #[serde(with = "crate::json::time_format")]
    pub updated_at: DateTimeUtc,
}

impl StoredDocumentMetadata {
    pub fn new(
        document_id: impl Into<String>,
        title: impl Into<String>,
        created_at: DateTimeUtc,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            title: title.into(),
            schema_version: STORED_DOCUMENT_SCHEMA_VERSION,
            created_at,
            updated_at: created_at,
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.document_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored document metadata must contain a non-empty document_id",
            ));
        }

        if self.schema_version != STORED_DOCUMENT_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored document metadata schema version `{}`",
                self.schema_version
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProjectDocument {
    pub revision: u64,
    pub flowsheet: Flowsheet,
    pub metadata: StoredDocumentMetadata,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(try_from = "ProjectFileWire")]
pub struct StoredProjectFile {
    pub kind: String,
    pub schema_version: u32,
    pub document: StoredProjectDocument,
    pub presentation: StoredProjectPresentation,
}

impl StoredProjectFile {
    pub fn new(flowsheet: Flowsheet, metadata: StoredDocumentMetadata) -> Self {
        Self {
            kind: STORED_PROJECT_FILE_KIND.to_string(),
            schema_version: STORED_PROJECT_FILE_SCHEMA_VERSION,
            presentation: StoredProjectPresentation::default(),
            document: StoredProjectDocument {
                revision: 0,
                flowsheet,
                metadata,
            },
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_PROJECT_FILE_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored project file kind `{}`",
                self.kind
            )));
        }

        if !matches!(self.schema_version, 1 | STORED_PROJECT_FILE_SCHEMA_VERSION) {
            return Err(RfError::invalid_input(format!(
                "unsupported stored project file schema version `{}`",
                self.schema_version
            )));
        }

        if self.schema_version == 1 && self.presentation != StoredProjectPresentation::default() {
            return Err(RfError::invalid_input(
                "project file version 1 cannot preserve non-SI presentation; upgrade explicitly before saving",
            ));
        }
        self.document.metadata.validate()?;
        validate_flowsheet_thermo_config(&self.document.flowsheet)
    }
}

fn validate_flowsheet_thermo_config(flowsheet: &Flowsheet) -> RfResult<()> {
    if flowsheet
        .property_package_id()
        .is_some_and(|package_id| package_id.trim().is_empty())
    {
        return Err(RfError::invalid_input(
            "stored project flowsheet thermo property_package_id must be non-empty",
        ));
    }

    Ok(())
}

// I1 keeps explicit v1 writes available until Studio has an upgrade confirmation in I2.
// Refuse lossy downgrade even when called through serde directly.
impl Serialize for StoredProjectFile {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validate().map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct(
            "StoredProjectFile",
            if self.schema_version == 1 { 3 } else { 4 },
        )?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("schemaVersion", &self.schema_version)?;
        state.serialize_field("document", &self.document)?;
        if self.schema_version != 1 {
            state.serialize_field("presentation", &self.presentation)?;
        }
        state.end()
    }
}

// Deserialize both versions directly from the input, including public serde callers.
// A missing presentation is permitted ONLY for v1; explicit null is never a valid block.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectFileWire {
    kind: String,
    schema_version: u32,
    document: StoredProjectDocument,
    #[serde(default, deserialize_with = "deserialize_present_block")]
    presentation: Option<StoredProjectPresentation>,
}

fn deserialize_present_block<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<StoredProjectPresentation>, D::Error> {
    StoredProjectPresentation::deserialize(deserializer).map(Some)
}

impl TryFrom<ProjectFileWire> for StoredProjectFile {
    type Error = RfError;

    fn try_from(wire: ProjectFileWire) -> Result<Self, Self::Error> {
        let presentation = match (wire.schema_version, wire.presentation) {
            (1, None) => StoredProjectPresentation::default(),
            (1, Some(_)) => {
                return Err(RfError::invalid_input(
                    "project file v1 must not contain presentation",
                ));
            }
            (STORED_PROJECT_FILE_SCHEMA_VERSION, Some(presentation)) => presentation,
            (STORED_PROJECT_FILE_SCHEMA_VERSION, None) => {
                return Err(RfError::invalid_input(
                    "project file v2 requires presentation",
                ));
            }
            (version, _) => {
                return Err(RfError::invalid_input(format!(
                    "unsupported stored project file schema version `{version}`"
                )));
            }
        };
        let project = Self {
            kind: wire.kind,
            schema_version: wire.schema_version,
            document: wire.document,
            presentation,
        };
        project.validate()?;
        Ok(project)
    }
}
