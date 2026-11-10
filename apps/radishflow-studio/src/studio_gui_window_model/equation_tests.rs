use super::*;
use crate::test_support::*;
#[test]
fn equation_current_inspector_export_and_display_units_keep_true_evidence() {
    let mut app = crate::qs3_test_support::case(true);
    crate::WorkspaceSolveService::new()
        .run_with_property_package(
            &mut app,
            &build_official_binary_hydrocarbon_in_memory_provider(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            ),
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        )
        .unwrap();
    let mut units = rf_types::units::DisplayUnitSet::si();
    units
        .set_unit(
            rf_types::units::QuantityKind::MolarFlow,
            rf_types::units::MeasurementUnit::KilomolePerHour,
        )
        .unwrap();
    let model =
        solve_snapshot_model_from_ui(rf_ui::latest_snapshot(&app.workspace).unwrap(), &units);
    assert!(model.steps.is_empty());
    assert!(model.partitioned.is_none());
    assert!(model.simultaneous.is_none());
    assert_eq!(model.block_count(), None);
    assert_eq!(model.final_unit_results().unwrap().len(), 6);
    let inspector = model.result_inspector_with_unit(Some("s-pa"), None, Some("splitter-1"));
    assert_eq!(
        inspector.selected_unit.unwrap().location,
        ResultExecutionLocation::EquationVerification
    );
    let text = model.light_text_export().unwrap();
    for s in [
        "Current EO condition",
        "stored",
        "solved",
        "Full Jacobian: n=29",
        "Conditional material system: n=28",
        "kmol/h",
        "core_backend",
        "Global total residual",
        "Consumed",
        "Produced",
    ] {
        assert!(text.contains(s), "{s}");
    }
    let guard = model.result_guard.unwrap();
    crate::qs1_test_support::select(
        &mut app,
        crate::qs3_test_support::explicit(&crate::qs3_test_support::case(true), true),
    );
    assert!(!guard.is_current(&app.workspace));
}
