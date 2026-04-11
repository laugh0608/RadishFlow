use rf_flash::PlaceholderTpFlashSolver;
use rf_rust_integration::{assert_close, build_binary_demo_provider};
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveStatus, SolverServices};
use rf_store::parse_project_file_json;
use rf_types::{ComponentId, PhaseLabel};

fn solve_example(project_json: &str) -> rf_solver::SolveSnapshot {
    solve_example_result(project_json).expect("expected solve snapshot")
}

fn solve_example_result(project_json: &str) -> rf_types::RfResult<rf_solver::SolveSnapshot> {
    let provider = build_binary_demo_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(project_json).expect("expected example project parse");

    SequentialModularSolver.solve(&services, &project.document.flowsheet)
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

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
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
    assert!(
        error
            .message()
            .contains("solver.connection_validation: solver connection validation failed")
    );
    assert!(error.message().contains("unsupported kind `pump`"));
}

#[test]
fn self_loop_cycle_reports_topological_ordering_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"
    ))
    .expect_err("expected self-loop cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("solver.topological_ordering: solver topological ordering failed")
    );
    assert!(error.message().contains("contains a cycle"));
    assert!(error.message().contains("[flash-1]"));
}

#[test]
fn multi_unit_cycle_reports_involved_units_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "heater-valve-cycle",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-a": {
          "id": "stream-a",
          "name": "Cycle Stream A",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        },
        "stream-b": {
          "id": "stream-b",
          "name": "Cycle Stream B",
          "temperature_k": 300.0,
          "pressure_pa": 95000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        }
      },
      "units": {
        "heater-1": {
          "id": "heater-1",
          "name": "Heater",
          "kind": "heater",
          "ports": [
            {
              "name": "inlet",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-b"
            },
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-a"
            }
          ]
        },
        "valve-1": {
          "id": "valve-1",
          "name": "Valve",
          "kind": "valve",
          "ports": [
            {
              "name": "inlet",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-a"
            },
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-b"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-heater-valve-cycle",
      "title": "Heater Valve Cycle Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected multi-unit cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("solver.topological_ordering: solver topological ordering failed")
    );
    assert!(error.message().contains("contains a cycle"));
    assert!(error.message().contains("[heater-1, valve-1]"));
}

#[test]
fn missing_upstream_source_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"
    ))
    .expect_err("expected missing upstream source failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver.connection_validation: solver connection validation failed")
    );
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
    assert!(
        error
            .message()
            .contains("solver.connection_validation: solver connection validation failed")
    );
    assert!(
        error
            .message()
            .contains("references missing stream `stream-missing`")
    );
}

#[test]
fn invalid_port_signature_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(include_str!(
        "../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"
    ))
    .expect_err("expected invalid port signature failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver.connection_validation: solver connection validation failed")
    );
    assert!(
        error
            .message()
            .contains("canonical built-in port signature")
    );
    assert!(error.message().contains("missing required port `outlet`"));
}
