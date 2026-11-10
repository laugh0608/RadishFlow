use super::*;
use crate::qs1_test_support::{recycle_case, select, write};
use rf_ui::{flow_plan::*, variable_browser::*};
#[test]
fn splitter_fraction_is_explicit_dimensionless_and_uses_normal_history() {
    let mut app = recycle_case(0.6);
    let created = crate::qs1_test_support::action(
        &mut app,
        FlowAction::CreateUnit(BuiltinUnitKind::Splitter),
    );
    let FlowActionEffect::Created(id) = created.effect else {
        panic!()
    };
    let row = VariableBrowser::from_workspace(&app.workspace)
        .variables(&ObjectId::Unit(id.clone()))
        .unwrap()
        .into_iter()
        .find(|r| r.id.field == VariableField::OutletASplitFraction)
        .unwrap();
    assert_eq!(row.value, None);
    assert_eq!(
        row.quantity,
        Some(rf_types::units::QuantityKind::DimensionlessRatio)
    );
    assert_eq!(
        app.workspace
            .numeric_field_presentation(&row.id)
            .unwrap()
            .text,
        ""
    );
    let before = app.workspace.document.revision;
    write(
        &mut app,
        ObjectId::Unit(id.clone()),
        VariableField::OutletASplitFraction,
        0.0,
    );
    assert_eq!(
        app.workspace.document.flowsheet.units[&id]
            .parameters
            .outlet_a_split_fraction,
        Some(0.0)
    );
    app.undo_document_command(std::time::UNIX_EPOCH).unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.units[&id]
            .parameters
            .outlet_a_split_fraction,
        None
    );
    assert!(app.workspace.document.revision > before);
    let request = FlowActionRequest {
        expected: FlowEditContext::capture(&app.workspace),
        action: FlowAction::Write {
            variable: row.id,
            value: VariableValue::Number(1.01),
        },
    };
    let old = app.clone();
    assert!(
        execute_flow_action(
            &mut app,
            FlowEditAccess::Editable,
            &request,
            std::time::UNIX_EPOCH
        )
        .is_err()
    );
    assert_eq!(app, old);
}
#[test]
fn dangling_plan_identities_are_reserved_after_reopen_and_binding_history_is_exact() {
    let mut app = recycle_case(0.6);
    let mut plan = app.workspace.flow_plan.current().clone();
    let FlowPlanConfiguration::Partitioned(o) = &mut plan else {
        panic!()
    };
    o.tears[0].source.unit_id = "splitter-2".into();
    o.tears[0].stream_id = "new-reserved-stream".into();
    select(&mut app, plan);
    let path = std::env::temp_dir().join(format!(
        "rf-plan-reserved-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    crate::dispatch_document_lifecycle(
        &mut app,
        crate::StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    let mut app = crate::load_project_app_state(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let created = crate::qs1_test_support::action(
        &mut app,
        FlowAction::CreateUnit(BuiltinUnitKind::Splitter),
    );
    assert_eq!(
        created.effect,
        FlowActionEffect::Created("splitter-3".into())
    );
    assert!(
        app.workspace
            .stream_identity_reserved(&"new-reserved-stream".into())
    );
    let mut app = recycle_case(0.6);
    let plan = app.workspace.flow_plan.current().clone();
    crate::qs1_test_support::action(
        &mut app,
        FlowAction::Disconnect(MaterialPortTarget {
            unit_id: "mixer-2".into(),
            port: "inlet_b".into(),
        }),
    );
    assert!(
        app.workspace
            .flow_plan
            .current()
            .validate_for(&app.workspace)
            .is_err()
    );
    assert_eq!(&plan, app.workspace.flow_plan.current());
    app.undo_document_command(std::time::UNIX_EPOCH).unwrap();
    assert!(
        app.workspace
            .flow_plan
            .current()
            .validate_for(&app.workspace)
            .is_ok()
    );
}

#[test]
fn automatic_stream_creation_skips_plan_reserved_identity_and_rejects_old_preview() {
    let mut app = AppState::new(rf_ui::FlowsheetDocument::new(
        rf_model::Flowsheet::new("reserved"),
        rf_ui::DocumentMetadata::new("reserved", "Reserved", std::time::UNIX_EPOCH),
    ));
    crate::qs1_test_support::action(&mut app, FlowAction::CreateUnit(BuiltinUnitKind::Feed));
    let old = crate::studio_local_rules::generate_local_canvas_suggestions(&app).remove(0);
    let Some(rf_ui::CanvasSuggestionAcceptance::MaterialConnection(connection)) = old.acceptance
    else {
        panic!()
    };
    let rf_ui::CanvasSuggestedStreamBinding::Create { stream } = &connection.stream else {
        panic!()
    };
    let reserved = stream.id.clone();
    let options = rf_solver::partitioned::RecycleSolveOptions::new(
        rf_solver::partitioned::FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650000.0,
        },
        vec![rf_solver::partitioned::TearSpecification {
            stream_id: reserved.clone(),
            source: rf_types::DiagnosticPortTarget::new("feed-1", "outlet"),
            sink: rf_types::DiagnosticPortTarget::new("missing", "inlet_b"),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        }],
    );
    select(&mut app, FlowPlanConfiguration::Partitioned(options));
    let revision = app.workspace.document.revision;
    assert!(
        app.commit_material_connection(&connection, std::time::UNIX_EPOCH)
            .is_err()
    );
    assert_eq!(app.workspace.document.revision, revision);
    let fresh = crate::studio_local_rules::generate_local_canvas_suggestions(&app).remove(0);
    let Some(rf_ui::CanvasSuggestionAcceptance::MaterialConnection(connection)) = fresh.acceptance
    else {
        panic!()
    };
    let rf_ui::CanvasSuggestedStreamBinding::Create { stream } = &connection.stream else {
        panic!()
    };
    assert_ne!(stream.id, reserved);
    assert_eq!(stream.id.as_str(), "stream-feed-1-outlet-2");
    app.commit_material_connection(&connection, std::time::UNIX_EPOCH)
        .unwrap();
    assert!(
        !app.workspace
            .document
            .flowsheet
            .streams
            .contains_key(&reserved)
    );
}
