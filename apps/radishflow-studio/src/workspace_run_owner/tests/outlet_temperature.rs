use super::*;
use crate::outlet_temperature::*;
use crate::workspace_run_owner::unit_control::UnitControlGate;
use rf_types::units::MeasurementUnit::{Celsius, Kelvin};

fn mixer_project(feed_b_temperature: f64) -> AppState {
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    project
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed-b".into())
        .unwrap()
        .temperature_k = feed_b_temperature;
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("mixer", "Mixer", UNIX_EPOCH),
    ))
}

fn rule(app: &AppState, stream: &str, threshold: TemperatureThreshold) -> TemperatureCondition {
    let outlet = MaterialOutlet::connected(&app.workspace.document.flowsheet)
        .find(|outlet| outlet.stream.as_str() == stream)
        .unwrap();
    TemperatureCondition {
        expected: app.workspace.flow_execution_qualification(),
        outlet,
        threshold,
    }
}

fn submit(
    owner: &mut WorkspaceRunOwner,
    app: &mut AppState,
    condition: Option<TemperatureCondition>,
) -> FlowRunId {
    let (root, index) = cache();
    let WorkspaceRunSubmission::Accepted(id) = owner
        .submit_cached(
            app,
            root,
            &index,
            &WorkspaceRunCommand::manual(PACKAGE),
            now(),
        )
        .unwrap()
    else {
        panic!("expected a new run")
    };
    if let Some(condition) = condition {
        owner.arm_new_temperature_condition(&id, condition);
    }
    id
}

fn current(app: &AppState) -> Option<FlowRunId> {
    match app.workspace.flow_result() {
        rf_ui::result_query::FlowResultView::Current { origin, .. } => {
            Some(origin.execution().unwrap().run_id.clone())
        }
        _ => None,
    }
}

#[test]
fn real_mixer_compares_unrounded_completed_temperature_and_publishes_only_on_acceptance() {
    for (feed, actual, hit, release) in [
        (
            319.999999,
            311.9999994,
            false,
            UnitRunCommand::Continue { step: 4 },
        ),
        (320., 312., false, UnitRunCommand::Continue { step: 4 }),
        (
            320.000001,
            312.0000006,
            true,
            UnitRunCommand::Continue { step: 4 },
        ),
        (
            320.000001,
            312.0000006,
            true,
            UnitRunCommand::Step { step: 4 },
        ),
    ] {
        let mut app = mixer_project(feed);
        let mut owner = WorkspaceRunOwner::new();
        let threshold = TemperatureThreshold::parse("38.85", Celsius).unwrap();
        let condition = rule(&app, "stream-mix-out", threshold);
        assert!(condition.is_current(&app.workspace));
        assert_eq!(condition.threshold.kelvin().value(), 312.);
        let previous = submit(&mut owner, &mut app, None);
        let ordinary = drain(&mut owner, &mut app);
        assert!(ordinary.observation.is_none());
        let id = submit(&mut owner, &mut app, Some(condition.clone()));
        if hit {
            wait_ready(&owner);
            // The GUI grants Ready once and then stops polling while the solver advances.
            until(|| {
                owner.poll(&mut app, now());
                owner.state().unwrap().started_at.is_some()
            });
            until(
                || matches!(owner.state().unwrap().unit_control.unwrap().phase,UnitRunPhase::Paused { step:4,total:4,ref unit } if unit.as_str()=="flash-1"),
            );
            assert_eq!(current(&app), Some(previous.clone()));
            assert_eq!(
                owner.control_units(&mut app, &id, release, now()),
                UnitControlReply::Accepted
            );
        }
        let result = drain(&mut owner, &mut app);
        let watch = result.observation.unwrap();
        assert_eq!(watch.condition, condition);
        assert_eq!(watch.total, 4);
        let TemperatureReading::Measured {
            step: 3,
            actual: measured,
        } = watch.reading
        else {
            panic!("{watch:?}")
        };
        assert_eq!(measured.value(), actual);
        assert_eq!(current(&app), Some(id));
        let rf_ui::result_query::FlowResultView::Current { snapshot, .. } =
            app.workspace.flow_result()
        else {
            panic!("no snapshot")
        };
        let stream = snapshot
            .streams
            .iter()
            .find(|s| s.stream_id.as_str() == "stream-mix-out")
            .unwrap();
        assert_eq!(stream.temperature_k, actual);
        assert_eq!(stream.total_molar_flow_mol_s, 5.);
    }
}

