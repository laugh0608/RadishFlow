use super::*;
use rf_ui::variable_browser::{
    ObjectId, VariableField, VariableId, VariableSection, VariableValue,
};
use rf_ui::variable_commands::VariableWriteRequest;

fn formal_change(app: &mut AppState) {
    app.write_variable(
        VariableWriteRequest {
            variable: VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object: ObjectId::Stream("source".into()),
                section: VariableSection::Inputs,
                field: VariableField::Temperature,
            },
            expected_revision: app.workspace.document.revision,
            value: VariableValue::Number(310.),
        },
        UNIX_EPOCH,
    )
    .unwrap();
}

#[test]
fn ignore_persists_on_same_revision_recheck_and_never_changes_blockers() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let key = key(&mut session, &app, RecommendationRule::NormalizeComposition);
    let before = app.clone();
    let count = session.checks().current_count();
    let blocker = crate::studio_modeling_run_blocker(&app.workspace.document);
    session.set_ignored(&app, &key, true).unwrap();
    for _ in 0..3 {
        session.recheck(&app);
        assert_eq!(
            session
                .entries()
                .iter()
                .find(|e| e.recommendation.key == key)
                .unwrap()
                .visibility,
            RecommendationVisibility::Ignored
        );
        assert_eq!(session.checks().current_count(), count);
        assert_eq!(
            crate::studio_modeling_run_blocker(&app.workspace.document),
            blocker
        );
    }
    assert_eq!(app, before);
    assert!(matches!(
        session.prepare(&app, &key),
        Err(RecommendationError::Ignored)
    ));
    session.set_ignored(&app, &key, false).unwrap();
    assert!(session.prepare(&app, &key).is_ok());
    session.set_ignored(&app, &key, true).unwrap();
    formal_change(&mut app);
    session.refresh(&app);
    assert_eq!(
        session
            .entries()
            .iter()
            .find(|e| e.recommendation.key == key)
            .unwrap()
            .visibility,
        RecommendationVisibility::Available
    );
}

