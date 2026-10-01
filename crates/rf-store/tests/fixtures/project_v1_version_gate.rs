// Frozen original v1 version gate from f9f986c0, crates/rf-store/src/json.rs.
// Function bodies below are verbatim; do not adapt them to the new reader.
// This tests the old rejection boundary, not old UI behavior or unknown-data round trips.
use rf_types::{RfError, RfResult};
use serde_json::Value;
const STORED_PROJECT_FILE_KIND: &str = "radishflow.project-file";
const STORED_PROJECT_FILE_SCHEMA_VERSION: u32 = 1;

pub fn check(contents: &str) -> RfResult<Value> {
    let value = serde_json::from_str(contents).expect("valid fixture JSON");
    migrate_project_file_value(value)
}

fn migrate_project_file_value(value: Value) -> RfResult<Value> {
    let envelope = parse_stored_envelope(&value, "stored project file")?;

    if envelope.kind.as_deref() != Some(STORED_PROJECT_FILE_KIND) {
        return Err(RfError::invalid_input(format!(
            "unsupported stored project file kind `{}`",
            envelope.kind.unwrap_or_default()
        )));
    }

    match envelope.schema_version {
        STORED_PROJECT_FILE_SCHEMA_VERSION => migrate_project_file_v1_to_current(value),
        version if version > STORED_PROJECT_FILE_SCHEMA_VERSION => Err(newer_schema_error(
            "stored project file",
            version,
            STORED_PROJECT_FILE_SCHEMA_VERSION,
        )),
        version => Err(older_schema_error(
            "stored project file",
            version,
            STORED_PROJECT_FILE_SCHEMA_VERSION,
        )),
    }
}

fn migrate_project_file_v1_to_current(value: Value) -> RfResult<Value> {
    Ok(value)
}

fn parse_stored_envelope(value: &Value, entity_name: &str) -> RfResult<StoredEnvelope> {
    let envelope: StoredEnvelope = serde_json::from_value(value.clone()).map_err(|error| {
        RfError::invalid_input(format!("deserialize {entity_name} envelope: {error}"))
    })?;

    if envelope.kind.is_none() {
        return Err(RfError::invalid_input(format!(
            "{entity_name} is missing required field `kind`"
        )));
    }

    if envelope.schema_version == 0 {
        return Err(RfError::invalid_input(format!(
            "{entity_name} is missing required field `schemaVersion`"
        )));
    }

    Ok(envelope)
}

fn newer_schema_error(entity_name: &str, version: u32, supported_version: u32) -> RfError {
    RfError::invalid_input(format!(
        "{entity_name} schema version `{version}` is newer than supported version `{supported_version}`; add a migration in rf-store before loading it"
    ))
}

fn older_schema_error(entity_name: &str, version: u32, supported_version: u32) -> RfError {
    RfError::invalid_input(format!(
        "{entity_name} schema version `{version}` is older than supported version `{supported_version}`; add an explicit migration path in rf-store before loading it"
    ))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredEnvelope {
    kind: Option<String>,
    #[serde(default)]
    schema_version: u32,
}
