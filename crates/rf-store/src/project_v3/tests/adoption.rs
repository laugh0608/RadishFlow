use super::*;
use rf_model::equipment::*;

// Produced by the C1 application transaction and real writer, using public
// synthetic inputs. Includes sizing, rating v1, matrix v2 and adopted v3.
pub(super) const ADOPTED: &str =
    include_str!("../../../tests/fixtures/project-v3-rating-adoption.json");
fn fixture() -> Value {
    serde_json::from_str(ADOPTED).unwrap()
}
fn adoption(value: &mut Value) -> &mut Value {
    &mut value["engineering"]["tasks"][3]["geometry"]["adoption"]
}

#[test]
fn fixed_adoption_and_mixed_capabilities_round_trip_without_result_eligibility() {
    let parsed = parse(ADOPTED).unwrap();
    let ProjectV3Read::Editable(project) = &parsed else {
        panic!()
    };
    assert_eq!(project.rating_method_version(), 3);
    assert_eq!(project.engineering.as_ref().unwrap().tasks.len(), 4);
    let task = project.engineering.as_ref().unwrap().tasks[3]
        .as_rating()
        .unwrap();
    let evidence = task.geometry.adoption.as_ref().unwrap();
    assert_eq!(evidence.source_task.id.as_str(), "sizing-source");
    assert_eq!(evidence.diameter.adopted_m, 1.6);
    assert_ne!(evidence.diameter.original_m, evidence.diameter.adopted_m);
    assert_eq!(evidence.axial_lengths_m.len(), 6);
    let text = to_pretty_json(&parsed).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), fixture());
    assert_eq!(parse(&text).unwrap(), parsed);
    for forbidden in ["calculationId", "checkpoint", "eligibility", "loadFraction"] {
        assert!(!text.contains(forbidden));
    }
}

#[test]
fn adoption_codec_rejects_malformed_identity_evidence_bindings_dimensions_and_fields() {
    for mutation in 0..17 {
        let mut value = fixture();
        let a = adoption(&mut value);
        match mutation {
            0 => a["sourceDocumentId"] = json!(" "),
            1 => a["sourceTask"]["id"] = json!(""),
            2 => a["sourceTask"]["case"]["id"] = json!(""),
            3 => a["sourceTask"]["method"]["version"] = json!(2),
            4 => {
                a["sourceTask"]["inputs"]["maximumWorkingLiquidHeight"]["sourceId"] =
                    json!("absent")
            }
            5 => a["sourceTask"]["inputs"]
                .as_object_mut()
                .unwrap()
                .remove("gasMassDensity")
                .map(|_| ())
                .unwrap(),
            6 => a["diameter"]["reason"] = json!(" "),
            7 => a["workingLiquidHeight"]["adoptedM"] = json!(0.0),
            8 => a["workingLiquidHeight"]["originalM"] = json!(-1.0),
            9 => a["axialSections"][0]["valueM"] = json!(999.0),
            10 => {
                let duplicate = a["axialSections"][0].clone();
                a["axialSections"].as_array_mut().unwrap().push(duplicate);
            }
            11 => a["sourceBasis"]["sources"] = json!([]),
            12 => a["unknown"] = json!(true),
            13 => a["diameter"]["unknown"] = json!(true),
            14 => value["engineering"]["tasks"][3]["method"]["version"] = json!(2),
            15 => value["engineering"]["tasks"][3]["geometry"]["adoption"] = Value::Null,
            16 => {
                a["sourceBasis"]["tasks"] = json!([{"id":"forbidden", "kind":"verticalGasLiquidSeparatorSizing", "method":{"id":"radishflow.separator.vertical-sb-working-volume", "version":1}}])
            }
            _ => unreachable!(),
        }
        assert!(parse(&value.to_string()).is_err(), "mutation {mutation}");
    }
}

#[test]
fn known_adoption_rejects_duplicate_and_overflow_while_future_capability_preserves_original() {
    let text = fixture().to_string();
    for (from, to) in [
        ("\"adoptedM\":1.6", "\"adoptedM\":1e400"),
        ("\"adoptedM\":1.6", "\"adoptedM\":1.6,\"adoptedM\":1.6"),
        (
            "\"sourceDocumentId\":",
            "\"extra\":null,\"sourceDocumentId\":",
        ),
    ] {
        assert!(text.contains(from));
        assert!(parse(&text.replace(from, to)).is_err());
    }
    let mut future = fixture();
    future["engineering"]["tasks"][3]["method"]["version"] = json!(4);
    let future = future
        .to_string()
        .replace("\"adoptedM\":1.6", "\"adoptedM\":1e400");
    let ProjectV3Read::ReadOnly(readonly) = parse(&future).unwrap() else {
        panic!()
    };
    assert_eq!(readonly.original_json(), future);
    assert!(to_pretty_json(&ProjectV3Read::ReadOnly(readonly)).is_err());
    let ProjectV3Read::Editable(mut project) = parse(ADOPTED).unwrap() else {
        panic!()
    };
    let EquipmentTask::SeparatorRating(task) = &mut project.engineering.as_mut().unwrap().tasks[3]
    else {
        panic!()
    };
    task.geometry.adoption.as_mut().unwrap().diameter.original_m = f64::NAN;
    assert!(to_pretty_json(&ProjectV3Read::Editable(project)).is_err());
}

#[test]
fn adoption_upgrade_checks_actual_target_and_preserves_bytes_on_every_rejection() {
    let root = temp_dir();
    let path = root.join("adopted.rfproj.json");
    let mut previous = fixture();
    previous["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .pop();
    let previous = previous.to_string();
    fs::write(&path, &previous).unwrap();
    let parsed = parse(ADOPTED).unwrap();
    assert!(write(&path, &parsed, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), previous);
    write_upgraded(&path, &parsed, FileOverwritePolicy::Allow).unwrap();
    write(&path, &parsed, FileOverwritePolicy::Allow).unwrap();
    assert_eq!(read(&path).unwrap(), parsed);
    let original = fs::read(&path).unwrap();
    assert!(write(&path, &parsed, FileOverwritePolicy::Forbid).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    let directory = root.join("directory");
    fs::create_dir(&directory).unwrap();
    assert!(write_upgraded(&directory, &parsed, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    let mut future = fixture();
    future["engineering"]["tasks"][3]["method"]["version"] = json!(4);
    let future = future.to_string();
    fs::write(&path, &future).unwrap();
    assert!(write_upgraded(&path, &parsed, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), future);
    fs::remove_dir_all(root).unwrap();
}
