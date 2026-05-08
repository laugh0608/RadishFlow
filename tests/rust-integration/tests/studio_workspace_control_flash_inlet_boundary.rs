use std::fs;
use std::path::Path;

use radishflow_studio::{
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppFacade,
    StudioAppResultDispatch, StudioWorkspaceRunOutcome,
    dispatch_run_panel_primary_action_with_auth_cache,
};
use rf_rust_integration::{
    NearBoundaryCaseKind, NearBoundaryStreamWindowCase, SYNTHETIC_LIQUID_ONLY_PACKAGE_ID,
    SYNTHETIC_VAPOR_ONLY_PACKAGE_ID, assert_close, sample_auth_cache_index,
    synthetic_single_phase_near_boundary_stream_window_cases, timestamp, unique_temp_path,
    write_synthetic_liquid_only_cached_package, write_synthetic_vapor_only_cached_package,
};
use rf_store::parse_project_file_json;
use rf_types::{ComponentId, StreamId, UnitId};
use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument};

fn app_state_from_project(
    project_json: &str,
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
) -> AppState {
    let project = parse_project_file_json(project_json).expect("expected project parse");
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(document_id, title, timestamp(created_at_seconds)),
    ))
}

fn find_snapshot_stream<'a>(
    snapshot: &'a rf_ui::SolveSnapshot,
    stream_id: &str,
) -> &'a rf_ui::StreamStateSnapshot {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == StreamId::new(stream_id))
        .expect("expected snapshot stream")
}

fn apply_synthetic_demo_composition(
    app_state: &mut AppState,
    stream_id: &str,
    overall_mole_fractions: [f64; 2],
) {
    let stream = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.overall_mole_fractions.clear();
    stream
        .overall_mole_fractions
        .insert(ComponentId::new("component-a"), overall_mole_fractions[0]);
    stream
        .overall_mole_fractions
        .insert(ComponentId::new("component-b"), overall_mole_fractions[1]);
}

fn assert_near_boundary_window_matches_case(
    stream: &rf_ui::StreamStateSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let window = stream
        .bubble_dew_window
        .as_ref()
        .expect("expected bubble/dew window");

    assert_close(stream.temperature_k, case.temperature_k, 1e-12);
    assert_close(stream.pressure_pa, case.pressure_pa, 1e-9);
    assert_eq!(
        window.phase_region, case.expected_phase_region,
        "{}",
        case.label
    );
    assert_close(
        window.bubble_pressure_pa,
        case.expected_bubble_pressure_pa,
        1e-6,
    );
    assert_close(window.dew_pressure_pa, case.expected_dew_pressure_pa, 1e-6);
    assert_close(
        window.bubble_temperature_k,
        case.expected_bubble_temperature_k,
        1e-4,
    );
    assert_close(
        window.dew_temperature_k,
        case.expected_dew_temperature_k,
        1e-4,
    );
}

fn assert_flash_consumed_stream_matches_inlet(
    snapshot: &rf_ui::SolveSnapshot,
    inlet_stream: &rf_ui::StreamStateSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let flash_step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == UnitId::new("flash-1"))
        .expect("expected flash step");
    assert_eq!(flash_step.consumed_streams.len(), 1, "{}", case.label);
    assert_eq!(
        &flash_step.consumed_streams[0], inlet_stream,
        "{}",
        case.label
    );
}

fn write_synthetic_cached_package_for_case(
    cache_root: &Path,
    auth_cache_index: &mut rf_store::StoredAuthCacheIndex,
    case: &NearBoundaryStreamWindowCase,
) {
    match case.package_id {
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID => {
            write_synthetic_liquid_only_cached_package(cache_root, auth_cache_index)
        }
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => {
            write_synthetic_vapor_only_cached_package(cache_root, auth_cache_index)
        }
        _ => panic!("unexpected synthetic package id `{}`", case.package_id),
    }
}

fn app_state_for_synthetic_cooler_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-cooler-flash.rfproj.json"),
        document_id,
        title,
        created_at_seconds,
    );
    apply_synthetic_demo_composition(&mut app_state, "stream-feed", case.overall_mole_fractions);
    let cooled = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-cooled".into())
        .expect("expected cooled stream");
    cooled.temperature_k = case.temperature_k;
    cooled.pressure_pa = case.pressure_pa;
    app_state
}

