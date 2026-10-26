use super::*;
use crate::run_actions::*;
use rf_ui::engineering::editor::{EquipmentActionContext, EquipmentEditError};

fn request(app: &AppState) -> StudioRunRequest {
    StudioRunRequest::Equipment {
        expected: EquipmentActionContext::capture(&app.workspace.engineering),
        target: EquipmentCalculationTarget::Rating(member()),
    }
}
fn facade(owner: &SharedStudioRunOwner, root: &PropertyPackageCacheRoot) -> crate::StudioAppFacade {
    let mut facade = crate::StudioAppFacade::default();
    facade.background = Some(BackgroundRunSession {
        owner: owner.clone(),
        root: root.clone(),
    });
    facade
}

#[test]
fn inline_and_background_share_terminal_but_never_confuse_acceptance_with_completion() {
    let (root, index) = cache();
    let context = crate::StudioAppAuthCacheContext::new(root.path(), &index);
    let mut inline = app();
    let mut background = inline.clone();
    let req = request(&inline);
    let reply = crate::StudioAppFacade::default().execute_run_action(
        &mut inline,
        &context,
        StudioRunAccess::Allowed,
        req.clone(),
    );
    let StudioRunReply::Completed(completed) = reply else {
        panic!("{reply:?}")
    };
    assert_eq!(completed.outcome, StudioRunOutcome::Succeeded);
    assert!(completed.applied);
    let owner = Rc::new(RefCell::new(StudioRunOwner::default()));
    let facade = facade(&owner, &root);
    let reply = facade.execute_run_action(
        &mut background,
        &context,
        StudioRunAccess::Allowed,
        req.clone(),
    );
    let StudioRunReply::Accepted(id) = reply else {
        panic!("{reply:?}")
    };
    assert_eq!(
        facade.execute_run_action(&mut background, &context, StudioRunAccess::Allowed, req),
        StudioRunReply::AlreadyRunning(id.clone())
    );
    drain(&mut owner.borrow_mut(), &mut background);
    let terminal = owner.borrow().history().back().unwrap().completion();
    assert_eq!(terminal.id, id);
    assert_eq!(terminal.outcome, completed.outcome);
    assert_eq!(terminal.applied, completed.applied);
    assert_eq!(
        inline.workspace.engineering.current_rating(&member()),
        background.workspace.engineering.current_rating(&member())
    );
}

#[test]
fn typed_run_rechecks_access_and_context_without_claiming_a_slot() {
    let (root, index) = cache();
    let context = crate::StudioAppAuthCacheContext::new(root.path(), &index);
    let mut app = app();
    let req = request(&app);
    let owner = Rc::new(RefCell::new(StudioRunOwner::default()));
    let facade = facade(&owner, &root);
    assert_eq!(
        facade.execute_run_action(&mut app, &context, StudioRunAccess::ReadOnly, req.clone()),
        StudioRunReply::Blocked(StudioRunRejection::ReadOnly)
    );
    app.workspace.engineering = EngineeringSession::from_inputs(
        "project".into(),
        app.workspace.engineering.inputs().clone(),
    )
    .unwrap();
    assert!(matches!(
        facade.execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req),
        StudioRunReply::Blocked(StudioRunRejection::EquipmentContext(_))
    ));
    assert!(owner.borrow().is_idle());
    assert!(owner.borrow().history().is_empty());
}

#[test]
fn exact_cancel_reports_requested_repeat_finished_and_unknown_without_cancelling_new_run() {
    use crate::WorkspaceRunCancelReply as Reply;
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    let id = equipment(&mut owner, &mut app);
    assert_eq!(owner.cancel(&mut app, &id), Reply::Requested);
    assert_eq!(owner.cancel(&mut app, &id), Reply::AlreadyRequested);
    drain(&mut owner, &mut app);
    let next = equipment(&mut owner, &mut app);
    assert_eq!(owner.cancel(&mut app, &id), Reply::AlreadyFinished);
    assert_eq!(owner.activity().unwrap().id, next);
    assert_eq!(owner.activity().unwrap().cancel_requested_at, None);
    let unknown = StudioRunId::Equipment(
        app.workspace
            .engineering
            .begin_rating(&member())
            .unwrap()
            .qualification()
            .id()
            .clone(),
    );
    assert_eq!(owner.cancel(&mut app, &unknown), Reply::UnknownRun);
    drain(&mut owner, &mut app);
}

#[test]
fn startup_failure_returns_reserved_id_and_revokes_pending_publication() {
    let mut app = app();
    app.workspace
        .engineering
        .calculate_rating(&member())
        .unwrap();
    let previous = app
        .workspace
        .engineering
        .case_calculation_id(&member())
        .unwrap()
        .clone();
    let mut owner = StudioRunOwner::default();
    let reply = owner.submit_equipment_with(&mut app, &command(), SystemTime::now(), |_| {
        Err(std::io::Error::other("test thread refusal"))
    });
    let Err(EquipmentRunError::StartFailed {
        id: StudioRunId::Equipment(id),
        error,
    }) = reply
    else {
        panic!("{reply:?}")
    };
    assert_ne!(id, previous);
    assert!(error.message().contains("test thread refusal"));
    assert!(owner.is_idle());
    assert!(
        !app.workspace
            .engineering
            .case_calculation_pending(&member())
    );
    assert_ne!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
}

