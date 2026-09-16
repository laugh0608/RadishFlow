use super::*;

impl Fixture {
    fn workflow_request(&self) -> Value {
        fs::write(
            self.root.join("basis.json"),
            include_bytes!("../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json"),
        )
        .unwrap();
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../../examples/automation/build-feed-heater-flash.request.json"
        ))
        .unwrap();
        request["project"] = json!("basis.json");
        request["auth_cache_index"] = json!("index.json");
        request
    }
}
fn step(id: &str, action: Value) -> Value {
    json!({"id":id, "action":action})
}
fn create(id: &str, kind: &str) -> Value {
    step(id, json!({"kind":"create_unit", "unit_kind":kind}))
}
fn connect(id: &str, source: Value, sink: Value) -> Value {
    step(id, json!({"kind":"connect", "source":source, "sink":sink}))
}
fn port(unit: &str, name: &str) -> Value {
    json!({"unit":{"step":unit}, "port":name})
}
fn write(id: &str, object: Value, field: Value, value: Value) -> Value {
    step(
        id,
        json!({"kind":"write", "target":{"object":object,"section":"inputs","field":field},"value":value}),
    )
}

#[test]
fn headless_v2_builds_with_returned_ids_and_repeats_without_disk_changes() {
    let f = Fixture::new();
    let request = f.workflow_request();
    let project = fs::read(f.root.join("basis.json")).unwrap();
    let index = fs::read(f.root.join("index.json")).unwrap();
    let (code, result) = f.run(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["schema_version"], 2);
    assert_eq!(
        result["steps"].as_array().unwrap().len(),
        request["steps"].as_array().unwrap().len()
    );
    assert_eq!(result["writes"], json!([]));
    let receipts = result["steps"].as_array().unwrap();
    for (i, receipt) in receipts.iter().enumerate() {
        assert_eq!(receipt["index"], i);
        assert_eq!(receipt["id"], request["steps"][i]["id"]);
        if i > 0 {
            assert!(receipt["revision"].as_u64() >= receipts[i - 1]["revision"].as_u64());
        }
    }
    assert_eq!(
        receipts[1]["effect"]["source"]["unit_id"],
        receipts[0]["effect"]["unit_id"]
    );
    assert_eq!(
        receipts[8]["effect"]["stream_id"],
        receipts[1]["effect"]["stream_id"]
    );
    assert_eq!(
        result["variables"][0]["target"]["object"]["stream"],
        receipts[9]["effect"]["stream_id"]
    );
    assert_eq!(result["variables"][0]["value"], 330.0);
    assert_eq!(result["variables"][1]["value"], 90000.0);
    assert_eq!(result["variables"][2]["value"], 5.0);
    let vapor = result["variables"][4]["value"].as_f64().unwrap();
    let liquid = result["variables"][3]["value"].as_f64().unwrap();
    assert!((vapor + liquid - 5.0).abs() < 1e-10);
    for row in result["variables"].as_array().unwrap() {
        assert_eq!(row["source"]["revision"], result["document"]["revision"]);
        assert_eq!(row["state"], "valid");
    }
    assert_eq!(f.run(&request), (code, result));
    assert_eq!(fs::read(f.root.join("basis.json")).unwrap(), project);
    assert_eq!(fs::read(f.root.join("index.json")).unwrap(), index);
    assert!(!f.root.join("basis.json.rfstudio-layout.json").exists());
}

#[test]
fn headless_v2_stops_at_reference_and_identity_errors() {
    let f = Fixture::new();
    let mut request = f.workflow_request();
    let original = fs::read(f.root.join("basis.json")).unwrap();
    let cases = [
        (create("feed", "heater"), "duplicate_step_id"),
        (create("  ", "heater"), "empty_step_id"),
        (
            connect("c", port("future", "outlet"), Value::Null),
            "unknown_step_reference",
        ),
        (
            write(
                "w",
                json!({"stream":{"step":"feed"}}),
                json!("molar_flow"),
                json!(1),
            ),
            "reference_kind_mismatch",
        ),
        (
            write(
                "w",
                json!({"unit":{"id":"absent"}}),
                json!("name"),
                json!("missing"),
            ),
            "object_missing",
        ),
        (
            connect("c", port("feed", "not-a-port"), Value::Null),
            "connection_unavailable",
        ),
    ];
    for (invalid, expected) in cases {
        request["steps"] = json!([
            create("feed", "feed"),
            invalid,
            create("unreached", "heater")
        ]);
        let (code, result) = f.run(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], expected);
        assert_eq!(result["error"]["stage"], "step");
        assert_eq!(result["error"]["index"], 1);
        assert_eq!(result["steps"].as_array().unwrap().len(), 1);
        assert_eq!(result["document"]["revision"], 1);
        assert_eq!(result["variables"], json!([]));
        assert!(result["diagnostic"].is_null());
        assert!(result["package_id"].is_null());
        assert_eq!(fs::read(f.root.join("basis.json")).unwrap(), original);
    }
}

#[test]
fn headless_v2_rechecks_candidates_and_does_not_invent_ambiguous_topology() {
    let f = Fixture::new();
    let mut request = f.workflow_request();
    for (steps, failed_index) in [
        (
            json!([
                create("feed", "feed"),
                connect("stream", port("feed", "outlet"), Value::Null),
                connect("again", port("feed", "outlet"), Value::Null)
            ]),
            2,
        ),
        (
            json!([
                create("a", "feed"),
                connect("as", port("a", "outlet"), Value::Null),
                create("b", "feed"),
                connect("bs", port("b", "outlet"), Value::Null),
                create("heater", "heater"),
                connect("ambiguous", port("a", "outlet"), port("heater", "inlet"))
            ]),
            5,
        ),
    ] {
        request["steps"] = steps;
        let (code, result) = f.run(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], "connection_unavailable");
        assert_eq!(result["error"]["index"], failed_index);
        assert_eq!(result["steps"].as_array().unwrap().len(), failed_index);
    }
}

