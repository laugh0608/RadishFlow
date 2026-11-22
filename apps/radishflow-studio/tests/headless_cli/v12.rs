use super::*;

const PROJECT: &str =
    include_str!("../../../../examples/flowsheets/two-feed-single-vapor-recycle.rfproj.json");
const NAME: &str = "single-vapor-recycle.rfproj.json";

fn requests(bytes: &[u8]) -> (Value, Value) {
    let query = json!({"schema_version":12, "project":NAME, "queries":[
        {"id":"plan", "query":{"kind":"results","scope":"flow_plan"}},
        {"id":"result", "query":{"kind":"results","scope":"flow"}}
    ]});
    let mut run = query.clone();
    run["expected_project"] = json!({"document_id":"example-two-feed-single-vapor-recycle","file_sha256":rf_store::sha256_hex(bytes)});
    run["steps"] = json!([{"id":"solve", "action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":0}}]);
    (query, run)
}

fn close(value: &Value, expected: f64, tolerance: f64) {
    let actual = value
        .as_f64()
        .unwrap_or_else(|| panic!("expected {expected}, found {value}"));
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, found {actual}"
    );
}

#[test]
fn v12_process_reopens_runs_and_reviews_four_recycle_cases() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    for (kind, target, duty, product_w, recycle_w) in [
        ("heater", 380.0, 7800.0, 17188.5, 25782.75),
        ("cooler", 320.0, -4800.0, 4588.5, 6882.75),
    ] {
        for inverse in [false, true] {
            let mut project: Value = serde_json::from_str(PROJECT).unwrap();
            let flow = &mut project["document"]["flowsheet"];
            let mut unit = flow["units"]
                .as_object_mut()
                .unwrap()
                .remove("heater")
                .unwrap();
            unit["id"] = json!(kind);
            unit["kind"] = json!(kind);
            if inverse {
                unit["parameters"]
                    .as_object_mut()
                    .unwrap()
                    .remove("outlet_temperature_k");
                project["simulation"]["steadyStatePlan"]["singlePhaseRecycle"]["singlePhase"]["heatDutiesW"]
                    [kind] = json!(duty);
            } else {
                unit["parameters"]["outlet_temperature_k"] = json!(target);
            }
            project["document"]["flowsheet"]["units"][kind] = unit;
            let bytes = serde_json::to_vec_pretty(&project).unwrap();
            fs::write(&path, &bytes).unwrap();
            let (query, run) = requests(&bytes);
            let (code, opened) = f.v3("query", &query);
            assert_eq!(code, 0, "{opened}");
            let plan = &opened["queries"][0]["value"];
            assert_eq!(plan["ready"], true, "{opened}");
            assert_eq!(
                plan["configuration"]["single_phase_recycle"]["tear"]["streamId"],
                "recycle"
            );
            assert_eq!(opened["queries"][1]["value"]["status"], "missing");
            let (code, response) = f.v3("execute", &run);
            assert_eq!(code, 0, "{response}");
            let result = &response["queries"][1]["value"];
            assert_eq!(response["steps"][0]["result"], *result);
            assert_eq!(result["status"], "current");
            assert_eq!(result["origin"]["execution"]["step"], "solve");
            assert!(result["origin"]["execution"]["package"].is_null());
            assert_eq!(
                result["origin"]["execution"]["plan"]["configuration_sha256"],
                plan["configuration_sha256"]
            );
            for (id, flow, enthalpy) in [("product", 5.0, product_w), ("recycle", 7.5, recycle_w)] {
                let stream = result["streams"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["id"] == id)
                    .unwrap();
                close(&stream["total_molar_flow_mol_s"], flow, 1e-9);
                close(&stream["temperature_k"], target, 1e-8);
                close(&stream["enthalpy_flow_w"], enthalpy, 1e-6);
            }
            close(&result["overall_energy_balance"]["heat_duty_w"], duty, 1e-6);
            close(&result["overall_energy_balance"]["residual_w"], 0.0, 1e-6);
            let verification = &result["tear"]["verification"];
            assert_eq!(verification["phase"], "verification");
            close(&verification["max_absolute_mol_s"], 0.0, 1e-10);
            close(&verification["scaled_enthalpy_flow_residual"], 0.0, 1e-10);
            assert_eq!(f.v3("query", &query), (0, opened));
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    assert!(!f.root.join("cache").exists());
}

#[test]
fn v12_process_checks_plan_revision_and_v11_protects_project() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    fs::write(&path, PROJECT).unwrap();
    let (mut query, mut run) = requests(PROJECT.as_bytes());
    run["steps"][0]["action"]["expected_plan_revision"] = json!(1);
    let (code, stale) = f.v3("execute", &run);
    assert_ne!(code, 0, "{stale}");
    assert_eq!(stale["error"]["step_id"], "solve");

    let (code, before) = f.v3("query", &query);
    assert_eq!(code, 0, "{before}");
    let mut configuration = before["queries"][0]["value"]["configuration"].clone();
    configuration["single_phase_recycle"]["maxIterationEvaluations"] = json!(120);
    let mut configured = run.clone();
    configured["steps"] = json!([
        {"id":"apply","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":configuration}},
        {"id":"solve","action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":1}}
    ]);
    let (code, executed) = f.v3("execute", &configured);
    assert_eq!(code, 0, "{executed}");
    assert_eq!(executed["steps"][0]["plan_revision"], 1);
    assert_eq!(executed["steps"][1]["result"]["status"], "current");
    assert_eq!(f.v3("query", &query), (0, before));

    query["schema_version"] = json!(11);
    let (code, old) = f.v3("query", &query);
    assert_eq!(code, 0, "{old}");
    assert_eq!(old["project"]["read_only"], true);
    run["schema_version"] = json!(11);
    let (code, old) = f.v3("execute", &run);
    assert_eq!(code, 3, "{old}");
    assert_eq!(old["error"]["code"], "read_only_project");
    assert!(old["steps"].as_array().unwrap().is_empty());
    assert_eq!(fs::read_to_string(path).unwrap(), PROJECT);
}
