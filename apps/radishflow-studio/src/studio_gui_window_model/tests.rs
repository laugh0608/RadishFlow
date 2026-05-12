use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rf_flash::estimate_bubble_dew_window;
use rf_store::{parse_project_file_json, project_file_to_pretty_json};
use rf_types::PhaseEquilibriumRegion;

use crate::{
    StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiHostCommandOutcome,
    StudioGuiWindowAreaId, StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion,
    StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutScopeKind,
    StudioGuiWindowStreamResultModel, StudioRuntimeConfig, StudioRuntimeEntitlementPreflight,
    StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
    test_support::{
        apply_official_binary_hydrocarbon_near_boundary_consumer_scenario,
        build_official_binary_hydrocarbon_provider,
        official_binary_hydrocarbon_near_boundary_consumer_scenarios,
    },
};

use super::test_support::{
    apply_stream_state_and_composition, build_synthetic_provider,
    solve_snapshot_model_from_project_with_provider_and_edit,
    solve_ui_and_window_snapshot_from_project_with_provider_and_edit, stream_target_detail_model,
    unit_target_detail_model,
};

mod synthetic_near_boundary_focus;

fn solve_binary_hydrocarbon_lite_snapshot(
    project_json: &str,
) -> crate::StudioGuiWindowSolveSnapshotModel {
    solve_snapshot_model_from_project_with_provider_and_edit(
        project_json,
        &build_official_binary_hydrocarbon_provider(),
        |_| {},
    )
}

fn find_ui_snapshot_stream<'a>(
    snapshot: &'a rf_ui::SolveSnapshot,
    stream_id: &str,
) -> &'a rf_ui::StreamStateSnapshot {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id.as_str() == stream_id)
        .expect("expected ui snapshot stream")
}

fn find_window_snapshot_stream<'a>(
    snapshot: &'a crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
) -> &'a StudioGuiWindowStreamResultModel {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .expect("expected window snapshot stream")
}

fn assert_flash_consumer_preserves_snapshot_stream_reference(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
) {
    let assert_stream_reference = |summary: &str, surface: &str| {
        assert!(
            summary.contains("T "),
            "expected {surface} to expose T for `{stream_id}`"
        );
        assert!(
            summary.contains("P "),
            "expected {surface} to expose P for `{stream_id}`"
        );
        assert!(
            summary.contains("F "),
            "expected {surface} to expose F for `{stream_id}`"
        );
        assert!(
            summary.contains("H "),
            "expected {surface} to expose H for `{stream_id}`"
        );
    };

    let step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .expect("expected flash solve step");
    let step_reference = step
        .consumed_stream_results
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .expect("expected flash solve step consumed stream");
    assert_stream_reference(&step_reference.summary, "flash solve step");

    let inspector = snapshot.result_inspector_with_unit(Some(stream_id), None, Some("flash-1"));
    let unit = inspector
        .selected_unit
        .as_ref()
        .expect("expected flash unit result");
    let unit_reference = unit
        .consumed_stream_results
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .expect("expected flash unit consumed stream");
    assert_stream_reference(&unit_reference.summary, "flash unit result");
}

fn assert_stream_reference_summary_matches_stream_model(
    summary: &str,
    stream: &StudioGuiWindowStreamResultModel,
    surface: &str,
) {
    assert!(
        summary.contains("T "),
        "expected {surface} to expose T for `{}`",
        stream.stream_id
    );
    assert!(
        summary.contains("P "),
        "expected {surface} to expose P for `{}`",
        stream.stream_id
    );
    assert!(
        summary.contains("F "),
        "expected {surface} to expose F for `{}`",
        stream.stream_id
    );
    if stream.molar_enthalpy_text.is_some() {
        assert!(
            summary.contains("H "),
            "expected {surface} to expose H for `{}`",
            stream.stream_id
        );
    }
}

fn assert_flash_step_and_unit_preserve_outlet_summaries(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
) {
    let inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some("flash-1"));
    let flash_step = inspector
        .related_steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .expect("expected flash solve step in result inspector");
    let selected_unit = inspector
        .selected_unit
        .as_ref()
        .expect("expected flash unit execution result");
    assert_eq!(selected_unit.unit_id, "flash-1");

    for stream_id in ["stream-liquid", "stream-vapor"] {
        let stream = find_window_snapshot_stream(snapshot, stream_id);
        let step_reference = flash_step
            .produced_stream_results
            .iter()
            .find(|reference| reference.stream_id == stream_id)
            .expect("expected flash step produced stream");
        assert_stream_reference_summary_matches_stream_model(
            &step_reference.summary,
            stream,
            "flash solve step",
        );

        let unit_reference = selected_unit
            .produced_stream_results
            .iter()
            .find(|reference| reference.stream_id == stream_id)
            .expect("expected flash unit produced stream");
        assert_stream_reference_summary_matches_stream_model(
            &unit_reference.summary,
            stream,
            "flash unit result",
        );
    }
}

fn assert_flash_outlet_comparison_matches_snapshot_stream_models(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
) {
    let liquid = find_window_snapshot_stream(snapshot, "stream-liquid");
    let vapor = find_window_snapshot_stream(snapshot, "stream-vapor");
    let comparison_inspector =
        snapshot.result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"));
    let comparison = comparison_inspector
        .comparison
        .as_ref()
        .expect("expected flash outlet comparison");

    let enthalpy_row = comparison
        .summary_rows
        .iter()
        .find(|row| row.label == "H")
        .expect("expected comparison enthalpy row");
    assert_eq!(
        enthalpy_row.base_value,
        liquid
            .molar_enthalpy_text
            .clone()
            .unwrap_or_else(|| "-".to_string())
    );
    assert_eq!(
        enthalpy_row.compared_value,
        vapor
            .molar_enthalpy_text
            .clone()
            .unwrap_or_else(|| "-".to_string())
    );
    if liquid.molar_enthalpy_text.is_some() && vapor.molar_enthalpy_text.is_some() {
        assert!(
            enthalpy_row.delta_text.ends_with(" J/mol"),
            "expected comparison delta to stay materialized when both flash outlets carry enthalpy"
        );
    } else {
        assert_eq!(enthalpy_row.delta_text, "-");
    }

    let overall_row = comparison
        .phase_rows
        .iter()
        .find(|row| row.phase_label == "overall")
        .expect("expected overall comparison phase row");
    if let Some(liquid_overall) = liquid
        .phase_rows
        .iter()
        .find(|phase| phase.label == "overall")
    {
        assert_eq!(
            overall_row.base_fraction_text,
            liquid_overall.phase_fraction_text
        );
        assert_eq!(
            overall_row.base_molar_flow_text,
            liquid_overall.molar_flow_text
        );
        assert_eq!(
            overall_row.base_molar_enthalpy_text,
            liquid_overall
                .molar_enthalpy_text
                .clone()
                .unwrap_or_else(|| "-".to_string())
        );
    } else {
        assert_eq!(overall_row.base_fraction_text, "-");
        assert_eq!(overall_row.base_molar_flow_text, "-");
        assert_eq!(overall_row.base_molar_enthalpy_text, "-");
    }
    if let Some(vapor_overall) = vapor
        .phase_rows
        .iter()
        .find(|phase| phase.label == "overall")
    {
        assert_eq!(
            overall_row.compared_fraction_text,
            vapor_overall.phase_fraction_text
        );
        assert_eq!(
            overall_row.compared_molar_flow_text,
            vapor_overall.molar_flow_text
        );
        assert_eq!(
            overall_row.compared_molar_enthalpy_text,
            vapor_overall
                .molar_enthalpy_text
                .clone()
                .unwrap_or_else(|| "-".to_string())
        );
    } else {
        assert_eq!(overall_row.compared_fraction_text, "-");
        assert_eq!(overall_row.compared_molar_flow_text, "-");
        assert_eq!(overall_row.compared_molar_enthalpy_text, "-");
    }

    for phase_label in ["liquid", "vapor"] {
        let comparison_row = comparison
            .phase_rows
            .iter()
            .find(|row| row.phase_label == phase_label);
        let liquid_phase = liquid
            .phase_rows
            .iter()
            .find(|phase| phase.label == phase_label);
        let vapor_phase = vapor
            .phase_rows
            .iter()
            .find(|phase| phase.label == phase_label);

        if liquid_phase.is_none() && vapor_phase.is_none() {
            assert!(
                comparison_row.is_none(),
                "expected no comparison row for absent `{phase_label}` phase"
            );
            continue;
        }

        let comparison_row = comparison_row.expect("expected comparison phase row");
        if let Some(phase) = liquid_phase {
            assert_eq!(comparison_row.base_fraction_text, phase.phase_fraction_text);
            assert_eq!(comparison_row.base_molar_flow_text, phase.molar_flow_text);
            assert_eq!(
                comparison_row.base_molar_enthalpy_text,
                phase
                    .molar_enthalpy_text
                    .clone()
                    .unwrap_or_else(|| "-".to_string())
            );
        } else {
            assert_eq!(comparison_row.base_fraction_text, "-");
            assert_eq!(comparison_row.base_molar_flow_text, "-");
            assert_eq!(comparison_row.base_molar_enthalpy_text, "-");
        }

        if let Some(phase) = vapor_phase {
            assert_eq!(
                comparison_row.compared_fraction_text,
                phase.phase_fraction_text
            );
            assert_eq!(
                comparison_row.compared_molar_flow_text,
                phase.molar_flow_text
            );
            assert_eq!(
                comparison_row.compared_molar_enthalpy_text,
                phase
                    .molar_enthalpy_text
                    .clone()
                    .unwrap_or_else(|| "-".to_string())
            );
        } else {
            assert_eq!(comparison_row.compared_fraction_text, "-");
            assert_eq!(comparison_row.compared_molar_flow_text, "-");
            assert_eq!(comparison_row.compared_molar_enthalpy_text, "-");
        }
    }
}

fn assert_result_inspector_with_unit_preserves_snapshot_stream(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    selected_unit_id: &str,
) {
    let inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some(selected_unit_id));
    let expected_stream = find_window_snapshot_stream(snapshot, selected_stream_id);
    assert_eq!(
        inspector.selected_stream_id.as_deref(),
        Some(selected_stream_id)
    );
    assert_eq!(
        inspector.selected_stream.as_ref(),
        Some(expected_stream),
        "expected unit view to preserve snapshot stream model for `{selected_stream_id}`"
    );
}

fn assert_result_inspector_with_comparison_preserves_snapshot_streams(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    base_stream_id: &str,
    compared_stream_id: &str,
) {
    let inspector =
        snapshot.result_inspector_with_comparison(Some(base_stream_id), Some(compared_stream_id));
    let expected_base = find_window_snapshot_stream(snapshot, base_stream_id);
    let expected_compared = find_window_snapshot_stream(snapshot, compared_stream_id);
    assert_eq!(
        inspector.selected_stream_id.as_deref(),
        Some(base_stream_id)
    );
    assert_eq!(
        inspector.selected_stream.as_ref(),
        Some(expected_base),
        "expected comparison base stream model to match snapshot stream `{base_stream_id}`"
    );
    assert_eq!(
        inspector.comparison_stream_id.as_deref(),
        Some(compared_stream_id)
    );
    assert_eq!(
        inspector.comparison_stream.as_ref(),
        Some(expected_compared),
        "expected comparison stream model to match snapshot stream `{compared_stream_id}`"
    );

    let comparison = inspector
        .comparison
        .as_ref()
        .expect("expected comparison model");
    assert_eq!(comparison.base_stream_id, base_stream_id);
    assert_eq!(comparison.compared_stream_id, compared_stream_id);
}

