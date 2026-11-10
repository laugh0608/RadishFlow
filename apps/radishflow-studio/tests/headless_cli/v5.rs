use super::*;
const CROSS: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json"
);
const RUN: &str =
    include_str!("../../../../examples/automation/run-cross-recycle-v5.request.template.json");
const BUILD: &str =
    include_str!("../../../../examples/automation/build-cross-recycle-v5.request.template.json");
fn request(f: &Fixture) -> Value {
    fs::write(f.root.join("cross.json"), CROSS).unwrap();
    fs::write(
        f.root.join("cross.json.rfstudio-layout.json"),
        b"sidecar sentinel",
    )
    .unwrap();
    let mut r: Value = serde_json::from_str(RUN).unwrap();
    r["project"] = json!("cross.json");
    r["flow_resources"]["cache_root"] = json!("cache");
    r["flow_resources"]["auth_cache_index"] = json!("index.json");
    r
}
fn verify(v: &Value) {
    let r = &v["queries"][1]["value"];
    assert_eq!(r["status"], "current");
    assert_eq!(r["execution_kind"], "simultaneous_modular");
    assert_eq!(r["blocks"].as_array().unwrap().len(), 3);
    assert_eq!(r["costs"]["local_unit_calls"], 10);
    assert_eq!(r["costs"]["triangular_solves"], 9);
    assert_eq!(r["blocks"][2]["coupling"]["linear"]["dimension"], 8);
    assert_eq!(
        r["origin"]["execution"]["plan"]["core_method"],
        "rf-solver.fixed-tp-affine-modular-lu/1"
    );
    assert!(
        (r["blocks"][2]["coupling"]["linear"]["rcond_indicator"]
            .as_f64()
            .unwrap()
            - 17.0 / 128.0)
            .abs()
            < 1e-12
    );
}
#[test]
fn v5_public_saved_and_formal_build_examples_preserve_every_disk_entry() {
    let f = Fixture::new();
    let mut r = request(&f);
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 0, "{v}");
    verify(&v);
    // The saved fixture contains deliberately wrong internal templates: outputs must be solved producers.
    let streams = v["queries"][1]["value"]["streams"].as_array().unwrap();
    for (id, a, b) in [
        ("s-ma", 35.0 / 17.0, 20.0 / 17.0),
        ("s-mb", 156.0 / 85.0, 264.0 / 85.0),
        ("s-pa", 14.0 / 17.0, 8.0 / 17.0),
        ("s-pb", 117.0 / 85.0, 198.0 / 85.0),
    ] {
        let s = streams.iter().find(|s| s["id"] == id).unwrap();
        for (id, expected) in [("methane", a), ("ethane", b)] {
            let c = s["components"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["component_id"] == id)
                .unwrap();
            assert!((c["flow_mol_s"].as_f64().unwrap() - expected).abs() < 1e-9);
        }
    }
    r["steps"].as_array_mut().unwrap().push(json!({"id":"change","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":{"method":"radishflow.acyclic-sm/1"}}}));
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["queries"][1]["value"]["status"], "stale");
    assert!(v["queries"][1]["value"].get("streams").is_none());
    assert_eq!(
        v["queries"][1]["value"]["origin"]["execution"]["plan"]["core_method"],
        "rf-solver.fixed-tp-affine-modular-lu/1"
    );
    let (code,v)=f.v3("query",&json!({"schema_version":5,"project":"cross.json","queries":[{"id":"r","query":{"kind":"results","scope":"flow"}}]}));
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["queries"][0]["value"]["status"], "missing");
    let mut build: Value = serde_json::from_str(BUILD).unwrap();
    let bytes =
        include_bytes!("../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json");
    fs::write(f.root.join("basis.json"), bytes).unwrap();
    build["project"] = json!("basis.json");
    build["flow_resources"] = r["flow_resources"].clone();
    let (code, v) = f.v3("execute", &build);
    assert_eq!(code, 0, "{v}");
    verify(&v);
    assert_eq!(v["project"]["project_schema_version"], 1);
    assert_eq!(v["project"]["in_memory_required_project_schema_version"], 4);
}
#[test]
fn v5_failure_old_protocol_and_strict_requests_do_not_touch_resources() {
    let f = Fixture::new();
    let mut r = request(&f);
    r["schema_version"] = json!(4);
    r["flow_resources"]["auth_cache_index"] = json!("missing");
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 3, "{v}");
    assert_eq!(v["error"]["code"], "read_only_project");
    assert!(v["steps"].as_array().unwrap().is_empty());
    for bytes in [
        r#"{"schema_version":5,"project":"missing","queries":[],"queries":[]}"#,
        r#"{"schema_version":5,"project":"missing","queries":[{"id":"p","query":{"kind":"results","scope":"flow_plan","extra":1}}]}"#,
    ] {
        let (code, v) = f.v3_bytes("query", bytes.as_bytes());
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["code"], "invalid_request");
        assert!(v["project"].is_null());
    }
    r = request(&f);
    let mut p: Value = serde_json::from_slice(CROSS).unwrap();
    for id in ["splitter-1", "splitter-2"] {
        p["document"]["flowsheet"]["units"][id]["parameters"]["outlet_a_split_fraction"] =
            json!(1.0);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    fs::write(f.root.join("cross.json"), &bytes).unwrap();
    r["expected_project"]["file_sha256"] = json!(rf_store::sha256_hex(&bytes));
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 4, "{v}");
    let result = &v["queries"][1]["value"];
    assert_eq!(
        result["diagnostic_evidence"]["completed_blocks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(result.get("fields").is_none());
    assert!(result.get("streams").is_none());
}