#[test]
fn unsupported_and_missing_target_remain_typed() {
    let mut app = app();
    let mut owner = StudioRunOwner::default();
    assert_eq!(
        owner.submit_equipment_typed(
            &mut app,
            &EquipmentEditorCommand::CreateTask,
            SystemTime::now()
        ),
        Err(EquipmentRunError::Rejected(
            EquipmentEditError::UnsupportedEditorAction
        ))
    );
    assert!(matches!(
        owner.submit_equipment_typed(
            &mut app,
            &EquipmentEditorCommand::Calculate("missing".into()),
            SystemTime::now()
        ),
        Err(EquipmentRunError::Rejected(EquipmentEditError::Domain(_)))
    ));
    assert!(owner.is_idle());
}

#[test]
fn flow_inline_returns_terminal_and_background_returns_same_domain_identity() {
    let (root, index) = cache();
    let context = crate::StudioAppAuthCacheContext::new(root.path(), &index);
    let mut app = app();
    let req = StudioRunRequest::Flow {
        expected: crate::flow_actions::FlowEditContext::capture(&app.workspace),
        command: WorkspaceRunCommand::manual(PACKAGE),
    };
    let reply = crate::StudioAppFacade::default().execute_run_action(
        &mut app,
        &context,
        StudioRunAccess::Allowed,
        req.clone(),
    );
    let StudioRunReply::Completed(terminal) = reply else {
        panic!("{reply:?}")
    };
    assert!(matches!(terminal.id, StudioRunId::Flow(_)));
    assert_eq!(terminal.outcome, StudioRunOutcome::Succeeded);
    assert!(terminal.applied);
    let owner = Rc::new(RefCell::new(StudioRunOwner::default()));
    let reply =
        facade(&owner, &root).execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req);
    let StudioRunReply::Accepted(id) = reply else {
        panic!("{reply:?}")
    };
    drain(&mut owner.borrow_mut(), &mut app);
    assert_eq!(owner.borrow().history().back().unwrap().completion().id, id);
}

#[test]
fn cache_start_failure_cannot_borrow_a_previous_success_identity() {
    let (root, index) = cache();
    let context = crate::StudioAppAuthCacheContext::new(root.path(), &index);
    let mut app = app();
    let req = StudioRunRequest::Flow {
        expected: crate::flow_actions::FlowEditContext::capture(&app.workspace),
        command: WorkspaceRunCommand::manual(PACKAGE),
    };
    let facade = crate::StudioAppFacade::default();
    assert!(matches!(
        facade.execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req.clone()),
        StudioRunReply::Completed(_)
    ));
    let empty = PropertyPackageCacheRoot::temporary("af4b-empty-cache").unwrap();
    let context = crate::StudioAppAuthCacheContext::new(empty.path(), &index);
    assert!(matches!(
        facade.execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req),
        StudioRunReply::StartFailed { id: None, .. }
    ));
}

#[test]
fn cross_domain_busy_retains_the_occupying_run_identity() {
    let (root, index) = cache();
    let context = crate::StudioAppAuthCacheContext::new(root.path(), &index);
    let mut app = app();
    let owner = Rc::new(RefCell::new(StudioRunOwner::default()));
    let facade = facade(&owner, &root);
    let req = request(&app);
    let StudioRunReply::Accepted(id) =
        facade.execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req)
    else {
        panic!()
    };
    let req = StudioRunRequest::Flow {
        expected: crate::flow_actions::FlowEditContext::capture(&app.workspace),
        command: WorkspaceRunCommand::manual(PACKAGE),
    };
    assert_eq!(
        facade.execute_run_action(&mut app, &context, StudioRunAccess::Allowed, req),
        StudioRunReply::Busy(id)
    );
    drain(&mut owner.borrow_mut(), &mut app);
}

#[test]
fn independent_equipment_entry_needs_no_cache_and_preserves_context_and_run_identity() {
    let mut app = app();
    let facade = crate::StudioAppFacade::default();
    let expected = EquipmentActionContext::capture(&app.workspace.engineering);
    let target = EquipmentCalculationTarget::Rating(member());
    assert_eq!(
        facade.execute_equipment_run_action(
            &mut app,
            StudioRunAccess::ReadOnly,
            expected.clone(),
            target.clone()
        ),
        StudioRunReply::Blocked(StudioRunRejection::ReadOnly)
    );
    let StudioRunReply::Completed(first) = facade.execute_equipment_run_action(
        &mut app,
        StudioRunAccess::Allowed,
        expected.clone(),
        target.clone(),
    ) else {
        panic!()
    };
    let value = app
        .workspace
        .engineering
        .current_rating(&member())
        .unwrap()
        .clone();
    let StudioRunReply::Completed(second) = facade.execute_equipment_run_action(
        &mut app,
        StudioRunAccess::Allowed,
        expected.clone(),
        target.clone(),
    ) else {
        panic!()
    };
    assert_ne!(first.id, second.id);
    assert!(second.applied);
    assert_eq!(
        app.workspace.engineering.current_rating(&member()),
        Some(&value)
    );
    app.workspace.engineering = EngineeringSession::from_inputs(
        "project".into(),
        app.workspace.engineering.inputs().clone(),
    )
    .unwrap();
    assert!(matches!(
        facade.execute_equipment_run_action(&mut app, StudioRunAccess::Allowed, expected, target),
        StudioRunReply::Blocked(StudioRunRejection::EquipmentContext(_))
    ));
}
