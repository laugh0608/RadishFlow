use super::*;
use rf_types::units::{ConversionError, DisplayUnitSet, MeasurementUnit, QuantityKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiQuantityPresentationError {
    pub stream_id: String,
    pub quantity: QuantityKind,
    pub unit: MeasurementUnit,
    pub cause: ConversionError,
}

impl std::fmt::Display for StudioGuiQuantityPresentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} / {} / {}: {}",
            self.stream_id,
            self.quantity.definition().id,
            self.unit.definition().symbol,
            self.cause
        )
    }
}

impl std::error::Error for StudioGuiQuantityPresentationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// Keep a current result current even when its display conversion fails. A field
/// exposes the error, and output can reject the collected typed errors atomically.
pub(super) struct ResultValuePresentation<'a> {
    units: &'a DisplayUnitSet,
    pub(super) guard: Option<crate::FlowResultGuard>,
    errors: Vec<StudioGuiQuantityPresentationError>,
}

impl<'a> ResultValuePresentation<'a> {
    fn new(units: &'a DisplayUnitSet) -> Self {
        Self {
            units,
            guard: None,
            errors: Vec::new(),
        }
    }

    fn quantity(&mut self, stream_id: &str, quantity: QuantityKind, si: f64) -> String {
        let unit = self.units.unit_for(quantity);
        match rf_ui::format_quantity(si, quantity, unit) {
            Ok(text) => text,
            Err(cause) => {
                let error = StudioGuiQuantityPresentationError {
                    stream_id: stream_id.into(),
                    quantity,
                    unit,
                    cause,
                };
                let text = format!("Conversion failed: {error}");
                self.errors.push(error);
                text
            }
        }
    }

