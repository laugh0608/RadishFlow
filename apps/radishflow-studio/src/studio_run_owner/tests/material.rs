use super::*;
use crate::{material_balance_actions::tests::app as material_app, run_actions::StudioRunReply};
use rf_ui::material_balance::MaterialBalanceResultView;
fn submit(owner: &mut StudioRunOwner, app: &mut AppState) -> StudioRunId {
    match owner
        .submit_material_balance(app, SystemTime::now())
        .unwrap()
    {
        StudioRunReply::Accepted(id) => id,
        other => panic!("{other:?}"),
    }
}
#[test]
fn actual_background_material_matches_inline_and_participates_in_the_single_worker_quota() {
    let mut owner = StudioRunOwner::default();
    let mut app = material_app();
    let id = submit(&mut owner, &mut app);
    assert!(
        matches!(owner.submit_material_balance(&mut app,SystemTime::now()).unwrap(),StudioRunReply::AlreadyRunning(same) if same==id)
    );
    // A different domain cannot acquire the numeric slot before this worker exits.
    let (root, index) = cache();
    let flow_command = WorkspaceRunCommand::new(
        crate::WorkspaceSolveTrigger::Manual,
        crate::WorkspaceRunPackageSelection::Preferred,
    );
    assert!(
        matches!(owner.submit_flow(&mut app,root,&index,&flow_command,SystemTime::now()).unwrap(),StudioRunSubmission::Busy(same) if same==id)
    );
    drain(&mut owner, &mut app);
    assert!(owner.history().back().unwrap().applied);
    let MaterialBalanceResultView::Current { solution, .. } =
        app.workspace.material_balance_result()
    else {
        panic!("current")
    };
    assert!((solution.values.mole_fractions[&"methane".into()] - 0.44).abs() < 1e-12);
    assert!(matches!(
        owner.cancel(&mut app, &id),
        crate::WorkspaceRunCancelReply::AlreadyFinished
    ));
}
#[test]
fn cancel_before_handshake_prevents_start_and_late_cancel_cannot_stop_a_new_task() {
    let mut owner = StudioRunOwner::default();
    let mut app = material_app();
    let first = submit(&mut owner, &mut app);
    assert_eq!(
        owner.cancel(&mut app, &first),
        crate::WorkspaceRunCancelReply::Requested
    );
    assert_eq!(
        owner.cancel(&mut app, &first),
        crate::WorkspaceRunCancelReply::AlreadyRequested
    );
    drain(&mut owner, &mut app);
    let summary = owner.history().back().unwrap();
    assert_eq!(summary.outcome, StudioRunOutcome::Cancelled);
    assert!(!summary.applied);
    assert!(matches!(
        app.workspace.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
    let second = submit(&mut owner, &mut app);
    assert_ne!(first, second);
    assert_eq!(
        owner.cancel(&mut app, &first),
        crate::WorkspaceRunCancelReply::AlreadyFinished
    );
    drain(&mut owner, &mut app);
    assert!(owner.history().back().unwrap().applied);
}
#[test]
fn plan_change_back_before_ready_revokes_old_worker_and_allows_a_fresh_calculation() {
    let mut owner = StudioRunOwner::default();
    let mut app = material_app();
    submit(&mut owner, &mut app);
    let old = app.workspace.material_balance_plan().unwrap().clone();
    let mut changed = old.clone();
    changed.options.scaled_tolerance = 1e-8;
    app.workspace.set_material_balance_plan(changed).unwrap();
    app.workspace.set_material_balance_plan(old).unwrap();
    drain(&mut owner, &mut app);
    let summary = owner.history().back().unwrap();
    assert!(!summary.applied);
    assert_eq!(
        summary.cancellation,
        Some(crate::WorkspaceRunCancelReason::QualificationChanged)
    );
    submit(&mut owner, &mut app);
    drain(&mut owner, &mut app);
    assert!(owner.history().back().unwrap().applied);
}
#[test]
fn shutdown_and_detached_poll_reap_workers_without_publishing_into_replacement_documents() {
    let mut owner = StudioRunOwner::default();
    let mut old = material_app();
    submit(&mut owner, &mut old);
    owner.begin_shutdown(&mut old);
    assert!(matches!(
        owner
            .submit_material_balance(&mut old, SystemTime::now())
            .unwrap(),
        StudioRunReply::Closing
    ));
    let mut replacement = material_app();
    drain(&mut owner, &mut replacement);
    assert!(!owner.history().back().unwrap().applied);
    assert!(owner.try_close().is_ok());
    let mut owner = StudioRunOwner::default();
    submit(&mut owner, &mut old);
    owner.detach(&mut old);
    until(|| {
        owner.poll(None, SystemTime::now());
        owner.is_idle()
    });
    assert!(!owner.history().back().unwrap().applied);
}

#[test]
fn running_cancel_keeps_the_slot_until_join_and_rejects_a_completed_late_result() {
    use crate::studio_run_owner::material::MaterialRun;
    use rf_solver::component_balance::{MaterialBalanceControl, MaterialBalanceProgress};
    use std::sync::mpsc;
    let mut owner = StudioRunOwner::default();
    let mut app = material_app();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let reply = owner
        .submit_material_with(&mut app, SystemTime::now(), move |input, receipt| {
            MaterialRun::spawn_with(input, receipt, move |input, _| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                struct Continue;
                impl MaterialBalanceControl for Continue {
                    fn checkpoint(
                        &mut self,
                        _: MaterialBalanceProgress,
                    ) -> std::ops::ControlFlow<()> {
                        std::ops::ControlFlow::Continue(())
                    }
                }
                input.execute(&mut Continue)
            })
        })
        .unwrap();
    let StudioRunReply::Accepted(id) = reply else {
        panic!("accepted")
    };
    owner.poll(Some(&mut app), SystemTime::now());
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        owner.cancel(&mut app, &id),
        crate::WorkspaceRunCancelReply::Requested
    );
    owner.poll(Some(&mut app), SystemTime::now());
    assert!(!owner.is_idle());
    assert!(matches!(
        owner
            .submit_material_balance(&mut app, SystemTime::now())
            .unwrap(),
        StudioRunReply::Busy(_)
    ));
    release_tx.send(()).unwrap();
    drain(&mut owner, &mut app);
    let summary = owner.history().back().unwrap();
    assert_eq!(summary.outcome, StudioRunOutcome::Succeeded);
    assert!(!summary.applied);
    assert!(matches!(
        app.workspace.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
}

#[test]
fn thread_start_failure_and_actual_worker_panic_publish_current_infrastructure_failure() {
    use crate::studio_run_owner::material::MaterialRun;
    use rf_ui::material_balance::MaterialBalanceFailure;
    let mut owner = StudioRunOwner::default();
    let mut app = material_app();
    let reply = owner
        .submit_material_with(&mut app, SystemTime::now(), |_, receipt| {
            Err((
                Box::new(receipt),
                std::io::Error::other("injected start failure"),
            ))
        })
        .unwrap();
    assert!(matches!(
        reply,
        StudioRunReply::StartFailed {
            id: Some(StudioRunId::Material(_)),
            ..
        }
    ));
    assert!(owner.is_idle());
    assert!(matches!(
        app.workspace.material_balance_result(),
        MaterialBalanceResultView::Failed {
            failure: MaterialBalanceFailure::Infrastructure(_),
            ..
        }
    ));
    owner
        .submit_material_with(&mut app, SystemTime::now(), |input, receipt| {
            MaterialRun::spawn_with(input, receipt, |_, _| panic!("injected material panic"))
        })
        .unwrap();
    drain(&mut owner, &mut app);
    let summary = owner.history().back().unwrap();
    assert_eq!(summary.outcome, StudioRunOutcome::InfrastructureFailed);
    assert!(summary.applied);
    assert_eq!(
        summary.diagnostic_code.as_deref(),
        Some("material.worker_panicked")
    );
    assert!(matches!(
        app.workspace.material_balance_result(),
        MaterialBalanceResultView::Failed { .. }
    ));
}

#[test]
fn occupied_equipment_rejects_material_without_allocating_a_second_receipt() {
    let mut owner = StudioRunOwner::default();
    let mut app = app();
    equipment(&mut owner, &mut app);
    let plan = material_app()
        .workspace
        .material_balance_plan()
        .unwrap()
        .clone();
    app.workspace.set_material_balance_plan(plan).unwrap();
    assert!(matches!(
        owner
            .submit_material_balance(&mut app, SystemTime::now())
            .unwrap(),
        StudioRunReply::Busy(StudioRunId::Equipment(_))
    ));
    assert!(matches!(
        app.workspace.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
    drain(&mut owner, &mut app);
}
