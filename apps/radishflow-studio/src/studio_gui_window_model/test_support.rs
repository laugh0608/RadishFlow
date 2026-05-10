use super::{
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowSolveSnapshotModel,
    inspector_target_detail_model_from_snapshot, solve_snapshot_model_from_ui,
};
use rf_flash::PlaceholderTpFlashSolver;
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolverServices};
use rf_store::{StoredProjectFile, parse_project_file_json};
use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, StreamId, UnitId};

fn build_demo_antoine_coefficients(k_value: f64, pressure_pa: f64) -> AntoineCoefficients {
    const TEST_ANTOINE_BOUNDARY_SLOPE: f64 = 250.0;
    const TEST_REFERENCE_TEMPERATURE_K: f64 = 300.0;

    AntoineCoefficients::new(
        ((k_value * pressure_pa) / 1_000.0).ln()
            + TEST_ANTOINE_BOUNDARY_SLOPE / TEST_REFERENCE_TEMPERATURE_K,
        TEST_ANTOINE_BOUNDARY_SLOPE,
        0.0,
    )
}

pub(crate) fn build_synthetic_provider(
    k_values: [f64; 2],
    pressure_pa: f64,
) -> PlaceholderThermoProvider {
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(k_values[0], pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(k_values[1], pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

pub(crate) fn apply_stream_state_and_composition(
    project: &mut StoredProjectFile,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
    temperature_k: f64,
    pressure_pa: f64,
) {
    let stream = project
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.temperature_k = temperature_k;
    stream.pressure_pa = pressure_pa;
    stream.overall_mole_fractions.clear();
    stream
        .overall_mole_fractions
        .insert(ComponentId::new("component-a"), overall_mole_fractions[0]);
    stream
        .overall_mole_fractions
        .insert(ComponentId::new("component-b"), overall_mole_fractions[1]);
}

pub(crate) fn solve_snapshot_model_from_project_with_provider_and_edit<F>(
    project_json: &str,
    provider: &PlaceholderThermoProvider,
    edit_project: F,
) -> StudioGuiWindowSolveSnapshotModel
where
    F: FnOnce(&mut StoredProjectFile),
{
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: provider,
        flash_solver: &flash_solver,
    };
    let mut project =
        parse_project_file_json(project_json).expect("expected example project parse");
    edit_project(&mut project);
    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");
    let ui_snapshot =
        rf_ui::SolveSnapshot::from_solver_snapshot("snapshot-window-model", 0, 1, &snapshot);
    solve_snapshot_model_from_ui(&ui_snapshot)
}

pub(crate) fn stream_target_detail_snapshot(
    stream_id: &str,
    title: &str,
) -> crate::StudioGuiInspectorTargetDetailSnapshot {
    crate::StudioGuiInspectorTargetDetailSnapshot {
        target: rf_ui::InspectorTarget::Stream(StreamId::new(stream_id)),
        title: title.to_string(),
        summary_rows: Vec::new(),
        property_fields: Vec::new(),
        property_notices: Vec::new(),
        property_composition_summary: None,
        property_batch_commit_command_id: None,
        property_batch_discard_command_id: None,
        property_composition_normalize_command_id: None,
        property_composition_component_actions: Vec::new(),
        unit_ports: Vec::new(),
    }
}

pub(crate) fn unit_target_detail_snapshot(
    unit_id: &str,
    title: &str,
) -> crate::StudioGuiInspectorTargetDetailSnapshot {
    crate::StudioGuiInspectorTargetDetailSnapshot {
        target: rf_ui::InspectorTarget::Unit(UnitId::new(unit_id)),
        title: title.to_string(),
        summary_rows: Vec::new(),
        property_fields: Vec::new(),
        property_notices: Vec::new(),
        property_composition_summary: None,
        property_batch_commit_command_id: None,
        property_batch_discard_command_id: None,
        property_composition_normalize_command_id: None,
        property_composition_component_actions: Vec::new(),
        unit_ports: Vec::new(),
    }
}

pub(crate) fn stream_target_detail_model(
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) -> StudioGuiWindowInspectorTargetDetailModel {
    inspector_target_detail_model_from_snapshot(
        &stream_target_detail_snapshot(stream_id, title),
        Some(snapshot),
        None,
    )
}

pub(crate) fn unit_target_detail_model(
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    unit_id: &str,
    title: &str,
) -> StudioGuiWindowInspectorTargetDetailModel {
    inspector_target_detail_model_from_snapshot(
        &unit_target_detail_snapshot(unit_id, title),
        Some(snapshot),
        None,
    )
}
