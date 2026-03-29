use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_types::{RfError, RfResult};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::auth_cache::{STORED_AUTH_CACHE_INDEX_KIND, STORED_AUTH_CACHE_SCHEMA_VERSION};
use crate::project::{STORED_PROJECT_FILE_KIND, STORED_PROJECT_FILE_SCHEMA_VERSION};
use crate::{StoredAuthCacheIndex, StoredProjectFile};

pub fn read_project_file(path: impl AsRef<Path>) -> RfResult<StoredProjectFile> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|error| map_io_error("read stored project file", path, &error))?;
    parse_project_file_json(&contents)
}

pub fn write_project_file(
    path: impl AsRef<Path>,
    project_file: &StoredProjectFile,
) -> RfResult<()> {
    project_file.validate()?;
    write_json_file(path.as_ref(), project_file, "write stored project file")
}

pub fn parse_project_file_json(contents: &str) -> RfResult<StoredProjectFile> {
    let raw_value: Value = parse_json(contents, "deserialize stored project file envelope")?;
    let migrated_value = migrate_project_file_value(raw_value)?;
    let project_file: StoredProjectFile =
        parse_json_value(migrated_value, "deserialize stored project file body")?;
    project_file.validate()?;
    Ok(project_file)
}

pub fn project_file_to_pretty_json(project_file: &StoredProjectFile) -> RfResult<String> {
    project_file.validate()?;
    to_pretty_json(project_file, "serialize stored project file")
}

pub fn read_auth_cache_index(path: impl AsRef<Path>) -> RfResult<StoredAuthCacheIndex> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|error| map_io_error("read stored auth cache index", path, &error))?;
    parse_auth_cache_index_json(&contents)
}

pub fn write_auth_cache_index(
    path: impl AsRef<Path>,
    auth_cache_index: &StoredAuthCacheIndex,
) -> RfResult<()> {
    auth_cache_index.validate()?;
    write_json_file(
        path.as_ref(),
        auth_cache_index,
        "write stored auth cache index",
    )
}

pub fn parse_auth_cache_index_json(contents: &str) -> RfResult<StoredAuthCacheIndex> {
    let raw_value: Value = parse_json(contents, "deserialize stored auth cache index envelope")?;
    let migrated_value = migrate_auth_cache_index_value(raw_value)?;
    let auth_cache_index: StoredAuthCacheIndex =
        parse_json_value(migrated_value, "deserialize stored auth cache index body")?;
    auth_cache_index.validate()?;
    Ok(auth_cache_index)
}

pub fn auth_cache_index_to_pretty_json(
    auth_cache_index: &StoredAuthCacheIndex,
) -> RfResult<String> {
    auth_cache_index.validate()?;
    to_pretty_json(auth_cache_index, "serialize stored auth cache index")
}

fn write_json_file<T>(path: &Path, value: &T, action: &str) -> RfResult<()>
where
    T: Serialize,
{
    let contents = to_pretty_json(value, action)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| map_io_error("create parent directories", parent, &error))?;
    }
    fs::write(path, contents).map_err(|error| map_io_error(action, path, &error))
}

fn parse_json<T>(contents: &str, action: &str) -> RfResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(contents)
        .map_err(|error| RfError::invalid_input(format!("{action}: {error}")))
}

fn parse_json_value<T>(value: Value, action: &str) -> RfResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value)
        .map_err(|error| RfError::invalid_input(format!("{action}: {error}")))
}

fn to_pretty_json<T>(value: &T, action: &str) -> RfResult<String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value)
        .map_err(|error| RfError::invalid_input(format!("{action}: {error}")))
}

