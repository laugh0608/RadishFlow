use super::*;

fn apply(
    editor: &mut EquipmentEditorState,
    session: &mut EngineeringSession,
    action: EquipmentAction,
) -> EquipmentActionReceipt {
    editor
        .execute_action(
            session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &EquipmentActionContext::capture(session),
            action,
        )
        .unwrap()
}
fn setup() -> (EquipmentEditorState, EngineeringSession, EquipmentTaskId) {
    let mut editor = EquipmentEditorState::default();
    let mut session = EngineeringSession::new("actions".into());
    let receipt = apply(&mut editor, &mut session, EquipmentAction::CreateSizingTask);
    assert_eq!(receipt.outcome, EquipmentActionOutcome::Committed);
    let Some(EquipmentEditorTarget::Task(id)) = receipt.target else {
        panic!()
    };
    (editor, session, id)
}

#[test]
fn context_rejects_reopen_fork_and_read_only_without_side_effects() {
    let (mut editor, mut session, _) = setup();
    let context = EquipmentActionContext::capture(&session);
    let before = (editor.clone(), session.clone());
    assert_eq!(
        editor.execute_action(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::ReadOnly,
            &context,
            EquipmentAction::CreateSource
        ),
        Err(EquipmentActionError::ReadOnly)
    );
    assert_eq!((editor.clone(), session.clone()), before);
    let mut reopened = EngineeringSession::new("actions".into());
    assert_eq!(
        editor.execute_action(
            &mut reopened,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::CreateSource
        ),
        Err(EquipmentActionError::DifferentSession)
    );
    apply(&mut editor, &mut session, EquipmentAction::CreateSource);
    assert_eq!(
        editor.execute_action(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::CreateSource
        ),
        Err(EquipmentActionError::StaleInput)
    );
}

#[test]
fn program_update_and_gui_draft_publish_identical_formal_records() {
    let (mut editor, mut session, id) = setup();
    let mut other_editor = editor.clone();
    let mut other_session = session.clone();
    let mut record = session.inputs().task(&id).unwrap().clone();
    record.name = "Separator A".into();
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::UpdateSizingTask(Box::new(record)),
    );
    assert_eq!(receipt.outcome, EquipmentActionOutcome::Committed);
    let target = EquipmentEditorTarget::Task(id);
    let draft = other_editor.draft(&target).unwrap();
    let generation = draft.generation();
    let mut value = draft.value().clone();
    let EquipmentDraft::Task(task) = &mut value else {
        panic!()
    };
    task.name = "Separator A".into();
    other_editor
        .dispatch(
            &mut other_session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Change {
                target: target.clone(),
                generation,
                value: Box::new(value),
            },
        )
        .unwrap();
    other_editor
        .dispatch(
            &mut other_session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Apply(target),
        )
        .unwrap();
    assert_eq!(session.inputs(), other_session.inputs());
}

#[test]
fn unchanged_write_preserves_formal_checkpoint_and_checks_return_issues() {
    let (mut editor, mut session, id) = setup();
    let record = session.inputs().task(&id).unwrap().clone();
    let before = session.checkpoint();
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::UpdateSizingTask(Box::new(record)),
    );
    assert_eq!(receipt.outcome, EquipmentActionOutcome::Unchanged);
    assert_eq!(before, session.checkpoint());
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::CheckSizing(id.clone()),
    );
    let EquipmentActionOutcome::SizingChecked { task, issues } = receipt.outcome else {
        panic!()
    };
    assert_eq!(task, id);
    assert!(!issues.is_empty());
    assert_eq!(before, session.checkpoint());
}

#[test]
fn preflight_does_not_allocate_or_publish_and_execution_rechecks_drafts() {
    let (mut editor, mut session, id) = setup();
    let before = (editor.clone(), session.clone());
    let context = EquipmentActionContext::capture(&session);
    editor
        .preflight_action(
            &session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::CreateSource,
        )
        .unwrap();
    assert_eq!((editor.clone(), session.clone()), before);
    let target = EquipmentEditorTarget::Task(id.clone());
    let draft = editor.draft(&target).unwrap();
    let generation = draft.generation();
    let mut value = draft.value().clone();
    let EquipmentDraft::Task(task) = &mut value else {
        panic!()
    };
    task.name = "Pending".into();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Change {
                target,
                generation,
                value: Box::new(value),
            },
        )
        .unwrap();
    let record = session.inputs().task(&id).unwrap().clone();
    let before = (editor.clone(), session.clone());
    assert_eq!(
        editor.execute_action(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::UpdateSizingTask(Box::new(record))
        ),
        Err(EquipmentActionError::Edit(
            EquipmentEditError::PendingDrafts
        ))
    );
    assert_eq!((editor.clone(), session.clone()), before);
    apply(&mut editor, &mut session, EquipmentAction::CreateSource);
    assert_eq!(editor.pending_count(), 1);
}

