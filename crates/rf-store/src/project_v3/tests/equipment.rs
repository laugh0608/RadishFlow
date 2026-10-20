use super::*;

fn task() -> Value {
    json!({"id":"task-1","revision":4,"tag":"V-101","name":"Storage fixture", "kind":"verticalGasLiquidSeparatorSizing",
        "method":{"id":"radishflow.separator.vertical-sb-working-volume","version":1},
        "case":{"id":"case-1","name":"Incomplete case","purpose":"Storage tests only","fluidSystem":"declared fluid",
            "applicabilitySourceIds":["source-test"],
            "applicability":{"orientation":"horizontal","gasDirection":"other","service":"foaming", "geometry":"reducedEffectiveArea",
                "kBasis":"mistEliminatorOnly","flowBasis":"standardConditions","internalsDescription":"Unsupported declarations must survive storage"}},
        "inputs":{
            "workingDuration":{"kind":"basisRef","basisEntryId":"basis-time"},
            "gasActualVolumeFlow":{"kind":"direct","quantityId":"actual_volume_flow","valueSi":-0.8,"sourceId":"source-test"}},
        "axialSections":{"internalsThickness":{"kind":"direct","quantityId":"length","valueSi":0.15,"sourceId":"source-test"}}
    })
}
fn with_tasks() -> Value {
    let mut value = fixture();
    value["engineering"]["tasks"] = json!([task()]);
    value
}

#[test]
fn complete_payload_and_all_bindings_round_trip_without_inference() {
    let mut value = with_tasks();
    let mut other = task();
    other["id"] = json!("task-2");
    other["case"]["id"] = json!("case-2");
    other["tag"] = json!("V-102");
    other["inputs"]["workingDuration"] = json!({"kind":"override","basisEntryId":"basis-time","quantityId":"duration","valueSi":180.0,"sourceId":"source-test","reason":"Explicit local criterion"});
    other["case"]["applicability"]["orientation"] = Value::Null;
    value["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(other);
    let parsed = parse(&value.to_string()).unwrap();
    let ProjectV3Read::Editable(project) = &parsed else {
        panic!("known method is editable");
    };
    let tasks = &project.engineering.as_ref().unwrap().tasks;
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].revision, 4);
    assert_eq!(
        tasks[0].case.applicability.orientation,
        Some(rf_model::equipment::SeparatorOrientation::Horizontal)
    );
    assert_eq!(tasks[1].case.applicability.orientation, None);
    let encoded = to_pretty_json(&parsed).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&encoded).unwrap(), value);
    assert_eq!(parse(&encoded).unwrap(), parsed);
}

#[test]
fn empty_task_inputs_and_unconfirmed_declarations_are_saved_as_incomplete() {
    let mut value = with_tasks();
    let task = &mut value["engineering"]["tasks"][0];
    task["inputs"] = json!({});
    task.as_object_mut().unwrap().remove("axialSections");
    for field in [
        "orientation",
        "gasDirection",
        "service",
        "geometry",
        "kBasis",
        "flowBasis",
    ] {
        task["case"]["applicability"][field] = Value::Null;
    }
    assert_eq!(
        serde_json::from_str::<Value>(
            &to_pretty_json(&parse(&value.to_string()).unwrap()).unwrap()
        )
        .unwrap(),
        value
    );
}

#[test]
fn strict_task_structure_and_graph_validation_reject_lossy_inputs() {
    for mutation in 0..13 {
        let mut value = with_tasks();
        let task = &mut value["engineering"]["tasks"][0];
        match mutation {
            0 => {
                task["inputs"]["workingDuration"] = Value::Null;
            }
            1 => {
                task["inputs"]["workingDuration"]["basisEntryId"] = json!("missing");
            }
            2 => {
                task["inputs"]["gasActualVolumeFlow"]["sourceId"] = json!("missing");
            }
            3 => {
                task["inputs"]["gasActualVolumeFlow"]["quantityId"] = json!("length");
            }
            4 => {
                task["inputs"]["typo"] = json!({});
            }
            5 => {
                task["case"]["applicability"]
                    .as_object_mut()
                    .unwrap()
                    .remove("service");
            }
            6 => {
                task["case"]["applicability"]["flowBasis"] = json!("typo");
            }
            7 => {
                task["method"]["typo"] = json!(true);
            }
            8 => {
                task["case"]["applicabilitySourceIds"] = json!(["missing"]);
            }
            9 => {
                task["inputs"]["workingDuration"] = json!({"kind":"override","basisEntryId":"basis-time","quantityId":"duration","valueSi":1,"sourceId":"source-test","reason":""});
            }
            10..=12 => {
                let mut duplicate = task.clone();
                duplicate["id"] = json!("another");
                if mutation != 10 {
                    duplicate["case"]["id"] = json!("another-case");
                }
                if mutation == 12 {
                    duplicate["id"] = json!("task-1");
                    duplicate["tag"] = json!("other");
                }
                value["engineering"]["tasks"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
            _ => unreachable!(),
        }
        let error = parse(&value.to_string()).unwrap_err();
        assert!(
            error.message().contains("tasks"),
            "mutation {mutation}: {error}"
        );
    }
    let text = with_tasks().to_string();
    for altered in [text.replace("\"inputs\":{", "\"inputs\":{\"workingDuration\":{\"kind\":\"basisRef\",\"basisEntryId\":\"basis-time\"},"),
        text.replace("\"revision\":4", "\"revision\":4,\"revision\":4"),
        text.replace("\"valueSi\":-0.8", "\"valueSi\":-0.8,\"valueSi\":1.0")] {
        assert!(parse(&altered).is_err(),"duplicate key accepted");
    }
}

#[test]
fn unknown_methods_and_legacy_payloads_never_enter_the_known_decoder() {
    let text =
        unsupported_text("future.method", 99).replace("\"future\":", "\"case\":42,\"future\":");
    let ProjectV3Read::ReadOnly(opaque) = parse(&text).unwrap() else {
        panic!("read-only expected");
    };
    assert_eq!(opaque.original_json(), text);
    let mut value = with_tasks();
    value["engineering"]["tasks"][0]["case"]["configuration"] = json!("unconfirmed");
    let text = value.to_string();
    let ProjectV3Read::ReadOnly(opaque) = parse(&text).unwrap() else {
        panic!("legacy shape must not infer declarations");
    };
    assert_eq!(opaque.original_json(), text);
}

#[test]
fn known_equipment_writes_upgrade_only_with_explicit_consent_and_protect_unknown_targets() {
    let root = temp_dir();
    let path = root.join("equipment.rfproj.json");
    let legacy = include_str!("../../../tests/fixtures/project-v2-engineering.json");
    fs::write(&path, legacy).unwrap();
    let project = parse(&with_tasks().to_string()).unwrap();
    assert!(write(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
    write_upgraded(&path, &project, FileOverwritePolicy::Allow).unwrap();
    assert_eq!(read(&path).unwrap(), project);
    let unknown = unsupported_text("future", 99);
    fs::write(&path, &unknown).unwrap();
    assert!(write_upgraded(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), unknown);
    let ProjectV3Read::Editable(mut invalid) = project else {
        unreachable!();
    };
    invalid.engineering.as_mut().unwrap().tasks[0]
        .method
        .version = 999;
    assert!(to_pretty_json(&ProjectV3Read::Editable(invalid)).is_err());
    fs::remove_dir_all(root).unwrap();
}
