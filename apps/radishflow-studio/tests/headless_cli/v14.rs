use super::*;

const WATER_40: &str =
    include_str!("../../../../examples/flowsheets/h3-linked-water-40.rfproj.json");
const WATER_50: &str =
    include_str!("../../../../examples/flowsheets/h3-linked-water-50.rfproj.json");
const ENGINEERING: &str = include_str!("../fixtures/h3-review-engineering.json");
const PROJECT: &str = "H3 工程.rfproj.json";

fn put_project(f: &Fixture, project: &Value) {
    fs::write(f.root.join(PROJECT), serde_json::to_vec(project).unwrap()).unwrap();
}

fn target(task: &str, case: &str) -> Value {
    json!({"case":{"task":{"id":task},"case":{"id":case}}})
}

fn rated_target() -> Value {
    target("h3-rating", "h3-water")
}

fn query() -> Value {
    json!({"schema_version":14,"project":PROJECT,"queries":[
        {"id":"objects","query":{"kind":"objects"}},
        {"id":"inputs","query":{"kind":"record","target":rated_target()}},
        result_query("result", rated_target()),
    ]})
}

fn result_query(id: &str, target: Value) -> Value {
    json!({"id":id,"query":{"kind":"results","scope":{"object":target}}})
}

fn execution(project: &Value, object: &Value) -> Value {
    json!({"schema_version":14,"project":PROJECT,"expected_project":{
        "document_id":project["document_id"],"file_sha256":project["file_sha256"],
    },"steps":[{"id":"rate","action":{
        "kind":"rate_concentric_tube_from_flow",
        "target":object["target"],"expected_revisions":object["revisions"],
        "qualification":{"id":"h3-pure-water","revision":1},
    }}],"queries":[result_query("result", object["target"].clone())]})
}

fn legacy_execution(run: &Value) -> Value {
    let mut legacy = run.clone();
    legacy["schema_version"] = json!(13);
    let action = &mut legacy["steps"][0]["action"];
    action["kind"] = json!("rate_concentric_tube");
    action.as_object_mut().unwrap().remove("qualification");
    legacy
}

fn discover(f: &Fixture) -> (Value, Value) {
    let (exit, result) = f.v3("query", &query());
    assert_eq!(exit, 0, "{result}");
    let object = result["queries"][0]["value"][0].clone();
    let execute = execution(&result["project"], &object);
    (result, execute)
}

fn close(number: &Value, expected: f64, tolerance: f64) {
    let actual = number
        .as_f64()
        .unwrap_or_else(|| panic!("expected {expected}, got {number}"));
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} vs {expected} within {tolerance}"
    );
}

