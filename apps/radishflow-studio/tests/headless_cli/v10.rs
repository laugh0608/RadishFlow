use super::*;

const PROJECT: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-product-composition-binary-hydrocarbon.rfproj.json"
);
const QUERY: &str =
    include_str!("../../../../examples/automation/query-product-composition-v10.request.json");
const WORKFLOW: &str = include_str!(
    "../../../../examples/automation/adopt-product-composition-v10.request.template.json"
);
const COPY: &str = "composition.rfproj.json";

fn fixture() -> (Fixture, Value, Value) {
    let f = Fixture::new();
    fs::write(f.root.join(COPY), PROJECT).unwrap();
    fs::write(
        f.root.join(format!("{COPY}.rfstudio-layout.json")),
        b"layout sentinel",
    )
    .unwrap();
    let mut query: Value = serde_json::from_str(QUERY).unwrap();
    query["project"] = json!(COPY);
    let mut run: Value = serde_json::from_str(WORKFLOW).unwrap();
    run["project"] = json!(COPY);
    run["flow_resources"]["cache_root"] = json!("cache");
    run["flow_resources"]["auth_cache_index"] = json!("index.json");
    (f, query, run)
}

fn find<'a>(response: &'a Value, records: &str, id: &str) -> &'a Value {
    response[records]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("{records}/{id} missing: {response}"))
}

fn near(value: &Value, expected: f64, tolerance: f64) {
    let actual = value
        .as_f64()
        .unwrap_or_else(|| panic!("expected a number, got {value}"));
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected}"
    );
}

#[test]
fn v10_process_queries_runs_reviews_adopts_and_reruns_forward() {
    let (f, query, run) = fixture();
    let (code, opened) = f.v3("query", &query);
    assert_eq!(code, 0, "{opened}");
    assert_eq!(opened["schema_version"], 10);
    assert_eq!(opened["project"]["read_only"], false);
    assert_eq!(opened["project"]["project_schema_version"], 4);
    assert_eq!(
        opened["project"]["file_sha256"],
        run["expected_project"]["file_sha256"]
    );
    assert_eq!(
        find(&opened, "queries", "result")["value"]["status"],
        "missing"
    );
    let plan = &find(&opened, "queries", "plan")["value"];
    assert_eq!(plan["ready"], true);
    assert_eq!(plan["configuration"]["equation_contract_version"], 2);
    assert_eq!(plan["configuration"]["acceptance_contract_version"], 2);
    assert_eq!(
        plan["configuration"]["specifications"][1],
        json!({
            "kind":"product_mole_fraction","stream":{"id":"s-pb"},
            "source":{"unit":{"id":"splitter-2"},"port":"outlet_b"},
            "component_id":"methane","value_mol_mol":0.35,
        })
    );

    let (code, response) = f.v3("execute", &run);
    assert_eq!(code, 0, "{response}");
    let inverse = &find(&response, "steps", "solve")["result"];
    assert_eq!(inverse["status"], "current");
    assert_eq!(inverse["origin"]["execution"]["step"], "solve");
    assert_eq!(inverse["equation_contract_version"], 2);
    assert_eq!(inverse["acceptance_contract_version"], 2);
    let origin = &inverse["origin"]["execution"]["plan"];
    assert_eq!(origin["capability"]["version"], 2);
    assert_eq!(origin["configuration_sha256"], plan["configuration_sha256"]);
    let evidence = &inverse["evidence"];
    near(&evidence["solved_parameter"]["solved_value"], 0.5, 1e-12);
    assert_eq!(evidence["solved_parameter"]["stored_value"], 0.6);
    let product = &evidence["verified_target"];
    assert_eq!(product["source"]["stream_id"], "s-pb");
    assert_eq!(product["component_id"], "methane");
    near(&product["composition"]["total_mol_s"], 24.0 / 7.0, 1e-11);
    near(&product["composition"]["component_mol_s"], 1.2, 1e-11);
    near(&product["composition"]["actual_mol_mol"], 0.35, 1e-10);
    near(&product["composition"]["residual_mol_mol"], 0.0, 1e-10);
    near(&product["residual_mol_s"], 0.0, 1e-10);

    let adoption = find(&response, "steps", "adopt");
    assert_eq!(adoption["source_execution"], "solve");
    assert_eq!(adoption["before_revision"], 0);
    assert_eq!(adoption["after_revision"], 1);
    assert_eq!(adoption["plan_revision"], 0);
    assert_eq!(adoption["result_status"], "stale");
    assert_eq!(adoption["persisted"], false);
    near(&adoption["adopted_value"], 0.5, 1e-12);
    assert_eq!(find(&response, "steps", "forward")["plan_revision"], 1);

    let forward = &find(&response, "queries", "result")["value"];
    assert_eq!(forward["status"], "current");
    assert_eq!(forward["origin"]["execution"]["step"], "rerun");
    assert_eq!(forward["equation_contract_version"], 1);
    assert!(forward["evidence"]["solved_parameter"].is_null());
    let stream = forward["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "s-pb")
        .unwrap();
    let methane = stream["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["component_id"] == "methane")
        .unwrap();
    near(&methane["mole_fraction"], 0.35, 1e-10);

    let (code, reopened) = f.v3("query", &query);
    assert_eq!(code, 0, "{reopened}");
    assert_eq!(reopened, opened);
    assert_eq!(fs::read(f.root.join(COPY)).unwrap(), PROJECT);
}

#[test]
fn v10_process_checks_snapshot_revisions_and_rejects_cross_request_adoption() {
    let (f, _, run) = fixture();
    let steps = run["steps"].as_array().unwrap();
    let mut prior_run = run.clone();
    prior_run["steps"] = json!([steps[0]]);
    let (code, result) = f.v3("execute", &prior_run);
    assert_eq!(code, 0, "{result}");
    assert_eq!(
        find(&result, "queries", "result")["value"]["status"],
        "current"
    );

    let mut foreign = run.clone();
    foreign.as_object_mut().unwrap().remove("flow_resources");
    foreign["steps"] = json!([steps[1]]);
    let (code, response) = f.v3("execute", &foreign);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["step_id"], "adopt");
    assert_eq!(response["error"]["code"], "invalid_request");
    assert_eq!(
        find(&response, "queries", "result")["value"]["status"],
        "missing"
    );

    let mut stale_input = run.clone();
    let mut stale_adoption = steps[1].clone();
    stale_adoption["id"] = json!("retry");
    stale_input["steps"] = json!([steps[0], steps[1], stale_adoption]);
    let (code, response) = f.v3("execute", &stale_input);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["step_id"], "retry");
    assert_eq!(response["error"]["code"], "record_conflict");

    let mut stale_plan = run.clone();
    stale_plan["steps"] = json!([steps[2], steps[0]]);
    let (code, response) = f.v3("execute", &stale_plan);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["step_id"], "solve");
    assert_eq!(response["error"]["code"], "record_conflict");
    assert_eq!(response["error"]["details"]["owner"], "plan");

    for (field, value, error) in [
        ("document_id", "foreign-document", "different_document"),
        ("file_sha256", "stale-bytes", "project_conflict"),
    ] {
        let mut mismatched = run.clone();
        mismatched["expected_project"][field] = json!(value);
        let (code, response) = f.v3("execute", &mismatched);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["code"], error);
        assert_eq!(response["steps"], json!([]));
    }
}