fn assert_window_model_preserves_ui_stream_window(
    ui_snapshot: &rf_ui::SolveSnapshot,
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) {
    let ui_stream = find_ui_snapshot_stream(ui_snapshot, stream_id);
    let ui_window = ui_stream
        .bubble_dew_window
        .as_ref()
        .expect("expected ui snapshot bubble/dew window");

    let result_inspector = snapshot.result_inspector(Some(stream_id));
    let result_stream = result_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected stream");
    let result_window = result_stream
        .bubble_dew_window
        .as_ref()
        .expect("expected result inspector bubble/dew window");

    assert_eq!(result_stream.stream_id, ui_stream.stream_id.as_str());
    assert_eq!(result_stream.temperature_k, ui_stream.temperature_k);
    assert_eq!(result_stream.pressure_pa, ui_stream.pressure_pa);
    assert_eq!(
        result_stream.total_molar_flow_mol_s,
        ui_stream.total_molar_flow_mol_s
    );
    assert_eq!(result_window.phase_region, ui_window.phase_region.as_str());
    assert_eq!(
        result_window.bubble_pressure_pa,
        ui_window.bubble_pressure_pa
    );
    assert_eq!(result_window.dew_pressure_pa, ui_window.dew_pressure_pa);
    assert_eq!(
        result_window.bubble_temperature_k,
        ui_window.bubble_temperature_k
    );
    assert_eq!(result_window.dew_temperature_k, ui_window.dew_temperature_k);
    assert_eq!(
        result_window.bubble_pressure_text,
        format!("{:.0} Pa", ui_window.bubble_pressure_pa)
    );
    assert_eq!(
        result_window.dew_pressure_text,
        format!("{:.0} Pa", ui_window.dew_pressure_pa)
    );
    assert_eq!(
        result_window.bubble_temperature_text,
        format!("{:.2} K", ui_window.bubble_temperature_k)
    );
    assert_eq!(
        result_window.dew_temperature_text,
        format!("{:.2} K", ui_window.dew_temperature_k)
    );

    let active_detail = stream_target_detail_model(snapshot, stream_id, title);
    let active_stream = active_detail
        .latest_stream_result
        .as_ref()
        .expect("expected active inspector latest stream result");
    assert_eq!(active_stream, result_stream);
}

fn assert_window_model_preserves_ui_stream_window_absence(
    ui_snapshot: &rf_ui::SolveSnapshot,
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) {
    let ui_stream = find_ui_snapshot_stream(ui_snapshot, stream_id);
    assert!(
        ui_stream.bubble_dew_window.is_none(),
        "expected ui snapshot bubble/dew window absence for `{stream_id}`"
    );

    let result_inspector = snapshot.result_inspector(Some(stream_id));
    let result_stream = result_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected stream");
    assert!(
        result_stream.bubble_dew_window.is_none(),
        "expected result inspector bubble/dew window absence for `{stream_id}`"
    );

    let active_detail = stream_target_detail_model(snapshot, stream_id, title);
    let active_stream = active_detail
        .latest_stream_result
        .as_ref()
        .expect("expected active inspector latest stream result");
    assert_eq!(active_stream, result_stream);
    assert!(
        active_stream.bubble_dew_window.is_none(),
        "expected active inspector bubble/dew window absence for `{stream_id}`"
    );
}

fn assert_stream_window_visible_in_result_and_active_inspectors(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
    assert_window: impl Fn(
        &StudioGuiWindowStreamResultModel,
        &super::StudioGuiWindowBubbleDewWindowModel,
    ),
) {
    let result_inspector = snapshot.result_inspector(Some(stream_id));
    let result_stream = result_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected stream");
    let result_window = result_stream
        .bubble_dew_window
        .as_ref()
        .expect("expected result inspector bubble/dew window");
    assert_window(result_stream, result_window);

    let active_detail = stream_target_detail_model(snapshot, stream_id, title);
    let active_window = active_detail
        .latest_stream_result
        .as_ref()
        .and_then(|stream| stream.bubble_dew_window.as_ref())
        .expect("expected active inspector bubble/dew window");
    assert_eq!(active_window, result_window);
}

fn assert_non_flash_intermediate_stream_summary_and_context(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
    producer_unit_id: &str,
) {
    let focus_stream_command_id = format!("inspector.focus_stream:{stream_id}");
    let focus_unit_command_id = format!("inspector.focus_unit:{producer_unit_id}");

    let assert_summary_rows = |stream: &StudioGuiWindowStreamResultModel, surface: &str| {
        let molar_enthalpy_text = stream
            .molar_enthalpy_text
            .as_deref()
            .expect("expected non-flash intermediate stream enthalpy text");
        assert_eq!(
            stream.summary_rows.len(),
            4,
            "expected {surface} stream `{stream_id}` to keep T/P/F/H summary rows"
        );
        assert!(stream.summary_rows.iter().any(|row| {
            row.label == "T"
                && row.detail_label == "Temperature"
                && row.value == stream.temperature_text
        }));
        assert!(stream.summary_rows.iter().any(|row| {
            row.label == "P" && row.detail_label == "Pressure" && row.value == stream.pressure_text
        }));
        assert!(stream.summary_rows.iter().any(|row| {
            row.label == "F"
                && row.detail_label == "Molar flow"
                && row.value == stream.molar_flow_text
        }));
        assert!(stream.summary_rows.iter().any(|row| {
            row.label == "H"
                && row.detail_label == "Molar enthalpy"
                && row.value == molar_enthalpy_text
        }));
        assert!(
            stream.phase_text.contains("mol/s"),
            "expected {surface} stream `{stream_id}` to keep phase molar flow context"
        );
    };

    let result_inspector = snapshot.result_inspector(Some(stream_id));
    let result_stream = result_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected non-flash intermediate stream");
    assert_summary_rows(result_stream, "result inspector");
    assert!(
        result_inspector.stream_options.iter().any(|option| {
            option.stream_id == stream_id
                && option.is_selected
                && option.summary.contains("T ")
                && option.summary.contains("P ")
                && option.summary.contains("F ")
                && option.summary.contains("H ")
        }),
        "expected result inspector option summary for `{stream_id}` to keep T/P/F/H"
    );

    let producer_step = result_inspector
        .related_steps
        .iter()
        .find(|step| step.unit_id == producer_unit_id)
        .expect("expected producing unit step in result inspector");
    assert!(
        producer_step
            .produced_stream_actions
            .iter()
            .any(|action| { action.command_id == focus_stream_command_id })
    );
    assert!(producer_step.produced_stream_results.iter().any(|stream| {
        stream.stream_id == stream_id
            && stream.summary.contains("T ")
            && stream.summary.contains("P ")
            && stream.summary.contains("F ")
            && stream.summary.contains("H ")
    }));

    let flash_step = result_inspector
        .related_steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .expect("expected downstream flash step in result inspector");
    assert!(
        flash_step
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == stream_id)
    );
    assert!(
        flash_step
            .consumed_stream_actions
            .iter()
            .any(|action| { action.command_id == focus_stream_command_id })
    );
    assert!(flash_step.consumed_stream_results.iter().any(|stream| {
        stream.stream_id == stream_id
            && stream.summary.contains("T ")
            && stream.summary.contains("P ")
            && stream.summary.contains("F ")
            && stream.summary.contains("H ")
    }));

    assert!(
        result_inspector
            .related_diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "solver.unit_executed"
                    && diagnostic
                        .related_unit_ids
                        .iter()
                        .any(|unit_id| unit_id == producer_unit_id)
            }),
        "expected result inspector to keep producing unit diagnostics for `{stream_id}`"
    );
    assert!(
        result_inspector
            .related_diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "solver.unit_executed"
                    && diagnostic
                        .related_unit_ids
                        .iter()
                        .any(|unit_id| unit_id == "flash-1")
            }),
        "expected result inspector to keep downstream flash diagnostics for `{stream_id}`"
    );
    assert!(result_inspector.diagnostic_actions.iter().any(|action| {
        action.source_label == "Selected stream"
            && action.action.command_id == focus_stream_command_id
    }));
    assert!(result_inspector.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step" && action.action.command_id == focus_unit_command_id
    }));

    let active_detail = stream_target_detail_model(snapshot, stream_id, title);
    let active_stream = active_detail
        .latest_stream_result
        .as_ref()
        .expect("expected active inspector latest stream result");
    assert_eq!(active_stream, result_stream);
    assert_summary_rows(active_stream, "active inspector");
    assert!(active_detail.related_steps.iter().any(|step| {
        step.unit_id == producer_unit_id
            && step
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == focus_stream_command_id)
    }));
    assert!(active_detail.related_steps.iter().any(|step| {
        step.unit_id == "flash-1"
            && step
                .consumed_stream_actions
                .iter()
                .any(|action| action.command_id == focus_stream_command_id)
    }));
    assert!(
        active_detail.related_diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "solver.unit_executed"
                && diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == producer_unit_id)
        }),
        "expected active inspector to keep producing unit diagnostics for `{stream_id}`"
    );
    assert!(
        active_detail.related_diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "solver.unit_executed"
                && diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == "flash-1")
        }),
        "expected active inspector to keep downstream flash diagnostics for `{stream_id}`"
    );
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Inspector target"
            && action.action.command_id == focus_stream_command_id
    }));
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step" && action.action.command_id == focus_unit_command_id
    }));
}

