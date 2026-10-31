use super::*;
const PROJECT: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-material-eo-binary-hydrocarbon.rfproj.json"
);
const RUN: &str =
    include_str!("../../../../examples/automation/run-material-eo-v6.request.template.json");
const ADOPT: &str =
    include_str!("../../../../examples/automation/adopt-material-eo-v6.request.template.json");
const BUILD: &str =
    include_str!("../../../../examples/automation/build-material-eo-v6.request.template.json");
fn request(f: &Fixture, text: &str) -> Value {
    fs::write(f.root.join("eo.json"), PROJECT).unwrap();
    fs::write(
        f.root.join("eo.json.rfstudio-layout.json"),
        b"EO sidecar sentinel",
    )
    .unwrap();
    let mut r: Value = serde_json::from_str(text).unwrap();
    r["project"] = json!("eo.json");
    r["flow_resources"]["cache_root"] = json!("cache");
    r["flow_resources"]["auth_cache_index"] = json!("index.json");
    r
}
fn verify(result: &Value, expected_dimension: u64) {
    assert_eq!(result["status"], "current");
    assert_eq!(result["execution_kind"], "equation_oriented");
    assert_eq!(
        result["evidence"]["full_jacobian"]["dimension"],
        expected_dimension
    );
    let product = result["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "s-pa")
        .unwrap();
    assert!((product["total_molar_flow_mol_s"].as_f64().unwrap() - 1.5).abs() < 1e-10);
}
#[test]
fn v6_real_saved_build_adopt_forward_workflows_preserve_all_disk_entries() {
    let f = Fixture::new();
    for (text, dim) in [(RUN, 29), (ADOPT, 28)] {
        let r = request(&f, text);
        let (code, v) = f.v3("execute", &r);
        assert_eq!(code, 0, "{v}");
        verify(&v["queries"][1]["value"], dim);
        assert_eq!(v["project"]["project_schema_version"], 4);
    }
    let mut build: Value = serde_json::from_str(BUILD).unwrap();
    let basis =
        include_bytes!("../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json");
    fs::write(f.root.join("basis.json"), basis).unwrap();
    build["project"] = json!("basis.json");
    build["flow_resources"] = request(&f, RUN)["flow_resources"].clone();
    let (code, v) = f.v3("execute", &build);
    assert_eq!(code, 0, "{v}");
    verify(&v["queries"][1]["value"], 29);
    assert_eq!(v["project"]["project_schema_version"], 1);
    let (_,q)=f.v3("query",&json!({"schema_version":6,"project":"eo.json","queries":[{"id":"result","query":{"kind":"results","scope":"flow"}}]}));
    assert_eq!(q["queries"][0]["value"]["status"], "missing");
}
#[test]
fn v6_old_protocols_duplicate_keys_and_foreign_execution_fail_without_writes() {
    let f = Fixture::new();
    let r = request(&f, RUN);
    for version in [4, 5] {
        let mut bad = r.clone();
        bad["schema_version"] = json!(version);
        bad["flow_resources"]["auth_cache_index"] = json!("missing");
        let (code, v) = f.v3("execute", &bad);
        assert_eq!(code, 3, "{v}");
        assert_eq!(v["error"]["code"], "read_only_project");
    }
    let mut r = request(&f, ADOPT);
    r["steps"][1]["action"]["execution"]["step"] = json!("foreign-process");
    let (code, v) = f.v3("execute", &r);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "unknown_reference");
    assert_eq!(v["queries"][1]["value"]["status"], "current");
    for text in [
        r#"{"schema_version":6,"project":"missing","queries":[],"queries":[]}"#,
        r#"{"schema_version":6,"project":"missing","queries":[{"id":"x","query":{"kind":"results","scope":"flow_plan","extra":true}}]}"#,
    ] {
        let (code, v) = f.v3_bytes("query", text.as_bytes());
        assert_eq!(code, 2);
        assert_eq!(v["error"]["code"], "invalid_request");
        assert!(v["project"].is_null());
    }
}
