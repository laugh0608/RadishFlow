mod action_receipts;
use super::*;
use crate::test_support::{
    OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID as PACKAGE,
    write_default_official_binary_hydrocarbon_cached_package,
};
use rf_ui::engineering::{
    EngineeringInputs, EngineeringSession, EquipmentCaseKey, EquipmentResultStatus,
};
use rf_ui::{DocumentMetadata, FlowRunDisposition, FlowsheetDocument};
use std::{
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

fn app() -> AppState {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    let mut app = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("project", "Project", UNIX_EPOCH),
    ));
    let rf_store::project_v3::ProjectV3Read::Editable(project) = rf_store::project_v3::parse(
        include_str!("../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!("editable")
    };
    let engineering = project.engineering.unwrap();
    app.workspace.engineering = EngineeringSession::from_inputs(
        "project".into(),
        EngineeringInputs {
            basis: engineering.basis,
            tasks: engineering.tasks,
        },
    )
    .unwrap();
    app
}
fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }
}
fn command() -> EquipmentEditorCommand {
    EquipmentEditorCommand::CalculateRating(member())
}
fn cache() -> (PropertyPackageCacheRoot, StoredAuthCacheIndex) {
    let root = PropertyPackageCacheRoot::temporary("af3c-test").unwrap();
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user",
        rf_store::StoredCredentialReference::new("studio", "user"),
    );
    write_default_official_binary_hydrocarbon_cached_package(root.path(), &mut index);
    (root, index)
}
fn until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !predicate() {
        assert!(Instant::now() < deadline, "worker failed to reach boundary");
        thread::yield_now();
    }
}
fn drain(owner: &mut StudioRunOwner, app: &mut AppState) {
    until(|| {
        owner.poll(Some(app), SystemTime::now());
        owner.is_idle()
    });
}
fn equipment(owner: &mut StudioRunOwner, app: &mut AppState) -> StudioRunId {
    match owner
        .submit_equipment(app, &command(), SystemTime::now())
        .unwrap()
    {
        StudioRunSubmission::Accepted(id) => id,
        other => panic!("expected accepted, got {other:?}"),
    }
}

#[test]
fn one_worker_quota_across_flow_and_equipment_busy_never_freezes_or_queues() {
    let (root, index) = cache();
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    let reply = owner
        .submit_flow(
            &mut app,
            root.clone(),
            &index,
            &WorkspaceRunCommand::manual(PACKAGE),
            SystemTime::now(),
        )
        .unwrap();
    let StudioRunSubmission::Flow(WorkspaceRunSubmission::Accepted(id)) = reply else {
        panic!("flow accepted")
    };
    let before = app.workspace.engineering.clone();
    assert_eq!(
        owner
            .submit_equipment(&mut app, &command(), SystemTime::now())
            .unwrap(),
        StudioRunSubmission::Busy(StudioRunId::Flow(id.clone()))
    );
    assert_eq!(app.workspace.engineering, before);
    owner.cancel(&mut app, &StudioRunId::Flow(id));
    drain(&mut owner, &mut app);
    assert!(app.workspace.snapshot_history.is_empty());
    let device = equipment(&mut owner, &mut app);
    let flow_qualification = app.flow_run_qualification();
    assert_eq!(
        owner
            .submit_flow(
                &mut app,
                root,
                &index,
                &WorkspaceRunCommand::manual(PACKAGE),
                SystemTime::now()
            )
            .unwrap(),
        StudioRunSubmission::Busy(device)
    );
    assert_eq!(app.flow_run_qualification(), flow_qualification);
    drain(&mut owner, &mut app);
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    for _ in 0..5 {
        assert!(!owner.poll(Some(&mut app), SystemTime::now()));
    }
    assert_eq!(owner.history.len(), 2);
    assert!(owner.try_close().is_ok());
}