#[test]
fn final_flash_hit_needs_no_release_and_upstream_failure_keeps_original_diagnostic() {
    let mut app = mixer_project(320.);
    let mut owner = WorkspaceRunOwner::new();
    let threshold = TemperatureThreshold::parse("25", Celsius).unwrap();
    let condition = rule(&app, "stream-vapor", threshold.clone());
    submit(&mut owner, &mut app, Some(condition));
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert!(
        matches!(summary.observation.unwrap().reading,TemperatureReading::Measured {step:4,actual} if actual.value()==300.)
    );
    app.workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed-b".into())
        .unwrap()
        .overall_mole_fractions = [("methane".into(), 1.0)].into_iter().collect();
    let condition = rule(&app, "stream-mix-out", threshold);
    submit(&mut owner, &mut app, Some(condition));
    let failure = drain(&mut owner, &mut app);
    assert_eq!(failure.outcome, WorkspaceExecutionOutcome::Failed);
    assert!(failure.diagnostic_code.is_some());
    assert_eq!(failure.cancellation, None);
    assert_eq!(
        failure.observation.unwrap().reading,
        TemperatureReading::Pending
    );
}

#[test]
fn missing_completed_stream_or_unreached_target_terminates_without_replacing_current() {
    for reason in [
        UnavailableTemperature::MissingStream,
        UnavailableTemperature::TargetNotReached,
    ] {
        let mut app = mixer_project(320.);
        let mut owner = WorkspaceRunOwner::new();
        let previous = submit(&mut owner, &mut app, None);
        drain(&mut owner, &mut app);
        let mut condition = rule(
            &app,
            "stream-mix-out",
            TemperatureThreshold::parse("312", Kelvin).unwrap(),
        );
        // Inject a mismatch below the submission guard to exercise an incomplete solver payload.
        if reason == UnavailableTemperature::MissingStream {
            condition.outlet.stream = "missing".into();
        } else {
            condition.outlet.source.unit_id = "not-planned".into();
        }
        assert!(!condition.is_current(&app.workspace));
        let id = submit(&mut owner, &mut app, Some(condition));
        if reason == UnavailableTemperature::MissingStream {
            owner.wait_for_unaccepted_worker_result(now());
            owner.cancel(&mut app, &id);
        }
        let summary = drain(&mut owner, &mut app);
        assert_eq!(
            summary.cancellation,
            Some(WorkspaceRunCancelReason::ObservationUnavailable)
        );
        assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
        assert!(matches!(
            summary.disposition,
            FlowRunDisposition::Discarded(_)
        ));
        assert_eq!(
            summary.diagnostic_code.as_deref(),
            Some("workspace.run.observation_unavailable")
        );
        assert_eq!(current(&app), Some(previous));
        let expected_step = (reason == UnavailableTemperature::MissingStream).then_some(3);
        assert_eq!(
            summary.observation.unwrap().reading,
            TemperatureReading::Unavailable {
                step: expected_step,
                reason
            }
        );
        let WorkspaceTraceRecord::Recorded(trace) = summary.trace else {
            panic!("missing trace")
        };
        assert_eq!(trace.last_started, expected_step.unwrap_or(4));
    }
}

#[test]
fn malformed_observation_is_terminal_and_valid_rule_ignores_display_precision() {
    let app = mixer_project(320.);
    let threshold = TemperatureThreshold::parse("38.850000000", Celsius).unwrap();
    assert_eq!(threshold.raw(), "38.850000000");
    assert_eq!(threshold.in_unit(Kelvin).unwrap().raw(), "312");
    for raw in ["", "1e-", "NaN", "inf", "0", "-1", "42 Pa", "40 °C"] {
        assert!(TemperatureThreshold::parse(raw, Kelvin).is_err(), "{raw}");
    }
    assert!(
        TemperatureThreshold::parse("10", rf_types::units::MeasurementUnit::KelvinDifference)
            .is_err()
    );
    for invalid in [0., -1., f64::NAN, f64::INFINITY] {
        let condition = rule(&app, "stream-mix-out", threshold.clone());
        let gate = UnitControlGate::new();
        gate.arm(condition);
        let mut stream = app.workspace.document.flowsheet.streams[&"stream-mix-out".into()].clone();
        stream.temperature_k = invalid;
        let step = rf_solver::UnitSolveStep {
            index: 2,
            unit_id: "mixer-1".into(),
            unit_name: "Mixer".into(),
            unit_kind: "mixer".into(),
            consumed_streams: vec![],
            produced_streams: vec![stream],
            energy_balance: None,
            summary: String::new(),
        };
        assert!(
            gate.observe(&rf_solver::SequentialExecutionEvent::StepCompleted {
                number: 3,
                step: &step
            })
        );
        assert_eq!(
            gate.snapshot().observation.unwrap().reading,
            TemperatureReading::Unavailable {
                step: Some(3),
                reason: UnavailableTemperature::InvalidTemperature
            }
        );
        assert!(gate.before_unit(4, 4, &"flash-1".into()).is_break());
    }
}

