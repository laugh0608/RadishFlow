use std::time::SystemTime;

use rf_model::Flowsheet;

pub type DateTimeUtc = SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDocumentMetadata {
    pub title: String,
    pub schema_version: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl StoredDocumentMetadata {
    pub fn new(title: impl Into<String>, created_at: DateTimeUtc) -> Self {
        Self {
            title: title.into(),
            schema_version: 1,
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
    pub schema_version: u32,
    pub document: StoredProjectDocument,
}

impl StoredProjectFile {
    pub fn new(flowsheet: Flowsheet, metadata: StoredDocumentMetadata) -> Self {
        Self {
            schema_version: 1,
            document: StoredProjectDocument {
                revision: 0,
                flowsheet,
                metadata,
            },
        }
    }
}
