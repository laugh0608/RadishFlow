use super::*;
use crate::solver_bridge::execution::WorkspacePackageLoadError;
use crate::test_support::{
    OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID as PACKAGE,
    build_official_binary_hydrocarbon_in_memory_provider,
    write_default_official_binary_hydrocarbon_cached_package,
};
use rf_property_package::{PropertyPackageProvider, prepare_local_context};
use rf_ui::{DocumentMetadata, FlowsheetDocument, RunStatus};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

fn app() -> AppState {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("same-project", "Project", UNIX_EPOCH),
    ))
}
fn now() -> SystemTime {
    SystemTime::now()
}
fn request(app: &AppState) -> StudioSolveRequest {
    build_workspace_solve_request(app, PACKAGE).unwrap()
}
fn load() -> PackageLoader {
    Box::new(|id| {
        build_official_binary_hydrocarbon_in_memory_provider(PACKAGE)
            .load_package(id, now())
            .and_then(prepare_local_context)
            .map_err(WorkspacePackageLoadError::Package)
    })
}
fn spawn(
    owner: &mut WorkspaceRunOwner,
    app: &mut AppState,
    loader: PackageLoader,
    spawner: &impl WorkerSpawner,
) -> FlowRunId {
    assert!(owner.is_idle());
    let request = request(app);
    let authorization = RunAuthorization::capture(app, PACKAGE, now()).unwrap();
    match owner
        .spawn(app, request, authorization, loader, false, now(), spawner)
        .unwrap()
    {
        WorkspaceRunSubmission::Accepted(id) => id,
        _ => panic!("expected accepted request"),
    }
}
fn until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "worker did not reach expected boundary"
        );
        thread::yield_now();
    }
}
fn drain(owner: &mut WorkspaceRunOwner, app: &mut AppState) -> WorkspaceRunSummary {
    let mut summary = None;
    until(|| {
        summary = owner.poll(app, now());
        summary.is_some()
    });
    assert!(owner.is_idle());
    summary.unwrap()
}
fn wait_ready(owner: &WorkspaceRunOwner) {
    until(|| {
        owner
            .active
            .as_ref()
            .unwrap()
            .channels
            .progress
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|p| p.progress == crate::WorkspaceSolveProgress::Ready)
    });
}
fn cache() -> (PropertyPackageCacheRoot, StoredAuthCacheIndex) {
    let root = PropertyPackageCacheRoot::temporary("af3b-test").unwrap();
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user",
        rf_store::StoredCredentialReference::new("studio", "user"),
    );
    write_default_official_binary_hydrocarbon_cached_package(root.path(), &mut index);
    (root, index)
}
fn rename(app: &mut AppState) {
    let mut flow = app.workspace.document.flowsheet.clone();
    flow.units.get_mut(&"heater-1".into()).unwrap().name = "changed".into();
    app.commit_document_change(
        rf_ui::DocumentCommand::RenameUnit {
            unit_id: "heater-1".into(),
            new_name: "changed".into(),
        },
        flow,
        now(),
    );
}