    fn composition(&mut self, stream_id: &str, values: &[(String, f64)]) -> String {
        if values.is_empty() {
            return "none".into();
        }
        values
            .iter()
            .map(|(id, fraction)| {
                format!(
                    "{id}={}",
                    self.quantity(stream_id, QuantityKind::MoleFraction, *fraction)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn stream(&mut self, stream: &rf_ui::StreamStateSnapshot) -> StudioGuiWindowStreamResultModel {
        use QuantityKind as Q;
        let id = stream.stream_id.as_str();
        let temperature_text = self.quantity(id, Q::AbsoluteTemperature, stream.temperature_k);
        let pressure_text = self.quantity(id, Q::AbsolutePressure, stream.pressure_pa);
        let molar_flow_text = self.quantity(id, Q::MolarFlow, stream.total_molar_flow_mol_s);
        let molar_enthalpy_j_per_mol = overall_molar_enthalpy(&stream.phases);
        let molar_enthalpy_text =
            molar_enthalpy_j_per_mol.map(|value| self.quantity(id, Q::MolarEnthalpy, value));
        let mut summary_rows = vec![
            StudioGuiWindowStreamSummaryRowModel {
                label: "T",
                detail_label: "Temperature",
                value: temperature_text.clone(),
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "P",
                detail_label: "Pressure",
                value: pressure_text.clone(),
            },
            StudioGuiWindowStreamSummaryRowModel {
                label: "F",
                detail_label: "Molar flow",
                value: molar_flow_text.clone(),
            },
        ];
        if let Some(value) = &molar_enthalpy_text {
            summary_rows.push(StudioGuiWindowStreamSummaryRowModel {
                label: "H",
                detail_label: "Molar enthalpy",
                value: value.clone(),
            });
        }
        let composition_rows = stream
            .overall_mole_fractions
            .iter()
            .map(
                |(component_id, fraction)| StudioGuiWindowCompositionResultModel {
                    component_id: component_id.clone(),
                    fraction: *fraction,
                    fraction_text: self.quantity(id, Q::MoleFraction, *fraction),
                },
            )
            .collect::<Vec<_>>();
        let composition_text = if composition_rows.is_empty() {
            "none".into()
        } else {
            composition_rows
                .iter()
                .map(|row| format!("{}={}", row.component_id, row.fraction_text))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let phase_rows = stream
            .phases
            .iter()
            .map(|phase| {
                let molar_flow_mol_s = phase.phase_fraction * stream.total_molar_flow_mol_s;
                StudioGuiWindowPhaseResultModel {
                    label: phase.label.clone(),
                    phase_fraction: phase.phase_fraction,
                    phase_fraction_text: self.quantity(
                        id,
                        Q::MolarPhaseFraction,
                        phase.phase_fraction,
                    ),
                    molar_flow_mol_s,
                    molar_flow_text: self.quantity(id, Q::MolarFlow, molar_flow_mol_s),
                    composition_text: self.composition(id, &phase.composition),
                    molar_enthalpy_j_per_mol: phase.molar_enthalpy_j_per_mol,
                    molar_enthalpy_text: phase
                        .molar_enthalpy_j_per_mol
                        .map(|value| self.quantity(id, Q::MolarEnthalpy, value)),
                }
            })
            .collect::<Vec<_>>();
        let phase_text = if phase_rows.is_empty() {
            "none".into()
        } else {
            phase_rows
                .iter()
                .map(|phase| {
                    format!(
                        "{}={} ({})",
                        phase.label, phase.phase_fraction_text, phase.molar_flow_text
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        StudioGuiWindowStreamResultModel {
            result_guard: self.guard.clone(),
            stream_id: id.into(),
            label: stream.label.clone(),
            temperature_k: stream.temperature_k,
            pressure_pa: stream.pressure_pa,
            total_molar_flow_mol_s: stream.total_molar_flow_mol_s,
            molar_enthalpy_j_per_mol,
            temperature_text,
            pressure_text,
            molar_flow_text,
            molar_enthalpy_text,
            summary_rows,
            composition_rows,
            composition_text,
            phase_rows,
            phase_text,
            bubble_dew_window: stream
                .bubble_dew_window
                .as_ref()
                .map(|window| self.bubble_dew_window(id, window)),
        }
    }

    pub(super) fn reference(
        &mut self,
        stream: &rf_ui::StreamStateSnapshot,
    ) -> StudioGuiWindowStreamResultReferenceModel {
        use QuantityKind as Q;
        let id = stream.stream_id.as_str();
        let enthalpy = overall_molar_enthalpy(&stream.phases)
            .map(|value| self.quantity(id, Q::MolarEnthalpy, value));
        StudioGuiWindowStreamResultReferenceModel {
            stream_id: id.into(),
            summary: stream_result_numeric_summary(
                &self.quantity(id, Q::AbsoluteTemperature, stream.temperature_k),
                &self.quantity(id, Q::AbsolutePressure, stream.pressure_pa),
                &self.quantity(id, Q::MolarFlow, stream.total_molar_flow_mol_s),
                enthalpy.as_deref(),
                None,
            ),
            focus_action: inspector_stream_action(id).with_result_guard(self.guard.as_ref()),
        }
    }

    fn bubble_dew_window(
        &mut self,
        id: &str,
        window: &rf_ui::BubbleDewWindowSnapshot,
    ) -> StudioGuiWindowBubbleDewWindowModel {
        use QuantityKind as Q;
        StudioGuiWindowBubbleDewWindowModel {
            phase_region: window.phase_region.as_str().into(),
            bubble_pressure_pa: window.bubble_pressure_pa,
            dew_pressure_pa: window.dew_pressure_pa,
            bubble_temperature_k: window.bubble_temperature_k,
            dew_temperature_k: window.dew_temperature_k,
            bubble_pressure_text: self.quantity(id, Q::AbsolutePressure, window.bubble_pressure_pa),
            dew_pressure_text: self.quantity(id, Q::AbsolutePressure, window.dew_pressure_pa),
            bubble_temperature_text: self.quantity(
                id,
                Q::AbsoluteTemperature,
                window.bubble_temperature_k,
            ),
            dew_temperature_text: self.quantity(
                id,
                Q::AbsoluteTemperature,
                window.dew_temperature_k,
            ),
        }
    }
}

fn overall_molar_enthalpy(phases: &[rf_ui::PhaseStateSnapshot]) -> Option<f64> {
    phases
        .iter()
        .find(|phase| phase.label == "overall")
        .and_then(|phase| phase.molar_enthalpy_j_per_mol)
}

pub(super) fn stream_result_reference_model_from_window_stream(
    stream: &StudioGuiWindowStreamResultModel,
) -> StudioGuiWindowStreamResultReferenceModel {
    StudioGuiWindowStreamResultReferenceModel {
        stream_id: stream.stream_id.clone(),
        summary: stream_result_numeric_summary(
            &stream.temperature_text,
            &stream.pressure_text,
            &stream.molar_flow_text,
            stream.molar_enthalpy_text.as_deref(),
            Some(&stream.composition_text),
        ),
        focus_action: inspector_stream_action(&stream.stream_id)
            .with_result_guard(stream.result_guard.as_ref()),
    }
}

pub(super) fn stream_result_numeric_summary(
    temperature_text: &str,
    pressure_text: &str,
    molar_flow_text: &str,
    molar_enthalpy_text: Option<&str>,
    composition_text: Option<&str>,
) -> String {
    let mut parts = vec![
        format!("T {temperature_text}"),
        format!("P {pressure_text}"),
        format!("F {molar_flow_text}"),
    ];
    if let Some(value) = molar_enthalpy_text {
        parts.push(format!("H {value}"));
    }
    if let Some(value) = composition_text {
        parts.push(value.to_string());
    }
    parts.join(" | ")
}

pub(super) fn solve_snapshot_model_from_ui(
    snapshot: &rf_ui::FlowResultRecord,
    units: &DisplayUnitSet,
) -> StudioGuiWindowSolveSnapshotModel {
    let mut presentation = ResultValuePresentation::new(units);
    presentation.guard = crate::FlowResultGuard::from_snapshot(snapshot);
    let status_label = run_status_label(snapshot.status);
    let streams = snapshot
        .streams
        .iter()
        .map(|stream| presentation.stream(stream))
        .collect::<Vec<_>>();
    let acyclic_steps = match snapshot {
        rf_ui::FlowResultRecord::Acyclic(s) => s.steps.as_slice(),
        rf_ui::FlowResultRecord::Partitioned(_) | rf_ui::FlowResultRecord::Simultaneous(_) => &[],
    };
    let steps = acyclic_steps
        .iter()
        .map(|step| {
            let produced_stream_results = step
                .streams
                .iter()
                .map(|stream| presentation.reference(stream))
                .collect::<Vec<_>>();
            let produced_streams = step
                .streams
                .iter()
                .map(|stream| stream.stream_id.as_str().to_string())
                .collect::<Vec<_>>();
            let consumed_stream_results = step
                .consumed_streams
                .iter()
                .map(|stream| presentation.reference(stream))
                .collect::<Vec<_>>();
            let consumed_streams = step
                .consumed_streams
                .iter()
                .map(|stream| stream.stream_id.as_str().to_string())
                .collect::<Vec<_>>();
            let unit_action = inspector_unit_action(step.unit_id.as_str())
                .with_result_guard(presentation.guard.as_ref());
            let consumed_stream_actions = consumed_stream_results
                .iter()
                .map(|stream| stream.focus_action.clone())
                .collect::<Vec<_>>();
            let produced_stream_actions = produced_stream_results
                .iter()
                .map(|stream| stream.focus_action.clone())
                .collect::<Vec<_>>();
            let diagnostic_actions = solve_step_diagnostic_actions(
                step.index,
                &step.unit_id,
                &unit_action,
                &consumed_streams,
                &consumed_stream_actions,
                &produced_streams,
                &produced_stream_actions,
            );
            StudioGuiWindowSolveStepModel {
                index: step.index,
                unit_id: step.unit_id.as_str().to_string(),
                summary: step.summary.clone(),
                execution_status_label: run_status_label(step.execution.status),
                consumed_stream_results,
                consumed_stream_actions,
                unit_action,
                produced_stream_results,
                produced_stream_actions,
                diagnostic_actions,
            }
        })
        .collect::<Vec<_>>();
    let diagnostics = snapshot
        .diagnostics
        .iter()
        .map(|diagnostic| {
            diagnostic_model_from_ui(diagnostic, &snapshot.streams, &mut presentation)
        })
        .collect::<Vec<_>>();
    let mut review_summary =
        result_review_summary_model_from_parts(status_label, diagnostics.len(), &streams, &steps);

    let mut partitioned = None;
    let mut simultaneous = None;
    let block_data = match snapshot {
        rf_ui::FlowResultRecord::Acyclic(_) => None,
        rf_ui::FlowResultRecord::Partitioned(p) => Some((
            p.result
                .blocks
                .iter()
                .map(|b| b.members.clone())
                .collect::<Vec<_>>(),
            &p.final_units,
        )),
        rf_ui::FlowResultRecord::Simultaneous(p) => Some((
            p.result
                .blocks
                .iter()
                .map(|b| b.members.clone())
                .collect::<Vec<_>>(),
            &p.final_units,
        )),
    };
    if let Some((block_members, final_units)) = block_data {
        let mut unit_results = Vec::new();
        for (block_index, ids) in block_members.iter().enumerate() {
            for unit_id in ids {
                let values = &final_units[unit_id];
                let consumed_stream_results = values
                    .consumed_streams
                    .iter()
                    .map(|v| presentation.reference(v))
                    .collect::<Vec<_>>();
                let produced_stream_results = values
                    .produced_streams
                    .iter()
                    .map(|v| presentation.reference(v))
                    .collect::<Vec<_>>();
                unit_results.push(StudioGuiWindowUnitExecutionResultModel {
                    unit_id: unit_id.to_string(),
                    location: ResultExecutionLocation::Block(block_index),
                    status_label: "Converged",
                    summary: "Final local evaluation; actual consumed and produced states".into(),
                    consumed_stream_actions: consumed_stream_results
                        .iter()
                        .map(|s| s.focus_action.clone())
                        .collect(),
                    produced_stream_actions: produced_stream_results
                        .iter()
                        .map(|s| s.focus_action.clone())
                        .collect(),
                    consumed_stream_results,
                    produced_stream_results,
                });
            }
        }
        review_summary.unit_results = unit_results
            .iter()
            .map(|u| StudioGuiWindowResultReviewUnitModel {
                unit_id: u.unit_id.clone(),
                location: u.location,
                status_label: u.status_label,
                summary: u.summary.clone(),
                consumed_stream_ids: u
                    .consumed_stream_results
                    .iter()
                    .map(|s| s.stream_id.clone())
                    .collect(),
                produced_stream_ids: u
                    .produced_stream_results
                    .iter()
                    .map(|s| s.stream_id.clone())
                    .collect(),
            })
            .collect();
        let consumed: std::collections::BTreeSet<_> = unit_results
            .iter()
            .flat_map(|u| u.consumed_stream_results.iter().map(|s| &s.stream_id))
            .collect();
        let produced: std::collections::BTreeSet<_> = unit_results
            .iter()
            .flat_map(|u| u.produced_stream_results.iter().map(|s| &s.stream_id))
            .collect();
        let sources: std::collections::BTreeSet<_> = unit_results
            .iter()
            .filter(|u| u.consumed_stream_results.is_empty())
            .flat_map(|u| u.produced_stream_results.iter().map(|s| &s.stream_id))
            .collect();
        review_summary.source_stream_results = streams
            .iter()
            .filter(|s| sources.contains(&s.stream_id) && consumed.contains(&s.stream_id))
            .map(stream_result_reference_model_from_window_stream)
            .collect();
        review_summary.intermediate_stream_results = streams
            .iter()
            .filter(|s| {
                !sources.contains(&s.stream_id)
                    && consumed.contains(&s.stream_id)
                    && produced.contains(&s.stream_id)
            })
            .map(stream_result_reference_model_from_window_stream)
            .collect();
        review_summary.terminal_stream_results = streams
            .iter()
            .filter(|s| produced.contains(&s.stream_id) && !consumed.contains(&s.stream_id))
            .map(stream_result_reference_model_from_window_stream)
            .collect();

        match snapshot {
            rf_ui::FlowResultRecord::Partitioned(p) => {
                partitioned = Some(StudioGuiWindowPartitionedModel {
                    result: p.result.clone(),
                    units: unit_results,
                })
            }
            rf_ui::FlowResultRecord::Simultaneous(p) => {
                let evidence = crate::modular_evidence::result_lines(&p.result, units);
                presentation.errors.extend(evidence.errors);
                simultaneous = Some(StudioGuiWindowSimultaneousModel {
                    result: p.result.clone(),
                    units: unit_results,
                    evidence_lines: evidence.lines,
                });
            }
            rf_ui::FlowResultRecord::Acyclic(_) => unreachable!("block data"),
        }
    }
    StudioGuiWindowSolveSnapshotModel {
        partitioned,
        simultaneous,
        origin: snapshot.origin.clone(),
        result_guard: crate::FlowResultGuard::from_snapshot(snapshot),
        snapshot_id: snapshot.id.as_str().to_string(),
        sequence: snapshot.sequence,
        document_revision: snapshot.document_revision,
        status_label,
        summary: snapshot.summary.primary_message.clone(),
        diagnostic_count: diagnostics.len(),
        step_count: steps.len(),
        stream_count: streams.len(),
        review_summary,
        display_units: units.clone(),
        presentation_errors: presentation.errors,
        streams,
        steps,
        diagnostics,
    }
}
