use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_flash::{estimate_bubble_dew_window, PlaceholderTpFlashSolver, TpFlashInput, TpFlashSolver};
use rf_store::{
    property_package_payload_integrity, write_property_package_manifest,
    write_property_package_payload, StoredAntoineCoefficients, StoredAuthCacheIndex,
    StoredCredentialReference, StoredPropertyPackageManifest, StoredPropertyPackagePayload,
    StoredPropertyPackageRecord, StoredPropertyPackageSource, StoredThermoComponent,
};
use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest, PropertyPackageSource, ThermoComponent, ThermoSystem,
};
use rf_types::{ComponentId, PhaseEquilibriumRegion};

pub const BINARY_HYDROCARBON_LITE_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
pub const SYNTHETIC_LIQUID_ONLY_PACKAGE_ID: &str = "binary-hydrocarbon-synthetic-liquid-only-v1";
pub const SYNTHETIC_VAPOR_ONLY_PACKAGE_ID: &str = "binary-hydrocarbon-synthetic-vapor-only-v1";
pub const BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K: f64 = 300.0;
pub const BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA: f64 = 650_000.0;
pub const SYNTHETIC_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K: f64 = 300.0;
pub const SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA: f64 = 100_000.0;
const BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA: f64 = 0.1;
const BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NearBoundaryCaseKind {
    Pressure,
    Temperature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NearBoundaryStreamWindowCase {
    pub package_id: &'static str,
    pub kind: NearBoundaryCaseKind,
    pub label: String,
    pub overall_mole_fractions: [f64; 2],
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub expected_phase_region: PhaseEquilibriumRegion,
    pub expected_bubble_pressure_pa: f64,
    pub expected_dew_pressure_pa: f64,
    pub expected_bubble_temperature_k: f64,
    pub expected_dew_temperature_k: f64,
}

pub fn binary_hydrocarbon_lite_near_boundary_stream_window_cases(
) -> Vec<NearBoundaryStreamWindowCase> {
    let provider = build_binary_hydrocarbon_lite_provider();
    let mut cases = Vec::new();

    for overall_mole_fractions in [[0.195, 0.805], [0.2, 0.8], [0.23, 0.77]] {
        let exact_window = estimate_bubble_dew_window(
            &provider,
            BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
            BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
            overall_mole_fractions.to_vec(),
        )
        .expect("expected exact near-boundary window");
        let composition_label = format!(
            "z=[{}, {}]",
            overall_mole_fractions[0], overall_mole_fractions[1]
        );

        for (boundary_label, pressure_pa, expected_phase_region) in [
            (
                "bubble-boundary - 0.1 Pa",
                exact_window.bubble_pressure_pa
                    - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "bubble-boundary + 0.1 Pa",
                exact_window.bubble_pressure_pa
                    + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::LiquidOnly,
            ),
            (
                "dew-boundary + 0.1 Pa",
                exact_window.dew_pressure_pa
                    + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-boundary - 0.1 Pa",
                exact_window.dew_pressure_pa
                    - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::VaporOnly,
            ),
        ] {
            let expected_window = estimate_bubble_dew_window(
                &provider,
                BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
                pressure_pa,
                overall_mole_fractions.to_vec(),
            )
            .expect("expected pressure-perturbed near-boundary window");
            cases.push(NearBoundaryStreamWindowCase {
                package_id: BINARY_HYDROCARBON_LITE_PACKAGE_ID,
                kind: NearBoundaryCaseKind::Pressure,
                label: format!("{composition_label} {boundary_label}"),
                overall_mole_fractions,
                temperature_k: BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
                pressure_pa,
                expected_phase_region,
                expected_bubble_pressure_pa: exact_window.bubble_pressure_pa,
                expected_dew_pressure_pa: exact_window.dew_pressure_pa,
                expected_bubble_temperature_k: expected_window.bubble_temperature_k,
                expected_dew_temperature_k: expected_window.dew_temperature_k,
            });
        }

        for (boundary_label, temperature_k, expected_phase_region) in [
            (
                "bubble-temperature - 0.001 K",
                exact_window.bubble_temperature_k
                    - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::LiquidOnly,
            ),
            (
                "bubble-temperature + 0.001 K",
                exact_window.bubble_temperature_k
                    + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-temperature - 0.001 K",
                exact_window.dew_temperature_k
                    - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-temperature + 0.001 K",
                exact_window.dew_temperature_k
                    + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::VaporOnly,
            ),
        ] {
            let expected_window = estimate_bubble_dew_window(
                &provider,
                temperature_k,
                BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
                overall_mole_fractions.to_vec(),
            )
            .expect("expected temperature-perturbed near-boundary window");
            cases.push(NearBoundaryStreamWindowCase {
                package_id: BINARY_HYDROCARBON_LITE_PACKAGE_ID,
                kind: NearBoundaryCaseKind::Temperature,
                label: format!("{composition_label} {boundary_label}"),
                overall_mole_fractions,
                temperature_k,
                pressure_pa: BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
                expected_phase_region,
                expected_bubble_pressure_pa: expected_window.bubble_pressure_pa,
                expected_dew_pressure_pa: expected_window.dew_pressure_pa,
                expected_bubble_temperature_k: exact_window.bubble_temperature_k,
                expected_dew_temperature_k: exact_window.dew_temperature_k,
            });
        }
    }

    cases
}

pub fn synthetic_single_phase_near_boundary_stream_window_cases(
) -> Vec<NearBoundaryStreamWindowCase> {
    let mut cases = Vec::new();
    cases.extend(build_synthetic_near_boundary_stream_window_cases(
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID,
        "synthetic liquid-only",
        [0.8, 0.6],
    ));
    cases.extend(build_synthetic_near_boundary_stream_window_cases(
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID,
        "synthetic vapor-only",
        [1.8, 1.3],
    ));
    cases
}

fn build_synthetic_near_boundary_stream_window_cases(
    package_id: &'static str,
    label_prefix: &'static str,
    k_values: [f64; 2],
) -> Vec<NearBoundaryStreamWindowCase> {
    let provider =
        build_synthetic_provider(k_values, SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA);
    let overall_mole_fractions = [0.25, 0.75];
    let exact_window = estimate_bubble_dew_window(
        &provider,
        SYNTHETIC_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
        SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected exact synthetic near-boundary window");
    let mut cases = Vec::new();

    for (boundary_label, pressure_pa, expected_phase_region) in [
        (
            "bubble-boundary - 0.1 Pa",
            exact_window.bubble_pressure_pa
                - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
            PhaseEquilibriumRegion::TwoPhase,
        ),
        (
            "bubble-boundary + 0.1 Pa",
            exact_window.bubble_pressure_pa
                + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
            PhaseEquilibriumRegion::LiquidOnly,
        ),
        (
            "dew-boundary + 0.1 Pa",
            exact_window.dew_pressure_pa + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
            PhaseEquilibriumRegion::TwoPhase,
        ),
        (
            "dew-boundary - 0.1 Pa",
            exact_window.dew_pressure_pa - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
            PhaseEquilibriumRegion::VaporOnly,
        ),
    ] {
        let expected_window = estimate_bubble_dew_window(
            &provider,
            SYNTHETIC_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
            pressure_pa,
            overall_mole_fractions.to_vec(),
        )
        .expect("expected pressure-perturbed synthetic near-boundary window");
        cases.push(NearBoundaryStreamWindowCase {
            package_id,
            kind: NearBoundaryCaseKind::Pressure,
            label: format!("{label_prefix} {boundary_label}"),
            overall_mole_fractions,
            temperature_k: SYNTHETIC_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
            pressure_pa,
            expected_phase_region,
            expected_bubble_pressure_pa: exact_window.bubble_pressure_pa,
            expected_dew_pressure_pa: exact_window.dew_pressure_pa,
            expected_bubble_temperature_k: expected_window.bubble_temperature_k,
            expected_dew_temperature_k: expected_window.dew_temperature_k,
        });
    }

    for (boundary_label, temperature_k, expected_phase_region) in [
        (
            "bubble-temperature - 0.001 K",
            exact_window.bubble_temperature_k
                - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
            PhaseEquilibriumRegion::LiquidOnly,
        ),
        (
            "bubble-temperature + 0.001 K",
            exact_window.bubble_temperature_k
                + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
            PhaseEquilibriumRegion::TwoPhase,
        ),
        (
            "dew-temperature - 0.001 K",
            exact_window.dew_temperature_k
                - BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
            PhaseEquilibriumRegion::TwoPhase,
        ),
        (
            "dew-temperature + 0.001 K",
            exact_window.dew_temperature_k
                + BINARY_HYDROCARBON_LITE_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
            PhaseEquilibriumRegion::VaporOnly,
        ),
    ] {
        let expected_window = estimate_bubble_dew_window(
            &provider,
            temperature_k,
            SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
            overall_mole_fractions.to_vec(),
        )
        .expect("expected temperature-perturbed synthetic near-boundary window");
        cases.push(NearBoundaryStreamWindowCase {
            package_id,
            kind: NearBoundaryCaseKind::Temperature,
            label: format!("{label_prefix} {boundary_label}"),
            overall_mole_fractions,
            temperature_k,
            pressure_pa: SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
            expected_phase_region,
            expected_bubble_pressure_pa: expected_window.bubble_pressure_pa,
            expected_dew_pressure_pa: expected_window.dew_pressure_pa,
            expected_bubble_temperature_k: exact_window.bubble_temperature_k,
            expected_dew_temperature_k: exact_window.dew_temperature_k,
        });
    }

    cases
}

pub fn build_demo_antoine_coefficients(k_value: f64, pressure_pa: f64) -> AntoineCoefficients {
    const TEST_ANTOINE_BOUNDARY_SLOPE: f64 = 300.0;
    const TEST_REFERENCE_TEMPERATURE_K: f64 = 300.0;

    AntoineCoefficients::new(
        ((k_value * pressure_pa) / 1_000.0).ln()
            + TEST_ANTOINE_BOUNDARY_SLOPE / TEST_REFERENCE_TEMPERATURE_K,
        TEST_ANTOINE_BOUNDARY_SLOPE,
        0.0,
    )
}

pub fn build_binary_demo_provider() -> PlaceholderThermoProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(2.0, pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(0.5, pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

fn build_binary_hydrocarbon_lite_provider() -> PlaceholderThermoProvider {
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

fn build_synthetic_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
    PlaceholderThermoProvider::new(build_synthetic_system(k_values, pressure_pa))
}

fn build_synthetic_system(k_values: [f64; 2], pressure_pa: f64) -> ThermoSystem {
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(k_values[0], pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(k_values[1], pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    ThermoSystem::binary([first, second])
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

pub fn expected_overall_molar_enthalpy_for_case(case: &NearBoundaryStreamWindowCase) -> f64 {
    let provider = near_boundary_thermo_provider_for_case(case);
    let flash_solver = PlaceholderTpFlashSolver;
    flash_solver
        .flash(
            &provider,
            &TpFlashInput::new(
                "enthalpy-reference",
                "Enthalpy Reference",
                case.temperature_k,
                case.pressure_pa,
                1.0,
                case.overall_mole_fractions.to_vec(),
            ),
        )
        .expect("expected overall enthalpy reference flash")
        .stream
        .phases
        .iter()
        .find(|phase| phase.label == rf_types::PhaseLabel::Overall)
        .and_then(|phase| phase.molar_enthalpy_j_per_mol)
        .expect("expected overall phase enthalpy")
}

fn near_boundary_thermo_provider_for_case(
    case: &NearBoundaryStreamWindowCase,
) -> PlaceholderThermoProvider {
    match case.package_id {
        BINARY_HYDROCARBON_LITE_PACKAGE_ID => build_binary_hydrocarbon_lite_provider(),
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID => {
            build_synthetic_provider([0.8, 0.6], SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA)
        }
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => {
            build_synthetic_provider([1.8, 1.3], SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA)
        }
        _ => panic!("unexpected near-boundary package id `{}`", case.package_id),
    }
}

pub fn build_binary_demo_package_provider() -> InMemoryPropertyPackageProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(2.0, pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(0.5, pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            BINARY_HYDROCARBON_LITE_PACKAGE_ID,
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
            vec!["component-a".into(), "component-b".into()],
        ),
        ThermoSystem::binary([first, second]),
    )])
}

pub fn build_binary_hydrocarbon_lite_package_provider() -> InMemoryPropertyPackageProvider {
    let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
    methane.antoine = Some(AntoineCoefficients::new(8.987, 659.7, -16.7));
    methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    methane.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
    ethane.antoine = Some(AntoineCoefficients::new(8.952, 699.7, -22.8));
    ethane.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    ethane.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            BINARY_HYDROCARBON_LITE_PACKAGE_ID,
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
            vec!["methane".into(), "ethane".into()],
        ),
        ThermoSystem::binary([methane, ethane]),
    )])
}

