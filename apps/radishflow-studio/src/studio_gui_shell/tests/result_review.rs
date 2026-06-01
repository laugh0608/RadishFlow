use super::*;

const RESULT_REVIEW_TOLERANCE: f64 = 1e-9;

pub(super) fn assert_flash_split_material_balance(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    inlet_stream_id: &str,
    liquid_stream_id: &str,
    vapor_stream_id: &str,
) {
    let inlet = snapshot_stream(snapshot, inlet_stream_id);
    let liquid = snapshot_stream(snapshot, liquid_stream_id);
    let vapor = snapshot_stream(snapshot, vapor_stream_id);

    assert_close(
        liquid.total_molar_flow_mol_s + vapor.total_molar_flow_mol_s,
        inlet.total_molar_flow_mol_s,
        "flash outlet total molar flow should match inlet flow",
    );
    for component in &inlet.composition_rows {
        let inlet_component_flow = inlet.total_molar_flow_mol_s * component.fraction;
        let outlet_component_flow = liquid.total_molar_flow_mol_s
            * stream_fraction(liquid, component.component_id.as_str())
            + vapor.total_molar_flow_mol_s
                * stream_fraction(vapor, component.component_id.as_str());
        assert_close(
            outlet_component_flow,
            inlet_component_flow,
            format!(
                "flash split should preserve component `{}` molar flow",
                component.component_id
            ),
        );
    }
}

pub(super) fn assert_flash_outlet_phase_review_semantics(
    app: &mut ReadyAppState,
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    liquid_stream_id: &str,
    vapor_stream_id: &str,
    selected_unit_id: Option<&str>,
) {
    let liquid = snapshot_stream(snapshot, liquid_stream_id);
    let vapor = snapshot_stream(snapshot, vapor_stream_id);
    let mut zero_flow_outlet_count = 0;

    for stream in [liquid, vapor] {
        let is_zero_flow = stream.total_molar_flow_mol_s.abs() <= RESULT_REVIEW_TOLERANCE;
        if is_zero_flow {
            zero_flow_outlet_count += 1;
            assert_eq!(
                stream.molar_enthalpy_j_per_mol, None,
                "zero-flow flash outlet `{}` must not synthesize molar enthalpy",
                stream.stream_id
            );
            assert_eq!(
                stream.molar_enthalpy_text, None,
                "zero-flow flash outlet `{}` must not synthesize enthalpy text",
                stream.stream_id
            );
            assert!(
                stream.phase_rows.is_empty(),
                "zero-flow flash outlet `{}` must keep phase rows absent",
                stream.stream_id
            );
            assert!(
                stream.bubble_dew_window.is_none(),
                "zero-flow flash outlet `{}` must keep bubble/dew window absent",
                stream.stream_id
            );

            let texts = render_result_inspector_review_texts(
                app,
                snapshot,
                stream.stream_id.as_str(),
                selected_unit_id,
            );
            let bubble_dew_header = app.locale.text(ShellText::BubbleDewWindow);
            assert!(
                !texts.iter().any(|text| text.contains(bubble_dew_header)),
                "expected result inspector to omit bubble/dew window for zero-flow outlet `{}`; rendered texts: {:?}",
                stream.stream_id,
                texts
            );
            assert!(
                texts.iter().any(|text| text == "none"),
                "expected result inspector to render `none` phase summary for zero-flow outlet `{}`; rendered texts: {:?}",
                stream.stream_id,
                texts
            );
        } else {
            assert!(
                stream.molar_enthalpy_j_per_mol.is_some(),
                "flowing flash outlet `{}` should carry molar enthalpy",
                stream.stream_id
            );
            assert!(
                stream.molar_enthalpy_text.is_some(),
                "flowing flash outlet `{}` should carry enthalpy text",
                stream.stream_id
            );
            assert!(
                !stream.phase_rows.is_empty(),
                "flowing flash outlet `{}` should carry phase rows",
                stream.stream_id
            );
            assert!(
                stream.bubble_dew_window.is_some(),
                "flowing flash outlet `{}` should carry bubble/dew window",
                stream.stream_id
            );
        }
    }

    assert!(
        zero_flow_outlet_count > 0,
        "expected current blank-project flash cases to cover a single-phase zero-flow outlet"
    );
}

