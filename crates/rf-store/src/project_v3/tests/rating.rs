use super::*;
use rf_model::equipment::*;

const RATING: &str = include_str!("../../../tests/fixtures/project-v3-rating.json");
fn rating() -> Value {
    serde_json::from_str(RATING).unwrap()
}
fn editable(text: &str) -> StoredProjectV3 {
    let ProjectV3Read::Editable(project) = parse(text).unwrap() else {
        panic!("known rating must be editable")
    };
    project
}

#[test]
fn rating_and_mixed_tasks_round_trip_all_bindings_revisions_and_given_geometry() {
    let mut value = rating();
    value["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(super::equipment::task());
    let parsed = parse(&value.to_string()).unwrap();
    let ProjectV3Read::Editable(project) = &parsed else {
        panic!("mixed tasks must be editable")
    };
    assert!(project.has_rating());
    let tasks = &project.engineering.as_ref().unwrap().tasks;
    let t = tasks[0].as_rating().unwrap();
    assert_eq!(
        (t.revision, t.geometry.revision, t.cases[0].revision),
        (5, 6, 9)
    );
    assert_eq!(t.cases[0].inputs.len(), 9);
    assert_eq!(t.geometry.inputs.len(), 2);
    assert_eq!(t.geometry.axial_sections.len(), 6);
    assert!(tasks[1].as_sizing().is_some());
    let text = to_pretty_json(&parsed).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), value);
    assert_eq!(parse(&text).unwrap(), parsed);
    assert!(!text.contains("loadFraction"));
    assert!(!text.contains("eligibility"));
}

#[test]
fn rating_incomplete_and_outside_scope_records_remain_formal_inputs() {
    let mut value = rating();
    let task = &mut value["engineering"]["tasks"][0];
    task["geometry"]["inputs"] = json!({});
    task["geometry"]
        .as_object_mut()
        .unwrap()
        .remove("axialSections");
    task["geometry"]["applicability"]["orientation"] = Value::Null;
    task["geometry"]["applicability"]["geometry"] = json!("nonCylindrical");
    task["cases"][0]["applicability"]["service"] = json!("threePhase");
    task["cases"][0]["inputs"]["gasActualVolumeFlow"]["valueSi"] = json!(-1.0);
    task["cases"][0]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("gasMassDensity");
    let project = parse(&value.to_string()).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&to_pretty_json(&project).unwrap()).unwrap(),
        value
    );
}

#[test]
fn rating_payload_rejects_wrong_shapes_dangling_bindings_and_sizing_only_fields() {
    for mutation in 0..18 {
        let mut value = rating();
        let t = &mut value["engineering"]["tasks"][0];
        match mutation {
            0 => t["geometry"]["inputs"]["internalDiameter"]["quantityId"] = json!("duration"),
            1 => t["geometry"]["inputs"]["internalDiameter"]["basisEntryId"] = json!("absent"),
            2 => t["geometry"]["inputs"]["workingLiquidHeight"]["sourceId"] = json!("absent"),
            3 => t["geometry"]["inputs"]["internalDiameter"]["reason"] = json!(""),
            4 => t["geometry"]["applicabilitySourceIds"] = json!(["absent"]),
            5 => t["cases"][0]["applicabilitySourceIds"] = json!(["absent"]),
            6 => t["cases"] = json!([]),
            7 => {
                let mut second = t["cases"][0].clone();
                second["id"] = json!("second");
                t["cases"].as_array_mut().unwrap().push(second);
            }
            8 => t["geometry"]["inputs"]["workingLiquidHeight"] = Value::Null,
            9 => {
                t["geometry"]["inputs"]["maximumWorkingLiquidHeight"] =
                    t["geometry"]["inputs"]["workingLiquidHeight"].clone()
            }
            10 => {
                t["cases"][0]["inputs"]["maximumWorkingLiquidHeight"] =
                    t["geometry"]["inputs"]["workingLiquidHeight"].clone()
            }
            11 => {
                t["geometry"]["applicability"]
                    .as_object_mut()
                    .unwrap()
                    .remove("orientation");
            }
            12 => {
                t["cases"][0]["applicability"]
                    .as_object_mut()
                    .unwrap()
                    .remove("service");
            }
            13 => {
                t["geometry"]["axialSections"]["internalsThickness"]["quantityId"] =
                    json!("duration")
            }
            14 => t["cases"][0]["inputs"]["workingDuration"]["basisEntryId"] = json!("absent"),
            15 => t["geometry"]["unexpected"] = json!(true),
            16 => t["cases"][0]["revision"] = json!(-1),
            17 => {
                t["cases"][0]["inputs"]["workingDuration"] = json!({"kind":"direct","quantityId":"length","valueSi":1,"sourceId":"source-test"})
            }
            _ => unreachable!(),
        }
        let error = parse(&value.to_string()).unwrap_err();
        assert!(
            error.message().contains("tasks"),
            "mutation {mutation}: {error}"
        );
    }
}