#[test]
fn device_ready_handshake_duplicate_identity_and_cancel_before_start_preserve_mode() {
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut owner = StudioRunOwner::default();
    let id = equipment(&mut owner, &mut app);
    assert_eq!(
        owner.activity().unwrap().phase,
        WorkspaceRunPhase::Preparing
    );
    assert_eq!(
        owner
            .submit_equipment(&mut app, &command(), SystemTime::now())
            .unwrap(),
        StudioRunSubmission::AlreadyRunning(id.clone())
    );
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    owner.cancel(&mut app, &id);
    assert!(!owner.is_idle());
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Active);
    assert_eq!(
        owner.activity().unwrap().phase,
        WorkspaceRunPhase::CancelRequested
    );
    drain(&mut owner, &mut app);
    assert_eq!(
        owner.history.back().unwrap().outcome,
        StudioRunOutcome::Cancelled
    );
    assert!(!owner.history.back().unwrap().applied);
    assert!(
        !app.workspace
            .engineering
            .case_calculation_pending(&member())
    );
}

#[test]
fn device_background_matches_inline_and_old_cancel_cannot_cancel_next_run() {
    let mut app = app();
    let mut expected = app.workspace.engineering.clone();
    expected.calculate_rating(&member()).unwrap();
    let mut owner = StudioRunOwner::default();
    let old = equipment(&mut owner, &mut app);
    drain(&mut owner, &mut app);
    assert_eq!(
        app.workspace.engineering.current_rating(&member()),
        expected.current_rating(&member())
    );
    assert_eq!(
        owner.history.back().unwrap().outcome,
        StudioRunOutcome::Succeeded
    );
    assert!(owner.history.back().unwrap().applied);
    let new = equipment(&mut owner, &mut app);
    assert_ne!(old, new);
    owner.cancel(&mut app, &old);
    assert_eq!(
        owner.activity().unwrap().phase,
        WorkspaceRunPhase::Preparing
    );
    drain(&mut owner, &mut app);
    assert_eq!(owner.history.len(), 2);
    assert!(owner.history.back().unwrap().applied);
}

#[test]
fn rating_rerun_cancel_keeps_old_result_stale_and_releases_no_partial_result() {
    let mut app = app();
    app.workspace
        .engineering
        .calculate_rating(&member())
        .unwrap();
    let old = app
        .workspace
        .engineering
        .case_calculation_id(&member())
        .cloned();
    let mut owner = StudioRunOwner::default();
    let id = equipment(&mut owner, &mut app);
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    owner.cancel(&mut app, &id);
    drain(&mut owner, &mut app);
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(
        app.workspace.engineering.case_calculation_id(&member()),
        old.as_ref()
    );
}

#[test]
fn reopening_same_ids_discards_old_device_and_retains_slot_until_joined() {
    let mut first = app();
    let mut second = app();
    let mut owner = StudioRunOwner::default();
    let id = equipment(&mut owner, &mut first);
    owner.detach(&mut first);
    let new_state = second.clone();
    assert_eq!(
        owner
            .submit_equipment(&mut second, &command(), SystemTime::now())
            .unwrap(),
        StudioRunSubmission::Busy(id)
    );
    assert_eq!(second, new_state);
    assert!(!owner.equipment_target_busy(&second, &member().task_id, None));
    drain(&mut owner, &mut second);
    assert_eq!(second, new_state);
    assert!(!owner.history.back().unwrap().applied);
    equipment(&mut owner, &mut second);
    drain(&mut owner, &mut second);
    assert!(owner.history.back().unwrap().applied);
}

#[test]
fn detached_poll_and_exit_return_drain_without_reviving_cancelled_work() {
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    equipment(&mut owner, &mut app);
    owner.begin_shutdown(&mut app);
    assert_eq!(
        owner
            .submit_equipment(&mut app, &command(), SystemTime::now())
            .unwrap(),
        StudioRunSubmission::Closing
    );
    owner.resume_submissions();
    assert_eq!(
        owner.activity().unwrap().phase,
        WorkspaceRunPhase::CancelRequested
    );
    until(|| {
        owner.poll(None, SystemTime::now());
        owner.is_idle()
    });
    assert!(!owner.history.back().unwrap().applied);
    assert!(!owner.should_attempt_automatic(&app));
}

