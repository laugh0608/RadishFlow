use super::*;
use crate::test_fixtures::binary_hydrocarbon_provider;
use crate::test_fixtures::cross_recycle as case;
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::Component;
use rf_unitops::{build_mixer_node, build_splitter_node};
mod backend;
mod boundaries;
mod faults;
mod numerical;

fn composition(a: f64, b: f64) -> BTreeMap<ComponentId, f64> {
    [("methane".into(), a), ("ethane".into(), b)].into()
}
fn stream(id: &str, f: f64, a: f64, b: f64) -> MaterialStreamState {
    MaterialStreamState::from_tpzf(id, id, 300.0, 650_000.0, f, composition(a, b))
}
fn options() -> ModularOptions {
    ModularOptions::new(FixedBoundary {
        temperature_k: 300.0,
        pressure_pa: 650_000.0,
    })
}
fn solve_options(f: &Flowsheet, o: &ModularOptions) -> Result<ModularResult, Box<ModularFailure>> {
    SimultaneousModularSolver.solve(
        &crate::SolverServices {
            thermo: &binary_hydrocarbon_provider(),
            flash_solver: &PlaceholderTpFlashSolver,
        },
        f,
        o,
    )
}
fn solve(f: &Flowsheet) -> Result<ModularResult, Box<ModularFailure>> {
    solve_options(f, &options())
}
fn close(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() <= t, "{a} != {b}, tolerance {t}");
}
fn assert_flow(r: &ModularResult, id: &str, a: f64, b: f64, t: f64) {
    let s = &r.streams[&StreamId::new(id)];
    close(
        s.total_molar_flow_mol_s * s.overall_mole_fractions[&ComponentId::new("methane")],
        a,
        t,
    );
    close(
        s.total_molar_flow_mol_s * s.overall_mole_fractions[&ComponentId::new("ethane")],
        b,
        t,
    );
    if a + b > 0.0 {
        close(
            s.overall_mole_fractions[&ComponentId::new("methane")],
            a / (a + b),
            1e-10,
        );
    }
}
