use super::*;

const M1: &str = include_str!(
    "../../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
);

fn mixed_recycle(initialization: StoredTearInitialization) -> ProjectV4Read {
    let ProjectV4Read::Editable(mut project) = parse(M1).unwrap() else {
        panic!("M1 fixture must be editable");
    };
    let StoredFlowPlan::Mixed(mut mixed) = project.steady_state_plan else {
        panic!("fixture must contain M1");
    };
    let equations = mixed
        .blocks
        .iter_mut()
        .find(|b| b.id == "equations")
        .unwrap();
    equations.members = vec!["mixer-1".into(), "splitter-1".into()];
    for id in ["mixer-2", "splitter-2"] {
        mixed.blocks.push(StoredMixedBlock {
            id: id.into(),
            members: vec![id.into()],
            method: StoredMixedBlockMethod::Local,
        });
    }
    project.steady_state_plan = StoredFlowPlan::MixedRecycle(Box::new(StoredMixedRecyclePlan {
        mixed,
        max_iteration_evaluations: 500,
        tear: StoredTear {
            stream_id: "s-rba".into(),
            source: StoredPortBinding {
                unit_id: "splitter-2".into(),
                port: "outlet_a".into(),
            },
            sink: StoredPortBinding {
                unit_id: "mixer-1".into(),
                port: "inlet_b".into(),
            },
            initialization,
            residual_scaling: StoredResidualScaling::EvaluatedFreshTotal,
        },
    }));
    ProjectV4Read::Editable(project)
}

fn explicit_initialization() -> StoredTearInitialization {
    StoredTearInitialization::ComponentFlows {
        values_mol_s: vec![
            StoredComponentValue {
                component_id: "ethane".into(),
                value: 1.0,
            },
            StoredComponentValue {
                component_id: "methane".into(),
                value: 10.0,
            },
        ],
    }
}

#[test]
fn mixed_recycle_v4_roundtrips_and_is_read_only_to_default_readers() {
    let support = FlowPlanSupport::MixedRecycle;
    let m1 = parse(M1).unwrap();
    assert_eq!(parse_with_support(M1, support).unwrap(), m1);
    assert_eq!(
        to_pretty_json_with_support(&m1, support).unwrap(),
        to_pretty_json(&m1).unwrap()
    );
    for initial in [
        StoredTearInitialization::ZeroComponentFlows,
        explicit_initialization(),
    ] {
        let mut project = mixed_recycle(initial);
        assert!(to_pretty_json(&project).is_err());
        let canonical = to_pretty_json_with_support(&project, support).unwrap();
        let original: Value = serde_json::from_str(&canonical).unwrap();
        assert_eq!(original["schemaVersion"], 4);
        assert_eq!(
            original["requiredCapabilities"],
            json!([
                {"id": FLOW_PLAN_CAPABILITY, "version": 1},
                {"id": MIXED_RECYCLE_CAPABILITY, "version": 1},
                {"id": SPLITTER_CAPABILITY, "version": 1},
            ])
        );
        let plan = &original["simulation"]["steadyStatePlan"];
        assert_eq!(
            plan["method"],
            json!({"id": MIXED_RECYCLE_METHOD, "version": 1})
        );
        assert_eq!(plan["acceptanceContractVersion"], 1);
        assert_eq!(plan["tears"].as_array().unwrap().len(), 1);
        let reopened = parse_with_support(&canonical, support).unwrap();
        assert_eq!(
            to_pretty_json_with_support(&reopened, support).unwrap(),
            canonical
        );

        let ProjectV4Read::Editable(inputs) = &mut project else {
            unreachable!()
        };
        let StoredFlowPlan::MixedRecycle(plan) = &mut inputs.steady_state_plan else {
            unreachable!()
        };
        plan.mixed.blocks.reverse();
        for block in &mut plan.mixed.blocks {
            block.members.reverse();
        }
        if let StoredTearInitialization::ComponentFlows { values_mol_s } =
            &mut plan.tear.initialization
        {
            values_mol_s.reverse();
        }
        assert_eq!(
            to_pretty_json_with_support(&project, support).unwrap(),
            canonical
        );

        let mut unmarked = original;
        unmarked["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .retain(|entry| entry["id"] != MIXED_RECYCLE_CAPABILITY);
        assert!(parse_with_support(&unmarked.to_string(), support).is_err());
        unmarked["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": MIXED_CAPABILITY, "version": 1}));
        assert!(parse_with_support(&unmarked.to_string(), support).is_err());
        for raw in [canonical, unmarked.to_string()] {
            let ProjectV4Read::ReadOnly(old) = parse(&raw).unwrap() else {
                panic!("default reader must keep the whole M2 project opaque");
            };
            assert_eq!(old.original_json(), raw);
        }
    }
}

