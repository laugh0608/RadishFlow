use super::*;

impl StudioGuiWindowSolveSnapshotModel {
    pub fn light_text_export(&self) -> String {
        let mut lines = vec![
            "RadishFlow Solve Snapshot".to_string(),
            format!("snapshot_id: {}", self.snapshot_id),
            format!("sequence: {}", self.sequence),
            format!("status: {}", self.status_label),
            format!("summary: {}", one_line(&self.summary)),
            format!(
                "counts: streams={}, steps={}, diagnostics={}",
                self.stream_count, self.step_count, self.diagnostic_count
            ),
            String::new(),
            "Streams".to_string(),
            tsv_row([
                "stream_id",
                "label",
                "T",
                "P",
                "F",
                "H",
                "composition",
                "phases",
                "bubble_dew_window",
            ]),
        ];

        for stream in &self.streams {
            lines.push(tsv_row([
                stream.stream_id.as_str(),
                stream.label.as_str(),
                stream.temperature_text.as_str(),
                stream.pressure_text.as_str(),
                stream.molar_flow_text.as_str(),
                stream.molar_enthalpy_text.as_deref().unwrap_or("-"),
                stream.composition_text.as_str(),
                stream.phase_text.as_str(),
                stream
                    .bubble_dew_window
                    .as_ref()
                    .map(format_bubble_dew_window)
                    .unwrap_or_else(|| "-".to_string())
                    .as_str(),
            ]));
        }

        lines.extend([
            String::new(),
            "Steps".to_string(),
            tsv_row([
                "index",
                "unit_id",
                "status",
                "summary",
                "consumed_streams",
                "produced_streams",
            ]),
        ]);
        for step in &self.steps {
            lines.push(tsv_row([
                step.index.to_string().as_str(),
                step.unit_id.as_str(),
                step.execution_status_label,
                step.summary.as_str(),
                format_stream_references(&step.consumed_stream_results).as_str(),
                format_stream_references(&step.produced_stream_results).as_str(),
            ]));
        }

        lines.extend([
            String::new(),
            "Diagnostics".to_string(),
            tsv_row(["severity", "code", "message", "units", "streams"]),
        ]);
        for diagnostic in &self.diagnostics {
            lines.push(tsv_row([
                diagnostic.severity_label,
                diagnostic.code.as_str(),
                diagnostic.message.as_str(),
                diagnostic.related_unit_ids.join(", ").as_str(),
                diagnostic.related_stream_ids.join(", ").as_str(),
            ]));
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

fn tsv_row<'a>(cells: impl IntoIterator<Item = &'a str>) -> String {
    cells
        .into_iter()
        .map(one_line)
        .collect::<Vec<_>>()
        .join("\t")
}

fn one_line(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn format_bubble_dew_window(window: &StudioGuiWindowBubbleDewWindowModel) -> String {
    format!(
        "phase_region={}; bubble_pressure={}; dew_pressure={}; bubble_temperature={}; dew_temperature={}",
        window.phase_region,
        window.bubble_pressure_text,
        window.dew_pressure_text,
        window.bubble_temperature_text,
        window.dew_temperature_text
    )
}

fn format_stream_references(streams: &[StudioGuiWindowStreamResultReferenceModel]) -> String {
    streams
        .iter()
        .map(|stream| format!("{} ({})", stream.stream_id, one_line(&stream.summary)))
        .collect::<Vec<_>>()
        .join("; ")
}
