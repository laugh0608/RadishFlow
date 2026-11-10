use super::*;
use rf_thermo::{PlaceholderThermoProvider, ThermoProvider};
#[test]
fn closed_products_zero_fresh_and_invalid_thermodynamic_boundaries_are_distinct() {
    assert_eq!(
        solve(&case(1.0, 1.0)).unwrap_err().kind,
        ModularFailureKind::NoFiniteSteadyState
    );
    let mut f = case(0.6, 0.25);
    f.streams
        .get_mut(&StreamId::new("fa-out"))
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    assert_eq!(
        solve(&f).unwrap_err().kind,
        ModularFailureKind::Initialization
    );
    f.streams
        .get_mut(&StreamId::new("fb-out"))
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    assert_eq!(solve(&f).unwrap_err().costs.local_unit_calls, 0);
    for value in [-1.0, f64::NAN, f64::INFINITY] {
        let mut f = case(0.6, 0.25);
        f.streams
            .get_mut(&StreamId::new("fa-out"))
            .unwrap()
            .total_molar_flow_mol_s = value;
        assert_eq!(solve(&f).unwrap_err().kind, ModularFailureKind::Boundary);
        let mut o = options();
        o.boundary.temperature_k = value;
        assert_eq!(
            solve_options(&case(0.6, 0.25), &o).unwrap_err().kind,
            ModularFailureKind::Assembly
        );
    }
    let mut f = case(0.6, 0.25);
    f.streams
        .get_mut(&StreamId::new("fa-out"))
        .unwrap()
        .pressure_pa = 650_001.0;
    assert_eq!(solve(&f).unwrap_err().kind, ModularFailureKind::Boundary);
}
#[test]
fn exact_domain_identities_specs_and_graph_shape_are_required_before_any_local_call() {
    for mutate in [0, 1, 2, 3, 4, 5, 6] {
        let mut f = case(0.6, 0.25);
        match mutate {
            0 => {
                f.units
                    .get_mut(&UnitId::new("ma"))
                    .unwrap()
                    .parameters
                    .outlet_temperature_k = Some(300.0)
            }
            1 => {
                f.units
                    .get_mut(&UnitId::new("sa"))
                    .unwrap()
                    .parameters
                    .outlet_a_split_fraction = None
            }
            2 => {
                f.units
                    .get_mut(&UnitId::new("sa"))
                    .unwrap()
                    .parameters
                    .outlet_a_split_fraction = Some(f64::NAN)
            }
            3 => f.units.get_mut(&UnitId::new("ma")).unwrap().id = "changed".into(),
            4 => f.units.get_mut(&UnitId::new("ma")).unwrap().ports[0].name = "wrong".into(),
            5 => f.streams.get_mut(&StreamId::new("ba")).unwrap().id = "changed".into(),
            _ => {
                f.components
                    .get_mut(&ComponentId::new("methane"))
                    .unwrap()
                    .id = "changed".into()
            }
        }
        let error = solve(&f).unwrap_err();
        assert_eq!(error.kind, ModularFailureKind::Assembly);
        assert_eq!(error.costs.local_unit_calls, 0);
    }
    let mut f = case(0.6, 0.25);
    f.units.get_mut(&UnitId::new("ma")).unwrap().kind = "heater".into();
    assert!(assemble(&f, &options()).is_err());
    let mut f = case(0.6, 0.25);
    f.remove_stream(&StreamId::new("ba")).unwrap();
    assert!(assemble(&f, &options()).is_err());
}
#[test]
fn capacity_and_complete_scale_maps_are_enforced_without_truncation() {
    let mut f = case(0.6, 0.25);
    for i in 0..30 {
        f.insert_component(Component::new(format!("extra-{i:02}"), "Extra"))
            .unwrap();
    }
    assert_eq!(
        assemble(&f, &options()).unwrap().blocks[2].couplings.len(),
        128
    );
    f.insert_component(Component::new("overflow", "Overflow"))
        .unwrap();
    let e = assemble(&f, &options()).unwrap_err();
    assert_eq!(
        e.context().diagnostic_code(),
        Some("solver.simultaneous.capacity")
    );
    let f = case(0.6, 0.25);
    let plan = assemble(&f, &options()).unwrap();
    let valid: BTreeMap<_, _> = plan.blocks[2]
        .couplings
        .iter()
        .cloned()
        .map(|c| {
            (
                c,
                CouplingScale {
                    variable_mol_s: 5.0,
                    residual_mol_s: 5.0,
                },
            )
        })
        .collect();
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut o = options();
        let mut scales = valid.clone();
        scales.first_entry().unwrap().get_mut().residual_mol_s = value;
        o.scales = Some(scales);
        assert!(assemble(&f, &o).is_err());
    }
    let mut o = options();
    let mut missing = valid.clone();
    missing.pop_first();
    o.scales = Some(missing);
    assert!(assemble(&f, &o).is_err());
    let mut extra = valid;
    let (id, s) = extra.first_key_value().unwrap();
    let mut id = id.clone();
    id.component = "missing".into();
    extra.insert(id, s.clone());
    o.scales = Some(extra);
    assert!(assemble(&f, &o).is_err());
}
#[test]
fn sparse_third_component_is_worker_only_and_thermo_registry_must_match() {
    let mut f = case(0.6, 0.25);
    f.insert_component(Component::new("unused", "Unused"))
        .unwrap();
    let frozen = f.clone();
    assert_eq!(
        solve(&f).unwrap_err().kind,
        ModularFailureKind::Initialization
    );
    let mut system = binary_hydrocarbon_provider().system().clone();
    let mut c = system.components[0].clone();
    c.id = "unused".into();
    system.components.push(c);
    let provider = PlaceholderThermoProvider::new(system);
    let result = SimultaneousModularSolver
        .solve(
            &crate::SolverServices {
                thermo: &provider,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &f,
            &options(),
        )
        .unwrap();
    assert_eq!(
        result.blocks[2]
            .coupling
            .as_ref()
            .unwrap()
            .connections
            .len(),
        12
    );
    for s in result.streams.values() {
        assert_eq!(
            s.overall_mole_fractions.get(&ComponentId::new("unused")),
            Some(&0.0)
        );
    }
    assert_eq!(f, frozen);
}
