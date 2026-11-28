use super::*;

const PROJECT: &[u8] =
    include_bytes!("../../../../examples/flowsheets/fixed-tp-q7-serial-recycle.rfproj.json");

fn request(fixture: &Fixture) -> Value {
    fs::write(fixture.root.join("q7.json"), PROJECT).unwrap();
    let mut request: Value = serde_json::from_str(include_str!(
        "../../../../examples/automation/run-q10-material-balance-v16.request.template.json"
    ))
    .unwrap();
    request["project"] = json!("q7.json");
    request["flow_resources"]["cache_root"] = json!("cache");
    request["flow_resources"]["auth_cache_index"] = json!("index.json");
    request
}

#[test]
fn v16_step_and_query_return_the_exact_rejected_balance() {
    let fixture = Fixture::new();
    let mut request = request(&fixture);
    for (feed, scope, inputs, output, inlet, outlet, total_output) in [
        (
            537793.0,
            json!({"kind": "unit", "block_index": 5, "unit_id": "heater-1"}),
            vec!["s-product-2"],
            "s-product",
            430234.79999999993_f64,
            430234.80000000005_f64,
            537795.0,
        ),
        (
            180029.0,
            json!({"kind": "recycle_region", "block_index": 3, "tear_stream_id": "s-recycle"}),
            vec!["s-fresh"],
            "s-bleed",
            144023.6,
            144023.5999999999,
            180030.9999999999,
        ),
        (
            158822.0,
            json!({"kind": "flowsheet"}),
            vec!["s-fa", "s-fb"],
            "s-product",
            127058.0,
            127057.9999999999,
            158823.99999999988,
        ),
    ] {
        request["steps"][0]["action"]["value"] = json!(feed);
        let (code, response) = fixture.v3("execute", &request);
        assert_eq!(code, 4, "{response}");
        assert_eq!(response["schema_version"], 16);
        let result = &response["queries"][0]["value"];
        assert_eq!(response["steps"][1]["result"], *result);
        assert_eq!(result["status"], "failed");
        assert!(result.get("fields").is_none() && result.get("streams").is_none());
        let difference = outlet - inlet;
        let total_input = feed + 2.0;
        assert_eq!(
            result["diagnostic_evidence"]["material_balance"],
            json!({
                "scope": scope, "input_stream_ids": inputs, "output_stream_ids": [output],
                "tolerances": {"component_absolute_flow_mol_s": 1e-10, "scaled_flow": 1e-10,
                    "total_absolute_flow_mol_s": 2e-10},
                "exceeded_components": [{"component_id": "ethane", "input_mol_s": inlet, "output_mol_s": outlet,
                    "difference_mol_s": difference, "scale_mol_s": total_input, "scaled_absolute": difference.abs() / total_input,
                    "absolute_exceeded": true, "scaled_exceeded": false}],
                "total": {"input_mol_s": total_input, "output_mol_s": total_output,
                    "difference_mol_s": total_output - total_input, "exceeded": false},
            })
        );
    }
    for version in [4, 5, 6, 7, 10, 11, 12] {
        request["schema_version"] = json!(version);
        let (code, response) = fixture.v3("execute", &request);
        assert_eq!(code, 4, "{response}");
        let result = &response["queries"][0]["value"];
        assert_eq!(&response["steps"][1]["result"], result);
        assert_eq!(result["diagnostic_evidence"]["failure_kind"], "balance");
        assert!(
            result["diagnostic_evidence"]
                .get("material_balance")
                .is_none()
        );
    }
}

