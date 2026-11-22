use super::*;

const SUPPORT: FlowPlanSupport = FlowPlanSupport::MixedSerialRecycle;
const REFERENCE: &str = include_str!(
    "../../../../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
);

fn serial_project() -> ProjectV4Read {
    let ProjectV4Read::Editable(mut project) = parse(REFERENCE).unwrap() else {
        panic!("reference project must be editable");
    };
    let mut blocks = [
        "feed-b",
        "feed-a",
        "splitter-recycle-b",
        "splitter-recycle-a",
        "mixer-makeup-b",
    ]
    .map(|id| StoredMixedBlock {
        id: id.into(),
        members: vec![id.into()],
        method: StoredMixedBlockMethod::Local,
    })
    .to_vec();
    for side in ["b", "a"] {
        blocks.push(StoredMixedBlock {
            id: format!("equations-{side}"),
            members: vec![
                format!("splitter-primary-{side}").into(),
                format!("mixer-{side}").into(),
            ],
            method: StoredMixedBlockMethod::EquationOriented {
                equation_contract_version: 1,
                scaling: StoredEquationScaling::EvaluatedFreshTotal,
            },
        });
    }
    let recycle_regions = [("b", 240), ("a", 100)].map(|(side, max_iteration_evaluations)| {
        StoredMixedRecycleRegion {
            tear: StoredMixedSerialRecycleTear {
                stream_id: format!("s-r{side}").into(),
                source: StoredPortBinding {
                    unit_id: format!("splitter-recycle-{side}").into(),
                    port: "outlet_a".into(),
                },
                sink: StoredPortBinding {
                    unit_id: format!("mixer-{side}").into(),
                    port: "inlet_b".into(),
                },
                initialization: if side == "a" {
                    StoredTearInitialization::ZeroComponentFlows
                } else {
                    StoredTearInitialization::ComponentFlows {
                        values_mol_s: vec![
                            StoredComponentValue {
                                component_id: "methane".into(),
                                value: 2.0,
                            },
                            StoredComponentValue {
                                component_id: "ethane".into(),
                                value: 3.0,
                            },
                        ],
                    }
                },
            },
            max_iteration_evaluations,
        }
    });
    project.steady_state_plan =
        StoredFlowPlan::MixedSerialRecycle(Box::new(StoredMixedSerialRecyclePlan {
            mixed: StoredMixedPlan {
                fixed_boundary: StoredFixedBoundary {
                    temperature_k: 300.0,
                    pressure_pa: 650_000.0,
                },
                blocks,
            },
            recycle_regions,
        }));
    ProjectV4Read::Editable(project)
}

fn serial_wire() -> Value {
    serde_json::from_str(&to_pretty_json_with_support(&serial_project(), SUPPORT).unwrap()).unwrap()
}

