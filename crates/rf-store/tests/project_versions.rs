use rf_store::{
    parse_project_file_json, project_file_to_pretty_json, read_project_file, write_project_file,
};
use rf_types::units::{DisplayUnitSet, MeasurementUnit, QuantityKind};
use serde_json::{Value, json};

const V2: &str = include_str!("fixtures/project-v2-engineering.json");
const V1: &str =
    include_str!("../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json");

fn rejected(value: Value, expected: &str) {
    let error = parse_project_file_json(&value.to_string()).unwrap_err();
    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(error.message().contains(expected), "{}", error.message());
}

#[test]
fn v1_loads_as_si_without_changing_document_or_upgrading_writes() {
    let project = parse_project_file_json(V1).unwrap();
    assert_eq!(project.schema_version, 1);
    assert_eq!(project.presentation.display_units, DisplayUnitSet::si());
    let before: Value = serde_json::from_str(V1).unwrap();
    let written: Value =
        serde_json::from_str(&project_file_to_pretty_json(&project).unwrap()).unwrap();
    assert_eq!(written["schemaVersion"], 1);
    assert!(written.get("presentation").is_none());
    assert_eq!(before["document"], written["document"]);
    assert_eq!(
        parse_project_file_json(&written.to_string()).unwrap(),
        project
    );
}

#[test]
fn v2_preserves_complete_choices_and_independent_metadata_version() {
    let project = parse_project_file_json(V2).unwrap();
    assert_eq!(project.schema_version, 2);
    assert_eq!(project.document.metadata.schema_version, 1);
    assert_eq!(project.document.revision, 7);
    assert_eq!(
        project.presentation.display_units,
        DisplayUnitSet::engineering()
    );
    let written = project_file_to_pretty_json(&project).unwrap();
    assert_eq!(parse_project_file_json(&written).unwrap(), project);
    assert_eq!(
        serde_json::from_str::<Value>(&written).unwrap(),
        serde_json::from_str::<Value>(V2).unwrap()
    );
}