pub(super) fn assert_single_inlet_unit_result_consistency(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    inlet_stream_id: &str,
    outlet_stream_id: &str,
    expected_outlet_temperature_k: f64,
    expected_outlet_pressure_pa: f64,
) {
    let inlet = snapshot_stream(snapshot, inlet_stream_id);
    let outlet = snapshot_stream(snapshot, outlet_stream_id);

    assert_close(
        outlet.total_molar_flow_mol_s,
        inlet.total_molar_flow_mol_s,
        "single-inlet unit outlet flow should match inlet flow",
    );
    assert_close(
        outlet.temperature_k,
        expected_outlet_temperature_k,
        "single-inlet unit outlet temperature should match submitted parameter or inlet carry-through",
    );
    assert_close(
        outlet.pressure_pa,
        expected_outlet_pressure_pa,
        "single-inlet unit outlet pressure should match submitted parameter",
    );
    for component in &inlet.composition_rows {
        assert_close(
            stream_fraction(outlet, component.component_id.as_str()),
            component.fraction,
            format!(
                "single-inlet unit should preserve component `{}` composition",
                component.component_id
            ),
        );
    }
}

pub(super) fn assert_mixer_weighted_result(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    inlet_stream_ids: &[&str],
    outlet_stream_id: &str,
    expected_outlet_pressure_pa: f64,
) {
    let outlet = snapshot_stream(snapshot, outlet_stream_id);
    let inlets = inlet_stream_ids
        .iter()
        .map(|stream_id| snapshot_stream(snapshot, stream_id))
        .collect::<Vec<_>>();
    let total_inlet_flow = inlets
        .iter()
        .map(|stream| stream.total_molar_flow_mol_s)
        .sum::<f64>();

    assert_close(
        outlet.total_molar_flow_mol_s,
        total_inlet_flow,
        "mixer outlet total molar flow should equal inlet flow sum",
    );
    assert_close(
        outlet.pressure_pa,
        expected_outlet_pressure_pa,
        "mixer outlet pressure should match submitted parameter",
    );
    for component in &outlet.composition_rows {
        let weighted_component_flow = inlets
            .iter()
            .map(|stream| {
                stream.total_molar_flow_mol_s
                    * stream_fraction(stream, component.component_id.as_str())
            })
            .sum::<f64>();
        assert_close(
            component.fraction,
            weighted_component_flow / total_inlet_flow,
            format!(
                "mixer outlet component `{}` should use flow-weighted composition",
                component.component_id
            ),
        );
    }
}