#[test]
fn m3_wire_is_canonical_and_uses_only_its_own_required_capability() {
    let project = serial_project();
    let canonical = to_pretty_json_with_support(&project, SUPPORT).unwrap();
    let mut wire: Value = serde_json::from_str(&canonical).unwrap();
    assert_eq!(wire["schemaVersion"], 4);
    assert_eq!(
        wire["requiredCapabilities"],
        json!([
            {"id":FLOW_PLAN_CAPABILITY,"version":1},
            {"id":MIXED_SERIAL_RECYCLE_CAPABILITY,"version":1},
            {"id":SPLITTER_CAPABILITY,"version":1}
        ])
    );
    let plan = &mut wire["simulation"]["steadyStatePlan"];
    assert_eq!(
        plan["method"],
        json!({"id":MIXED_SERIAL_RECYCLE_METHOD,"version":1})
    );
    assert_eq!(plan["acceptanceContractVersion"], 1);
    assert_eq!(
        plan["scaling"],
        json!({"kind":"evaluated_region_inlet_and_global_feed_total"})
    );
    assert!(plan.get("tears").is_none() && plan.get("maxIterationEvaluations").is_none());
    let regions = plan["recycleRegions"].as_array_mut().unwrap();
    assert_eq!(regions[0]["tear"]["streamId"], "s-ra");
    assert_eq!(regions[0]["maxIterationEvaluations"], 100);
    assert_eq!(
        regions[0]["tear"]["initialization"]["kind"],
        "zero_component_flows"
    );
    assert_eq!(regions[1]["tear"]["streamId"], "s-rb");
    assert_eq!(regions[1]["maxIterationEvaluations"], 240);
    assert!(
        regions
            .iter()
            .all(|region| region["tear"].get("residualScaling").is_none())
    );
    let initial = regions[1]["tear"]["initialization"]["valuesMolS"]
        .as_array_mut()
        .unwrap();
    assert_eq!(initial[0], json!({"componentId":"ethane","value":3.0}));
    initial.reverse();
    regions.reverse();
    let blocks = plan["blocks"].as_array_mut().unwrap();
    for block in blocks.iter_mut() {
        block["members"].as_array_mut().unwrap().reverse();
    }
    blocks.reverse();
    let loaded = parse_with_support(&wire.to_string(), SUPPORT).unwrap();
    assert_eq!(
        to_pretty_json_with_support(&loaded, SUPPORT).unwrap(),
        canonical
    );
}

