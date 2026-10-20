use super::*;
use rf_ui::composition_commands::{
    CompositionNormalizationRequest, preview_composition_normalization,
};

#[test]
fn normalization_is_read_only_until_one_atomic_acceptance_and_history_roundtrip() {
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let before = app.clone();
    let preview = normalization(&mut session, &app);
    let RecommendedAction::NormalizeComposition(values) = &preview.recommendation().action else {
        panic!()
    };
    assert_eq!(
        values.original,
        [("a".into(), 0.2), ("b".into(), 0.3)].into()
    );
    assert_eq!(
        values.normalized,
        [("a".into(), 0.4), ("b".into(), 0.6)].into()
    );
    assert_eq!(values.sum, 0.5);
    assert_eq!(app, before);
    let receipt = session.accept(&mut app, &preview, UNIX_EPOCH).unwrap();
    assert_eq!(receipt.revision, 1);
    assert_eq!(app.workspace.command_history.len(), 1);
    assert_eq!(source(&app).overall_mole_fractions, values.normalized);
    assert!(matches!(
        receipt.command,
        DocumentCommand::SetStreamSpecifications { .. }
    ));
    assert_eq!(session.checks().current_count(), Some(0));
    assert!(app.workspace.solve_session.latest_snapshot.is_none());
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    session.refresh(&app);
    assert_eq!(flow(&app), flow(&before));
    assert!(
        session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition)
    );
    app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
    session.refresh(&app);
    assert_eq!(source(&app).overall_mole_fractions, values.normalized);
    assert_eq!(session.checks().current_count(), Some(0));
}

#[test]
fn normalization_requires_valid_committed_fractions_and_project_membership() {
    let cases: &[(&[f64], bool)] = &[
        (&[], false),
        (&[0., 0.], false),
        (&[-0.1, 0.3], false),
        (&[1.2, 0.3], false),
        (&[f64::NAN, 0.3], false),
        (&[f64::INFINITY, 0.3], false),
        (&[0.4, 0.6], false),
        (&[0.5, 0.5 + 0.5e-9], false),
        (&[0.5, 0.5 + 2e-9], true),
        (&[0., 0.3], true),
        (&[0.2, 0.3], true),
    ];
    for (values, expected) in cases {
        let mut app = feed_app();
        app.workspace
            .document
            .flowsheet
            .streams
            .get_mut(&"source".into())
            .unwrap()
            .overall_mole_fractions = ["a", "b"]
            .into_iter()
            .zip(values.iter().copied())
            .map(|(k, v)| (k.into(), v))
            .collect();
        let mut session = ModelingRecommendationSession::new(&app.workspace.document);
        session.refresh(&app);
        assert_eq!(
            session
                .entries()
                .iter()
                .any(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition),
            *expected,
            "{values:?}"
        );
        assert_eq!(app.workspace.document.revision, 0);
        assert_eq!(app.workspace.command_history.len(), 0);
    }
    let mut app = feed_app();
    app.workspace
        .document
        .flowsheet
        .components
        .remove(&ComponentId::new("a"));
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    session.refresh(&app);
    assert!(
        !session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition)
    );
    assert!(
        session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::LocateInput)
    );
}

#[test]
fn committed_normalization_reuses_inspector_transaction_without_requiring_focus() {
    let mut app = feed_app();
    let mut inspector = app.clone();
    let preview =
        preview_composition_normalization(&app.workspace.document, &"source".into()).unwrap();
    app.normalize_committed_stream_composition(
        CompositionNormalizationRequest {
            document_id: app.workspace.document.metadata.document_id.clone(),
            expected_revision: 0,
            preview,
        },
        UNIX_EPOCH,
    )
    .unwrap();
    inspector.focus_inspector_target(InspectorTarget::Stream("source".into()));
    inspector
        .normalize_stream_inspector_composition_drafts(&"source".into(), UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert_eq!(app.workspace.document, inspector.workspace.document);
    assert_eq!(
        app.workspace.command_history,
        inspector.workspace.command_history
    );
}

#[test]
fn committed_normalization_rejects_forged_or_stale_previews_without_changes() {
    let mut app = feed_app();
    let preview =
        preview_composition_normalization(&app.workspace.document, &"source".into()).unwrap();
    let mut request = CompositionNormalizationRequest {
        document_id: app.workspace.document.metadata.document_id.clone(),
        expected_revision: 0,
        preview,
    };
    request.preview.normalized.insert("a".into(), 0.7);
    let before = app.clone();
    assert_eq!(
        app.normalize_committed_stream_composition(request.clone(), UNIX_EPOCH),
        Err(CompositionNormalizationError::PreviewChanged)
    );
    assert_eq!(app, before);
    request.document_id = "foreign".into();
    assert_eq!(
        app.normalize_committed_stream_composition(request.clone(), UNIX_EPOCH),
        Err(CompositionNormalizationError::DifferentDocument)
    );
    assert_eq!(app, before);
    request.document_id = app.workspace.document.metadata.document_id.clone();
    request.expected_revision = 99;
    assert!(matches!(
        app.normalize_committed_stream_composition(request, UNIX_EPOCH),
        Err(CompositionNormalizationError::RevisionConflict { .. })
    ));
    assert_eq!(app, before);
}

#[test]
fn legacy_inspector_weight_normalization_is_preserved_but_not_recommended() {
    let mut app = feed_app();
    app.workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"source".into())
        .unwrap()
        .overall_mole_fractions = [("a".into(), 1.2), ("b".into(), 0.3)].into();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    session.refresh(&app);
    assert!(
        !session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition)
    );
    app.focus_inspector_target(InspectorTarget::Stream("source".into()));
    app.normalize_stream_inspector_composition_drafts(&"source".into(), UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert!((source(&app).overall_mole_fractions[&"a".into()] - 0.8).abs() < 1e-15);
}
