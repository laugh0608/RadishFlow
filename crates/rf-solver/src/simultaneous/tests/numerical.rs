use super::*;

pub(super) fn serial_case(gamma: f64) -> Flowsheet {
    let mut flow = case(0.6, 0.25);
    for id in ["second-circulation", "second-return", "second-product"] {
        flow.insert_stream(stream(id, 99.0, 0.9, 0.1)).unwrap();
    }
    let mut mixer = build_mixer_node("mc", "MC", "pa", "second-return", "second-circulation");
    mixer.parameters.outlet_pressure_pa = Some(650_000.0);
    flow.insert_unit(mixer).unwrap();
    let mut splitter = build_splitter_node(
        "sc",
        "SC",
        "second-circulation",
        "second-return",
        "second-product",
    );
    splitter.parameters.outlet_a_split_fraction = Some(gamma);
    flow.insert_unit(splitter).unwrap();
    flow
}
#[test]
fn serial_coupled_blocks_preserve_boundaries_and_late_block_failure_is_diagnostic_only() {
    let result = solve(&serial_case(0.25)).unwrap();
    assert_eq!(result.blocks.len(), 4);
    assert_flow(&result, "second-return", 14.0 / 51.0, 8.0 / 51.0, 1e-9);
    assert_flow(&result, "second-product", 14.0 / 17.0, 8.0 / 17.0, 1e-9);
    assert_eq!(result.costs.local_unit_calls, 14);
    assert_eq!(result.costs.factorizations, 2);
    assert_eq!(result.costs.triangular_solves, 14);
    let failed = solve(&serial_case(1.0)).unwrap_err();
    assert_eq!(failed.kind, ModularFailureKind::NoFiniteSteadyState);
    assert_eq!(failed.completed_blocks.len(), 3);
    assert!(failed.completed_blocks[2].coupling.is_some());
    assert_eq!(failed.costs.verification_evaluations, 1);
    assert_eq!(failed.costs.local_unit_calls, 10);
}