#[test]
fn real_cached_background_matches_sync_without_executing_on_the_submitter() {
    let (root, index) = cache();
    let mut app = app();
    let mut synchronous = app.clone();
    crate::solve_workspace_from_auth_cache(&mut synchronous, root.path(), &index, &request(&app))
        .unwrap();
    let mut owner = WorkspaceRunOwner::new();
    assert!(matches!(
        owner
            .submit_cached(
                &mut app,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Accepted(_)
    ));
    assert!(app.workspace.snapshot_history.is_empty());
    let result = drain(&mut owner, &mut app);
    assert_eq!(result.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(result.disposition, FlowRunDisposition::Applied);
    assert!(result.started_at.is_some());
    let rf_ui::result_query::FlowResultView::Current {
        snapshot: actual,
        origin: actual_origin,
    } = app.workspace.flow_result()
    else {
        panic!("background current")
    };
    let rf_ui::result_query::FlowResultView::Current {
        snapshot: expected,
        origin: expected_origin,
    } = synchronous.workspace.flow_result()
    else {
        panic!("inline current")
    };
    let actual_execution = actual_origin.execution().unwrap();
    let expected_execution = expected_origin.execution().unwrap();
    assert_ne!(actual_execution.run_id, expected_execution.run_id);
    assert_eq!(actual_execution.package, expected_execution.package);
    assert_eq!(
        actual_execution.solver_implementation,
        expected_execution.solver_implementation
    );
    assert_eq!(actual_origin.document_id(), expected_origin.document_id());
    assert_eq!(actual_origin.revision(), expected_origin.revision());
    let mut actual = actual.clone();
    actual.origin = None;
    let mut expected = expected.clone();
    expected.origin = None;
    assert_eq!(actual, expected);
    assert!(owner.try_close().is_ok());
}

#[test]
fn submission_is_nonblocking_and_duplicate_or_busy_does_not_change_qualification() {
    let (root, index) = cache();
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let caller = thread::current().id();
    let id = spawn(
        &mut owner,
        &mut app,
        Box::new(move |id| {
            entered_tx.send(thread::current().id()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load()(id)
        }),
        &ThreadSpawner,
    );
    assert_ne!(
        entered.recv_timeout(Duration::from_secs(10)).unwrap(),
        caller
    );
    let before = app.clone();
    assert_eq!(
        owner
            .submit_cached(
                &mut app,
                root.clone(),
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::AlreadyRunning(id.clone())
    );
    assert_eq!(app, before);
    rename(&mut app);
    let changed = app.clone();
    assert_eq!(
        owner
            .submit_cached(
                &mut app,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Busy(id)
    );
    assert_eq!(app, changed);
    owner.poll(&mut app, now());
    assert_eq!(
        owner.state().unwrap().phase,
        WorkspaceRunPhase::CancelRequested
    );
    assert!(!owner.is_idle());
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
    assert_eq!(
        summary.cancellation,
        Some(WorkspaceRunCancelReason::QualificationChanged)
    );
    assert_eq!(app, changed);
}

#[test]
fn cancellation_waits_for_loading_and_never_reports_stopped_early() {
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let id = spawn(
        &mut owner,
        &mut app,
        Box::new(move |id| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load()(id)
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(
        owner.cancel(&mut app, &id),
        WorkspaceRunCancelReply::Requested
    );
    assert_eq!(
        owner.cancel(&mut app, &id),
        WorkspaceRunCancelReply::AlreadyRequested
    );
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Hold);
    assert!(owner.poll(&mut app, now()).is_none());
    assert_eq!(
        owner.state().unwrap().phase,
        WorkspaceRunPhase::CancelRequested
    );
    owner = owner
        .try_close()
        .expect_err("live worker retains its owner");
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::AcceptanceRevoked)
    );
    assert_eq!(
        owner.cancel(&mut app, &id),
        WorkspaceRunCancelReply::AlreadyFinished
    );
}

struct GatedSpawner {
    entered: mpsc::SyncSender<()>,
    release: std::sync::Mutex<Option<mpsc::Receiver<()>>>,
}
impl WorkerSpawner for GatedSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        let entered = self.entered.clone();
        let release = self.release.lock().unwrap().take().unwrap();
        thread::Builder::new().spawn(move || {
            job();
            entered.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(10)).unwrap();
        })
    }
}

#[test]
fn finished_payload_cancel_race_keeps_execution_success_and_discards_publication() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let spawner = GatedSpawner {
        entered: entered_tx,
        release: std::sync::Mutex::new(Some(released)),
    };
    let id = spawn(&mut owner, &mut app, load(), &spawner);
    wait_ready(&owner);
    until(|| {
        owner.poll(&mut app, now());
        owner.state().unwrap().started_at.is_some()
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(
        owner.poll(&mut app, now()).is_none(),
        "terminal payload alone must not free worker slot"
    );
    assert!(app.workspace.snapshot_history.is_empty());
    owner.cancel(&mut app, &id);
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert!(matches!(
        summary.disposition,
        FlowRunDisposition::Discarded(_)
    ));
    assert_eq!(
        summary.cancellation,
        Some(WorkspaceRunCancelReason::UserRequested)
    );
    assert!(app.workspace.snapshot_history.is_empty());
}

#[test]
fn applied_completion_wins_over_later_cancel_and_hold_does_not_cancel() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let id = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    wait_ready(&owner);
    app.set_simulation_mode(SimulationMode::Hold);
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    let before = app.clone();
    assert_eq!(
        owner.cancel(&mut app, &id),
        WorkspaceRunCancelReply::AlreadyFinished
    );
    assert_eq!(app, before);
    let unknown = app.prepare_flow_run().receipt().run_id().clone();
    assert_eq!(
        owner.cancel(&mut app, &unknown),
        WorkspaceRunCancelReply::UnknownRun
    );
}

#[test]
fn cancel_at_ready_wakes_worker_without_granting_numeric_start() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let id = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    wait_ready(&owner);
    owner.cancel(&mut app, &id);
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.started_at, None);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
}

