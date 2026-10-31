use super::*;

fn plan() -> StoredFlowPlan {
    StoredFlowPlan::Simultaneous(StoredModularPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        scaling: StoredCouplingScaling::EvaluatedFreshTotal,
    })
}

fn scale(stream: &str) -> StoredCouplingScale {
    StoredCouplingScale {
        stream_id: stream.into(),
        source: StoredPortBinding {
            unit_id: "splitter".into(),
            port: "outlet_a".into(),
        },
        sink: StoredPortBinding {
            unit_id: "mixer".into(),
            port: "inlet_b".into(),
        },
        component_id: "methane".into(),
        variable_mol_s: 5.0,
        residual_mol_s: 7.0,
    }
}

#[test]
fn modular_codec_is_canonical_and_protects_old_consumers_before_domain_decode() {
    let mut project = fixture();
    project.steady_state_plan = plan();
    let default = json_project(project.clone());
    assert_eq!(
        parse(&default).unwrap(),
        ProjectV4Read::Editable(Box::new(project.clone()))
    );
    let ProjectV4Read::ReadOnly(old) =
        parse_with_support(&default, FlowPlanSupport::Sequential).unwrap()
    else {
        panic!("old consumer must be read-only")
    };
    assert_eq!(old.original_json(), default);
    assert!(to_pretty_json(&ProjectV4Read::ReadOnly(old)).is_err());
    let mut hidden: Value = serde_json::from_str(&default).unwrap();
    hidden["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["id"] != SIMULTANEOUS_CAPABILITY);
    assert!(parse(&hidden.to_string()).is_err());
    assert!(matches!(
        parse_with_support(&hidden.to_string(), FlowPlanSupport::Sequential).unwrap(),
        ProjectV4Read::ReadOnly(_)
    ));

    let StoredFlowPlan::Simultaneous(o) = &mut project.steady_state_plan else {
        unreachable!()
    };
    o.scaling = StoredCouplingScaling::ExplicitCouplings {
        values: vec![scale("z"), scale("a")],
    };
    let a = json_project(project.clone());
    let StoredFlowPlan::Simultaneous(o) = &mut project.steady_state_plan else {
        unreachable!()
    };
    let StoredCouplingScaling::ExplicitCouplings { values } = &mut o.scaling else {
        unreachable!()
    };
    values.reverse();
    let b = json_project(project);
    assert_eq!(a, b);
    let ProjectV4Read::Editable(decoded) = parse(&a).unwrap() else {
        panic!()
    };
    assert_eq!(json_project(*decoded), a);
}

#[test]
fn modular_versions_and_strict_fields_do_not_fall_back() {
    let mut project = fixture();
    project.steady_state_plan = plan();
    let original: Value = serde_json::from_str(&json_project(project)).unwrap();
    for field in ["acceptanceContractVersion", "responseContractVersion"] {
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
        "\"responseContractVersion\":1",
        "\"responseContractVersion\":1,\"responseContractVersion\":1",
    );
    assert!(parse(&text).is_err());

    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut value = scale("s");
        value.variable_mol_s = bad;
        let p = StoredFlowPlan::Simultaneous(StoredModularPlan {
            fixed_boundary: StoredFixedBoundary {
                temperature_k: 300.0,
                pressure_pa: 650_000.0,
            },
            scaling: StoredCouplingScaling::ExplicitCouplings {
                values: vec![value],
            },
        });
        assert!(serde_json::to_string(&p).is_err());
    }
    let StoredFlowPlan::Simultaneous(mut o) = plan() else {
        unreachable!()
    };
    o.scaling = StoredCouplingScaling::ExplicitCouplings {
        values: vec![scale("s"), scale("s")],
    };
    assert!(serde_json::to_string(&StoredFlowPlan::Simultaneous(o)).is_err());
}

#[test]
fn modular_capability_upgrade_and_target_fingerprint_are_enforced_by_writer() {
    let path =
        std::env::temp_dir().join(format!("rf-store-qs2-target-{}.json", std::process::id()));
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