#[test]
fn v16_omits_balance_readings_for_other_result_states() {
    let fixture = Fixture::new();
    let mut request = request(&fixture);
    let query = json!({"schema_version": 16, "project": "q7.json", "queries": request["queries"]});
    let (code, opened) = fixture.v3("query", &query);
    assert_eq!(code, 0, "{opened}");
    let missing = &opened["queries"][0]["value"];
    assert_eq!(missing["status"], "missing");
    assert!(missing.get("diagnostic_evidence").is_none());

    let mut run_then_edit = request.clone();
    run_then_edit["steps"] = json!([request["steps"][1], request["steps"][0]]);
    run_then_edit["steps"][0]["action"]["expected_revision"] = json!(0);
    let (code, response) = fixture.v3("execute", &run_then_edit);
    assert_eq!(code, 0, "{response}");
    let current = &response["steps"][0]["result"];
    assert_eq!(current["status"], "current");
    assert_eq!(current["streams"].as_array().unwrap().len(), 10);
    assert!(current.get("diagnostic_evidence").is_none());
    let stale = &response["queries"][0]["value"];
    assert_eq!(stale["status"], "stale");
    assert!(stale.get("diagnostic_evidence").is_none());

    let action = &mut request["steps"][0]["action"];
    action["target"] = json!({"object": {"unit": {"id":"splitter-1"}}, "section":"inputs", "field":"outlet_a_split_fraction"});
    action["value"] = json!(0.99);
    let (code, response) = fixture.v3("execute", &request);
    assert_eq!(code, 4, "{response}");
    let evidence = &response["queries"][0]["value"]["diagnostic_evidence"];
    assert_eq!(evidence["failure_kind"], "iteration_limit");
    assert!(evidence.get("material_balance").is_none());
}

#[test]
fn v16_total_only_failure_has_no_component_rows() {
    let fixture = Fixture::new();
    let mut request = request(&fixture);
    request["steps"][0]["action"]["value"] = json!(1048646);
    let (code, response) = fixture.v3("execute", &request);
    assert_eq!(code, 4, "{response}");
    let result = &response["steps"][1]["result"];
    assert_eq!(&response["queries"][0]["value"], result);
    let balance = &result["diagnostic_evidence"]["material_balance"];
    assert_eq!(balance["exceeded_components"], json!([]));
    assert_eq!(
        balance["total"],
        json!({
            "input_mol_s": 2055350.0799999998, "output_mol_s": 2055350.08,
            "difference_mol_s": 2.3283064365386963e-10, "exceeded": true,
        })
    );
}

#[test]
fn v16_frozen_component_scales_preserve_triggers_and_earlier_invalid_residuals() {
    let fixture = Fixture::new();
    let mut request = request(&fixture);
    let (code, response) = fixture.v3(
        "query",
        &json!({
            "schema_version": 16, "project": "q7.json",
            "queries": [{"id": "plan", "query": {"kind": "results", "scope": "flow_plan"}}],
        }),
    );
    assert_eq!(code, 0, "{response}");
    request["steps"].as_array_mut().unwrap().insert(1, json!({
        "id": "configure", "action": {"kind": "configure_flow_plan", "expected_plan_revision": 0,
            "plan": response["queries"][0]["value"]["configuration"]},
    }));
    request["steps"][2]["action"]["expected_plan_revision"] = json!(1);
    for (feed, scale, expected) in [
        (537793, 1e-8, vec![("methane", false, true)]),
        (
            537800,
            1e-8,
            vec![("ethane", true, true), ("methane", false, true)],
        ),
        (10, f64::MIN_POSITIVE, vec![]),
    ] {
        request["steps"][0]["action"]["value"] = json!(feed);
        request["steps"][1]["action"]["plan"]["tears"][0]["residual_scaling"] = json!({
            "kind": "component_values", "values": [
                {"component_id": "ethane", "value_mol_s": scale},
                {"component_id": "methane", "value_mol_s": scale}
            ]
        });
        let (code, response) = fixture.v3("execute", &request);
        assert_eq!(code, 4, "{response}");
        let result = &response["steps"][2]["result"];
        assert_eq!(&response["queries"][0]["value"], result);
        let evidence = &result["diagnostic_evidence"];
        if expected.is_empty() {
            assert_eq!(
                result["diagnostic"]["code"],
                "solver.partitioned.nonfinite_residual"
            );
            assert!(evidence.get("material_balance").is_none());
        } else {
            let rows = evidence["material_balance"]["exceeded_components"]
                .as_array()
                .unwrap();
            assert_eq!(rows.len(), expected.len());
            for (row, (id, absolute, scaled)) in rows.iter().zip(expected) {
                assert_eq!(row["component_id"], id);
                assert_eq!(row["scale_mol_s"], scale);
                assert_eq!(row["absolute_exceeded"], absolute);
                assert_eq!(row["scaled_exceeded"], scaled);
            }
        }
    }
}
