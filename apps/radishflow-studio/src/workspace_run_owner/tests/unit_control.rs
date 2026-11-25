use super::*;
use UnitControlReply as Reply;
use UnitRunCommand::{Continue, Pause, Step};

fn current_run(app: &AppState) -> Option<FlowRunId> {
    match app.workspace.flow_result() {
        rf_ui::result_query::FlowResultView::Current { origin, .. } => {
            Some(origin.execution().unwrap().run_id.clone())
        }
        _ => None,
    }
}

fn start(owner: &mut WorkspaceRunOwner, app: &mut AppState) -> FlowRunId {
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
        panic!("expected accepted flow")
    };
    assert_eq!(
        owner.control_units(app, &id, Pause { last_released: 0 }, now()),
        Reply::Accepted
    );
    id
}

fn wait_at(owner: &mut WorkspaceRunOwner, app: &mut AppState, step: usize) -> rf_types::UnitId {
    let mut stopped = None;
    until(|| {
        assert!(
            owner.poll(app, now()).is_none(),
            "run ended before stop {step}"
        );
        if let UnitRunPhase::Paused {
            step: actual,
            total,
            unit,
        } = owner.state().unwrap().unit_control.unwrap().phase
        {
            assert_eq!((actual, total), (step, 3));
            stopped = Some(unit);
        }
        stopped.is_some()
    });
    stopped.unwrap()
}

#[test]
fn mixer_steps_release_one_entry_once_and_only_the_terminal_can_publish() {
    let mut app = AppState::new(FlowsheetDocument::new(
        mixer_flow(),
        DocumentMetadata::new("m", "Mixer", UNIX_EPOCH),
    ));
    let mut owner = WorkspaceRunOwner::new();
    for finish in [Continue { step: 3 }, Step { step: 3 }] {
        let previous = current_run(&app);
        let id = start(&mut owner, &mut app);
        assert_eq!(
            owner.control_units(&mut app, &id, Pause { last_released: 0 }, now()),
            Reply::AlreadyRequested
        );
        assert_eq!(
            owner.control_units(&mut app, &id, Continue { step: 1 }, now()),
            Reply::NotPaused
        );
        for (index, name) in ["feed-a", "feed-b", "mixer-1"].into_iter().enumerate() {
            let step = index + 1;
            assert_eq!(wait_at(&mut owner, &mut app, step).as_str(), name);
            assert_eq!(current_run(&app), previous);
            if step > 1 {
                assert_eq!(
                    owner.control_units(&mut app, &id, Continue { step: step - 1 }, now()),
                    Reply::Stale
                );
                assert_eq!(
                    owner.control_units(&mut app, &id, Pause { last_released: 0 }, now()),
                    Reply::Stale
                );
            }
            let command = if step == 3 { finish } else { Step { step } };
            assert_eq!(
                owner.control_units(&mut app, &id, command, now()),
                Reply::Accepted
            );
        }
        let summary = drain(&mut owner, &mut app);
        assert_eq!(summary.disposition, FlowRunDisposition::Applied);
        assert_eq!(
            owner.control_units(&mut app, &id, finish, now()),
            Reply::Finished
        );
        let WorkspaceTraceRecord::Recorded(trace) = summary.trace else {
            panic!("expected trace")
        };
        assert_eq!(trace.steps.len(), 3);
        assert!(
            trace
                .steps
                .iter()
                .all(|s| s.completed_sequence > Some(s.started_sequence))
        );
        let rf_ui::result_query::FlowResultView::Current { snapshot, origin } =
            app.workspace.flow_result()
        else {
            panic!("expected result")
        };
        assert_eq!(origin.execution().unwrap().run_id, id);
        assert_eq!(
            snapshot
                .streams
                .iter()
                .find(|s| s.stream_id.as_str() == "stream-mix-out")
                .unwrap()
                .total_molar_flow_mol_s,
            5.
        );
    }
}

