use super::*;
const PROJECT: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
);
const RUN: &str =
    include_str!("../../../../examples/automation/run-mixed-material-v7.request.template.json");
const BUILD: &str =
    include_str!("../../../../examples/automation/build-mixed-material-v7.request.template.json");
fn request(f: &Fixture) -> Value {
    fs::write(f.root.join("mixed.json"), PROJECT).unwrap();
    fs::write(
        f.root.join("mixed.json.rfstudio-layout.json"),
        b"Mixed layout sentinel",
    )
    .unwrap();
    let mut r: Value = serde_json::from_str(RUN).unwrap();
    r["project"] = json!("mixed.json");
    r["flow_resources"]["cache_root"] = json!("cache");
    r["flow_resources"]["auth_cache_index"] = json!("index.json");
    r
}
fn verify(v: &Value) {
    assert_eq!(v["status"], "current", "{v}");
    assert_eq!(v["execution_kind"], "mixed_material");
    assert_eq!(v["blocks"].as_array().unwrap().len(), 4);
    assert_eq!(v["boundaries"].as_array().unwrap().len(), 3);
    assert_eq!(v["costs"]["local_unit_calls"], 7);
    for b in v["boundaries"].as_array().unwrap() {
        assert_eq!(b["producer"], b["consumed"]);
    }
    let s = v["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "s-heated")
        .unwrap();
    assert!((s["total_molar_flow_mol_s"].as_f64().unwrap() - 22.0 / 17.0).abs() < 1e-10);
    let eo = v["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "equations")
        .unwrap();
    assert_eq!(
        eo["evidence"]["result"]["evidence"]["definition"]["structure"]["variable_count"],
        24
    );
    assert_eq!(
        v["origin"]["execution"]["plan"]["core_method"],
        "radishflow.fixed-tp-mixed-material/1"
    );
}
#[test]
fn v7_real_saved_and_created_mixed_flows_leave_all_disk_entries_unchanged() {
    let f = Fixture::new();
    let r = request(&f);
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 0, "{v}");
    verify(&v["queries"][1]["value"]);
    let mut build: Value = serde_json::from_str(BUILD).unwrap();
    fs::write(
        f.root.join("basis.json"),
        include_bytes!("../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json"),
    )
    .unwrap();
    build["project"] = json!("basis.json");
    build["flow_resources"] = r["flow_resources"].clone();
    let (code, v) = f.v3("execute", &build);
    assert_eq!(code, 0, "{v}");
    verify(&v["queries"][1]["value"]);
    let(code,v)=f.v3("query",&json!({"schema_version":7,"project":"mixed.json","queries":[{"id":"plan","query":{"kind":"results","scope":"flow_plan"}},{"id":"result","query":{"kind":"results","scope":"flow"}}]}));
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["queries"][1]["value"]["status"], "missing");
    assert_eq!(
        v["queries"][0]["value"]["blocks"][2]["equation_layout"]["numerical_solvability"],
        "not_proven"
    );
}
#[test]
fn v7_legacy_readers_gate_before_io_and_nested_vocabulary_is_strict() {
    let f = Fixture::new();
    let r = request(&f);
    for version in [3, 4, 5, 6] {
        let mut old = r.clone();
        old["schema_version"] = json!(version);
        old["queries"] = json!([]);
        if version == 3 {
            old["steps"][0]["action"]
                .as_object_mut()
                .unwrap()
                .remove("expected_plan_revision");
        }
        old["flow_resources"]["cache_root"] = json!("missing");
        let (code, v) = f.v3("execute", &old);
        let (expected_code, expected_error) = if version == 3 {
            (2, "unsupported_capability")
        } else {
            (3, "read_only_project")
        };
        assert_eq!(code, expected_code, "{v}");
        assert_eq!(v["error"]["code"], expected_error);
    }
    let mut hidden: Value = serde_json::from_slice(PROJECT).unwrap();
    hidden["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["id"] != "radishflow.solver.fixed-tp-mixed-material");
    fs::write(
        f.root.join("hidden.json"),
        serde_json::to_vec(&hidden).unwrap(),
    )
    .unwrap();
    let (code, v) = f.v3(
        "query",
        &json!({"schema_version":6,"project":"hidden.json","queries":[]}),
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["project"]["read_only"], true);
    let mut build: Value = serde_json::from_str(BUILD).unwrap();
    build["project"] = json!("missing");
    let index = build["steps"]
        .as_array()
        .unwrap()
        .iter()
        .position(|s| s["id"] == "configure")
        .unwrap();
    for (key, val) in [("extra", json!(true)), ("blocks", Value::Null)] {
        let mut bad = build.clone();
        bad["steps"][index]["action"]["plan"][key] = val;
        let (code, v) = f.v3("execute", &bad);
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["code"], "invalid_request");
        assert!(v["project"].is_null());
    }
    let text = serde_json::to_string(&build).unwrap().replace(
        "\"equation_contract_version\":1",
        "\"equation_contract_version\":1,\"equation_contract_version\":1",
    );
    let (code, v) = f.v3_bytes("execute", text.as_bytes());
    assert_eq!(code, 2);
    assert_eq!(v["error"]["code"], "invalid_request");
}
