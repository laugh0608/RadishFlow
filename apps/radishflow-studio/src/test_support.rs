use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_flash::estimate_bubble_dew_window;
use rf_store::{
    StoredAuthCacheIndex, StoredProjectFile, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    parse_project_file_json, project_file_to_pretty_json, property_package_payload_integrity,
    write_property_package_manifest, write_property_package_payload,
};
use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest as ThermoPropertyPackageManifest,
    PropertyPackageSource as ThermoPropertyPackageSource, ThermoComponent, ThermoSystem,
};
use rf_types::{ComponentId, PhaseEquilibriumRegion, StreamId, UnitId};

#[doc(hidden)]
pub const OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS: [(&str, &str); 2] =
    [("methane", "Methane"), ("ethane", "Ethane")];
#[doc(hidden)]
pub const OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
#[doc(hidden)]
pub const OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K: f64 = 300.0;
#[doc(hidden)]
pub const OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA: f64 = 650_000.0;
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_A_ID: &str = "synthetic-component-a";
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_B_ID: &str = "synthetic-component-b";
#[doc(hidden)]
pub const SYNTHETIC_COMPONENT_C_ID: &str = "synthetic-component-c";
#[doc(hidden)]
pub const SYNTHETIC_BINARY_COMPONENT_SPECS: [(&str, &str); 2] = [
    (SYNTHETIC_COMPONENT_A_ID, "Synthetic Component A"),
    (SYNTHETIC_COMPONENT_B_ID, "Synthetic Component B"),
];
const OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_PRESSURE_DELTA_PA: f64 = 0.1;
const OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_TEMPERATURE_DELTA_K: f64 = 0.001;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialBinaryHydrocarbonNearBoundaryCaseKind {
    Pressure,
    Temperature,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct OfficialBinaryHydrocarbonNearBoundaryCase {
    pub kind: OfficialBinaryHydrocarbonNearBoundaryCaseKind,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfficialBinaryHydrocarbonNearBoundaryPath {
    Heater,
    Cooler,
    Valve,
    Mixer,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OfficialBinaryHydrocarbonNearBoundaryConsumerScenario {
    path: OfficialBinaryHydrocarbonNearBoundaryPath,
    pub project_json: &'static str,
    pub source_stream_ids: &'static [&'static str],
    pub flash_inlet_stream_id: &'static str,
    pub flash_inlet_title: &'static str,
    pub case: OfficialBinaryHydrocarbonNearBoundaryCase,
}
#[doc(hidden)]
pub const OFFICIAL_HEATER_BINARY_HYDROCARBON_PROJECT_FILE_NAME: &str =
    "feed-heater-flash-binary-hydrocarbon.rfproj.json";
#[doc(hidden)]
pub const OFFICIAL_VALVE_BINARY_HYDROCARBON_PROJECT_FILE_NAME: &str =
    "feed-valve-flash-binary-hydrocarbon.rfproj.json";
#[doc(hidden)]
pub const OFFICIAL_HEATER_BINARY_HYDROCARBON_AUTORUN_SNAPSHOT_ID: &str =
    "example-feed-heater-flash-binary-hydrocarbon-rev-1-seq-1";

fn fixture_timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[doc(hidden)]
pub fn official_heater_binary_hydrocarbon_project_json() -> &'static str {
    include_str!("../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json")
}

#[doc(hidden)]
pub fn official_valve_binary_hydrocarbon_project_json() -> &'static str {
    include_str!("../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json")
}

#[doc(hidden)]
pub fn build_flash_drum_local_rules_project_json() -> String {
    let mut project = official_heater_binary_hydrocarbon_project_file();
    disconnect_project_unit_port(&mut project, "flash-1", "inlet");
    disconnect_project_unit_port(&mut project, "flash-1", "liquid");
    disconnect_project_unit_port(&mut project, "flash-1", "vapor");
    serialize_project_file(&project)
}

#[doc(hidden)]
pub fn build_flash_drum_local_rules_synced_project_json() -> String {
    let mut project = official_heater_binary_hydrocarbon_project_file();
    project
        .document
        .flowsheet
        .streams
        .remove(&StreamId::new("stream-vapor"));
    disconnect_project_unit_port(&mut project, "flash-1", "vapor");
    serialize_project_file(&project)
}

fn official_heater_binary_hydrocarbon_project_file() -> StoredProjectFile {
    parse_project_file_json(official_heater_binary_hydrocarbon_project_json())
        .expect("expected official heater binary hydrocarbon project")
}

fn disconnect_project_unit_port(project: &mut StoredProjectFile, unit_id: &str, port_name: &str) {
    let unit = project
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new(unit_id))
        .expect("expected project unit");
    let port = unit
        .ports
        .iter_mut()
        .find(|port| port.name == port_name)
        .expect("expected project unit port");
    port.stream_id = None;
}