#[test]
fn rating_duplicate_keys_and_nonfinite_numbers_are_never_lost_in_dispatch() {
    let text = rating().to_string();
    for (from, to) in [
        ("\"revision\":6", "\"revision\":6,\"revision\":6"),
        (
            "\"internalDiameter\":",
            "\"workingLiquidHeight\":{\"kind\":\"direct\",\"quantityId\":\"length\",\"valueSi\":0.1,\"sourceId\":\"drawing\"},\"internalDiameter\":",
        ),
        (
            "\"kind\":\"verticalGasLiquidSeparatorRating\"",
            "\"kind\":\"verticalGasLiquidSeparatorRating\",\"kind\":\"verticalGasLiquidSeparatorRating\"",
        ),
        ("\"valueSi\":0.6", "\"valueSi\":0.6,\"valueSi\":0.9"),
        ("\"valueSi\":0.6", "\"valueSi\":1e400"),
        (
            "\"workingDuration\":",
            "\"gasUtilization\":{\"kind\":\"direct\",\"quantityId\":\"dimensionless_ratio\",\"valueSi\":0.5,\"sourceId\":\"source-test\"},\"workingDuration\":",
        ),
    ] {
        assert!(text.contains(from));
        assert!(
            parse(&text.replace(from, to)).is_err(),
            "accepted duplicate/nonfinite: {from}"
        );
    }
    let mut project = editable(RATING);
    let EquipmentTask::SeparatorRating(t) = &mut project.engineering.as_mut().unwrap().tasks[0]
    else {
        unreachable!()
    };
    if let rf_model::engineering::InputBinding::Direct { value, .. } = t
        .geometry
        .inputs
        .get_mut(&SeparatorGeometryInput::WorkingLiquidHeight)
        .unwrap()
    {
        value.value_si = f64::NAN;
    }
    assert!(to_pretty_json(&ProjectV3Read::Editable(project)).is_err());
}

#[test]
fn mixed_task_id_case_and_tag_collisions_are_rejected() {
    for field in ["task", "case", "tag"] {
        let mut value = rating();
        let mut sizing = super::equipment::task();
        match field {
            "task" => sizing["id"] = json!("rating-1"),
            "case" => sizing["case"]["id"] = json!("rating-case-1"),
            "tag" => sizing["tag"] = json!(" v-102 "),
            _ => unreachable!(),
        }
        value["engineering"]["tasks"]
            .as_array_mut()
            .unwrap()
            .push(sizing);
        assert!(parse(&value.to_string()).is_err(), "{field}");
    }
}

#[test]
fn future_rating_version_and_kind_preserve_original_text_without_partial_decoding() {
    for field in ["version", "kind"] {
        let mut value = rating();
        let t = &mut value["engineering"]["tasks"][0];
        if field == "version" {
            t["method"]["version"] = json!(2);
        } else {
            t["kind"] = json!("futureRating");
        }
        t["cases"] = json!("future opaque payload");
        let text = value
            .to_string()
            .replace("\"geometry\":{", "\"opaqueNumber\":1e400,\"geometry\":{");
        let ProjectV3Read::ReadOnly(project) = parse(&text).unwrap() else {
            panic!("future rating must be read-only")
        };
        assert_eq!(project.original_json(), text);
        assert_eq!(project.task_count(), 1);
        assert!(to_pretty_json(&ProjectV3Read::ReadOnly(project)).is_err());
    }
}

#[test]
fn adding_rating_requires_capability_consent_and_never_overwrites_unknown_targets() {
    let root = temp_dir();
    let path = root.join("rating.rfproj.json");
    let original = fixture().to_string();
    fs::write(&path, &original).unwrap();
    let project = parse(RATING).unwrap();
    assert!(write(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    write_upgraded(&path, &project, FileOverwritePolicy::Allow).unwrap();
    // Already-capable targets do not require repeated capability consent.
    write(&path, &project, FileOverwritePolicy::Allow).unwrap();
    assert_eq!(read(&path).unwrap(), project);
    let mut future = rating();
    future["engineering"]["tasks"][0]["method"]["version"] = json!(2);
    let future = future.to_string();
    fs::write(&path, &future).unwrap();
    assert!(write_upgraded(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), future);
    fs::write(&path, b"damaged").unwrap();
    assert!(write_upgraded(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"damaged");
    fs::remove_dir_all(root).unwrap();
}
