use super::super::rating_tests::{member, session, task};
use super::*;
use rf_model::equipment::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit, QuantityKind};

fn dispatch(
    editor: &mut EquipmentEditorState,
    session: &mut EngineeringSession,
    command: EquipmentEditorCommand,
) {
    editor
        .dispatch(session, &DisplayUnitSet::si(), command)
        .unwrap();
}
fn change(
    editor: &mut EquipmentEditorState,
    session: &mut EngineeringSession,
    target: &EquipmentEditorTarget,
    edit: impl FnOnce(&mut EquipmentDraft),
) {
    let record = editor.draft(target).unwrap();
    let generation = record.generation();
    let mut value = record.value().clone();
    edit(&mut value);
    dispatch(
        editor,
        session,
        EquipmentEditorCommand::Change {
            target: target.clone(),
            generation,
            value: Box::new(value),
        },
    );
}
fn geometry() -> EquipmentEditorTarget {
    EquipmentEditorTarget::RatingGeometry(member().task_id)
}
fn case() -> EquipmentEditorTarget {
    EquipmentEditorTarget::RatingCase(member())
}

#[test]
fn create_rating_starts_incomplete_and_uses_distinct_geometry_and_case_drafts() {
    let mut s = EngineeringSession::new("editor".into());
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::CreateRatingTask);
    let t = s.inputs().tasks[0].as_rating().unwrap();
    assert_eq!(
        (t.revision, t.geometry.revision, t.cases[0].revision),
        (1, 1, 1)
    );
    assert_eq!(t.cases.len(), 1);
    assert!(t.geometry.inputs.is_empty());
    let key = EquipmentCaseKey {
        task_id: t.id.clone(),
        case_id: t.cases[0].id.clone(),
    };
    assert_eq!(
        e.active(),
        Some(&EquipmentEditorTarget::RatingGeometry(t.id.clone()))
    );
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::CheckRating(key.clone()),
    );
    assert!(!e.checked_rating_issues(&key, &s).unwrap().is_empty());
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::CalculateRating(key.clone()),
    );
    assert_eq!(s.case_result_status(&key), EquipmentResultStatus::Failed);
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::History { redo: false },
    );
    assert!(s.inputs().tasks.is_empty());
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::History { redo: true },
    );
    assert_eq!(s.case_result_status(&key), EquipmentResultStatus::Stale);
}

#[test]
fn invalid_geometry_text_survives_navigation_and_save_and_blocks_only_its_dependencies() {
    let mut s = session();
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(geometry()));
    change(&mut e, &mut s, &geometry(), |draft| {
        let EquipmentDraft::RatingGeometry(g) = draft else {
            panic!()
        };
        g.fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::WorkingLiquidHeight,
            ))
            .unwrap()
            .value
            .set_text("1e-".into());
    });
    let before = task(&s);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(case()));
    assert!(e.pending_for_case(&member(), &s));
    for command in [
        EquipmentEditorCommand::CheckRating(member()),
        EquipmentEditorCommand::CalculateRating(member()),
    ] {
        assert_eq!(
            e.dispatch(&mut s, &DisplayUnitSet::si(), command),
            Err(EquipmentEditError::PendingDrafts)
        );
    }
    assert!(matches!(
        e.dispatch(
            &mut s,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Apply(geometry())
        ),
        Err(EquipmentEditError::RatingGeometryField(_, _))
    ));
    s.record_saved(s.capture_save()).unwrap();
    assert_eq!(task(&s), before);
    assert_eq!(e.pending_count(), 1);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(geometry()));
    let EquipmentDraft::RatingGeometry(g) = e.draft(&geometry()).unwrap().value() else {
        panic!()
    };
    assert_eq!(
        g.fields[&RatingGeometryField::Input(SeparatorGeometryInput::WorkingLiquidHeight)]
            .value
            .text(),
        "1e-"
    );
}

#[test]
fn geometry_confirmation_is_bound_to_formal_and_draft_checkpoints_and_preserves_case_draft() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(geometry()));
    change(&mut e, &mut s, &geometry(), |draft| {
        let EquipmentDraft::RatingGeometry(g) = draft else {
            panic!()
        };
        g.fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::WorkingLiquidHeight,
            ))
            .unwrap()
            .value
            .set_text("0.7".into());
    });
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(geometry()));
    let pending = e.confirmation().unwrap();
    assert_eq!(pending.impacts()[0].case_id, member().case_id);
    let stale = pending.key();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(case()));
    change(&mut e, &mut s, &case(), |draft| {
        let EquipmentDraft::RatingCase(c) = draft else {
            panic!()
        };
        c.name = "Pending label".into();
    });
    let before = s.clone();
    assert_eq!(
        e.dispatch(
            &mut s,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Confirm(stale)
        ),
        Err(EquipmentEditError::StaleEditor)
    );
    assert_eq!(s, before);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::CancelConfirmation);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(geometry()));
    let key = e.confirmation().unwrap().key();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Confirm(key));
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert!(e.draft(&case()).unwrap().is_pending());
    assert!(!e.draft(&geometry()).is_some_and(|d| d.is_pending()));
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(case()));
    assert_eq!(task(&s).cases[0].name, "Pending label");
}