pub fn build_synthetic_liquid_only_package_provider() -> InMemoryPropertyPackageProvider {
    build_synthetic_package_provider(SYNTHETIC_LIQUID_ONLY_PACKAGE_ID, [0.8, 0.6])
}

pub fn build_synthetic_vapor_only_package_provider() -> InMemoryPropertyPackageProvider {
    build_synthetic_package_provider(SYNTHETIC_VAPOR_ONLY_PACKAGE_ID, [1.8, 1.3])
}

fn build_synthetic_package_provider(
    package_id: &'static str,
    k_values: [f64; 2],
) -> InMemoryPropertyPackageProvider {
    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            package_id,
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
            vec!["component-a".into(), "component-b".into()],
        ),
        build_synthetic_system(k_values, SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA),
    )])
}

pub fn near_boundary_package_provider_for_case(
    case: &NearBoundaryStreamWindowCase,
) -> InMemoryPropertyPackageProvider {
    match case.package_id {
        BINARY_HYDROCARBON_LITE_PACKAGE_ID => build_binary_hydrocarbon_lite_package_provider(),
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID => build_synthetic_liquid_only_package_provider(),
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => build_synthetic_vapor_only_package_provider(),
        _ => panic!("unexpected near-boundary package id `{}`", case.package_id),
    }
}