#[test]
fn paused_run_revalidates_immediately_and_all_departure_paths_wake_the_worker() {
    for reason in [
        WorkspaceRunCancelReason::QualificationChanged,
        WorkspaceRunCancelReason::Authorization(RunAuthorizationRefusal::Changed),
        WorkspaceRunCancelReason::Detached,
        WorkspaceRunCancelReason::Closing,
        WorkspaceRunCancelReason::UserRequested,
    ] {
        let mut app = app();
        grant(&mut app, now() + Duration::from_secs(3600));
        let mut owner = WorkspaceRunOwner::new();
        let id = start(&mut owner, &mut app);
        wait_at(&mut owner, &mut app, 1);
        match reason {
            WorkspaceRunCancelReason::ObservationUnavailable => unreachable!(),
            WorkspaceRunCancelReason::QualificationChanged => rename(&mut app),
            WorkspaceRunCancelReason::Authorization(_) => app
                .entitlement
                .snapshot
                .as_mut()
                .unwrap()
                .allowed_package_ids
                .clear(),
            WorkspaceRunCancelReason::Detached => owner.detach(&mut app),
            WorkspaceRunCancelReason::Closing => {
                owner.begin_shutdown(Some(&mut app));
                owner.resume_submissions();
            }
            WorkspaceRunCancelReason::UserRequested => {
                assert_eq!(
                    owner.cancel(&mut app, &id),
                    WorkspaceRunCancelReply::Requested
                );
            }
        }
        // A stale UI can issue the action before the next owner poll.
        assert_eq!(
            owner.control_units(&mut app, &id, Step { step: 1 }, now()),
            Reply::Cancelling
        );
        let summary = drain(&mut owner, &mut app);
        assert_eq!(summary.cancellation, Some(reason));
        assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
        assert_ne!(summary.disposition, FlowRunDisposition::Applied);
        let WorkspaceTraceRecord::Recorded(trace) = summary.trace else {
            panic!("expected trace")
        };
        assert_eq!(trace.last_started, 0);
    }
}

#[test]
fn pause_during_a_real_unit_call_waits_for_the_next_entry_or_the_original_error() {
    use rf_flash::{TpFlashInput, TpFlashResult, TpFlashSolver};
    use rf_solver::{ControlledSolveOutcome, SequentialModularSolver, SolverServices};
    struct HeldFlash<'a> {
        delegate: &'a dyn TpFlashSolver,
        entered: mpsc::SyncSender<()>,
        release: mpsc::Receiver<()>,
        held_stream: &'static str,
        fail: bool,
    }
    impl TpFlashSolver for HeldFlash<'_> {
        fn flash(
            &self,
            thermo: &dyn rf_thermo::ThermoProvider,
            input: &TpFlashInput,
        ) -> RfResult<TpFlashResult> {
            if input.stream_id.as_str() == self.held_stream {
                self.entered.send(()).unwrap();
                self.release.recv_timeout(Duration::from_secs(10)).unwrap();
                if self.fail {
                    return Err(rf_types::RfError::invalid_input("held unit failed"));
                }
            }
            self.delegate.flash(thermo, input)
        }
    }
    for (held_stream, fail) in [
        ("stream-feed-a", false),
        ("stream-feed-a", true),
        ("stream-mix-out", false),
        ("stream-mix-out", true),
    ] {
        let flow = mixer_flow();
        let id = app().prepare_flow_run().receipt().run_id().clone();
        let (channels, mut worker, _) = WorkerChannels::new(id, true, true);
        let gate = channels.units.as_ref().unwrap();
        let (entered, inside) = mpsc::sync_channel(1);
        let (resume, release) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let Ok(context) = load()(PACKAGE) else {
                panic!("expected fixture package")
            };
            let flash = HeldFlash {
                delegate: context.tp_flash(),
                entered,
                release,
                held_stream,
                fail,
            };
            SequentialModularSolver.solve_controlled(
                &SolverServices {
                    thermo: context.thermo(),
                    flash_solver: &flash,
                },
                &flow,
                &mut worker,
            )
        });
        inside.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(gate.request(Pause { last_released: 0 }), Reply::Accepted);
        assert_eq!(gate.snapshot().phase, UnitRunPhase::PauseRequested);
        resume.send(()).unwrap();
        if fail {
            assert!(
                handle
                    .join()
                    .unwrap()
                    .unwrap_err()
                    .message()
                    .contains("held unit failed")
            );
        } else if held_stream == "stream-mix-out" {
            assert!(matches!(
                handle.join().unwrap().unwrap(),
                ControlledSolveOutcome::Completed(_)
            ));
        } else {
            until(|| matches!(gate.snapshot().phase, UnitRunPhase::Paused { step: 2, .. }));
            channels.request_cancel();
            assert_eq!(
                handle.join().unwrap().unwrap(),
                ControlledSolveOutcome::Cancelled
            );
        }
    }
}