#[test]
fn a_step_into_the_producer_and_a_condition_hit_share_one_release() {
    let mut app = mixer_project(320.);
    let mut owner = WorkspaceRunOwner::new();
    let condition = rule(
        &app,
        "stream-mix-out",
        TemperatureThreshold::parse("35", Celsius).unwrap(),
    );
    let id = submit(&mut owner, &mut app, Some(condition));
    let mut command = UnitRunCommand::Pause { last_released: 0 };
    for step in 1..=4 {
        assert_eq!(
            owner.control_units(&mut app, &id, command, now()),
            UnitControlReply::Accepted
        );
        until(|| {
            assert!(owner.poll(&mut app, now()).is_none());
            matches!(owner.state().unwrap().unit_control.unwrap().phase,UnitRunPhase::Paused { step: current,.. } if current==step)
        });
        command = UnitRunCommand::Step { step };
    }
    assert_eq!(
        owner.control_units(&mut app, &id, UnitRunCommand::Continue { step: 3 }, now()),
        UnitControlReply::Stale
    );
    owner.control_units(&mut app, &id, command, now());
    assert_eq!(
        drain(&mut owner, &mut app).outcome,
        WorkspaceExecutionOutcome::Succeeded
    );
}

#[test]
fn trace_truncation_does_not_change_the_watch_or_rearm_it_after_release() {
    use crate::solver_bridge::execution::WorkspaceSolveControl;
    use rf_solver::{SequentialExecutionEvent as Event, SolverControl};
    let mut app = mixer_project(320.);
    let mut owner = WorkspaceRunOwner::new();
    let id = submit(&mut owner, &mut app, None);
    drain(&mut owner, &mut app);
    let condition = rule(
        &app,
        "stream-mix-out",
        TemperatureThreshold::parse("25", Celsius).unwrap(),
    );
    let (channels, mut worker, _) = WorkerChannels::new(id, true, true);
    let gate = channels.units.as_ref().unwrap();
    gate.arm(condition);
    channels.start.send(true).unwrap();
    assert!(worker.ready().unwrap().is_continue());
    let mut plan = vec![rf_types::UnitId::new("other"); 2050];
    plan.push("mixer-1".into());
    plan.push("flash-1".into());
    worker.observe_sequential(Event::Planned(&plan));
    let mut step = rf_solver::UnitSolveStep {
        index: 0,
        unit_id: "other".into(),
        unit_name: String::new(),
        unit_kind: String::new(),
        consumed_streams: vec![],
        produced_streams: vec![],
        energy_balance: None,
        summary: String::new(),
    };
    for (index, unit) in plan.iter().take(2051).enumerate() {
        step.unit_id = unit.clone();
        if unit.as_str() == "mixer-1" {
            step.produced_streams
                .push(app.workspace.document.flowsheet.streams[&"stream-mix-out".into()].clone());
        }
        let number = index + 1;
        worker.observe_sequential(Event::StepStarted { number, unit });
        worker.observe_sequential(Event::StepCompleted {
            number,
            step: &step,
        });
    }
    let state = gate.snapshot();
    assert_eq!(state.phase, UnitRunPhase::PauseRequested);
    assert!(matches!(
        state.observation.as_ref().unwrap().reading,
        TemperatureReading::Measured { step: 2051, .. }
    ));
    // A repeated event with corrupted evidence cannot replace the already consumed reading.
    step.produced_streams.clear();
    worker.observe_sequential(Event::StepCompleted {
        number: 2051,
        step: &step,
    });
    assert_eq!(gate.snapshot(), state);
}
