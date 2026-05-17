use super::*;

#[derive(Clone, Copy)]
struct SyntheticNearBoundaryCase {
    label: &'static str,
    k_values: [f64; 2],
    temperature_k: f64,
    phase_region: PhaseEquilibriumRegion,
    flowing_outlet_stream_id: &'static str,
    zero_outlet_stream_id: &'static str,
}

#[derive(Clone, Copy)]
struct SyntheticNearBoundaryScenario {
    label: &'static str,
    project_json: &'static str,
    flash_inlet_stream_id: &'static str,
    seeded_stream_ids: &'static [&'static str],
}

fn synthetic_near_boundary_cases() -> Vec<SyntheticNearBoundaryCase> {
    const REFERENCE_TEMPERATURE_K: f64 = 300.0;
    const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
    const BOUNDARY_DELTA_K: f64 = 0.001;
    const OVERALL_MOLE_FRACTIONS: [f64; 2] = [0.25, 0.75];

    let liquid_only_k_values = [0.8, 0.6];
    let liquid_only_provider =
        build_synthetic_provider(liquid_only_k_values, REFERENCE_PRESSURE_PA);
    let liquid_only_window = estimate_bubble_dew_window(
        &liquid_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        OVERALL_MOLE_FRACTIONS.to_vec(),
    )
    .expect("expected liquid-only synthetic bubble/dew window");

    let vapor_only_k_values = [1.8, 1.3];
    let vapor_only_provider = build_synthetic_provider(vapor_only_k_values, REFERENCE_PRESSURE_PA);
    let vapor_only_window = estimate_bubble_dew_window(
        &vapor_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        OVERALL_MOLE_FRACTIONS.to_vec(),
    )
    .expect("expected vapor-only synthetic bubble/dew window");

    vec![
        SyntheticNearBoundaryCase {
            label: "synthetic liquid-only bubble-temperature - 0.001 K",
            k_values: liquid_only_k_values,
            temperature_k: liquid_only_window.bubble_temperature_k - BOUNDARY_DELTA_K,
            phase_region: PhaseEquilibriumRegion::LiquidOnly,
            flowing_outlet_stream_id: "stream-liquid",
            zero_outlet_stream_id: "stream-vapor",
        },
        SyntheticNearBoundaryCase {
            label: "synthetic vapor-only dew-temperature + 0.001 K",
            k_values: vapor_only_k_values,
            temperature_k: vapor_only_window.dew_temperature_k + BOUNDARY_DELTA_K,
            phase_region: PhaseEquilibriumRegion::VaporOnly,
            flowing_outlet_stream_id: "stream-vapor",
            zero_outlet_stream_id: "stream-liquid",
        },
    ]
}

fn synthetic_near_boundary_scenarios() -> [SyntheticNearBoundaryScenario; 4] {
    [
        SyntheticNearBoundaryScenario {
            label: "heater",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"
            ),
            flash_inlet_stream_id: "stream-heated",
            seeded_stream_ids: &["stream-feed", "stream-heated"],
        },
        SyntheticNearBoundaryScenario {
            label: "cooler",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-synthetic-demo.rfproj.json"
            ),
            flash_inlet_stream_id: "stream-cooled",
            seeded_stream_ids: &["stream-feed", "stream-cooled"],
        },
        SyntheticNearBoundaryScenario {
            label: "valve",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-synthetic-demo.rfproj.json"
            ),
            flash_inlet_stream_id: "stream-throttled",
            seeded_stream_ids: &["stream-feed", "stream-throttled"],
        },
        SyntheticNearBoundaryScenario {
            label: "mixer",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
            ),
            flash_inlet_stream_id: "stream-mix-out",
            seeded_stream_ids: &["stream-feed-a", "stream-feed-b"],
        },
    ]
}

