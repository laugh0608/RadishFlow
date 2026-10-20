use super::*;
use crate::FileOverwritePolicy;
use rf_types::units::{MeasurementUnit, QuantityKind};
use serde_json::{Value, json};
use std::fs;
mod rating;

// These are constructed storage fixtures, not real equipment data or B04 evidence.
fn fixture() -> Value {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/project-v2-engineering.json"
    ))
    .unwrap();
    value["schemaVersion"] = json!(3);
    let units = value["presentation"]["displayUnits"]
        .as_array_mut()
        .unwrap();
    for (quantity, unit) in [
        ("actual_volume_flow", "cubic_metre_per_hour"),
        ("mass_density", "kilogram_per_cubic_metre"),
        ("velocity", "metre_per_second"),
        ("duration", "minute"),
        ("length", "millimetre"),
        ("area", "square_metre"),
        ("volume", "cubic_metre"),
        ("dimensionless_ratio", "one"),
    ] {
        units.push(json!({"quantityId":quantity,"unitId":unit}));
    }
    value["engineering"] = json!({
        "basisEntries":[{"id":"basis-time","revision":7,"name":"constructed time","quantityId":"duration","valueSi":120.0,"sourceId":"source-test"}],
        "sources":[{
            "id":"source-test","revision":3,"kind":"assumed","title":"Structure test only",
            "reference":"","locator":"","publicationVersion":"","accessedOn":"2026-10-11",
            "adoptionNote":"Constructed fixture; no engineering claim","review":"unreviewed",
            "applicability":{"fluidSystem":"","phase":"notApplicable","operatingConditions":{},"scope":"storage structure only"},
            "observations":[{"id":"time","quantityId":"duration","originalValue":2.0,"originalUnit":"min","canonicalValue":120.0,"conversionNote":"2 * 60","dependsOn":[]}]
        }],"tasks":[]
    });
    value
}
fn foundation(value: &Value) -> ProjectV3Read {
    parse(&value.to_string()).unwrap()
}
fn rejected(value: Value, path: &str) {
    let error = parse(&value.to_string()).unwrap_err();
    assert!(error.message().contains(path), "{}", error.message());
}
fn temp_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rf-v3-foundation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn foundation_round_trip_preserves_document_provenance_and_all_fifteen_units() {
    let expected = fixture();
    let data = foundation(&expected);
    let ProjectV3Read::Editable(project) = &data else {
        panic!("foundation expected")
    };
    assert_eq!(
        project.display_units.unit_for(QuantityKind::Duration),
        MeasurementUnit::Minute
    );
    let basis = &project.engineering.as_ref().unwrap().basis;
    assert_eq!(basis.basis_entries[0].revision, 7);
    assert_eq!(basis.sources[0].observations[0].original_unit, "min");
    assert_eq!(
        basis.sources[0].review,
        rf_model::engineering::SourceReview::Unreviewed
    );
    let json = to_pretty_json(&data).unwrap();
    assert_eq!(parse(&json).unwrap(), data);
    let actual: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual["document"]["metadata"]["schemaVersion"], 1);
    assert_eq!(
        actual["presentation"]["displayUnits"]
            .as_array()
            .unwrap()
            .len(),
        15
    );
    assert_eq!(actual["engineering"]["tasks"], json!([]));
}

#[test]
fn omitted_and_empty_engineering_are_preserved_and_null_is_rejected() {
    let mut value = fixture();
    value.as_object_mut().unwrap().remove("engineering");
    let json = to_pretty_json(&foundation(&value)).unwrap();
    assert!(
        serde_json::from_str::<Value>(&json)
            .unwrap()
            .get("engineering")
            .is_none()
    );
    value["engineering"] = json!({"sources":[],"basisEntries":[],"tasks":[]});
    assert_eq!(
        serde_json::from_str::<Value>(&to_pretty_json(&foundation(&value)).unwrap()).unwrap(),
        value
    );
    value["engineering"] = Value::Null;
    rejected(value, "engineering");
}

