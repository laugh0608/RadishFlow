use super::*;
use crate::qs1_test_support::select;
fn cross_case(_: f64, _: f64) -> AppState {
    crate::qs3_test_support::case(true)
}
fn explicit_plan(app: &AppState) -> FlowPlanConfiguration {
    crate::qs3_test_support::explicit(app, true)
}
fn plan() -> FlowPlanConfiguration {
    FlowPlanConfiguration::EquationOriented(crate::qs3_test_support::options(true))
}

#[test]
fn equation_save_reopen_preserves_bound_scales_units_and_missing_results() {
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
            .get(project_v4::EQUATION_ORIENTED_CAPABILITY),
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
            .any(|c| c.id == project_v4::EQUATION_ORIENTED_CAPABILITY)
    );
    assert_eq!(
        added_from_source
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec![project_v4::EQUATION_ORIENTED_CAPABILITY]
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
    assert!(!text.contains(project_v4::EQUATION_ORIENTED_CAPABILITY));
    assert_eq!(
        project_save_version(&load_project_app_state(&new_path).unwrap()),
        4
    );
}
