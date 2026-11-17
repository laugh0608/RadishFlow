use super::*;
use crate::{
    build_workspace_solve_request, complete_workspace_solve, prepare_workspace_solve,
    qs1_test_support::select,
    qs4_test_support::recycle_case,
    solve_workspace_with_property_package,
    test_support::{
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        build_official_binary_hydrocarbon_in_memory_provider,
    },
};
use rf_model::equipment::{EquipmentMethod, SeparatorCase, SeparatorSizingTask};
use rf_types::units::{MeasurementUnit, QuantityKind};
use rf_ui::{
    FlowResultRecord, FlowRunDisposition, ProjectPresentationCommand,
    engineering::EngineeringCommand,
    result_query::{FlowExecutionEvidence, FlowResultView},
    variable_browser::{
        ObjectId, VariableField, VariableId, VariableSection, VariableWatchCommand,
    },
};

const M1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
));

fn source(dir: &Directory) -> (AppState, PathBuf) {
    let path = dir.0.join("m1.rfproj.json");
    fs::write(&path, M1).unwrap();
    (load_project_app_state(&path).unwrap(), path)
}

fn selected_m2(budget: usize) -> FlowPlanConfiguration {
    recycle_case(budget).workspace.flow_plan.current().clone()
}

fn reopen(path: &Path) -> AppState {
    let StudioProjectOpen::Editable(app) =
        read_studio_project_with_support(path, FlowPlanSupport::MixedRecycle).unwrap()
    else {
        panic!("an M2 reader must reopen the project as editable");
    };
    *app
}

fn verified_result(app: &AppState) -> FlowExecutionEvidence {
    use rf_solver::partitioned::{ABSOLUTE_FLOW_TOLERANCE, EvaluationPhase, SCALED_FLOW_TOLERANCE};
    let FlowResultView::Current {
        snapshot: FlowResultRecord::Mixed(snapshot),
        origin,
    } = app.workspace.flow_result()
    else {
        panic!("an explicit solve must publish a verified result");
    };
    let result = &snapshot.result;
    for (stream, expected) in [
        ("s-rba", [39.0 / 85.0, 66.0 / 85.0]),
        ("s-heated", [14.0 / 17.0, 8.0 / 17.0]),
        ("s-pb", [117.0 / 85.0, 198.0 / 85.0]),
    ] {
        let actual = &result.streams[&stream.into()];
        for (component, flow) in ["methane", "ethane"].into_iter().zip(expected) {
            let molar_flow =
                actual.total_molar_flow_mol_s * actual.overall_mole_fractions[&component.into()];
            assert!((molar_flow - flow).abs() <= 1e-9, "{stream}/{component}");
        }
    }
    let recycle = result.recycle.as_ref().unwrap();
    assert!(
        recycle
            .component_scales_mol_s
            .values()
            .all(|scale| *scale == 5.0)
    );
    assert_eq!(recycle.residual_trace.len(), 16);
    let verification = recycle.residual_trace.last().unwrap();
    assert_eq!(verification.phase, EvaluationPhase::Verification);
    assert!(verification.max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE);
    assert!(verification.max_scaled <= SCALED_FLOW_TOLERANCE);
    for (component, expected) in [("methane", 11.0 / 5.0), ("ethane", 14.0 / 5.0)] {
        let component = component.into();
        assert!(
            (result.global_material.product_components_mol_s[&component] - expected).abs()
                <= ABSOLUTE_FLOW_TOLERANCE
        );
        assert!(
            result.global_material.component_residuals_mol_s[&component].abs()
                <= ABSOLUTE_FLOW_TOLERANCE
        );
    }
    let evidence = origin.execution().unwrap().clone();
    let plan = evidence.plan.as_ref().unwrap();
    assert_eq!(plan.method, rf_solver::mixed::RECYCLE_METHOD);
    assert_eq!(plan.acceptance_contract_version, 1);
    evidence
}