#[test]
fn v10_process_accepts_round_tripped_configuration_and_older_protocols_decline_it() {
    let (f, mut query, run) = fixture();
    let (code, opened) = f.v3("query", &query);
    assert_eq!(code, 0, "{opened}");
    let configuration = &find(&opened, "queries", "plan")["value"]["configuration"];
    let mut configure = run.clone();
    configure.as_object_mut().unwrap().remove("flow_resources");
    configure["steps"] = json!([{
        "id":"configure","action":{"kind":"configure_flow_plan","expected_plan_revision":0,
        "plan":configuration}
    }]);
    let (code, response) = f.v3("execute", &configure);
    assert_eq!(code, 0, "{response}");
    assert_eq!(
        find(&response, "steps", "configure")["outcome"],
        "unchanged"
    );

    for version in [1, 2, 3, 4, 5, 6, 7, 8, 9] {
        query["schema_version"] = json!(version);
        query["queries"] = json!([]);
        let (code, response) = f.v3("query", &query);
        match version {
            1 | 2 => assert_eq!(response["error"]["code"], "unsupported_version"),
            3 => assert_eq!(response["error"]["code"], "unsupported_capability"),
            _ => {
                assert_eq!(code, 0, "{response}");
                assert_eq!(response["project"]["read_only"], true);
            }
        }
    }
    for version in [4, 5, 6, 7, 9] {
        let mut old = run.clone();
        old["schema_version"] = json!(version);
        old["steps"] = json!([run["steps"][0]]);
        old["flow_resources"]["auth_cache_index"] = json!("not-present.json");
        let (code, response) = f.v3("execute", &old);
        assert_eq!(code, 3, "{response}");
        assert_eq!(response["error"]["code"], "read_only_project");
        assert_eq!(response["steps"], json!([]));
    }
    configure["schema_version"] = json!(6);
    let (code, response) = f.v3("execute", &configure);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["code"], "invalid_request");
    assert!(response["project"].is_null());
}