fn assert_non_flash_intermediate_unit_summary_and_context(
    snapshot: &crate::StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    unit_id: &str,
    title: &str,
    consumed_stream_ids: &[&str],
    produced_stream_id: &str,
) {
    let focus_unit_command_id = format!("inspector.focus_unit:{unit_id}");
    let consumed_focus_command_ids: Vec<String> = consumed_stream_ids
        .iter()
        .map(|stream_id| format!("inspector.focus_stream:{stream_id}"))
        .collect();
    let produced_focus_command_id = format!("inspector.focus_stream:{produced_stream_id}");

    let assert_unit_result = |unit: &crate::StudioGuiWindowUnitExecutionResultModel,
                              surface: &str| {
        assert_eq!(unit.unit_id, unit_id);
        assert_eq!(unit.status_label, "Converged");
        assert!(
            unit.step_index >= 1,
            "expected {surface} unit `{unit_id}` to point at an executed solve step"
        );
        assert_eq!(
            unit.consumed_stream_results
                .iter()
                .map(|stream| stream.stream_id.clone())
                .collect::<Vec<_>>(),
            consumed_stream_ids
                .iter()
                .map(|stream_id| stream_id.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            unit.produced_stream_results
                .iter()
                .map(|stream| stream.stream_id.clone())
                .collect::<Vec<_>>(),
            vec![produced_stream_id.to_string()]
        );
        assert!(
            unit.summary.contains(produced_stream_id),
            "expected {surface} unit `{unit_id}` summary to mention produced stream `{produced_stream_id}`"
        );
        for command_id in &consumed_focus_command_ids {
            assert!(
                unit.consumed_stream_actions
                    .iter()
                    .any(|action| action.command_id == *command_id)
            );
        }
        for stream_id in consumed_stream_ids {
            assert!(unit.consumed_stream_results.iter().any(|stream| {
                stream.stream_id == *stream_id
                    && stream.summary.contains("T ")
                    && stream.summary.contains("P ")
                    && stream.summary.contains("F ")
                    && stream.summary.contains("H ")
            }));
        }
        assert!(
            unit.produced_stream_actions
                .iter()
                .any(|action| action.command_id == produced_focus_command_id)
        );
        assert!(unit.produced_stream_results.iter().any(|stream| {
            stream.stream_id == produced_stream_id
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")
                && stream.summary.contains("H ")
        }));
    };

    let unit_inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some(unit_id));
    assert_eq!(unit_inspector.selected_unit_id.as_deref(), Some(unit_id));
    let selected_unit = unit_inspector
        .selected_unit
        .as_ref()
        .expect("expected selected unit execution result");
    assert_unit_result(selected_unit, "result inspector");
    assert!(
        unit_inspector
            .unit_options
            .iter()
            .any(|option| option.unit_id == unit_id
                && option.is_selected
                && option.focus_action.command_id == focus_unit_command_id
                && option.summary.contains("step #")
                && option.summary.contains(produced_stream_id)),
        "expected unit option for `{unit_id}` to expose selected step summary"
    );
    assert!(
        unit_inspector
            .unit_related_steps
            .iter()
            .all(|step| step.unit_id == unit_id),
        "expected unit-related steps to stay filtered to `{unit_id}`"
    );
    assert!(
        unit_inspector
            .unit_related_diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .related_unit_ids
                .iter()
                .any(|candidate| candidate == unit_id)
                || diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|candidate| candidate == produced_stream_id)),
        "expected unit-related diagnostics to include `{unit_id}` outputs"
    );
    assert!(
        unit_inspector
            .unit_related_diagnostics
            .iter()
            .any(
                |diagnostic| consumed_stream_ids.iter().all(|stream_id| diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|candidate| candidate == *stream_id))
                    || consumed_stream_ids.iter().any(|stream_id| diagnostic
                        .related_stream_ids
                        .iter()
                        .any(|candidate| candidate == *stream_id))
            ),
        "expected unit-related diagnostics to include consumed stream context for `{unit_id}`"
    );
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Selected unit" && action.action.command_id == focus_unit_command_id
    }));
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step" && action.action.command_id == focus_unit_command_id
    }));
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Diagnostic"
            && consumed_focus_command_ids.contains(&action.action.command_id)
    }));
    assert!(!unit_inspector.has_stale_unit_selection);

    let active_detail = unit_target_detail_model(snapshot, unit_id, title);
    let active_unit = active_detail
        .latest_unit_result
        .as_ref()
        .expect("expected active unit detail to expose latest execution result");
    assert_eq!(active_unit, selected_unit);
    assert_unit_result(active_unit, "active inspector");
    assert_eq!(active_detail.latest_stream_result, None);
    assert!(
        active_detail
            .related_steps
            .iter()
            .all(|step| step.unit_id == unit_id),
        "expected active unit inspector related steps to stay filtered to `{unit_id}`"
    );
    assert!(
        active_detail
            .related_diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .related_unit_ids
                .iter()
                .any(|candidate| candidate == unit_id)
                || diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|candidate| candidate == produced_stream_id)),
        "expected active unit inspector diagnostics to include `{unit_id}` outputs"
    );
    assert!(
        active_detail
            .related_diagnostics
            .iter()
            .any(
                |diagnostic| consumed_stream_ids.iter().any(|stream_id| diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|candidate| candidate == *stream_id))
            ),
        "expected active unit inspector diagnostics to include consumed stream context for `{unit_id}`"
    );
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Inspector target"
            && action.action.command_id == focus_unit_command_id
    }));
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step" && action.action.command_id == focus_unit_command_id
    }));
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Diagnostic"
            && consumed_focus_command_ids.contains(&action.action.command_id)
    }));
}

fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

fn synced_example_config(project_file_name: &str) -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join(project_file_name),
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        ..StudioRuntimeConfig::default()
    }
}

fn official_near_boundary_consumer_synced_config(
    scenario: &crate::test_support::OfficialBinaryHydrocarbonNearBoundaryConsumerScenario,
) -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-near-boundary-{timestamp}.rfproj.json"
    ));
    let mut project =
        parse_project_file_json(scenario.project_json).expect("expected near-boundary project");
    apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(&mut project, scenario);
    let project_json =
        project_file_to_pretty_json(&project).expect("expected near-boundary project json");
    fs::write(&project_path, project_json).expect("expected near-boundary project write");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..StudioRuntimeConfig::default()
        },
        project_path,
    )
}

fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-window-model-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::build_flash_drum_local_rules_project_json();
    fs::write(&project_path, project).expect("expected local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
    )
}

fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("failures")
            .join("unbound-outlet-port.rfproj.json"),
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
    }
}

fn missing_upstream_failure_synced_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("failures")
            .join("missing-upstream-source.rfproj.json"),
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
    }
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

#[test]
fn studio_gui_window_model_groups_snapshot_into_window_regions() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window = dispatch.snapshot.window_model();

    assert_eq!(window.header.title, "RadishFlow Studio");
    assert_eq!(window.header.registered_window_count, 1);
    assert_eq!(window.header.foreground_window_id, Some(1));
    assert_eq!(window.header.entitlement_timer_owner_window_id, Some(1));
    assert!(window.header.status_line.contains("registered windows: 1"));
    assert!(window.header.status_line.contains("foreground: #1"));
    assert!(window.header.status_line.contains("timer owner: #1"));

    assert_eq!(
        window.commands.total_command_count,
        dispatch
            .snapshot
            .command_registry
            .sections
            .iter()
            .map(|section| section.commands.len())
            .sum::<usize>()
    );
    assert!(
        window.commands.enabled_command_count >= 1,
        "expected at least one enabled command"
    );
    assert_eq!(
        window
            .commands
            .command_list_sections
            .first()
            .map(|section| section.title),
        Some("File")
    );
    assert!(
        window
            .commands
            .command_list_sections
            .iter()
            .any(|section| section.title == "Canvas"),
        "expected canvas command section when suggestions exist"
    );

    assert_eq!(window.canvas.title, "Canvas");
    assert_eq!(window.canvas.suggestion_count, 3);
    assert_eq!(window.canvas.enabled_action_count, 10);
    assert_eq!(
        window.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
    );
    assert_eq!(
        window.canvas.widget.primary_action().label,
        "Accept suggestion"
    );

    assert_eq!(window.runtime.title, "Runtime");
    assert_eq!(
        window.runtime.control_state.run_status,
        rf_ui::RunStatus::Idle
    );
    assert_eq!(
        window.runtime.run_panel.view().primary_action.label,
        "Resume"
    );
    assert_eq!(window.runtime.example_projects.len(), 6);
    assert_eq!(
        window
            .runtime
            .example_projects
            .iter()
            .find(|example| example.is_current)
            .map(|example| example.id),
        None,
        "temporary edited project should not be marked as a bundled example"
    );
    assert!(window.runtime.entitlement_host.is_some());
    assert!(window.runtime.platform_timer_lines.is_empty());
    assert!(window.runtime.gui_activity_lines.is_empty());
    assert_eq!(
        window.runtime.latest_log_entry,
        window.runtime.log_entries.last().cloned()
    );
    assert_eq!(
        window.layout_state.scope.kind,
        StudioGuiWindowLayoutScopeKind::Window
    );
    assert_eq!(window.layout_state.scope.layout_slot, Some(1));
    assert_eq!(
        window.layout_state.scope.layout_key,
        "studio.window.owner.slot-1"
    );
    assert_eq!(window.drop_preview, None);

    let _ = fs::remove_file(project_path);
}

