use super::*;
use crate::{qs2_test_support::*, test_support::*};

#[test]
fn modular_current_drives_inspector_block_summary_navigation_and_lossless_evidence_source() {
    let mut app = cross_case(0.6, 0.25);
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
    assert!(model.partitioned.is_none());
    assert_eq!(model.block_count(), Some(3));
    assert_eq!(model.final_unit_results().unwrap().len(), 6);
    assert_eq!(model.review_summary.source_stream_results.len(), 2);
    assert_eq!(model.review_summary.terminal_stream_results.len(), 2);
    let inspector = model.result_inspector_with_unit(Some("s-ma"), None, Some("mixer-1"));
    assert_eq!(inspector.unit_options.len(), 6);
    let unit = inspector.selected_unit.unwrap();
    assert_eq!(unit.location, ResultExecutionLocation::Block(2));
    assert!(
        unit.consumed_stream_results
            .iter()
            .any(|s| s.stream_id == "s-rba")
    );
    let text = model.light_text_export().unwrap();
    for needle in [
        "Simultaneous modular result",
        "Assumed:",
        "Produced:",
        "Variable scale:",
        "residual scale:",
        "rcond_indicator:",
        "backward_error:",
        "Anchor: 1; responses: 4; LU factorizations: 1; triangular solves: 9; independent verification: 1",
        "local calls: 10",
        "configuration_sha256",
    ] {
        assert!(text.contains(needle), "missing {needle}");
    }
    assert!(!text.contains("iterations: 48"));
    let guard = model.result_guard.unwrap();
    crate::qs1_test_support::select(&mut app, explicit_plan(&cross_case(0.6, 0.25)));
    assert!(!guard.is_current(&app.workspace));
}

#[test]
fn modular_diagnostic_text_keeps_missing_and_all_nonfinite_values_distinct() {
    use crate::modular_evidence::diagnostic_number;
    let values = [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
        Some(0.0),
    ];
    let rendered: std::collections::BTreeSet<_> =
        values.into_iter().map(diagnostic_number).collect();
    assert_eq!(rendered.len(), 5);
    assert!(rendered.iter().any(|s| s.contains("not computed")));
    assert!(rendered.iter().any(|s| s.contains("NaN")));
}