#[test]
fn mixed_recycle_save_reopens_formal_inputs_and_runs_only_on_explicit_request() {
    let dir = Directory::new();
    let (mut app, path) = source(&dir);
    let identity = prepare_object_identity_initialization(&app, &path).unwrap();
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::InitializeObjectIdentities(Box::new(identity)),
    )
    .unwrap();
    app.workspace
        .engineering
        .apply(vec![EngineeringCommand::CreateTask(Box::new(
            SeparatorSizingTask {
                id: "sizing-1".into(),
                revision: 0,
                tag: "V-101".into(),
                name: "Separator sizing".into(),
                method: EquipmentMethod {
                    id: "radishflow.separator.vertical-sb-working-volume".into(),
                    version: 1,
                },
                case: SeparatorCase {
                    id: "case-1".into(),
                    name: "Design case".into(),
                    purpose: "Save roundtrip".into(),
                    fluid_system: String::new(),
                    applicability_source_ids: vec![],
                    inputs: Default::default(),
                    applicability: Default::default(),
                },
                axial_sections: Default::default(),
            },
        ))])
        .unwrap();
    let watched = VariableId {
        document: app.workspace.document.metadata.document_id.clone(),
        object: ObjectId::Unit("heater-1".into()),
        section: VariableSection::Inputs,
        field: VariableField::OutletTemperature,
    };
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Add(watched))
        .unwrap();
    let mut units = app.workspace.project_presentation.display_units().clone();
    units
        .set_unit(QuantityKind::AbsolutePressure, MeasurementUnit::Kilopascal)
        .unwrap();
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(units.clone()));

    let mut config = selected_m2(500);
    let FlowPlanConfiguration::Mixed(options) = &mut config else {
        unreachable!()
    };
    options
        .recycle
        .as_mut()
        .unwrap()
        .tear
        .initial_component_flows_mol_s =
        Some([("methane".into(), 10.0), ("ethane".into(), 1.0)].into());
    let document = app.workspace.document.clone();
    select(&mut app, config.clone());
    assert_eq!(app.workspace.document, document);
    assert!(app.workspace.project_save_state().simulation_dirty);
    let provider = build_official_binary_hydrocarbon_in_memory_provider(
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    );
    let request =
        build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    solve_workspace_with_property_package(&mut app, &provider, &request).unwrap();
    let first = verified_result(&app);
    let pending =
        build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    let mut late = prepare_workspace_solve(&mut app, &pending)
        .unwrap()
        .execute_with_property_package(&provider);
    assert!(late.completion_result().is_ok());
    let mut draft = FlowPlanDraft::new(&app.workspace, config.clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650_000.0,
            MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft.clone());

    let before = app.workspace.clone();
    let original = fs::read(&path).unwrap();
    let review = prepare_mixed_recycle_save(&app, &path).unwrap();
    let Some(ProjectSaveCompatibility::FlowCapabilities {
        added_from_source, ..
    }) = review.compatibility()
    else {
        panic!("the new required capability within v4 must be confirmed");
    };
    assert!(
        added_from_source
            .iter()
            .any(|cap| cap.id == project_v4::MIXED_RECYCLE_CAPABILITY)
    );
    assert_eq!(review.path(), path);
    let declined = review.commit(&mut app, false).unwrap_err();
    assert_eq!(
        declined.context().diagnostic_code(),
        Some("workspace.project.save_confirmation_required")
    );
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read(&path).unwrap(), original);

    let outcome = prepare_mixed_recycle_save(&app, &path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    assert_eq!(outcome.action, StudioDocumentLifecycleAction::Save);
    assert!(!outcome.has_unsaved_changes);
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));

    let mut reopened = reopen(&path);
    assert_eq!(reopened.workspace.document, document);
    assert_eq!(
        reopened.workspace.engineering.inputs(),
        before.engineering.inputs()
    );
    assert_eq!(
        reopened.workspace.current_object_identities(),
        app.workspace.current_object_identities()
    );
    assert_eq!(
        reopened.workspace.variable_watch_list.entries(),
        before.variable_watch_list.entries()
    );
    assert_eq!(
        reopened.workspace.project_presentation.display_units(),
        &units
    );
    assert_eq!(reopened.workspace.flow_plan.current(), &config);
    assert_eq!(
        reopened
            .workspace
            .flow_plan
            .saved_capabilities()
            .get(project_v4::MIXED_RECYCLE_CAPABILITY),
        Some(&1)
    );
    assert!(reopened.workspace.flow_plan.draft().is_none());
    assert!(reopened.workspace.snapshot_history.is_empty());
    assert!(
        !reopened
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert!(matches!(
        complete_workspace_solve(&mut reopened, &mut late),
        FlowRunDisposition::Discarded(_)
    ));
    assert!(matches!(
        reopened.workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));

    let request =
        build_workspace_solve_request(&reopened, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    solve_workspace_with_property_package(&mut reopened, &provider, &request).unwrap();
    let second = verified_result(&reopened);
    assert_eq!(second.plan, first.plan);
    assert_ne!(second.run_id, first.run_id);
    let review = prepare_mixed_recycle_save(&reopened, &path).unwrap();
    assert!(review.compatibility().is_none());
    let saved = fs::read_to_string(&path).unwrap();
    review.commit(&mut reopened, false).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);

    let StudioProjectOpen::ReadOnly(old) = read_studio_project(&path).unwrap() else {
        panic!("default Studio reader must keep M2 read only");
    };
    assert_eq!(old.original_json(), saved);
    let mut old_writer = crate::qs4_test_support::case();
    let error = dispatch_document_lifecycle(
        &mut old_writer,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unsupported capabilities; overwriting prohibited")
    );
    assert_eq!(fs::read_to_string(path).unwrap(), saved);
}