#[test]
fn studio_gui_window_model_surfaces_bootstrap_workspace_results_and_diagnostics() {
    let config = synced_example_config("feed-heater-flash-binary-hydrocarbon.rfproj.json");
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected run dispatch");
    let window = dispatch.window;

    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
    assert_eq!(window.runtime.workspace_document.revision, 0);
    assert_eq!(window.runtime.workspace_document.unit_count, 3);
    assert_eq!(window.runtime.workspace_document.snapshot_history_count, 1);

    let snapshot = window
        .runtime
        .latest_solve_snapshot
        .expect("expected latest solve snapshot");
    assert_eq!(snapshot.status_label, "Converged");
    assert_eq!(snapshot.stream_count, 4);
    assert_eq!(snapshot.step_count, 3);
    assert_eq!(snapshot.diagnostic_count, 4);
    let heated_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-heated")
        .expect("expected heated stream");
    assert_eq!(heated_stream.temperature_text, "345.00 K");
    assert!(heated_stream.composition_text.contains("methane="));
    assert_eq!(heated_stream.summary_rows.len(), 4);
    assert!(heated_stream.summary_rows.iter().any(|row| row.label == "T"
        && row.detail_label == "Temperature"
        && row.value == "345.00 K"));
    assert!(heated_stream.summary_rows.iter().any(|row| {
        row.label == "H" && row.detail_label == "Molar enthalpy" && row.value.ends_with(" J/mol")
    }));
    assert!(
        heated_stream
            .composition_rows
            .iter()
            .any(|row| row.component_id == "methane" && !row.fraction_text.is_empty())
    );
    assert!(
        heated_stream.phase_text.contains("mol/s"),
        "expected phase summary text to include phase molar flow"
    );
    let liquid_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-liquid")
        .expect("expected liquid outlet stream");
    assert!(
        liquid_stream.total_molar_flow_mol_s.abs() < 1e-12,
        "expected bootstrap sample liquid outlet to stay zero-flow in the current single-phase runtime baseline"
    );
    assert!(
        liquid_stream.molar_enthalpy_j_per_mol.is_none(),
        "expected zero-flow bootstrap liquid outlet to omit overall molar enthalpy"
    );
    assert!(
        !liquid_stream
            .summary_rows
            .iter()
            .any(|row| { row.label == "H" || row.detail_label == "Molar enthalpy" }),
        "expected zero-flow bootstrap liquid outlet to avoid surfacing H summary rows"
    );
    assert!(
        liquid_stream.phase_rows.is_empty(),
        "expected zero-flow bootstrap liquid outlet to avoid phase rows"
    );

    let vapor_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-vapor")
        .expect("expected vapor outlet stream");
    assert!(vapor_stream.total_molar_flow_mol_s > 0.0);
    assert!(
        vapor_stream.molar_enthalpy_j_per_mol.is_some(),
        "expected flowing bootstrap vapor outlet to expose overall molar enthalpy from flash result"
    );
    assert!(vapor_stream.summary_rows.iter().any(|row| {
        row.label == "H" && row.detail_label == "Molar enthalpy" && row.value.ends_with(" J/mol")
    }));
    assert!(vapor_stream.phase_rows.iter().any(|row| {
        row.phase_fraction_text == "1.0000"
            && (row.molar_flow_mol_s - vapor_stream.total_molar_flow_mol_s).abs() < 1e-9
            && row.molar_flow_text == vapor_stream.molar_flow_text
            && row.composition_text.contains("methane=")
            && row
                .molar_enthalpy_text
                .as_deref()
                .is_some_and(|value| value.ends_with(" J/mol"))
    }));
    assert!(
        snapshot
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "solver.unit_executed")
    );
    assert!(
        snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.target_candidates.iter().any(|target| {
                target.kind_label == "Unit"
                    && target.target_id == "heater-1"
                    && target.command_id == "inspector.focus_unit:heater-1"
                    && target.action.command_id == "inspector.focus_unit:heater-1"
                    && target.action.label == "Unit heater-1"
            })
        }),
        "expected diagnostics to expose unit inspector target candidates"
    );
    assert!(
        snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.related_stream_results.iter().any(|stream| {
                stream.summary.contains("T ")
                    && stream.summary.contains("P ")
                    && stream.summary.contains("F ")
            })
        }),
        "expected diagnostics to expose related stream numeric summaries from solve snapshot"
    );
    let inspector = snapshot.result_inspector(Some("stream-heated"));
    assert_eq!(
        inspector.selected_stream_id.as_deref(),
        Some("stream-heated")
    );
    assert_eq!(
        inspector
            .selected_stream
            .as_ref()
            .map(|stream| stream.temperature_text.as_str()),
        Some("345.00 K")
    );
    assert!(
        inspector
            .stream_options
            .iter()
            .any(|option| option.stream_id == "stream-heated"
                && option.is_selected
                && option.summary.contains("P 95000 Pa")
                && option.summary.contains("H ")
                && option.focus_action.label == "Inspect"
                && option.focus_action.command_id == "inspector.focus_stream:stream-heated")
    );
    assert!(
        inspector
            .stream_options
            .iter()
            .any(|option| option.stream_id == "stream-vapor"
                && option.summary.contains("H ")
                && option.summary.contains("J/mol")),
        "expected stream result options to include enthalpy when it exists"
    );
    assert!(
        inspector
            .related_steps
            .iter()
            .any(|step| step.unit_id == "heater-1")
    );
    assert!(
        inspector
            .related_steps
            .iter()
            .any(|step| step.unit_id == "flash-1"
                && step
                    .consumed_stream_results
                    .iter()
                    .any(|stream| stream.stream_id == "stream-heated")),
        "expected stream result inspector to include downstream consuming step"
    );
    let related_heater_step = inspector
        .related_steps
        .iter()
        .find(|step| step.unit_id == "heater-1")
        .expect("expected related heater step");
    assert_eq!(
        related_heater_step.unit_action.command_id,
        "inspector.focus_unit:heater-1"
    );
    assert!(related_heater_step.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step"
            && action.target_label == "Unit"
            && action.action.command_id == "inspector.focus_unit:heater-1"
    }));
    assert_eq!(
        related_heater_step
            .consumed_stream_results
            .iter()
            .map(|stream| stream.stream_id.clone())
            .collect::<Vec<_>>(),
        vec!["stream-feed".to_string()]
    );
    assert!(
        related_heater_step
            .consumed_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-feed")
    );
    assert!(
        related_heater_step
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-feed"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")
                && stream.summary.contains("H ")),
        "expected solve step consumed stream summaries to expose T/P/F/H"
    );
    assert!(
        related_heater_step
            .produced_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
    );
    assert!(
        related_heater_step
            .produced_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-heated"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")),
        "expected solve step produced stream summaries to expose T/P/F"
    );
    assert!(related_heater_step.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step"
            && action.target_label == "Stream"
            && action.summary == "Step #1 input stream stream-feed"
            && action.action.command_id == "inspector.focus_stream:stream-feed"
    }));
    assert!(inspector.diagnostic_actions.iter().any(|action| {
        action.source_label == "Selected stream"
            && action.action.command_id == "inspector.focus_stream:stream-heated"
    }));
    assert!(inspector.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step"
            && action.action.command_id == "inspector.focus_unit:heater-1"
    }));
    let flash_step = inspector
        .related_steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .expect("expected related flash step");
    assert!(
        flash_step
            .produced_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-vapor" && stream.summary.contains("H ")),
        "expected flash step produced stream summaries to expose enthalpy when materialized"
    );
    assert!(
        inspector.related_diagnostics.iter().any(|diagnostic| {
            diagnostic.target_candidates.iter().any(|target| {
                target.kind_label == "Stream"
                    && target.target_id == "stream-heated"
                    && target.command_id == "inspector.focus_stream:stream-heated"
            })
        }),
        "expected related diagnostics to expose stream target candidates"
    );
    assert!(
        inspector.related_diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "solver.unit_executed"
                && diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == "heater-1")
        }),
        "expected result inspector to include diagnostics from the unit that produced the selected stream"
    );
    assert!(
        inspector.related_diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "solver.unit_executed"
                && diagnostic
                    .related_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == "flash-1")
        }),
        "expected result inspector to include diagnostics from units that consumed the selected stream"
    );
    let stream_target_command_id = inspector
        .stream_options
        .iter()
        .find(|option| option.stream_id == "stream-heated")
        .map(|option| option.focus_action.command_id.clone())
        .expect("expected stream result option focus command id");

    let target_dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: stream_target_command_id,
        })
        .expect("expected inspector target dispatch");
    assert_eq!(
        target_dispatch
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-heated"))
    );
    let active_detail = target_dispatch
        .window
        .runtime
        .active_inspector_detail
        .expect("expected active stream inspector detail");
    assert!(
        active_detail.property_fields.iter().any(|field| {
            field.key == "stream:stream-heated:temperature_k"
                && field.value_kind_label == "Number"
                && field.status_label == "Synced"
                && !field.is_dirty
                && field.draft_update_command_id
                    == "inspector.update_stream_draft:stream:stream-heated:temperature_k"
                && field.commit_command_id.is_none()
        }),
        "expected stream inspector detail to expose field-level property presentation"
    );
    assert!(
        active_detail.property_fields.iter().any(|field| {
            field.key == "stream:stream-heated:overall_mole_fraction:methane"
                && field.label == "Overall mole fraction (methane)"
                && field.value_kind_label == "Number"
                && field.draft_update_command_id
                    == "inspector.update_stream_draft:stream:stream-heated:overall_mole_fraction:methane"
        }),
        "expected stream inspector detail to expose composition field presentation"
    );
    assert_eq!(
        active_detail
            .property_composition_summary
            .as_ref()
            .map(|summary| (
                summary.status_label,
                summary.current_sum_text.as_str(),
                summary.normalized_preview_text.contains("methane=")
            )),
        Some(("Synced", "1.000000", true)),
        "expected stream inspector detail to expose composition sum and normalized preview"
    );
    assert_eq!(
        active_detail
            .latest_stream_result
            .as_ref()
            .map(|stream| stream.stream_id.as_str()),
        Some("stream-heated")
    );
    assert!(
        active_detail
            .related_steps
            .iter()
            .any(|step| step.unit_id == "heater-1")
    );
    assert!(
        active_detail
            .related_steps
            .iter()
            .any(|step| step.unit_id == "flash-1"
                && step
                    .consumed_stream_actions
                    .iter()
                    .any(|action| action.command_id == "inspector.focus_stream:stream-heated")),
        "expected active stream inspector to include downstream consuming step"
    );
    assert!(active_detail.related_steps.iter().any(|step| {
        step.unit_action.command_id == "inspector.focus_unit:heater-1"
            && step
                .consumed_stream_actions
                .iter()
                .any(|action| action.command_id == "inspector.focus_stream:stream-feed")
            && step
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
    }));
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Inspector target"
            && action.action.command_id == "inspector.focus_stream:stream-heated"
    }));
    assert!(active_detail.diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step"
            && action.action.command_id == "inspector.focus_unit:heater-1"
    }));
    assert!(
        active_detail
            .related_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "solver.unit_executed")
    );

    let unit_target_dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_unit:heater-1".to_string(),
        })
        .expect("expected unit inspector target dispatch");
    let unit_detail = unit_target_dispatch
        .window
        .runtime
        .active_inspector_detail
        .expect("expected active unit inspector detail");
    let unit_result = unit_detail
        .latest_unit_result
        .as_ref()
        .expect("expected active unit detail to expose latest execution result");
    assert_eq!(unit_result.unit_id, "heater-1");
    assert_eq!(unit_result.status_label, "Converged");
    assert_eq!(unit_result.step_index, 1);
    assert_eq!(
        unit_result
            .consumed_stream_results
            .iter()
            .map(|stream| stream.stream_id.clone())
            .collect::<Vec<_>>(),
        vec!["stream-feed".to_string()]
    );
    assert_eq!(
        unit_result
            .produced_stream_results
            .iter()
            .map(|stream| stream.stream_id.clone())
            .collect::<Vec<_>>(),
        vec!["stream-heated".to_string()]
    );
    assert!(unit_result.summary.contains("stream-heated"));
    assert!(
        unit_result
            .consumed_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-feed")
    );
    assert!(
        unit_result
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-feed"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")
                && stream.summary.contains("H ")),
        "expected active unit result to expose consumed stream numeric summary"
    );
    assert!(
        unit_result
            .produced_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-heated")
    );
    assert!(
        unit_result
            .produced_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-heated"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")),
        "expected active unit result to expose produced stream numeric summary"
    );
    assert_eq!(unit_detail.latest_stream_result, None);

    let fallback_inspector = snapshot.result_inspector(Some("missing-stream"));
    assert!(fallback_inspector.has_stale_selection);
    assert_eq!(
        fallback_inspector.selected_stream_id.as_deref(),
        snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str())
    );

    let comparison_inspector =
        snapshot.result_inspector_with_comparison(Some("stream-feed"), Some("stream-heated"));
    assert_eq!(
        comparison_inspector.selected_stream_id.as_deref(),
        Some("stream-feed")
    );
    assert_eq!(
        comparison_inspector.comparison_stream_id.as_deref(),
        Some("stream-heated")
    );
    assert!(
        comparison_inspector
            .comparison_options
            .iter()
            .all(|option| option.stream_id != "stream-feed")
    );
    assert!(
        comparison_inspector
            .comparison_options
            .iter()
            .any(|option| option.stream_id == "stream-vapor"
                && option.summary.contains("H ")
                && option.summary.contains("J/mol")
                && option.focus_action.command_id == "inspector.focus_stream:stream-vapor"),
        "expected comparison stream options to expose enthalpy and the same inspector focus action"
    );
    let comparison = comparison_inspector
        .comparison
        .as_ref()
        .expect("expected stream comparison model");
    assert_eq!(
        comparison.base_stream_focus_action.command_id,
        "inspector.focus_stream:stream-feed"
    );
    assert_eq!(
        comparison.compared_stream_focus_action.command_id,
        "inspector.focus_stream:stream-heated"
    );
    assert_eq!(comparison.base_stream_focus_action.label, "Inspect");
    assert_eq!(comparison.compared_stream_focus_action.label, "Inspect");
    assert!(comparison.summary_rows.iter().any(|row| {
        row.label == "T"
            && row.detail_label == "Temperature"
            && row.base_value.ends_with(" K")
            && row.compared_value.ends_with(" K")
            && row.delta_text.ends_with(" K")
    }));
    assert!(comparison.composition_rows.iter().any(|row| {
        row.component_id == "methane"
            && !row.base_fraction_text.is_empty()
            && !row.compared_fraction_text.is_empty()
            && (row.delta_text.starts_with('+') || row.delta_text.starts_with('-'))
    }));
    let phase_comparison =
        snapshot.result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"));
    let phase_comparison = phase_comparison
        .comparison
        .as_ref()
        .expect("expected phase outlet comparison model");
    assert!(phase_comparison.summary_rows.iter().any(|row| {
        row.label == "H"
            && row.detail_label == "Molar enthalpy"
            && row.base_value == "-"
            && row.compared_value.ends_with(" J/mol")
            && row.delta_text == "-"
    }));
    assert!(phase_comparison.phase_rows.iter().any(|row| {
        row.phase_label == "overall"
            && row.base_fraction_text == "-"
            && row.compared_fraction_text == "1.0000"
            && row.fraction_delta_text == "-"
            && row.base_molar_flow_text == "-"
            && row.compared_molar_flow_text.ends_with(" mol/s")
            && row.molar_flow_delta_text == "-"
            && row.base_molar_enthalpy_text == "-"
            && row.compared_molar_enthalpy_text.ends_with(" J/mol")
            && row.molar_enthalpy_delta_text == "-"
    }));
    assert!(
        !phase_comparison
            .phase_rows
            .iter()
            .any(|row| row.phase_label == "liquid"),
        "expected zero-flow bootstrap liquid outlet to contribute no liquid phase comparison row"
    );
    assert!(phase_comparison.phase_rows.iter().any(|row| {
        row.phase_label == "vapor"
            && row.base_molar_flow_text == "-"
            && row.compared_molar_flow_text.ends_with(" mol/s")
            && row.molar_flow_delta_text == "-"
            && row.base_molar_enthalpy_text == "-"
            && row.compared_molar_enthalpy_text.ends_with(" J/mol")
            && row.molar_enthalpy_delta_text == "-"
    }));
    let stale_comparison =
        snapshot.result_inspector_with_comparison(Some("stream-feed"), Some("stream-feed"));
    assert!(stale_comparison.has_stale_comparison);
    assert_eq!(stale_comparison.comparison, None);

    // unit-centric Result Inspector contract
    let default_inspector = snapshot.result_inspector(Some("stream-heated"));
    assert!(
        !default_inspector.unit_options.is_empty(),
        "expected unit options in result inspector"
    );
    assert!(
        default_inspector
            .unit_options
            .iter()
            .any(|option| option.unit_id == "feed-1"
                && option.focus_action.command_id == "inspector.focus_unit:feed-1"
                && option.focus_action.label == "Inspect"
                && option.summary.contains("step #")
                && option.summary.contains("out stream-feed")),
        "expected unit option to expose canonical inspector command"
    );
    assert!(
        default_inspector
            .unit_options
            .iter()
            .any(|option| option.unit_id == "heater-1"
                && option.focus_action.command_id == "inspector.focus_unit:heater-1"
                && option.summary.contains("step #1")
                && option.summary.contains("in stream-feed")
                && option.summary.contains("out stream-heated")),
        "expected unit option summary to expose consumed and produced stream context"
    );
    let default_unit_id = default_inspector
        .selected_unit_id
        .as_deref()
        .expect("expected default unit selection");
    assert!(
        default_inspector
            .unit_options
            .iter()
            .find(|option| option.unit_id == default_unit_id)
            .is_some_and(|option| option.is_selected),
        "expected default unit option to be marked selected"
    );
    assert!(!default_inspector.has_stale_unit_selection);

    let unit_inspector =
        snapshot.result_inspector_with_unit(Some("stream-heated"), None, Some("heater-1"));
    assert_eq!(unit_inspector.selected_unit_id.as_deref(), Some("heater-1"));
    let selected_unit = unit_inspector
        .selected_unit
        .as_ref()
        .expect("expected selected unit execution result");
    assert_eq!(selected_unit.unit_id, "heater-1");
    assert_eq!(selected_unit.status_label, "Converged");
    assert!(
        selected_unit
            .consumed_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-feed"),
        "expected unit execution result to expose consumed stream actions"
    );
    assert!(
        selected_unit
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-feed"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")
                && stream.summary.contains("H ")),
        "expected unit execution result to expose consumed stream numeric summary"
    );
    assert!(
        selected_unit
            .produced_stream_actions
            .iter()
            .any(|action| action.command_id == "inspector.focus_stream:stream-heated"),
        "expected unit execution result to expose produced stream actions"
    );
    assert!(
        selected_unit
            .produced_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-heated"
                && stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F ")),
        "expected unit execution result to expose produced stream numeric summary"
    );
    assert!(
        unit_inspector
            .unit_related_steps
            .iter()
            .all(|step| step.unit_id == "heater-1"),
        "expected unit-related steps to be filtered to the selected unit"
    );
    assert!(
        unit_inspector
            .unit_related_diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .related_unit_ids
                .iter()
                .any(|unit_id| unit_id == "heater-1")
                || diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|stream_id| stream_id == "stream-heated")),
        "expected unit-related diagnostics to include the selected unit's outputs"
    );
    assert!(
        unit_inspector
            .unit_related_diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .related_unit_ids
                .iter()
                .any(|unit_id| unit_id == "feed-1")
                && diagnostic
                    .related_stream_ids
                    .iter()
                    .any(|stream_id| stream_id == "stream-feed")),
        "expected unit-related diagnostics to include consumed stream context"
    );
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Diagnostic"
            && action.action.command_id == "inspector.focus_stream:stream-feed"
    }));
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Selected unit"
            && action.action.command_id == "inspector.focus_unit:heater-1"
    }));
    assert!(unit_inspector.unit_diagnostic_actions.iter().any(|action| {
        action.source_label == "Solve step"
            && action.action.command_id == "inspector.focus_unit:heater-1"
    }));
    assert!(!unit_inspector.has_stale_unit_selection);

    let stale_unit =
        snapshot.result_inspector_with_unit(Some("stream-heated"), None, Some("missing-unit"));
    assert!(stale_unit.has_stale_unit_selection);
    assert_eq!(
        stale_unit.selected_unit_id.as_deref(),
        snapshot.steps.first().map(|step| step.unit_id.as_str())
    );
}

