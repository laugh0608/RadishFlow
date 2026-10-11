use std::time::UNIX_EPOCH;

use rf_model::{Component, MaterialStreamState, UnitOperationParameters};
use rf_types::PortDirection;
use rf_ui::BuiltinUnitKind;
use rf_ui::{AppState, DocumentMetadata, ProjectPresentationCommand};

use super::*;
use crate::{StudioModelingReadinessTask, studio_modeling_run_blocker};

fn test_unit(
    id: &str,
    kind: BuiltinUnitKind,
    inputs: &[&str],
    outputs: &[&str],
) -> rf_model::UnitNode {
    let input_names: &[&str] = if kind == BuiltinUnitKind::Mixer {
        &["inlet_a", "inlet_b"]
    } else {
        &["inlet"]
    };
    let output_names: &[&str] = if kind == BuiltinUnitKind::FlashDrum {
        &["liquid", "vapor"]
    } else {
        &["outlet"]
    };
    let ports = input_names
        .iter()
        .zip(inputs)
        .map(|(port, stream)| (*port, *stream, PortDirection::Inlet))
        .chain(
            output_names
                .iter()
                .zip(outputs)
                .map(|(port, stream)| (*port, *stream, PortDirection::Outlet)),
        )
        .map(|(port, stream, direction)| {
            rf_model::UnitPort::new(
                port,
                direction,
                rf_types::PortKind::Material,
                Some(stream.into()),
            )
        })
        .collect();
    rf_model::UnitNode::new(id, id, kind.as_str(), ports)
}

fn document(kind: BuiltinUnitKind) -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("specification-checks");
    for component in ["a", "b"] {
        flowsheet
            .insert_component(Component::new(component, component))
            .unwrap();
    }
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "in",
            "Feed",
            300.0,
            100_000.0,
            1.0,
            [("a".into(), 0.4), ("b".into(), 0.6)].into(),
        ))
        .unwrap();
    flowsheet
        .insert_unit(test_unit("feed", BuiltinUnitKind::Feed, &[], &["in"]))
        .unwrap();
    if kind != BuiltinUnitKind::Feed {
        let mut unit = match kind {
            BuiltinUnitKind::Heater => {
                test_unit("unit", BuiltinUnitKind::Heater, &["in"], &["out"])
            }
            BuiltinUnitKind::Cooler => {
                test_unit("unit", BuiltinUnitKind::Cooler, &["in"], &["out"])
            }
            BuiltinUnitKind::Valve => test_unit("unit", BuiltinUnitKind::Valve, &["in"], &["out"]),
            BuiltinUnitKind::Mixer => {
                let mut second = flowsheet.streams[&"in".into()].clone();
                second.id = "second".into();
                second.pressure_pa = 150_000.0;
                flowsheet.insert_stream(second).unwrap();
                flowsheet
                    .insert_unit(test_unit(
                        "feed-second",
                        BuiltinUnitKind::Feed,
                        &[],
                        &["second"],
                    ))
                    .unwrap();
                test_unit("unit", BuiltinUnitKind::Mixer, &["in", "second"], &["out"])
            }
            BuiltinUnitKind::FlashDrum => {
                flowsheet
                    .insert_stream(MaterialStreamState::new("vapor", "Vapor template"))
                    .unwrap();
                test_unit(
                    "unit",
                    BuiltinUnitKind::FlashDrum,
                    &["in"],
                    &["out", "vapor"],
                )
            }
            BuiltinUnitKind::Feed => unreachable!(),
        };
        unit.parameters.outlet_pressure_pa = Some(90_000.0);
        if matches!(
            kind,
            BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler | BuiltinUnitKind::FlashDrum
        ) {
            unit.parameters.outlet_temperature_k = Some(300.0);
        }
        flowsheet.insert_unit(unit).unwrap();
        flowsheet
            .insert_stream(MaterialStreamState::new("out", "Outlet template"))
            .unwrap();
    }
    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("a1-document", "Specifications", UNIX_EPOCH),
    )
}

fn report(document: &FlowsheetDocument) -> ModelingSpecificationReport {
    check_modeling_specifications(document, ModelingCheckSessionId(1))
}

