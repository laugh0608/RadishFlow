use super::*;
use crate::project_v4 as v4;
use rf_model::equipment::EquipmentTask;

const SUPPORT: EquipmentSupport = EquipmentSupport::ConcentricTubeRating;
const FLOW: v4::FlowPlanSupport = v4::FlowPlanSupport::Mixed;
const TASK: &str = "/engineering/tasks/0";

fn project_v3() -> Value {
    let mut p = fixture();
    p["engineering"]["tasks"] = json!([{
        "kind": "concentricTubeHeatExchangerRating",
        "id": "h3",
        "revision": 17,
        "tag": "E-301",
        "name": "Incomplete rating",
        "method": { "id": "radishflow.heat-exchanger.concentric-tube-rating", "version": 1 },
        "geometry": { "revision": 5, "sourceId": "source-test", "activeLengthM": -1.0 },
        "case": {
            "id": "h3-case",
            "revision": 8,
            "name": "Water case",
            "purpose": "Storage acceptance",
            "hot": { "inlet": { "temperatureK": 303.15 }, "properties": {
                "sourceId": "source-test", "liquidDomain": { "maxPressurePa": 120000 }
            }},
            "cold": { "inlet": {}, "properties": {
                "sourceId": "source-test", "liquidDomain": {}
            }},
            "limits": {}
        }
    }]);
    p
}

fn project_v4(legacy: bool) -> Value {
    let mut p: Value = serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    if !legacy {
        p["engineering"] = project_v3()["engineering"].clone();
        p["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id":v4::CONCENTRIC_TUBE_RATING_CAPABILITY,
                "version":1
            }));
    }
    p
}

fn reject_both(p: &Value, expected: &str) {
    let error = parse_with_equipment_support(&p.to_string(), SUPPORT).unwrap_err();
    assert!(
        error.message().contains(expected),
        "v3: {}",
        error.message()
    );
    let mut outer = project_v4(false);
    outer["engineering"] = p["engineering"].clone();
    let error = v4::parse_with_equipment_support(&outer.to_string(), FLOW, SUPPORT).unwrap_err();
    assert!(
        error.message().contains(expected),
        "v4: {}",
        error.message()
    );
}