pub(super) fn assert_case_review_summary_covers_flow(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    source_stream_ids: &[&str],
    intermediate_stream_ids: &[&str],
    terminal_stream_ids: &[&str],
    unit_ids: &[&str],
) {
    let review = &snapshot.review_summary;
    assert_eq!(
        review.status_label, snapshot.status_label,
        "review summary should preserve snapshot status"
    );
    assert_eq!(
        review.diagnostic_count, snapshot.diagnostic_count,
        "review summary should preserve diagnostic count"
    );
    assert_review_stream_group(
        &review.source_stream_results,
        source_stream_ids,
        "source streams",
    );
    assert_review_stream_group(
        &review.intermediate_stream_results,
        intermediate_stream_ids,
        "intermediate streams",
    );
    assert_review_stream_group(
        &review.terminal_stream_results,
        terminal_stream_ids,
        "terminal streams",
    );
    assert_eq!(
        review
            .unit_results
            .iter()
            .map(|unit| unit.unit_id.as_str())
            .collect::<Vec<_>>(),
        unit_ids,
        "review summary should preserve latest unit execution order"
    );
    for unit in &review.unit_results {
        let step = snapshot
            .steps
            .iter()
            .rev()
            .find(|step| step.unit_id == unit.unit_id)
            .unwrap_or_else(|| panic!("expected latest step for unit `{}`", unit.unit_id));
        assert_eq!(unit.status_label, step.execution_status_label);
        assert_eq!(
            unit.consumed_stream_ids,
            step.consumed_stream_results
                .iter()
                .map(|stream| stream.stream_id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            unit.produced_stream_ids,
            step.produced_stream_results
                .iter()
                .map(|stream| stream.stream_id.clone())
                .collect::<Vec<_>>()
        );
    }
}

pub(super) fn assert_bottom_result_table_contains_streams_and_steps(
    app: &mut ReadyAppState,
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    stream_ids: &[&str],
    unit_ids: &[&str],
) {
    let texts = render_bottom_results_table_texts(app);
    for expected in [
        "流股",
        "T (K)",
        "P (Pa)",
        "F (mol/s)",
        "H (J/mol)",
        "相态",
        "单元",
        "消费流股",
        "产出流股",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected bottom result table to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    for stream_id in stream_ids {
        let stream = snapshot_stream(snapshot, stream_id);
        let expected_values = vec![
            stream.label.clone(),
            format!("{:.2}", stream.temperature_k),
            format!("{:.0}", stream.pressure_pa),
            format!("{:.6}", stream.total_molar_flow_mol_s),
        ];
        for expected in expected_values {
            assert!(
                texts.iter().any(|text| text.contains(&expected)),
                "expected bottom result table to render `{expected}` for `{stream_id}`, rendered texts: {:?}",
                texts
            );
        }
        if let Some(molar_enthalpy) = stream.molar_enthalpy_j_per_mol {
            let expected = format!("{molar_enthalpy:.3}");
            assert!(
                texts.iter().any(|text| text.contains(&expected)),
                "expected bottom result table to render enthalpy `{expected}` for `{stream_id}`, rendered texts: {:?}",
                texts
            );
        }
    }

    for unit_id in unit_ids {
        assert!(
            texts.iter().any(|text| text.contains(unit_id)),
            "expected bottom result table to render unit `{unit_id}`, rendered texts: {:?}",
            texts
        );
        let step = snapshot
            .steps
            .iter()
            .find(|step| step.unit_id == *unit_id)
            .unwrap_or_else(|| panic!("expected step for unit {unit_id}"));
        for stream_ref in step
            .consumed_stream_results
            .iter()
            .chain(step.produced_stream_results.iter())
        {
            assert!(
                texts
                    .iter()
                    .any(|text| text.contains(&stream_ref.stream_id)),
                "expected bottom result table to render step stream reference `{}` for `{unit_id}`, rendered texts: {:?}",
                stream_ref.stream_id,
                texts
            );
        }
    }
}

pub(super) fn assert_result_inspector_renders_stream_review_object(
    app: &mut ReadyAppState,
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    selected_unit_id: Option<&str>,
) {
    let stream = snapshot_stream(snapshot, stream_id);
    assert!(
        !stream.composition_rows.is_empty(),
        "expected `{stream_id}` to carry composition rows"
    );
    assert!(
        stream.bubble_dew_window.is_some() || stream.total_molar_flow_mol_s == 0.0,
        "expected flowing stream `{stream_id}` to carry a bubble/dew window"
    );

    let texts = render_result_inspector_review_texts(app, snapshot, stream_id, selected_unit_id);
    for expected in [
        stream.stream_id.as_str(),
        stream.temperature_text.as_str(),
        stream.pressure_text.as_str(),
        stream.molar_flow_text.as_str(),
        app.locale.text(ShellText::OverallComposition),
        app.locale.text(ShellText::PhaseResults),
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected result inspector to render `{expected}` for `{stream_id}`, rendered texts: {:?}",
            texts
        );
    }
    if let Some(molar_enthalpy_text) = stream.molar_enthalpy_text.as_deref() {
        assert!(
            texts.iter().any(|text| text.contains(molar_enthalpy_text)),
            "expected result inspector to render enthalpy `{molar_enthalpy_text}` for `{stream_id}`, rendered texts: {:?}",
            texts
        );
    }
    if let Some(window) = stream.bubble_dew_window.as_ref() {
        assert!(window.bubble_pressure_pa.is_finite());
        assert!(window.dew_pressure_pa.is_finite());
        assert!(window.bubble_temperature_k.is_finite());
        assert!(window.dew_temperature_k.is_finite());
        let expected = app.locale.text(ShellText::BubbleDewWindow);
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected result inspector to render bubble/dew window entry for `{stream_id}`, rendered texts: {:?}",
            texts
        );
    }
    for row in &stream.composition_rows {
        for expected in [row.component_id.as_str(), row.fraction_text.as_str()] {
            assert!(
                texts.iter().any(|text| text.contains(expected)),
                "expected result inspector to render composition `{expected}` for `{stream_id}`, rendered texts: {:?}",
                texts
            );
        }
    }
    for row in &stream.phase_rows {
        for expected in [row.label.as_str(), row.phase_fraction_text.as_str()] {
            assert!(
                texts.iter().any(|text| text.contains(expected)),
                "expected result inspector to render phase `{expected}` for `{stream_id}`, rendered texts: {:?}",
                texts
            );
        }
    }
}