#[test]
fn delete_confirmation_is_explicit_stale_and_single_use() {
    let (mut editor, mut session, id) = setup();
    let target = EquipmentEditorTarget::Task(id.clone());
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::Delete(target.clone()),
    );
    let EquipmentActionOutcome::ConfirmationRequired { key, .. } = receipt.outcome else {
        panic!()
    };
    assert!(session.inputs().task(&id).is_some());
    apply(&mut editor, &mut session, EquipmentAction::CreateSource);
    let context = EquipmentActionContext::capture(&session);
    assert_eq!(
        editor.execute_action(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::Confirm(key)
        ),
        Err(EquipmentActionError::Edit(EquipmentEditError::StaleEditor))
    );
    let receipt = apply(&mut editor, &mut session, EquipmentAction::Delete(target));
    let EquipmentActionOutcome::ConfirmationRequired { key, .. } = receipt.outcome else {
        panic!()
    };
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::Confirm(key.clone()),
    );
    assert_eq!(receipt.outcome, EquipmentActionOutcome::Committed);
    assert!(session.inputs().task(&id).is_none());
    let context = EquipmentActionContext::capture(&session);
    assert_eq!(
        editor.execute_action(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            EquipmentAction::Confirm(key)
        ),
        Err(EquipmentActionError::Edit(EquipmentEditError::StaleEditor))
    );
}

#[test]
fn shared_basis_preview_preserves_calculation_completed_while_waiting() {
    use rf_model::engineering::{EngineeringValue, InputBinding};
    use rf_types::units::QuantityKind;
    let (mut editor, mut session, id) = setup();
    let source = apply(&mut editor, &mut session, EquipmentAction::CreateSource)
        .target
        .unwrap();
    let EquipmentEditorTarget::Source(source_id) = source else {
        panic!()
    };
    let basis = apply(
        &mut editor,
        &mut session,
        EquipmentAction::CreateBasis {
            name: "Temperature".into(),
            value: EngineeringValue {
                quantity: QuantityKind::AbsoluteTemperature,
                value_si: 300.0,
            },
            source_id,
        },
    )
    .target
    .unwrap();
    let EquipmentEditorTarget::Basis(basis_id) = basis else {
        panic!()
    };
    let mut task = session.inputs().task(&id).unwrap().clone();
    task.case.inputs.insert(
        rf_model::equipment::SeparatorInput::OperatingTemperature,
        InputBinding::BasisRef {
            basis_entry_id: basis_id.clone(),
        },
    );
    apply(
        &mut editor,
        &mut session,
        EquipmentAction::UpdateSizingTask(Box::new(task)),
    );
    let other = apply(&mut editor, &mut session, EquipmentAction::CreateSizingTask)
        .target
        .unwrap();
    let EquipmentEditorTarget::Task(other_id) = other else {
        panic!()
    };
    let request = session.begin_calculation(&other_id).unwrap();
    let run_id = request.qualification().id().clone();
    let mut record = session
        .inputs()
        .basis
        .basis_entries
        .iter()
        .find(|b| b.id == basis_id)
        .unwrap()
        .clone();
    record.value.value_si = 310.0;
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::UpdateBasis(record),
    );
    assert_eq!(receipt.impacts.len(), 1);
    assert_eq!(receipt.impacts[0].task_id, id);
    let EquipmentActionOutcome::ConfirmationRequired { key, .. } = receipt.outcome else {
        panic!()
    };
    assert_eq!(
        session.inputs().basis.basis_entries[0].value.value_si,
        300.0
    );
    session.complete_calculation(request.execute()).unwrap();
    let receipt = apply(&mut editor, &mut session, EquipmentAction::Confirm(key));
    assert_eq!(receipt.target, Some(EquipmentEditorTarget::Basis(basis_id)));
    assert_eq!(
        session.inputs().basis.basis_entries[0].value.value_si,
        310.0
    );
    assert_eq!(session.calculation_id(&other_id), Some(&run_id));
    assert!(session.current_failure(&other_id).is_some());
}

#[test]
fn rating_membership_reports_allocated_identity_and_exact_row_check() {
    let (mut editor, mut session, _) = setup();
    let receipt = apply(&mut editor, &mut session, EquipmentAction::CreateRatingTask);
    let Some(EquipmentEditorTarget::RatingGeometry(task)) = receipt.target else {
        panic!()
    };
    let first = session.inputs().rating_task(&task).unwrap().cases[0]
        .id
        .clone();
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::AddRatingCase {
            task: task.clone(),
            copy_from: Some(first.clone()),
        },
    );
    let Some(EquipmentEditorTarget::RatingCase(member)) = receipt.target else {
        panic!()
    };
    assert_ne!(member.case_id, first);
    let receipt = apply(
        &mut editor,
        &mut session,
        EquipmentAction::CheckRating(member.clone()),
    );
    assert!(
        matches!(receipt.outcome, EquipmentActionOutcome::RatingChecked { member: checked, issues } if checked == member && !issues.is_empty())
    );
    apply(
        &mut editor,
        &mut session,
        EquipmentAction::ReorderRatingCases {
            task: task.clone(),
            cases: vec![member.case_id.clone(), first],
        },
    );
    assert_eq!(
        session.inputs().rating_task(&task).unwrap().cases[0].id,
        member.case_id
    );
}
