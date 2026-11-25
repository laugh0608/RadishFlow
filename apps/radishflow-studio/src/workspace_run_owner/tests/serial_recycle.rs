use super::*;
use crate::{flow_plan_actions, qs1_test_support::select, qs4_test_support::serial_case};
use rf_solver::mixed::{MixedFailureKind, MixedRecycleConfiguration, SERIAL_RECYCLE_METHOD};
use rf_ui::{FlowResultRecord, flow_plan::FlowPlanConfiguration, result_query::FlowResultView};

#[test]
fn reopened_m3_save_discards_pre_save_receipts_and_publishes_only_from_the_new_run() {
    use crate::{
        StudioProjectOpen, prepare_mixed_serial_recycle_save, read_studio_project_with_support,
    };
    use rf_store::project_v4::FlowPlanSupport;

    let root = PropertyPackageCacheRoot::temporary("m3-v4-worker").unwrap();
    for (budget, expected) in [
        (500, WorkspaceExecutionOutcome::Succeeded),
        (1, WorkspaceExecutionOutcome::Failed),
    ] {
        let mut original = serial_case(budget);
        let mut owner = WorkspaceRunOwner::new();
        let (old, release) = finish_without_accepting(&mut owner, &mut original);
        let path = root.path().join(format!("serial-{budget}.rfproj.json"));
        prepare_mixed_serial_recycle_save(&original, &path)
            .unwrap()
            .commit(&mut original, true)
            .unwrap();
        owner.detach(&mut original);
        let StudioProjectOpen::Editable(mut reopened) =
            read_studio_project_with_support(&path, FlowPlanSupport::MixedSerialRecycle).unwrap()
        else {
            panic!("saved project")
        };
        assert!(matches!(
            reopened.workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));
        release.send(()).unwrap();
        let summary = drain(&mut owner, &mut reopened);
        assert_eq!(summary.run_id, old);
        assert_eq!(summary.outcome, expected);
        assert_eq!(summary.trace, WorkspaceTraceRecord::NotCollected);
        assert_eq!(
            summary.disposition,
            FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession)
        );
        assert!(matches!(
            reopened.workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));

        if budget == 500 {
            let (cancelled, release) = finish_without_accepting(&mut owner, &mut reopened);
            assert_eq!(
                owner.cancel(&mut reopened, &cancelled),
                WorkspaceRunCancelReply::Requested
            );
            release.send(()).unwrap();
            let summary = drain(&mut owner, &mut reopened);
            assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
            assert_eq!(
                summary.disposition,
                FlowRunDisposition::Discarded(FlowRunDiscardReason::AcceptanceRevoked)
            );
            assert!(matches!(
                reopened.workspace.flow_result(),
                FlowResultView::Missing { .. }
            ));
        }

        let next = spawn(&mut owner, &mut reopened, load(), &ThreadSpawner);
        assert_ne!(next, old);
        let summary = drain(&mut owner, &mut reopened);
        assert_eq!(summary.outcome, expected);
        assert_eq!(summary.disposition, FlowRunDisposition::Applied);
        if budget == 500 {
            let FlowResultView::Current {
                snapshot: FlowResultRecord::Mixed(snapshot),
                ..
            } = reopened.workspace.flow_result()
            else {
                panic!("qualified M3 result")
            };
            assert_eq!(snapshot.result.serial_recycles.len(), 2);
            assert_eq!(snapshot.result.streams.len(), 13);
        } else {
            assert!(matches!(
                reopened.workspace.flow_result(),
                FlowResultView::Failed { .. }
            ));
            assert!(reopened.workspace.snapshot_history.is_empty());
            let failure = reopened.workspace.mixed_failure_evidence().unwrap();
            assert_eq!(failure.kind, MixedFailureKind::IterationLimit);
            assert!(!failure.costs.calls_by_unit.contains_key(&"mixer-b".into()));
        }
    }
}

fn finish_without_accepting(
    owner: &mut WorkspaceRunOwner,
    app: &mut AppState,
) -> (FlowRunId, mpsc::SyncSender<()>) {
    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    let spawner = GatedSpawner {
        entered: entered_tx,
        release: std::sync::Mutex::new(Some(released)),
    };
    let id = spawn(owner, app, load(), &spawner);
    wait_ready(owner);
    until(|| {
        assert!(owner.poll(app, now()).is_none());
        owner.state().unwrap().started_at.is_some()
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    // The real terminal payload is ready while the worker still owns the slot.
    (id, release)
}

#[test]
fn serial_recycle_upstream_failure_applies_diagnostics_without_current_or_b_calls() {
    let mut app = serial_case(1);
    let mut owner = WorkspaceRunOwner::new();
    spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Failed);
    assert_eq!(summary.disposition, FlowRunDisposition::Applied);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Failed { .. }
    ));
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    let failure = app.workspace.mixed_failure_evidence().unwrap();
    assert_eq!(failure.method, SERIAL_RECYCLE_METHOD);
    assert_eq!(failure.kind, MixedFailureKind::IterationLimit);
    assert_eq!(failure.serial_recycles.len(), 1);
    for id in [
        "feed-b",
        "mixer-makeup-b",
        "mixer-b",
        "splitter-primary-b",
        "splitter-recycle-b",
    ] {
        assert!(
            !failure.costs.calls_by_unit.contains_key(&id.into()),
            "{id}"
        );
    }
}

