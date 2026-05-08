use std::fs;
use std::path::PathBuf;

use rf_thermo::{
    AntoineCoefficients, BubbleDewPressureInput, BubbleDewTemperatureInput, PhaseThermoState,
    PlaceholderThermoProvider, ThermoComponent, ThermoProvider, ThermoState, ThermoSystem,
};
use rf_types::{
    ComponentId, PhaseEquilibriumRegion, PhaseLabel, phase_equilibrium_region_from_pressure,
    phase_equilibrium_region_from_temperature,
};
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
struct ThermoGoldenCase {
    name: String,
    temperature_k: f64,
    pressure_pa: f64,
    overall_mole_fractions: Vec<f64>,
    components: Vec<GoldenComponent>,
    expected_saturation_pressure_pa: Vec<f64>,
    expected_k_values: Vec<f64>,
    expected_phase_region: GoldenPhaseRegion,
    expected_bubble_pressure_pa: f64,
    expected_dew_pressure_pa: f64,
    expected_bubble_temperature_k: f64,
    expected_dew_temperature_k: f64,
    expected_liquid_molar_enthalpy_j_per_mol: f64,
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

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/thermo-golden")
}

fn load_cases() -> Vec<(String, ThermoGoldenCase)> {
    let mut paths = fs::read_dir(golden_dir())
        .expect("expected thermo golden dir")
        .map(|entry| entry.expect("expected thermo golden dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("expected thermo golden file name")
                .to_string();
            let json = fs::read_to_string(&path).expect("expected thermo golden file");
            let case = serde_json::from_str(&json).expect("expected thermo golden json");
            (file_name, case)
        })
        .collect()
}

fn build_provider(case: &ThermoGoldenCase) -> PlaceholderThermoProvider {
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
fn thermo_golden_cases_match_expected_results() {
    for (file_name, case) in load_cases() {
        let provider = build_provider(&case);
        let state = ThermoState::new(
            case.temperature_k,
            case.pressure_pa,
            case.overall_mole_fractions.clone(),
        );

        assert_eq!(
            provider.system().component_count(),
            case.expected_saturation_pressure_pa.len(),
            "unexpected component count for golden case `{}` in `{file_name}`",
            case.name
        );

        for (component, expected_pressure) in provider
            .system()
            .components
            .iter()
            .zip(case.expected_saturation_pressure_pa.iter())
        {
            let actual_pressure = component
                .saturation_pressure_pa(case.temperature_k)
                .expect("expected saturation pressure");
            assert_close(actual_pressure, *expected_pressure, 1e-6);
        }

        let k_values = provider
            .estimate_k_values(&state)
            .expect("expected K-value estimation");
        assert_eq!(k_values.len(), case.expected_k_values.len());

        for (actual, expected) in k_values.iter().zip(case.expected_k_values.iter()) {
            assert_close(*actual, *expected, 1e-12);
        }

        let pressures = provider
            .estimate_bubble_dew_pressures(&BubbleDewPressureInput::new(
                case.temperature_k,
                case.overall_mole_fractions.clone(),
            ))
            .expect("expected bubble/dew pressures");
        assert_close(
            pressures.bubble_pressure_pa,
            case.expected_bubble_pressure_pa,
            1e-6,
        );
        assert_close(
            pressures.dew_pressure_pa,
            case.expected_dew_pressure_pa,
            1e-6,
        );

        let temperatures = provider
            .estimate_bubble_dew_temperatures(&BubbleDewTemperatureInput::new(
                case.pressure_pa,
                case.overall_mole_fractions.clone(),
            ))
            .expect("expected bubble/dew temperatures");
        assert_close(
            temperatures.bubble_temperature_k,
            case.expected_bubble_temperature_k,
            1e-4,
        );
        assert_close(
            temperatures.dew_temperature_k,
            case.expected_dew_temperature_k,
            1e-4,
        );

        let expected_phase_region = match case.expected_phase_region {
            GoldenPhaseRegion::LiquidOnly => PhaseEquilibriumRegion::LiquidOnly,
            GoldenPhaseRegion::TwoPhase => PhaseEquilibriumRegion::TwoPhase,
            GoldenPhaseRegion::VaporOnly => PhaseEquilibriumRegion::VaporOnly,
        };
        assert_eq!(
            phase_equilibrium_region_from_pressure(
                case.pressure_pa,
                pressures.bubble_pressure_pa,
                pressures.dew_pressure_pa,
            ),
            expected_phase_region
        );
        assert_eq!(
            phase_equilibrium_region_from_temperature(
                case.temperature_k,
                temperatures.bubble_temperature_k,
                temperatures.dew_temperature_k,
            ),
            expected_phase_region
        );

        let liquid_enthalpy = provider
            .phase_molar_enthalpy(&PhaseThermoState::new(
                PhaseLabel::Liquid,
                case.temperature_k,
                case.pressure_pa,
                case.overall_mole_fractions.clone(),
            ))
            .expect("expected liquid enthalpy");
        let vapor_enthalpy = provider
            .phase_molar_enthalpy(&PhaseThermoState::new(
                PhaseLabel::Vapor,
                case.temperature_k,
                case.pressure_pa,
                case.overall_mole_fractions.clone(),
            ))
            .expect("expected vapor enthalpy");

        assert_close(
            liquid_enthalpy,
            case.expected_liquid_molar_enthalpy_j_per_mol,
            1e-10,
        );
        assert_close(
            vapor_enthalpy,
            case.expected_vapor_molar_enthalpy_j_per_mol,
            1e-10,
        );
    }
}
