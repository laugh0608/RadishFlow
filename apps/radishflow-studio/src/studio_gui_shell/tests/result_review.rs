use super::*;

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