#[test]
fn studio_gui_window_model_surfaces_official_two_phase_flash_outlet_enthalpy_in_runtime_snapshot() {
    let config = synced_example_config("feed-cooler-flash-binary-hydrocarbon.rfproj.json");
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let _ = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected run dispatch");
    let window = dispatch.window;
    let snapshot = window
        .runtime
        .latest_solve_snapshot
        .expect("expected latest solve snapshot");

    assert_eq!(snapshot.status_label, "Converged");
    assert_eq!(snapshot.stream_count, 4);
    assert_eq!(snapshot.step_count, 3);
    assert_eq!(snapshot.diagnostic_count, 4);

    let liquid_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-liquid")
        .expect("expected liquid outlet stream");
    assert!(liquid_stream.total_molar_flow_mol_s > 0.0);
    assert!(liquid_stream.composition_text.contains("methane="));
    assert!(liquid_stream.molar_enthalpy_j_per_mol.is_some());
    assert!(liquid_stream.summary_rows.iter().any(|row| {
        row.label == "H" && row.detail_label == "Molar enthalpy" && row.value.ends_with(" J/mol")
    }));
    assert!(liquid_stream.phase_rows.iter().any(|row| {
        row.label == "overall"
            && row.phase_fraction_text == "1.0000"
            && row.molar_flow_text == liquid_stream.molar_flow_text
            && row
                .molar_enthalpy_text
                .as_deref()
                .is_some_and(|value| value.ends_with(" J/mol"))
    }));
    assert!(liquid_stream.phase_rows.iter().any(|row| {
        row.label == "liquid"
            && row.phase_fraction_text == "1.0000"
            && row
                .molar_enthalpy_text
                .as_deref()
                .is_some_and(|value| value.ends_with(" J/mol"))
    }));

    let vapor_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-vapor")
        .expect("expected vapor outlet stream");
    assert!(vapor_stream.total_molar_flow_mol_s > 0.0);
    assert!(vapor_stream.composition_text.contains("methane="));
    assert!(vapor_stream.molar_enthalpy_j_per_mol.is_some());
    assert!(vapor_stream.summary_rows.iter().any(|row| {
        row.label == "H" && row.detail_label == "Molar enthalpy" && row.value.ends_with(" J/mol")
    }));
    assert!(vapor_stream.phase_rows.iter().any(|row| {
        row.label == "overall"
            && row.phase_fraction_text == "1.0000"
            && row.molar_flow_text == vapor_stream.molar_flow_text
            && row
                .molar_enthalpy_text
                .as_deref()
                .is_some_and(|value| value.ends_with(" J/mol"))
    }));
    assert!(vapor_stream.phase_rows.iter().any(|row| {
        row.label == "vapor"
            && row.phase_fraction_text == "1.0000"
            && row
                .molar_enthalpy_text
                .as_deref()
                .is_some_and(|value| value.ends_with(" J/mol"))
    }));

    let phase_comparison = snapshot
        .result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"))
        .comparison
        .expect("expected phase outlet comparison model");
    assert!(phase_comparison.summary_rows.iter().any(|row| {
        row.label == "H"
            && row.detail_label == "Molar enthalpy"
            && row.base_value.ends_with(" J/mol")
            && row.compared_value.ends_with(" J/mol")
            && row.delta_text.ends_with(" J/mol")
    }));
    assert!(phase_comparison.phase_rows.iter().any(|row| {
        row.phase_label == "overall"
            && row.base_fraction_text == "1.0000"
            && row.compared_fraction_text == "1.0000"
            && row.fraction_delta_text == "+0.0000"
            && row.base_molar_flow_text.ends_with(" mol/s")
            && row.compared_molar_flow_text.ends_with(" mol/s")
            && row.molar_flow_delta_text.ends_with(" mol/s")
            && row.base_molar_enthalpy_text.ends_with(" J/mol")
            && row.compared_molar_enthalpy_text.ends_with(" J/mol")
            && row.molar_enthalpy_delta_text.ends_with(" J/mol")
    }));
    assert!(phase_comparison.phase_rows.iter().any(|row| {
        row.phase_label == "liquid"
            && row.base_molar_flow_text.ends_with(" mol/s")
            && row.compared_molar_flow_text == "-"
            && row.molar_flow_delta_text == "-"
            && row.base_molar_enthalpy_text.ends_with(" J/mol")
            && row.compared_molar_enthalpy_text == "-"
            && row.molar_enthalpy_delta_text == "-"
    }));
    assert!(phase_comparison.phase_rows.iter().any(|row| {
        row.phase_label == "vapor"
            && row.base_molar_flow_text == "-"
            && row.compared_molar_flow_text.ends_with(" mol/s")
            && row.molar_flow_delta_text == "-"
            && row.base_molar_enthalpy_text == "-"
            && row.compared_molar_enthalpy_text.ends_with(" J/mol")
            && row.molar_enthalpy_delta_text == "-"
    }));

    let active_detail = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-liquid".to_string(),
        })
        .expect("expected liquid inspector target dispatch")
        .window
        .runtime
        .active_inspector_detail
        .expect("expected active liquid stream inspector detail");
    let active_stream = active_detail
        .latest_stream_result
        .as_ref()
        .expect("expected active inspector latest stream result");
    assert_eq!(active_stream.stream_id, "stream-liquid");
    assert!(active_stream.molar_enthalpy_j_per_mol.is_some());
    assert!(active_stream.summary_rows.iter().any(|row| {
        row.label == "H" && row.detail_label == "Molar enthalpy" && row.value.ends_with(" J/mol")
    }));
}

#[test]
fn studio_gui_window_model_surfaces_bubble_dew_window_in_stream_inspectors() {
    let snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
    ));

    let cooled_stream = snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-cooled")
        .expect("expected cooled stream");
    let cooled_window = cooled_stream
        .bubble_dew_window
        .as_ref()
        .expect("expected cooled stream bubble/dew window");
    assert_eq!(cooled_window.phase_region, "two_phase");
    assert!(cooled_window.dew_pressure_pa < cooled_stream.pressure_pa);
    assert!(cooled_window.bubble_pressure_pa > cooled_stream.pressure_pa);
    assert!(cooled_window.bubble_temperature_k < cooled_stream.temperature_k);
    assert!(cooled_window.dew_temperature_k > cooled_stream.temperature_k);

    let liquid_inspector = snapshot.result_inspector(Some("stream-liquid"));
    let liquid_stream = liquid_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected liquid stream");
    let liquid_window = liquid_stream
        .bubble_dew_window
        .clone()
        .expect("expected liquid stream bubble/dew window");
    assert_eq!(liquid_window.phase_region, "two_phase");
    assert_close(
        liquid_window.bubble_pressure_pa,
        liquid_stream.pressure_pa,
        1e-6,
    );
    assert_eq!(
        liquid_window.bubble_pressure_text,
        liquid_stream.pressure_text
    );
    assert_close(
        liquid_window.bubble_temperature_k,
        liquid_stream.temperature_k,
        1e-4,
    );
    assert_eq!(
        liquid_window.bubble_temperature_text,
        liquid_stream.temperature_text
    );
    assert!(liquid_window.dew_pressure_pa < liquid_window.bubble_pressure_pa);
    assert!(liquid_window.dew_temperature_k > liquid_window.bubble_temperature_k);

    let vapor_inspector = snapshot.result_inspector(Some("stream-vapor"));
    let vapor_stream = vapor_inspector
        .selected_stream
        .as_ref()
        .expect("expected selected vapor stream");
    let vapor_window = vapor_stream
        .bubble_dew_window
        .as_ref()
        .expect("expected vapor stream bubble/dew window");
    assert_eq!(vapor_window.phase_region, "two_phase");
    assert_close(vapor_window.dew_pressure_pa, vapor_stream.pressure_pa, 1e-6);
    assert_eq!(vapor_window.dew_pressure_text, vapor_stream.pressure_text);
    assert_close(
        vapor_window.dew_temperature_k,
        vapor_stream.temperature_k,
        1e-4,
    );
    assert_eq!(
        vapor_window.dew_temperature_text,
        vapor_stream.temperature_text
    );
    assert!(vapor_window.bubble_pressure_pa > vapor_window.dew_pressure_pa);
    assert!(vapor_window.bubble_temperature_k < vapor_window.dew_temperature_k);

    let active_detail = stream_target_detail_model(&snapshot, "stream-liquid", "Liquid Outlet");
    let active_window = active_detail
        .latest_stream_result
        .as_ref()
        .and_then(|stream| stream.bubble_dew_window.as_ref())
        .expect("expected active stream inspector bubble/dew window");
    assert_eq!(active_window, &liquid_window);
}