#[test]
fn malformed_presentation_is_not_repaired_or_downgraded() {
    for mutation in 0..5 {
        let mut value = fixture();
        let entries = value["presentation"]["displayUnits"]
            .as_array_mut()
            .unwrap();
        match mutation {
            0 => {
                entries.pop();
            }
            1 => {
                entries.push(entries[0].clone());
            }
            2 => {
                entries[0]["quantityId"] = json!("future_quantity");
            }
            3 => {
                entries[0]["unitId"] = json!("metre");
            }
            _ => {
                entries[0]["unitId"] = json!("unknown_unit");
            }
        }
        rejected(value, "presentation.displayUnits");
    }
}

#[test]
fn source_and_basis_structure_errors_include_context_and_never_partially_load() {
    for mutation in 0..13 {
        let mut value = fixture();
        match mutation {
            0 => value["engineering"]["basisEntries"][0]["sourceId"] = json!("missing"),
            1 => {
                let b = value["engineering"]["basisEntries"][0].clone();
                value["engineering"]["basisEntries"]
                    .as_array_mut()
                    .unwrap()
                    .push(b);
            }
            2 => value["engineering"]["sources"][0]["accessedOn"] = json!("2026-02-29"),
            3 => {
                value["engineering"]["sources"][0]["applicability"]["operatingConditions"] =
                    json!({"temperatureK":300.0})
            }
            4 => {
                value["engineering"]["sources"][0]["applicability"]["operatingConditions"] =
                    json!({"temperatureK":null,"pressurePa":1.0})
            }
            5 => {
                value["engineering"]["sources"][0]["applicability"]["operatingConditions"] =
                    json!({"temperatureK":300.0,"pressurePa":-1.0})
            }
            6 => {
                value["engineering"]["sources"][0]["observations"][0]["dependsOn"] =
                    json!([{"sourceId":"source-test","observationId":"time"}])
            }
            7 => {
                value["engineering"]["sources"][0]["observations"][0]["dependsOn"] =
                    json!([{"sourceId":"missing","observationId":"time"}])
            }
            8 => value["engineering"]["sources"][0]["review"] = json!("certified"),
            9 => value["engineering"]["basisEntries"][0]["quantityId"] = json!("unknown_quantity"),
            10 => value["engineering"]["sources"][0]["futureField"] = json!(true),
            11 => value["engineering"]["sources"][0]
                .as_object_mut()
                .unwrap()
                .remove("observations")
                .map(|_| ())
                .unwrap(),
            _ => value["engineering"]["basisEntries"][0]["valueSi"] = json!("120"),
        }
        rejected(value, "engineering");
    }
}

#[test]
fn source_dependencies_and_paired_conditions_round_trip_without_executing_text() {
    let mut value = fixture();
    let mut source = value["engineering"]["sources"][0].clone();
    source["id"] = json!("derived");
    source["accessedOn"] = json!("");
    source["observations"][0]["dependsOn"] =
        json!([{"sourceId":"source-test","observationId":"time"}]);
    source["observations"][0]["originalUnit"] = json!("external documentary unit");
    source["observations"][0]["conversionNote"] = json!("externally adopted; do not execute");
    source["applicability"]["operatingConditions"] =
        json!({"temperatureK":300.0,"pressurePa":100000.0});
    value["engineering"]["sources"]
        .as_array_mut()
        .unwrap()
        .push(source);
    let stored = to_pretty_json(&foundation(&value)).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(), value);
}

#[test]
fn versions_unknown_fields_and_original_duplicate_keys_are_rejected() {
    let text = fixture().to_string();
    for invalid in [
        text.replace(
            "\"schemaVersion\":3",
            "\"schemaVersion\":3,\"schemaVersion\":3",
        ),
        text.replace("\"sources\":[", "\"sources\":[],\"sources\":["),
        text.replace("\"valueSi\":120.0", "\"valueSi\":120.0,\"valueSi\":1.0"),
        text.replace("\"engineering\":", "\"extra\":1,\"engineering\":"),
        text.replace("\"schemaVersion\":3", "\"schemaVersion\":4"),
        text.replace("\"schemaVersion\":3", "\"schemaVersion\":2"),
    ] {
        assert_ne!(invalid, text);
        assert!(parse(&invalid).is_err(), "{invalid}");
    }
    // Product entry points still reject every v3, including an omitted engineering block.
    let mut value = fixture();
    for present in [true, false] {
        if !present {
            value.as_object_mut().unwrap().remove("engineering");
        }
        assert!(
            crate::parse_project_file_json(&value.to_string())
                .unwrap_err()
                .message()
                .contains("schema")
        );
        assert!(serde_json::from_str::<crate::StoredProjectFile>(&value.to_string()).is_err());
    }
    assert_eq!(crate::STORED_PROJECT_FILE_SCHEMA_VERSION, 2);
}

