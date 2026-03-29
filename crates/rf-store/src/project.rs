use std::time::SystemTime;

use rf_model::Flowsheet;
use rf_types::{RfError, RfResult};
use serde::{Deserialize, Serialize};

pub type DateTimeUtc = SystemTime;
pub const STORED_PROJECT_FILE_KIND: &str = "radishflow.project-file";
pub const STORED_PROJECT_FILE_SCHEMA_VERSION: u32 = 1;
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
            schema_version: STORED_PROJECT_FILE_SCHEMA_VERSION,
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

        if self.schema_version != STORED_PROJECT_FILE_SCHEMA_VERSION {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProjectFile {
    pub kind: String,
    pub schema_version: u32,
    pub document: StoredProjectDocument,
}

impl StoredProjectFile {
    pub fn new(flowsheet: Flowsheet, metadata: StoredDocumentMetadata) -> Self {
        Self {
            kind: STORED_PROJECT_FILE_KIND.to_string(),
            schema_version: STORED_PROJECT_FILE_SCHEMA_VERSION,
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

        if self.schema_version != STORED_PROJECT_FILE_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored project file schema version `{}`",
                self.schema_version
            )));
        }

        self.document.metadata.validate()
    }
}