#[test]
fn cross_recycle_matches_independent_fractions_and_real_call_evidence() {
    let flow = case(0.6, 0.25);
    let frozen = flow.clone();
    let result = solve(&flow).unwrap();
    for (id, a, b) in [
        ("ma-out", 35.0 / 17.0, 20.0 / 17.0),
        ("ab", 21.0 / 17.0, 12.0 / 17.0),
        ("mb-out", 156.0 / 85.0, 264.0 / 85.0),
        ("ba", 39.0 / 85.0, 66.0 / 85.0),
        ("pa", 14.0 / 17.0, 8.0 / 17.0),
        ("pb", 117.0 / 85.0, 198.0 / 85.0),
    ] {
        assert_flow(&result, id, a, b, 1e-9);
    }
    assert_eq!(result.blocks.len(), 3);
    let block = result.blocks.last().unwrap();
    assert_eq!(block.members.len(), 4);
    assert_eq!(block.final_units.len(), 4);
    let evidence = block.coupling.as_ref().unwrap();
    assert_eq!(evidence.connections.len(), 8);
    for c in &evidence.connections {
        assert!(c.residual_mol_s.abs() <= 1e-10 && c.scaled_residual.abs() <= 1e-10);
        let unit = block
            .final_units
            .iter()
            .find(|u| u.unit_id == c.binding.sink.unit_id)
            .unwrap();
        let consumed = unit
            .consumed_streams
            .iter()
            .find(|s| s.id == c.binding.stream)
            .unwrap();
        close(
            consumed.total_molar_flow_mol_s * consumed.overall_mole_fractions[&c.binding.component],
            c.assumed_mol_s,
            1e-14,
        );
        let out = &result.streams[&c.binding.stream];
        assert_eq!(
            c.produced_mol_s,
            out.total_molar_flow_mol_s * out.overall_mole_fractions[&c.binding.component]
        );
    }
    let linear = evidence.linear.as_ref().unwrap();
    close(linear.rcond_indicator.unwrap(), 17.0 / 128.0, 1e-14);
    assert!(linear.backward_error.unwrap() <= 1e-14);
    assert_eq!(result.costs.local_unit_calls, 10);
    assert_eq!(result.costs.operation_constructions, 6);
    assert_eq!(result.costs.anchor_evaluations, 1);
    assert_eq!(result.costs.verification_evaluations, 1);
    assert_eq!(result.costs.response_evaluations, 4);
    assert_eq!(result.costs.factorizations, 1);
    assert_eq!(result.costs.triangular_solves, 9);
    for id in ["ma", "mb", "sa", "sb"] {
        assert_eq!(result.costs.calls_by_unit[&UnitId::new(id)], 2);
    }
    let repeat = solve(&flow).unwrap();
    assert_eq!(repeat.streams, result.streams);
    assert_eq!(repeat.blocks, result.blocks);
    assert_eq!(flow, frozen);
}
#[test]
fn high_recycle_and_complete_explicit_scaling_keep_the_independent_solution() {
    let flow = case(0.99, 0.99);
    let result = solve(&flow).unwrap();
    assert_flow(&result, "ma-out", 21940.0 / 199.0, 27760.0 / 199.0, 1e-8);
    assert_flow(&result, "mb-out", 21840.0 / 199.0, 27960.0 / 199.0, 1e-8);
    assert_flow(&result, "pa", 1097.0 / 995.0, 1388.0 / 995.0, 1e-8);
    assert_flow(&result, "pb", 1092.0 / 995.0, 1398.0 / 995.0, 1e-8);
    close(
        result.blocks[2]
            .coupling
            .as_ref()
            .unwrap()
            .linear
            .as_ref()
            .unwrap()
            .rcond_indicator
            .unwrap(),
        0.0025,
        1e-14,
    );
    let plan = assemble(&flow, &options()).unwrap();
    let mut opts = options();
    opts.scales = Some(
        plan.blocks
            .iter()
            .flat_map(|b| {
                b.couplings.iter().enumerate().map(|(i, c)| {
                    (
                        c.clone(),
                        CouplingScale {
                            variable_mol_s: 2.0 + i as f64,
                            residual_mol_s: 3.0 + i as f64,
                        },
                    )
                })
            })
            .collect(),
    );
    let scaled = solve_options(&flow, &opts).unwrap();
    assert_flow(&scaled, "ma-out", 21940.0 / 199.0, 27760.0 / 199.0, 1e-8);
    assert_eq!(scaled.costs.local_unit_calls, 10);
}
#[test]
fn near_singular_is_rejected_with_typed_condition_evidence() {
    let error = solve(&case(0.999999999999, 0.999999999999)).unwrap_err();
    assert_eq!(error.kind, ModularFailureKind::IllConditioned);
    let linear = error.active_coupling.unwrap().linear.unwrap();
    assert!(linear.rcond_indicator.unwrap() <= 1e-12);
    assert_eq!(linear.backward_error, None);
    assert_eq!(error.costs.verification_evaluations, 0);
    assert_eq!(error.completed_blocks.len(), 2);
}