fn serialize_project_file(project: &StoredProjectFile) -> String {
    project_file_to_pretty_json(project).expect("expected project json serialization")
}

#[doc(hidden)]
pub fn build_valve_solver_failure_project_json() -> String {
    official_valve_binary_hydrocarbon_project_json().replacen(
        "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 650000.0,",
        "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 730000.0,",
        1,
    )
}

#[doc(hidden)]
pub fn build_official_binary_hydrocarbon_provider() -> PlaceholderThermoProvider {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let [first, second] = binary_hydrocarbon_lite_components(&payload);

    PlaceholderThermoProvider::new(ThermoSystem::binary([
        build_thermo_component_from_stored_component(first),
        build_thermo_component_from_stored_component(second),
    ]))
}

#[doc(hidden)]
pub fn official_binary_hydrocarbon_near_boundary_stream_window_cases()
-> Vec<OfficialBinaryHydrocarbonNearBoundaryCase> {
    let provider = build_official_binary_hydrocarbon_provider();
    let mut cases = Vec::new();

    for overall_mole_fractions in [[0.195, 0.805], [0.2, 0.8], [0.23, 0.77]] {
        let exact_window = estimate_bubble_dew_window(
            &provider,
            OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
            OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
            overall_mole_fractions.to_vec(),
        )
        .expect("expected official near-boundary window");
        let composition_label = format!(
            "z=[{}, {}]",
            overall_mole_fractions[0], overall_mole_fractions[1]
        );

        for (boundary_label, pressure_pa, expected_phase_region) in [
            (
                "bubble-boundary - 0.1 Pa",
                exact_window.bubble_pressure_pa
                    - OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "bubble-boundary + 0.1 Pa",
                exact_window.bubble_pressure_pa
                    + OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::LiquidOnly,
            ),
            (
                "dew-boundary + 0.1 Pa",
                exact_window.dew_pressure_pa
                    + OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-boundary - 0.1 Pa",
                exact_window.dew_pressure_pa
                    - OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_PRESSURE_DELTA_PA,
                PhaseEquilibriumRegion::VaporOnly,
            ),
        ] {
            let expected_window = estimate_bubble_dew_window(
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
                pressure_pa,
                overall_mole_fractions.to_vec(),
            )
            .expect("expected pressure-perturbed official near-boundary window");
            cases.push(OfficialBinaryHydrocarbonNearBoundaryCase {
                kind: OfficialBinaryHydrocarbonNearBoundaryCaseKind::Pressure,
                label: format!("{composition_label} {boundary_label}"),
                overall_mole_fractions,
                temperature_k: OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_TEMPERATURE_K,
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
                    - OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::LiquidOnly,
            ),
            (
                "bubble-temperature + 0.001 K",
                exact_window.bubble_temperature_k
                    + OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-temperature - 0.001 K",
                exact_window.dew_temperature_k
                    - OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::TwoPhase,
            ),
            (
                "dew-temperature + 0.001 K",
                exact_window.dew_temperature_k
                    + OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_TEMPERATURE_DELTA_K,
                PhaseEquilibriumRegion::VaporOnly,
            ),
        ] {
            let expected_window = estimate_bubble_dew_window(
                &provider,
                temperature_k,
                OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
                overall_mole_fractions.to_vec(),
            )
            .expect("expected temperature-perturbed official near-boundary window");
            cases.push(OfficialBinaryHydrocarbonNearBoundaryCase {
                kind: OfficialBinaryHydrocarbonNearBoundaryCaseKind::Temperature,
                label: format!("{composition_label} {boundary_label}"),
                overall_mole_fractions,
                temperature_k,
                pressure_pa: OFFICIAL_BINARY_HYDROCARBON_NEAR_BOUNDARY_REFERENCE_PRESSURE_PA,
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

fn official_binary_hydrocarbon_near_boundary_case(
    kind: OfficialBinaryHydrocarbonNearBoundaryCaseKind,
    expected_phase_region: PhaseEquilibriumRegion,
    overall_mole_fractions: [f64; 2],
) -> OfficialBinaryHydrocarbonNearBoundaryCase {
    official_binary_hydrocarbon_near_boundary_stream_window_cases()
        .into_iter()
        .find(|case| {
            case.kind == kind
                && case.expected_phase_region == expected_phase_region
                && case.overall_mole_fractions == overall_mole_fractions
        })
        .expect("expected official near-boundary case")
}

#[doc(hidden)]
pub fn official_binary_hydrocarbon_near_boundary_consumer_scenarios()
-> Vec<OfficialBinaryHydrocarbonNearBoundaryConsumerScenario> {
    vec![
        OfficialBinaryHydrocarbonNearBoundaryConsumerScenario {
            path: OfficialBinaryHydrocarbonNearBoundaryPath::Heater,
            project_json: include_str!(
                "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-heated",
            flash_inlet_title: "Heated Outlet",
            case: official_binary_hydrocarbon_near_boundary_case(
                OfficialBinaryHydrocarbonNearBoundaryCaseKind::Temperature,
                PhaseEquilibriumRegion::VaporOnly,
                [0.23, 0.77],
            ),
        },
        OfficialBinaryHydrocarbonNearBoundaryConsumerScenario {
            path: OfficialBinaryHydrocarbonNearBoundaryPath::Cooler,
            project_json: include_str!(
                "../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-cooled",
            flash_inlet_title: "Cooled Outlet",
            case: official_binary_hydrocarbon_near_boundary_case(
                OfficialBinaryHydrocarbonNearBoundaryCaseKind::Temperature,
                PhaseEquilibriumRegion::TwoPhase,
                [0.2, 0.8],
            ),
        },
        OfficialBinaryHydrocarbonNearBoundaryConsumerScenario {
            path: OfficialBinaryHydrocarbonNearBoundaryPath::Valve,
            project_json: include_str!(
                "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-throttled",
            flash_inlet_title: "Valve Outlet",
            case: official_binary_hydrocarbon_near_boundary_case(
                OfficialBinaryHydrocarbonNearBoundaryCaseKind::Pressure,
                PhaseEquilibriumRegion::TwoPhase,
                [0.195, 0.805],
            ),
        },
        OfficialBinaryHydrocarbonNearBoundaryConsumerScenario {
            path: OfficialBinaryHydrocarbonNearBoundaryPath::Mixer,
            project_json: include_str!(
                "../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            source_stream_ids: &["stream-feed-a", "stream-feed-b"],
            flash_inlet_stream_id: "stream-mix-out",
            flash_inlet_title: "Mixer Outlet",
            case: official_binary_hydrocarbon_near_boundary_case(
                OfficialBinaryHydrocarbonNearBoundaryCaseKind::Pressure,
                PhaseEquilibriumRegion::LiquidOnly,
                [0.23, 0.77],
            ),
        },
    ]
}

fn build_thermo_component_from_stored_component(
    component: &rf_store::StoredThermoComponent,
) -> ThermoComponent {
    let mut thermo_component = ThermoComponent::new(component.id.clone(), component.name.clone());
    thermo_component.antoine = component.antoine.as_ref().map(|coefficients| {
        AntoineCoefficients::new(coefficients.a, coefficients.b, coefficients.c)
    });
    thermo_component.liquid_heat_capacity_j_per_mol_k = component.liquid_heat_capacity_j_per_mol_k;
    thermo_component.vapor_heat_capacity_j_per_mol_k = component.vapor_heat_capacity_j_per_mol_k;
    thermo_component
}

fn binary_hydrocarbon_lite_components(
    payload: &StoredPropertyPackagePayload,
) -> [&rf_store::StoredThermoComponent; 2] {
    match payload.components.as_slice() {
        [first, second] => [first, second],
        _ => panic!("expected property package payload to stay binary"),
    }
}

#[doc(hidden)]
fn build_stored_payload_from_official_binary_hydrocarbon_sample(
    package_id: &str,
    component_specs: [(&str, &str); 2],
) -> StoredPropertyPackagePayload {
    let mut payload = crate::parse_property_package_download_json(include_str!(
        "../../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json"
    ))
    .expect("expected official sample property package download")
    .to_stored_payload()
    .expect("expected stored payload");
    payload.package_id = package_id.to_string();
    for (component, (component_id, component_name)) in
        payload.components.iter_mut().zip(component_specs)
    {
        component.id = ComponentId::new(component_id);
        component.name = component_name.to_string();
    }
    payload
        .validate()
        .expect("expected mapped stored payload to stay valid");
    payload
}

#[doc(hidden)]
pub fn write_official_binary_hydrocarbon_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    package_id: &str,
    downloaded_at: SystemTime,
    expires_at: Option<SystemTime>,
) {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        package_id,
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let integrity =
        property_package_payload_integrity(&payload).expect("expected payload integrity");
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        &payload.version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        payload.component_ids(),
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
        downloaded_at,
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

#[doc(hidden)]
pub fn write_default_official_binary_hydrocarbon_cached_package(
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
) {
    write_official_binary_hydrocarbon_cached_package(
        cache_root,
        auth_cache_index,
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        fixture_timestamp(60),
        Some(SystemTime::now() + Duration::from_secs(3_600)),
    );
}

#[doc(hidden)]
pub fn build_official_binary_hydrocarbon_in_memory_provider(
    package_id: &str,
) -> InMemoryPropertyPackageProvider {
    let payload = build_stored_payload_from_official_binary_hydrocarbon_sample(
        package_id,
        OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS,
    );
    let [first, second] = binary_hydrocarbon_lite_components(&payload);

    InMemoryPropertyPackageProvider::new(vec![(
        ThermoPropertyPackageManifest::new(
            package_id,
            &payload.version,
            ThermoPropertyPackageSource::LocalBundled,
            payload.component_ids(),
        ),
        ThermoSystem::binary([
            build_thermo_component_from_stored_component(first),
            build_thermo_component_from_stored_component(second),
        ]),
    )])
}

#[doc(hidden)]
pub fn apply_official_binary_hydrocarbon_stream_state_and_composition(
    project: &mut StoredProjectFile,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
    temperature_k: f64,
    pressure_pa: f64,
) {
    let stream = project
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.temperature_k = temperature_k;
    stream.pressure_pa = pressure_pa;
    stream.overall_mole_fractions.clear();
    stream.overall_mole_fractions.insert(
        ComponentId::new(OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS[0].0),
        overall_mole_fractions[0],
    );
    stream.overall_mole_fractions.insert(
        ComponentId::new(OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS[1].0),
        overall_mole_fractions[1],
    );
}

#[doc(hidden)]
pub fn apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(
    project: &mut StoredProjectFile,
    scenario: &OfficialBinaryHydrocarbonNearBoundaryConsumerScenario,
) {
    match scenario.path {
        OfficialBinaryHydrocarbonNearBoundaryPath::Heater => {
            let feed_temperature_k = project
                .document
                .flowsheet
                .streams
                .get(&"stream-feed".into())
                .expect("expected feed stream")
                .temperature_k;
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-feed",
                scenario.case.overall_mole_fractions,
                feed_temperature_k,
                700_000.0,
            );
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-heated",
                scenario.case.overall_mole_fractions,
                scenario.case.temperature_k,
                scenario.case.pressure_pa,
            );
        }
        OfficialBinaryHydrocarbonNearBoundaryPath::Cooler => {
            let feed_temperature_k = project
                .document
                .flowsheet
                .streams
                .get(&"stream-feed".into())
                .expect("expected feed stream")
                .temperature_k;
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-feed",
                scenario.case.overall_mole_fractions,
                feed_temperature_k,
                700_000.0,
            );
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-cooled",
                scenario.case.overall_mole_fractions,
                scenario.case.temperature_k,
                scenario.case.pressure_pa,
            );
        }
        OfficialBinaryHydrocarbonNearBoundaryPath::Valve => {
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-feed",
                scenario.case.overall_mole_fractions,
                scenario.case.temperature_k,
                700_000.0,
            );
            apply_official_binary_hydrocarbon_stream_state_and_composition(
                project,
                "stream-throttled",
                scenario.case.overall_mole_fractions,
                scenario.case.temperature_k,
                scenario.case.pressure_pa,
            );
        }
        OfficialBinaryHydrocarbonNearBoundaryPath::Mixer => {
            for stream_id in ["stream-feed-a", "stream-feed-b"] {
                apply_official_binary_hydrocarbon_stream_state_and_composition(
                    project,
                    stream_id,
                    scenario.case.overall_mole_fractions,
                    scenario.case.temperature_k,
                    scenario.case.pressure_pa,
                );
            }
        }
    }
}

#[doc(hidden)]
pub fn build_synthetic_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_synthetic_provider(k_values, pressure_pa)
}

#[doc(hidden)]
pub fn apply_stream_state_and_composition(
    project: &mut StoredProjectFile,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
    temperature_k: f64,
    pressure_pa: f64,
) {
    crate::studio_gui_window_model::test_support::apply_stream_state_and_composition(
        project,
        stream_id,
        overall_mole_fractions,
        temperature_k,
        pressure_pa,
    )
}

#[doc(hidden)]
pub fn solve_snapshot_model_from_project_with_provider_and_edit<F>(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
    edit_project: F,
) -> crate::StudioGuiWindowSolveSnapshotModel
where
    F: FnOnce(&mut StoredProjectFile),
{
    crate::studio_gui_window_model::test_support::solve_snapshot_model_from_project_with_provider_and_edit(
        project_json,
        provider,
        edit_project,
    )
}

#[doc(hidden)]
pub fn stream_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::stream_target_detail_model(
        snapshot, stream_id, title,
    )
}

#[doc(hidden)]
pub fn unit_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    unit_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::unit_target_detail_model(snapshot, unit_id, title)
}

#[doc(hidden)]
pub fn stream_target_detail_snapshot(
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiInspectorTargetDetailSnapshot {
    crate::studio_gui_window_model::test_support::stream_target_detail_snapshot(stream_id, title)
}
