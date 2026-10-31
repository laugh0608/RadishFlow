use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::*;

fn setup() -> (
    EngineeringSession,
    EquipmentEditorState,
    EquipmentEditorTarget,
) {
    let mut session = EngineeringSession::new("editor-project".into());
    let mut editor = EquipmentEditorState::default();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::CreateTask,
        )
        .unwrap();
    let target = editor.active().unwrap().clone();
    (session, editor, target)
}
fn change(
    editor: &mut EquipmentEditorState,
    session: &mut EngineeringSession,
    target: &EquipmentEditorTarget,
    edit: impl FnOnce(&mut EquipmentDraft),
) {
    let record = editor.draft(target).unwrap();
    let mut value = record.value().clone();
    edit(&mut value);
    editor
        .dispatch(
            session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Change {
                target: target.clone(),
                generation: record.generation(),
                value: Box::new(value),
            },
        )
        .unwrap();
}
#[test]
fn incomplete_text_is_preserved_by_navigation_save_capture_and_rejected_apply() {
    let (mut session, mut editor, target) = setup();
    change(&mut editor, &mut session, &target, |draft| {
        let EquipmentDraft::Task(task) = draft else {
            panic!()
        };
        let field = task
            .fields
            .get_mut(&EquipmentField::Input(SeparatorInput::OperatingTemperature))
            .unwrap();
        field.mode = BindingMode::Direct;
        field.value.set_text("1e-".into());
    });
    let checkpoint = editor.checkpoint();
    let before = session.clone();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::CreateSource,
        )
        .unwrap();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Select(target.clone()),
        )
        .unwrap();
    assert_eq!(editor.pending_count(), 1);
    let task = session.inputs().tasks[0].id().clone();
    assert_eq!(
        editor.dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Calculate(task)
        ),
        Err(EquipmentEditError::PendingDrafts)
    );
    assert!(
        editor
            .dispatch(
                &mut session,
                &DisplayUnitSet::si(),
                EquipmentEditorCommand::Apply(target.clone())
            )
            .is_err()
    );
    assert_eq!(session.inputs().tasks, before.inputs().tasks);
    session.record_saved(session.capture_save()).unwrap();
    assert_eq!(editor.checkpoint(), checkpoint);
    assert_eq!(editor.pending_count(), 1);
}
#[test]
fn unit_changes_preserve_precision_and_cannot_reinterpret_invalid_text() {
    let original = 0.12345678901234567;
    let mut value =
        QuantityDraft::from_si(original, QuantityKind::Length, MeasurementUnit::Millimetre);
    value
        .select_unit(QuantityKind::Length, MeasurementUnit::Metre)
        .unwrap();
    value
        .select_unit(QuantityKind::Length, MeasurementUnit::Millimetre)
        .unwrap();
    assert_eq!(
        value.value(QuantityKind::Length).unwrap().to_bits(),
        original.to_bits()
    );
    value.set_text("1e-".into());
    let before = value.clone();
    assert!(
        value
            .select_unit(QuantityKind::Length, MeasurementUnit::Metre)
            .is_err()
    );
    assert_eq!(value, before);
}
#[test]
fn history_confirmation_rechecks_drafts_and_does_not_discard_on_cancel_or_stale_confirm() {
    let (mut session, mut editor, target) = setup();
    change(&mut editor, &mut session, &target, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name = "Draft A".into();
    });
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::History { redo: false },
        )
        .unwrap();
    let key = editor.confirmation().unwrap().key();
    change(&mut editor, &mut session, &target, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name = "Draft B".into();
    });
    assert_eq!(
        editor.dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Confirm(key)
        ),
        Err(EquipmentEditError::StaleEditor)
    );
    assert_eq!(session.inputs().tasks.len(), 1);
    assert_eq!(editor.pending_count(), 1);
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::CancelConfirmation,
        )
        .unwrap();
    assert_eq!(editor.pending_count(), 1);
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::History { redo: false },
        )
        .unwrap();
    let key = editor.confirmation().unwrap().key();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Confirm(key),
        )
        .unwrap();
    assert!(session.inputs().tasks.is_empty());
    assert_eq!(editor.pending_count(), 0);
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::History { redo: true },
        )
        .unwrap();
    assert_eq!(session.inputs().tasks.len(), 1);
}
#[test]
fn shared_source_edits_preview_impacts_and_referenced_deletion_is_rejected() {
    let (mut session, mut editor, task_target) = setup();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::CreateSource,
        )
        .unwrap();
    let source_target = editor.active().unwrap().clone();
    let EquipmentEditorTarget::Source(source_id) = &source_target else {
        panic!()
    };
    change(&mut editor, &mut session, &task_target, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.applicability_sources.push(source_id.clone());
    });
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Apply(task_target),
        )
        .unwrap();
    change(&mut editor, &mut session, &source_target, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.kind = SourceKind::Assumed;
        s.adoption_note = "Explicit design assumption".into();
        s.review = SourceReview::Adopted;
    });
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Apply(source_target.clone()),
        )
        .unwrap();
    let pending = editor.confirmation().unwrap();
    assert_eq!(pending.impacts().len(), 1);
    assert!(pending.impacts()[0].evidence_changed);
    assert_eq!(session.inputs().basis.sources[0].kind, SourceKind::Unknown);
    let key = pending.key();
    editor
        .dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Confirm(key),
        )
        .unwrap();
    assert_eq!(session.inputs().basis.sources[0].kind, SourceKind::Assumed);
    assert!(matches!(
        editor.dispatch(
            &mut session,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Delete(source_target)
        ),
        Err(EquipmentEditError::Domain(
            EngineeringError::Referenced { .. }
        ))
    ));
}
