use super::*;
fn plan() -> Value {
    json!({"method":"radishflow.fixed-tp-affine-modular-lu/1","acceptance_contract_version":1,"response_contract_version":1,"temperature_k":300.0,"pressure_pa":650000.0,"scaling":{"kind":"evaluated_fresh_total"}})
}
fn project(f: &Fixture, a: f64, b: f64) -> Value {
    let mut app = crate::qs2_test_support::cross_case(a, b);
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
    r["schema_version"] = json!(5);
    r
}
#[test]
fn v5_real_cross_recycle_and_old_v4_protection() {
    for (a, b, success) in [(0.4, 0.3, true), (0.99, 0.99, true), (1.0, 1.0, false)] {
        let f = Fixture::new();
        let p = project(&f, a, b);
        let mut r = request(
            &f,
            &p,
            json!([{"id":"solve","action":{"kind":"run_flow","expected_revision":p["document"]["revision"],"expected_plan_revision":0}}]),
        );
        r["flow_resources"] = super::v4::resources(&f);
        r["queries"] = json!([{"id":"plan","query":{"kind":"results","scope":"flow_plan"}},{"id":"result","query":{"kind":"results","scope":"flow"}}]);
        let (code, v) = f.execute(&r);
        assert_eq!(code, if success { 0 } else { 4 }, "{v}");
        assert_eq!(v["schema_version"], 5);
        assert!(
            v["project"]["in_memory_required_capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == rf_store::project_v4::SIMULTANEOUS_CAPABILITY)
        );
        let result = &v["queries"][1]["value"];
        if success {
            assert_eq!(result["execution_kind"], "simultaneous_modular");
            assert_eq!(result["costs"]["local_unit_calls"], 10);
            assert_eq!(result["costs"]["triangular_solves"], 9);
            assert_eq!(result["blocks"][2]["coupling"]["linear"]["dimension"], 8);
            assert!(result["costs"].get("iteration_evaluations").is_none());
        } else {
            assert_eq!(result["status"], "failed");
            assert!(result.get("fields").is_none());
            assert!(result.get("streams").is_none());
            assert_eq!(
                result["diagnostic_evidence"]["failure_kind"],
                "no_finite_steady_state"
            );
            assert!(result["diagnostic_evidence"]["active_coupling"]["linear"].is_null());
        }
        r["schema_version"] = json!(4);
        r["flow_resources"]["cache_root"] = json!("missing-cache");
        let (code, v) = f.execute(&r);
        assert_eq!(code, 3, "{v}");
        assert_eq!(v["error"]["code"], "read_only_project");
        assert!(v["steps"].as_array().unwrap().is_empty());
        let (code,v)=f.query(json!({"schema_version":4,"project":"project.json","queries":[{"id":"plan","query":{"kind":"results","scope":"flow_plan"}}]}));
        assert_eq!(code, 0);
        assert_eq!(v["queries"][0]["value"]["read_only"], true);
    }
}
#[test]
fn v5_strict_plan_configuration_and_in_memory_capability_changes() {
    let f = Fixture::new();
    let mut r = request(
        &f,
        &empty_project(),
        json!([{"id":"config","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":plan()}}]),
    );
    r["queries"] = json!([{"id":"p","query":{"kind":"results","scope":"flow_plan"}}]);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["project"]["project_schema_version"], 1);
    assert_eq!(v["project"]["in_memory_required_project_schema_version"], 4);
    assert_eq!(v["queries"][0]["value"]["ready"], false);
    for field in [
        "acceptance_contract_version",
        "response_contract_version",
        "scaling",
    ] {
        let mut bad = r.clone();
        bad["steps"][0]["action"]["plan"][field] = Value::Null;
        let (code, v) = f.execute(&bad);
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["code"], "invalid_request");
    }
    let mut bad = r.clone();
    bad["steps"][0]["action"]["plan"]["tears"] = json!([]);
    assert_eq!(f.execute(&bad).0, 2);
    let mut bad = r.clone();
    bad["steps"][0]["action"]["plan"]["response_contract_version"] = json!(2);
    let (_, v) = f.execute(&bad);
    assert_eq!(v["error"]["code"], "unsupported_capability");
    let mut bad = r.clone();
    bad["schema_version"] = json!(4);
    assert_eq!(f.execute(&bad).0, 2);
    let scale = json!({"stream":{"id":"s"},"source":{"unit":{"id":"u"},"port":"outlet"},"sink":{"unit":{"id":"v"},"port":"inlet"},"component_id":"methane","variable_mol_s":2.0,"residual_mol_s":5.0});
    r["steps"][0]["action"]["plan"]["scaling"] =
        json!({"kind":"explicit_couplings","values":[scale.clone(),scale]});
    let (_, v) = f.execute(&r);
    assert_eq!(v["error"]["code"], "invalid_request");
    r["steps"][0]["action"]["plan"]["scaling"]["values"]
        .as_array_mut()
        .unwrap()
        .pop();
    r["steps"][0]["action"]["plan"]["scaling"]["values"][0]["stream"] =
        json!({"step":"future","output":"stream"});
    let (_, v) = f.execute(&r);
    assert_eq!(v["error"]["code"], "unknown_reference");
}
#[test]
fn v5_confirmation_namespace_and_plan_revision_receipts() {
    let f = Fixture::new();
    let mut r = request(
        &f,
        &empty_project(),
        json!([
            {"id":"config","action":{"kind":"configure_flow_plan","expected_plan_revision":0,"plan":plan()}},
            {"id":"same","action":{"kind":"configure_flow_plan","expected_plan_revision":{"step":"config","output":"plan"},"plan":plan()}},
            {"id":"source","action":{"kind":"create_source"}},
            {"id":"delete","action":{"kind":"delete_equipment_record","target":{"source":{"step":"source","output":"source"}},"expected_revision":{"step":"source","output":"source"}}}
        ]),
    );
    let (code, v) = f.execute(&r);
    assert_eq!(code, 6, "{v}");
    assert_eq!(v["steps"][1]["plan_revision"], 1);
    assert_eq!(v["steps"][1]["outcome"], "unchanged");
    let digest = v["error"]["details"]["confirmation_digest"].clone();
    assert!(
        digest
            .as_str()
            .unwrap()
            .starts_with("headless-v5-confirm-v1:")
    );
    let (_, again) = f.execute(&r);
    assert_eq!(again["error"]["details"]["confirmation_digest"], digest);
    r["steps"][3]["confirmation"] = json!({"digest":digest});
    assert_eq!(f.execute(&r).0, 0);
    r["steps"][0]["action"]["plan"]["pressure_pa"] = json!(640000.0);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 6, "{v}");
    assert_eq!(v["error"]["code"], "confirmation_mismatch");
}
#[test]
fn v5_explicit_scales_resolve_prior_receipts_and_canonical_order_is_a_noop() {
    let f = Fixture::new();
    let p = empty_project();
    let mut r: Value = serde_json::from_str(include_str!(
        "../../../../../examples/automation/build-cross-recycle-v5.request.template.json"
    ))
    .unwrap();
    r["project"] = json!("project.json");
    let file = f.write("project.json", &p);
    r["expected_project"]["file_sha256"] = json!(rf_store::sha256_hex(&fs::read(file).unwrap()));
    r["flow_resources"] = super::v4::resources(&f);
    let mut values = vec![];
    for (stream, source, out, sink, inp) in [
        ("ma", "mixer_a", "outlet", "splitter_a", "inlet"),
        ("mb", "mixer_b", "outlet", "splitter_b", "inlet"),
        ("rab", "splitter_a", "outlet_a", "mixer_b", "inlet_b"),
        ("rba", "splitter_b", "outlet_a", "mixer_a", "inlet_b"),
    ] {
        for component in ["methane", "ethane"] {
            values.push(json!({"stream":{"step":stream,"output":"stream"},"source":{"unit":{"step":source,"output":"unit"},"port":out},"sink":{"unit":{"step":sink,"output":"unit"},"port":inp},"component_id":component,"variable_mol_s":2.0,"residual_mol_s":5.0}));
        }
    }
    let steps = r["steps"].as_array_mut().unwrap();
    let config = steps.iter_mut().find(|s| s["id"] == "configure").unwrap();
    config["action"]["plan"]["scaling"] = json!({"kind":"explicit_couplings","values":values});
    let mut same = config.clone();
    same["id"] = json!("same");
    same["action"]["expected_plan_revision"] = json!({"step":"configure","output":"plan"});
    same["action"]["plan"]["scaling"]["values"]
        .as_array_mut()
        .unwrap()
        .reverse();
    steps.insert(steps.len() - 1, same);
    let (code, v) = f.execute(&r);
    assert_eq!(code, 0, "{v}");
    let same = v["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "same")
        .unwrap();
    assert_eq!(same["outcome"], "unchanged");
    let coupling = &v["queries"][1]["value"]["blocks"][2]["coupling"];
    for c in coupling["connections"].as_array().unwrap() {
        assert_eq!(c["scale"]["variable_mol_s"], 2.0);
        assert_eq!(c["scale"]["residual_mol_s"], 5.0);
    }
}
#[test]
fn open_exact_outlet_is_v5_only_and_explicit_null_is_never_accepted() {
    let f = Fixture::new();
    let p = empty_project();
    let steps = json!([{"id":"feed","action":{"kind":"create_unit","expected_revision":0,"unit_kind":"feed"}},{"id":"outlet","action":{"kind":"connect_exact","expected_revision":{"step":"feed","output":"document"},"source":{"unit":{"step":"feed","output":"unit"},"port":"outlet"},"stream":{"id":"product"}}}]);
    let mut r = request(&f, &p, steps);
    assert_eq!(f.execute(&r).0, 0);
    r["steps"][1]["action"]["sink"] = Value::Null;
    let (_, v) = f.execute(&r);
    assert_eq!(v["error"]["code"], "invalid_request");
    r["steps"][1]["action"]
        .as_object_mut()
        .unwrap()
        .remove("sink");
    r["schema_version"] = json!(4);
    let (_, v) = f.execute(&r);
    assert_eq!(v["error"]["code"], "invalid_request");
    assert!(v["steps"].as_array().unwrap().is_empty());
}
