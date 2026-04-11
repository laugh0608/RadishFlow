use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest, PropertyPackageSource, ThermoComponent, ThermoSystem,
};
use rf_types::ComponentId;

pub fn build_binary_demo_provider() -> PlaceholderThermoProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(AntoineCoefficients::new(
        ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(AntoineCoefficients::new(
        ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

pub fn build_binary_demo_package_provider() -> InMemoryPropertyPackageProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(AntoineCoefficients::new(
        ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(AntoineCoefficients::new(
        ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
            vec!["component-a".into(), "component-b".into()],
        ),
        ThermoSystem::binary([first, second]),
    )])
}
