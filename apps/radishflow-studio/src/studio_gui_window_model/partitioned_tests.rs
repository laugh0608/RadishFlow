use super::*;
use crate::{qs1_test_support::*, test_support::*};
#[test]
fn block_results_drive_review_inspector_and_guarded_text_output() {
    let mut app = recycle_case(0.6);
    crate::WorkspaceSolveService::new()
        .run_with_property_package(
            &mut app,
            &build_official_binary_hydrocarbon_in_memory_provider(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            ),
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        )
        .unwrap();
    let raw = rf_ui::latest_snapshot(&app.workspace).unwrap();
    let model = solve_snapshot_model_from_ui(raw, &rf_types::units::DisplayUnitSet::si());
    assert!(model.steps.is_empty());
    assert_eq!(model.partitioned.as_ref().unwrap().units.len(), 6);
    assert_eq!(model.review_summary.source_stream_results.len(), 2);
    assert_eq!(
        model.review_summary.terminal_stream_results[0].stream_id,
        "s-product"
    );
    let inspector = model.result_inspector_with_unit(Some("s-recycle"), None, Some("mixer-2"));
    assert_eq!(inspector.unit_options.len(), 6);
    let unit = inspector.selected_unit.unwrap();
    assert_eq!(unit.location, ResultExecutionLocation::Block(3));
    assert!(
        unit.consumed_stream_results
            .iter()
            .any(|s| s.stream_id == "s-recycle")
    );
    let text = model.light_text_export().unwrap();
    for expected in [
        "Dependency blocks",
        "independent verification: 1",
        "iterations: 48",
        "actual calls: local 102",
        "assumed methane:",
        "returned methane:",
        "radishflow.partitioned-sm/1",
        "configuration_sha256",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    assert!(
        model
            .result_guard
            .as_ref()
            .unwrap()
            .is_current(&app.workspace)
    );
    select(&mut app, rf_ui::flow_plan::FlowPlanConfiguration::Acyclic);
    assert!(
        !model
            .result_guard
            .as_ref()
            .unwrap()
            .is_current(&app.workspace)
    );
}