#[test]
fn temporary_cache_lease_survives_old_session_drop_and_ends_after_loading() {
    let (root, index) = cache();
    let path = root.path().to_path_buf();
    let worker_lease = root.clone();
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    spawn(
        &mut owner,
        &mut app,
        Box::new(move |id| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load_cached_context(worker_lease.path(), &index, id)
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    drop(root);
    assert!(path.exists());
    release.send(()).unwrap();
    wait_ready(&owner);
    assert!(
        !path.exists(),
        "load lease must not retain the cache throughout solving"
    );
    assert_eq!(
        drain(&mut owner, &mut app).outcome,
        WorkspaceExecutionOutcome::Succeeded
    );
}

#[test]
fn detached_old_worker_holds_slot_and_cannot_change_reopened_document() {
    let (root, index) = cache();
    let mut old = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let id = spawn(
        &mut owner,
        &mut old,
        Box::new(move |id| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load()(id)
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    owner.detach(&mut old);
    drop(old);
    let mut reopened = app();
    let before = reopened.clone();
    assert_eq!(
        owner
            .submit_cached(
                &mut reopened,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Busy(id)
    );
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut reopened);
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession)
    );
    assert_eq!(reopened, before);
}

struct FailingSpawner;
impl WorkerSpawner for FailingSpawner {
    fn spawn(&self, _: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        Err(std::io::Error::other("injected spawn failure"))
    }
}
struct SilentSpawner;
impl WorkerSpawner for SilentSpawner {
    fn spawn(&self, _: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        thread::Builder::new().spawn(|| {})
    }
}

#[test]
fn spawn_panic_and_disconnected_terminal_are_distinct_failures_and_release_qualification() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let authorization = RunAuthorization::capture(&app, PACKAGE, now()).unwrap();
    let req = request(&app);
    assert!(
        owner
            .spawn(
                &mut app,
                req,
                authorization,
                load(),
                false,
                now(),
                &FailingSpawner
            )
            .is_err()
    );
    assert!(owner.is_idle());
    assert!(app.workspace.pending_flow_run().is_none());
    assert_eq!(
        owner.history().back().unwrap().diagnostic_code.as_deref(),
        Some("workspace.run.spawn_failed")
    );
    spawn(
        &mut owner,
        &mut app,
        Box::new(|_| panic!("injected worker panic")),
        &ThreadSpawner,
    );
    let panic = drain(&mut owner, &mut app);
    assert_eq!(
        panic.outcome,
        WorkspaceExecutionOutcome::InfrastructureFailed
    );
    assert_eq!(
        panic.diagnostic_code.as_deref(),
        Some("workspace.run.worker_panicked")
    );
    spawn(&mut owner, &mut app, load(), &SilentSpawner);
    let disconnected = drain(&mut owner, &mut app);
    assert_eq!(
        disconnected.diagnostic_code.as_deref(),
        Some("workspace.run.completion_disconnected")
    );
    assert!(app.workspace.pending_flow_run().is_none());
    assert_eq!(app.workspace.solve_session.status, RunStatus::Error);
}

#[test]
fn late_worker_panic_is_recorded_without_polluting_new_input() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    spawn(
        &mut owner,
        &mut app,
        Box::new(move |_| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            panic!("late worker panic")
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    rename(&mut app);
    let before = app.clone();
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(
        summary.outcome,
        WorkspaceExecutionOutcome::InfrastructureFailed
    );
    assert!(matches!(
        summary.disposition,
        FlowRunDisposition::Discarded(_)
    ));
    assert_eq!(app, before);
}

#[test]
fn shutdown_is_nonblocking_and_cancel_exit_never_uncancels_the_worker() {
    let (root, index) = cache();
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    wait_ready(&owner);
    owner.begin_shutdown(Some(&mut app));
    assert!(matches!(
        owner
            .submit_cached(
                &mut app,
                root.clone(),
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Blocked(WorkspaceRunBlock::Closing)
    ));
    owner.resume_submissions();
    assert!(matches!(
        owner
            .submit_cached(
                &mut app,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Busy(_)
    ));
    let mut terminal = None;
    until(|| {
        terminal = owner.poll_detached(now());
        terminal.is_some()
    });
    assert_eq!(
        terminal.unwrap().outcome,
        WorkspaceExecutionOutcome::Cancelled
    );
    assert!(owner.try_close().is_ok());
}

#[test]
fn summary_history_is_bounded_across_one_hundred_real_background_runs() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let mut first = None;
    for i in 0..100 {
        let id = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
        if i == 0 {
            first = Some(id);
        } else {
            assert_ne!(Some(&id), first.as_ref());
        }
        assert_eq!(
            drain(&mut owner, &mut app).outcome,
            WorkspaceExecutionOutcome::Succeeded
        );
        assert!(owner.history().len() <= HISTORY_LIMIT);
    }
    assert_eq!(owner.history().len(), HISTORY_LIMIT);
    assert_eq!(
        owner.cancel(&mut app, first.as_ref().unwrap()),
        WorkspaceRunCancelReply::UnknownRun
    );
    assert!(owner.try_close().is_ok());
}

struct BeforeSpawner {
    entered: mpsc::SyncSender<()>,
    release: std::sync::Mutex<Option<mpsc::Receiver<()>>>,
}
impl WorkerSpawner for BeforeSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        let entered = self.entered.clone();
        let release = self.release.lock().unwrap().take().unwrap();
        thread::Builder::new().spawn(move || {
            entered.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(10)).unwrap();
            job();
        })
    }
}
#[test]
fn cancel_before_loading_prevents_all_cache_io() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let spawner = BeforeSpawner {
        entered: entered_tx,
        release: std::sync::Mutex::new(Some(released)),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let id = spawn(
        &mut owner,
        &mut app,
        Box::new(move |id| {
            count.fetch_add(1, Ordering::Relaxed);
            load()(id)
        }),
        &spawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    owner.cancel(&mut app, &id);
    release.send(()).unwrap();
    assert_eq!(
        drain(&mut owner, &mut app).outcome,
        WorkspaceExecutionOutcome::Cancelled
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

fn grant(app: &mut AppState, expires: SystemTime) {
    app.update_entitlement(
        rf_ui::EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user".into(),
            tenant_id: None,
            issued_at: UNIX_EPOCH,
            expires_at: expires,
            offline_lease_expires_at: None,
            features: Default::default(),
            allowed_package_ids: [PACKAGE.into()].into_iter().collect(),
        },
        vec![],
        now(),
    );
}

#[test]
fn ready_rechecks_frozen_authorization_deadline_and_current_permission() {
    for expired in [false, true] {
        let mut app = app();
        let deadline = now() + Duration::from_secs(3600);
        grant(&mut app, deadline);
        let mut owner = WorkspaceRunOwner::new();
        spawn(&mut owner, &mut app, load(), &ThreadSpawner);
        wait_ready(&owner);
        if !expired {
            app.entitlement
                .snapshot
                .as_mut()
                .unwrap()
                .allowed_package_ids
                .clear();
        }
        let at = if expired { deadline } else { now() };
        let mut summary = None;
        until(|| {
            summary = owner.poll(&mut app, at);
            summary.is_some()
        });
        let summary = summary.unwrap();
        assert_eq!(summary.started_at, None);
        assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
        assert_eq!(
            summary.cancellation,
            Some(WorkspaceRunCancelReason::Authorization(if expired {
                RunAuthorizationRefusal::Expired
            } else {
                RunAuthorizationRefusal::Changed
            }))
        );
        assert!(app.workspace.snapshot_history.is_empty());
    }
}

#[test]
fn after_start_elapsed_time_preserves_results_but_explicit_revocation_discards_them() {
    for revoke in [false, true] {
        let mut app = app();
        let deadline = now() + Duration::from_secs(3600);
        grant(&mut app, deadline);
        let mut owner = WorkspaceRunOwner::new();
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        let spawner = GatedSpawner {
            entered: entered_tx,
            release: std::sync::Mutex::new(Some(released)),
        };
        spawn(&mut owner, &mut app, load(), &spawner);
        wait_ready(&owner);
        until(|| {
            owner.poll(&mut app, now());
            owner.state().unwrap().started_at.is_some()
        });
        entered.recv_timeout(Duration::from_secs(10)).unwrap();
        if revoke {
            app.entitlement
                .snapshot
                .as_mut()
                .unwrap()
                .allowed_package_ids
                .clear();
        }
        release.send(()).unwrap();
        let mut summary = None;
        until(|| {
            summary = owner.poll(&mut app, deadline + Duration::from_secs(10));
            summary.is_some()
        });
        let summary = summary.unwrap();
        assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
        assert_eq!(summary.disposition == FlowRunDisposition::Applied, !revoke);
        assert_eq!(app.workspace.snapshot_history.len(), usize::from(!revoke));
    }
}

#[test]
fn changed_input_at_ready_cancels_without_touching_active_mode_or_restarting_automatically() {
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut owner = WorkspaceRunOwner::new();
    spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    wait_ready(&owner);
    rename(&mut app);
    let before = app.clone();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.started_at, None);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Cancelled);
    assert_eq!(app, before);
    assert!(owner.poll(&mut app, now()).is_none());
    assert_eq!(owner.history().len(), 1);
}

