use super::*;
const RAW: &str = include_str!(
    "../../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
);
#[test]
fn mixed_codec_canonicalizes_and_protects_old_and_future_contracts() {
    let ProjectV4Read::Editable(p) = parse(RAW).unwrap() else {
        panic!()
    };
    let text = json_project(*p.clone());
    assert_eq!(parse(&text).unwrap(), ProjectV4Read::Editable(p.clone()));
    let mut reverse = *p;
    let StoredFlowPlan::Mixed(m) = &mut reverse.steady_state_plan else {
        panic!()
    };
    m.blocks.reverse();
    for b in &mut m.blocks {
        b.members.reverse();
    }
    assert_eq!(json_project(reverse), text);
    for support in [
        FlowPlanSupport::Sequential,
        FlowPlanSupport::Simultaneous,
        FlowPlanSupport::EquationOriented,
    ] {
        for hidden in [false, true] {
            let mut v: Value = serde_json::from_str(RAW).unwrap();
            if hidden {
                v["requiredCapabilities"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|c| c["id"] != MIXED_CAPABILITY);
            }
            let raw = v.to_string();
            let ProjectV4Read::ReadOnly(old) = parse_with_support(&raw, support).unwrap() else {
                panic!()
            };
            assert_eq!(old.original_json(), raw);
            assert!(to_pretty_json(&ProjectV4Read::ReadOnly(old)).is_err());
        }
    }
    for (path, value) in [
        (
            "/simulation/steadyStatePlan/blocks/0/method/equationContractVersion",
            json!(2),
        ),
        (
            "/simulation/steadyStatePlan/blocks/0/method/kind",
            json!("future"),
        ),
        (
            "/simulation/steadyStatePlan/acceptanceContractVersion",
            json!(2),
        ),
    ] {
        let mut v: Value = serde_json::from_str(RAW).unwrap();
        *v.pointer_mut(path).unwrap() = value;
        assert!(
            matches!(parse(&v.to_string()).unwrap(), ProjectV4Read::ReadOnly(_)),
            "{path}"
        );
    }
}
#[test]
fn mixed_rejects_duplicate_identity_null_contract_bad_scale_and_false_capability() {
    let original: Value = serde_json::from_str(RAW).unwrap();
    for (path, value) in [
        ("/simulation/steadyStatePlan/blocks", Value::Null),
        (
            "/simulation/steadyStatePlan/blocks/0/method/equationContractVersion",
            json!(0),
        ),
        (
            "/simulation/steadyStatePlan/blocks/0/method/equationContractVersion",
            Value::Null,
        ),
        ("/simulation/steadyStatePlan/blocks/0/id", json!("feed-a")),
        (
            "/simulation/steadyStatePlan/blocks/1/members/0",
            json!("mixer-1"),
        ),
    ] {
        let mut v = original.clone();
        *v.pointer_mut(path).unwrap() = value;
        assert!(parse(&v.to_string()).is_err(), "{path}");
    }
    let mut v = original.clone();
    v["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["id"] != MIXED_CAPABILITY);
    assert!(parse(&v.to_string()).is_err());
    let mut v = original.clone();
    let duplicate = v["simulation"]["steadyStatePlan"]["blocks"][0]["members"][0].clone();
    v["simulation"]["steadyStatePlan"]["blocks"][0]["members"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(parse(&v.to_string()).is_err());
    for scale in [0.0, -1.0] {
        let mut v = original.clone();
        v["simulation"]["steadyStatePlan"]["blocks"][0]["method"]["scaling"] = json!({"kind":"explicit_equations","variables":[],"equations":[{"binding":{"kind":"boundary","streamId":"s-fa","source":{"unitId":"feed-1","port":"outlet"},"sink":{"unitId":"mixer-1","port":"inlet_a"},"componentId":"methane"},"scaleSi":scale}]});
        assert!(parse(&v.to_string()).is_err());
    }
}
