use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::ComponentId;

pub(crate) fn binary_hydrocarbon_provider() -> PlaceholderThermoProvider {
    let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
    methane.antoine = Some(AntoineCoefficients::new(8.987, 659.7, -16.7));
    methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    methane.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
    ethane.antoine = Some(AntoineCoefficients::new(8.952, 699.7, -22.8));
    ethane.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    ethane.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]))
}

use rf_model::{Component, Flowsheet, MaterialStreamState};
use rf_types::UnitId;
use rf_unitops::{build_feed_node, build_mixer_node, build_splitter_node};
use std::collections::BTreeMap;
fn stream(id: &str, f: f64, a: f64, b: f64) -> MaterialStreamState {
    let composition: BTreeMap<ComponentId, f64> =
        [("methane".into(), a), ("ethane".into(), b)].into();
    MaterialStreamState::from_tpzf(id, id, 300.0, 650_000.0, f, composition)
}
pub(crate) fn cross_recycle(a: f64, b: f64) -> Flowsheet {
    let mut f = Flowsheet::new("QS2 cross recycle");
    for c in ["methane", "ethane"] {
        f.insert_component(Component::new(c, c)).unwrap();
    }
    for (id, flow, x, y) in [
        ("fa-out", 2.0, 0.8, 0.2),
        ("fb-out", 3.0, 0.2, 0.8),
        ("ma-out", 97.0, 0.9, 0.1),
        ("mb-out", 83.0, 0.1, 0.9),
        ("ab", 79.0, 0.6, 0.4),
        ("ba", 71.0, 0.4, 0.6),
        ("pa", 91.0, 0.8, 0.2),
        ("pb", 101.0, 0.3, 0.7),
    ] {
        f.insert_stream(stream(id, flow, x, y)).unwrap();
    }
    for mut u in [
        build_feed_node("fa", "FA", "fa-out"),
        build_feed_node("fb", "FB", "fb-out"),
        build_mixer_node("ma", "MA", "fa-out", "ba", "ma-out"),
        build_mixer_node("mb", "MB", "fb-out", "ab", "mb-out"),
        build_splitter_node("sa", "SA", "ma-out", "ab", "pa"),
        build_splitter_node("sb", "SB", "mb-out", "ba", "pb"),
    ] {
        if u.kind == "mixer" {
            u.parameters.outlet_pressure_pa = Some(650_000.0);
        }
        if u.kind == "splitter" {
            u.parameters.outlet_a_split_fraction =
                Some(if u.id == UnitId::new("sa") { a } else { b });
        }
        f.insert_unit(u).unwrap();
    }
    f
}
