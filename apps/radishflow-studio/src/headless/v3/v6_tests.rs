use super::*;
fn project(f: &Fixture) -> Value {
    let mut app = crate::qs3_test_support::case(true);
    crate::dispatch_document_lifecycle(
        &mut app,
        crate::StudioDocumentLifecycleCommand::SaveUpgraded {
            path: f.0.join("built.json"),
        },
    )
    .unwrap();
    serde_json::from_slice(&fs::read(f.0.join("built.json")).unwrap()).unwrap()
}
fn request(f: &Fixture, p: &Value, steps: Value) -> Value {
    let mut r = super::v4::request(f, p, steps);
    r["schema_version"] = json!(6);
    r
}
fn plan(inverse: bool) -> Value {
    use crate::headless::planned::PlanProtocol;
    <crate::headless::v6::Plan as PlanProtocol>::wire(
        &rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(
            crate::qs3_test_support::options(inverse),
        ),
    )
    .unwrap()
}
fn adoption(revision: Value) -> Value {
    json!({"id":"adopt","action":{"kind":"adopt_eo_split_fraction","expected_revision":revision,"expected_plan_revision":0,"execution":{"step":"solve"},"target":{"object":{"unit":{"id":"splitter-1"}},"section":"inputs","field":"outlet_a_split_fraction"}}})
}
#[test]
fn v6_inverse_adoption_and_forward_rerun_use_only_current_request_execution() {
    let f = Fixture::new();
    let p = project(&f);
    let rev = p["document"]["revision"].clone();
    let mut r = request(
        &f,
        &p,
        json!([
            {"id":"solve","action":{"kind":"run_flow","expected_revision":rev,"expected_plan_revision":0}},
            adoption(rev.clone())
        ]),
    );
    r["flow_resources"] = super::v4::resources(&f);
    r["queries"] = json!([{"id":"plan","query":{"kind":"results","scope":"flow_plan"}},{"id":"flow","query":{"kind":"results","scope":"flow"}}]);
    let before = fs::read(f.0.join("project.json")).unwrap();
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    let flow = &v["steps"][0]["result"]; // execution receipt retains its successful evidence
    assert_eq!(flow["execution_kind"], "equation_oriented");
    assert_eq!(flow["evidence"]["full_jacobian"]["dimension"], 29);
    assert_eq!(
        v["steps"][1]["after_revision"].as_u64(),
        Some(rev.as_u64().unwrap() + 1)
    );
    assert_eq!(v["queries"][1]["value"]["status"], "stale");
    assert!(v["queries"][1]["value"].get("fields").is_none());
    assert_eq!(fs::read(f.0.join("project.json")).unwrap(), before);
    r["steps"].as_array_mut().unwrap().extend([
        json!({"id":"forward","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":plan(false)}}),
        json!({"id":"rerun","action":{"kind":"run_flow","expected_revision":{"step":"adopt","output":"document"},"expected_plan_revision":{"step":"forward","output":"plan"}}})
    ]);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["queries"][1]["value"]["status"], "current");
    assert_eq!(
        v["queries"][1]["value"]["origin"]["execution"]["step"],
        "rerun"
    );
    let streams = v["queries"][1]["value"]["streams"].as_array().unwrap();
    let product = streams.iter().find(|s| s["id"] == "s-pa").unwrap();
    assert!((product["total_molar_flow_mol_s"].as_f64().unwrap() - 1.5).abs() < 1e-10);
    for (pointer, value) in [
        ("/steps/1/action/execution/step", json!("prior-request")),
        ("/steps/1/action/target/object/unit/id", json!("splitter-2")),
    ] {
        let mut bad = r.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        let (code, v) = f.execute(&bad);
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["step_id"], "adopt");
    }
    let mut bad = r.clone();
    bad["steps"][1]["action"]["value"] = json!(0.7);
    assert_eq!(f.execute(&bad).0, 2);
}
#[test]
fn v6_storage_v4_and_v5_readonly_boundaries_and_strict_fields() {
    let f = Fixture::new();
    let p = project(&f);
    for version in [4, 5] {
        let mut r = request(&f, &p, json!([]));
        r["schema_version"] = json!(version);
        let (code, v) = f.execute(&r);
        assert_eq!(code, 3, "{v}");
        assert_eq!(v["error"]["code"], "read_only_project");
        let (_,q)=f.query(json!({"schema_version":version,"project":"project.json","queries":[{"id":"p","query":{"kind":"results","scope":"flow_plan"}}]}));
        assert_eq!(q["queries"][0]["value"]["read_only"], true);
    }
    let mut r = request(
        &f,
        &empty_project(),
        json!([{"id":"configure","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":plan(true)}}]),
    );
    r["queries"] = json!([{"id":"p","query":{"kind":"results","scope":"flow_plan"}}]);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["project"]["project_schema_version"], 1);
    assert_eq!(v["project"]["in_memory_required_project_schema_version"], 4);
    for field in [
        "equation_contract_version",
        "acceptance_contract_version",
        "temperature_k",
        "pressure_pa",
        "specifications",
        "scaling",
    ] {
        let mut bad = r.clone();
        bad["steps"][0]["action"]["plan"][field] = Value::Null;
        assert_eq!(f.execute(&bad).0, 2, "{field}");
    }
    for version in [4, 5] {
        let mut bad = r.clone();
        bad["schema_version"] = json!(version);
        assert_eq!(f.execute(&bad).0, 2);
    }
    let mut bad = r.clone();
    bad["steps"][0]["action"]["plan"]["equation_contract_version"] = json!(2);
    assert_eq!(f.execute(&bad).1["error"]["code"], "unsupported_capability");
    let mut bad = r.clone();
    let duplicate = bad["steps"][0]["action"]["plan"]["specifications"][0].clone();
    bad["steps"][0]["action"]["plan"]["specifications"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert_ne!(f.execute(&bad).0, 0);
}
#[test]
fn v6_complete_scale_roundtrip_and_typed_references() {
    use crate::headless::planned::PlanProtocol;
    use crate::headless::v3::protocol::Output;
    use crate::headless::v3::references::{Bindings, BoundOutput};
    type P = crate::headless::v6::Plan;
    let app = crate::qs3_test_support::case(true);
    let config = crate::qs3_test_support::explicit(&app, true);
    let wire = P::wire(&config).unwrap();
    let decoded: P = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(
        P::configuration(&decoded, &Bindings::default()).unwrap(),
        config
    );
    let mut wire = wire;
    wire["specifications"][0]["unit"] = json!({"step":"created","output":"unit"});
    let decoded: P = serde_json::from_value(wire).unwrap();
    assert!(P::configuration(&decoded, &Bindings::default()).is_err());
    let refs = Bindings(std::collections::BTreeMap::from([(
        "created".into(),
        std::collections::BTreeMap::from([(
            Output::Unit,
            BoundOutput {
                id: "splitter-1".into(),
                revision: 1,
            },
        )]),
    )]));
    assert_eq!(P::configuration(&decoded, &refs).unwrap(), config);
}
#[test]
fn v6_unbracketed_failure_and_incomplete_structure_have_no_success_fields() {
    let f = Fixture::new();
    let p = project(&f);
    let mut config = plan(true);
    config["specifications"][1]["value_mol_s"] = json!(3.0);
    let mut r = request(
        &f,
        &p,
        json!([
            {"id":"config","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":config}},
            {"id":"solve","action":{"kind":"run_flow","expected_revision":p["document"]["revision"],"expected_plan_revision":{"step":"config","output":"plan"}}}
        ]),
    );
    r["flow_resources"] = super::v4::resources(&f);
    r["queries"] = json!([{"id":"flow","query":{"kind":"results","scope":"flow"}}]);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 4, "{v}");
    let result = &v["queries"][0]["value"];
    assert_eq!(
        result["diagnostic_evidence"]["failure_kind"],
        "target_not_bracketed"
    );
    assert!(result.get("streams").is_none());
    assert!(result.get("fields").is_none());
    r["steps"][0]["action"]["plan"]["specifications"]
        .as_array_mut()
        .unwrap()
        .pop();
    r["steps"].as_array_mut().unwrap().pop();
    r.as_object_mut().unwrap().remove("flow_resources");
    r["queries"][0]["query"]["scope"] = json!("flow_plan");
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["queries"][0]["value"]["ready"], false);
    assert_eq!(
        v["queries"][0]["value"]["equation_preview"]["structure"]["variable_count"],
        29
    );
    assert_eq!(
        v["queries"][0]["value"]["equation_preview"]["structure"]["equation_count"],
        28
    );
}
