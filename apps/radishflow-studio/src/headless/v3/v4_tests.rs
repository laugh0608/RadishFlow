//! Cross-version CLI checks exercise the public one-shot dispatcher and disk boundary.
use super::*;
fn request(f: &Fixture, project: &Value, steps: Value) -> Value {
    let path = f.write("project.json", project);
    json!({"schema_version":4,"project":"project.json","expected_project":{"document_id":project["document"]["metadata"]["documentId"],"file_sha256":rf_store::sha256_hex(&fs::read(path).unwrap())},"steps":steps,"queries":[]})
}
fn recycle_project(f: &Fixture, alpha: f64) -> Value {
    let mut app = crate::qs1_test_support::recycle_case(alpha);
    crate::dispatch_document_lifecycle(
        &mut app,
        crate::StudioDocumentLifecycleCommand::SaveUpgraded {
            path: f.0.join("built.json"),
        },
    )
    .unwrap();
    serde_json::from_slice(&fs::read(f.0.join("built.json")).unwrap()).unwrap()
}
fn resources(f: &Fixture) -> Value {
    let mut index = rf_store::StoredAuthCacheIndex::new(
        "https://example.invalid",
        "v4-test",
        rf_store::StoredCredentialReference::new("test", "v4-test"),
    );
    crate::test_support::write_official_binary_hydrocarbon_cached_package(
        &f.0.join("cache"),
        &mut index,
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        std::time::SystemTime::now(),
        None,
    );
    rf_store::write_auth_cache_index(f.0.join("index.json"), &index).unwrap();
    json!({"cache_root":"cache","auth_cache_index":"index.json","package":{"kind":"preferred"}})
}
#[test]
fn v4_real_recycle_keeps_file_identity_and_returns_typed_verified_blocks() {
    let f = Fixture::new();
    let project = recycle_project(&f, 0.6);
    let revision = project["document"]["revision"].as_u64().unwrap();
    let mut r = request(
        &f,
        &project,
        json!([{"id":"solve","action":{"kind":"run_flow","expected_revision":revision,"expected_plan_revision":0}}]),
    );
    r["flow_resources"] = resources(&f);
    r["queries"] = json!([{"id":"plan","query":{"kind":"results","scope":"flow_plan"}},{"id":"result","query":{"kind":"results","scope":"flow"}}]);
    let before = fs::read(f.0.join("project.json")).unwrap();
    let cache_before = fs::read(f.0.join("index.json")).unwrap();
    let (code, result) = f.execute(&r);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["schema_version"], 4);
    assert_eq!(result["project"]["project_schema_version"], 4);
    let flow = &result["queries"][1]["value"];
    assert_eq!(flow["status"], "current");
    assert_eq!(flow["execution_kind"], "partitioned_sm");
    assert_eq!(flow["blocks"].as_array().unwrap().len(), 5);
    assert_eq!(flow["costs"]["local_unit_calls"], 102);
    let recycle = flow["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["kind"] == "recycle")
        .unwrap();
    assert_eq!(recycle["recycle"]["iteration_evaluations"], 48);
    assert_eq!(recycle["recycle"]["verification_evaluations"], 1);
    assert_eq!(
        flow["origin"]["execution"]["plan"]["method"],
        "radishflow.partitioned-sm/1"
    );
    assert!(
        flow["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row.to_string().contains("s-recycle"))
    );
    assert_eq!(fs::read(f.0.join("project.json")).unwrap(), before);
    assert_eq!(fs::read(f.0.join("index.json")).unwrap(), cache_before);
    r["steps"].as_array_mut().unwrap().push(json!({"id":"change","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.acyclic-sm/1"}}}));
    let (code, result) = f.execute(&r);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["queries"][1]["value"]["status"], "stale");
    assert!(result["queries"][1]["value"].get("fields").is_none());
    // The historical protocol refuses this project before the missing cache is touched.
    r["schema_version"] = json!(3);
    r["steps"] = json!([]);
    r.as_object_mut().unwrap().remove("flow_resources");
    r["queries"] = json!([]);
    let (_, result) = f.execute(&r);
    assert_eq!(result["error"]["code"], "unsupported_capability");
    assert!(result["steps"].as_array().unwrap().is_empty());
}
#[test]
fn v4_plan_receipts_conflicts_and_in_memory_version_are_separate_from_file_version() {
    let f = Fixture::new();
    let project = empty_project();
    let mut r = request(
        &f,
        &project,
        json!([
            {"id":"plan","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.partitioned-sm/1","acceptance_contract_version":1,"temperature_k":300,"pressure_pa":650000,"max_iteration_evaluations":500,"tears":[]}}},
            {"id":"split","action":{"kind":"create_unit","expected_revision":0,"unit_kind":"splitter"}},
            {"id":"alpha","action":{"kind":"write","expected_revision":{"step":"split","output":"document"},"target":{"object":{"unit":{"step":"split","output":"unit"}},"section":"inputs","field":"outlet_a_split_fraction"},"value":0.6}},
            {"id":"same","action":{"kind":"configure_flow_plan","expected_plan_revision":{"step":"plan","output":"plan"},"plan":{"method":"radishflow.acyclic-sm/1"}}},
            {"id":"conflict","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.acyclic-sm/1"}}}
        ]),
    );
    r["queries"] = json!([{"id":"plan","query":{"kind":"results","scope":"flow_plan"}}]);
    let (code, result) = f.execute(&r);
    assert_eq!(code, 2, "{result}");
    assert_eq!(result["error"]["step_id"], "conflict");
    assert_eq!(result["project"]["project_schema_version"], 1);
    assert_eq!(
        result["project"]["in_memory_required_project_schema_version"],
        4
    );
    assert_eq!(result["steps"][2]["outcome"], "committed");
    assert_eq!(result["queries"][0]["value"]["plan_revision"], 2);
}
#[test]
fn v4_confirmation_is_versioned_repeatable_and_includes_plan_actions() {
    let f = Fixture::new();
    let mut r = request(
        &f,
        &empty_project(),
        json!([
            {"id":"source","action":{"kind":"create_source"}},
            {"id":"delete","action":{"kind":"delete_equipment_record","target":{"source":{"step":"source","output":"source"}},"expected_revision":{"step":"source","output":"source"}}}
        ]),
    );
    let (code, result) = f.execute(&r);
    assert_eq!(code, 6, "{result}");
    let digest = result["error"]["details"]["confirmation_digest"].clone();
    assert!(
        digest
            .as_str()
            .unwrap()
            .starts_with("headless-v4-confirm-v1:")
    );
    let (_, again) = f.execute(&r);
    assert_eq!(again["error"]["details"]["confirmation_digest"], digest);
    r["steps"][1]["confirmation"] = json!({"digest":digest});
    let (code, result) = f.execute(&r);
    assert_eq!(code, 0, "{result}");
    r["steps"].as_array_mut().unwrap().insert(1,json!({"id":"plan","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.acyclic-sm/1"}}}));
    let (code, result) = f.execute(&r);
    assert_eq!(code, 6, "{result}");
    assert_eq!(result["error"]["code"], "confirmation_mismatch");
}
#[test]
fn v4_failed_run_has_only_diagnostic_evidence_and_unknown_capabilities_are_readonly() {
    let f = Fixture::new();
    let mut p = recycle_project(&f, 0.99);
    let mut r = request(
        &f,
        &p,
        json!([{"id":"solve","action":{"kind":"run_flow","expected_revision":p["document"]["revision"],"expected_plan_revision":0}}]),
    );
    r["flow_resources"] = resources(&f);
    r["queries"] = json!([{"id":"r","query":{"kind":"results","scope":"flow"}}]);
    let (code, result) = f.execute(&r);
    assert_eq!(code, 4, "{result}");
    let result = &result["queries"][0]["value"];
    assert_eq!(result["status"], "failed");
    assert!(result.get("fields").is_none());
    assert_eq!(
        result["diagnostic_evidence"]["failure_kind"],
        "iteration_limit"
    );
    assert_eq!(
        result["diagnostic_evidence"]["costs"]["iteration_evaluations"],
        500
    );
    p["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"future","version":1}));
    f.write("project.json", &p);
    let (code,result)=f.query(json!({"schema_version":4,"project":"project.json","queries":[{"id":"p","query":{"kind":"results","scope":"flow_plan"}}]}));
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["project"]["read_only"], true);
    assert_eq!(result["queries"][0]["value"]["read_only"], true);
    assert!(result["queries"][0]["value"].get("configuration").is_none());
}