#[test]
fn headless_v2_write_receipts_preserve_noop_and_failure_semantics() {
    let f = Fixture::new();
    let mut request = f.workflow_request();
    request["steps"] = json!([
        create("feed", "feed"),
        connect("stream", port("feed", "outlet"), Value::Null),
        write(
            "set",
            json!({"unit":{"step":"feed"}}),
            json!("outlet_temperature"),
            json!(310.0)
        ),
        write(
            "same",
            json!({"unit":{"step":"feed"}}),
            json!("outlet_temperature"),
            json!(310.0)
        ),
        write(
            "bad",
            json!({"unit":{"step":"feed"}}),
            json!("outlet_temperature"),
            json!(-1.0)
        ),
        create("unreached", "heater")
    ]);
    let (code, result) = f.run(&request);
    assert_eq!(code, 2, "{result}");
    assert_eq!(result["error"]["index"], 4);
    assert_eq!(result["steps"].as_array().unwrap().len(), 4);
    assert_eq!(result["steps"][2]["effect"]["changed"], true);
    assert_eq!(result["steps"][3]["effect"]["changed"], false);
    assert_eq!(
        result["steps"][2]["revision"],
        result["steps"][3]["revision"]
    );
    assert_eq!(
        result["document"]["revision"],
        result["steps"][3]["revision"]
    );
}

#[test]
fn headless_v2_accepts_existing_ids_and_rejects_unknown_read_alias_without_partial_values() {
    let f = Fixture::new();
    let mut v1 = f.request();
    let write_target = v1["writes"][0]["target"].clone();
    let unit_id = write_target["object"]["unit"].clone();
    v1["schema_version"] = json!(2);
    v1.as_object_mut().unwrap().remove("writes");
    v1["steps"] = json!([write(
        "t",
        json!({"unit":{"id":unit_id}}),
        json!("outlet_temperature"),
        json!(330.0)
    )]);
    for read in v1["reads"].as_array_mut().unwrap() {
        let id = read["object"]["stream"].clone();
        read["object"] = json!({"stream":{"id":id}});
    }
    assert_eq!(f.run(&v1).0, 0);
    v1["reads"][2]["object"] = json!({"stream":{"step":"unknown"}});
    let (code, result) = f.run(&v1);
    assert_eq!(code, 2, "{result}");
    assert_eq!(result["error"]["stage"], "read");
    assert_eq!(result["error"]["index"], 2);
    assert_eq!(result["error"]["code"], "unknown_step_reference");
    assert_eq!(result["variables"], json!([]));
    assert_eq!(result["steps"].as_array().unwrap().len(), 1);
}

#[test]
fn headless_v2_reports_run_failures_after_successful_steps() {
    let f = Fixture::new();
    let mut request = f.workflow_request();
    request["steps"] = json!([create("feed", "feed")]);
    let (code, result) = f.run(&request);
    assert!(code == 3 || code == 4, "{result}");
    assert_eq!(result["error"]["stage"], "run");
    assert_eq!(result["steps"].as_array().unwrap().len(), 1);
    assert_eq!(result["variables"], json!([]));
    let request = f.workflow_request();
    let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
    let manifest = index.property_packages[0].manifest_path_under(f.root.join("cache"));
    fs::remove_file(&manifest).unwrap();
    let (code, result) = f.run(&request);
    assert_eq!(code, 4, "{result}");
    assert_eq!(result["error"]["code"], "local_cache_unavailable");
    assert_eq!(
        result["steps"].as_array().unwrap().len(),
        request["steps"].as_array().unwrap().len()
    );
    assert!(!manifest.exists());
}

#[test]
fn headless_v2_rejects_document_conflicts_before_applying_steps() {
    let f = Fixture::new();
    let base = f.workflow_request();
    for (key, value, expected) in [
        ("document_id", json!("foreign"), "different_document"),
        ("expected_revision", json!(1), "revision_conflict"),
    ] {
        let mut request = base.clone();
        request[key] = value;
        let (code, result) = f.run(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], expected);
        assert_eq!(result["steps"], json!([]));
        assert_eq!(result["document"]["revision"], 0);
    }
}

#[test]
fn headless_v2_never_binds_a_write_as_an_object_or_writes_results() {
    let f = Fixture::new();
    let mut request = f.workflow_request();
    let mut result_write = write(
        "invalid",
        json!({"unit":{"step":"feed"}}),
        json!("outlet_temperature"),
        json!(330.0),
    );
    result_write["action"]["target"]["section"] = json!("results");
    for (invalid, expected) in [
        (
            connect("invalid", port("rename", "outlet"), Value::Null),
            "reference_kind_mismatch",
        ),
        (result_write, "read_only"),
    ] {
        request["steps"] = json!([
            create("feed", "feed"),
            connect("outlet", port("feed", "outlet"), Value::Null),
            write(
                "rename",
                json!({"unit":{"step":"feed"}}),
                json!("name"),
                json!("Renamed feed")
            ),
            invalid
        ]);
        let (code, result) = f.run(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], expected);
        assert_eq!(result["error"]["index"], 3);
        assert_eq!(result["steps"].as_array().unwrap().len(), 3);
    }
}
