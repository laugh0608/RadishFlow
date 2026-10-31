use super::*;
fn plan() -> StoredFlowPlan {
    StoredFlowPlan::EquationOriented(StoredEquationPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: 300.0,
            pressure_pa: 650000.0,
        },
        specifications: vec![
            StoredEquationSpecification::ReleaseSplitFraction {
                unit_id: "sa".into(),
                field: StoredEquationParameter::OutletASplitFraction,
                interval: StoredFractionInterval {
                    lower: 0.0,
                    upper: 0.95,
                    max_iterations: 64,
                },
            },
            StoredEquationSpecification::ProductTotalMolarFlow {
                stream_id: "product".into(),
                source: StoredPortBinding {
                    unit_id: "sa".into(),
                    port: "outlet_b".into(),
                },
                value_mol_s: 1.5,
            },
        ],
        scaling: StoredEquationScaling::EvaluatedFreshTotal,
    })
}
#[test]
fn equation_codec_preserves_partial_specs_and_protects_both_old_consumers() {
    let mut p = fixture();
    p.steady_state_plan = plan();
    let original = json_project(p.clone());
    assert_eq!(
        parse(&original).unwrap(),
        ProjectV4Read::Editable(Box::new(p.clone()))
    );
    for support in [FlowPlanSupport::Sequential, FlowPlanSupport::Simultaneous] {
        let ProjectV4Read::ReadOnly(old) = parse_with_support(&original, support).unwrap() else {
            panic!()
        };
        assert_eq!(old.original_json(), original);
        assert!(to_pretty_json(&ProjectV4Read::ReadOnly(old)).is_err());
        let mut hidden: Value = serde_json::from_str(&original).unwrap();
        hidden["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v["id"] != EQUATION_ORIENTED_CAPABILITY);
        assert!(parse(&hidden.to_string()).is_err());
        assert!(matches!(
            parse_with_support(&hidden.to_string(), support).unwrap(),
            ProjectV4Read::ReadOnly(_)
        ));
    }
    let StoredFlowPlan::EquationOriented(o) = &mut p.steady_state_plan else {
        panic!()
    };
    o.specifications.reverse();
    assert_eq!(json_project(p.clone()), original);
    // Configuration shape may be saved while incomplete or bound to missing objects.
    let StoredFlowPlan::EquationOriented(o) = &mut p.steady_state_plan else {
        panic!()
    };
    o.specifications.pop();
    let text = json_project(p.clone());
    assert_eq!(parse(&text).unwrap(), ProjectV4Read::Editable(Box::new(p)));
}
#[test]
fn equation_specs_scales_and_duplicates_are_strict() {
    let mut p = fixture();
    p.steady_state_plan = plan();
    let original: Value = serde_json::from_str(&json_project(p)).unwrap();
    for (path, bad) in [
        (
            "/simulation/steadyStatePlan/specifications/0/interval/lower",
            json!(-0.1),
        ),
        (
            "/simulation/steadyStatePlan/specifications/0/interval/maxIterations",
            json!(0),
        ),
        (
            "/simulation/steadyStatePlan/specifications/1/valueMolS",
            json!(0),
        ),
        (
            "/simulation/steadyStatePlan/specifications/0/field",
            json!("outlet_pressure"),
        ),
        ("/simulation/steadyStatePlan/specifications", Value::Null),
    ] {
        let mut v = original.clone();
        *v.pointer_mut(path).unwrap() = bad;
        assert!(parse(&v.to_string()).is_err(), "{path}");
    }
    let mut v = original.clone();
    let duplicate = v["simulation"]["steadyStatePlan"]["specifications"][0].clone();
    v["simulation"]["steadyStatePlan"]["specifications"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(parse(&v.to_string()).is_err());
    let StoredFlowPlan::EquationOriented(mut o) = plan() else {
        panic!()
    };
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        o.scaling = StoredEquationScaling::ExplicitEquations {
            variables: vec![StoredEquationScale {
                binding: StoredEquationVariableBinding::Parameter {
                    unit_id: "sa".into(),
                    field: StoredEquationParameter::OutletASplitFraction,
                },
                scale_si: bad,
            }],
            equations: vec![],
        };
        assert!(serde_json::to_string(&StoredFlowPlan::EquationOriented(o.clone())).is_err());
    }
}
#[test]
fn equation_versions_and_strict_fields_do_not_fall_back() {
    let mut project = fixture();
    project.steady_state_plan = plan();
    let original: Value = serde_json::from_str(&json_project(project)).unwrap();
    for field in ["acceptanceContractVersion", "equationContractVersion"] {
        for bad in [Value::Null, json!(0), json!("1"), json!(-1)] {
            let mut v = original.clone();
            v["simulation"]["steadyStatePlan"][field] = bad;
            assert!(parse(&v.to_string()).is_err(), "{field}: {v}");
        }
        let mut v = original.clone();
        v["simulation"]["steadyStatePlan"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(parse(&v.to_string()).is_err());
        v["simulation"]["steadyStatePlan"][field] = json!(2);
        assert!(matches!(
            parse(&v.to_string()).unwrap(),
            ProjectV4Read::ReadOnly(_)
        ));
    }
    for (field, value) in [
        ("tears", json!([])),
        ("maxIterationEvaluations", json!(500)),
        ("unknown", json!(true)),
        ("scaling", Value::Null),
    ] {
        let mut v = original.clone();
        v["simulation"]["steadyStatePlan"][field] = value;
        assert!(parse(&v.to_string()).is_err());
    }
    let text = original.to_string().replace(
        "\"equationContractVersion\":1",
        "\"equationContractVersion\":1,\"equationContractVersion\":1",
    );
    assert!(parse(&text).is_err());
}
#[test]
fn equation_capability_upgrade_and_target_fingerprint_are_enforced_by_writer() {
    let path =
        std::env::temp_dir().join(format!("rf-store-qs3-target-{}.json", std::process::id()));
    let old = fixture();
    fs::write(&path, json_project(old.clone())).unwrap();
    let mut next = old;
    next.steady_state_plan = plan();
    let new = ProjectV4Read::Editable(Box::new(next));
    let prepared = prepare_write(&path, &new, FileOverwritePolicy::Allow).unwrap();
    assert!(prepared.requires_upgrade());
    let before = fs::read(&path).unwrap();
    assert!(write_prepared(&prepared, false).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    fs::write(&path, b"changed").unwrap();
    assert!(write_prepared(&prepared, true).is_err());
    fs::write(&path, before).unwrap();
    write_prepared(&prepared, true).unwrap();
    assert!(
        !prepare_write(&path, &new, FileOverwritePolicy::Allow)
            .unwrap()
            .requires_upgrade()
    );
    let mut unsupported: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    unsupported["requiredCapabilities"][0]["version"] = json!(99);
    fs::write(&path, unsupported.to_string()).unwrap();
    assert!(prepare_write(&path, &new, FileOverwritePolicy::Allow).is_err());
    fs::remove_file(&path).unwrap();
}
