use std::time::SystemTime;

use rf_model::Flowsheet;

pub type DateTimeUtc = SystemTime;
pub const STORED_PROJECT_FILE_KIND: &str = "radishflow.project-file";
pub const STORED_PROJECT_FILE_SCHEMA_VERSION: u32 = 1;
pub const STORED_PROJECT_FILE_EXTENSION: &str = ".rfproj.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDocumentMetadata {
    pub document_id: String,
    pub title: String,
    pub schema_version: u32,
    pub created_at: DateTimeUtc,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredProjectDocument {
    pub revision: u64,
    pub flowsheet: Flowsheet,
    pub metadata: StoredDocumentMetadata,
}

#[derive(Debug, Clone, PartialEq)]
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
}
