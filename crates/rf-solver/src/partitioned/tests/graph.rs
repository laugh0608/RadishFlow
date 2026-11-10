use super::*;
use rf_unitops::build_flash_drum_node;
#[test]
fn condensation_is_stable_and_keeps_engineering_connections_intact() {
    let (mut flow, options) = case(0.6);
    let original = flow.clone();
    let plan = assemble_partitioned_plan(&flow, &options).unwrap();
    let expected: Vec<Vec<UnitId>> = vec![
        vec!["fa".into()],
        vec!["fb".into()],
        vec!["m0".into()],
        vec!["m1".into(), "s1".into()],
        vec!["h1".into()],
    ];
    assert_eq!(
        plan.blocks
            .iter()
            .map(|b| b.members.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(plan.connections.len(), 7);
    assert_eq!(flow, original);
    flow.units
        .get_mut(&UnitId::new("s1"))
        .unwrap()
        .parameters
        .outlet_a_split_fraction = Some(0.8);
    assert_eq!(plan.flowsheet, original);
    assert_eq!(plan.options, options);
}
fn plan_error(flow: &Flowsheet, options: &RecycleSolveOptions, expected: &str) {
    let error = assemble_partitioned_plan(flow, options).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some(expected),
        "{error:?}"
    );
}
#[test]
fn tear_configuration_errors_retain_distinct_codes_and_domain_bindings() {
    let (flow, original) = case(0.6);
    let mut options = original.clone();
    options.tears.clear();
    plan_error(&flow, &options, "solver.partitioned.missing_tear");
    let mut options = original.clone();
    options.tears.push(options.tears[0].clone());
    plan_error(&flow, &options, "solver.partitioned.duplicate_tear");
    let mut options = original.clone();
    options.tears[0].stream_id = "missing".into();
    plan_error(&flow, &options, "solver.partitioned.missing_tear_stream");
    let error = assemble_partitioned_plan(&flow, &options).unwrap_err();
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("missing")]
    );
    assert_eq!(error.context().related_port_targets().len(), 2);
    let mut options = original.clone();
    options.tears[0].sink.port_name = "inlet_a".into();
    plan_error(&flow, &options, "solver.partitioned.tear_binding");
    let mut options = original.clone();
    options.tears = vec![TearSpecification {
        stream_id: "s-fa".into(),
        source: DiagnosticPortTarget::new("fa", "outlet"),
        sink: DiagnosticPortTarget::new("m0", "inlet_a"),
        ..original.tears[0].clone()
    }];
    plan_error(&flow, &options, "solver.partitioned.outside_tear");
    let mut options = original.clone();
    options.tears = vec![TearSpecification {
        stream_id: "s-circulation".into(),
        source: DiagnosticPortTarget::new("m1", "outlet"),
        sink: DiagnosticPortTarget::new("s1", "inlet"),
        ..original.tears[0].clone()
    }];
    plan_error(&flow, &options, "solver.partitioned.unsupported_recycle");
    options.tears.push(original.tears[0].clone());
    plan_error(&flow, &options, "solver.partitioned.multiple_tears");
    let mut invalid_flow = flow.clone();
    invalid_flow.streams.remove(&StreamId::new("s-recycle"));
    assert!(
        assemble_partitioned_plan(&invalid_flow, &original)
            .unwrap_err()
            .context()
            .diagnostic_code()
            .unwrap()
            .contains("missing_stream_reference")
    );
}
#[test]
fn cutting_one_edge_does_not_claim_to_resolve_an_interwoven_scc() {
    let (mut flow, mut options) = serial_case(0.25);
    flow.units
        .get_mut(&UnitId::new("m1"))
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet_a")
        .unwrap()
        .stream_id = Some("s-product-2".into());
    flow.units
        .get_mut(&UnitId::new("h1"))
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet")
        .unwrap()
        .stream_id = Some("s-fresh".into());
    options.tears.pop();
    plan_error(&flow, &options, "solver.partitioned.remaining_cycle");
}
#[test]
fn legacy_parallel_edges_use_one_unit_dependency_without_dropping_either_stream() {
    let mut flow = Flowsheet::new("parallel phase outlets");
    for id in ["feed", "liquid", "vapor", "product"] {
        flow.insert_stream(stream(id, 1.0, 0.5, 0.5)).unwrap();
    }
    flow.insert_unit(build_feed_node("feed", "F", "feed"))
        .unwrap();
    flow.insert_unit(build_flash_drum_node(
        "flash", "D", "feed", "liquid", "vapor",
    ))
    .unwrap();
    flow.insert_unit(build_mixer_node("mixer", "M", "liquid", "vapor", "product"))
        .unwrap();
    let facts = rf_flowsheet::dependencies::material_dependencies(&flow).unwrap();
    assert_eq!(facts.connections.len(), 4);
    assert_eq!(facts.successors[&UnitId::new("flash")].len(), 1);
    assert_eq!(
        crate::sequential_execution_order(&flow).unwrap(),
        vec![
            UnitId::new("feed"),
            UnitId::new("flash"),
            UnitId::new("mixer")
        ]
    );
}
#[test]
fn old_strategy_retains_two_unit_and_self_loop_rejections() {
    let (flow, _) = case(0.6);
    let error = crate::sequential_execution_order(&flow).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.two_unit_cycle")
    );
    let mut flow = Flowsheet::new("self loop");
    flow.insert_stream(stream("loop", 1.0, 0.5, 0.5)).unwrap();
    flow.insert_unit(build_heater_node("heater", "H", "loop", "loop"))
        .unwrap();
    assert_eq!(
        crate::sequential_execution_order(&flow)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.topological_ordering.self_loop_cycle")
    );
}
#[test]
fn identity_not_names_or_insertion_order_binds_the_loop() {
    let (mut flow, mut options) = case(0.6);
    let mut renamed = BTreeMap::new();
    for (i, (id, mut state)) in flow.streams.into_iter().rev().enumerate() {
        let new_id = StreamId::new(format!("id-{i}"));
        for unit in flow.units.values_mut() {
            for port in &mut unit.ports {
                if port.stream_id.as_ref() == Some(&id) {
                    port.stream_id = Some(new_id.clone());
                }
            }
        }
        if options.tears[0].stream_id == id {
            options.tears[0].stream_id = new_id.clone();
        }
        state.id = new_id.clone();
        state.name = "same display name".into();
        renamed.insert(new_id, state);
    }
    flow.streams = renamed;
    let mut mixer = flow.units.remove(&UnitId::new("m1")).unwrap();
    mixer.id = "z-mixer".into();
    mixer.name = "no semantic hint".into();
    flow.insert_unit(mixer).unwrap();
    options.tears[0].sink.unit_id = "z-mixer".into();
    let result = solve(&flow, &options).unwrap();
    let report = result
        .blocks
        .iter()
        .find_map(|b| b.recycle.as_ref())
        .unwrap();
    close(report.returned.total_molar_flow_mol_s, 7.5, 1e-9);
    assert!(result.blocks[3].members.contains(&UnitId::new("z-mixer")));
}