#[test]
fn v14_process_rates_both_frozen_water_loads_and_preserves_disk() {
    let f = Fixture::empty();
    fs::write(
        f.root.join(format!("{PROJECT}.rfstudio-layout.json")),
        b"existing layout",
    )
    .unwrap();
    fs::create_dir(f.root.join("cache")).unwrap();
    fs::write(f.root.join("cache/untouched"), b"existing cache entry").unwrap();
    for (data, flow, heat, ua, hot, cold, drop, limit) in [
        (
            WATER_40,
            40.0,
            5_521.707_636_437_1,
            874.553_423_114_86,
            297.914_262_742_51,
            294.982_508_040_12,
            23_679.130_040_479,
            "passed",
        ),
        (
            WATER_50,
            50.0,
            5_877.489_522_370_7,
            945.093_778_801_02,
            297.576_906_873_93,
            294.710_466_075_30,
            34_962.547_685_264,
            "failed",
        ),
    ] {
        fs::write(f.root.join(PROJECT), data).unwrap();
        let (found, request) = discover(&f);
        assert_eq!(
            found["project"]["file_sha256"],
            rf_store::sha256_hex(data.as_bytes())
        );
        assert_eq!(found["queries"][2]["value"], json!({"status":"missing"}));
        let inputs = &found["queries"][1]["value"];
        assert_eq!(inputs["property_package_id"], "h3-pure-water-lite-v1");
        let association = &inputs["case"]["flow_association"];
        assert_eq!(association["hot"]["stream_id"], "hot-feed");
        assert_eq!(association["cold"]["stream_id"], "cold-feed");
        assert_ne!(
            association["hot"]["creation_id"],
            association["cold"]["creation_id"]
        );
        assert!(
            inputs["sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["id"] == "nist-water-table-8")
        );

        let (exit, response) = f.v3("execute", &request);
        assert_eq!(exit, 0, "{response}");
        let step = &response["steps"][0];
        assert_eq!(step["outcome"], "completed");
        let value = &step["result"];
        assert_eq!(*value, response["queries"][0]["value"]);
        assert_eq!(value["status"], "current");
        for (key, expected) in [("heat_duty_w", heat), ("ua_w_per_k", ua)] {
            close(&value[key], expected, 1e-6);
        }
        let hot_drop = 9_799.896_961_973_8;
        for (side, temp, dp, molar_flow) in
            [("hot", hot, hot_drop, 14.0), ("cold", cold, drop, flow)]
        {
            close(&value[side]["outlet"]["temperature_k"], temp, 1e-8);
            close(&value[side]["passage"]["pressure_drop_pa"], dp, 1e-4);
            close(&value[side]["outlet"]["pressure_pa"], 100_000.0 - dp, 1e-4);
            assert_eq!(value[side]["outlet"]["molar_flow_mol_s"], molar_flow);
            close(
                &value[side]["sensible_energy_residual_w"],
                0.0,
                1e-6_f64.max(1e-10 * heat),
            );
        }
        for key in [
            "overall_sensible_energy_residual_w",
            "lmtd_heat_duty_residual_w",
        ] {
            close(&value[key], 0.0, 1e-6_f64.max(1e-10 * heat));
        }
        assert_eq!(
            value["checks"],
            json!({"heat_duty":"passed","hot_pressure_drop":"passed","cold_pressure_drop":limit})
        );
        let origin = &value["origin"];
        assert_eq!(
            origin["file_sha256"],
            request["expected_project"]["file_sha256"]
        );
        assert_eq!(
            origin["document_id"],
            request["expected_project"]["document_id"]
        );
        assert_eq!(
            origin["revisions"],
            request["steps"][0]["action"]["expected_revisions"]
        );
        assert_eq!(origin["hot"]["stream"], association["hot"]);
        assert_eq!(origin["cold"]["stream"], association["cold"]);
        assert_eq!(origin["cold"]["inlet"]["molar_flow_mol_s"], flow);
        assert_eq!(origin["flow_run"]["run_id"], "rate");
        assert_eq!(origin["flow_run"]["scope"], "request");
        assert_eq!(
            origin["flow_run"]["flow_revision"],
            found["project"]["flow_revision"]
        );
        assert!(
            !origin["flow_run"]["snapshot_id"]
                .as_str()
                .unwrap()
                .is_empty()
        );
        assert!(origin["flow_run"]["sequence"].as_u64().unwrap() > 0);
        let package = json!({"id":"h3-pure-water-lite-v1","version":"1",
            "payload_fingerprint":"sha256:1d175d0a82c4d6f586219a32c1bdbad46555356e5d2d50eab3e6bd7a0cec0c26"});
        assert_eq!(origin["flow_run"]["package"], package);
        assert_eq!(origin["qualification"]["id"], "h3-pure-water");
        assert_eq!(origin["qualification"]["revision"], 1);
        assert_eq!(
            origin["sources"]["cold_properties"],
            json!({"id":"nist-water-table-8","revision":3})
        );
        assert_eq!(
            f.v3("query", &query()).1["queries"][2]["value"],
            json!({"status":"missing"})
        );

        let mut v13 = query();
        v13["schema_version"] = json!(13);
        let (exit, blocked) = f.v3("query", &v13);
        assert_eq!(exit, 3, "{blocked}");
        assert_eq!(blocked["error"]["code"], "read_only_project");
        assert_eq!(
            blocked["project"]["file_sha256"],
            found["project"]["file_sha256"]
        );
        v13["queries"] = json!([]);
        let (exit, metadata) = f.v3("query", &v13);
        assert_eq!(exit, 0, "{metadata}");
        assert_eq!(metadata["project"]["read_only"], true);
        let (exit, blocked) = f.v3("execute", &legacy_execution(&request));
        assert_eq!(exit, 3, "{blocked}");
        assert_eq!(blocked["error"]["code"], "read_only_project");
        assert_eq!(blocked["steps"], json!([]));
    }
}

#[test]
fn v14_process_exposes_only_associated_cases_and_keeps_other_results_missing() {
    let f = Fixture::empty();
    let mut saved: Value = serde_json::from_str(WATER_40).unwrap();
    let mut other = saved["engineering"]["tasks"][0].clone();
    other["id"] = json!("other-rating");
    other["tag"] = json!("E-303");
    other["case"]["id"] = json!("other-case");
    let independent = serde_json::from_str::<Value>(ENGINEERING).unwrap()["tasks"][1].clone();
    saved["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .extend([other, independent]);
    put_project(&f, &saved);

    let (found, mut run) = discover(&f);
    assert_eq!(found["queries"][0]["value"].as_array().unwrap().len(), 2);
    run["queries"]
        .as_array_mut()
        .unwrap()
        .push(result_query("other", target("other-rating", "other-case")));
    let (exit, result) = f.v3("execute", &run);
    assert_eq!(exit, 0, "{result}");
    assert_eq!(result["queries"][1]["value"], json!({"status":"missing"}));

    run["steps"][0]["action"]["target"] = target("h3-rating-50", "h3-water-50");
    let (exit, rejected) = f.v3("execute", &run);
    assert_eq!(exit, 2, "{rejected}");
    assert_eq!(rejected["error"]["code"], "object_missing");
    assert_eq!(rejected["queries"][0]["value"], json!({"status":"missing"}));
    let (exit, protected) = f.v3("execute", &legacy_execution(&run));
    assert_eq!(exit, 3, "{protected}");
    assert_eq!(protected["error"]["code"], "read_only_project");
    assert_eq!(protected["steps"], json!([]));

    saved["engineering"] = serde_json::from_str(ENGINEERING).unwrap();
    saved["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap["id"] != "radishflow.equipment.concentric-tube-flow-association");
    put_project(&f, &saved);
    let mut objects_only = query();
    objects_only["queries"].as_array_mut().unwrap().truncate(1);
    let (exit, empty) = f.v3("query", &objects_only);
    assert_eq!(exit, 0, "{empty}");
    assert_eq!(empty["queries"][0]["value"], json!([]));

    let independent_v3 =
        include_str!("../../../../examples/flowsheets/standalone-separator-basis.rfproj.json");
    fs::write(f.root.join(PROJECT), independent_v3).unwrap();
    let (exit, unsupported) = f.v3("query", &objects_only);
    assert_eq!(exit, 2, "{unsupported}");
    assert_eq!(unsupported["error"]["code"], "unsupported_capability");
}

#[test]
fn v14_process_guards_project_bytes_case_revisions_and_stream_creation() {
    let f = Fixture::empty();
    let saved: Value = serde_json::from_str(WATER_40).unwrap();
    put_project(&f, &saved);
    let (_, request) = discover(&f);
    let mut wrong = request.clone();
    wrong["expected_project"]["document_id"] = json!("wrong-project");
    assert_eq!(
        f.v3("execute", &wrong).1["error"]["code"],
        "different_document"
    );
    for field in ["task", "geometry", "case"] {
        let mut stale = request.clone();
        stale["steps"][0]["action"]["expected_revisions"][field] = json!(99);
        let (exit, rejected) = f.v3("execute", &stale);
        assert_eq!(exit, 2, "{rejected}");
        assert_eq!(rejected["error"]["code"], "record_conflict");
    }

    let mut changed = saved.clone();
    let replacement = json!("00000000-0000-4000-8000-000000000777");
    changed["objectIdentities"]["streams"][0]["creationId"] = replacement.clone();
    put_project(&f, &changed);
    let (exit, invalid) = f.v3("query", &query());
    assert_eq!(exit, 2, "{invalid}");
    assert_eq!(invalid["error"]["code"], "invalid_project");
    changed["engineering"]["tasks"][0]["case"]["flowAssociation"]["hot"]["creationId"] =
        replacement;
    put_project(&f, &changed);
    let (exit, stale) = f.v3("execute", &request);
    assert_eq!(exit, 2, "{stale}");
    assert_eq!(stale["error"]["code"], "project_conflict");
    assert_eq!(stale["steps"], json!([]));

    changed["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"radishflow.future","version":1}));
    put_project(&f, &changed);
    let (exit, metadata) = f.v3(
        "query",
        &json!({"schema_version":14,"project":PROJECT,"queries":[]}),
    );
    assert_eq!(exit, 0, "{metadata}");
    assert_eq!(
        metadata["project"]["read_only_reason"],
        "unsupported_capability"
    );
    let mut guarded = request;
    guarded["expected_project"]["file_sha256"] = metadata["project"]["file_sha256"].clone();
    let (exit, blocked) = f.v3("execute", &guarded);
    assert_eq!(exit, 3, "{blocked}");
    assert_eq!(blocked["error"]["code"], "read_only_project");
    assert_eq!(blocked["steps"], json!([]));
}

#[test]
fn v14_process_rejects_unqualified_sources_packages_and_stream_states() {
    let f = Fixture::empty();
    let saved: Value = serde_json::from_str(WATER_40).unwrap();
    for (path, replacement, expected, side) in [
        (
            "/document/flowsheet/thermo/property_package_id",
            json!("water-ethanol-lite-v1"),
            "package_unqualified",
            None,
        ),
        (
            "/engineering/sources/0/revision",
            json!(4),
            "property_source_unqualified",
            Some("hot"),
        ),
        (
            "/engineering/sources/0/locator",
            json!("Table 7"),
            "property_source_unqualified",
            Some("hot"),
        ),
        (
            "/engineering/tasks/0/case/cold/properties/densityKgPerM3",
            json!(998.0),
            "properties_unqualified",
            Some("cold"),
        ),
        (
            "/engineering/tasks/0/case/hot/properties/liquidDomain/maxPressurePa",
            json!(121000.0),
            "properties_unqualified",
            Some("hot"),
        ),
        (
            "/document/flowsheet/streams/cold-feed/pressure_pa",
            json!(1000.0),
            "not_single_liquid",
            Some("cold"),
        ),
        (
            "/document/flowsheet/streams/hot-feed/temperature_k",
            json!(305.15),
            "outside_phase_domain",
            Some("hot"),
        ),
    ] {
        let mut project = saved.clone();
        *project.pointer_mut(path).unwrap() = replacement;
        put_project(&f, &project);
        let (_, run) = discover(&f);
        let (exit, blocked) = f.v3("execute", &run);
        assert_eq!(exit, 3, "{path}: {blocked}");
        assert_eq!(blocked["error"]["code"], "action_rejected");
        assert_eq!(blocked["error"]["target"], rated_target());
        let diagnostic = &blocked["error"]["details"]["diagnostic"];
        assert_eq!(
            diagnostic["code"],
            format!("equipment.h3.flow.{expected}"),
            "{blocked}"
        );
        if let Some(side) = side {
            assert_eq!(diagnostic["streams"][0]["side"], side);
            assert_eq!(
                diagnostic["streams"][0]["stream"]["stream_id"],
                format!("{side}-feed")
            );
        }
        assert_eq!(blocked["queries"][0]["value"], json!({"status":"missing"}));
    }
    put_project(&f, &saved);
    let (_, mut run) = discover(&f);
    run["steps"][0]["action"]["qualification"]["revision"] = json!(2);
    let (exit, blocked) = f.v3("execute", &run);
    assert_eq!(exit, 3, "{blocked}");
    assert_eq!(
        blocked["error"]["details"]["diagnostic"]["code"],
        "equipment.h3.flow.qualification_unavailable"
    );

    let mut narrowed = saved;
    narrowed["engineering"]["tasks"][0]["case"]["cold"]["properties"]["liquidDomain"]["minPressurePa"] =
        json!(75_000.0);
    put_project(&f, &narrowed);
    let (_, run) = discover(&f);
    let (exit, result) = f.v3("execute", &run);
    assert_eq!(exit, 0, "{result}");
    assert_eq!(result["queries"][0]["value"]["status"], "current");
}

#[test]
fn v14_process_returns_failed_results_with_flow_and_rating_diagnostics() {
    let f = Fixture::empty();
    let saved: Value = serde_json::from_str(WATER_40).unwrap();
    for (path, replacement, diagnostic_code) in [
        (
            "/document/flowsheet/streams/cold-feed/temperature_k",
            json!(0.0),
            "solver.step.execution",
        ),
        (
            "/engineering/tasks/0/geometry/activeLengthM",
            json!(0.5),
            "unit.heat_exchanger.concentric_tube.outside_correlation_domain",
        ),
        (
            "/engineering/tasks/0/case/cold/properties/liquidDomain/minPressurePa",
            json!(90_000.0),
            "unit.heat_exchanger.concentric_tube.outlet_outside_domain",
        ),
    ] {
        let mut project = saved.clone();
        *project.pointer_mut(path).unwrap() = replacement;
        put_project(&f, &project);
        let (_, run) = discover(&f);
        let (exit, failed) = f.v3("execute", &run);
        assert_eq!(exit, 4, "{path}: {failed}");
        assert_eq!(failed["error"]["code"], "run_failed");
        assert_eq!(failed["steps"][0]["outcome"], "failed");
        let result = &failed["queries"][0]["value"];
        assert_eq!(*result, failed["steps"][0]["result"]);
        assert_eq!(result["status"], "failed");
        assert!(result.get("heat_duty_w").is_none());
        assert_eq!(result["diagnostic"]["code"], diagnostic_code, "{result}");
    }
}

#[test]
fn v14_process_rejects_invalid_requests_before_project_io() {
    let f = Fixture::empty();
    let metadata = json!({"document_id":"fixture","file_sha256":"unread"});
    let object = json!({"target":rated_target(),"revisions":{"task":6,"geometry":7,"case":12}});
    let valid = execution(&metadata, &object);
    let mut unknown = valid.clone();
    unknown["steps"][0]["action"]["qualification"]["phase_reference"] = json!("caller supplied");
    let mut multi = valid.clone();
    let step = multi["steps"][0].clone();
    multi["steps"].as_array_mut().unwrap().push(step);
    let mut legacy = valid.clone();
    legacy["schema_version"] = json!(13);
    legacy["steps"][0]["action"]["kind"] = json!("rate_concentric_tube");
    let duplicate = valid
        .to_string()
        .replace("\"revision\":1", "\"revision\":1,\"revision\":1");
    for bytes in [
        unknown.to_string(),
        multi.to_string(),
        legacy.to_string(),
        duplicate,
    ] {
        let (exit, response) = f.v3_bytes("execute", bytes.as_bytes());
        assert_eq!(exit, 2, "{response}");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(response["project"].is_null());
    }
}