fn map_io_error(action: &str, path: &Path, error: &std::io::Error) -> RfError {
    RfError::invalid_input(format!("{action} `{}`: {error}", path.display()))
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

fn migrate_auth_cache_index_value(value: Value) -> RfResult<Value> {
    let envelope = parse_stored_envelope(&value, "stored auth cache index")?;

    if envelope.kind.as_deref() != Some(STORED_AUTH_CACHE_INDEX_KIND) {
        return Err(RfError::invalid_input(format!(
            "unsupported stored auth cache index kind `{}`",
            envelope.kind.unwrap_or_default()
        )));
    }

    match envelope.schema_version {
        STORED_AUTH_CACHE_SCHEMA_VERSION => migrate_auth_cache_index_v1_to_current(value),
        version if version > STORED_AUTH_CACHE_SCHEMA_VERSION => Err(newer_schema_error(
            "stored auth cache index",
            version,
            STORED_AUTH_CACHE_SCHEMA_VERSION,
        )),
        version => Err(older_schema_error(
            "stored auth cache index",
            version,
            STORED_AUTH_CACHE_SCHEMA_VERSION,
        )),
    }
}

fn migrate_project_file_v1_to_current(value: Value) -> RfResult<Value> {
    Ok(value)
}

fn migrate_auth_cache_index_v1_to_current(value: Value) -> RfResult<Value> {
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

pub mod time_format {
    use std::time::SystemTime;

    use serde::{Deserialize, Deserializer, Serializer};

    use super::{OffsetDateTime, Rfc3339, system_time_from_datetime};

    pub fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = OffsetDateTime::from(*value)
            .format(&Rfc3339)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&formatted)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let datetime = OffsetDateTime::parse(&value, &Rfc3339).map_err(serde::de::Error::custom)?;
        system_time_from_datetime(datetime).map_err(serde::de::Error::custom)
    }
}

pub mod option_time_format {
    use std::time::SystemTime;

    use serde::{Deserialize, Deserializer, Serializer};

    use super::time_format;

    pub fn serialize<S>(value: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_some(&Rfc3339Time(*value)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        value
            .map(|value| {
                let wrapped = serde::de::value::StringDeserializer::<D::Error>::new(value);
                time_format::deserialize(wrapped)
            })
            .transpose()
    }

    struct Rfc3339Time(SystemTime);

    impl serde::Serialize for Rfc3339Time {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            time_format::serialize(&self.0, serializer)
        }
    }
}

pub mod relative_path_format {
    use std::path::{Component, Path, PathBuf};

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Path, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let normalized = normalize_relative_path(value);
        serializer.serialize_str(&normalized)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(PathBuf::from(value))
    }

    fn normalize_relative_path(path: &Path) -> String {
        path.components()
            .filter_map(|component| match component {
                Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
                Component::CurDir => Some(".".to_string()),
                Component::ParentDir => Some("..".to_string()),
                Component::Prefix(_) | Component::RootDir => None,
            })
            .collect::<Vec<_>>()
            .join("/")
    }
}

pub mod option_relative_path_format {
    use std::path::PathBuf;

    use serde::{Deserialize, Deserializer, Serializer};

    use super::relative_path_format;

    pub fn serialize<S>(value: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_some(&NormalizedPath(value.clone())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        Ok(value.map(PathBuf::from))
    }

    struct NormalizedPath(PathBuf);

    impl serde::Serialize for NormalizedPath {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            relative_path_format::serialize(&self.0, serializer)
        }
    }
}

fn system_time_from_datetime(datetime: OffsetDateTime) -> Result<SystemTime, String> {
    let unix_timestamp_nanos = datetime.unix_timestamp_nanos();

    if unix_timestamp_nanos >= 0 {
        let nanos = u64::try_from(unix_timestamp_nanos)
            .map_err(|_| format!("timestamp out of range: {datetime}"))?;
        Ok(UNIX_EPOCH + Duration::from_nanos(nanos))
    } else {
        let nanos = u64::try_from(unix_timestamp_nanos.unsigned_abs())
            .map_err(|_| format!("timestamp out of range: {datetime}"))?;
        Ok(UNIX_EPOCH - Duration::from_nanos(nanos))
    }
}
