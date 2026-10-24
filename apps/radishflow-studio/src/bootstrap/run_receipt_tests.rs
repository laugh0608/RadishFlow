use super::*;
use crate::run_actions::{StudioEquipmentDispatch, StudioRunRejection, StudioRunReply};
use crate::{StudioBackgroundRunAction, StudioRunOwner, WorkspaceRunCancelReply};
use rf_ui::engineering::{
    EquipmentCaseKey,
    editor::{EquipmentEditError, EquipmentEditorCommand},
};
use std::{cell::RefCell, rc::Rc};

fn runtime() -> StudioRuntime {
    StudioRuntime::new(&StudioBootstrapConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/rf-store/tests/fixtures/project-v3-rating.json"),
        ..Default::default()
    })
    .unwrap()
}
fn calculate() -> StudioBootstrapTrigger {
    StudioBootstrapTrigger::Equipment(EquipmentEditorCommand::CalculateRating(EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }))
}
fn dispatch(report: StudioBootstrapReport) -> StudioEquipmentDispatch {
    let StudioBootstrapDispatch::Equipment(dispatch) = report.dispatch else {
        panic!()
    };
    *dispatch
}

#[test]
fn runtime_preserves_accepted_duplicate_cancel_and_closing_receipts() {
    let mut runtime = runtime();
    let owner = Rc::new(RefCell::new(StudioRunOwner::default()));
    runtime
        .background_run_action(StudioBackgroundRunAction::Install(owner.clone()))
        .unwrap();
    let first = dispatch(runtime.dispatch_trigger(&calculate()).unwrap());
    let StudioEquipmentDispatch::Run(StudioRunReply::Accepted(id)) = first else {
        panic!("{first:?}")
    };
    assert_eq!(
        dispatch(runtime.dispatch_trigger(&calculate()).unwrap()),
        StudioEquipmentDispatch::Run(StudioRunReply::AlreadyRunning(id.clone()))
    );
    assert_eq!(
        runtime
            .background_run_action(StudioBackgroundRunAction::Cancel(id.clone()))
            .unwrap()
            .cancellation,
        Some(WorkspaceRunCancelReply::Requested)
    );
    assert_eq!(
        runtime
            .background_run_action(StudioBackgroundRunAction::Cancel(id))
            .unwrap()
            .cancellation,
        Some(WorkspaceRunCancelReply::AlreadyRequested)
    );
    runtime
        .background_run_action(StudioBackgroundRunAction::BeginShutdown)
        .unwrap();
    assert_eq!(
        dispatch(runtime.dispatch_trigger(&calculate()).unwrap()),
        StudioEquipmentDispatch::Run(StudioRunReply::Closing)
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !owner.borrow().is_idle() {
        assert!(std::time::Instant::now() < deadline);
        runtime
            .background_run_action(StudioBackgroundRunAction::Poll { attached: true })
            .unwrap();
        std::thread::yield_now();
    }
}

#[test]
fn runtime_keeps_domain_rejection_typed_and_inline_reports_terminal() {
    let mut runtime = runtime();
    let result = dispatch(
        runtime
            .dispatch_trigger(&StudioBootstrapTrigger::Equipment(
                EquipmentEditorCommand::Calculate("absent".into()),
            ))
            .unwrap(),
    );
    assert!(matches!(
        result,
        StudioEquipmentDispatch::Run(StudioRunReply::Blocked(StudioRunRejection::Equipment(
            EquipmentEditError::Domain(_)
        )))
    ));
    let result = dispatch(runtime.dispatch_trigger(&calculate()).unwrap());
    assert!(
        matches!(result, StudioEquipmentDispatch::Run(StudioRunReply::Completed(completed)) if completed.applied)
    );
}
