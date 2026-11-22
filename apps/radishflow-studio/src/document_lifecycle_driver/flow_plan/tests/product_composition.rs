use super::*;
use crate::{
    build_workspace_solve_request,
    qs1_test_support::select,
    solve_workspace_with_property_package,
    test_support::{
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        build_official_binary_hydrocarbon_in_memory_provider,
    },
};
use rf_solver::equation_oriented::{
    MaterialPort, PortBinding, ProductMoleFractionTarget, ProductTarget,
};
use rf_ui::{FlowResultRecord, result_query::FlowResultView};

const TOTAL: &str = include_str!(
    "../../../../../../examples/flowsheets/fixed-tp-material-eo-binary-hydrocarbon.rfproj.json"
);
const COMPOSITION: &str = include_str!(
    "../../../../../../examples/flowsheets/fixed-tp-product-composition-binary-hydrocarbon.rfproj.json"
);

fn composition_plan(app: &AppState) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::EquationOriented(mut o) = app.workspace.flow_plan.current().clone()
    else {
        panic!("expected EO")
    };
    o.target = Some(ProductTarget::MoleFraction(Box::new(
        ProductMoleFractionTarget {
            source: PortBinding {
                unit: "splitter-2".into(),
                port: MaterialPort::OutletB,
                stream: "s-pb".into(),
            },
            component: "methane".into(),
            value_mol_mol: 0.35,
        },
    )));
    FlowPlanConfiguration::EquationOriented(o)
}
fn reopen(path: &Path) -> AppState {
    let StudioProjectOpen::Editable(app) =
        read_studio_project_with_support(path, project_v4::FlowPlanSupport::ProductComposition)
            .unwrap()
    else {
        panic!("composition-aware reader must return an editable project");
    };
    *app
}

#[test]
fn same_cross_recycle_project_saves_input_and_plan_then_reopens_missing() {
    let dir = Directory::new();
    let path = dir.0.join("blend.json");
    fs::write(&path, TOTAL).unwrap();
    let mut app = reopen(&path);
    let original = app.workspace.document.clone();
    let total = app.workspace.flow_plan.current().clone();
    let provider = build_official_binary_hydrocarbon_in_memory_provider(
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    );
    let request =
        build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    solve_workspace_with_property_package(&mut app, &provider, &request).unwrap();
    let FlowResultView::Current {
        snapshot: FlowResultRecord::EquationOriented(s),
        ..
    } = app.workspace.flow_result()
    else {
        panic!("expected verified total-flow inverse")
    };
    assert!(
        (s.result.evidence.solved_parameter.as_ref().unwrap().value - 10.0 / 19.0).abs() < 1e-12
    );

    let composition = composition_plan(&app);
    select(&mut app, composition.clone());
    let mut draft = FlowPlanDraft::new(&app.workspace, total.clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650000.0,
            rf_types::units::MeasurementUnit::Pascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft.clone());
    let review = prepare_product_composition_save(&app, &path).unwrap();
    let Some(ProjectSaveCompatibility::FlowCapabilities {
        added_from_source, ..
    }) = review.compatibility()
    else {
        panic!("EO v1 -> v2 must request capability confirmation")
    };
    assert!(
        added_from_source
            .iter()
            .any(|c| c.id == project_v4::EQUATION_ORIENTED_CAPABILITY && c.version == 2)
    );
    let before = app.workspace.clone();
    assert!(review.commit(&mut app, false).is_err());
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), TOTAL);
    prepare_product_composition_save(&app, &path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));
    let saved = fs::read_to_string(&path).unwrap();
    assert_eq!(saved, COMPOSITION.trim_end_matches('\n'));

    let mut restored = reopen(&path);
    assert_eq!(restored.workspace.document, original);
    assert_eq!(restored.workspace.flow_plan.current(), &composition);
    assert_eq!(restored.workspace.flow_plan.saved(), &composition);
    assert!(restored.workspace.flow_plan.draft().is_none());
    assert!(
        !restored
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert!(matches!(
        restored.workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    assert!(
        prepare_product_composition_save(&restored, &path)
            .unwrap()
            .compatibility()
            .is_none()
    );

    select(&mut restored, total);
    assert!(
        dispatch_document_lifecycle(
            &mut restored,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    let copy = dir.0.join("total-copy.json");
    dispatch_document_lifecycle(
        &mut restored,
        StudioDocumentLifecycleCommand::SaveAs { path: copy.clone() },
    )
    .unwrap();
    assert_eq!(
        load_project_app_state(&copy)
            .unwrap()
            .workspace
            .flow_plan
            .current(),
        restored.workspace.flow_plan.current()
    );
}

#[test]
fn composition_save_as_rechecks_plan_generation_and_destination_bytes() {
    let dir = Directory::new();
    let source = dir.0.join("source.json");
    let target = dir.0.join("target.json");
    fs::write(&source, TOTAL).unwrap();
    let mut app = reopen(&source);
    let total = app.workspace.flow_plan.current().clone();
    let composition = composition_plan(&app);
    select(&mut app, composition.clone());
    let stale = prepare_product_composition_save(&app, &target).unwrap();
    select(&mut app, total);
    select(&mut app, composition);
    assert_eq!(
        stale
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("workspace.project.save_source_changed")
    );
    assert!(!target.exists());

    let changed = prepare_product_composition_save(&app, &target).unwrap();
    let before = app.workspace.clone();
    fs::write(&target, TOTAL).unwrap();
    assert_eq!(
        changed
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&target).unwrap(), TOTAL);
    prepare_product_composition_save(&app, &target)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    assert_eq!(fs::read_to_string(source).unwrap(), TOTAL);
}
