use rf_flash::PlaceholderTpFlashSolver;
use rf_rust_integration::{assert_close, build_binary_demo_provider};
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveStatus, SolverServices};
use rf_store::parse_project_file_json;
use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, PhaseEquilibriumRegion, PhaseLabel, StreamId, UnitId};

fn solve_example(project_json: &str) -> rf_solver::SolveSnapshot {
    solve_example_result(project_json).expect("expected solve snapshot")
}

fn solve_example_result(project_json: &str) -> rf_types::RfResult<rf_solver::SolveSnapshot> {
    let provider = build_binary_demo_provider();
    solve_example_result_with_provider(project_json, &provider)
}

fn solve_example_with_provider(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
) -> rf_solver::SolveSnapshot {
    solve_example_result_with_provider(project_json, provider).expect("expected solve snapshot")
}

fn solve_example_result_with_provider(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
) -> rf_types::RfResult<rf_solver::SolveSnapshot> {
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(project_json).expect("expected example project parse");

    SequentialModularSolver.solve(&services, &project.document.flowsheet)
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

fn assert_two_phase_window_spans_solver_stream(
    snapshot: &rf_solver::SolveSnapshot,
    stream_id: &str,
) {
    let stream = snapshot
        .stream(&StreamId::new(stream_id))
        .expect("expected stream");
    let window = stream
        .bubble_dew_window
        .as_ref()
        .expect("expected bubble/dew window");

    assert_eq!(window.phase_region, PhaseEquilibriumRegion::TwoPhase);
    assert!(window.dew_pressure_pa < stream.pressure_pa);
    assert!(window.bubble_pressure_pa > stream.pressure_pa);
    assert!(window.bubble_temperature_k < stream.temperature_k);
    assert!(window.dew_temperature_k > stream.temperature_k);
}

fn assert_flash_outlet_boundary_windows(
    snapshot: &rf_solver::SolveSnapshot,
    liquid_stream_id: &str,
    vapor_stream_id: &str,
) {
    let liquid = snapshot
        .stream(&StreamId::new(liquid_stream_id))
        .expect("expected liquid outlet");
    let liquid_window = liquid
        .bubble_dew_window
        .as_ref()
        .expect("expected liquid outlet bubble/dew window");
    assert_eq!(liquid_window.phase_region, PhaseEquilibriumRegion::TwoPhase);
    assert_close(liquid_window.bubble_pressure_pa, liquid.pressure_pa, 1e-6);
    assert_close(
        liquid_window.bubble_temperature_k,
        liquid.temperature_k,
        1e-4,
    );
    assert!(liquid_window.dew_pressure_pa < liquid_window.bubble_pressure_pa);
    assert!(liquid_window.dew_temperature_k > liquid_window.bubble_temperature_k);

    let vapor = snapshot
        .stream(&StreamId::new(vapor_stream_id))
        .expect("expected vapor outlet");
    let vapor_window = vapor
        .bubble_dew_window
        .as_ref()
        .expect("expected vapor outlet bubble/dew window");
    assert_eq!(vapor_window.phase_region, PhaseEquilibriumRegion::TwoPhase);
    assert_close(vapor_window.dew_pressure_pa, vapor.pressure_pa, 1e-6);
    assert_close(vapor_window.dew_temperature_k, vapor.temperature_k, 1e-4);
    assert!(vapor_window.bubble_pressure_pa > vapor_window.dew_pressure_pa);
    assert!(vapor_window.bubble_temperature_k < vapor_window.dew_temperature_k);
}

fn assert_flash_consumes_stream(snapshot: &rf_solver::SolveSnapshot, stream_id: &str) {
    let flash_step = snapshot.steps.last().expect("expected flash step");

    assert_eq!(flash_step.unit_id, UnitId::new("flash-1"));
    assert_eq!(
        flash_step.consumed_stream_ids,
        vec![StreamId::new(stream_id)]
    );
}

#[test]
fn feed_mixer_flash_project_solves_end_to_end() {
    let snapshot = solve_example(include_str!(
        "../../../examples/flowsheets/feed-mixer-flash.rfproj.json"
    ));

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 4);

    let mixer_out = snapshot
        .stream(&"stream-mix-out".into())
        .expect("expected mixer outlet");
    assert_close(mixer_out.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(mixer_out.temperature_k, 336.0, 1e-12);
    assert_close(
        *mixer_out
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-mix-out");
    assert_flash_consumes_stream(&snapshot, "stream-mix-out");

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_mixer_heater_flash_project_solves_end_to_end() {
    let snapshot = solve_example(include_str!(
        "../../../examples/flowsheets/feed-mixer-heater-flash.rfproj.json"
    ));

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 5);

    let mixed = snapshot
        .stream(&"stream-mix-out".into())
        .expect("expected mixer outlet");
    assert_close(mixed.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(mixed.temperature_k, 336.0, 1e-12);
    assert_close(
        *mixed
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 350.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *heated
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-mix-out");
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-heated");
    assert_flash_consumes_stream(&snapshot, "stream-heated");
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_heater_flash_project_solves_end_to_end() {
    let snapshot = solve_example(include_str!(
        "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
    ));

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 3);

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 345.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *heated
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-heated");
    assert_flash_consumes_stream(&snapshot, "stream-heated");
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_cooler_flash_project_solves_end_to_end() {
    let snapshot = solve_example(include_str!(
        "../../../examples/flowsheets/feed-cooler-flash.rfproj.json"
    ));

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 3);

    let cooled = snapshot
        .stream(&"stream-cooled".into())
        .expect("expected cooled outlet");
    assert_close(cooled.temperature_k, 305.0, 1e-12);
    assert_close(cooled.pressure_pa, 98_000.0, 1e-12);
    assert_close(cooled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *cooled
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-cooled");
    assert_flash_consumes_stream(&snapshot, "stream-cooled");
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_cooler_flash_binary_hydrocarbon_project_solves_end_to_end() {
    let provider = build_binary_hydrocarbon_lite_provider();
    let snapshot = solve_example_with_provider(
        include_str!(
            "../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
        ),
        &provider,
    );

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 3);

    let cooled = snapshot
        .stream(&"stream-cooled".into())
        .expect("expected cooled outlet");
    assert_close(cooled.temperature_k, 300.0, 1e-12);
    assert_close(cooled.pressure_pa, 650_000.0, 1e-12);
    assert_close(cooled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *cooled
            .overall_mole_fractions
            .get(&ComponentId::new("methane"))
            .expect("expected methane"),
        0.2,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-cooled");
    assert_flash_consumes_stream(&snapshot, "stream-cooled");

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_valve_flash_project_solves_end_to_end() {
    let snapshot = solve_example(include_str!(
        "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
    ));

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 3);

    let throttled = snapshot
        .stream(&"stream-throttled".into())
        .expect("expected valve outlet");
    assert_close(throttled.temperature_k, 315.0, 1e-12);
    assert_close(throttled.pressure_pa, 90_000.0, 1e-12);
    assert_close(throttled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *throttled
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-throttled");
    assert_flash_consumes_stream(&snapshot, "stream-throttled");
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn feed_valve_flash_binary_hydrocarbon_project_solves_end_to_end() {
    let provider = build_binary_hydrocarbon_lite_provider();
    let snapshot = solve_example_with_provider(
        include_str!(
            "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
        ),
        &provider,
    );

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.steps.len(), 3);

    let throttled = snapshot
        .stream(&"stream-throttled".into())
        .expect("expected valve outlet");
    assert_close(throttled.temperature_k, 300.0, 1e-12);
    assert_close(throttled.pressure_pa, 650_000.0, 1e-12);
    assert_close(throttled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *throttled
            .overall_mole_fractions
            .get(&ComponentId::new("methane"))
            .expect("expected methane"),
        0.2,
        1e-12,
    );
    assert_two_phase_window_spans_solver_stream(&snapshot, "stream-throttled");
    assert_flash_consumes_stream(&snapshot, "stream-throttled");

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    assert_flash_outlet_boundary_windows(&snapshot, "stream-liquid", "stream-vapor");
}

#[test]
fn valve_execution_failure_reports_step_execution_code_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/valve-execution-failure.rfproj.json"
    ))
    .expect_err("expected valve execution failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(error.message().contains("solver.step.execution:"));
    assert!(
        error
            .message()
            .contains("solver step 2 unit execution failed")
    );
    assert!(error.message().contains("unit `valve-1` (`valve`)"));
    assert!(error.message().contains("after consuming [stream-feed]"));
}

#[test]
fn unsupported_unit_kind_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/unsupported-unit-kind.rfproj.json"
    ))
    .expect_err("expected unsupported unit kind failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.unsupported_unit_kind")
    );
    assert_eq!(error.context().related_unit_ids(), &[UnitId::new("pump-1")]);
    assert!(error.message().contains(
        "solver.connection_validation.unsupported_unit_kind: solver connection validation failed"
    ));
    assert!(error.message().contains("unsupported kind `pump`"));
}

#[test]
fn self_loop_cycle_reports_topological_ordering_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"
    ))
    .expect_err("expected self-loop cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.self_loop_cycle")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("flash-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[rf_types::StreamId::new("stream-loop")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("flash-1", "liquid"),
        ]
    );
    assert!(error.message().contains(
        "solver.topological_ordering.self_loop_cycle: solver topological ordering failed"
    ));
    assert!(error.message().contains("forms a self loop"));
    assert!(error.message().contains("stream `stream-loop`"));
}

