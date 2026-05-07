use std::fs;
use std::path::PathBuf;

use rf_flash::{
    FlashPhaseRegion, FlashStatus, PlaceholderTpFlashSolver, TpFlashInput, TpFlashSolver,
};
use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, PhaseLabel};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenAntoine {
    a: f64,
    b: f64,
    c: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenComponent {
    id: String,
    name: String,
    antoine: GoldenAntoine,
    liquid_heat_capacity_j_per_mol_k: f64,
    vapor_heat_capacity_j_per_mol_k: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlashGoldenCase {
    name: String,
    temperature_k: f64,
    pressure_pa: f64,
    total_molar_flow_mol_s: f64,
    overall_mole_fractions: Vec<f64>,
    components: Vec<GoldenComponent>,
    expected_k_values: Vec<f64>,
    expected_phase_region: GoldenPhaseRegion,
    expected_bubble_pressure_pa: f64,
    expected_dew_pressure_pa: f64,
    expected_vapor_fraction: f64,
    expected_overall_molar_enthalpy_j_per_mol: f64,
    expected_liquid_mole_fractions: Vec<f64>,
    expected_liquid_molar_enthalpy_j_per_mol: f64,
    expected_vapor_mole_fractions: Vec<f64>,
    expected_vapor_molar_enthalpy_j_per_mol: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum GoldenPhaseRegion {
    LiquidOnly,
    TwoPhase,
    VaporOnly,
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

fn load_case(file_name: &str) -> FlashGoldenCase {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/flash-golden")
        .join(file_name);
    let json = fs::read_to_string(&path).expect("expected flash golden file");
    serde_json::from_str(&json).expect("expected flash golden json")
}

fn build_provider(case: &FlashGoldenCase) -> PlaceholderThermoProvider {
    let components = case
        .components
        .iter()
        .map(|component| {
            let mut runtime = ThermoComponent::new(
                ComponentId::new(component.id.clone()),
                component.name.clone(),
            );
            runtime.antoine = Some(AntoineCoefficients::new(
                component.antoine.a,
                component.antoine.b,
                component.antoine.c,
            ));
            runtime.liquid_heat_capacity_j_per_mol_k =
                Some(component.liquid_heat_capacity_j_per_mol_k);
            runtime.vapor_heat_capacity_j_per_mol_k =
                Some(component.vapor_heat_capacity_j_per_mol_k);
            runtime
        })
        .collect();

    PlaceholderThermoProvider::new(ThermoSystem::new(components))
}

#[test]
fn binary_hydrocarbon_lite_flash_case_matches_expected_result() {
    let case = load_case("binary-hydrocarbon-lite-v1-300k-650kpa-z0.2-0.8.json");
    let provider = build_provider(&case);
    let solver = PlaceholderTpFlashSolver;
    let input = TpFlashInput::new(
        "golden-stream",
        case.name.clone(),
        case.temperature_k,
        case.pressure_pa,
        case.total_molar_flow_mol_s,
        case.overall_mole_fractions.clone(),
    );

    let result = solver
        .flash(&provider, &input)
        .expect("expected flash result");

    assert_eq!(result.status, FlashStatus::Converged);
    let expected_phase_region = match case.expected_phase_region {
        GoldenPhaseRegion::LiquidOnly => FlashPhaseRegion::LiquidOnly,
        GoldenPhaseRegion::TwoPhase => FlashPhaseRegion::TwoPhase,
        GoldenPhaseRegion::VaporOnly => FlashPhaseRegion::VaporOnly,
    };
    assert_eq!(result.phase_region, expected_phase_region);
    assert_close(
        result.bubble_pressure_pa,
        case.expected_bubble_pressure_pa,
        1e-6,
    );
    assert_close(result.dew_pressure_pa, case.expected_dew_pressure_pa, 1e-6);
    assert_close(
        result.vapor_fraction.expect("expected vapor fraction"),
        case.expected_vapor_fraction,
        1e-10,
    );

    let actual_k_values = result.k_values.expect("expected K-values");
    assert_eq!(actual_k_values.len(), case.expected_k_values.len());
    for (actual, expected) in actual_k_values.iter().zip(case.expected_k_values.iter()) {
        assert_close(*actual, *expected, 1e-10);
    }

    let liquid = result
        .stream
        .phases
        .iter()
        .find(|phase| phase.label == PhaseLabel::Liquid)
        .expect("expected liquid phase");
    let vapor = result
        .stream
        .phases
        .iter()
        .find(|phase| phase.label == PhaseLabel::Vapor)
        .expect("expected vapor phase");

    assert_close(
        result.stream.phases[0]
            .molar_enthalpy_j_per_mol
            .expect("expected overall enthalpy"),
        case.expected_overall_molar_enthalpy_j_per_mol,
        1e-10,
    );
    assert_close(
        liquid
            .molar_enthalpy_j_per_mol
            .expect("expected liquid enthalpy"),
        case.expected_liquid_molar_enthalpy_j_per_mol,
        1e-10,
    );
    assert_close(
        vapor
            .molar_enthalpy_j_per_mol
            .expect("expected vapor enthalpy"),
        case.expected_vapor_molar_enthalpy_j_per_mol,
        1e-10,
    );

    for (component, expected) in case
        .components
        .iter()
        .zip(case.expected_liquid_mole_fractions.iter())
    {
        let actual = liquid
            .mole_fractions
            .get(&ComponentId::new(component.id.clone()))
            .expect("expected liquid component");
        assert_close(*actual, *expected, 1e-10);
    }

    for (component, expected) in case
        .components
        .iter()
        .zip(case.expected_vapor_mole_fractions.iter())
    {
        let actual = vapor
            .mole_fractions
            .get(&ComponentId::new(component.id.clone()))
            .expect("expected vapor component");
        assert_close(*actual, *expected, 1e-10);
    }
}