#[test]
fn heat_exchanger_h3_strict_shared_inputs() {
    let p = project_v3();
    for (suffix, value, expected) in [
        ("/geometry/activeLengthM", Value::Null, "expected f64"),
        ("/geometry/sourceId", Value::Null, "string"),
        ("/geometry/sourceId", json!("missing"), "geometry.sourceId"),
        (
            "/case/hot/properties/sourceId",
            json!("missing"),
            "case.hot.properties.sourceId",
        ),
        (
            "/case/cold/properties/sourceId",
            json!("missing"),
            "case.cold.properties.sourceId",
        ),
        ("/case/cold/inlet", json!([]), "JSON object"),
        (
            "/case/cold/properties/liquidDomain",
            Value::Null,
            "JSON object",
        ),
        (
            "/case/limits",
            json!({ "maxHotPressureDropPa": "100" }),
            "expected f64",
        ),
        ("/case/limits", json!({ "result": 123 }), "unknown field"),
    ] {
        let mut invalid = p.clone();
        *invalid.pointer_mut(&format!("{TASK}{suffix}")).unwrap() = value;
        reject_both(&invalid, expected);
    }
    let mut invalid = p.clone();
    invalid.pointer_mut(&format!("{TASK}/case")).unwrap()["configuration"] = json!({});
    reject_both(&invalid, "unknown field");
    let mut invalid = p.clone();
    invalid
        .pointer_mut(&format!("{TASK}/case/hot/properties"))
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("sourceId");
    reject_both(&invalid, "sourceId");
    let mut invalid = p.clone();
    invalid["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(p["engineering"]["tasks"][0].clone());
    reject_both(&invalid, "engineering.tasks[1].id: duplicate ID");

    for project in [p, project_v4(false)] {
        let text = project.to_string();
        for (old, new) in [
            (
                r#""activeLengthM":-1.0"#,
                r#""activeLengthM":-1.0,"activeLengthM":1.0"#,
            ),
            (r#""activeLengthM":-1.0"#, r#""activeLengthM":1e999"#),
        ] {
            let invalid = text.replace(old, new);
            assert_ne!(invalid, text);
            if project["schemaVersion"] == VERSION {
                assert!(parse_with_equipment_support(&invalid, SUPPORT).is_err());
            } else {
                assert!(v4::parse_with_equipment_support(&invalid, FLOW, SUPPORT).is_err());
            }
        }
    }
}

#[test]
fn heat_exchanger_h3_nonfinite_values_and_unsupported_writes_are_rejected() {
    let mut p3 = parse_with_equipment_support(&project_v3().to_string(), SUPPORT).unwrap();
    let mut p4 =
        v4::parse_with_equipment_support(&project_v4(false).to_string(), FLOW, SUPPORT).unwrap();
    assert!(to_pretty_json(&p3).is_err());
    assert!(v4::to_pretty_json(&p4).is_err());
    let ProjectV3Read::Editable(v3) = &mut p3 else {
        unreachable!()
    };
    let v4::ProjectV4Read::Editable(v4) = &mut p4 else {
        unreachable!()
    };
    for inputs in [
        v3.engineering.as_mut().unwrap(),
        v4.engineering.as_mut().unwrap(),
    ] {
        let EquipmentTask::ConcentricTubeRating(task) = &mut inputs.tasks[0] else {
            unreachable!()
        };
        task.case.hot.properties.liquid_domain.min_temperature_k = Some(f64::NAN);
    }
    for error in [
        to_pretty_json_with_equipment_support(&p3, SUPPORT).unwrap_err(),
        v4::to_pretty_json_with_equipment_support(&p4, FLOW, SUPPORT).unwrap_err(),
    ] {
        assert!(error.message().contains("case.hot.properties.liquidDomain"));
    }
}

#[test]
fn heat_exchanger_h3_preserves_finite_si_bits_across_both_writers() {
    let mut v3 = parse_with_equipment_support(&project_v3().to_string(), SUPPORT).unwrap();
    let mut v4 =
        v4::parse_with_equipment_support(&project_v4(false).to_string(), FLOW, SUPPORT).unwrap();
    let ProjectV3Read::Editable(p3) = &mut v3 else {
        unreachable!()
    };
    let v4::ProjectV4Read::Editable(p4) = &mut v4 else {
        unreachable!()
    };
    for inputs in [
        p3.engineering.as_mut().unwrap(),
        p4.engineering.as_mut().unwrap(),
    ] {
        let EquipmentTask::ConcentricTubeRating(task) = &mut inputs.tasks[0] else {
            unreachable!()
        };
        task.geometry.inner_tube_inner_diameter_m = Some(0.014_459_999_999_999_985);
        task.case.hot.inlet.pressure_pa = Some(99_999.999_999_999_85);
        task.case.cold.inlet.pressure_pa = Some(f64::MAX);
    }
    let v3_text = to_pretty_json_with_equipment_support(&v3, SUPPORT).unwrap();
    assert_eq!(parse_with_equipment_support(&v3_text, SUPPORT).unwrap(), v3);
    let v4_text = v4::to_pretty_json_with_equipment_support(&v4, FLOW, SUPPORT).unwrap();
    assert_eq!(
        v4::parse_with_equipment_support(&v4_text, FLOW, SUPPORT).unwrap(),
        v4
    );
}

#[test]
fn heat_exchanger_h3_v3_confirms_first_addition_and_protects_opaque_targets() {
    let mut combined: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/project-v3-rating.json"
    ))
    .unwrap();
    combined["engineering"]["tasks"][0]["method"]["version"] = json!(2);
    let old = parse(&combined.to_string()).unwrap();
    combined["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(project_v3()["engineering"]["tasks"][0].clone());
    let project = parse_with_equipment_support(&combined.to_string(), SUPPORT).unwrap();
    let dir = temp_dir();
    let target = dir.join("h3-v3.rfproj.json");
    let copy_path = dir.join("copy.rfproj.json");
    write(&target, &old, FileOverwritePolicy::Forbid).unwrap();
    let before = fs::read(&target).unwrap();
    let error = write_with_equipment_support(
        &target,
        &project,
        FileOverwritePolicy::Allow,
        SUPPORT,
        false,
    )
    .unwrap_err();
    assert!(error.message().contains("requires confirmation"));
    assert_eq!(fs::read(&target).unwrap(), before);
    write_with_equipment_support(&target, &project, FileOverwritePolicy::Allow, SUPPORT, true)
        .unwrap();
    let text = fs::read_to_string(&target).unwrap();
    assert_eq!(
        read_with_equipment_support(&target, SUPPORT).unwrap(),
        project
    );
    let opaque = read(&target).unwrap();
    let ProjectV3Read::ReadOnly(readonly) = &opaque else {
        panic!("expected whole project read-only")
    };
    assert_eq!(readonly.original_json(), text);
    assert_eq!(readonly.task_count(), 2);
    assert!(write(&copy_path, &opaque, FileOverwritePolicy::Forbid).is_err());
    assert!(!copy_path.exists());
    assert!(write(&target, &old, FileOverwritePolicy::Allow).is_err());
    assert!(write_upgraded(&target, &old, FileOverwritePolicy::Allow).is_err());
    let old_v4 = v4::parse(&project_v4(true).to_string()).unwrap();
    assert!(v4::prepare_write(&target, &old_v4, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), text);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn heat_exchanger_h3_v4_confirms_first_addition_and_protects_opaque_targets() {
    let old = v4::parse(&project_v4(true).to_string()).unwrap();
    let project =
        v4::parse_with_equipment_support(&project_v4(false).to_string(), FLOW, SUPPORT).unwrap();
    let dir = temp_dir();
    let target = dir.join("h3-v4.rfproj.json");
    let copy_path = dir.join("copy.rfproj.json");
    v4::write(&target, &old, FileOverwritePolicy::Forbid).unwrap();
    let before = fs::read(&target).unwrap();
    let prepare = || {
        v4::prepare_write_with_equipment_support(
            &target,
            &project,
            FileOverwritePolicy::Allow,
            FLOW,
            SUPPORT,
        )
        .unwrap()
    };
    let review = prepare();
    assert!(review.captures_project(&project).unwrap());
    assert!(review.requires_upgrade());
    assert!(v4::write_prepared(&review, false).is_err());
    assert_eq!(fs::read(&target).unwrap(), before);
    let mut changed = before;
    changed.push(b'\n');
    fs::write(&target, &changed).unwrap();
    assert!(v4::write_prepared(&review, true).is_err());
    assert_eq!(fs::read(&target).unwrap(), changed);
    v4::write_prepared(&prepare(), true).unwrap();
    let text = fs::read_to_string(&target).unwrap();
    let opaque = v4::read(&target).unwrap();
    let v4::ProjectV4Read::ReadOnly(readonly) = &opaque else {
        panic!("expected whole project read-only")
    };
    assert_eq!(readonly.original_json(), text);
    assert!(v4::write(&copy_path, &opaque, FileOverwritePolicy::Forbid).is_err());
    assert!(!copy_path.exists());
    assert!(v4::prepare_write(&target, &old, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), text);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn heat_exchanger_h3_future_method_or_capability_remains_opaque() {
    let mut future = project_v3();
    future["engineering"]["tasks"][0]["method"]["version"] = json!(2);
    future["engineering"]["tasks"][0]["case"] = json!({"futureModel": [1, 2]});
    let raw = future.to_string();
    let ProjectV3Read::ReadOnly(opaque) = parse_with_equipment_support(&raw, SUPPORT).unwrap()
    else {
        panic!("future method must be opaque")
    };
    assert_eq!(opaque.original_json(), raw);
    let mut v4 = project_v4(false);
    v4["engineering"] = future["engineering"].clone();
    let raw = v4.to_string();
    let v4::ProjectV4Read::ReadOnly(opaque) =
        v4::parse_with_equipment_support(&raw, FLOW, SUPPORT).unwrap()
    else {
        panic!("future method must be opaque")
    };
    assert_eq!(opaque.original_json(), raw);
    let mut v4 = project_v4(false);
    let caps = v4["requiredCapabilities"].as_array_mut().unwrap();
    caps.last_mut().unwrap()["version"] = json!(2);
    assert!(matches!(
        v4::parse_with_equipment_support(&v4.to_string(), FLOW, SUPPORT).unwrap(),
        v4::ProjectV4Read::ReadOnly(_)
    ));
    v4["requiredCapabilities"].as_array_mut().unwrap().pop();
    let error = v4::parse_with_equipment_support(&v4.to_string(), FLOW, SUPPORT).unwrap_err();
    assert!(
        error
            .message()
            .contains("capability declarations do not match")
    );
    let mut v4 = project_v4(false);
    v4.as_object_mut().unwrap().remove("engineering");
    let error = v4::parse_with_equipment_support(&v4.to_string(), FLOW, SUPPORT).unwrap_err();
    assert!(
        error
            .message()
            .contains("capability declarations do not match")
    );
}