fn solve_synthetic_near_boundary_snapshot(
    scenario: &SyntheticNearBoundaryScenario,
    case: &SyntheticNearBoundaryCase,
) -> crate::StudioGuiWindowSolveSnapshotModel {
    const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
    const OVERALL_MOLE_FRACTIONS: [f64; 2] = [0.25, 0.75];

    let provider = build_synthetic_provider(case.k_values, REFERENCE_PRESSURE_PA);
    solve_snapshot_model_from_project_with_provider_and_edit(
        scenario.project_json,
        &provider,
        |project| {
            for stream_id in scenario.seeded_stream_ids {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    OVERALL_MOLE_FRACTIONS,
                    case.temperature_k,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    )
}

fn phase_region_label(region: PhaseEquilibriumRegion) -> &'static str {
    match region {
        PhaseEquilibriumRegion::LiquidOnly => "liquid_only",
        PhaseEquilibriumRegion::TwoPhase => "two_phase",
        PhaseEquilibriumRegion::VaporOnly => "vapor_only",
    }
}

#[test]
fn studio_gui_window_model_surfaces_synthetic_single_phase_flash_focus_actions() {
    for scenario in synthetic_near_boundary_scenarios() {
        for case in synthetic_near_boundary_cases() {
            let snapshot = solve_synthetic_near_boundary_snapshot(&scenario, &case);

            let flash_unit_focus_command = "inspector.focus_unit:flash-1";
            let flash_inlet_focus_command =
                format!("inspector.focus_stream:{}", scenario.flash_inlet_stream_id);
            let flowing_outlet_focus_command =
                format!("inspector.focus_stream:{}", case.flowing_outlet_stream_id);
            let zero_outlet_focus_command =
                format!("inspector.focus_stream:{}", case.zero_outlet_stream_id);

            for command_id in [
                flash_unit_focus_command,
                &flash_inlet_focus_command,
                &flowing_outlet_focus_command,
                &zero_outlet_focus_command,
            ] {
                assert!(
                    crate::inspector_target_from_command_id(command_id).is_some(),
                    "{} {} expected inspector focus command, got {command_id}",
                    scenario.label,
                    case.label
                );
            }

            let flowing_outlet =
                find_window_snapshot_stream(&snapshot, case.flowing_outlet_stream_id);
            let flowing_window = flowing_outlet
                .bubble_dew_window
                .as_ref()
                .expect("expected flowing outlet boundary window");
            assert_eq!(
                flowing_window.phase_region,
                phase_region_label(case.phase_region)
            );
            let zero_outlet = find_window_snapshot_stream(&snapshot, case.zero_outlet_stream_id);
            assert!(zero_outlet.total_molar_flow_mol_s.abs() < 1e-12);
            assert!(
                zero_outlet.bubble_dew_window.is_none(),
                "{} {} expected zero outlet to avoid a duplicated boundary window",
                scenario.label,
                case.label
            );

            let inlet_inspector = snapshot.result_inspector(Some(scenario.flash_inlet_stream_id));
            assert!(inlet_inspector.diagnostic_actions.iter().any(|action| {
                action.source_label == "Selected stream"
                    && action.action.command_id == flash_inlet_focus_command
            }));
            assert!(inlet_inspector.diagnostic_actions.iter().any(|action| {
                action.source_label == "Solve step"
                    && action.action.command_id == flash_unit_focus_command
            }));
            let flash_related_step = inlet_inspector
                .related_steps
                .iter()
                .find(|step| step.unit_id == "flash-1")
                .expect("expected downstream flash step for synthetic near-boundary inlet");
            assert_eq!(
                flash_related_step.unit_action.command_id,
                flash_unit_focus_command
            );
            assert!(
                flash_related_step
                    .consumed_stream_actions
                    .iter()
                    .any(|action| action.command_id == flash_inlet_focus_command)
            );
            assert!(
                flash_related_step
                    .produced_stream_actions
                    .iter()
                    .any(|action| action.command_id == flowing_outlet_focus_command)
            );
            assert!(
                flash_related_step
                    .produced_stream_actions
                    .iter()
                    .any(|action| action.command_id == zero_outlet_focus_command)
            );

            let comparison_inspector = snapshot.result_inspector_with_comparison(
                Some(case.flowing_outlet_stream_id),
                Some(case.zero_outlet_stream_id),
            );
            let comparison = comparison_inspector
                .comparison
                .as_ref()
                .expect("expected synthetic single-phase outlet comparison");
            assert_eq!(
                comparison.base_stream_focus_action.command_id,
                flowing_outlet_focus_command
            );
            assert_eq!(
                comparison.compared_stream_focus_action.command_id,
                zero_outlet_focus_command
            );

            let unit_inspector = snapshot.result_inspector_with_unit(
                Some(case.flowing_outlet_stream_id),
                Some(case.zero_outlet_stream_id),
                Some("flash-1"),
            );
            let selected_unit = unit_inspector
                .selected_unit
                .as_ref()
                .expect("expected synthetic single-phase flash unit result");
            assert!(
                selected_unit
                    .consumed_stream_actions
                    .iter()
                    .any(|action| action.command_id == flash_inlet_focus_command)
            );
            assert!(
                selected_unit
                    .produced_stream_actions
                    .iter()
                    .any(|action| action.command_id == flowing_outlet_focus_command)
            );
            assert!(
                selected_unit
                    .produced_stream_actions
                    .iter()
                    .any(|action| action.command_id == zero_outlet_focus_command)
            );
            assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
                action.source_label == "Selected unit"
                    && action.action.command_id == flash_unit_focus_command
            }));
            assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
                action.source_label == "Solve step"
                    && action.action.command_id == flash_unit_focus_command
            }));
            assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
                action.source_label == "Diagnostic"
                    && action.action.command_id == flash_inlet_focus_command
            }));

            let active_stream_detail = stream_target_detail_model(
                &snapshot,
                case.flowing_outlet_stream_id,
                "Flowing Outlet",
            );
            assert!(
                active_stream_detail
                    .diagnostic_actions
                    .iter()
                    .any(|action| {
                        action.source_label == "Inspector target"
                            && action.action.command_id == flowing_outlet_focus_command
                    })
            );
            assert!(
                active_stream_detail
                    .diagnostic_actions
                    .iter()
                    .any(|action| {
                        action.source_label == "Solve step"
                            && action.action.command_id == flash_unit_focus_command
                    })
            );

            let active_unit_detail = unit_target_detail_model(&snapshot, "flash-1", "Flash Drum");
            let active_unit = active_unit_detail
                .latest_unit_result
                .as_ref()
                .expect("expected active flash unit result");
            assert!(
                active_unit
                    .consumed_stream_results
                    .iter()
                    .any(|stream| stream.stream_id == scenario.flash_inlet_stream_id)
            );
            assert!(
                active_unit
                    .produced_stream_results
                    .iter()
                    .any(|stream| stream.stream_id == case.flowing_outlet_stream_id)
            );
            assert!(
                active_unit
                    .produced_stream_results
                    .iter()
                    .any(|stream| stream.stream_id == case.zero_outlet_stream_id)
            );
        }
    }
}