#[test]
fn automatic_latest_input_eligible_once_but_clear_results_and_detach_do_not_restart() {
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    app.set_simulation_mode(SimulationMode::Active);
    assert!(owner.should_attempt_automatic(&app));
    owner.detach(&mut app);
    assert!(!owner.should_attempt_automatic(&app));
    app.workspace.clear_results();
    assert!(!owner.should_attempt_automatic(&app));
    let mut flow = app.workspace.document.flowsheet.clone();
    flow.units.get_mut(&"heater-1".into()).unwrap().name = "new".into();
    app.commit_document_change(
        rf_ui::DocumentCommand::RenameUnit {
            unit_id: "heater-1".into(),
            new_name: "new".into(),
        },
        flow,
        SystemTime::now(),
    );
    assert!(owner.should_attempt_automatic(&app));
    let (root, index) = cache();
    owner
        .submit_flow(
            &mut app,
            root,
            &index,
            &WorkspaceRunCommand::automatic_preferred(),
            SystemTime::now(),
        )
        .unwrap();
    drain(&mut owner, &mut app);
    assert!(!owner.should_attempt_automatic(&app));
    assert_eq!(
        owner.flow.history().back().unwrap().disposition,
        FlowRunDisposition::Applied
    );
}

#[test]
fn bounded_history_under_repeated_real_calculations() {
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    for _ in 0..32 {
        equipment(&mut owner, &mut app);
        drain(&mut owner, &mut app);
    }
    assert_eq!(owner.history.len(), 8);
    assert_eq!(owner.history.back().unwrap().activity.number, 32);
    assert!(owner.try_close().is_ok());
}

fn gated_device(
    owner: &mut StudioRunOwner,
    app: &mut AppState,
    execute: impl FnOnce(
        rf_ui::engineering::CalculationRequest,
    ) -> rf_ui::engineering::CompletedCalculation
    + Send
    + 'static,
) -> StudioRunId {
    let request = app
        .workspace
        .equipment_editor
        .prepare_calculation(&mut app.workspace.engineering, &command())
        .unwrap();
    let id = StudioRunId::Equipment(request.qualification().id().clone());
    owner.equipment = Some(EquipmentRun::spawn_with(request, execute).unwrap());
    owner.set_activity(
        id.clone(),
        "rating-1 / rating-case-1".into(),
        SystemTime::now(),
    );
    id
}

#[test]
fn cancellation_during_indivisible_device_call_keeps_slot_until_exit_and_discards_actual_success() {
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    let (entered, entry) = std::sync::mpsc::sync_channel(1);
    let (release, released) = std::sync::mpsc::sync_channel(1);
    let owner_thread = thread::current().id();
    let id = gated_device(&mut owner, &mut app, move |request| {
        assert_ne!(thread::current().id(), owner_thread);
        let completed = request.execute();
        entered.send(()).unwrap();
        released.recv_timeout(Duration::from_secs(10)).unwrap();
        completed
    });
    assert!(!owner.poll(Some(&mut app), SystemTime::now()));
    entry.recv_timeout(Duration::from_secs(10)).unwrap();
    owner.cancel(&mut app, &id);
    for _ in 0..20 {
        assert!(!owner.poll(Some(&mut app), SystemTime::now()));
        assert!(!owner.is_idle());
    }
    assert!(matches!(
        owner
            .submit_equipment(&mut app, &command(), SystemTime::now())
            .unwrap(),
        StudioRunSubmission::Busy(_)
    ));
    release.send(()).unwrap();
    drain(&mut owner, &mut app);
    let summary = owner.history.back().unwrap();
    assert_eq!(summary.outcome, StudioRunOutcome::Succeeded);
    assert!(!summary.applied);
    assert!(
        app.workspace
            .engineering
            .current_rating(&member())
            .is_none()
    );
}