#[test]
fn revisions_history_and_duplicate_acceptance_reject_old_previews() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let original = normalization(&mut session, &app);
    formal_change(&mut app);
    let before = app.clone();
    assert!(matches!(
        session.accept(&mut app, &original, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, before);
    app.undo_document_command(UNIX_EPOCH).unwrap();
    let before = app.clone();
    assert!(matches!(
        session.accept(&mut app, &original, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, before);
    let current = normalization(&mut session, &app);
    session.accept(&mut app, &current, UNIX_EPOCH).unwrap();
    let before = app.clone();
    assert!(matches!(
        session.accept(&mut app, &current, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, before);
}

#[test]
fn reopening_same_id_revision_and_separate_owners_reject_foreign_capabilities() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let old = normalization(&mut session, &app);
    let before = app.clone();
    let mut other = ModelingRecommendationSession::new(&app.workspace.document);
    let _ = normalization(&mut other, &app);
    assert_eq!(
        other.accept(&mut app, &old, UNIX_EPOCH),
        Err(RecommendationError::DifferentSession)
    );
    session.replace_document(&app.workspace.document);
    assert_eq!(
        session.accept(&mut app, &old, UNIX_EPOCH),
        Err(RecommendationError::DifferentSession)
    );
    assert_eq!(app, before);
    let current = normalization(&mut session, &app);
    app.workspace.document.metadata.document_id = "another".into();
    let before = app.clone();
    assert_eq!(
        session.accept(&mut app, &current, UNIX_EPOCH),
        Err(RecommendationError::DifferentDocument)
    );
    assert_eq!(app, before);
}

#[test]
fn removal_and_same_named_replacement_do_not_rebind_a_connection_preview() {
    let mut app = missing_outlet(BuiltinUnitKind::Feed);
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let key = key(&mut session, &app, RecommendationRule::CreateOutletStream);
    let preview = session.prepare(&app, &key).unwrap();
    app.delete_unit(&"feed".into(), UNIX_EPOCH).unwrap();
    let created = app
        .create_builtin_unit(BuiltinUnitKind::Feed, UNIX_EPOCH)
        .unwrap();
    assert_ne!(created.unit_id, UnitId::new("feed"));
    let before = app.clone();
    assert!(matches!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, before);
}

#[test]
fn numeric_and_composition_drafts_block_writes_but_navigation_stays_read_only() {
    for numeric in [false, true] {
        let mut app = feed_app();
        let mut session = ModelingRecommendationSession::new(&app.workspace.document);
        let preview = normalization(&mut session, &app);
        app.focus_inspector_target(InspectorTarget::Stream("source".into()));
        if numeric {
            let id = VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object: ObjectId::Stream("source".into()),
                section: VariableSection::Inputs,
                field: VariableField::Pressure,
            };
            let generation = app
                .begin_numeric_edit(id.clone(), Some(rf_types::units::MeasurementUnit::Bar))
                .unwrap();
            app.edit_numeric(
                &id,
                generation,
                rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
            )
            .unwrap();
        } else {
            app.update_stream_inspector_draft(
                &"source".into(),
                StreamInspectorDraftField::OverallMoleFraction("a".into()),
                "0.25",
            )
            .unwrap();
        }
        let before = app.clone();
        assert_eq!(
            session.accept(&mut app, &preview, UNIX_EPOCH),
            Err(RecommendationError::PendingDrafts)
        );
        assert_eq!(app, before);
        let nav = key(&mut session, &app, RecommendationRule::LocateInput);
        assert_eq!(
            session.navigation_target(&app, &nav).unwrap(),
            SpecificationTarget::Composition("source".into())
        );
        assert_eq!(
            app, before,
            "navigation must not consume raw text, units or generation"
        );
    }
}

#[test]
fn pending_canvas_edit_and_hidden_preview_cannot_be_accepted() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let preview = normalization(&mut session, &app);
    app.begin_canvas_place_unit("heater");
    let before = app.clone();
    assert_eq!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::PendingCanvasEdit)
    );
    assert_eq!(app, before);
    app.workspace.canvas_interaction.cancel_pending_edit();
    session
        .set_ignored(&app, &preview.recommendation().key, true)
        .unwrap();
    session
        .set_ignored(&app, &preview.recommendation().key, false)
        .unwrap();
    let before = app.clone();
    assert_eq!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::PreviewExpired)
    );
    assert_eq!(app, before);
}

#[test]
fn regenerated_content_is_compared_even_if_a_caller_bypasses_revision_discipline() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let preview = normalization(&mut session, &app);
    app.workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"source".into())
        .unwrap()
        .overall_mole_fractions
        .insert("a".into(), 0.25);
    let before = app.clone();
    assert_eq!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::PreviewChanged)
    );
    assert_eq!(app, before);
    let mut app = missing_outlet(BuiltinUnitKind::Feed);
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let key = key(&mut session, &app, RecommendationRule::CreateOutletStream);
    let preview = session.prepare(&app, &key).unwrap();
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"feed".into())
        .unwrap()
        .ports[0]
        .direction = PortDirection::Inlet;
    let before = app.clone();
    assert_eq!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::Unavailable)
    );
    assert_eq!(app, before);
}

#[test]
fn successful_commit_with_failed_recheck_returns_success_and_cannot_be_replayed() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let preview = normalization(&mut session, &app);
    let receipt = session
        .accept_with_recheck(&mut app, &preview, UNIX_EPOCH, |checks, document| {
            let ticket = checks.begin(document);
            checks.finish(&ticket, Err(ModelingCheckFailure::Panicked), document);
        })
        .unwrap();
    assert_eq!(receipt.revision, 1);
    assert_eq!(
        receipt.check_phase,
        ModelingCheckPhase::Failed(ModelingCheckFailure::Panicked)
    );
    assert_eq!(
        source(&app).overall_mole_fractions,
        [("a".into(), 0.4), ("b".into(), 0.6)].into()
    );
    assert_eq!(session.last_receipt(), Some(&receipt));
    assert_eq!(session.checks().current_count(), None);
    let before = app.clone();
    assert!(matches!(
        session.accept(&mut app, &preview, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, before);
    session.recheck(&app);
    assert_eq!(session.checks().current_count(), Some(0));
}
