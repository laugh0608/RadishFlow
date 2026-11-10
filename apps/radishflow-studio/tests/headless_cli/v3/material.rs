use super::*;
fn request(f: &Fixture) -> Value {
    fs::write(
        f.root.join("material.json"),
        include_bytes!(
            "../../../../../examples/flowsheets/feed-mixer-material-balance.rfproj.json"
        ),
    )
    .unwrap();
    let mut r: Value = serde_json::from_str(include_str!(
        "../../../../../examples/automation/material-balance-v3.request.json"
    ))
    .unwrap();
    r["project"] = json!("material.json");
    r
}
#[test]
fn material_public_example_runs_without_flow_resources_preserves_disk_and_never_fabricates_thermal_results()
 {
    let f = Fixture::new();
    let request = request(&f);
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 0, "{r}");
    let result = &r["queries"][0]["value"];
    assert_eq!(result["status"], "current");
    assert_eq!(result["origin"]["execution"]["step"], "material");
    assert!(
        result["origin"]["execution"]["solver_implementation"]
            .as_str()
            .unwrap()
            .contains("lower-triangular-substitution/1@")
    );
    assert_eq!(result["variables"].as_array().unwrap().len(), 6);
    assert_eq!(result["statistics"]["residual_evaluations"], 1);
    assert_eq!(result["outlet"]["total_molar_flow"]["value_si"], 5.0);
    let methane = result["outlet"]["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["component_id"] == "methane")
        .unwrap();
    assert!((methane["mole_fraction"]["value_si"].as_f64().unwrap() - 0.44).abs() < 1e-12);
    assert!(result.get("phases").is_none());
    assert_eq!(r["queries"][1]["value"]["status"], "missing");
    let reopened=f.query_v3("material.json",json!([{"id":"material","query":{"kind":"results","scope":"material_balance"}},{"id":"actions","query":{"kind":"actions","scope":"material_balance"}}]));
    assert_eq!(reopened["queries"][0]["value"]["status"], "missing");
    assert!(
        reopened["queries"][1]["value"]
            .to_string()
            .contains("run_material_balance")
    );
}
#[test]
fn material_invalid_plan_redundancy_and_unknown_fields_are_explicit_rejections() {
    let f = Fixture::new();
    let original = request(&f);
    let mut r = original.clone();
    r["steps"][0]["action"]["settings"]["scaled_tolerance"] = json!(0.0);
    let (code, result) = f.v3("execute", &r);
    assert_eq!(code, 3, "{result}");
    assert_eq!(
        result["error"]["details"]["diagnostic"]["code"],
        "material.invalid_tolerance"
    );
    let mut r = original.clone();
    r["steps"][0]["action"]["settings"]["outlet_total_molar_flow_mol_s"] = json!(5.0);
    let (code, result) = f.v3("execute", &r);
    assert_eq!(code, 4, "{result}");
    assert_eq!(
        result["steps"][0]["result"]["diagnostic"]["code"],
        "material.redundant_specification"
    );
    assert_eq!(
        result["steps"][0]["result"]["diagnostic"]["unit_ids"],
        json!(["mixer-1"])
    );
    let mut r = original;
    r["steps"][0]["action"]["settings"]["typo"] = json!(1);
    let (code, _) = f.v3("execute", &r);
    assert_eq!(code, 2);
}
#[test]
fn formal_input_edit_invalidates_material_result_and_rerun_uses_the_new_revision() {
    let f = Fixture::new();
    let mut r = request(&f);
    let edit = json!({"id":"change","action":{"kind":"write","expected_revision":0,"target":{"object":{"stream":{"id":"stream-feed-a"}},"section":"inputs","field":"molar_flow"},"value":4.0}});
    r["steps"].as_array_mut().unwrap().push(edit);
    let (code, stale) = f.v3("execute", &r);
    assert_eq!(code, 0, "{stale}");
    assert_eq!(stale["queries"][0]["value"]["status"], "stale");
    assert!(stale["queries"][0]["value"].get("outlet").is_none());
    let mut repeat = r["steps"][0].clone();
    repeat["id"] = json!("material_again");
    repeat["action"]["expected_revision"] = json!({"step":"change","output":"document"});
    r["steps"].as_array_mut().unwrap().push(repeat);
    let (code, current) = f.v3("execute", &r);
    assert_eq!(code, 0, "{current}");
    assert!(
        (current["queries"][0]["value"]["outlet"]["total_molar_flow"]["value_si"]
            .as_f64()
            .unwrap()
            - 7.0)
            .abs()
            < 1e-10
    );
    assert_eq!(
        current["queries"][0]["value"]["origin"]["execution"]["step"],
        "material_again"
    );
}