#[test]
fn undoing_a_plan_change_cannot_resume_its_old_worker() {
    use rf_ui::flow_plan::{FlowPlanCommand, FlowPlanEditContext};
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let old = start(&mut owner, &mut app);
    wait_at(&mut owner, &mut app, 1);
    let config = FlowPlanConfiguration::SinglePhase(
        rf_solver::flow_plan::SinglePhaseOptions::new(
            [("methane".into(), 30.0), ("ethane".into(), 50.0)].into(),
            200.0..=550.0,
            100_000.0,
            Default::default(),
        )
        .unwrap(),
    );
    let apply = |app: &mut AppState, command| {
        assert!(
            app.workspace
                .apply_flow_plan(&FlowPlanEditContext::capture(&app.workspace), command)
                .unwrap()
        );
    };
    apply(&mut app, FlowPlanCommand::Apply(config));
    apply(&mut app, FlowPlanCommand::Undo);
    assert_eq!(
        app.workspace.flow_plan.current(),
        &FlowPlanConfiguration::Acyclic
    );
    assert_eq!(
        owner.control_units(&mut app, &old, Continue { step: 1 }, now()),
        Reply::Cancelling
    );
    assert_eq!(
        drain(&mut owner, &mut app).outcome,
        WorkspaceExecutionOutcome::Cancelled
    );
    let next = start(&mut owner, &mut app);
    assert_eq!(
        owner.control_units(&mut app, &old, Step { step: 1 }, now()),
        Reply::Finished
    );
    assert_eq!(wait_at(&mut owner, &mut app, 1).as_str(), "feed-1");
    owner.cancel(&mut app, &next);
    drain(&mut owner, &mut app);
}

#[test]
fn empty_plan_finishes_without_a_stop_and_late_pause_cannot_change_its_receipt() {
    let mut flow = mixer_flow();
    flow.units.clear();
    flow.streams.clear();
    let mut app = AppState::new(FlowsheetDocument::new(
        flow,
        DocumentMetadata::new("empty", "Empty", UNIX_EPOCH),
    ));
    let mut owner = WorkspaceRunOwner::new();
    // Bypass the UI preflight to exercise the worker's zero-entry plan.
    let req = request(&app);
    let auth = RunAuthorization::capture(&app, PACKAGE, now()).unwrap();
    let WorkspaceRunSubmission::Accepted(id) = owner
        .spawn(&mut app, req, auth, load(), true, now(), &ThreadSpawner)
        .unwrap()
    else {
        panic!("expected worker")
    };
    assert_eq!(
        owner.control_units(&mut app, &id, Pause { last_released: 0 }, now()),
        Reply::Accepted
    );
    owner.wait_for_unaccepted_worker_result(now());
    assert_eq!(
        owner.control_units(&mut app, &id, Pause { last_released: 0 }, now()),
        Reply::Finished
    );
    let summary = drain(&mut owner, &mut app);
    let WorkspaceTraceRecord::Recorded(trace) = summary.trace else {
        panic!("expected trace")
    };
    assert_eq!(trace.planned_steps, Some(0));
    assert_eq!(trace.last_started, 0);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(summary.disposition, FlowRunDisposition::Applied);
}