pub fn near_boundary_component_ids_for_package(package_id: &str) -> [&'static str; 2] {
    match package_id {
        BINARY_HYDROCARBON_LITE_PACKAGE_ID => ["methane", "ethane"],
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID | SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => {
            ["component-a", "component-b"]
        }
        _ => panic!("unexpected near-boundary package id `{package_id}`"),
    }
}

pub fn unique_temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
}

pub fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

pub fn sample_auth_cache_index(package_ids: &[&str]) -> StoredAuthCacheIndex {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
    );
    index.property_packages = package_ids
        .iter()
        .map(|package_id| {
            let mut record = StoredPropertyPackageRecord::new(
                *package_id,
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                "sha256:test",
                128,
                timestamp(20),
            );
            record.expires_at = Some(timestamp(9_999_999_999));
            record
        })
        .collect();
    index
}

pub fn write_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
) {
    let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
    let first_antoine = build_demo_antoine_coefficients(2.0, 100_000.0);
    first.antoine = Some(StoredAntoineCoefficients::new(
        first_antoine.a,
        first_antoine.b,
        first_antoine.c,
    ));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);
    let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
    let second_antoine = build_demo_antoine_coefficients(0.5, 100_000.0);
    second.antoine = Some(StoredAntoineCoefficients::new(
        second_antoine.a,
        second_antoine.b,
        second_antoine.c,
    ));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    let payload = StoredPropertyPackagePayload::new(package_id, "2026.03.1", vec![first, second]);
    let integrity = property_package_payload_integrity(&payload).expect("expected payload hash");
    let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![
            ComponentId::new("component-a"),
            ComponentId::new("component-b"),
        ],
    );
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = expires_at;
    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        manifest.hash.clone(),
        manifest.size_bytes,
        timestamp(60),
    );
    record.expires_at = expires_at;

    write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(cache_root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    auth_cache_index.property_packages.push(record);
}