#[test]
fn studio_gui_window_model_preserves_non_flash_intermediate_window_dto_from_ui_snapshot() {
    for (project_json, stream_id, title) in [
        (
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "Heated Outlet",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "Cooled Outlet",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "Valve Outlet",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "Mixer Outlet",
        ),
    ] {
        let (ui_snapshot, window_snapshot) =
            solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
                project_json,
                &build_official_binary_hydrocarbon_provider(),
                |_| {},
            );
        assert_window_model_preserves_ui_stream_window(
            &ui_snapshot,
            &window_snapshot,
            stream_id,
            title,
        );
    }
}

#[test]
fn studio_gui_window_model_preserves_flowing_flash_outlet_window_dto_from_ui_snapshot() {
    let (ui_snapshot, window_snapshot) =
        solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            &build_official_binary_hydrocarbon_provider(),
            |_| {},
        );

    for (stream_id, title) in [
        ("stream-liquid", "Liquid Outlet"),
        ("stream-vapor", "Vapor Outlet"),
    ] {
        assert_window_model_preserves_ui_stream_window(
            &ui_snapshot,
            &window_snapshot,
            stream_id,
            title,
        );
    }
}

#[test]
fn studio_gui_window_model_preserves_snapshot_stream_models_in_unit_view_for_non_flash_intermediates()
 {
    for (project_json, selected_stream_id, selected_unit_id) in [
        (
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "heater-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "cooler-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "valve-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "mixer-1",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_lite_snapshot(project_json);
        assert_result_inspector_with_unit_preserves_snapshot_stream(
            &snapshot,
            selected_stream_id,
            selected_unit_id,
        );
    }
}

#[test]
fn studio_gui_window_model_preserves_snapshot_stream_models_in_unit_and_comparison_views_for_flowing_flash_outlets()
 {
    let snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
    ));

    for selected_stream_id in ["stream-liquid", "stream-vapor"] {
        assert_result_inspector_with_unit_preserves_snapshot_stream(
            &snapshot,
            selected_stream_id,
            "flash-1",
        );
    }

    assert_result_inspector_with_comparison_preserves_snapshot_streams(
        &snapshot,
        "stream-liquid",
        "stream-vapor",
    );
}

#[test]
fn studio_gui_window_model_preserves_single_phase_flash_outlet_window_absence_in_stream_inspectors()
{
    const REFERENCE_TEMPERATURE_K: f64 = 300.0;
    const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
    const BOUNDARY_DELTA_K: f64 = 0.001;
    let overall_mole_fractions = [0.25, 0.75];

    let liquid_only_provider = build_synthetic_provider([0.8, 0.6], REFERENCE_PRESSURE_PA);
    let liquid_only_window = estimate_bubble_dew_window(
        &liquid_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected liquid-only boundary window");
    let liquid_only_snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"),
        &liquid_only_provider,
        |project| {
            for stream_id in ["stream-feed-a", "stream-feed-b"] {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    overall_mole_fractions,
                    liquid_only_window.bubble_temperature_k - BOUNDARY_DELTA_K,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    );

    let liquid_only_liquid = liquid_only_snapshot
        .result_inspector(Some("stream-liquid"))
        .selected_stream
        .expect("expected selected liquid stream");
    let liquid_only_liquid_window = liquid_only_liquid
        .bubble_dew_window
        .as_ref()
        .expect("expected liquid-only liquid outlet bubble/dew window");
    assert_eq!(liquid_only_liquid_window.phase_region, "liquid_only");
    assert!(liquid_only_liquid.total_molar_flow_mol_s > 0.0);

    let liquid_only_vapor = liquid_only_snapshot
        .result_inspector(Some("stream-vapor"))
        .selected_stream
        .expect("expected selected vapor stream");
    assert!(liquid_only_vapor.total_molar_flow_mol_s.abs() < 1e-12);
    assert!(liquid_only_vapor.bubble_dew_window.is_none());
    assert!(liquid_only_vapor.phase_rows.is_empty());

    let liquid_only_vapor_detail =
        stream_target_detail_model(&liquid_only_snapshot, "stream-vapor", "Vapor Outlet");
    assert!(
        liquid_only_vapor_detail
            .latest_stream_result
            .as_ref()
            .is_some_and(|stream| stream.bubble_dew_window.is_none())
    );

    let vapor_only_provider = build_synthetic_provider([1.8, 1.3], REFERENCE_PRESSURE_PA);
    let vapor_only_window = estimate_bubble_dew_window(
        &vapor_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected vapor-only boundary window");
    let vapor_only_snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"),
        &vapor_only_provider,
        |project| {
            for stream_id in ["stream-feed-a", "stream-feed-b"] {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    overall_mole_fractions,
                    vapor_only_window.dew_temperature_k + BOUNDARY_DELTA_K,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    );

    let vapor_only_liquid = vapor_only_snapshot
        .result_inspector(Some("stream-liquid"))
        .selected_stream
        .expect("expected selected liquid stream");
    assert!(vapor_only_liquid.total_molar_flow_mol_s.abs() < 1e-12);
    assert!(vapor_only_liquid.bubble_dew_window.is_none());
    assert!(vapor_only_liquid.phase_rows.is_empty());

    let vapor_only_liquid_detail =
        stream_target_detail_model(&vapor_only_snapshot, "stream-liquid", "Liquid Outlet");
    assert!(
        vapor_only_liquid_detail
            .latest_stream_result
            .as_ref()
            .is_some_and(|stream| stream.bubble_dew_window.is_none())
    );

    let vapor_only_vapor = vapor_only_snapshot
        .result_inspector(Some("stream-vapor"))
        .selected_stream
        .expect("expected selected vapor stream");
    let vapor_only_vapor_window = vapor_only_vapor
        .bubble_dew_window
        .as_ref()
        .expect("expected vapor-only vapor outlet bubble/dew window");
    assert_eq!(vapor_only_vapor_window.phase_region, "vapor_only");
    assert!(vapor_only_vapor.total_molar_flow_mol_s > 0.0);

    let vapor_only_vapor_detail =
        stream_target_detail_model(&vapor_only_snapshot, "stream-vapor", "Vapor Outlet");
    assert_eq!(
        vapor_only_vapor_detail
            .latest_stream_result
            .as_ref()
            .and_then(|stream| stream.bubble_dew_window.as_ref()),
        Some(vapor_only_vapor_window)
    );

    assert_result_inspector_with_unit_preserves_snapshot_stream(
        &liquid_only_snapshot,
        "stream-liquid",
        "flash-1",
    );
    assert_result_inspector_with_unit_preserves_snapshot_stream(
        &liquid_only_snapshot,
        "stream-vapor",
        "flash-1",
    );
    assert_result_inspector_with_comparison_preserves_snapshot_streams(
        &liquid_only_snapshot,
        "stream-liquid",
        "stream-vapor",
    );

    assert_result_inspector_with_unit_preserves_snapshot_stream(
        &vapor_only_snapshot,
        "stream-liquid",
        "flash-1",
    );
    assert_result_inspector_with_unit_preserves_snapshot_stream(
        &vapor_only_snapshot,
        "stream-vapor",
        "flash-1",
    );
    assert_result_inspector_with_comparison_preserves_snapshot_streams(
        &vapor_only_snapshot,
        "stream-liquid",
        "stream-vapor",
    );
}

#[test]
fn studio_gui_window_model_preserves_single_phase_outlet_window_presence_and_absence_from_ui_snapshot()
 {
    const LOCAL_REFERENCE_TEMPERATURE_K: f64 = 300.0;
    const LOCAL_REFERENCE_PRESSURE_PA: f64 = 100_000.0;
    const LOCAL_BOUNDARY_DELTA_K: f64 = 0.001;
    let overall_mole_fractions = [0.25, 0.75];

    let liquid_only_provider = build_synthetic_provider([0.8, 0.6], LOCAL_REFERENCE_PRESSURE_PA);
    let liquid_only_boundary = estimate_bubble_dew_window(
        &liquid_only_provider,
        LOCAL_REFERENCE_TEMPERATURE_K,
        LOCAL_REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected liquid-only boundary window");
    let (liquid_only_ui_snapshot, liquid_only_window_snapshot) =
        solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
            ),
            &liquid_only_provider,
            |project| {
                for stream_id in ["stream-feed-a", "stream-feed-b"] {
                    apply_stream_state_and_composition(
                        project,
                        stream_id,
                        overall_mole_fractions,
                        liquid_only_boundary.bubble_temperature_k - LOCAL_BOUNDARY_DELTA_K,
                        LOCAL_REFERENCE_PRESSURE_PA,
                    );
                }
            },
        );
    assert_window_model_preserves_ui_stream_window(
        &liquid_only_ui_snapshot,
        &liquid_only_window_snapshot,
        "stream-liquid",
        "Liquid Outlet",
    );
    assert_window_model_preserves_ui_stream_window_absence(
        &liquid_only_ui_snapshot,
        &liquid_only_window_snapshot,
        "stream-vapor",
        "Vapor Outlet",
    );

    let vapor_only_provider = build_synthetic_provider([1.8, 1.3], LOCAL_REFERENCE_PRESSURE_PA);
    let vapor_only_boundary = estimate_bubble_dew_window(
        &vapor_only_provider,
        LOCAL_REFERENCE_TEMPERATURE_K,
        LOCAL_REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected vapor-only boundary window");
    let (vapor_only_ui_snapshot, vapor_only_window_snapshot) =
        solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
            ),
            &vapor_only_provider,
            |project| {
                for stream_id in ["stream-feed-a", "stream-feed-b"] {
                    apply_stream_state_and_composition(
                        project,
                        stream_id,
                        overall_mole_fractions,
                        vapor_only_boundary.dew_temperature_k + LOCAL_BOUNDARY_DELTA_K,
                        LOCAL_REFERENCE_PRESSURE_PA,
                    );
                }
            },
        );
    assert_window_model_preserves_ui_stream_window_absence(
        &vapor_only_ui_snapshot,
        &vapor_only_window_snapshot,
        "stream-liquid",
        "Liquid Outlet",
    );
    assert_window_model_preserves_ui_stream_window(
        &vapor_only_ui_snapshot,
        &vapor_only_window_snapshot,
        "stream-vapor",
        "Vapor Outlet",
    );
}

#[test]
fn studio_gui_window_model_preserves_source_stream_bubble_dew_window_from_ui_snapshot() {
    for (project_json, stream_ids) in [
        (
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed-a", "stream-feed-b"][..],
        ),
    ] {
        let provider = build_official_binary_hydrocarbon_provider();
        let (ui_snapshot, window_snapshot) =
            solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
                project_json,
                &provider,
                |_| {},
            );

        for stream_id in stream_ids {
            let title = find_window_snapshot_stream(&window_snapshot, stream_id)
                .label
                .clone();
            assert_window_model_preserves_ui_stream_window(
                &ui_snapshot,
                &window_snapshot,
                stream_id,
                title.as_str(),
            );
        }
    }
}

#[test]
fn studio_gui_window_model_surfaces_bubble_dew_window_for_non_flash_intermediate_streams() {
    let heater_snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ));
    assert_stream_window_visible_in_result_and_active_inspectors(
        &heater_snapshot,
        "stream-heated",
        "Heated Outlet",
        |stream, window| {
            assert_eq!(window.phase_region, "vapor_only");
            assert!(stream.pressure_pa < window.dew_pressure_pa);
            assert!(stream.temperature_k > window.dew_temperature_k);
        },
    );

    let cooler_snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
    ));
    assert_stream_window_visible_in_result_and_active_inspectors(
        &cooler_snapshot,
        "stream-cooled",
        "Cooled Outlet",
        |stream, window| {
            assert_eq!(window.phase_region, "two_phase");
            assert!(window.dew_pressure_pa < stream.pressure_pa);
            assert!(window.bubble_pressure_pa > stream.pressure_pa);
            assert!(window.bubble_temperature_k < stream.temperature_k);
            assert!(window.dew_temperature_k > stream.temperature_k);
        },
    );

    let valve_snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
    ));
    assert_stream_window_visible_in_result_and_active_inspectors(
        &valve_snapshot,
        "stream-throttled",
        "Valve Outlet",
        |stream, window| {
            assert_eq!(window.phase_region, "two_phase");
            assert!(window.dew_pressure_pa < stream.pressure_pa);
            assert!(window.bubble_pressure_pa > stream.pressure_pa);
            assert!(window.bubble_temperature_k < stream.temperature_k);
            assert!(window.dew_temperature_k > stream.temperature_k);
        },
    );

    let mixer_snapshot = solve_binary_hydrocarbon_lite_snapshot(include_str!(
        "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
    ));
    assert_stream_window_visible_in_result_and_active_inspectors(
        &mixer_snapshot,
        "stream-mix-out",
        "Mixer Outlet",
        |stream, window| {
            assert_eq!(window.phase_region, "two_phase");
            assert!(window.dew_pressure_pa < stream.pressure_pa);
            assert!(window.bubble_pressure_pa > stream.pressure_pa);
            assert!(window.bubble_temperature_k < stream.temperature_k);
            assert!(window.dew_temperature_k > stream.temperature_k);
        },
    );
}

