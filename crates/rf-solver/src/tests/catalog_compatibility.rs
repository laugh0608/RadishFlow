use super::*;

#[test]
fn saving_and_reopening_representative_chains_preserves_catalog_execution() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    for flowsheet in [
        build_demo_flowsheet(),
        build_feed_mixer_heater_flash_flowsheet(),
        build_feed_valve_flash_flowsheet(),
        build_feed_cooler_flash_flowsheet(),
    ] {
        let expected = SequentialModularSolver
            .solve(&services, &flowsheet)
            .unwrap();
        let file = rf_store::StoredProjectFile::new(
            flowsheet,
            rf_store::StoredDocumentMetadata::new(
                "catalog-project",
                "Catalog",
                std::time::UNIX_EPOCH,
            ),
        );
        let json = rf_store::project_file_to_pretty_json(&file).unwrap();
        let reopened = parse_project_file_json(&json).unwrap();
        assert_eq!(reopened, file);
        assert_eq!(
            SequentialModularSolver
                .solve(&services, &reopened.document.flowsheet)
                .unwrap(),
            expected
        );
    }
}

#[test]
fn stream_validation_still_precedes_invalid_unit_parameters() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = build_feed_heater_flash_flowsheet();
    flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .unwrap()
        .overall_mole_fractions
        .clear();
    flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(-1.0);
    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.step.stream_input")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("heater-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-feed")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[DiagnosticPortTarget::new("heater-1", "inlet")]
    );

    flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .unwrap()
        .overall_mole_fractions = binary_composition(0.35, 0.65);
    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.step.parameter")
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-feed"), StreamId::new("stream-heated")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[DiagnosticPortTarget::new("heater-1", "outlet")]
    );
}

#[test]
fn successive_runs_use_their_own_parameters_and_do_not_modify_project_templates() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let first = build_feed_heater_flash_flowsheet();
    let original = first.clone();
    let mut second = first.clone();
    second
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(330.0);
    let snapshot = SequentialModularSolver.solve(&services, &first).unwrap();
    let changed = SequentialModularSolver.solve(&services, &second).unwrap();
    assert_eq!(
        snapshot
            .stream(&"stream-heated".into())
            .unwrap()
            .temperature_k,
        345.0
    );
    assert_eq!(
        changed
            .stream(&"stream-heated".into())
            .unwrap()
            .temperature_k,
        330.0
    );
    assert_eq!(
        SequentialModularSolver.solve(&services, &first).unwrap(),
        snapshot
    );
    assert_eq!(first, original);
    assert_eq!(second.streams, first.streams);
    assert_eq!(
        second.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(330.0)
    );
}