pub fn write_binary_hydrocarbon_lite_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
) {
    let mut methane = StoredThermoComponent::new(ComponentId::new("methane"), "Methane");
    methane.antoine = Some(StoredAntoineCoefficients::new(8.987, 659.7, -16.7));
    methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    methane.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut ethane = StoredThermoComponent::new(ComponentId::new("ethane"), "Ethane");
    ethane.antoine = Some(StoredAntoineCoefficients::new(8.952, 699.7, -22.8));
    ethane.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    ethane.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    let payload = StoredPropertyPackagePayload::new(
        "binary-hydrocarbon-lite-v1",
        "2026.03.1",
        vec![methane, ethane],
    );
    let integrity = property_package_payload_integrity(&payload).expect("expected payload hash");
    let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
    let mut manifest = StoredPropertyPackageManifest::new(
        "binary-hydrocarbon-lite-v1",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![ComponentId::new("methane"), ComponentId::new("ethane")],
    );
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = expires_at;
    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        manifest.hash.clone(),
        manifest.size_bytes,
        timestamp(60),
    );
    record.expires_at = expires_at;

    write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(cache_root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    auth_cache_index.property_packages.push(record);
}

pub fn write_near_boundary_cached_package_for_case(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    case: &NearBoundaryStreamWindowCase,
) {
    match case.package_id {
        BINARY_HYDROCARBON_LITE_PACKAGE_ID => {
            write_binary_hydrocarbon_lite_cached_package(cache_root, auth_cache_index)
        }
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID => {
            write_synthetic_liquid_only_cached_package(cache_root, auth_cache_index)
        }
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => {
            write_synthetic_vapor_only_cached_package(cache_root, auth_cache_index)
        }
        _ => panic!("unexpected near-boundary package id `{}`", case.package_id),
    }
}

