use std::collections::{BTreeMap, BTreeSet};

use super::*;

impl StudioGuiWindowSolveSnapshotModel {
    pub fn result_inspector(
        &self,
        requested_stream_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        self.result_inspector_with_comparison(requested_stream_id, None)
    }

    pub fn result_inspector_with_comparison(
        &self,
        requested_stream_id: Option<&str>,
        requested_comparison_stream_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        self.result_inspector_with_unit(requested_stream_id, requested_comparison_stream_id, None)
    }

    pub fn result_inspector_with_unit(
        &self,
        requested_stream_id: Option<&str>,
        requested_comparison_stream_id: Option<&str>,
        requested_unit_id: Option<&str>,
    ) -> StudioGuiWindowResultInspectorModel {
        let selected_stream_id = requested_stream_id
            .filter(|stream_id| {
                self.streams
                    .iter()
                    .any(|stream| stream.stream_id == *stream_id)
            })
            .map(str::to_string)
            .or_else(|| self.streams.first().map(|stream| stream.stream_id.clone()));
        let has_stale_selection = requested_stream_id.is_some()
            && requested_stream_id.map(str::to_string) != selected_stream_id;
        let selected_stream = selected_stream_id
            .as_deref()
            .and_then(|selected_id| {
                self.streams
                    .iter()
                    .find(|stream| stream.stream_id == selected_id)
            })
            .cloned();
        let comparison_stream_id = requested_comparison_stream_id
            .filter(|stream_id| {
                selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id != *stream_id)
                    .unwrap_or(true)
                    && self
                        .streams
                        .iter()
                        .any(|stream| stream.stream_id == *stream_id)
            })
            .map(str::to_string);
        let has_stale_comparison = requested_comparison_stream_id.is_some()
            && requested_comparison_stream_id.map(str::to_string) != comparison_stream_id;
        let comparison_stream = comparison_stream_id
            .as_deref()
            .and_then(|comparison_id| {
                self.streams
                    .iter()
                    .find(|stream| stream.stream_id == comparison_id)
            })
            .cloned();
        let comparison = selected_stream
            .as_ref()
            .zip(comparison_stream.as_ref())
            .map(|(base, compared)| result_inspector_comparison_model(base, compared));
        let related_steps: Vec<StudioGuiWindowSolveStepModel> = selected_stream_id
            .as_deref()
            .map(|selected_id| {
                self.steps
                    .iter()
                    .filter(|step| {
                        step.produced_streams
                            .iter()
                            .any(|stream_id| stream_id == selected_id)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let related_step_unit_ids = related_steps
            .iter()
            .map(|step| step.unit_id.as_str())
            .collect::<BTreeSet<_>>();
        let related_diagnostics: Vec<StudioGuiWindowDiagnosticModel> = selected_stream_id
            .as_deref()
            .map(|selected_id| {
                self.diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic
                            .related_stream_ids
                            .iter()
                            .any(|stream_id| stream_id == selected_id)
                            || diagnostic
                                .related_unit_ids
                                .iter()
                                .any(|unit_id| related_step_unit_ids.contains(unit_id.as_str()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let diagnostic_actions = result_inspector_diagnostic_actions(
            selected_stream_id.as_deref(),
            &related_steps,
            &related_diagnostics,
        );
        let stream_options = self
            .streams
            .iter()
            .map(|stream| StudioGuiWindowResultInspectorStreamOptionModel {
                stream_id: stream.stream_id.clone(),
                label: stream.label.clone(),
                summary: format!(
                    "{} | T {} | P {} | F {}",
                    stream.stream_id,
                    stream.temperature_text,
                    stream.pressure_text,
                    stream.molar_flow_text
                ),
                is_selected: selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id == stream.stream_id)
                    .unwrap_or(false),
                focus_action: result_inspector_stream_focus_action(&stream.stream_id),
            })
            .collect();
        let comparison_options = self
            .streams
            .iter()
            .filter(|stream| {
                selected_stream_id
                    .as_deref()
                    .map(|selected_id| selected_id != stream.stream_id)
                    .unwrap_or(true)
            })
            .map(|stream| StudioGuiWindowResultInspectorStreamOptionModel {
                stream_id: stream.stream_id.clone(),
                label: stream.label.clone(),
                summary: format!(
                    "{} | T {} | P {} | F {}",
                    stream.stream_id,
                    stream.temperature_text,
                    stream.pressure_text,
                    stream.molar_flow_text
                ),
                is_selected: comparison_stream_id
                    .as_deref()
                    .map(|comparison_id| comparison_id == stream.stream_id)
                    .unwrap_or(false),
                focus_action: result_inspector_stream_focus_action(&stream.stream_id),
            })
            .collect();

        let unit_id_order: Vec<String> = self.steps.iter().map(|step| step.unit_id.clone()).fold(
            Vec::new(),
            |mut acc, unit_id| {
                if !acc.iter().any(|existing| existing == &unit_id) {
                    acc.push(unit_id);
                }
                acc
            },
        );
        let selected_unit_id = requested_unit_id
            .filter(|unit_id| unit_id_order.iter().any(|existing| existing == *unit_id))
            .map(str::to_string)
            .or_else(|| unit_id_order.first().cloned());
        let has_stale_unit_selection = requested_unit_id.is_some()
            && requested_unit_id.map(str::to_string) != selected_unit_id;
        let unit_related_steps: Vec<StudioGuiWindowSolveStepModel> = selected_unit_id
            .as_deref()
            .map(|selected_unit| {
                self.steps
                    .iter()
                    .filter(|step| step.unit_id == selected_unit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let unit_related_diagnostics: Vec<StudioGuiWindowDiagnosticModel> = selected_unit_id
            .as_deref()
            .map(|selected_unit| {
                let step_streams: BTreeSet<&str> = unit_related_steps
                    .iter()
                    .flat_map(|step| {
                        step.consumed_streams
                            .iter()
                            .chain(step.produced_streams.iter())
                            .map(String::as_str)
                    })
                    .collect();
                self.diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic
                            .related_unit_ids
                            .iter()
                            .any(|unit_id| unit_id == selected_unit)
                            || diagnostic
                                .related_stream_ids
                                .iter()
                                .any(|stream_id| step_streams.contains(stream_id.as_str()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let selected_unit = selected_unit_id.as_deref().and_then(|selected_unit| {
            unit_related_steps
                .last()
                .map(|step| StudioGuiWindowUnitExecutionResultModel {
                    unit_id: selected_unit.to_string(),
                    step_index: step.index,
                    status_label: step.execution_status_label,
                    summary: step.summary.clone(),
                    consumed_stream_ids: step.consumed_streams.clone(),
                    consumed_stream_actions: step.consumed_stream_actions.clone(),
                    produced_stream_ids: step.produced_streams.clone(),
                    produced_stream_actions: step.produced_stream_actions.clone(),
                })
        });
        let unit_options: Vec<StudioGuiWindowResultInspectorUnitOptionModel> = unit_id_order
            .iter()
            .map(|unit_id| {
                let last_step = self
                    .steps
                    .iter()
                    .rev()
                    .find(|step| &step.unit_id == unit_id);
                let status_label = last_step
                    .map(|step| step.execution_status_label)
                    .unwrap_or("Idle");
                let step_index = last_step.map(|step| step.index).unwrap_or(0);
                let produced_streams_text = last_step
                    .map(|step| step.produced_streams.join(", "))
                    .unwrap_or_default();
                let summary = if produced_streams_text.is_empty() {
                    format!("{unit_id} | {status_label} | step #{step_index}")
                } else {
                    format!(
                        "{unit_id} | {status_label} | step #{step_index} | -> {produced_streams_text}"
                    )
                };
                StudioGuiWindowResultInspectorUnitOptionModel {
                    unit_id: unit_id.clone(),
                    status_label,
                    step_index,
                    summary,
                    is_selected: selected_unit_id
                        .as_deref()
                        .map(|selected_unit| selected_unit == unit_id)
                        .unwrap_or(false),
                    focus_action: result_inspector_unit_focus_action(unit_id),
                }
            })
            .collect();
        let unit_diagnostic_actions = result_inspector_unit_diagnostic_actions(
            selected_unit_id.as_deref(),
            &unit_related_steps,
            &unit_related_diagnostics,
        );

        StudioGuiWindowResultInspectorModel {
            snapshot_id: self.snapshot_id.clone(),
            selected_stream_id,
            selected_stream,
            comparison_stream_id,
            comparison_stream,
            stream_options,
            comparison_options,
            comparison,
            related_steps,
            related_diagnostics,
            diagnostic_actions,
            has_stale_selection,
            has_stale_comparison,
            unit_options,
            selected_unit_id,
            selected_unit,
            unit_related_steps,
            unit_related_diagnostics,
            unit_diagnostic_actions,
            has_stale_unit_selection,
        }
    }
}

fn result_inspector_diagnostic_actions(
    selected_stream_id: Option<&str>,
    related_steps: &[StudioGuiWindowSolveStepModel],
    related_diagnostics: &[StudioGuiWindowDiagnosticModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let selected_stream = selected_stream_id.map(|stream_id| {
        diagnostic_target_action_from_action(
            "Selected stream",
            "Stream",
            format!("Selected result stream {stream_id}"),
            &inspector_stream_action(stream_id),
        )
    });
    let step_actions = related_steps
        .iter()
        .flat_map(|step| step.diagnostic_actions.iter().cloned());
    let diagnostic_actions = related_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.diagnostic_actions.iter().cloned());
    dedupe_diagnostic_actions(
        selected_stream
            .into_iter()
            .chain(step_actions)
            .chain(diagnostic_actions),
    )
}

fn result_inspector_unit_diagnostic_actions(
    selected_unit_id: Option<&str>,
    related_steps: &[StudioGuiWindowSolveStepModel],
    related_diagnostics: &[StudioGuiWindowDiagnosticModel],
) -> Vec<StudioGuiWindowDiagnosticTargetActionModel> {
    let selected_unit = selected_unit_id.map(|unit_id| {
        diagnostic_target_action_from_action(
            "Selected unit",
            "Unit",
            format!("Selected result unit {unit_id}"),
            &inspector_unit_action(unit_id),
        )
    });
    let step_actions = related_steps
        .iter()
        .flat_map(|step| step.diagnostic_actions.iter().cloned());
    let diagnostic_actions = related_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.diagnostic_actions.iter().cloned());
    dedupe_diagnostic_actions(
        selected_unit
            .into_iter()
            .chain(step_actions)
            .chain(diagnostic_actions),
    )
}

fn result_inspector_stream_focus_action(stream_id: &str) -> StudioGuiWindowCommandActionModel {
    let mut action = inspector_stream_action(stream_id);
    action.label = "Inspect".to_string();
    action.hover_text = format!("Open Stream Inspector for {stream_id}");
    action
}

fn result_inspector_unit_focus_action(unit_id: &str) -> StudioGuiWindowCommandActionModel {
    let mut action = inspector_unit_action(unit_id);
    action.label = "Inspect".to_string();
    action.hover_text = format!("Open Unit Inspector for {unit_id}");
    action
}

fn result_inspector_comparison_model(
    base: &StudioGuiWindowStreamResultModel,
    compared: &StudioGuiWindowStreamResultModel,
) -> StudioGuiWindowResultInspectorComparisonModel {
    let base_composition = base
        .composition_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.fraction))
        .collect::<BTreeMap<_, _>>();
    let compared_composition = compared
        .composition_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.fraction))
        .collect::<BTreeMap<_, _>>();
    let component_ids = base_composition
        .keys()
        .chain(compared_composition.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let base_phases = base
        .phase_rows
        .iter()
        .map(|row| (row.label.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let compared_phases = compared
        .phase_rows
        .iter()
        .map(|row| (row.label.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let phase_labels = base_phases
        .keys()
        .chain(compared_phases.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    let mut summary_rows = vec![
        StudioGuiWindowResultInspectorComparisonRowModel {
            label: "T",
            detail_label: "Temperature",
            base_value: base.temperature_text.clone(),
            compared_value: compared.temperature_text.clone(),
            delta_text: format_signed_delta(compared.temperature_k - base.temperature_k, "K", 2),
        },
        StudioGuiWindowResultInspectorComparisonRowModel {
            label: "P",
            detail_label: "Pressure",
            base_value: base.pressure_text.clone(),
            compared_value: compared.pressure_text.clone(),
            delta_text: format_signed_delta(compared.pressure_pa - base.pressure_pa, "Pa", 0),
        },
        StudioGuiWindowResultInspectorComparisonRowModel {
            label: "F",
            detail_label: "Molar flow",
            base_value: base.molar_flow_text.clone(),
            compared_value: compared.molar_flow_text.clone(),
            delta_text: format_signed_delta(
                compared.total_molar_flow_mol_s - base.total_molar_flow_mol_s,
                "mol/s",
                6,
            ),
        },
    ];
    if base.molar_enthalpy_j_per_mol.is_some() || compared.molar_enthalpy_j_per_mol.is_some() {
        summary_rows.push(StudioGuiWindowResultInspectorComparisonRowModel {
            label: "H",
            detail_label: "Molar enthalpy",
            base_value: base
                .molar_enthalpy_text
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            compared_value: compared
                .molar_enthalpy_text
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            delta_text: match (
                base.molar_enthalpy_j_per_mol,
                compared.molar_enthalpy_j_per_mol,
            ) {
                (Some(base), Some(compared)) => format_signed_delta(compared - base, "J/mol", 3),
                _ => "-".to_string(),
            },
        });
    }

    StudioGuiWindowResultInspectorComparisonModel {
        base_stream_id: base.stream_id.clone(),
        compared_stream_id: compared.stream_id.clone(),
        summary_rows,
        composition_rows: component_ids
            .into_iter()
            .map(|component_id| {
                let base_fraction = base_composition.get(component_id).copied().unwrap_or(0.0);
                let compared_fraction = compared_composition
                    .get(component_id)
                    .copied()
                    .unwrap_or(0.0);
                StudioGuiWindowResultInspectorCompositionComparisonRowModel {
                    component_id: component_id.to_string(),
                    base_fraction_text: format_fraction(base_fraction),
                    compared_fraction_text: format_fraction(compared_fraction),
                    delta_text: format_signed_delta(compared_fraction - base_fraction, "", 4),
                }
            })
            .collect(),
        phase_rows: phase_labels
            .into_iter()
            .map(|phase_label| {
                let base_phase = base_phases.get(phase_label).copied();
                let compared_phase = compared_phases.get(phase_label).copied();
                StudioGuiWindowResultInspectorPhaseComparisonRowModel {
                    phase_label: phase_label.to_string(),
                    base_fraction_text: base_phase
                        .map(|phase| phase.phase_fraction_text.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    compared_fraction_text: compared_phase
                        .map(|phase| phase.phase_fraction_text.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    fraction_delta_text: match (base_phase, compared_phase) {
                        (Some(base), Some(compared)) => format_signed_delta(
                            compared.phase_fraction - base.phase_fraction,
                            "",
                            4,
                        ),
                        _ => "-".to_string(),
                    },
                    base_molar_enthalpy_text: base_phase
                        .and_then(|phase| phase.molar_enthalpy_text.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    compared_molar_enthalpy_text: compared_phase
                        .and_then(|phase| phase.molar_enthalpy_text.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    molar_enthalpy_delta_text: match (
                        base_phase.and_then(|phase| phase.molar_enthalpy_j_per_mol),
                        compared_phase.and_then(|phase| phase.molar_enthalpy_j_per_mol),
                    ) {
                        (Some(base), Some(compared)) => {
                            format_signed_delta(compared - base, "J/mol", 3)
                        }
                        _ => "-".to_string(),
                    },
                }
            })
            .collect(),
    }
}

fn format_signed_delta(value: f64, unit: &str, decimals: usize) -> String {
    let sign = if value >= 0.0 { "+" } else { "" };
    let number = format!("{value:.decimals$}");
    if unit.is_empty() {
        format!("{sign}{number}")
    } else {
        format!("{sign}{number} {unit}")
    }
}