#[test]
fn studio_gui_window_model_surfaces_non_flash_intermediate_stream_summary_and_context() {
    for (project_json, stream_id, title, producer_unit_id) in [
        (
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "Heated Outlet",
            "heater-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "Cooled Outlet",
            "cooler-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "Valve Outlet",
            "valve-1",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "Mixer Outlet",
            "mixer-1",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_lite_snapshot(project_json);
        assert_non_flash_intermediate_stream_summary_and_context(
            &snapshot,
            stream_id,
            title,
            producer_unit_id,
        );
    }
}

#[test]
fn studio_gui_window_model_surfaces_non_flash_intermediate_unit_summary_and_context() {
    for (
        project_json,
        selected_stream_id,
        unit_id,
        title,
        consumed_stream_ids,
        produced_stream_id,
    ) in [
        (
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "heater-1",
            "Heater",
            &["stream-feed"][..],
            "stream-heated",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "cooler-1",
            "Cooler",
            &["stream-feed"][..],
            "stream-cooled",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "valve-1",
            "Valve",
            &["stream-feed"][..],
            "stream-throttled",
        ),
        (
            include_str!(
                "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "mixer-1",
            "Mixer",
            &["stream-feed-a", "stream-feed-b"][..],
            "stream-mix-out",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_lite_snapshot(project_json);
        assert_non_flash_intermediate_unit_summary_and_context(
            &snapshot,
            selected_stream_id,
            unit_id,
            title,
            consumed_stream_ids,
            produced_stream_id,
        );
    }
}

#[test]
fn studio_gui_window_model_preserves_official_near_boundary_flash_chain_consumers_from_ui_snapshot()
{
    let provider = build_official_binary_hydrocarbon_provider();

    for scenario in official_binary_hydrocarbon_near_boundary_consumer_scenarios() {
        let (ui_snapshot, window_snapshot) =
            solve_ui_and_window_snapshot_from_project_with_provider_and_edit(
                scenario.project_json,
                &provider,
                |project| {
                    apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(
                        project, &scenario,
                    );
                },
            );

        for stream_id in scenario.source_stream_ids {
            let title = find_window_snapshot_stream(&window_snapshot, stream_id)
                .label
                .clone();
            assert_window_model_preserves_ui_stream_window(
                &ui_snapshot,
                &window_snapshot,
                stream_id,
                title.as_str(),
            );
        }

        let inlet_stream = find_ui_snapshot_stream(&ui_snapshot, scenario.flash_inlet_stream_id);
        assert_eq!(
            inlet_stream
                .bubble_dew_window
                .as_ref()
                .expect("expected flash inlet bubble/dew window")
                .phase_region,
            scenario.case.expected_phase_region,
            "{}",
            scenario.case.label
        );
        assert_window_model_preserves_ui_stream_window(
            &ui_snapshot,
            &window_snapshot,
            scenario.flash_inlet_stream_id,
            scenario.flash_inlet_title,
        );
        assert_flash_consumer_preserves_snapshot_stream_reference(
            &window_snapshot,
            scenario.flash_inlet_stream_id,
        );

        match scenario.case.expected_phase_region {
            PhaseEquilibriumRegion::LiquidOnly => {
                assert_window_model_preserves_ui_stream_window(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-liquid",
                    "Liquid Outlet",
                );
                assert_window_model_preserves_ui_stream_window_absence(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-vapor",
                    "Vapor Outlet",
                );
            }
            PhaseEquilibriumRegion::TwoPhase => {
                assert_window_model_preserves_ui_stream_window(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-liquid",
                    "Liquid Outlet",
                );
                assert_window_model_preserves_ui_stream_window(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-vapor",
                    "Vapor Outlet",
                );
            }
            PhaseEquilibriumRegion::VaporOnly => {
                assert_window_model_preserves_ui_stream_window_absence(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-liquid",
                    "Liquid Outlet",
                );
                assert_window_model_preserves_ui_stream_window(
                    &ui_snapshot,
                    &window_snapshot,
                    "stream-vapor",
                    "Vapor Outlet",
                );
            }
        }
    }
}

#[test]
fn studio_gui_window_model_surfaces_official_near_boundary_flash_unit_step_and_comparison_semantics()
 {
    let provider = build_official_binary_hydrocarbon_provider();

    for scenario in official_binary_hydrocarbon_near_boundary_consumer_scenarios() {
        let snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
            scenario.project_json,
            &provider,
            |project| {
                apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(
                    project, &scenario,
                );
            },
        );

        for selected_stream_id in ["stream-liquid", "stream-vapor"] {
            assert_result_inspector_with_unit_preserves_snapshot_stream(
                &snapshot,
                selected_stream_id,
                "flash-1",
            );
            assert_flash_step_and_unit_preserve_outlet_summaries(&snapshot, selected_stream_id);
        }

        assert_result_inspector_with_comparison_preserves_snapshot_streams(
            &snapshot,
            "stream-liquid",
            "stream-vapor",
        );
        assert_flash_outlet_comparison_matches_snapshot_stream_models(&snapshot);
    }
}

#[test]
fn studio_gui_window_model_surfaces_official_near_boundary_flash_focus_actions_and_diagnostic_targets()
 {
    let provider = build_official_binary_hydrocarbon_provider();

    for scenario in official_binary_hydrocarbon_near_boundary_consumer_scenarios() {
        let snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
            scenario.project_json,
            &provider,
            |project| {
                apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(
                    project, &scenario,
                );
            },
        );

        let flash_unit_focus_command = "inspector.focus_unit:flash-1";
        let flash_inlet_focus_command =
            format!("inspector.focus_stream:{}", scenario.flash_inlet_stream_id);
        let liquid_focus_command = "inspector.focus_stream:stream-liquid";
        let vapor_focus_command = "inspector.focus_stream:stream-vapor";

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
            .expect("expected downstream flash step for near-boundary inlet");
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
                .any(|action| action.command_id == liquid_focus_command)
        );
        assert!(
            flash_related_step
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == vapor_focus_command)
        );

        let comparison_inspector =
            snapshot.result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"));
        let comparison = comparison_inspector
            .comparison
            .as_ref()
            .expect("expected near-boundary flash outlet comparison");
        assert_eq!(
            comparison.base_stream_focus_action.command_id,
            liquid_focus_command
        );
        assert_eq!(
            comparison.compared_stream_focus_action.command_id,
            vapor_focus_command
        );

        let unit_inspector = snapshot.result_inspector_with_unit(
            Some("stream-liquid"),
            Some("stream-vapor"),
            Some("flash-1"),
        );
        let selected_unit = unit_inspector
            .selected_unit
            .as_ref()
            .expect("expected near-boundary flash unit result");
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
                .any(|action| action.command_id == liquid_focus_command)
        );
        assert!(
            selected_unit
                .produced_stream_actions
                .iter()
                .any(|action| action.command_id == vapor_focus_command)
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
    }
}

#[test]
fn studio_gui_window_model_dispatches_official_near_boundary_flash_focus_commands_to_active_inspector()
 {
    for scenario in official_binary_hydrocarbon_near_boundary_consumer_scenarios() {
        let (config, project_path) = official_near_boundary_consumer_synced_config(&scenario);
        let mut driver = StudioGuiDriver::new(&config).expect("expected near-boundary driver");
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let run = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.run_manual".to_string(),
            })
            .expect("expected near-boundary run dispatch");
        let snapshot = run
            .window
            .runtime
            .latest_solve_snapshot
            .expect("expected near-boundary solve snapshot");

        let comparison = snapshot
            .result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"))
            .comparison
            .expect("expected near-boundary comparison");
        let liquid_detail = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: comparison.base_stream_focus_action.command_id.clone(),
            })
            .expect("expected liquid focus dispatch")
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active liquid inspector detail");
        let liquid_stream = liquid_detail
            .latest_stream_result
            .as_ref()
            .expect("expected active liquid stream result");
        assert_eq!(liquid_stream.stream_id, "stream-liquid");
        assert_eq!(
            liquid_stream.bubble_dew_window.is_some(),
            scenario.case.expected_phase_region != PhaseEquilibriumRegion::VaporOnly,
            "{}",
            scenario.case.label
        );

        let vapor_detail = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: comparison.compared_stream_focus_action.command_id.clone(),
            })
            .expect("expected vapor focus dispatch")
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active vapor inspector detail");
        let vapor_stream = vapor_detail
            .latest_stream_result
            .as_ref()
            .expect("expected active vapor stream result");
        assert_eq!(vapor_stream.stream_id, "stream-vapor");
        assert_eq!(
            vapor_stream.bubble_dew_window.is_some(),
            scenario.case.expected_phase_region != PhaseEquilibriumRegion::LiquidOnly,
            "{}",
            scenario.case.label
        );

        let unit_inspector = snapshot.result_inspector_with_unit(
            Some("stream-liquid"),
            Some("stream-vapor"),
            Some("flash-1"),
        );
        let inlet_diagnostic_action = unit_inspector
            .unit_diagnostic_actions
            .iter()
            .find(|action| {
                action.source_label == "Diagnostic"
                    && action.action.command_id
                        == format!("inspector.focus_stream:{}", scenario.flash_inlet_stream_id)
            })
            .expect("expected flash inlet diagnostic action");
        let inlet_detail = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: inlet_diagnostic_action.action.command_id.clone(),
            })
            .expect("expected flash inlet diagnostic dispatch")
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active flash inlet inspector detail");
        assert_eq!(
            inlet_detail.target.target_id,
            scenario.flash_inlet_stream_id
        );
        assert_eq!(
            inlet_detail
                .latest_stream_result
                .as_ref()
                .map(|stream| stream.stream_id.as_str()),
            Some(scenario.flash_inlet_stream_id)
        );

        let unit_focus_command = unit_inspector
            .unit_diagnostic_actions
            .iter()
            .find(|action| {
                action.source_label == "Selected unit"
                    && action.action.command_id == "inspector.focus_unit:flash-1"
            })
            .expect("expected flash unit diagnostic focus action")
            .action
            .command_id
            .clone();
        let unit_detail = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: unit_focus_command,
            })
            .expect("expected flash unit focus dispatch")
            .window
            .runtime
            .active_inspector_detail
            .expect("expected active flash unit detail");
        let active_unit = unit_detail
            .latest_unit_result
            .as_ref()
            .expect("expected active flash unit result");
        assert_eq!(unit_detail.target.target_id, "flash-1");
        assert_eq!(active_unit.unit_id, "flash-1");
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
                .any(|stream| stream.stream_id == "stream-liquid")
        );
        assert!(
            active_unit
                .produced_stream_results
                .iter()
                .any(|stream| stream.stream_id == "stream-vapor")
        );

        let _ = fs::remove_file(project_path);
    }
}