pub(super) fn assert_result_inspector_renders_unit_stream_references(
    app: &mut ReadyAppState,
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    unit_id: &str,
) {
    let inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some(unit_id));
    let unit = inspector
        .selected_unit
        .as_ref()
        .unwrap_or_else(|| panic!("expected result inspector unit {unit_id}"));
    assert!(
        !unit.consumed_stream_results.is_empty(),
        "expected unit `{unit_id}` to carry consumed stream references"
    );
    assert!(
        !unit.produced_stream_results.is_empty(),
        "expected unit `{unit_id}` to carry produced stream references"
    );

    let texts =
        render_result_inspector_review_texts(app, snapshot, selected_stream_id, Some(unit_id));
    let header = app.locale.text(ShellText::ResultUnitView);
    assert!(
        texts.iter().any(|text| text.contains(header)),
        "expected result inspector to render unit view entry `{header}` for `{unit_id}`, rendered texts: {:?}",
        texts
    );

    let unit_texts = render_unit_execution_result_texts(app, unit);
    for expected in [
        app.locale.text(ShellText::InspectorConsumedStreams),
        app.locale.text(ShellText::InspectorProducedStreams),
        unit_id,
    ] {
        assert!(
            unit_texts.iter().any(|text| text.contains(expected)),
            "expected result inspector unit surface to render `{expected}` for `{unit_id}`, rendered texts: {:?}",
            unit_texts
        );
    }
    for stream_ref in unit
        .consumed_stream_results
        .iter()
        .chain(unit.produced_stream_results.iter())
    {
        assert!(
            unit_texts
                .iter()
                .any(|text| text.contains(&stream_ref.stream_id)),
            "expected result inspector unit surface to render stream reference `{}` for `{unit_id}`, rendered texts: {:?}",
            stream_ref.stream_id,
            unit_texts
        );
    }
}

fn assert_review_stream_group(
    streams: &[radishflow_studio::StudioGuiWindowStreamResultReferenceModel],
    expected_stream_ids: &[&str],
    group_name: &str,
) {
    assert_eq!(
        streams
            .iter()
            .map(|stream| stream.stream_id.as_str())
            .collect::<Vec<_>>(),
        expected_stream_ids,
        "review summary should classify {group_name}"
    );
    for stream in streams {
        assert_eq!(
            stream.focus_action.command_id,
            format!("inspector.focus_stream:{}", stream.stream_id),
            "review summary stream `{}` should carry focus action",
            stream.stream_id
        );
        assert!(
            stream.summary.contains("T ")
                && stream.summary.contains("P ")
                && stream.summary.contains("F "),
            "review summary stream `{}` should carry numeric summary, got `{}`",
            stream.stream_id,
            stream.summary
        );
    }
}

fn render_bottom_results_table_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 6000.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_bottom_results_table(ui, &window);
            });
        },
    );

    collect_output_texts(&output)
}

fn render_result_inspector_review_texts(
    app: &mut ReadyAppState,
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    selected_unit_id: Option<&str>,
) -> Vec<String> {
    let inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, selected_unit_id);
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 6000.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                open_collapsing_sections(ui, &[app.locale.text(ShellText::ResultUnitView)]);
                ui.push_id(
                    format!(
                        "result-inspector:selected-stream:{}:{}",
                        snapshot.snapshot_id, selected_stream_id
                    ),
                    |ui| {
                        open_collapsing_sections(
                            ui,
                            &[
                                app.locale.text(ShellText::BubbleDewWindow),
                                app.locale.text(ShellText::OverallComposition),
                                app.locale.text(ShellText::PhaseResults),
                            ],
                        );
                    },
                );
                app.render_result_inspector(ui, &inspector);
            });
        },
    );

    collect_output_texts(&output)
}

fn render_unit_execution_result_texts(
    app: &mut ReadyAppState,
    unit: &radishflow_studio::StudioGuiWindowUnitExecutionResultModel,
) -> Vec<String> {
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 6000.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_unit_execution_result_inspector(ui, unit);
            });
        },
    );

    collect_output_texts(&output)
}

fn open_collapsing_sections(ui: &mut egui::Ui, open_section_labels: &[&str]) {
    for label in open_section_labels {
        let id = ui.make_persistent_id(label);
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        state.set_open(true);
        state.store(ui.ctx());
    }
}

fn collect_output_texts(output: &egui::FullOutput) -> Vec<String> {
    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn collect_shape_texts(shape: &egui::epaint::Shape, texts: &mut Vec<String>) {
    match shape {
        egui::epaint::Shape::Text(text) => texts.push(text.galley.job.text.clone()),
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_shape_texts(shape, texts);
            }
        }
        _ => {}
    }
}

fn snapshot_stream<'a>(
    snapshot: &'a radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
) -> &'a radishflow_studio::StudioGuiWindowStreamResultModel {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .unwrap_or_else(|| panic!("expected stream result {stream_id}"))
}

fn stream_fraction(
    stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
    component_id: &str,
) -> f64 {
    stream
        .composition_rows
        .iter()
        .find(|row| row.component_id == component_id)
        .unwrap_or_else(|| {
            panic!(
                "expected stream `{}` to carry component `{component_id}`",
                stream.stream_id
            )
        })
        .fraction
}

fn assert_close(actual: f64, expected: f64, context: impl AsRef<str>) {
    assert!(
        (actual - expected).abs() <= RESULT_REVIEW_TOLERANCE,
        "{}: expected {actual} to equal {expected}",
        context.as_ref()
    );
}
