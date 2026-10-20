use super::*;
use crate::{StudioDocumentLifecycleCommand, dispatch_document_lifecycle, load_project_app_state};
use rf_ui::variable_browser::{
    ObjectId, VariableField, VariableId, VariableSection, VariableValue,
};
use rf_ui::variable_commands::VariableWriteRequest;

fn temp_root(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "radishflow-a2-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn save_reopen_rebuilds_recommendations_and_never_persists_ignore_or_preview() {
    let root = temp_root("save");
    let path = root.join("project.rfproj.json");
    let mut app = feed_app();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let preview = normalization(&mut session, &app);
    session
        .set_ignored(&app, &preview.recommendation().key, true)
        .unwrap();
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    let bytes = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    let fields = value["document"].as_object().unwrap();
    assert_eq!(fields.len(), 3);
    assert!(
        fields.contains_key("flowsheet")
            && fields.contains_key("metadata")
            && fields.contains_key("revision")
    );
    let mut reopened = load_project_app_state(&path).unwrap();
    assert_eq!(flow(&reopened), flow(&app));
    assert!(rf_ui::latest_snapshot(&reopened.workspace).is_none());
    session.replace_document(&reopened.workspace.document);
    session.refresh(&reopened);
    assert!(
        session
            .entries()
            .iter()
            .all(|e| e.visibility == RecommendationVisibility::Available)
    );
    assert_eq!(
        session.accept(&mut reopened, &preview, UNIX_EPOCH),
        Err(RecommendationError::DifferentSession)
    );
    let preview = normalization(&mut session, &reopened);
    session.accept(&mut reopened, &preview, UNIX_EPOCH).unwrap();
    dispatch_document_lifecycle(&mut reopened, StudioDocumentLifecycleCommand::Save).unwrap();
    let mut clean = load_project_app_state(&path).unwrap();
    assert_eq!(
        source(&clean).overall_mole_fractions,
        [("a".into(), 0.4), ("b".into(), 0.6)].into()
    );
    session.replace_document(&clean.workspace.document);
    session.refresh(&clean);
    assert_eq!(session.checks().current_count(), Some(0));
    assert!(
        !session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition)
    );
    assert!(rf_ui::latest_snapshot(&clean.workspace).is_none());
    let before = clean.clone();
    assert_eq!(
        session.accept(&mut clean, &preview, UNIX_EPOCH),
        Err(RecommendationError::DifferentSession)
    );
    assert_eq!(clean, before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn normalization_does_not_restore_old_result_qualification_before_real_run() {
    let root = temp_root("run");
    let mut index = rf_store::StoredAuthCacheIndex::new(
        "https://example.invalid",
        "a2-test",
        rf_store::StoredCredentialReference::new("test", "a2-test"),
    );
    crate::test_support::write_official_binary_hydrocarbon_cached_package(
        &root,
        &mut index,
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        std::time::SystemTime::now(),
        None,
    );
    let context = crate::StudioAppAuthCacheContext::new(&root, &index);
    let project = rf_store::parse_project_file_json(
        crate::test_support::official_heater_binary_hydrocarbon_project_json(),
    )
    .unwrap();
    let mut app = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("a2-run", "A2 Run", UNIX_EPOCH),
    ));
    let run = |app: &mut AppState| {
        crate::dispatch_workspace_control_action_with_auth_cache(
            &crate::StudioAppFacade::new(),
            app,
            &context,
            &crate::WorkspaceControlAction::run_manual(
                crate::WorkspaceRunPackageSelection::Preferred,
            ),
        )
        .unwrap()
    };
    run(&mut app);
    assert!(rf_ui::latest_snapshot(&app.workspace).is_some());
    for (component, fraction) in [("methane", 0.2), ("ethane", 0.3)] {
        app.write_variable(
            VariableWriteRequest {
                variable: VariableId {
                    document: app.workspace.document.metadata.document_id.clone(),
                    object: ObjectId::Stream("stream-feed".into()),
                    section: VariableSection::Inputs,
                    field: VariableField::MoleFraction(component.into()),
                },
                expected_revision: app.workspace.document.revision,
                value: VariableValue::Number(fraction),
            },
            UNIX_EPOCH,
        )
        .unwrap();
    }
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    assert!(rf_ui::stale_snapshot(&app.workspace).is_some());
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let preview = normalization(&mut session, &app);
    session.accept(&mut app, &preview, UNIX_EPOCH).unwrap();
    assert_eq!(session.checks().current_count(), Some(0));
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    assert!(rf_ui::stale_snapshot(&app.workspace).is_some());
    run(&mut app);
    let snapshot = rf_ui::latest_snapshot(&app.workspace).unwrap();
    assert_eq!(snapshot.document_revision, app.workspace.document.revision);
    assert_eq!(
        app.workspace.solve_session.status,
        rf_ui::RunStatus::Converged
    );
    std::fs::remove_dir_all(root).unwrap();
}