#[test]
fn ten_thousand_progress_updates_coalesce_without_blocking_terminal_delivery() {
    use rf_solver::{SolveProgress, SolveStage, SolverControl};
    let mut app = app();
    let id = app.prepare_flow_run().receipt().run_id().clone();
    let (channels, mut control, terminal) = WorkerChannels::new(id.clone());
    for completed_units in 0..10_000 {
        assert!(
            control
                .checkpoint(SolveProgress {
                    stage: SolveStage::UnitCompleted,
                    total_units: Some(10_000),
                    completed_units,
                    current_unit: None
                })
                .is_continue()
        );
    }
    let latest = channels.progress.lock().unwrap().clone().unwrap();
    assert_eq!(latest.run_id, id);
    assert_eq!(latest.sequence, 10_000);
    terminal.send(WorkerResult::Panicked).unwrap();
    assert!(matches!(
        channels.result.try_recv().unwrap(),
        WorkerResult::Panicked
    ));
    assert_eq!(channels.progress.lock().unwrap().take(), Some(latest));
    assert!(channels.progress.lock().unwrap().is_none());
}

#[test]
fn old_duplicate_and_out_of_order_progress_never_regress_current_projection() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let spawner = GatedSpawner {
        entered: entered_tx,
        release: std::sync::Mutex::new(Some(released)),
    };
    let id = spawn(&mut owner, &mut app, load(), &spawner);
    wait_ready(&owner);
    until(|| {
        owner.poll(&mut app, now());
        owner.state().unwrap().started_at.is_some()
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let current = WorkspaceRunProgress {
        run_id: id.clone(),
        sequence: 100,
        progress: crate::WorkspaceSolveProgress::Ready,
    };
    *owner
        .active
        .as_ref()
        .unwrap()
        .channels
        .progress
        .lock()
        .unwrap() = Some(current.clone());
    owner.poll(&mut app, now());
    assert_eq!(owner.state().unwrap().progress, Some(current.clone()));
    let foreign = app.clone().prepare_flow_run().receipt().run_id().clone();
    for (run_id, sequence) in [(foreign, 101), (id.clone(), 100), (id, 99)] {
        *owner
            .active
            .as_ref()
            .unwrap()
            .channels
            .progress
            .lock()
            .unwrap() = Some(WorkspaceRunProgress {
            run_id,
            sequence,
            progress: crate::WorkspaceSolveProgress::Preparing,
        });
        owner.poll(&mut app, now());
        assert_eq!(owner.state().unwrap().progress, Some(current.clone()));
    }
    release.send(()).unwrap();
    assert_eq!(
        drain(&mut owner, &mut app).outcome,
        WorkspaceExecutionOutcome::Succeeded
    );
}

#[test]
fn input_preflight_skip_and_authorization_refusal_never_register_or_spawn() {
    let (root, index) = cache();
    for scenario in 0..5 {
        let mut app = app();
        let command = match scenario {
            0 => WorkspaceRunCommand::automatic_preferred(),
            1 => WorkspaceRunCommand::manual("missing"),
            2 => {
                app.focus_inspector_target(rf_ui::InspectorTarget::Unit("heater-1".into()));
                app.update_unit_inspector_draft(
                    &"heater-1".into(),
                    rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                    "400",
                )
                .unwrap();
                WorkspaceRunCommand::manual(PACKAGE)
            }
            3 => {
                app.workspace.document.flowsheet.components.clear();
                WorkspaceRunCommand::manual(PACKAGE)
            }
            _ => {
                grant(&mut app, UNIX_EPOCH);
                WorkspaceRunCommand::manual(PACKAGE)
            }
        };
        let before = app.clone();
        let mut owner = WorkspaceRunOwner::new();
        let result = owner
            .submit_cached(&mut app, root.clone(), &index, &command, now())
            .unwrap();
        assert!(
            matches!(
                result,
                WorkspaceRunSubmission::Blocked(_) | WorkspaceRunSubmission::Skipped(_)
            ),
            "scenario {scenario}: {result:?}"
        );
        assert!(owner.is_idle());
        assert!(owner.history().is_empty());
        assert_eq!(app, before);
    }
}

#[test]
fn package_expiry_is_checked_at_loading_even_when_submission_metadata_was_valid() {
    let (root, mut index) = cache();
    let mut app = app();
    // Resource eligibility is a loader fact; the selected index still identifies it.
    for package in &mut index.property_packages {
        package.expires_at = Some(UNIX_EPOCH);
    }
    let mut owner = WorkspaceRunOwner::new();
    assert!(matches!(
        owner
            .submit_cached(
                &mut app,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                now()
            )
            .unwrap(),
        WorkspaceRunSubmission::Accepted(_)
    ));
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Failed);
    assert_eq!(summary.started_at, None);
    assert_eq!(
        summary.diagnostic_code.as_deref(),
        Some("workspace.run.property_package_missing")
    );
    assert_eq!(app.workspace.solve_session.status, RunStatus::Error);
}