#[test]
fn serial_recycle_cancel_rejects_completed_success_and_failure_before_acceptance() {
    for (budget, expected) in [
        (500, WorkspaceExecutionOutcome::Succeeded),
        (1, WorkspaceExecutionOutcome::Failed),
    ] {
        let mut app = serial_case(budget);
        let mut owner = WorkspaceRunOwner::new();
        if budget == 500 {
            spawn(&mut owner, &mut app, load(), &ThreadSpawner);
            assert_eq!(
                drain(&mut owner, &mut app).disposition,
                FlowRunDisposition::Applied
            );
        }
        let before = app.workspace.snapshot_history.clone();
        let (id, release) = finish_without_accepting(&mut owner, &mut app);
        assert_eq!(
            owner.cancel(&mut app, &id),
            WorkspaceRunCancelReply::Requested
        );
        assert!(owner.poll(&mut app, now()).is_none());
        assert!(!owner.is_idle());
        release.send(()).unwrap();
        let summary = drain(&mut owner, &mut app);
        assert_eq!(summary.run_id, id);
        assert_eq!(summary.outcome, expected);
        assert_eq!(
            summary.cancellation,
            Some(WorkspaceRunCancelReason::UserRequested)
        );
        assert_eq!(
            summary.disposition,
            FlowRunDisposition::Discarded(FlowRunDiscardReason::AcceptanceRevoked)
        );
        assert_eq!(app.workspace.snapshot_history, before);
        assert!(app.workspace.mixed_failure_evidence().is_none());
        if budget == 1 {
            assert!(matches!(
                app.workspace.flow_result(),
                FlowResultView::Missing { .. }
            ));
            select(
                &mut app,
                serial_case(500).workspace.flow_plan.current().clone(),
            );
        }
        let next = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
        assert_ne!(next, id);
        let summary = drain(&mut owner, &mut app);
        assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
        assert_eq!(summary.disposition, FlowRunDisposition::Applied);
        let FlowResultView::Current {
            snapshot: FlowResultRecord::Mixed(snapshot),
            ..
        } = app.workspace.flow_result()
        else {
            panic!("qualified serial rerun must publish Current");
        };
        assert_eq!(snapshot.result.streams.len(), 13);
        assert_eq!(snapshot.result.serial_recycles.len(), 2);
        let plan = snapshot
            .origin
            .as_ref()
            .unwrap()
            .execution()
            .unwrap()
            .plan
            .as_ref()
            .unwrap();
        assert_eq!(plan.method, SERIAL_RECYCLE_METHOD);
        assert_eq!(
            plan.configuration_sha256,
            flow_plan_actions::runtime_configuration_sha256(app.workspace.flow_plan.current())
        );
    }
}

#[test]
fn serial_recycle_plan_and_input_aba_and_reopen_reject_completed_payloads() {
    let mut owner = WorkspaceRunOwner::new();
    let mut app = serial_case(500);
    let (id, release) = finish_without_accepting(&mut owner, &mut app);
    let selected = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    select(&mut app, selected);
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.run_id, id);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::PlanChanged)
    );

    let (id, release) = finish_without_accepting(&mut owner, &mut app);
    let unit_id = "mixer-b".into();
    let original = app.workspace.document.flowsheet.units[&unit_id]
        .name
        .clone();
    for name in ["B changed".to_owned(), original] {
        let mut updated = app.workspace.document.flowsheet.clone();
        updated.units.get_mut(&unit_id).unwrap().name = name.clone();
        app.commit_document_change(
            rf_ui::DocumentCommand::RenameUnit {
                unit_id: unit_id.clone(),
                new_name: name,
            },
            updated,
            now(),
        )
        .unwrap();
    }
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.run_id, id);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::InputChanged)
    );

    let (old, release) = finish_without_accepting(&mut owner, &mut app);
    owner.detach(&mut app);
    let mut reopened = serial_case(500);
    release.send(()).unwrap();
    let summary = drain(&mut owner, &mut reopened);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession)
    );
    assert!(matches!(
        reopened.workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    assert_ne!(
        spawn(&mut owner, &mut reopened, load(), &ThreadSpawner),
        old
    );
    assert_eq!(
        drain(&mut owner, &mut reopened).disposition,
        FlowRunDisposition::Applied
    );
}

#[test]
fn serial_configuration_has_its_own_stable_runtime_digest_and_requires_v4_opt_in() {
    let mut plan = serial_case(500).workspace.flow_plan.current().clone();
    let digest = flow_plan_actions::runtime_configuration_sha256(&plan);
    let FlowPlanConfiguration::Mixed(options) = &mut plan else {
        panic!("serial fixture")
    };
    options.blocks.reverse();
    let Some(MixedRecycleConfiguration::Serial(regions)) = options.recycle.as_deref_mut() else {
        panic!("serial fixture")
    };
    regions.reverse();
    assert_eq!(
        flow_plan_actions::runtime_configuration_sha256(&plan),
        digest
    );
    let FlowPlanConfiguration::Mixed(options) = &mut plan else {
        panic!("serial fixture")
    };
    let Some(MixedRecycleConfiguration::Serial(regions)) = options.recycle.as_deref_mut() else {
        panic!("serial fixture")
    };
    regions[0].tear.initial_component_flows_mol_s =
        Some([("methane".into(), 1.0), ("ethane".into(), 2.0)].into());
    assert_ne!(
        flow_plan_actions::runtime_configuration_sha256(&plan),
        digest
    );
    let different_budget = serial_case(499).workspace.flow_plan.current().clone();
    assert_ne!(
        flow_plan_actions::runtime_configuration_sha256(&different_budget),
        digest
    );
    let error = flow_plan_actions::stored_flow_plan_with_support(
        &plan,
        rf_store::project_v4::FlowPlanSupport::SinglePhaseRecycle,
    )
    .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("workspace.plan.in_memory_only")
    );
}
