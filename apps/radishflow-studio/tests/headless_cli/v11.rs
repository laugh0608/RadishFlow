use super::*;

const PROJECT: &str =
    include_str!("../../../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json");
const NAME: &str = "single-vapor.rfproj.json";

fn request(bytes: &[u8]) -> (Value, Value) {
    let query = json!({"schema_version": 11, "project": NAME, "queries": [
        {"id": "plan", "query": {"kind": "results", "scope": "flow_plan"}},
        {"id": "result", "query": {"kind": "results", "scope": "flow"}}
    ]});
    let mut run = query.clone();
    run["expected_project"] = json!({"document_id":"example-two-feed-single-vapor", "file_sha256": rf_store::sha256_hex(bytes)});
    run["steps"] = json!([{"id": "solve", "action":{"kind":"run_flow", "expected_revision": 0, "expected_plan_revision": 0}}]);
    (query, run)
}

#[test]
fn v11_process_reopens_and_runs_both_thermal_specifications_with_no_cache() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    for (kind, target_k, duty_w) in [("heater", 380.0, 7800.0), ("cooler", 320.0, -4800.0)] {
        for inverse in [false, true] {
            let mut project: Value = serde_json::from_str(PROJECT).unwrap();
            let units = project["document"]["flowsheet"]["units"]
                .as_object_mut()
                .unwrap();
            let mut thermal = units.remove("heater").unwrap();
            thermal["id"] = json!(kind);
            thermal["kind"] = json!(kind);
            if inverse {
                thermal["parameters"]
                    .as_object_mut()
                    .unwrap()
                    .remove("outlet_temperature_k");
                project["simulation"]["steadyStatePlan"]["singlePhase"]["heatDutiesW"][kind] =
                    json!(duty_w);
            } else {
                thermal["parameters"]["outlet_temperature_k"] = json!(target_k);
            }
            project["document"]["flowsheet"]["units"][kind] = thermal;
            let bytes = serde_json::to_vec_pretty(&project).unwrap();
            fs::write(&path, &bytes).unwrap();
            let (query, run) = request(&bytes);
            let (code, opened) = f.v3("query", &query);
            assert_eq!(code, 0, "{opened}");
            let plan = &opened["queries"][0]["value"];
            assert_eq!(plan["ready"], true, "{opened}");
            assert_eq!(opened["queries"][1]["value"]["status"], "missing");
            let (code, response) = f.v3("execute", &run);
            assert_eq!(code, 0, "{response}");
            let result = &response["queries"][1]["value"];
            assert_eq!(response["steps"][0]["result"], *result);
            assert_eq!(result["status"], "current");
            assert_eq!(
                result["origin"]["execution"]["plan"]["configuration_sha256"],
                plan["configuration_sha256"]
            );
            assert!(result["origin"]["execution"]["package"].is_null());
            let product = result["streams"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == "product")
                .unwrap();
            assert!((product["temperature_k"].as_f64().unwrap() - target_k).abs() <= 1e-8);
            let energy = result["energy_balances"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["unit_id"] == kind)
                .unwrap();
            assert!((energy["heat_duty_w"].as_f64().unwrap() - duty_w).abs() <= 1e-6);
            assert!(
                (energy["outlet_enthalpy_flow_w"].as_f64().unwrap() - 9388.5 - duty_w).abs()
                    <= 1e-6
            );
            assert!(energy["residual_w"].as_f64().unwrap().abs() <= 1e-6);
            assert!(energy["scaled_residual"].as_f64().unwrap().abs() <= 1e-10);
            assert_eq!(f.v3("query", &query), (0, opened));
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    assert!(!f.root.join("cache").exists());
}

#[test]
fn v11_process_enforces_revision_and_old_protocol_protects_complete_file() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    fs::write(&path, PROJECT).unwrap();
    let (mut query, mut run) = request(PROJECT.as_bytes());
    run["steps"][0]["action"]["expected_plan_revision"] = json!(1);
    let (code, stale) = f.v3("execute", &run);
    assert_ne!(code, 0, "{stale}");
    assert_eq!(stale["error"]["step_id"], "solve");

    query["schema_version"] = json!(10);
    let (code, old) = f.v3("query", &query);
    assert_eq!(code, 0, "{old}");
    assert_eq!(old["project"]["read_only"], true);
    run["schema_version"] = json!(10);
    run["flow_resources"] = json!({"cache_root":"missing-cache","auth_cache_index":"missing-index.json","package":{"kind":"preferred"}});
    let (code, old) = f.v3("execute", &run);
    assert_eq!(code, 3, "{old}");
    assert_eq!(old["error"]["code"], "read_only_project");
    assert_eq!(old["steps"].as_array().unwrap().len(), 0);
    assert_eq!(fs::read_to_string(path).unwrap(), PROJECT);
}
