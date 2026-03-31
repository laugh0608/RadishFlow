use std::fs;
use std::path::PathBuf;

use rf_thermo::{
    AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoProvider, ThermoState,
    ThermoSystem,
};
use rf_types::ComponentId;
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
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

fn load_case(file_name: &str) -> ThermoGoldenCase {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/thermo-golden")
        .join(file_name);
    let json = fs::read_to_string(&path).expect("expected thermo golden file");
    serde_json::from_str(&json).expect("expected thermo golden json")
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
            runtime
        })
        .collect();

    PlaceholderThermoProvider::new(ThermoSystem::new(components))
}

#[test]
fn binary_hydrocarbon_lite_case_matches_expected_k_values() {
    let case = load_case("binary-hydrocarbon-lite-v1-300k-650kpa.json");
    let provider = build_provider(&case);
    let state = ThermoState::new(
        case.temperature_k,
        case.pressure_pa,
        case.overall_mole_fractions.clone(),
    );

    assert_eq!(
        provider.system().component_count(),
        case.expected_saturation_pressure_pa.len(),
        "unexpected component count for golden case `{}`",
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
}