pub fn write_synthetic_liquid_only_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
) {
    write_synthetic_cached_package(
        cache_root,
        auth_cache_index,
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID,
        [0.8, 0.6],
    );
}

pub fn write_synthetic_vapor_only_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
) {
    write_synthetic_cached_package(
        cache_root,
        auth_cache_index,
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID,
        [1.8, 1.3],
    );
}

fn write_synthetic_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
    k_values: [f64; 2],
) {
    let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
    let first_antoine =
        build_demo_antoine_coefficients(k_values[0], SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA);
    first.antoine = Some(StoredAntoineCoefficients::new(
        first_antoine.a,
        first_antoine.b,
        first_antoine.c,
    ));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
    let second_antoine =
        build_demo_antoine_coefficients(k_values[1], SYNTHETIC_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA);
    second.antoine = Some(StoredAntoineCoefficients::new(
        second_antoine.a,
        second_antoine.b,
        second_antoine.c,
    ));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    let payload = StoredPropertyPackagePayload::new(package_id, "2026.03.1", vec![first, second]);
    let integrity = property_package_payload_integrity(&payload).expect("expected payload hash");
    let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![
            ComponentId::new("component-a"),
            ComponentId::new("component-b"),
        ],
    );
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = expires_at;
    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        manifest.hash.clone(),
        manifest.size_bytes,
        timestamp(60),
    );
    record.expires_at = expires_at;

    write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
        .expect("expected manifest write");
    write_property_package_payload(
        record
            .payload_path_under(cache_root)
            .expect("expected payload path"),
        &payload,
    )
    .expect("expected payload write");
    auth_cache_index.property_packages.push(record);
}