#[test]
fn external_case_edit_rejects_old_draft_without_erasing_user_text() {
    let mut s = session();
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(case()));
    change(&mut e, &mut s, &case(), |draft| {
        let EquipmentDraft::RatingCase(c) = draft else {
            panic!()
        };
        c.purpose = "My pending note".into();
    });
    let mut updated = task(&s).cases.remove(0);
    updated.name = "External edit".into();
    s.apply(vec![EngineeringCommand::UpdateRatingCase {
        task: member().task_id,
        case: Box::new(updated),
    }])
    .unwrap();
    let before = s.clone();
    assert_eq!(
        e.dispatch(
            &mut s,
            &DisplayUnitSet::si(),
            EquipmentEditorCommand::Apply(case())
        ),
        Err(EquipmentEditError::StaleEditor)
    );
    assert_eq!(s, before);
    let EquipmentDraft::RatingCase(c) = e.draft(&case()).unwrap().value() else {
        panic!()
    };
    assert_eq!(c.purpose, "My pending note");
}

#[test]
fn shared_source_drafts_gate_rating_and_unrelated_source_drafts_do_not() {
    let mut s = session();
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::CreateSource);
    let unrelated = e.active().unwrap().clone();
    change(&mut e, &mut s, &unrelated, |draft| {
        let EquipmentDraft::Source(source) = draft else {
            panic!()
        };
        source.title = "Unrelated draft".into();
    });
    assert!(!e.pending_for_case(&member(), &s));
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::CalculateRating(member()),
    );
    let drawing = EquipmentEditorTarget::Source("drawing".into());
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::Select(drawing.clone()),
    );
    change(&mut e, &mut s, &drawing, |draft| {
        let EquipmentDraft::Source(source) = draft else {
            panic!()
        };
        source.locator = "Revised drawing location".into();
    });
    assert!(e.pending_for_case(&member(), &s));
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(drawing));
    let pending = e.confirmation().unwrap();
    assert_eq!(pending.impacts().len(), 1);
    assert_eq!(pending.impacts()[0].case_id, member().case_id);
    let key = pending.key();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Confirm(key));
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_eq!(e.pending_targets(), vec![unrelated]);
}

#[test]
fn deleting_rating_discards_its_child_drafts_only_after_confirm_and_undo_stays_stale() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(case()));
    change(&mut e, &mut s, &case(), |draft| {
        let EquipmentDraft::RatingCase(c) = draft else {
            panic!()
        };
        c.name = "Pending".into();
    });
    let root = EquipmentEditorTarget::RatingTask(member().task_id);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(root.clone()));
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Delete(root.clone()));
    assert_eq!(e.confirmation().unwrap().discarded_drafts(), &[case()]);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::CancelConfirmation);
    assert_eq!(e.pending_count(), 1);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Delete(root));
    let key = e.confirmation().unwrap().key();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Confirm(key));
    assert!(s.inputs().tasks.is_empty());
    assert!(e.pending_targets().is_empty());
    assert!(e.active().is_none());
    dispatch(
        &mut e,
        &mut s,
        EquipmentEditorCommand::History { redo: false },
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Stale
    );
    assert_ne!(task(&s).cases[0].name, "Pending");
}

#[test]
fn rating_unit_roundtrip_keeps_exact_formal_geometry_and_metadata_edits_keep_current_result() {
    let mut s = session();
    s.calculate_rating(&member()).unwrap();
    let before = task(&s);
    let mut e = EquipmentEditorState::default();
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(geometry()));
    change(&mut e, &mut s, &geometry(), |draft| {
        let EquipmentDraft::RatingGeometry(g) = draft else {
            panic!()
        };
        let value = &mut g
            .fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::InternalDiameter,
            ))
            .unwrap()
            .value;
        value
            .select_unit(QuantityKind::Length, MeasurementUnit::Millimetre)
            .unwrap();
        value
            .select_unit(QuantityKind::Length, MeasurementUnit::Metre)
            .unwrap();
    });
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(geometry()));
    assert_eq!(task(&s), before);
    assert!(!s.is_dirty());
    let root = EquipmentEditorTarget::RatingTask(member().task_id);
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Select(root.clone()));
    change(&mut e, &mut s, &root, |draft| {
        let EquipmentDraft::RatingTask(labels) = draft else {
            panic!()
        };
        labels.name = "New task name".into();
    });
    dispatch(&mut e, &mut s, EquipmentEditorCommand::Apply(root));
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    assert_eq!(
        s.current_rating(&member()).unwrap().capture().input.name,
        before.name
    );
}