#[test]
fn multi_unit_cycle_reports_involved_units_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"
    ))
    .expect_err("expected multi-unit cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.two_unit_cycle")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("heater-1"), UnitId::new("valve-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[
            rf_types::StreamId::new("stream-a"),
            rf_types::StreamId::new("stream-b"),
        ]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            rf_types::DiagnosticPortTarget::new("valve-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("heater-1", "inlet"),
        ]
    );
    assert!(error.message().contains(
        "solver.topological_ordering.two_unit_cycle: solver topological ordering failed"
    ));
    assert!(error.message().contains("form a two-unit cycle"));
    assert!(
        error
            .message()
            .contains("streams `stream-a` and `stream-b`")
    );
}

#[test]
fn missing_upstream_source_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"
    ))
    .expect_err("expected missing upstream source failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.missing_upstream_source")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("mixer-1")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[rf_types::DiagnosticPortTarget::new("mixer-1", "inlet_a")]
    );
    assert!(error.message().contains(
        "solver.connection_validation.missing_upstream_source: solver connection validation failed"
    ));
    assert!(
        error
            .message()
            .contains("missing an upstream outlet connection")
    );
}

#[test]
fn missing_stream_reference_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/missing-stream-reference.rfproj.json"
    ))
    .expect_err("expected missing stream reference failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.missing_stream_reference")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("heater-1")]
    );
    assert!(error.message().contains(
        "solver.connection_validation.missing_stream_reference: solver connection validation failed"
    ));
    assert!(
        error
            .message()
            .contains("references missing stream `stream-missing`")
    );
}

