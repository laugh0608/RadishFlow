use std::collections::BTreeSet;

use rf_types::{RfError, RfResult};
use serde::{Deserialize, Serialize};

pub const STORED_STUDIO_PREFERENCES_FILE_KIND: &str = "radishflow.studio-preferences-file";
pub const STORED_STUDIO_PREFERENCES_SCHEMA_VERSION: u32 = 1;
pub const STORED_STUDIO_PREFERENCES_FILE_NAME: &str = "preferences.rfstudio-preferences.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioPreferencesFile {
    pub kind: String,
    pub schema_version: u32,
    #[serde(default)]
    pub recent_project_paths: Vec<String>,
}

impl StoredStudioPreferencesFile {
    pub fn new(recent_project_paths: Vec<String>) -> Self {
        Self {
            kind: STORED_STUDIO_PREFERENCES_FILE_KIND.to_string(),
            schema_version: STORED_STUDIO_PREFERENCES_SCHEMA_VERSION,
            recent_project_paths,
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_STUDIO_PREFERENCES_FILE_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored studio preferences file kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != STORED_STUDIO_PREFERENCES_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored studio preferences file schema version `{}`",
                self.schema_version
            )));
        }

        let mut paths = BTreeSet::new();
        for path in &self.recent_project_paths {
            if path.trim().is_empty() {
                return Err(RfError::invalid_input(
                    "stored studio preferences file contains an empty recent_project_paths entry",
                ));
            }
            if !paths.insert(path.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored studio preferences file contains duplicate recent project path `{path}`"
                )));
            }
        }

        Ok(())
    }
}