#[test]
fn serialization_uses_catalog_order_and_preserves_custom_complete_choices() {
    let mut value: Value = serde_json::from_str(V2).unwrap();
    value["presentation"]["displayUnits"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let mut project = parse_project_file_json(&value.to_string()).unwrap();
    project
        .presentation
        .display_units
        .set_unit(
            QuantityKind::MolarEnthalpy,
            MeasurementUnit::KilojoulePerMole,
        )
        .unwrap();
    let written: Value =
        serde_json::from_str(&project_file_to_pretty_json(&project).unwrap()).unwrap();
    assert_eq!(
        written["presentation"]["displayUnits"][0]["quantityId"],
        "absolute_temperature"
    );
    assert_eq!(
        written["presentation"]["displayUnits"][6]["unitId"],
        "kilojoule_per_mole"
    );
    assert_eq!(written["document"], value["document"]);
}

#[test]
fn v2_rejects_missing_null_duplicate_unknown_and_incompatible_configuration() {
    let base: Value = serde_json::from_str(V2).unwrap();
    let mut value = base.clone();
    value.as_object_mut().unwrap().remove("presentation");
    rejected(value, "presentation");
    for invalid in [
        Value::Null,
        json!({}),
        json!({"displayUnits": []}),
        json!({"displayUnits": null}),
    ] {
        let mut value = base.clone();
        value["presentation"] = invalid;
        assert!(parse_project_file_json(&value.to_string()).is_err());
    }
    let mut value = base.clone();
    value["presentation"]["displayUnits"]
        .as_array_mut()
        .unwrap()
        .pop();
    rejected(value, "missing display unit for molar_enthalpy");
    let mut value = base.clone();
    let duplicate = value["presentation"]["displayUnits"][0].clone();
    value["presentation"]["displayUnits"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    rejected(value, "duplicate display unit for absolute_temperature");
    for (key, invalid, error) in [
        ("quantityId", "future_quantity", "unknown quantity ID"),
        ("unitId", "future_unit", "unknown unit ID"),
        ("unitId", "°C", "unknown unit ID"),
        ("unitId", "celsius_difference", "incompatible"),
    ] {
        let mut value = base.clone();
        value["presentation"]["displayUnits"][0][key] = json!(invalid);
        rejected(value, error);
    }
    let mut value = base.clone();
    value["presentation"]["displayUnits"][2]["unitId"] = json!("bar_gauge");
    rejected(value, "requires ReferencePressure");
    let mut value = base.clone();
    value["presentation"]["precision"] = json!(8);
    rejected(value, "unknown field `precision`");
    let mut value = base;
    value["presentation"]["displayUnits"][0]["scale"] = json!(1);
    rejected(value, "unknown field `scale`");
}

#[test]
fn original_json_duplicate_keys_are_rejected_before_information_is_lost() {
    for (old, new) in [
        (
            "\"schemaVersion\": 2",
            "\"schemaVersion\": 2, \"schemaVersion\": 2",
        ),
        (
            "\"kind\":",
            "\"kind\": \"radishflow.project-file\", \"kind\":",
        ),
        ("\"document\":", "\"document\": {}, \"document\":"),
        (
            "\"presentation\":",
            "\"presentation\": {}, \"presentation\":",
        ),
        (
            "\"displayUnits\":",
            "\"displayUnits\": [], \"displayUnits\":",
        ),
        (
            "\"quantityId\": \"absolute_temperature\"",
            "\"quantityId\": \"absolute_temperature\", \"quantityId\": \"absolute_temperature\"",
        ),
        (
            "\"unitId\": \"celsius\"",
            "\"unitId\": \"celsius\", \"unitId\": \"kelvin\"",
        ),
    ] {
        let changed = V2.replacen(old, new, 1);
        assert_ne!(changed, V2);
        assert!(
            parse_project_file_json(&changed).is_err(),
            "accepted duplicate: {old}"
        );
    }
    // A duplicate empty document may fail body validation first; neither case may load.
    let duplicate = V1.replacen(
        "\"schemaVersion\": 1",
        "\"schemaVersion\": 1, \"schemaVersion\": 1",
        1,
    );
    assert!(
        parse_project_file_json(&duplicate)
            .unwrap_err()
            .message()
            .contains("duplicate field")
    );
}

#[test]
fn versions_are_explicit_and_lossy_downgrade_cannot_be_serialized() {
    let base: Value = serde_json::from_str(V2).unwrap();
    let mut future = base.clone();
    future["schemaVersion"] = json!(3);
    rejected(future, "newer than supported version");
    let mut old_metadata = base.clone();
    old_metadata["document"]["metadata"]["schemaVersion"] = json!(2);
    rejected(old_metadata, "metadata schema version");
    let mut disguised = base;
    disguised["schemaVersion"] = json!(1);
    rejected(disguised, "v1 must not contain presentation");
    let mut project = parse_project_file_json(V2).unwrap();
    project.schema_version = 1;
    assert!(
        project_file_to_pretty_json(&project)
            .unwrap_err()
            .message()
            .contains("cannot preserve non-SI")
    );
    assert!(serde_json::to_string(&project).is_err());
}

#[test]
fn rejected_read_and_write_leave_original_bytes_unchanged() {
    let root = std::env::temp_dir().join(format!(
        "rf-store-presentation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("project.rfproj.json");
    let invalid = V2.replace("\"celsius\"", "\"unknown-unit\"");
    std::fs::write(&path, &invalid).unwrap();
    assert!(read_project_file(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
    std::fs::write(&path, V1).unwrap();
    let mut project = parse_project_file_json(V2).unwrap();
    project.schema_version = 1;
    assert!(write_project_file(&path, &project).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), V1);
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "fixtures/project_v1_version_gate.rs"]
mod frozen_v1;

#[test]
fn frozen_old_version_gate_accepts_v1_and_rejects_actual_v2_writer_output() {
    assert!(frozen_v1::check(V1).is_ok());
    let project = parse_project_file_json(V2).unwrap();
    let written = project_file_to_pretty_json(&project).unwrap();
    let error = frozen_v1::check(&written).unwrap_err();
    assert!(
        error
            .message()
            .contains("schema version `2` is newer than supported version `1`")
    );
}

#[test]
fn direct_serde_callers_share_version_validation_and_round_trip_both_versions() {
    for source in [V1, V2] {
        let project: rf_store::StoredProjectFile = serde_json::from_str(source).unwrap();
        assert_eq!(project, parse_project_file_json(source).unwrap());
        let serialized = serde_json::to_string(&project).unwrap();
        assert_eq!(
            serde_json::from_str::<rf_store::StoredProjectFile>(&serialized).unwrap(),
            project
        );
    }
    let mut value: Value = serde_json::from_str(V2).unwrap();
    value.as_object_mut().unwrap().remove("presentation");
    assert!(serde_json::from_value::<rf_store::StoredProjectFile>(value).is_err());
    for version in [1, 2] {
        let mut value: Value = serde_json::from_str(V2).unwrap();
        value["schemaVersion"] = json!(version);
        value["presentation"] = Value::Null;
        assert!(parse_project_file_json(&value.to_string()).is_err());
    }
}
