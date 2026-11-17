use super::*;
use crate::qs4_test_support::recycle_case;
use rf_ui::result_query::FlowResultView;

#[test]
fn mixed_recycle_cancel_rejects_finished_success_and_failure_and_keeps_slot_until_join() {
    for budget in [500, 1] {
        let mut app = recycle_case(budget);
        let mut owner = WorkspaceRunOwner::new();
        if budget == 500 {
            let id = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
            assert_eq!(
                drain(&mut owner, &mut app).disposition,
                FlowRunDisposition::Applied
            );
            assert!(matches!(
                app.workspace.flow_result(),
                FlowResultView::Current { .. }
            ));
            assert_eq!(
                owner.cancel(&mut app, &id),
                WorkspaceRunCancelReply::AlreadyFinished
            );
        }
        let before = app.workspace.snapshot_history.clone();
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        let spawner = GatedSpawner {
            entered: entered_tx,
            release: std::sync::Mutex::new(Some(released)),
        };
        let id = spawn(&mut owner, &mut app, load(), &spawner);
        wait_ready(&owner);
        until(|| {
            assert!(owner.poll(&mut app, now()).is_none());
            owner.state().unwrap().started_at.is_some()
        });
        entered.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(owner.poll(&mut app, now()).is_none());
        owner.cancel(&mut app, &id);
        assert!(!owner.is_idle());
        release.send(()).unwrap();
        let summary = drain(&mut owner, &mut app);
        assert_eq!(
            summary.outcome,
            if budget == 500 {
                WorkspaceExecutionOutcome::Succeeded
            } else {
                WorkspaceExecutionOutcome::Failed
            }
        );
        assert!(matches!(
            summary.disposition,
            FlowRunDisposition::Discarded(_)
        ));
        assert_eq!(
            summary.cancellation,
            Some(WorkspaceRunCancelReason::UserRequested)
        );
        assert_eq!(app.workspace.snapshot_history, before);
        assert!(app.workspace.mixed_failure_evidence().is_none());
        if budget == 500 {
            assert!(matches!(
                app.workspace.flow_result(),
                FlowResultView::Current { .. }
            ));
        } else {
            assert!(matches!(
                app.workspace.flow_result(),
                FlowResultView::Missing { .. }
            ));
        }
        // Even a cancelled attempt that already produced a payload releases the
        // shared slot only after join and admits a new run with a fresh receipt.
        let next = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
        assert_ne!(id, next);
        assert_eq!(
            drain(&mut owner, &mut app).disposition,
            FlowRunDisposition::Applied
        );
    }
}