#[test]
fn scaled_residual_is_required_even_when_absolute_flow_defect_is_small() {
    let flow = case(0.6, 0.25);
    let plan = assemble(&flow, &options()).unwrap();
    let mut opts = options();
    opts.scales = Some(
        plan.blocks[2]
            .couplings
            .iter()
            .cloned()
            .map(|c| {
                (
                    c,
                    CouplingScale {
                        variable_mol_s: 5.0,
                        residual_mol_s: 1e-30,
                    },
                )
            })
            .collect(),
    );
    let error = solve_options(&flow, &opts).unwrap_err();
    assert_eq!(error.kind, ModularFailureKind::Verification);
    let report = error.active_coupling.unwrap();
    assert!(
        report
            .connections
            .iter()
            .all(|c| c.residual_mol_s.abs() <= 1e-10)
    );
    assert!(
        report
            .connections
            .iter()
            .any(|c| c.scaled_residual.abs() > 1e-10)
    );
}
#[test]
fn zero_and_single_full_split_and_sparse_registered_components_are_valid() {
    for (a, b) in [(0.0, 0.25), (0.6, 0.0), (1.0, 0.25), (0.6, 1.0)] {
        let result = solve(&case(a, b)).unwrap();
        let f = [1.6, 0.4];
        let g = [0.6, 2.4];
        let ma = [
            (f[0] + b * g[0]) / (1.0 - a * b),
            (f[1] + b * g[1]) / (1.0 - a * b),
        ];
        assert_flow(&result, "ma-out", ma[0], ma[1], 1e-9);
        let zero = if a == 0.0 {
            Some("ab")
        } else if b == 0.0 {
            Some("ba")
        } else if a == 1.0 {
            Some("pa")
        } else if b == 1.0 {
            Some("pb")
        } else {
            None
        };
        if let Some(id) = zero {
            assert_eq!(
                result.streams[&StreamId::new(id)].total_molar_flow_mol_s,
                0.0
            );
        }
    }
    let mut flow = case(0.6, 0.25);
    for id in ["fa-out", "fb-out"] {
        let s = flow.streams.get_mut(&StreamId::new(id)).unwrap();
        s.overall_mole_fractions = composition(1.0, 0.0);
    }
    let result = solve(&flow).unwrap();
    assert_flow(&result, "ma-out", (2.0 + 0.25 * 3.0) / 0.85, 0.0, 1e-9);
}

#[test]
fn original_qs1_case_is_compared_without_changing_its_algorithm_or_limits() {
    let flow = rf_store::project_v4::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
    )))
    .unwrap();
    let rf_store::project_v4::ProjectV4Read::Editable(project) = flow else {
        panic!("known fixture")
    };
    let graph = &project.document.flowsheet;
    let result = solve(graph).unwrap();
    let recycle = graph.units.values().find(|u| u.kind == "splitter").unwrap();
    let return_id = recycle
        .ports
        .iter()
        .find(|p| p.name == "outlet_a")
        .unwrap()
        .stream_id
        .as_ref()
        .unwrap();
    assert_flow(&result, return_id.as_str(), 3.3, 4.2, 1e-9);
    let old_opts = crate::partitioned::RecycleSolveOptions::new(
        options().boundary,
        vec![crate::partitioned::TearSpecification {
            stream_id: return_id.clone(),
            source: DiagnosticPortTarget::new(recycle.id.clone(), "outlet_a"),
            sink: result
                .blocks
                .iter()
                .find_map(|b| b.coupling.as_ref())
                .unwrap()
                .connections
                .iter()
                .find(|c| c.binding.stream == *return_id)
                .unwrap()
                .binding
                .sink
                .clone(),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        }],
    );
    let provider = binary_hydrocarbon_provider();
    let services = crate::SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let old = crate::partitioned::PartitionedSequentialSolver
        .solve(&services, graph, &old_opts)
        .unwrap();
    assert_eq!(old.costs.iteration_evaluations, 48);
    assert_eq!(old.costs.local_unit_calls, 102);
    assert_eq!(result.costs.local_unit_calls, 8);
    let mut high = graph.clone();
    high.units
        .get_mut(&recycle.id)
        .unwrap()
        .parameters
        .outlet_a_split_fraction = Some(0.99);
    let result = solve(&high).unwrap();
    assert_flow(&result, return_id.as_str(), 217.8, 277.2, 1e-8);
    assert_eq!(
        crate::partitioned::PartitionedSequentialSolver
            .solve(&services, &high, &old_opts)
            .unwrap_err()
            .kind,
        crate::partitioned::PartitionedFailureKind::IterationLimit
    );
}