#[test]
fn m3_rejects_malformed_known_fields_and_preserves_unknown_contracts() {
    let original = serial_wire();
    let plan_pointer = "/simulation/steadyStatePlan";
    for (field, value) in [
        ("recycleRegions", json!(null)),
        (
            "recycleRegions",
            json!([original
                .pointer(&format!("{plan_pointer}/recycleRegions/0"))
                .unwrap()]),
        ),
        ("recycleRegions/0/maxIterationEvaluations", json!(0)),
        ("recycleRegions/0/maxIterationEvaluations", json!(501)),
        ("recycleRegions/0/maxIterationEvaluations", json!(1.25)),
        ("recycleRegions/1/tear/streamId", json!("s-ra")),
        (
            "recycleRegions/1/tear/source/unitId",
            json!("splitter-recycle-a"),
        ),
        ("recycleRegions/1/tear/sink/unitId", json!("mixer-a")),
        (
            "recycleRegions/1/tear/initialization/valuesMolS/0/value",
            json!(-0.5),
        ),
        (
            "recycleRegions/1/tear/initialization/valuesMolS/0/componentId",
            json!("methane"),
        ),
        ("method/version", json!(0)),
        ("acceptanceContractVersion", json!(0)),
        ("blocks/0/method/equationContractVersion", json!(0)),
        ("scaling/kind", json!("evaluated_fresh_total")),
    ] {
        let mut invalid = original.clone();
        *invalid
            .pointer_mut(&format!("{plan_pointer}/{field}"))
            .unwrap() = value;
        assert!(
            parse_with_support(&invalid.to_string(), SUPPORT).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("tears", json!([])),
        ("maxIterationEvaluations", json!(500)),
        ("unexpected", json!(false)),
    ] {
        let mut invalid = original.clone();
        invalid.pointer_mut(plan_pointer).unwrap()[field] = value;
        assert!(
            parse_with_support(&invalid.to_string(), SUPPORT).is_err(),
            "{field}"
        );
    }
    let mut missing = original.clone();
    missing
        .pointer_mut(plan_pointer)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("recycleRegions");
    assert!(parse_with_support(&missing.to_string(), SUPPORT).is_err());
    let repeated_key = original.to_string().replacen(
        "\"maxIterationEvaluations\":100",
        "\"maxIterationEvaluations\":100,\"maxIterationEvaluations\":100",
        1,
    );
    assert!(parse_with_support(&repeated_key, SUPPORT).is_err());

    for (pointer, value) in [
        ("/requiredCapabilities/1/version".to_owned(), json!(2)),
        (format!("{plan_pointer}/method/version"), json!(2)),
        (
            format!("{plan_pointer}/acceptanceContractVersion"),
            json!(2),
        ),
        (
            format!("{plan_pointer}/blocks/0/method/equationContractVersion"),
            json!(2),
        ),
        (
            format!("{plan_pointer}/blocks/0/method/kind"),
            json!("future_method"),
        ),
    ] {
        let mut future = original.clone();
        *future.pointer_mut(&pointer).unwrap() = value;
        let raw = future.to_string();
        let ProjectV4Read::ReadOnly(read_only) = parse_with_support(&raw, SUPPORT).unwrap() else {
            panic!("{pointer} must preserve the entire future document");
        };
        assert_eq!(read_only.original_json(), raw);
    }
    let mut m1: Value = serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    m1.pointer_mut(plan_pointer).unwrap()["recycleRegions"] =
        original.pointer(plan_pointer).unwrap()["recycleRegions"].clone();
    assert!(parse_with_support(&m1.to_string(), SUPPORT).is_err());
}

#[test]
fn m3_file_requires_explicit_support_and_protects_target_from_downgrade() {
    let dir = Directory::new();
    let target = dir.0.join("target.rfproj.json");
    let copy = dir.0.join("copy.rfproj.json");
    let ProjectV4Read::Editable(downgraded) = parse(REFERENCE).unwrap() else {
        panic!("reference")
    };
    let downgraded = ProjectV4Read::Editable(downgraded);
    let project = serial_project();
    fs::write(&target, REFERENCE).unwrap();
    let review =
        prepare_write_with_support(&target, &project, FileOverwritePolicy::Allow, SUPPORT).unwrap();
    assert!(review.requires_upgrade());
    assert!(write_prepared(&review, false).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), REFERENCE);
    write_prepared(&review, true).unwrap();
    let saved = fs::read_to_string(&target).unwrap();
    let already_m3 =
        prepare_write_with_support(&target, &project, FileOverwritePolicy::Allow, SUPPORT).unwrap();
    assert!(!already_m3.requires_upgrade());
    write_prepared(&already_m3, false).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), saved);

    let mut unmarked: Value = serde_json::from_str(&saved).unwrap();
    unmarked["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap["id"] != MIXED_SERIAL_RECYCLE_CAPABILITY);
    assert!(parse_with_support(&unmarked.to_string(), SUPPORT).is_err());
    let mut mismarked = unmarked.clone();
    mismarked["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":MIXED_RECYCLE_CAPABILITY,"version":1}));
    assert!(parse_with_support(&mismarked.to_string(), SUPPORT).is_err());
    for raw in [saved.clone(), unmarked.to_string()] {
        fs::write(&target, &raw).unwrap();
        let old = read_with_support(&target, FlowPlanSupport::SinglePhaseRecycle).unwrap();
        let ProjectV4Read::ReadOnly(read_only) = &old else {
            panic!("unsupported M3");
        };
        assert_eq!(read_only.original_json(), raw);
        assert!(to_pretty_json_with_support(&old, SUPPORT).is_err());
        assert!(
            prepare_write_with_support(&copy, &old, FileOverwritePolicy::Forbid, SUPPORT).is_err()
        );
        assert!(
            prepare_write_with_support(
                &target,
                &downgraded,
                FileOverwritePolicy::Allow,
                FlowPlanSupport::SinglePhaseRecycle
            )
            .is_err()
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), raw);
        assert!(!copy.exists());
    }
    fs::write(&target, &saved).unwrap();
    let error =
        prepare_write_with_support(&target, &downgraded, FileOverwritePolicy::Allow, SUPPORT)
            .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("store.project.mixed_serial_recycle_target")
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), saved);
    let prepare_copy =
        prepare_write_with_support(&copy, &downgraded, FileOverwritePolicy::Forbid, SUPPORT)
            .unwrap();
    write_prepared(&prepare_copy, false).unwrap();
    assert_eq!(read(&copy).unwrap(), downgraded);
}
