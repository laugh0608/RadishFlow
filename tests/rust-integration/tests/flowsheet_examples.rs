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
fn unsupported_unit_kind_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "unsupported-unit-kind",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-feed": {
          "id": "stream-feed",
          "name": "Feed",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        }
      },
      "units": {
        "pump-1": {
          "id": "pump-1",
          "name": "Pump",
          "kind": "pump",
          "ports": [
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-feed"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-unsupported-unit-kind",
      "title": "Unsupported Unit Kind Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected unsupported unit kind failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver connection validation failed")
    );
    assert!(error.message().contains("unsupported kind `pump`"));
}

#[test]
fn self_loop_cycle_reports_topological_ordering_context_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "self-loop",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-loop": {
          "id": "stream-loop",
          "name": "Loop Stream",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        },
        "stream-vapor": {
          "id": "stream-vapor",
          "name": "Vapor Outlet",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 0.0,
          "overall_mole_fractions": {
            "component-a": 0.5,
            "component-b": 0.5
          },
          "phases": []
        }
      },
      "units": {
        "flash-1": {
          "id": "flash-1",
          "name": "Flash Drum",
          "kind": "flash_drum",
          "ports": [
            {
              "name": "inlet",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-loop"
            },
            {
              "name": "liquid",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-loop"
            },
            {
              "name": "vapor",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-vapor"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-self-loop",
      "title": "Self Loop Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected self-loop cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("solver topological ordering failed")
    );
    assert!(error.message().contains("contains a cycle"));
    assert!(error.message().contains("[flash-1]"));
}

#[test]
fn missing_upstream_source_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "missing-upstream-source",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-feed-a": {
          "id": "stream-feed-a",
          "name": "Feed A",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        },
        "stream-feed-b": {
          "id": "stream-feed-b",
          "name": "Feed B",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        },
        "stream-out": {
          "id": "stream-out",
          "name": "Outlet",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 0.0,
          "overall_mole_fractions": {
            "component-a": 0.5,
            "component-b": 0.5
          },
          "phases": []
        }
      },
      "units": {
        "mixer-1": {
          "id": "mixer-1",
          "name": "Mixer",
          "kind": "mixer",
          "ports": [
            {
              "name": "inlet_a",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-feed-a"
            },
            {
              "name": "inlet_b",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-feed-b"
            },
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-out"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-missing-upstream-source",
      "title": "Missing Upstream Source Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected missing upstream source failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver connection validation failed")
    );
    assert!(
        error
            .message()
            .contains("missing an upstream outlet connection")
    );
}

#[test]
fn missing_stream_reference_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "missing-stream-reference",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-feed": {
          "id": "stream-feed",
          "name": "Feed",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        }
      },
      "units": {
        "feed-1": {
          "id": "feed-1",
          "name": "Feed",
          "kind": "feed",
          "ports": [
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-feed"
            }
          ]
        },
        "heater-1": {
          "id": "heater-1",
          "name": "Heater",
          "kind": "heater",
          "ports": [
            {
              "name": "inlet",
              "direction": "inlet",
              "kind": "material",
              "stream_id": "stream-feed"
            },
            {
              "name": "outlet",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-missing"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-missing-stream-reference",
      "title": "Missing Stream Reference Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected missing stream reference failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver connection validation failed")
    );
    assert!(
        error
            .message()
            .contains("references missing stream `stream-missing`")
    );
}

#[test]
fn invalid_port_signature_reports_connection_validation_context_end_to_end() {
    let error = solve_example_result(
        r#"
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "invalid-port-signature",
      "components": {
        "component-a": { "id": "component-a", "name": "Component A", "formula": null },
        "component-b": { "id": "component-b", "name": "Component B", "formula": null }
      },
      "streams": {
        "stream-feed": {
          "id": "stream-feed",
          "name": "Feed",
          "temperature_k": 320.0,
          "pressure_pa": 100000.0,
          "total_molar_flow_mol_s": 5.0,
          "overall_mole_fractions": {
            "component-a": 0.35,
            "component-b": 0.65
          },
          "phases": []
        }
      },
      "units": {
        "feed-1": {
          "id": "feed-1",
          "name": "Feed",
          "kind": "feed",
          "ports": [
            {
              "name": "unexpected",
              "direction": "outlet",
              "kind": "material",
              "stream_id": "stream-feed"
            }
          ]
        }
      }
    },
    "metadata": {
      "documentId": "example-invalid-port-signature",
      "title": "Invalid Port Signature Example",
      "schemaVersion": 1,
      "createdAt": "2026-04-05T00:00:00Z",
      "updatedAt": "2026-04-05T00:00:00Z"
    }
  }
}
"#,
    )
    .expect_err("expected invalid port signature failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert!(
        error
            .message()
            .contains("solver connection validation failed")
    );
    assert!(
        error
            .message()
            .contains("canonical built-in port signature")
    );
    assert!(error.message().contains("missing required port `outlet`"));
}