#[test]
fn mixed_recycle_distinguishes_malformed_inputs_from_future_contracts() {
    let support = FlowPlanSupport::MixedRecycle;
    let raw =
        to_pretty_json_with_support(&mixed_recycle(explicit_initialization()), support).unwrap();
    let original: Value = serde_json::from_str(&raw).unwrap();
    let plan_at = "/simulation/steadyStatePlan";
    let tear = &original.pointer(plan_at).unwrap()["tears"][0];
    let mut m1_with_recycle_fields: Value = serde_json::from_str(M1).unwrap();
    m1_with_recycle_fields.pointer_mut(plan_at).unwrap()["tears"] = json!([tear]);
    let m1_with_recycle_fields = m1_with_recycle_fields.to_string();
    assert!(parse(&m1_with_recycle_fields).is_err());
    assert!(parse_with_support(&m1_with_recycle_fields, support).is_err());

    for (field, value) in [
        ("maxIterationEvaluations", Value::Null),
        ("maxIterationEvaluations", json!(0)),
        ("maxIterationEvaluations", json!(501)),
        ("maxIterationEvaluations", json!(3.5)),
        ("tears", Value::Null),
        ("tears", json!([])),
        ("tears", json!([tear, tear])),
        ("responseContractVersion", json!(1)),
        ("tears/0/initialization/valuesMolS/0/value", json!(-0.1)),
        (
            "tears/0/initialization/valuesMolS/0/componentId",
            json!("methane"),
        ),
        (
            "tears/0/residualScaling",
            json!({"kind": "component_values", "valuesMolS": []}),
        ),
        ("method/version", json!(0)),
        ("acceptanceContractVersion", json!(0)),
        ("blocks/0/method/equationContractVersion", json!(0)),
    ] {
        let mut modified = original.clone();
        let pointer = format!("{plan_at}/{field}");
        if let Some(slot) = modified.pointer_mut(&pointer) {
            *slot = value;
        } else {
            assert_eq!(field, "responseContractVersion");
            modified.pointer_mut(plan_at).unwrap()[field] = value;
        }
        assert!(
            parse_with_support(&modified.to_string(), support).is_err(),
            "{field}"
        );
    }
    for field in ["maxIterationEvaluations", "tears"] {
        let mut modified = original.clone();
        modified
            .pointer_mut(plan_at)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            parse_with_support(&modified.to_string(), support).is_err(),
            "{field}"
        );
    }
    let duplicate = raw.replace(
        "\"maxIterationEvaluations\": 500",
        "\"maxIterationEvaluations\": 2, \"maxIterationEvaluations\": 500",
    );
    assert_ne!(duplicate, raw);
    assert!(parse_with_support(&duplicate, support).is_err());
    for pointer in [
        "/requiredCapabilities/1/version",
        "/simulation/steadyStatePlan/method/version",
        "/simulation/steadyStatePlan/acceptanceContractVersion",
        "/simulation/steadyStatePlan/blocks/0/method/equationContractVersion",
    ] {
        let mut modified = original.clone();
        *modified.pointer_mut(pointer).unwrap() = json!(2);
        let raw = modified.to_string();
        let ProjectV4Read::ReadOnly(read) = parse_with_support(&raw, support).unwrap() else {
            panic!("future contract must be read only: {pointer}");
        };
        assert_eq!(read.original_json(), raw);
    }
}
