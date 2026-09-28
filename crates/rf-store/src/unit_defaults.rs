use std::{fs, path::Path};

use rf_types::{RfError, RfResult, units::DisplayUnitSet};
use serde::{Deserialize, Serialize};

use crate::{FileOverwritePolicy, StoredProjectPresentation, write_text_file};

pub const UNIT_DEFAULTS_FILE_NAME: &str = "unit-defaults.rfstudio-preferences.json";
const KIND: &str = "radishflow.studio-unit-defaults";
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitDefaultsFile {
    kind: String,
    schema_version: u32,
    presentation: StoredProjectPresentation,
}

pub fn parse_unit_defaults(contents: &str) -> RfResult<DisplayUnitSet> {
    let file: UnitDefaultsFile = serde_json::from_str(contents)
        .map_err(|error| RfError::invalid_input(format!("read unit defaults: {error}")))?;
    if file.kind != KIND || file.schema_version != VERSION {
        return Err(RfError::invalid_input(format!(
            "unsupported unit defaults kind {:?} or version {}",
            file.kind, file.schema_version
        )));
    }
    Ok(file.presentation.display_units)
}

pub fn read_unit_defaults(path: &Path) -> RfResult<Option<DisplayUnitSet>> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_unit_defaults(&contents).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(RfError::invalid_input(format!(
            "read unit defaults {}: {error}",
            path.display()
        ))),
    }
}

/// The caller controls recovery consent. Normal writes first validate any existing file.
pub fn write_unit_defaults(path: &Path, units: &DisplayUnitSet) -> RfResult<()> {
    read_unit_defaults(path)?;
    write_validated_unit_defaults(path, units)
}

pub(crate) fn write_validated_unit_defaults(path: &Path, units: &DisplayUnitSet) -> RfResult<()> {
    let file = UnitDefaultsFile {
        kind: KIND.to_string(),
        schema_version: VERSION,
        presentation: StoredProjectPresentation {
            display_units: units.clone(),
        },
    };
    let contents = serde_json::to_string_pretty(&file)
        .map_err(|error| RfError::invalid_input(format!("serialize unit defaults: {error}")))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            RfError::invalid_input(format!(
                "create unit defaults directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    write_text_file(path, &contents, FileOverwritePolicy::Allow)
}

/// Explicit recovery preserves the exact old bytes before any replacement is attempted.
/// A failed replacement leaves the backup available and does not mark the default as saved.
pub fn recover_unit_defaults(path: &Path, units: &DisplayUnitSet) -> RfResult<std::path::PathBuf> {
    use std::io::Write;
    let old = fs::read(path).map_err(|error| {
        RfError::invalid_input(format!(
            "read unit defaults for recovery {}: {error}",
            path.display()
        ))
    })?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| {
            RfError::invalid_input(format!("unit defaults backup timestamp: {error}"))
        })?
        .as_nanos();
    let backup = path.with_extension(format!("json.{stamp}.bak"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
        .map_err(|error| {
            RfError::invalid_input(format!(
                "create unit defaults backup {}: {error}",
                backup.display()
            ))
        })?;
    file.write_all(&old)
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            RfError::invalid_input(format!(
                "write unit defaults backup {}: {error}",
                backup.display()
            ))
        })?;
    write_validated_unit_defaults(path, units).map_err(|error| {
        RfError::invalid_input(format!(
            "{}; original retained at {}",
            error,
            backup.display()
        ))
    })?;
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_reject_unknown_duplicate_and_incomplete_configuration() {
        let f = UnitDefaultsFile {
            kind: KIND.into(),
            schema_version: VERSION,
            presentation: StoredProjectPresentation {
                display_units: DisplayUnitSet::engineering(),
            },
        };
        let json = serde_json::to_string(&f).unwrap();
        assert_eq!(
            parse_unit_defaults(&json).unwrap(),
            DisplayUnitSet::engineering()
        );
        for bad in [
            json.replace("celsius", "unknown-unit"),
            json.replace("\"schemaVersion\":1", "\"schemaVersion\":99"),
            json.replace("\"schemaVersion\":1", "\"schemaVersion\":1,\"schemaVersion\":1"),
            json.replace("\"presentation\":", "\"extra\":true,\"presentation\":"),
            "{\"kind\":\"radishflow.studio-unit-defaults\",\"schemaVersion\":1,\"presentation\":{\"displayUnits\":[]}}".into(),
        ] { assert!(parse_unit_defaults(&bad).is_err(), "{bad}"); }
    }

    #[test]
    fn ordinary_write_protects_corrupt_defaults_and_explicit_recovery_preserves_exact_bytes() {
        let root = std::env::temp_dir().join(format!(
            "rf-unit-default-recovery-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join(UNIT_DEFAULTS_FILE_NAME);
        assert_eq!(read_unit_defaults(&path).unwrap(), None);
        assert!(!path.exists());
        let recent = root.join("preferences.rfstudio-preferences.json");
        fs::write(&recent, "recent-list-original").unwrap();
        let bad = b"invalid defaults\xff";
        fs::write(&path, bad).unwrap();
        assert!(write_unit_defaults(&path, &DisplayUnitSet::engineering()).is_err());
        assert_eq!(fs::read(&path).unwrap(), bad);
        let backup = recover_unit_defaults(&path, &DisplayUnitSet::engineering()).unwrap();
        assert_eq!(fs::read(backup).unwrap(), bad);
        assert_eq!(
            read_unit_defaults(&path).unwrap(),
            Some(DisplayUnitSet::engineering())
        );
        assert_eq!(fs::read_to_string(recent).unwrap(), "recent-list-original");
        fs::remove_dir_all(root).unwrap();
    }
}
