use super::*;
use crate::qs1_test_support::select;
use crate::qs2_test_support::{cross_case, explicit_plan, plan};

#[test]
fn modular_save_reopen_preserves_bound_scales_units_and_missing_results() {
    let dir = Directory::new();
    let path = dir.0.join("cross.json");
    let mut app = cross_case(0.6, 0.25);
    let config = explicit_plan(&app);
    select(&mut app, config.clone());
    let original_flow = app.workspace.document.flowsheet.clone();
    let mut units = rf_types::units::DisplayUnitSet::si();
    units
        .set_unit(
            rf_types::units::QuantityKind::AbsolutePressure,
            rf_types::units::MeasurementUnit::Kilopascal,
        )
        .unwrap();
    app.workspace
        .project_presentation
        .apply(rf_ui::ProjectPresentationCommand::Apply(units.clone()));
    let mut draft = FlowPlanDraft::new(&app.workspace, config.clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650_000.0,
            rf_types::units::MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft.clone());
    let before = app.workspace.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(app.workspace, before);
    assert!(!path.exists());
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));
    let reopened = load_project_app_state(&path).unwrap();
    assert_eq!(reopened.workspace.flow_plan.current(), &config);
    assert_eq!(reopened.workspace.document.flowsheet, original_flow);
    assert_eq!(
        reopened.workspace.project_presentation.display_units(),
        &units
    );
    assert!(
        !reopened
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert!(reopened.workspace.flow_plan.draft().is_none());
    assert!(matches!(
        reopened.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Missing { .. }
    ));
    assert!(
        reopened
            .workspace
            .flow_plan
            .current()
            .validate_for(&reopened.workspace)
            .is_ok()
    );
    assert_eq!(
        project_save_compatibility(&reopened, &dir.0.join("copy.json")).unwrap(),
        None
    );
    assert_eq!(
        reopened
            .workspace
            .flow_plan
            .saved_capabilities()
            .get(project_v4::SIMULTANEOUS_CAPABILITY),
        Some(&1)
    );
}

#[test]
fn v4_save_as_checks_source_capabilities_and_keeps_all_baselines_on_failure() {
    let dir = Directory::new();
    let old_path = dir.0.join("old.json");
    let new_path = dir.0.join("new.json");
    let mut app = cross_case(0.6, 0.25);
    select(&mut app, FlowPlanConfiguration::Acyclic);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: old_path.clone(),
        },
    )
    .unwrap();
    let original = fs::read(&old_path).unwrap();
    select(&mut app, plan());
    let compatibility = project_save_compatibility(&app, &new_path)
        .unwrap()
        .unwrap();
    let ProjectSaveCompatibility::FlowCapabilities {
        required,
        added_from_source,
    } = compatibility
    else {
        panic!("same outer version must be a capability review")
    };
    assert!(
        required
            .iter()
            .any(|c| c.id == project_v4::SIMULTANEOUS_CAPABILITY)
    );
    assert_eq!(
        added_from_source
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec![project_v4::SIMULTANEOUS_CAPABILITY]
    );
    let before = app.workspace.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs {
                path: new_path.clone()
            }
        )
        .is_err()
    );
    assert_eq!(app.workspace, before);
    assert!(!new_path.exists());
    let bad_path = dir.0.join("directory");
    fs::create_dir(&bad_path).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: bad_path }
        )
        .is_err()
    );
    assert_eq!(app.workspace, before);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: new_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(fs::read(&old_path).unwrap(), original);
    assert_eq!(
        project_save_compatibility(&app, &dir.0.join("next.json")).unwrap(),
        None
    );
    assert!(
        project_save_compatibility(&app, &old_path)
            .unwrap()
            .is_some()
    );
    let mut reopened = load_project_app_state(&new_path).unwrap();
    select(&mut reopened, FlowPlanConfiguration::Acyclic);
    assert_eq!(project_save_version(&reopened), 4);
    dispatch_document_lifecycle(&mut reopened, StudioDocumentLifecycleCommand::Save).unwrap();
    let text = fs::read_to_string(&new_path).unwrap();
    assert!(!text.contains(project_v4::SIMULTANEOUS_CAPABILITY));
    assert_eq!(
        project_save_version(&load_project_app_state(&new_path).unwrap()),
        4
    );
}

#[test]
fn incomplete_modular_scale_bindings_roundtrip_and_old_protocol_stays_readonly() {
    let dir = Directory::new();
    let path = dir.0.join("incomplete.json");
    let mut app = cross_case(0.6, 0.25);
    let FlowPlanConfiguration::Simultaneous(mut options) = explicit_plan(&app) else {
        unreachable!()
    };
    let (mut binding, scale) = options.scales.as_mut().unwrap().pop_first().unwrap();
    binding.source.unit_id = "mixer-99".into();
    binding.stream = "reserved-stream".into();
    options.scales.as_mut().unwrap().insert(binding, scale);
    let config = FlowPlanConfiguration::Simultaneous(options);
    select(&mut app, config.clone());
    assert!(
        app.workspace
            .flow_plan
            .current()
            .validate_for(&app.workspace)
            .is_err()
    );
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    let reopened = load_project_app_state(&path).unwrap();
    assert_eq!(reopened.workspace.flow_plan.current(), &config);
    assert!(
        reopened
            .workspace
            .flow_plan
            .current()
            .referenced_unit_ids()
            .contains(&"mixer-99".into())
    );
    assert!(
        reopened
            .workspace
            .flow_plan
            .current()
            .referenced_stream_ids()
            .contains(&"reserved-stream".into())
    );
    assert!(
        reopened
            .workspace
            .flow_plan
            .current()
            .validate_for(&reopened.workspace)
            .is_err()
    );
    let StudioProjectOpen::ReadOnly(readonly) =
        read_studio_project_with_support(&path, project_v4::FlowPlanSupport::Sequential).unwrap()
    else {
        panic!("old consumers must not get AppState")
    };
    assert_eq!(readonly.original_json(), fs::read_to_string(&path).unwrap());
    let request = dir.0.join("query.json");
    fs::write(&request, serde_json::json!({"schema_version":4,"project":path,"queries":[{"id":"plan","query":{"kind":"results","scope":"flow_plan"}}]}).to_string()).unwrap();
    let mut stdout = vec![];
    let mut stderr = vec![];
    assert_eq!(
        crate::headless::run_cli(
            &["query".into(), request.into_os_string()],
            &mut stdout,
            &mut stderr
        ),
        0
    );
    let response: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(response["project"]["read_only"], true);
    assert_eq!(response["queries"][0]["value"]["read_only"], true);
    assert!(
        response["queries"][0]["value"]
            .get("configuration")
            .is_none()
    );
}