#[test]
fn mixed_recycle_save_as_revalidates_source_and_target_and_can_retry_a_write_failure() {
    let dir = Directory::new();
    let (mut app, source) = source(&dir);
    let path = dir.0.join("copy.rfproj.json");
    let m2 = selected_m2(500);
    select(&mut app, m2.clone());
    let initial = prepare_mixed_recycle_save(&app, &path).unwrap();
    assert!(initial.compatibility().is_some());
    drop(initial);
    assert!(!path.exists());
    assert_eq!(fs::read_to_string(&source).unwrap(), M1);

    let stale = prepare_mixed_recycle_save(&app, &path).unwrap();
    select(&mut app, selected_m2(20));
    select(&mut app, m2);
    let before = app.workspace.clone();
    let error = stale.commit(&mut app, true).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("workspace.project.save_source_changed")
    );
    assert_eq!(app.workspace, before);
    assert!(!path.exists());

    let prepared = prepare_mixed_recycle_save(&app, &path).unwrap();
    let mut units = app.workspace.project_presentation.display_units().clone();
    units
        .set_unit(QuantityKind::AbsolutePressure, MeasurementUnit::Kilopascal)
        .unwrap();
    assert!(
        app.workspace
            .project_presentation
            .apply(ProjectPresentationCommand::Apply(units))
    );
    let changed = app.workspace.clone();
    let error = prepared.commit(&mut app, true).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("workspace.project.save_source_changed")
    );
    assert_eq!(app.workspace, changed);
    assert!(!path.exists());
    let before = changed;

    fs::write(&path, M1).unwrap();
    let review = prepare_mixed_recycle_save(&app, &path).unwrap();
    let new_target = format!("{M1}\n");
    fs::write(&path, &new_target).unwrap();
    let error = review.commit(&mut app, true).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), new_target);

    let parent = dir.0.join("blocked");
    let retry_path = parent.join("copy.rfproj.json");
    let prepared = prepare_mixed_recycle_save(&app, &retry_path).unwrap();
    fs::write(&parent, "not a directory").unwrap();
    assert!(prepared.commit(&mut app, true).is_err());
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&source).unwrap(), M1);
    assert_eq!(fs::read_to_string(&parent).unwrap(), "not a directory");
    fs::remove_file(parent).unwrap();
    let outcome = prepare_mixed_recycle_save(&app, &retry_path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    assert_eq!(outcome.action, StudioDocumentLifecycleAction::SaveAs);
    assert_eq!(outcome.path, retry_path);
    assert!(!outcome.has_unsaved_changes);
    assert_eq!(fs::read_to_string(source).unwrap(), M1);
    assert_eq!(
        reopen(&retry_path).workspace.flow_plan.current(),
        app.workspace.flow_plan.current()
    );
}

#[test]
fn mixed_recycle_preserves_a_missing_tear_reference_and_rejects_running_it() {
    let dir = Directory::new();
    let (mut app, path) = source(&dir);
    let mut plan = selected_m2(500);
    let FlowPlanConfiguration::Mixed(options) = &mut plan else {
        unreachable!()
    };
    options.recycle.as_mut().unwrap().tear.stream_id = "missing-tear".into();
    select(&mut app, plan.clone());
    prepare_mixed_recycle_save(&app, &path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    let mut reopened = reopen(&path);
    assert_eq!(reopened.workspace.flow_plan.current(), &plan);
    let request =
        build_workspace_solve_request(&reopened, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    let Err(error) = prepare_workspace_solve(&mut reopened, &request) else {
        panic!("the missing tear must be diagnosed before running");
    };
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.mixed.missing_tear")
    );
    assert!(reopened.workspace.snapshot_history.is_empty());
}
