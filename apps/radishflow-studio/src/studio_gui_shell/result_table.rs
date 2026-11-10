use super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_bottom_results_table(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        super::result_review::flow(ui, self.locale, &window.runtime.result_review);
        let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() else {
            if let Some(failure) = window.runtime.latest_failure.as_ref() {
                self.render_latest_failure_summary(ui, failure);
                return;
            }
            if let Some(stale_snapshot) = window.runtime.stale_solve_snapshot.as_ref() {
                self.render_stale_solve_snapshot_notice(ui, stale_snapshot);
                return;
            }
            ui.small(self.locale.text(ShellText::NoVisibleSolveResults));
            return;
        };
        let current = snapshot
            .result_guard
            .as_ref()
            .is_some_and(|guard| guard.is_current(&self.platform_host.modeling_inputs().workspace));
        if snapshot.streams.is_empty() {
            ui.small(self.locale.text(ShellText::NoStreamResults));
            return;
        }
        egui::Grid::new(format!("bottom-results-table:{}", snapshot.snapshot_id))
            .num_columns(6)
            .striped(true)
            .min_col_width(78.0)
            .show(ui, |ui| {
                ui.strong(result_table_header(self.locale, ResultTableHeader::Stream));
                ui.strong("T");
                ui.strong("P");
                ui.strong("F");
                ui.strong("H");
                ui.strong(result_table_header(self.locale, ResultTableHeader::Phase));
                ui.end_row();
                for stream in &snapshot.streams {
                    let response = ui
                        .add_enabled(current, egui::Button::new(&stream.label).frame(false))
                        .on_hover_text(&stream.stream_id);
                    if response.clicked()
                        && snapshot.result_guard.as_ref().is_some_and(|guard| {
                            guard.is_current(&self.platform_host.modeling_inputs().workspace)
                        })
                    {
                        self.focus_result_table_stream(
                            &snapshot.snapshot_id,
                            stream.stream_id.clone(),
                        );
                    }
                    ui.label(&stream.temperature_text);
                    ui.label(&stream.pressure_text);
                    ui.label(&stream.molar_flow_text);
                    ui.label(stream.molar_enthalpy_text.as_deref().unwrap_or("-"));
                    ui.label(result_table_phase_summary(self.locale, stream))
                        .on_hover_text(&stream.phase_text);
                    ui.end_row();
                }
            });

        if !snapshot.review_summary.unit_results.is_empty() {
            ui.add_space(8.0);
            ui.strong(self.locale.text(ShellText::Units));
            egui::Grid::new(format!(
                "bottom-results-units-table:{}",
                snapshot.snapshot_id
            ))
            .num_columns(5)
            .striped(true)
            .min_col_width(78.0)
            .show(ui, |ui| {
                ui.strong(result_table_header(self.locale, ResultTableHeader::Unit));
                ui.strong(result_table_header(self.locale, ResultTableHeader::Status));
                ui.strong(result_table_header(self.locale, ResultTableHeader::Step));
                ui.strong(self.locale.text(ShellText::InspectorConsumedStreams));
                ui.strong(self.locale.text(ShellText::InspectorProducedStreams));
                ui.end_row();

                for unit in &snapshot.review_summary.unit_results {
                    let unit_response = ui
                        .add_enabled(current, egui::Button::new(&unit.unit_id).frame(false))
                        .on_hover_text(&unit.summary);
                    if unit_response.clicked()
                        && snapshot.result_guard.as_ref().is_some_and(|guard| {
                            guard.is_current(&self.platform_host.modeling_inputs().workspace)
                        })
                    {
                        self.focus_result_table_unit(&snapshot.snapshot_id, unit.unit_id.clone());
                    }
                    ui.label(self.locale.runtime_label(unit.status_label).as_ref());
                    ui.label(unit.location.to_string());
                    render_wrapped_small(ui, result_table_stream_ids(&unit.consumed_stream_ids));
                    render_wrapped_small(ui, result_table_stream_ids(&unit.produced_stream_ids));
                    ui.end_row();
                }
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ResultTableHeader {
    Stream,
    Unit,
    Status,
    Step,
    Phase,
}

fn result_table_header(locale: StudioShellLocale, header: ResultTableHeader) -> &'static str {
    match locale {
        StudioShellLocale::En => match header {
            ResultTableHeader::Stream => "Stream",
            ResultTableHeader::Unit => "Unit",
            ResultTableHeader::Status => "Status",
            ResultTableHeader::Step => "Step",
            ResultTableHeader::Phase => "Phase",
        },
        StudioShellLocale::ZhCn => match header {
            ResultTableHeader::Stream => "流股",
            ResultTableHeader::Unit => "单元",
            ResultTableHeader::Status => "状态",
            ResultTableHeader::Step => "步骤",
            ResultTableHeader::Phase => "相态",
        },
    }
}

fn result_table_stream_ids(stream_ids: &[String]) -> String {
    if stream_ids.is_empty() {
        return "-".to_string();
    }

    stream_ids.join(", ")
}

fn result_table_phase_summary(
    locale: StudioShellLocale,
    stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
) -> String {
    if stream.phase_rows.is_empty() {
        return locale.text(ShellText::NoneValue).to_string();
    }

    let visible_rows = stream
        .phase_rows
        .iter()
        .filter(|row| row.label != "overall")
        .collect::<Vec<_>>();
    let visible_rows = if visible_rows.is_empty() {
        stream.phase_rows.iter().collect::<Vec<_>>()
    } else {
        visible_rows
    };
    let hidden_count = visible_rows.len().saturating_sub(2);
    let mut parts = visible_rows
        .iter()
        .take(2)
        .map(|row| {
            format!(
                "{} {}",
                locale.runtime_label(&row.label),
                row.phase_fraction_text
            )
        })
        .collect::<Vec<_>>();
    if hidden_count > 0 {
        parts.push(format!("+{hidden_count}"));
    }
    parts.join(" / ")
}