fn unsupported_text(method: &str, version: u64) -> String {
    let task = format!(
        r#"{{"id":"task","kind":"verticalGasLiquidSeparatorSizing","method":{{"id":"{method}","version":{version}}},"future":{{"integer":900719925474099312345678901234567890,"decimal":0.123456789012345678901234567890123456789,"large":1e9999}}}}"#
    );
    fixture()
        .to_string()
        .replace("\"tasks\":[]", &format!("\"tasks\":[{task}]"))
}

#[test]
fn unsupported_equipment_tasks_are_opaque_read_only_with_exact_original_numeric_text() {
    for (method, version) in [
        ("future.method", 99),
        ("radishflow.separator.vertical-sb-working-volume", 99),
    ] {
        let text = unsupported_text(method, version);
        let read = parse(&text).unwrap();
        let ProjectV3Read::ReadOnly(opaque) = &read else {
            panic!("task must be read-only")
        };
        assert_eq!(opaque.original_json().as_bytes(), text.as_bytes());
        assert_eq!(opaque.task_count(), 1);
        assert_eq!(opaque.document().metadata.schema_version, 1);
        assert!(
            to_pretty_json(&read)
                .unwrap_err()
                .message()
                .contains("read-only")
        );
    }
}

#[test]
fn safe_write_round_trips_and_protects_original_unsupported_or_legacy_files() {
    let root = temp_dir();
    let path = root.join("project.rfproj.json");
    let data = foundation(&fixture());
    write(&path, &data, FileOverwritePolicy::Forbid).unwrap();
    assert_eq!(read(&path).unwrap(), data);
    let original = fs::read(&path).unwrap();
    assert!(write(&path, &data, FileOverwritePolicy::Forbid).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    let mut changed = data.clone();
    let ProjectV3Read::Editable(project) = &mut changed else {
        unreachable!()
    };
    project.engineering.as_mut().unwrap().basis.basis_entries[0].name = "renamed".into();
    write(&path, &changed, FileOverwritePolicy::Allow).unwrap();
    assert_eq!(read(&path).unwrap(), changed);
    let unknown = unsupported_text("future", 2);
    fs::write(&path, &unknown).unwrap();
    assert!(write(&path, &data, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), unknown);
    assert!(
        write(
            root.join("copy.json"),
            &parse(&unknown).unwrap(),
            FileOverwritePolicy::Forbid
        )
        .is_err()
    );
    assert!(!root.join("copy.json").exists());
    let legacy = include_str!("../../tests/fixtures/project-v2-engineering.json");
    fs::write(&path, legacy).unwrap();
    assert!(write(&path, &data, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_writes_and_io_failures_leave_existing_data_and_no_staging_files() {
    let root = temp_dir();
    let path = root.join("project.rfproj.json");
    let mut data = foundation(&fixture());
    write(&path, &data, FileOverwritePolicy::Forbid).unwrap();
    let before = fs::read(&path).unwrap();
    let ProjectV3Read::Editable(project) = &mut data else {
        unreachable!()
    };
    project.engineering.as_mut().unwrap().basis.sources[0].observations[0].canonical_value =
        f64::NAN;
    assert!(write(&path, &data, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    let data = foundation(&fixture());
    assert!(
        write(
            root.join("missing-parent/project.json"),
            &data,
            FileOverwritePolicy::Allow
        )
        .is_err()
    );
    assert!(write(&root, &data, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

mod equipment;
