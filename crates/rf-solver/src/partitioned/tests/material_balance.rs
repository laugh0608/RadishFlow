use super::*;
use crate::fixed_tp_material::BalanceCheckError;

#[test]
fn serial_failures_capture_the_rejected_boundary_and_signed_flows() {
    for (feed, scope, inputs, output, inlet, outlet, total_delta) in [
        (
            537793.0,
            MaterialBalanceScope::Unit {
                block_index: 5,
                unit_id: "h1".into(),
            },
            vec!["s-product-2"],
            "s-product",
            430234.79999999993,
            430234.80000000005,
            0.0,
        ),
        (
            180029.0,
            MaterialBalanceScope::RecycleRegion {
                block_index: 3,
                tear_stream_id: "s-recycle".into(),
            },
            vec!["s-fresh"],
            "s-bleed",
            144023.6,
            144023.5999999999,
            -8.731149137020111e-11,
        ),
        (
            158822.0,
            MaterialBalanceScope::Flowsheet,
            vec!["s-fa", "s-fb"],
            "s-product",
            127058.0,
            127057.9999999999,
            -1.1641532182693481e-10,
        ),
    ] {
        let (mut flow, options) = serial_case(0.25);
        flow.streams
            .get_mut(&StreamId::new("s-fb"))
            .unwrap()
            .total_molar_flow_mol_s = feed;
        let failure = solve(&flow, &options).unwrap_err();
        assert_eq!(failure.kind, PartitionedFailureKind::Balance);
        let evidence = failure.material_balance.unwrap();
        assert_eq!(evidence.scope, scope);
        let r = evidence.readings;
        assert_eq!(
            r.inputs.iter().map(StreamId::as_str).collect::<Vec<_>>(),
            inputs
        );
        assert_eq!(r.outputs, [StreamId::new(output)]);
        assert_eq!(r.total.difference_mol_s, total_delta);
        assert!(!r.total_exceeded);
        assert_eq!(r.exceeded_components.len(), 1);
        let c = &r.exceeded_components[0];
        assert_eq!(c.component_id, ComponentId::new("ethane"));
        assert_eq!(c.flow.input_mol_s, inlet);
        assert_eq!(c.flow.output_mol_s, outlet);
        assert_eq!(c.flow.difference_mol_s, outlet - inlet);
        assert_eq!(c.scale_mol_s, feed + 2.0);
        assert_eq!(c.scaled_absolute, (outlet - inlet).abs() / c.scale_mol_s);
        assert!(c.absolute_exceeded && !c.scaled_exceeded);
        assert_eq!(r.tolerances, RecycleTolerances::for_components(2));
    }
}

#[test]
fn violations_distinguish_limits_and_earlier_invalid_residuals() {
    let (flow, options) = case(0.6);
    let plan = assemble_partitioned_plan(&flow, &options).unwrap();
    let zero = stream("zero", 0.0, 1.0, 0.0);
    let check =
        |output, scales| checks::balance(&plan, std::slice::from_ref(&zero), &[output], &scales);
    assert!(
        check(
            stream("equal", ABSOLUTE_FLOW_TOLERANCE, 1.0, 0.0),
            composition(1.0, 1.0)
        )
        .is_ok()
    );
    let Err(BalanceCheckError::Exceeded(r)) =
        check(stream("scaled", 5e-11, 1.0, 0.0), composition(0.1, 1.0))
    else {
        panic!("expected only the methane scaled check to fail")
    };
    let [methane] = r.exceeded_components.as_slice() else {
        panic!("expected one component")
    };
    assert_eq!(methane.component_id.as_str(), "methane");
    assert!(!methane.absolute_exceeded && methane.scaled_exceeded);
    assert!(!r.total_exceeded);
    let Err(BalanceCheckError::Exceeded(r)) = check(
        stream("two", 1.8e-10, 1.0 / 6.0, 5.0 / 6.0),
        composition(0.01, 10.0),
    ) else {
        panic!("expected different components to exceed different thresholds")
    };
    let [ethane, methane] = r.exceeded_components.as_slice() else {
        panic!("expected both components")
    };
    assert_eq!(
        (ethane.component_id.as_str(), methane.component_id.as_str()),
        ("ethane", "methane")
    );
    assert!(ethane.absolute_exceeded && !ethane.scaled_exceeded);
    assert!(!methane.absolute_exceeded && methane.scaled_exceeded);
    // Each side's total overflows, but the accumulated components can remain finite.
    let huge = stream("huge", f64::MAX, 0.5, 0.5);
    let pair = [huge.clone(), huge];
    let scales = composition(1.0, 1.0);
    let Err(BalanceCheckError::Exceeded(r)) = checks::balance(&plan, &pair, &pair, &scales) else {
        panic!("expected nonfinite total")
    };
    assert!(r.total_exceeded && r.exceeded_components.is_empty());
    assert!(r.total.input_mol_s.is_infinite() && r.total.output_mol_s.is_infinite());
    assert!(r.total.difference_mol_s.is_nan());
    let huge = stream("huge", f64::MAX, 1.0, 0.0);
    let pair = [huge.clone(), huge];
    let Err(BalanceCheckError::Invalid(error)) = checks::balance(&plan, &pair, &pair, &scales)
    else {
        panic!("expected invalid component residual")
    };
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.partitioned.nonfinite_residual")
    );
}
