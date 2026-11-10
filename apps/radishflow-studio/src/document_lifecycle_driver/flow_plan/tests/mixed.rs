use super::*;
#[test]
fn mixed_save_roundtrip_keeps_full_scales_and_formal_inputs_but_no_result_or_draft() {
    let dir = Directory::new();
    let path = dir.0.join("mixed.json");
    let mut app = crate::qs4_test_support::case();
    let config = crate::qs4_test_support::explicit(&app);
    crate::qs1_test_support::select(&mut app, config.clone());
    let flow = app.workspace.document.flowsheet.clone();
    let mut draft = FlowPlanDraft::new(&app.workspace, config.clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650000.0,
            rf_types::units::MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft.clone());
    assert_eq!(project_save_version(&app), 4);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveAs { path: path.clone() },
    )
    .unwrap();
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));
    let reopened = load_project_app_state(&path).unwrap();
    assert_eq!(reopened.workspace.flow_plan.current(), &config);
    assert_eq!(reopened.workspace.document.flowsheet, flow);
    assert!(reopened.workspace.flow_plan.draft().is_none());
    assert!(
        !reopened
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert!(matches!(
        reopened.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Missing { .. }
    ));
    assert_eq!(
        reopened
            .workspace
            .flow_plan
            .saved_capabilities()
            .get(project_v4::MIXED_CAPABILITY),
        Some(&1)
    );
    let mut old = super::app();
    let before = old.workspace.document.revision;
    crate::qs1_test_support::select(&mut old, config);
    assert_eq!(project_save_version(&old), 4);
    assert_eq!(old.workspace.document.revision, before);
    assert!(
        dispatch_document_lifecycle(
            &mut old,
            StudioDocumentLifecycleCommand::SaveAs {
                path: dir.0.join("upgrade.json")
            }
        )
        .is_err()
    );
}
#[test]
fn mixed_capability_upgrade_inside_v4_is_explicit_and_cancel_does_not_write() {
    let dir = Directory::new();
    let path = dir.0.join("existing-v4.json");
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-material-eo-binary-hydrocarbon.rfproj.json"
    ));
    fs::write(&path, raw).unwrap();
    let mut app = load_project_app_state(&path).unwrap();
    let config = crate::qs4_test_support::case()
        .workspace
        .flow_plan
        .current()
        .clone();
    crate::qs1_test_support::select(&mut app, config.clone());
    let compatibility = project_save_compatibility(&app, &path).unwrap().unwrap();
    let _ = compatibility;
    let before = app.workspace.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), raw);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    let reopened = load_project_app_state(&path).unwrap();
    assert_eq!(reopened.workspace.flow_plan.current(), &config);
    assert!(matches!(
        reopened.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Missing { .. }
    ));
}
