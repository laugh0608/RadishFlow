use rf_store::StoredProjectFile;
use rf_thermo::PlaceholderThermoProvider;

#[doc(hidden)]
pub fn build_binary_demo_provider() -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_binary_demo_provider()
}

#[doc(hidden)]
pub fn build_binary_hydrocarbon_lite_provider() -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_binary_hydrocarbon_lite_provider()
}

#[doc(hidden)]
pub fn build_synthetic_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
    crate::studio_gui_window_model::test_support::build_synthetic_provider(k_values, pressure_pa)
}

#[doc(hidden)]
pub fn apply_stream_state_and_composition(
    project: &mut StoredProjectFile,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
    temperature_k: f64,
    pressure_pa: f64,
) {
    crate::studio_gui_window_model::test_support::apply_stream_state_and_composition(
        project,
        stream_id,
        overall_mole_fractions,
        temperature_k,
        pressure_pa,
    )
}

#[doc(hidden)]
pub fn solve_snapshot_model_from_project_with_provider_and_edit<F>(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
    edit_project: F,
) -> crate::StudioGuiWindowSolveSnapshotModel
where
    F: FnOnce(&mut StoredProjectFile),
{
    crate::studio_gui_window_model::test_support::solve_snapshot_model_from_project_with_provider_and_edit(
        project_json,
        provider,
        edit_project,
    )
}

#[doc(hidden)]
pub fn stream_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::stream_target_detail_model(
        snapshot, stream_id, title,
    )
}

#[doc(hidden)]
pub fn unit_target_detail_model(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    unit_id: &str,
    title: &str,
) -> crate::StudioGuiWindowInspectorTargetDetailModel {
    crate::studio_gui_window_model::test_support::unit_target_detail_model(snapshot, unit_id, title)
}

#[doc(hidden)]
pub fn stream_target_detail_snapshot(
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiInspectorTargetDetailSnapshot {
    crate::studio_gui_window_model::test_support::stream_target_detail_snapshot(stream_id, title)
}