fn app_state_for_synthetic_valve_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-valve-flash.rfproj.json"),
        document_id,
        title,
        created_at_seconds,
    );
    apply_synthetic_demo_composition(&mut app_state, "stream-feed", case.overall_mole_fractions);
    let feed = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .expect("expected feed stream");
    feed.temperature_k = case.temperature_k;
    feed.pressure_pa = case.pressure_pa.max(feed.pressure_pa) + 20_000.0;
    app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-throttled".into())
        .expect("expected throttled stream")
        .pressure_pa = case.pressure_pa;
    app_state
}

fn assert_run_panel_synthetic_near_boundary_cases_across_chain<F>(
    case_kind: NearBoundaryCaseKind,
    temp_prefix: &str,
    flash_inlet_stream_id: &str,
    build_app_state: F,
) where
    F: Fn(usize, &NearBoundaryStreamWindowCase) -> AppState,
{
    let facade = StudioAppFacade::new();

    for (index, case) in synthetic_single_phase_near_boundary_stream_window_cases()
        .into_iter()
        .filter(|case| case.kind == case_kind)
        .enumerate()
    {
        let cache_root = unique_temp_path(&format!("{temp_prefix}-{index}"));
        let mut auth_cache_index = sample_auth_cache_index(&[]);
        write_synthetic_cached_package_for_case(&cache_root, &mut auth_cache_index, &case);
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let mut app_state = build_app_state(index, &case);

        let outcome =
            dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
                .expect("expected primary action dispatch");

        match outcome.dispatch {
            RunPanelWidgetDispatchOutcome::Executed(outcome) => match outcome.dispatch {
                StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                    assert!(matches!(
                        dispatch.outcome,
                        StudioWorkspaceRunOutcome::Started(_)
                    ));
                }
                _ => panic!("expected workspace run dispatch"),
            },
            _ => panic!("expected executed run panel outcome"),
        }

        let snapshot = app_state
            .workspace
            .snapshot_history
            .back()
            .expect("expected stored snapshot");
        let inlet_stream = find_snapshot_stream(snapshot, flash_inlet_stream_id);
        assert_near_boundary_window_matches_case(inlet_stream, &case);
        assert_flash_consumed_stream_matches_inlet(snapshot, inlet_stream, &case);

        fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }
}

#[test]
fn run_panel_primary_action_preserves_synthetic_single_phase_pressure_near_boundary_windows_across_cooler_flash_inlet_end_to_end()
 {
    assert_run_panel_synthetic_near_boundary_cases_across_chain(
        NearBoundaryCaseKind::Pressure,
        "integration-run-panel-synthetic-cooler-pressure",
        "stream-cooled",
        |index, case| {
            app_state_for_synthetic_cooler_boundary_case(
                &format!("doc-control-synthetic-cooler-pressure-{index}"),
                &case.label,
                260 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn run_panel_primary_action_preserves_synthetic_single_phase_temperature_near_boundary_windows_across_cooler_flash_inlet_end_to_end()
 {
    assert_run_panel_synthetic_near_boundary_cases_across_chain(
        NearBoundaryCaseKind::Temperature,
        "integration-run-panel-synthetic-cooler-temperature",
        "stream-cooled",
        |index, case| {
            app_state_for_synthetic_cooler_boundary_case(
                &format!("doc-control-synthetic-cooler-temperature-{index}"),
                &case.label,
                300 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn run_panel_primary_action_preserves_synthetic_single_phase_pressure_near_boundary_windows_across_valve_flash_inlet_end_to_end()
 {
    assert_run_panel_synthetic_near_boundary_cases_across_chain(
        NearBoundaryCaseKind::Pressure,
        "integration-run-panel-synthetic-valve-pressure",
        "stream-throttled",
        |index, case| {
            app_state_for_synthetic_valve_boundary_case(
                &format!("doc-control-synthetic-valve-pressure-{index}"),
                &case.label,
                340 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn run_panel_primary_action_preserves_synthetic_single_phase_temperature_near_boundary_windows_across_valve_flash_inlet_end_to_end()
 {
    assert_run_panel_synthetic_near_boundary_cases_across_chain(
        NearBoundaryCaseKind::Temperature,
        "integration-run-panel-synthetic-valve-temperature",
        "stream-throttled",
        |index, case| {
            app_state_for_synthetic_valve_boundary_case(
                &format!("doc-control-synthetic-valve-temperature-{index}"),
                &case.label,
                380 + index as u64,
                case,
            )
        },
    );
}
