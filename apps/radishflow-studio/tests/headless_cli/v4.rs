//! Real executable checks: no GUI/display and every byte/directory entry retained.
use super::*;
const RECYCLE: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
);
fn request(f: &Fixture) -> Value {
    fs::write(f.root.join("recycle.json"), RECYCLE).unwrap();
    fs::write(
        f.root.join("recycle.json.rfstudio-layout.json"),
        b"opaque sidecar sentinel",
    )
    .unwrap();
    json!({"schema_version":4,"project":"recycle.json","expected_project":{"document_id":"example-qs1-fixed-tp-recycle","file_sha256":rf_store::sha256_hex(RECYCLE)},"flow_resources":{"cache_root":"cache","auth_cache_index":"index.json","package":{"kind":"preferred"}},"steps":[{"id":"solve","action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":0}}],"queries":[{"id":"plan","query":{"kind":"results","scope":"flow_plan"}},{"id":"result","query":{"kind":"results","scope":"flow"}}]})
}
#[test]
fn v4_executable_returns_verified_recycle_and_preserves_all_disk_bytes() {
    let f = Fixture::new();
    let mut request = request(&f);
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["schema_version"], 4);
    assert_eq!(
        response["project"]["in_memory_required_project_schema_version"],
        4
    );
    let result = &response["queries"][1]["value"];
    assert_eq!(result["status"], "current");
    assert_eq!(result["execution_kind"], "partitioned_sm");
    assert_eq!(result["costs"]["local_unit_calls"], 102);
    assert_eq!(result["costs"]["verification_evaluations"], 1);
    assert_eq!(result["origin"]["execution"]["step"], "solve");
    assert_eq!(result["blocks"][3]["recycle"]["iteration_evaluations"], 48);
    request["steps"].as_array_mut().unwrap().push(json!({"id":"plan","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.acyclic-sm/1"}}}));
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["queries"][1]["value"]["status"], "stale");
    assert!(response["queries"][1]["value"].get("fields").is_none());
    let mut project: Value = serde_json::from_slice(RECYCLE).unwrap();
    project["document"]["flowsheet"]["units"]["splitter-1"]["parameters"]["outlet_a_split_fraction"] =
        json!(0.99);
    let bytes = serde_json::to_vec(&project).unwrap();
    fs::write(f.root.join("recycle.json"), &bytes).unwrap();
    request["expected_project"]["file_sha256"] = json!(rf_store::sha256_hex(&bytes));
    request["steps"].as_array_mut().unwrap().truncate(1);
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 4, "{response}");
    let result = &response["queries"][1]["value"];
    assert_eq!(
        result["diagnostic_evidence"]["failure_kind"],
        "iteration_limit"
    );
    assert_eq!(
        result["diagnostic_evidence"]["costs"]["iteration_evaluations"],
        500
    );
    assert!(result.get("fields").is_none());
}
#[test]
fn v4_and_historical_protocols_reject_before_resources_and_unknown_v4_stays_readonly() {
    let f = Fixture::new();
    let mut request = request(&f);
    request["steps"][0]["action"]["expected_plan_revision"] = json!(1);
    request["flow_resources"]["auth_cache_index"] = json!("does-not-exist");
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["code"], "record_conflict");
    assert_eq!(response["error"]["details"]["owner"], "plan");
    request["schema_version"] = json!(3);
    request["steps"][0]["action"]
        .as_object_mut()
        .unwrap()
        .remove("expected_plan_revision");
    request["queries"] = json!([]);
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["code"], "unsupported_capability");
    assert!(response["steps"].as_array().unwrap().is_empty());
    let (code, _) = f.command("inspect", &f.root.join("recycle.json"));
    assert_eq!(code, 2);
    for text in [
        r#"{"schema_version":4,"project":"missing","queries":[{"id":"q","query":{"kind":"results","scope":"flow_plan","unknown":1}}]}"#,
        r#"{"schema_version":4,"project":"missing","queries":[],"queries":[]}"#,
        r#"{"schema_version":3,"project":"missing","queries":[{"id":"q","query":{"kind":"results","scope":"flow_plan"}}]}"#,
    ] {
        let (code, response) = f.v3_bytes("query", text.as_bytes());
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(response["project"].is_null());
    }
    let mut project: Value = serde_json::from_slice(RECYCLE).unwrap();
    project["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"unknown-capability","version":1}));
    fs::write(
        f.root.join("recycle.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let (code,response)=f.v3("query",&json!({"schema_version":4,"project":"recycle.json","queries":[{"id":"p","query":{"kind":"results","scope":"flow_plan"}}]}));
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["project"]["read_only"], true);
    assert!(
        response["queries"][0]["value"]
            .get("configuration")
            .is_none()
    );
}