#[test]
fn duplicate_upstream_source_reports_connection_validation_stream_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/duplicate-upstream-source.rfproj.json"
    ))
    .expect_err("expected duplicate upstream source failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.duplicate_upstream_source")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("feed-1"), UnitId::new("feed-2")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[rf_types::StreamId::new("shared-stream")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            rf_types::DiagnosticPortTarget::new("feed-1", "outlet"),
            rf_types::DiagnosticPortTarget::new("feed-2", "outlet"),
        ]
    );
    assert!(
        error
            .message()
            .contains(
                "solver.connection_validation.duplicate_upstream_source: solver connection validation failed"
            )
    );
    assert!(error.message().contains("produced by both"));
}

#[test]
fn invalid_port_signature_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"
    ))
    .expect_err("expected invalid port signature failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.invalid_port_signature")
    );
    assert_eq!(error.context().related_unit_ids(), &[UnitId::new("feed-1")]);
    assert!(error.message().contains(
        "solver.connection_validation.invalid_port_signature: solver connection validation failed"
    ));
    assert!(
        error
            .message()
            .contains("canonical built-in port signature")
    );
    assert!(error.message().contains("missing required port `outlet`"));
}

#[test]
fn duplicate_downstream_sink_reports_connection_validation_stream_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/duplicate-downstream-sink.rfproj.json"
    ))
    .expect_err("expected duplicate downstream sink failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.duplicate_downstream_sink")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("flash-1"), UnitId::new("mixer-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[rf_types::StreamId::new("shared-stream")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("mixer-1", "inlet_a"),
        ]
    );
    assert!(
        error
            .message()
            .contains(
                "solver.connection_validation.duplicate_downstream_sink: solver connection validation failed"
            )
    );
    assert!(error.message().contains("consumed by both"));
}

#[test]
fn orphan_stream_reports_connection_validation_stream_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/orphan-stream.rfproj.json"
    ))
    .expect_err("expected orphan stream failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.orphan_stream")
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[rf_types::StreamId::new("stream-orphan")]
    );
    assert!(error.context().related_unit_ids().is_empty());
    assert!(error.message().contains(
        "solver.connection_validation.orphan_stream: solver connection validation failed"
    ));
    assert!(
        error
            .message()
            .contains("is not connected to any material port")
    );
}

#[test]
fn unbound_outlet_port_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"
    ))
    .expect_err("expected unbound outlet port failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.unbound_outlet_port")
    );
    assert_eq!(error.context().related_unit_ids(), &[UnitId::new("feed-1")]);
    assert_eq!(
        error.context().related_port_targets(),
        &[rf_types::DiagnosticPortTarget::new("feed-1", "outlet")]
    );
    assert!(error.context().related_stream_ids().is_empty());
    assert!(error.message().contains(
        "solver.connection_validation.unbound_outlet_port: solver connection validation failed"
    ));
    assert!(error.message().contains("is not connected to any stream"));
}

#[test]
fn unbound_inlet_port_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/unbound-inlet-port.rfproj.json"
    ))
    .expect_err("expected unbound inlet port failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.unbound_inlet_port")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("heater-1")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[rf_types::DiagnosticPortTarget::new("heater-1", "inlet")]
    );
    assert!(error.context().related_stream_ids().is_empty());
    assert!(error.message().contains(
        "solver.connection_validation.unbound_inlet_port: solver connection validation failed"
    ));
    assert!(error.message().contains("is not connected to any stream"));
}