#[test]
fn six_normal_models_report_only_their_evaluated_local_scope_not_global_solvability() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::FlashDrum,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Valve,
    ] {
        let document = document(kind);
        let report = report(&document);
        assert!(
            report.problems.is_empty(),
            "{kind:?}: {:?}",
            report.problems
        );
        assert!(studio_modeling_run_blocker(&document).is_none());
        assert!(report.coverage.contains(&SpecificationCheckCoverage {
            scope: SpecificationCheckScope::StructuralDegreesOfFreedom,
            coverage: SpecificationCoverage::NotAnalyzed
        }));
        assert!(report.coverage.contains(&SpecificationCheckCoverage {
            scope: SpecificationCheckScope::NumericalSolvability,
            coverage: SpecificationCoverage::NotAnalyzed
        }));
        assert_eq!(
            report.method,
            SpecificationAnalysisMethod::BuiltinSteadyStateInputRules
        );
        assert_eq!(report.contract_version, 1);
    }
}

#[test]
fn absent_explicit_parameters_are_all_reported_despite_visible_templates() {
    for (kind, expected) in [
        (
            BuiltinUnitKind::Heater,
            vec![
                UnitParameter::OutletTemperature,
                UnitParameter::OutletPressure,
            ],
        ),
        (
            BuiltinUnitKind::Cooler,
            vec![
                UnitParameter::OutletTemperature,
                UnitParameter::OutletPressure,
            ],
        ),
        (
            BuiltinUnitKind::FlashDrum,
            vec![
                UnitParameter::OutletTemperature,
                UnitParameter::OutletPressure,
            ],
        ),
        (BuiltinUnitKind::Mixer, vec![UnitParameter::OutletPressure]),
        (BuiltinUnitKind::Valve, vec![UnitParameter::OutletPressure]),
    ] {
        let mut document = document(kind);
        document
            .flowsheet
            .units
            .get_mut(&"unit".into())
            .unwrap()
            .parameters = UnitOperationParameters::default();
        let report = report(&document);
        let missing = report
            .problems
            .iter()
            .filter_map(|problem| match problem {
                SpecificationProblem::MissingParameter { parameter, .. } => Some(*parameter),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(missing, expected, "{kind:?}");
        assert!(matches!(
            studio_modeling_run_blocker(&document).unwrap().task,
            StudioModelingReadinessTask::CommitUnitParameter { .. }
        ));
        assert_eq!(
            document.flowsheet.streams[&"out".into()].temperature_k,
            298.15
        );
    }
}

#[test]
fn feed_check_uses_effective_overrides_without_changing_failure_stage() {
    let mut document = document(BuiltinUnitKind::Feed);
    let stream = document.flowsheet.streams.get_mut(&"in".into()).unwrap();
    stream.temperature_k = -1.0;
    stream.pressure_pa = 0.0;
    let feed = document.flowsheet.units.get_mut(&"feed".into()).unwrap();
    feed.parameters.outlet_temperature_k = Some(300.0);
    feed.parameters.outlet_pressure_pa = Some(100_000.0);
    let valid = report(&document);
    assert!(valid.problems.is_empty());
    assert!(studio_modeling_run_blocker(&document).is_none());
    assert_eq!(
        valid.feed_inputs[0].input.source,
        rf_flowsheet::specifications::FeedInputSource::UnitParameter(
            UnitParameter::OutletTemperature
        )
    );

    document
        .flowsheet
        .units
        .get_mut(&"feed".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(-1.0);
    let invalid = report(&document);
    assert!(matches!(
        invalid.problems.as_slice(),
        [SpecificationProblem::InvalidParameter {
            parameter: UnitParameter::OutletPressure,
            ..
        }]
    ));
    assert_eq!(
        invalid.problems[0].enforcement(),
        SpecificationEnforcement::Solver
    );
    assert!(studio_modeling_run_blocker(&document).is_none());

    document
        .flowsheet
        .units
        .get_mut(&"feed".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = None;
    assert!(matches!(
        studio_modeling_run_blocker(&document).unwrap().task,
        StudioModelingReadinessTask::FixFeedSourcePressure { .. }
    ));
}

#[test]
fn feed_scalar_gaps_are_distinct_and_zero_flow_is_only_an_authoring_constraint() {
    for field in [
        FeedInputField::Temperature,
        FeedInputField::Pressure,
        FeedInputField::MolarFlow,
    ] {
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let mut document = document(BuiltinUnitKind::Feed);
            let stream = document.flowsheet.streams.get_mut(&"in".into()).unwrap();
            match field {
                FeedInputField::Temperature => stream.temperature_k = value,
                FeedInputField::Pressure => stream.pressure_pa = value,
                FeedInputField::MolarFlow => stream.total_molar_flow_mol_s = value,
            }
            let report = report(&document);
            assert!(
                matches!(&report.problems[0], SpecificationProblem::InvalidFeedInput { field: actual, .. } if *actual == field)
            );
            assert!(studio_modeling_run_blocker(&document).is_some());
        }
    }
    // Phase outlet templates are zero-flow and composition-free; not Feed specs.
    assert!(
        report(&document(BuiltinUnitKind::FlashDrum))
            .problems
            .is_empty()
    );
}

#[test]
fn compositions_report_independent_bad_fields_and_never_normalize_during_query() {
    let mut document = document(BuiltinUnitKind::Feed);
    document
        .flowsheet
        .streams
        .get_mut(&"in".into())
        .unwrap()
        .overall_mole_fractions = [("a".into(), 0.2), ("b".into(), 0.3)].into();
    let before = document.clone();
    assert!(matches!(
        report(&document).problems.as_slice(),
        [SpecificationProblem::UnnormalizedComposition { sum: 0.5, .. }]
    ));
    assert_eq!(document, before);
    document
        .flowsheet
        .streams
        .get_mut(&"in".into())
        .unwrap()
        .overall_mole_fractions = [("a".into(), -0.1), ("b".into(), 1.1)].into();
    let report = report(&document);
    assert_eq!(report.problems.len(), 2);
    assert!(
        report
            .problems
            .iter()
            .all(|problem| matches!(problem, SpecificationProblem::InvalidMoleFraction { .. }))
    );
    assert!(matches!(
        studio_modeling_run_blocker(&document).unwrap().task,
        StudioModelingReadinessTask::FixStreamCompositionValues { .. }
    ));
}

#[test]
fn component_selection_and_empty_feed_composition_keep_readiness_priority() {
    let mut document = document(BuiltinUnitKind::Feed);
    document.flowsheet.components.clear();
    let result = report(&document);
    assert_eq!(result.problems.len(), 3);
    assert!(matches!(
        result.problems[0],
        SpecificationProblem::MissingProjectComponents
    ));
    assert!(matches!(
        studio_modeling_run_blocker(&document).unwrap().task,
        StudioModelingReadinessTask::SelectProjectComponents
    ));
    document
        .flowsheet
        .components
        .insert("a".into(), Component::new("a", "A"));
    assert!(
        matches!(studio_modeling_run_blocker(&document).unwrap().task, StudioModelingReadinessTask::SelectReferencedProjectComponent { component_id, .. } if component_id == "b")
    );
    document
        .flowsheet
        .streams
        .get_mut(&"in".into())
        .unwrap()
        .overall_mole_fractions
        .clear();
    assert!(matches!(
        report(&document).problems.as_slice(),
        [SpecificationProblem::MissingFeedComposition { .. }]
    ));
}

#[test]
fn topology_failure_does_not_hide_independent_fields_or_change_legacy_run_priority() {
    let mut document = document(BuiltinUnitKind::Heater);
    let unit = document.flowsheet.units.get_mut(&"unit".into()).unwrap();
    unit.ports[0].stream_id = None;
    unit.parameters = UnitOperationParameters::default();
    let result = report(&document);
    assert!(
        matches!(&result.problems[0], SpecificationProblem::Topology(error) if error.context().diagnostic_code() == Some("solver.connection_validation.unbound_inlet_port"))
    );
    assert_eq!(
        result
            .problems
            .iter()
            .filter(|p| matches!(p, SpecificationProblem::MissingParameter { .. }))
            .count(),
        2
    );
    assert!(result.coverage.contains(&SpecificationCheckCoverage {
        scope: SpecificationCheckScope::FeedInputs,
        coverage: SpecificationCoverage::NotCheckedTopology
    }));
    assert!(studio_modeling_run_blocker(&document).is_none());
}

#[test]
fn pressure_checks_use_committed_upstream_sources_for_all_constrained_models() {
    for kind in [
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
    ] {
        let mut document = document(kind);
        for (value, conflicts) in [(90_000.0, false), (100_000.0, false), (100_001.0, true)] {
            document
                .flowsheet
                .units
                .get_mut(&"unit".into())
                .unwrap()
                .parameters
                .outlet_pressure_pa = Some(value);
            let result = report(&document);
            assert_eq!(
                result
                    .problems
                    .iter()
                    .any(|p| matches!(p, SpecificationProblem::OutletPressureAboveInlet { .. })),
                conflicts,
                "{kind:?} / {value}"
            );
            assert!(studio_modeling_run_blocker(&document).is_none());
        }
    }
}

#[test]
fn flash_pressure_and_heater_cooler_temperature_have_no_unimplemented_direction_rules() {
    for (kind, temperature, pressure) in [
        (BuiltinUnitKind::Heater, 280.0, 90_000.0),
        (BuiltinUnitKind::Cooler, 350.0, 90_000.0),
        (BuiltinUnitKind::FlashDrum, 300.0, 150_000.0),
    ] {
        let mut document = document(kind);
        let unit = document.flowsheet.units.get_mut(&"unit".into()).unwrap();
        unit.parameters.outlet_temperature_k = Some(temperature);
        unit.parameters.outlet_pressure_pa = Some(pressure);
        assert!(report(&document).problems.is_empty());
    }
}

#[test]
fn readonly_reports_ignore_drafts_display_units_and_existing_results() {
    let mut app = AppState::new(document(BuiltinUnitKind::Heater));
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"unit".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = None;
    let before = report(&app.workspace.document);
    app.focus_inspector_target(rf_ui::InspectorTarget::Unit("unit".into()));
    app.update_unit_inspector_draft(
        &"unit".into(),
        rf_ui::UnitInspectorDraftField::OutletTemperatureK,
        "330",
    )
    .unwrap();
    assert!(
        app.workspace
            .project_presentation
            .apply(ProjectPresentationCommand::Apply(
                rf_types::units::DisplayUnitSet::engineering(),
            ))
    );
    app.store_snapshot(rf_ui::SolveSnapshot::new(
        "old-result",
        0,
        1,
        rf_ui::RunStatus::Converged,
        rf_ui::DiagnosticSummary::new(0, rf_ui::DiagnosticSeverity::Info, "previous solve"),
    ));
    let unchanged = app.clone();
    let after = report(&app.workspace.document);
    assert_eq!(after, before);
    assert_eq!(app, unchanged);
    assert_eq!(
        after.identity.input_revision,
        app.workspace.document.revision
    );
    let another_session =
        check_modeling_specifications(&app.workspace.document, ModelingCheckSessionId(2));
    assert_ne!(another_session.identity, after.identity);
    assert_eq!(another_session.problems, after.problems);
}

#[test]
fn unsupported_kinds_and_cycles_are_not_claimed_as_zero_degrees_of_freedom() {
    let mut document = document(BuiltinUnitKind::Valve);
    document
        .flowsheet
        .units
        .get_mut(&"unit".into())
        .unwrap()
        .kind = "unknown".into();
    assert!(
        matches!(&report(&document).problems[0], SpecificationProblem::Topology(error) if error.context().diagnostic_code() == Some("solver.connection_validation.unsupported_unit_kind"))
    );
    document
        .flowsheet
        .units
        .get_mut(&"unit".into())
        .unwrap()
        .kind = "valve".into();
    document.flowsheet.units.remove(&"feed".into());
    document.flowsheet.streams.remove(&"in".into());
    let unit = document.flowsheet.units.get_mut(&"unit".into()).unwrap();
    unit.ports
        .iter_mut()
        .find(|p| p.direction == PortDirection::Inlet)
        .unwrap()
        .stream_id = Some("out".into());
    assert!(
        matches!(&report(&document).problems[0], SpecificationProblem::Topology(error) if error.context().diagnostic_code() == Some("solver.topological_ordering.self_loop_cycle"))
    );
    assert!(studio_modeling_run_blocker(&document).is_none());
}

#[test]
fn empty_document_retains_the_place_first_unit_task() {
    let document = FlowsheetDocument::new(
        Flowsheet::new("empty"),
        DocumentMetadata::new("empty", "Empty", UNIX_EPOCH),
    );
    assert!(matches!(
        report(&document).problems[0],
        SpecificationProblem::EmptyFlowsheet
    ));
    assert_eq!(
        studio_modeling_run_blocker(&document).unwrap().task,
        StudioModelingReadinessTask::PlaceFirstUnit
    );
}

#[test]
fn connection_failure_matrix_preserves_solver_codes_and_targets() {
    for (case, suffix) in [
        ("outlet", "unbound_outlet_port"),
        ("missing", "missing_stream_reference"),
        ("source", "duplicate_upstream_source"),
        ("sink", "duplicate_downstream_sink"),
        ("orphan", "orphan_stream"),
        ("no_source", "missing_upstream_source"),
    ] {
        let mut document = document(BuiltinUnitKind::Heater);
        match case {
            "outlet" => {
                document
                    .flowsheet
                    .units
                    .get_mut(&"unit".into())
                    .unwrap()
                    .ports[1]
                    .stream_id = None
            }
            "missing" => {
                document
                    .flowsheet
                    .units
                    .get_mut(&"unit".into())
                    .unwrap()
                    .ports[0]
                    .stream_id = Some("absent".into())
            }
            "source" => {
                document
                    .flowsheet
                    .insert_unit(test_unit("other-feed", BuiltinUnitKind::Feed, &[], &["in"]))
                    .unwrap();
            }
            "sink" => {
                document
                    .flowsheet
                    .insert_stream(MaterialStreamState::new("other-out", "Other"))
                    .unwrap();
                document
                    .flowsheet
                    .insert_unit(test_unit(
                        "other",
                        BuiltinUnitKind::Valve,
                        &["in"],
                        &["other-out"],
                    ))
                    .unwrap();
            }
            "orphan" => {
                document
                    .flowsheet
                    .insert_stream(MaterialStreamState::new("orphan", "Orphan"))
                    .unwrap();
            }
            "no_source" => {
                document.flowsheet.units.remove(&"feed".into());
            }
            _ => unreachable!(),
        }
        let result = report(&document);
        let SpecificationProblem::Topology(error) = &result.problems[0] else {
            panic!("{case}: {result:?}")
        };
        assert_eq!(
            error.context().diagnostic_code(),
            Some(format!("solver.connection_validation.{suffix}").as_str()),
            "{case}"
        );
        assert!(
            !error.context().related_stream_ids().is_empty()
                || !error.context().related_port_targets().is_empty(),
            "{case}"
        );
        assert!(studio_modeling_run_blocker(&document).is_none());
        assert!(result.coverage.contains(&SpecificationCheckCoverage {
            scope: SpecificationCheckScope::InletPressure("unit".into()),
            coverage: SpecificationCoverage::NotCheckedTopology,
        }));
    }
}

#[test]
fn two_unit_cycle_and_independent_parameter_problems_have_separate_evidence() {
    let mut document = document(BuiltinUnitKind::Heater);
    document.flowsheet.units.remove(&"feed".into());
    document
        .flowsheet
        .insert_unit(test_unit(
            "other",
            BuiltinUnitKind::Cooler,
            &["out"],
            &["in"],
        ))
        .unwrap();
    document
        .flowsheet
        .units
        .get_mut(&"unit".into())
        .unwrap()
        .parameters = UnitOperationParameters::default();
    let result = report(&document);
    assert!(
        matches!(&result.problems[0], SpecificationProblem::Topology(error) if error.context().diagnostic_code() == Some("solver.topological_ordering.two_unit_cycle"))
    );
    let missing: Vec<_> = result
        .problems
        .iter()
        .filter_map(|problem| match problem {
            SpecificationProblem::MissingParameter { unit_id, parameter } => {
                Some((unit_id.as_str(), *parameter))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        missing,
        vec![
            ("other", UnitParameter::OutletTemperature),
            ("other", UnitParameter::OutletPressure),
            ("unit", UnitParameter::OutletTemperature),
            ("unit", UnitParameter::OutletPressure),
        ]
    );
    assert_eq!(report(&document), result);
}

#[test]
fn scalar_and_composition_boundaries_are_projected_without_implicit_repair() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::FlashDrum,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Valve,
    ] {
        let parameters: &[UnitParameter] =
            if matches!(kind, BuiltinUnitKind::Mixer | BuiltinUnitKind::Valve) {
                &[UnitParameter::OutletPressure]
            } else {
                &[
                    UnitParameter::OutletTemperature,
                    UnitParameter::OutletPressure,
                ]
            };
        for &parameter in parameters {
            for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
                let mut document = document(kind);
                let id = if kind == BuiltinUnitKind::Feed {
                    "feed"
                } else {
                    "unit"
                };
                let params = &mut document
                    .flowsheet
                    .units
                    .get_mut(&id.into())
                    .unwrap()
                    .parameters;
                match parameter {
                    UnitParameter::OutletPressure => params.outlet_pressure_pa = Some(value),
                    UnitParameter::OutletTemperature => params.outlet_temperature_k = Some(value),
                }
                assert!(report(&document).problems.iter().any(|problem| matches!(problem, SpecificationProblem::InvalidParameter { unit_id, parameter: actual, .. } if unit_id.as_str() == id && *actual == parameter)));
                assert!(studio_modeling_run_blocker(&document).is_none());
            }
        }
    }
    for (a, b, expected) in [
        (0.0, 0.0, "total"),
        (0.5, 0.5 + 0.5e-9, "valid"),
        (0.5, 0.5 + 2e-9, "sum"),
        (f64::NAN, 0.5, "fraction"),
        (f64::INFINITY, 0.5, "fraction"),
    ] {
        let mut document = document(BuiltinUnitKind::Feed);
        document
            .flowsheet
            .streams
            .get_mut(&"in".into())
            .unwrap()
            .overall_mole_fractions = [("a".into(), a), ("b".into(), b)].into();
        let report = report(&document);
        match expected {
            "valid" => assert!(report.problems.is_empty()),
            "total" => assert!(matches!(
                report.problems[0],
                SpecificationProblem::InvalidCompositionTotal { .. }
            )),
            "sum" => assert!(matches!(
                report.problems[0],
                SpecificationProblem::UnnormalizedComposition { .. }
            )),
            "fraction" => assert!(matches!(
                report.problems[0],
                SpecificationProblem::InvalidMoleFraction { .. }
            )),
            _ => unreachable!(),
        }
    }
}

#[test]
fn missing_upstream_pressure_is_unchecked_even_when_a_template_is_available() {
    let mut document = document(BuiltinUnitKind::Heater);
    document
        .flowsheet
        .insert_stream(MaterialStreamState::new("final", "Final"))
        .unwrap();
    let mut valve = test_unit("valve", BuiltinUnitKind::Valve, &["out"], &["final"]);
    valve.parameters.outlet_pressure_pa = Some(70_000.0);
    document.flowsheet.insert_unit(valve).unwrap();
    document
        .flowsheet
        .units
        .get_mut(&"unit".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = None;
    let report = report(&document);
    assert!(report.coverage.iter().any(|coverage| coverage.scope
        == SpecificationCheckScope::InletPressure("valve".into())
        && matches!(
            coverage.coverage,
            SpecificationCoverage::NotCheckedInputPressure(_)
        )));
    assert!(report.problems.iter().any(|problem| matches!(problem, SpecificationProblem::MissingParameter { unit_id, parameter: UnitParameter::OutletPressure } if unit_id.as_str() == "unit")));
}