#[test]
fn cancellation_during_failing_load_preserves_actual_error_as_discarded_evidence() {
    let mut app = app();
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let id = spawn(
        &mut owner,
        &mut app,
        Box::new(move |_| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load()("missing")
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    owner.cancel(&mut app, &id);
    let before = app.clone();
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Failed);
    assert!(matches!(
        summary.disposition,
        FlowRunDisposition::Discarded(_)
    ));
    assert_eq!(
        summary.diagnostic_code.as_deref(),
        Some("workspace.run.property_package_missing")
    );
    assert_eq!(app, before);
}

#[test]
fn disconnected_ready_and_start_handshakes_are_infrastructure_failures() {
    for disconnect_ready in [false, true] {
        let mut app = app();
        let mut owner = WorkspaceRunOwner::new();
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        let spawner = BeforeSpawner {
            entered: entered_tx,
            release: std::sync::Mutex::new(Some(released)),
        };
        spawn(&mut owner, &mut app, load(), &spawner);
        entered.recv_timeout(Duration::from_secs(10)).unwrap();
        if disconnect_ready {
            let (_, replacement) = mpsc::sync_channel(1);
            drop(std::mem::replace(
                &mut owner.active.as_mut().unwrap().channels.ready,
                replacement,
            ));
        } else {
            let (replacement, _) = mpsc::sync_channel(1);
            drop(std::mem::replace(
                &mut owner.active.as_mut().unwrap().channels.start,
                replacement,
            ));
        }
        release.send(()).unwrap();
        let summary = drain(&mut owner, &mut app);
        assert_eq!(
            summary.outcome,
            WorkspaceExecutionOutcome::InfrastructureFailed
        );
        assert_eq!(summary.started_at, None);
        assert_eq!(
            summary.diagnostic_code.as_deref(),
            Some(if disconnect_ready {
                "workspace.run.ready_disconnected"
            } else {
                "workspace.run.start_disconnected"
            })
        );
        assert!(app.workspace.pending_flow_run().is_none());
        assert_eq!(app.workspace.solve_session.status, RunStatus::Error);
    }
}

#[test]
fn explicit_cancel_after_automatic_invalidation_also_holds_future_runs() {
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut owner = WorkspaceRunOwner::new();
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let id = spawn(
        &mut owner,
        &mut app,
        Box::new(move |id| {
            entered_tx.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            load()(id)
        }),
        &ThreadSpawner,
    );
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    rename(&mut app);
    owner.poll(&mut app, now());
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Active);
    assert_eq!(
        owner.cancel(&mut app, &id),
        WorkspaceRunCancelReply::AlreadyRequested
    );
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Hold);
    release.send(()).unwrap();
    drain(&mut owner, &mut app);
}

#[test]
fn current_cache_preparation_failure_is_published_and_clears_its_pending_attempt() {
    let (_, index) = cache();
    let root = PropertyPackageCacheRoot::temporary("af3b-missing-payload").unwrap();
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut owner = WorkspaceRunOwner::new();
    owner
        .submit_cached(
            &mut app,
            root,
            &index,
            &WorkspaceRunCommand::manual(PACKAGE),
            now(),
        )
        .unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Failed);
    assert_eq!(summary.disposition, FlowRunDisposition::Applied);
    assert_eq!(
        summary.diagnostic_code.as_deref(),
        Some("workspace.run.local_cache_unavailable")
    );
    assert_eq!(app.workspace.solve_session.status, RunStatus::Error);
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Hold);
    assert!(app.workspace.pending_flow_run().is_none());
    assert!(app.workspace.snapshot_history.is_empty());
}

mod partitioned;

mod simultaneous;
