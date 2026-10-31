use super::*;
use rf_flash::PlaceholderTpFlashSolver;

#[test]
fn analytic_responses_match_real_run_differences_at_interior_and_zero_boundaries() {
    let components = vec![
        ComponentId::new("component-a"),
        ComponentId::new("component-b"),
    ];
    let thermo = crate::tests::build_provider([1.8, 0.55], 650_000.0);
    let services = UnitOperationServices {
        thermo: Some(&thermo),
        flash_solver: Some(&PlaceholderTpFlashSolver),
    };
    let make_stream = |id: &str, n: &[f64; 2]| {
        let total = n.iter().sum::<f64>();
        let z = if total == 0.0 {
            [0.5, 0.5]
        } else {
            [n[0] / total, n[1] / total]
        };
        MaterialStreamState::from_tpzf(
            id,
            id,
            300.0,
            650_000.0,
            total,
            [(components[0].clone(), z[0]), (components[1].clone(), z[1])].into(),
        )
    };
    for (split, a) in [(false, 0.0), (true, 0.0), (true, 0.6), (true, 1.0)] {
        let mut u = if split {
            build_splitter_node("s", "S", "in", "a", "b")
        } else {
            build_mixer_node("m", "M", "in-a", "in-b", "out")
        };
        if split {
            u.parameters.outlet_a_split_fraction = Some(a);
        } else {
            u.parameters.outlet_pressure_pa = Some(650_000.0);
        }
        let response =
            AffineComponentResponse::for_unit(&u, 300.0, 650_000.0, &components).unwrap();
        let templates = u
            .ports
            .iter()
            .filter_map(|p| p.stream_id.as_ref())
            .map(|id| (id.clone(), make_stream(id.as_str(), &[1.0, 1.0])))
            .collect();
        let op = catalog::instantiate_builtin(&u, &templates).unwrap();
        let evaluate = |n: &[[f64; 2]; 2]| {
            let mut inputs = UnitOperationInputs::new();
            if split {
                inputs.insert_material_stream("inlet", make_stream("in", &n[0]));
            } else {
                inputs.insert_material_stream("inlet_a", make_stream("in-a", &n[0]));
                inputs.insert_material_stream("inlet_b", make_stream("in-b", &n[1]));
            }
            let frozen = inputs.clone();
            let out = op.run(&services, &inputs).unwrap();
            assert_eq!(inputs, frozen);
            out
        };
        for base in [[[1.6, 0.4], [0.6, 2.4]], [[0.0, 0.0], [0.6, 2.4]]] {
            for input in response.inputs() {
                for output in response.outputs() {
                    let idx = usize::from(input.component == components[1]);
                    let port = usize::from(input.port == "inlet_b");
                    let expected = if input.component != output.component {
                        0.0
                    } else if !split {
                        1.0
                    } else if output.port == "outlet_a" {
                        a
                    } else {
                        1.0 - a
                    };
                    assert_eq!(response.derivative(output, input).unwrap(), expected);
                    for h in [5e-3, 5e-4, 5e-5] {
                        let mut plus = base;
                        plus[port][idx] += h;
                        let mut minus = base;
                        let central = base[port][idx] >= h;
                        if central {
                            minus[port][idx] -= h;
                        }
                        let p = evaluate(&plus);
                        let m = evaluate(&minus);
                        let p = p.stream(&output.port).unwrap();
                        let m = m.stream(&output.port).unwrap();
                        let diff = (p.total_molar_flow_mol_s
                            * p.overall_mole_fractions[&output.component]
                            - m.total_molar_flow_mol_s
                                * m.overall_mole_fractions[&output.component])
                            / (if central { 2.0 * h } else { h });
                        assert!((diff - expected).abs() <= 1e-7, "{diff} != {expected}");
                    }
                }
            }
        }
    }
}
#[test]
fn capability_and_domain_rejections_are_explicit() {
    let cs = vec![ComponentId::new("a")];
    let mut m = build_mixer_node("m", "M", "a", "b", "o");
    assert!(AffineComponentResponse::for_unit(&m, 300.0, 650_000.0, &cs).is_err());
    m.parameters.outlet_pressure_pa = Some(650_000.0);
    assert!(AffineComponentResponse::for_unit(&m, 300.0, 650_000.0, &cs).is_ok());
    let response = AffineComponentResponse::for_unit(&m, 300.0, 650_000.0, &cs).unwrap();
    assert!(
        response
            .derivative(
                &ComponentPort {
                    port: "outlet".into(),
                    component: "unknown".into()
                },
                &response.inputs()[0]
            )
            .is_err()
    );
    assert!(
        AffineComponentResponse::for_unit(&m, 300.0, 650_000.0, &[cs[0].clone(), cs[0].clone()])
            .is_err()
    );
    m.parameters.outlet_temperature_k = Some(300.0);
    assert!(AffineComponentResponse::for_unit(&m, 300.0, 650_000.0, &cs).is_err());
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::FlashDrum,
    ] {
        assert_eq!(
            catalog::builtin_model(kind).component_response_capability(),
            None
        );
    }
    let mut s = build_splitter_node("s", "S", "i", "a", "b");
    for alpha in [None, Some(-0.1), Some(1.1), Some(f64::NAN)] {
        s.parameters.outlet_a_split_fraction = alpha;
        assert!(AffineComponentResponse::for_unit(&s, 300.0, 650_000.0, &cs).is_err());
    }
}
