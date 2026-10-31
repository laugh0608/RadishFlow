use super::*;

impl StudioGuiWindowSolveSnapshotModel {
    pub fn light_text_export(&self) -> Result<String, StudioGuiQuantityPresentationError> {
        if let Some(error) = self.presentation_errors.first() {
            return Err(error.clone());
        }
        let mut lines = vec![
            "RadishFlow Solve Snapshot".to_string(),
            format!("snapshot_id: {}", self.snapshot_id),
            format!("sequence: {}", self.sequence),
            format!("document_revision: {}", self.document_revision),
            format!(
                "display_units: {}",
                rf_types::units::LEGACY_QUANTITIES
                    .iter()
                    .map(|quantity| {
                        format!(
                            "{}={}",
                            quantity.definition().id,
                            self.display_units.unit_for(*quantity).definition().id
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "number_format: 6 significant digits; decimal point; display text, not lossless SI"
                .into(),
            format!("status: {}", self.status_label),
            format!("summary: {}", one_line(&self.summary)),
            format!(
                "counts: streams={}, steps={}, diagnostics={}",
                self.stream_count, self.step_count, self.diagnostic_count
            ),
            String::new(),
            "Review".to_string(),
            tsv_row(["category", "items"]),
            tsv_row([
                "source_streams",
                format_stream_references(&self.review_summary.source_stream_results).as_str(),
            ]),
            tsv_row([
                "intermediate_streams",
                format_stream_references(&self.review_summary.intermediate_stream_results).as_str(),
            ]),
            tsv_row([
                "terminal_streams",
                format_stream_references(&self.review_summary.terminal_stream_results).as_str(),
            ]),
            tsv_row([
                "units",
                format_review_units(&self.review_summary.unit_results).as_str(),
            ]),
            tsv_row([
                "diagnostics",
                self.review_summary.diagnostic_count.to_string().as_str(),
            ]),
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

        // Append provenance without changing the established numerical tables.
        let provenance = crate::flow_origin_lines(self.origin.as_ref());
        let mut source_lines = vec!["Execution origin".to_string()];
        source_lines.extend(
            provenance
                .into_iter()
                .map(|(key, value)| format!("{key}: {}", one_line(&value))),
        );

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
            "Units".to_string(),
            tsv_row([
                "unit_id",
                "step",
                "status",
                "summary",
                "consumed_streams",
                "produced_streams",
            ]),
        ]);
        for step in latest_unit_steps(&self.steps) {
            lines.push(tsv_row([
                step.unit_id.as_str(),
                step.index.to_string().as_str(),
                step.execution_status_label,
                step.summary.as_str(),
                format_stream_references(&step.consumed_stream_results).as_str(),
                format_stream_references(&step.produced_stream_results).as_str(),
            ]));
        }

        if let Some(eo) = &self.equation_oriented {
            for unit in &eo.units {
                lines.push(tsv_row([
                    unit.unit_id.as_str(),
                    unit.location.to_string().as_str(),
                    unit.status_label,
                    unit.summary.as_str(),
                    format_stream_references(&unit.consumed_stream_results).as_str(),
                    format_stream_references(&unit.produced_stream_results).as_str(),
                ]));
            }
            lines.extend(eo.evidence_lines.clone());
        } else if let Some(partitioned) = &self.partitioned {
            for unit in &partitioned.units {
                lines.push(tsv_row([
                    unit.unit_id.as_str(),
                    unit.location.to_string().as_str(),
                    unit.status_label,
                    unit.summary.as_str(),
                    format_stream_references(&unit.consumed_stream_results).as_str(),
                    format_stream_references(&unit.produced_stream_results).as_str(),
                ]));
            }
            lines.extend(partitioned_export_lines(&partitioned.result));
        } else if let Some(simultaneous) = &self.simultaneous {
            for unit in &simultaneous.units {
                lines.push(tsv_row([
                    unit.unit_id.as_str(),
                    unit.location.to_string().as_str(),
                    unit.status_label,
                    unit.summary.as_str(),
                    format_stream_references(&unit.consumed_stream_results).as_str(),
                    format_stream_references(&unit.produced_stream_results).as_str(),
                ]));
            }
            lines.extend(simultaneous.evidence_lines.clone());
        } else {
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
        lines.push(String::new());
        lines.extend(source_lines);
        Ok(lines.join("\n"))
    }
}

fn latest_unit_steps(
    steps: &[StudioGuiWindowSolveStepModel],
) -> Vec<&StudioGuiWindowSolveStepModel> {
    let mut unit_steps: Vec<&StudioGuiWindowSolveStepModel> = Vec::new();
    for step in steps {
        if let Some(index) = unit_steps
            .iter()
            .position(|existing| existing.unit_id == step.unit_id)
        {
            unit_steps[index] = step;
        } else {
            unit_steps.push(step);
        }
    }
    unit_steps
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

fn format_review_units(units: &[StudioGuiWindowResultReviewUnitModel]) -> String {
    units
        .iter()
        .map(|unit| {
            format!(
                "{} status={} consumed=[{}] produced=[{}]",
                unit.unit_id,
                unit.status_label,
                unit.consumed_stream_ids.join(", "),
                unit.produced_stream_ids.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn partitioned_export_lines(
    result: &rf_solver::partitioned::PartitionedSolveResult,
) -> Vec<String> {
    let mut lines = vec![
        String::new(),
        "Dependency blocks".into(),
        "Fixed T/P material closure; full energy solution is not provided.".into(),
    ];
    for (index, block) in result.blocks.iter().enumerate() {
        lines.push(format!(
            "block {}: {:?} · {}",
            index + 1,
            block.kind,
            block
                .members
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        if let Some(r) = &block.recycle {
            lines.push(format!(
                "tear: {}.{} → {} → {}.{}",
                r.tear.source.unit_id,
                r.tear.source.port_name,
                r.tear.stream_id,
                r.tear.sink.unit_id,
                r.tear.sink.port_name
            ));
            lines.push(format!(
                "boundary: {} K / {} Pa; iterations: {}; independent verification: {}",
                r.boundary.temperature_k,
                r.boundary.pressure_pa,
                r.iteration_evaluations,
                r.verification_evaluations
            ));
            lines.push(format!("tolerances: component {:.6e} mol/s; scaled {:.6e}; total {:.6e} mol/s; temperature {:.6e} K; pressure {:.6e} Pa",r.tolerances.component_absolute_flow_mol_s,r.tolerances.scaled_flow,r.tolerances.total_absolute_flow_mol_s,r.tolerances.temperature_k,r.tolerances.pressure_pa));
            for (id, residual) in &r.residuals_mol_s {
                lines.push(format!(
                    "{id}: residual {residual:.9e} mol/s; scale {:.9e} mol/s",
                    r.component_scales_mol_s[id]
                ));
            }
            for (kind, stream) in [("assumed", &r.assumed), ("returned", &r.returned)] {
                lines.push(format!(
                    "{kind}: {} | T {} K | P {} Pa | F {:.9e} mol/s",
                    stream.id,
                    stream.temperature_k,
                    stream.pressure_pa,
                    stream.total_molar_flow_mol_s
                ));
                for (id, x) in &stream.overall_mole_fractions {
                    lines.push(format!(
                        "{kind} {id}: {:.9e} mol/s",
                        x * stream.total_molar_flow_mol_s
                    ));
                }
            }
            for sample in &r.residual_trace {
                lines.push(format!(
                    "{:?} {}: raw {:.9e} mol/s; scaled {:.9e}",
                    sample.phase, sample.evaluation, sample.max_absolute_mol_s, sample.max_scaled
                ));
            }
        }
    }
    let c = &result.costs;
    lines.push(format!(
        "actual calls: local {}; thermo {}; flash {}; constructions {}; elapsed {:.6e} s",
        c.local_unit_calls,
        c.thermo_calls,
        c.flash_calls,
        c.operation_constructions,
        c.elapsed.as_secs_f64()
    ));
    lines
}