#[test]
fn current_device_panic_is_reported_but_detached_panic_does_not_pollute_new_document() {
    for detached in [false, true] {
        let mut first = app();
        let mut owner = StudioRunOwner::default();
        let (entered, entry) = std::sync::mpsc::sync_channel(1);
        let (release, released) = std::sync::mpsc::sync_channel(1);
        gated_device(&mut owner, &mut first, move |_| {
            entered.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            panic!("injected numeric panic")
        });
        owner.poll(Some(&mut first), SystemTime::now());
        entry.recv_timeout(Duration::from_secs(10)).unwrap();
        let mut second = app();
        let before = second.clone();
        if detached {
            owner.detach(&mut first);
        }
        release.send(()).unwrap();
        drain(&mut owner, if detached { &mut second } else { &mut first });
        assert_eq!(
            owner.history.back().unwrap().outcome,
            StudioRunOutcome::InfrastructureFailed
        );
        if detached {
            assert_eq!(second, before);
        } else {
            assert!(
                first
                    .log_feed
                    .entries
                    .iter()
                    .any(|entry| entry.message.contains("worker failed"))
            );
            assert!(
                !first
                    .workspace
                    .engineering
                    .case_calculation_pending(&member())
            );
        }
    }
}

#[test]
fn running_rating_accepts_unrelated_case_edit_but_rejects_shared_geometry_edit() {
    use rf_model::engineering::InputBinding;
    use rf_model::equipment::{SeparatorGeometryInput, SeparatorRatingInput};
    use rf_ui::engineering::EngineeringCommand;
    for shared_geometry in [false, true] {
        let mut app = app();
        let session = &mut app.workspace.engineering;
        session
            .apply(vec![EngineeringCommand::UpgradeRatingMethod(
                member().task_id,
            )])
            .unwrap();
        let mut other = session
            .inputs()
            .rating_task(&member().task_id)
            .unwrap()
            .cases[0]
            .clone();
        other.id = "other".into();
        session
            .apply(vec![EngineeringCommand::AddRatingCase {
                task: member().task_id,
                case: Box::new(other),
            }])
            .unwrap();
        let mut owner = StudioRunOwner::default();
        let (entered, entry) = std::sync::mpsc::sync_channel(1);
        let (release, released) = std::sync::mpsc::sync_channel(1);
        gated_device(&mut owner, &mut app, move |request| {
            entered.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(10)).unwrap();
            request.execute()
        });
        owner.poll(Some(&mut app), SystemTime::now());
        entry.recv_timeout(Duration::from_secs(10)).unwrap();
        let task = app
            .workspace
            .engineering
            .inputs()
            .rating_task(&member().task_id)
            .unwrap()
            .clone();
        let change = if shared_geometry {
            let mut geometry = task.geometry;
            let (InputBinding::Direct { value, .. } | InputBinding::Override { value, .. }) =
                geometry
                    .inputs
                    .get_mut(&SeparatorGeometryInput::InternalDiameter)
                    .unwrap()
            else {
                panic!("direct")
            };
            value.value_si *= 1.1;
            EngineeringCommand::UpdateRatingGeometry {
                task: member().task_id,
                geometry: Box::new(geometry),
            }
        } else {
            let mut case = task.cases[1].clone();
            let InputBinding::Direct { value, .. } = case
                .inputs
                .get_mut(&SeparatorRatingInput::GasVolumeFlow)
                .unwrap()
            else {
                panic!("direct")
            };
            value.value_si *= 1.1;
            EngineeringCommand::UpdateRatingCase {
                task: member().task_id,
                case: Box::new(case),
            }
        };
        app.workspace.engineering.apply(vec![change]).unwrap();
        owner.poll(Some(&mut app), SystemTime::now());
        assert_eq!(
            owner.activity().unwrap().phase == WorkspaceRunPhase::CancelRequested,
            shared_geometry
        );
        release.send(()).unwrap();
        drain(&mut owner, &mut app);
        assert_eq!(owner.history.back().unwrap().applied, !shared_geometry);
    }
}