#[test]
fn studio_gui_window_model_surfaces_failure_result_until_rerun_succeeds() {
    let mut driver =
        StudioGuiDriver::new(&unbound_outlet_failure_synced_config()).expect("expected driver");
    let _ = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let failed = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected failed run dispatch");
    assert_eq!(
        failed.window.runtime.control_state.run_status,
        rf_ui::RunStatus::Error
    );
    assert_eq!(failed.window.runtime.latest_solve_snapshot, None);
    let failure = failed
        .window
        .runtime
        .latest_failure
        .expect("expected visible failure result");
    assert_eq!(failure.status_label, "Error");
    assert_eq!(failure.title, "Unbound outlet port");
    assert!(
        failure.message.contains("unbound_outlet_port"),
        "expected solver diagnostic in failure message, got {}",
        failure.message
    );
    let diagnostic_detail = failure
        .diagnostic_detail
        .as_ref()
        .expect("expected structured failure diagnostic detail");
    assert_eq!(
        diagnostic_detail.primary_code.as_deref(),
        Some("solver.connection_validation.unbound_outlet_port")
    );
    assert_eq!(diagnostic_detail.document_revision, 0);
    assert_eq!(diagnostic_detail.severity_label, "Error");
    assert_eq!(diagnostic_detail.diagnostic_count, 1);
    assert!(
        diagnostic_detail
            .related_units
            .iter()
            .any(|target| target.target_id == "feed-1"
                && target.action.command_id == "inspector.focus_unit:feed-1")
    );
    assert!(diagnostic_detail.related_ports.iter().any(|target| {
        target.unit_id == "feed-1"
            && target.port_name == "outlet"
            && target.unit_action.command_id == "inspector.focus_unit:feed-1"
    }));
    assert_eq!(failure.recovery_title, Some("Create outlet stream"));
    assert!(failure.recovery_detail.is_some());
    assert_eq!(
        failure
            .recovery_action
            .as_ref()
            .map(|action| (action.label.as_str(), action.command_id.as_str())),
        Some(("Create outlet stream", "run_panel.recover_failure"))
    );
    assert_eq!(
        failure
            .recovery_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "feed-1"))
    );
    assert_eq!(
        failure
            .recovery_target
            .as_ref()
            .map(|target| target.action.command_id.as_str()),
        Some("inspector.focus_unit:feed-1")
    );
    assert!(failure.diagnostic_actions.iter().any(|action| {
        action.source_label == "Recovery"
            && action.target_label == "Run panel"
            && action.action.command_id == "run_panel.recover_failure"
    }));
    assert!(failure.diagnostic_actions.iter().any(|action| {
        action.source_label == "Recovery target"
            && action.target_label == "Unit"
            && action.action.command_id == "inspector.focus_unit:feed-1"
    }));
    assert!(failure.diagnostic_actions.iter().any(|action| {
        action.source_label == "Failure port"
            && action.target_label == "Port"
            && action.summary == "Unit feed-1 port outlet"
            && action.action.command_id == "inspector.focus_unit:feed-1"
    }));
    assert!(failure.latest_log_message.is_some());

    let focused_failure_unit = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_unit:feed-1".to_string(),
        })
        .expect("expected failure target focus dispatch");
    let failure_detail = focused_failure_unit
        .window
        .runtime
        .active_inspector_detail
        .expect("expected active failure unit inspector detail");
    assert!(failure_detail.unit_ports.iter().any(|port| {
        port.name == "outlet"
            && port.attention_summary.as_ref().is_some_and(|summary| {
                summary.contains("port feed-1:outlet")
                    && summary.contains("solver.connection_validation.unbound_outlet_port")
            })
    }));

    let recovery = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.recover_failure".to_string(),
        })
        .expect("expected recovery dispatch");
    assert_eq!(
        recovery
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "feed-1"))
    );
    let detail = recovery
        .window
        .runtime
        .active_inspector_detail
        .expect("expected active unit inspector detail");
    assert_eq!(detail.target.target_id, "feed-1");
    assert!(
        detail
            .summary_rows
            .iter()
            .any(|row| row.label == "Kind" && row.value == "feed")
    );
    assert!(detail.unit_ports.iter().any(|port| port.name == "outlet"
        && port.stream_id.as_ref().is_some_and(|stream_id| {
            port.stream_action
                .as_ref()
                .map(|action| action.command_id == format!("inspector.focus_stream:{stream_id}"))
                .unwrap_or(false)
        })));
    assert!(
        detail
            .unit_ports
            .iter()
            .all(|port| port.attention_summary.is_none()),
        "expected stale failure attention to stay off the post-recovery unit port list"
    );
    let rerun = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.resume_workspace".to_string(),
        })
        .expect("expected successful rerun dispatch");

    assert_eq!(
        rerun.window.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(rerun.window.runtime.latest_failure, None);
    let snapshot = rerun
        .window
        .runtime
        .latest_solve_snapshot
        .expect("expected solve snapshot after recovery rerun");
    let inspector = snapshot.result_inspector(None);
    assert!(inspector.selected_stream.is_some());
    assert!(!inspector.has_stale_selection);
}

#[test]
fn studio_gui_window_model_surfaces_failure_stream_context_from_document_state() {
    let mut driver =
        StudioGuiDriver::new(&missing_upstream_failure_synced_config()).expect("expected driver");
    let _ = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let failed = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected failed run dispatch");
    let failure = failed
        .window
        .runtime
        .latest_failure
        .expect("expected visible failure result");
    let diagnostic_detail = failure
        .diagnostic_detail
        .as_ref()
        .expect("expected structured failure diagnostic detail");

    assert_eq!(
        diagnostic_detail.primary_code.as_deref(),
        Some("solver.connection_validation.missing_upstream_source")
    );
    assert!(
        diagnostic_detail
            .related_streams
            .iter()
            .any(|target| target.target_id == "stream-feed-a"),
        "expected failure diagnostic to keep related stream target buttons"
    );
    assert!(
        diagnostic_detail
            .related_stream_results
            .iter()
            .any(|stream| {
                stream.stream_id == "stream-feed-a"
                    && stream.summary.contains("T ")
                    && stream.summary.contains("P ")
                    && stream.summary.contains("F ")
                    && stream.summary.contains("z:")
                    && !stream.summary.contains("H ")
            }),
        "expected failure diagnostic to expose document-state stream numeric context"
    );
    assert!(diagnostic_detail.related_ports.iter().any(|target| {
        target.unit_id == "mixer-1"
            && target.port_name == "inlet_a"
            && target.unit_action.command_id == "inspector.focus_unit:mixer-1"
            && target.stream_result.as_ref().is_some_and(|stream| {
                stream.stream_id == "stream-feed-a" && stream.summary.contains("z:")
            })
    }));
}

#[test]
fn studio_gui_window_command_area_surfaces_palette_items_through_shared_model() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window = dispatch.snapshot.window_model();
    let palette_items = window.commands.palette_items("diagnostic");

    assert_eq!(
        palette_items
            .into_iter()
            .map(|item| (item.command_id, item.label, item.menu_path_text))
            .collect::<Vec<_>>(),
        vec![(
            "run_panel.recover_failure".to_string(),
            "Recover run panel failure (F8) [disabled]".to_string(),
            "Run > Recovery > Recover Run Panel Failure".to_string(),
        )]
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn studio_gui_window_command_area_surfaces_toolbar_sections_through_shared_model() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window = dispatch.snapshot.window_model();

    assert_eq!(
        window
            .commands
            .toolbar_sections
            .iter()
            .map(|section| section.title)
            .collect::<Vec<_>>(),
        vec![
            "File",
            "Edit",
            "Run Panel",
            "Recovery",
            "Entitlement",
            "Canvas"
        ]
    );
    assert_eq!(
        window.commands.toolbar_sections[2]
            .items
            .iter()
            .map(|item| item.command_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "run_panel.run_manual",
            "run_panel.resume_workspace",
            "run_panel.set_hold",
            "run_panel.set_active",
        ]
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn studio_gui_window_command_area_surfaces_command_list_sections_through_shared_model() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window = dispatch.snapshot.window_model();

    assert_eq!(
        window
            .commands
            .command_list_sections
            .iter()
            .map(|section| section.title)
            .collect::<Vec<_>>(),
        vec![
            "File",
            "Edit",
            "Run Panel",
            "Recovery",
            "Entitlement",
            "Canvas"
        ]
    );
    assert_eq!(
        window.commands.command_list_sections[2]
            .items
            .iter()
            .map(|item| item.command_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "run_panel.run_manual",
            "run_panel.resume_workspace",
            "run_panel.set_hold",
            "run_panel.set_active",
        ]
    );
    assert!(
        window.commands.command_list_sections[2].items[0]
            .label
            .contains("F5")
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn studio_gui_window_model_reports_parked_timer_after_last_window_closes() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer dispatch");
    let _ = driver
        .dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id })
        .expect("expected close dispatch");

    let window = driver.snapshot().window_model();

    assert_eq!(window.header.registered_window_count, 0);
    assert_eq!(window.header.foreground_window_id, None);
    assert_eq!(window.header.entitlement_timer_owner_window_id, None);
    assert!(window.header.has_parked_entitlement_timer);
    assert!(window.header.status_line.contains("timer owner: parked"));
    assert_eq!(
        window.layout_state.scope.kind,
        StudioGuiWindowLayoutScopeKind::EmptyWorkspace
    );
    assert_eq!(window.layout_state.scope.layout_key, "studio.window.empty");
    assert_eq!(window.drop_preview, None);
}

#[test]
fn studio_gui_window_model_surfaces_preview_layout_presentation() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected preview dispatch");

    let window = driver.window_model_for_window(Some(window_id));
    let preview = window.drop_preview.expect("expected preview model");
    assert_eq!(preview.overlay.drag_area_id, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        preview.overlay.target_dock_region,
        StudioGuiWindowDockRegion::LeftSidebar
    );
    assert_eq!(preview.overlay.target_stack_group, 10);
    assert_eq!(
        preview.overlay.target_stack_area_ids,
        vec![StudioGuiWindowAreaId::Runtime]
    );
    assert_eq!(
        preview.changed_area_ids,
        vec![
            StudioGuiWindowAreaId::Commands,
            StudioGuiWindowAreaId::Runtime
        ]
    );
    assert_eq!(
        preview
            .preview_layout
            .panel(StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}
