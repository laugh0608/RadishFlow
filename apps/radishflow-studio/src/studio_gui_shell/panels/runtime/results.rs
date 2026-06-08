use super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_stream_result_inspector(
        &self,
        ui: &mut egui::Ui,
        stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
    ) {
        ui.label(egui::RichText::new(&stream.stream_id).strong());
        render_wrapped_small(ui, &stream.label);

        ui.small(egui::RichText::new(self.locale.text(ShellText::StreamSummary)).strong());
        egui::Grid::new(format!("stream-summary:{}", stream.stream_id))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for row in &stream.summary_rows {
                    ui.small(format!(
                        "{} · {}",
                        row.label,
                        self.locale.runtime_label(row.detail_label)
                    ));
                    ui.small(&row.value);
                    ui.end_row();
                }
            });

        if let Some(window) = stream.bubble_dew_window.as_ref() {
            ui.collapsing(self.locale.text(ShellText::BubbleDewWindow), |ui| {
                egui::Grid::new(format!("stream-bubble-dew-window:{}", stream.stream_id))
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.small(
                            egui::RichText::new(self.locale.runtime_label("Phase region").as_ref())
                                .strong(),
                        );
                        ui.small(self.locale.runtime_label(&window.phase_region).as_ref());
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Bubble pressure").as_ref());
                        ui.small(&window.bubble_pressure_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Dew pressure").as_ref());
                        ui.small(&window.dew_pressure_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Bubble temperature").as_ref());
                        ui.small(&window.bubble_temperature_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Dew temperature").as_ref());
                        ui.small(&window.dew_temperature_text);
                        ui.end_row();
                    });
            });
        }

        ui.collapsing(self.locale.text(ShellText::OverallComposition), |ui| {
            if stream.composition_rows.is_empty() {
                ui.small(self.locale.text(ShellText::NoComposition));
                return;
            }
            egui::Grid::new(format!("stream-composition:{}", stream.stream_id))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Component)).strong());
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::MoleFraction)).strong(),
                    );
                    ui.end_row();
                    for row in &stream.composition_rows {
                        ui.small(&row.component_id);
                        ui.small(&row.fraction_text);
                        ui.end_row();
                    }
                });
        });

        ui.collapsing(self.locale.text(ShellText::PhaseResults), |ui| {
            if stream.phase_rows.is_empty() {
                ui.small(self.locale.text(ShellText::NoPhases));
                return;
            }
            egui::Grid::new(format!("stream-phases:{}", stream.stream_id))
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Phase)).strong());
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Fraction)).strong());
                    ui.small(egui::RichText::new(self.locale.runtime_label("Molar flow")).strong());
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::OverallComposition))
                            .strong(),
                    );
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Enthalpy)).strong());
                    ui.end_row();
                    for row in &stream.phase_rows {
                        ui.small(&row.label);
                        ui.small(&row.phase_fraction_text);
                        ui.small(&row.molar_flow_text);
                        render_wrapped_small(ui, &row.composition_text);
                        ui.small(row.molar_enthalpy_text.as_deref().unwrap_or("-"));
                        ui.end_row();
                    }
                });
        });

        ui.small(egui::RichText::new(self.locale.text(ShellText::StreamSummary)).strong());
        egui::Grid::new(format!("stream-context-summary:{}", stream.stream_id))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::OverallComposition));
                render_wrapped_small(ui, &stream.composition_text);
                ui.end_row();

                ui.small(self.locale.text(ShellText::PhaseResults));
                render_wrapped_small(ui, &stream.phase_text);
                ui.end_row();
            });
        ui.add_space(8.0);
    }

    pub(in crate::studio_gui_shell) fn render_unit_execution_result_inspector(
        &mut self,
        ui: &mut egui::Ui,
        unit: &radishflow_studio::StudioGuiWindowUnitExecutionResultModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&unit.unit_id).strong());
            render_status_chip(
                ui,
                self.locale.runtime_label(unit.status_label).as_ref(),
                run_status_color(unit.status_label),
            );
            ui.small(format!("#{}", unit.step_index));
        });
        render_wrapped_label(ui, &unit.summary);
        if !unit.consumed_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!("unit-consumed-streams:{}:{}", unit.unit_id, unit.step_index),
                self.locale.text(ShellText::InspectorConsumedStreams),
                &unit.consumed_stream_results,
            );
        }
        if !unit.produced_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!("unit-produced-streams:{}:{}", unit.unit_id, unit.step_index),
                self.locale.text(ShellText::InspectorProducedStreams),
                &unit.produced_stream_results,
            );
        }
        ui.add_space(8.0);
    }

    pub(super) fn render_solve_step_inspector(
        &mut self,
        ui: &mut egui::Ui,
        step: &radishflow_studio::StudioGuiWindowSolveStepModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(format!("#{}", step.index));
            render_status_chip(
                ui,
                self.locale
                    .runtime_label(step.execution_status_label)
                    .as_ref(),
                run_status_color(step.execution_status_label),
            );
            if !step.consumed_stream_actions.is_empty() {
                for action in &step.consumed_stream_actions {
                    let _ = self.render_small_command_action(ui, action);
                }
                ui.small("->");
            }
            let _ = self.render_small_command_action(ui, &step.unit_action);
            if !step.produced_stream_actions.is_empty() {
                ui.small("->");
                for action in &step.produced_stream_actions {
                    let _ = self.render_small_command_action(ui, action);
                }
            }
        });
        render_wrapped_label(ui, &step.summary);
        if !step.consumed_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!(
                    "solve-step-consumed-streams:{}:{}",
                    step.unit_id, step.index
                ),
                self.locale.text(ShellText::InspectorConsumedStreams),
                &step.consumed_stream_results,
            );
        }
        if !step.produced_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!(
                    "solve-step-produced-streams:{}:{}",
                    step.unit_id, step.index
                ),
                self.locale.text(ShellText::InspectorProducedStreams),
                &step.produced_stream_results,
            );
        }
        ui.add_space(4.0);
    }

    pub(super) fn render_stream_result_reference_grid(
        &mut self,
        ui: &mut egui::Ui,
        grid_id: String,
        title: &str,
        streams: &[radishflow_studio::StudioGuiWindowStreamResultReferenceModel],
    ) {
        ui.add_space(4.0);
        ui.small(egui::RichText::new(title).strong());
        egui::Grid::new(grid_id)
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for stream in streams {
                    let _ = self.render_small_command_action(ui, &stream.focus_action);
                    render_wrapped_small(ui, &stream.summary);
                    ui.end_row();
                }
            });
    }

    pub(in crate::studio_gui_shell) fn render_result_inspector(
        &mut self,
        ui: &mut egui::Ui,
        inspector: &radishflow_studio::StudioGuiWindowResultInspectorModel,
    ) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::ResultInspector)).strong());
        ui.small(self.locale.text(ShellText::SelectStream));
        ui.horizontal_wrapped(|ui| {
            for option in &inspector.stream_options {
                let label = if option.label.is_empty() {
                    option.stream_id.as_str()
                } else {
                    option.label.as_str()
                };
                let response = ui
                    .add(egui::Button::new(label).selected(option.is_selected))
                    .on_hover_text(&option.summary);
                if response.clicked() {
                    self.result_inspector
                        .select_stream(&inspector.snapshot_id, option.stream_id.clone());
                }
            }
        });
        if inspector.has_stale_selection {
            render_wrapped_small(ui, self.locale.text(ShellText::StaleStreamSelection));
        }
        ui.separator();

        if let Some(stream) = inspector.selected_stream.as_ref() {
            ui.push_id(
                format!(
                    "result-inspector:selected-stream:{}:{}",
                    inspector.snapshot_id, stream.stream_id
                ),
                |ui| self.render_stream_result_inspector(ui, stream),
            );
        } else {
            ui.small(self.locale.text(ShellText::NoStreamResults));
            return;
        }

        if !inspector.unit_options.is_empty() {
            ui.collapsing(self.locale.text(ShellText::ResultUnitView), |ui| {
                ui.small(self.locale.text(ShellText::SelectUnit));
                ui.horizontal_wrapped(|ui| {
                    for option in &inspector.unit_options {
                        let response = ui
                            .add(egui::Button::new(&option.unit_id).selected(option.is_selected))
                            .on_hover_text(&option.summary);
                        if response.clicked() {
                            self.result_inspector
                                .select_unit(&inspector.snapshot_id, option.unit_id.clone());
                        }
                    }
                });
                if inspector.has_stale_unit_selection {
                    render_wrapped_small(ui, self.locale.text(ShellText::StaleUnitSelection));
                }
                if let Some(unit) = inspector.selected_unit.as_ref() {
                    ui.add_space(4.0);
                    self.render_unit_execution_result_inspector(ui, unit);
                } else {
                    ui.small(self.locale.text(ShellText::NoUnitResults));
                }

                if !inspector.unit_diagnostic_actions.is_empty() {
                    ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                        self.render_diagnostic_target_actions(
                            ui,
                            &inspector.unit_diagnostic_actions,
                        );
                    });
                }

                ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
                    if inspector.unit_related_steps.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRelatedSteps));
                        return;
                    }
                    for step in &inspector.unit_related_steps {
                        self.render_solve_step_inspector(ui, step);
                    }
                });

                ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                    if inspector.unit_related_diagnostics.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRelatedDiagnostics));
                        return;
                    }
                    for (index, diagnostic) in inspector.unit_related_diagnostics.iter().enumerate()
                    {
                        self.render_diagnostic_summary(
                            ui,
                            diagnostic,
                            format!(
                                "result-unit:{}:{}:diagnostic:{index}",
                                inspector.snapshot_id,
                                inspector.selected_unit_id.as_deref().unwrap_or("none")
                            ),
                        );
                        ui.add_space(4.0);
                    }
                });
            });
        }

        if !inspector.comparison_options.is_empty() {
            ui.collapsing(self.locale.text(ShellText::StreamComparison), |ui| {
                ui.small(self.locale.text(ShellText::CompareWith));
                ui.horizontal_wrapped(|ui| {
                    for option in &inspector.comparison_options {
                        let label = if option.label.is_empty() {
                            option.stream_id.as_str()
                        } else {
                            option.label.as_str()
                        };
                        let response = ui
                            .add(egui::Button::new(label).selected(option.is_selected))
                            .on_hover_text(&option.summary);
                        if response.clicked() {
                            self.result_inspector.select_comparison_stream(
                                &inspector.snapshot_id,
                                option.stream_id.clone(),
                            );
                        }
                    }
                });
                if inspector.has_stale_comparison {
                    render_wrapped_small(ui, self.locale.text(ShellText::StaleStreamSelection));
                }
                if let Some(comparison) = inspector.comparison.as_ref() {
                    self.render_result_inspector_comparison(ui, comparison);
                } else {
                    ui.small(self.locale.text(ShellText::NoComparison));
                }
            });
        }

        if !inspector.diagnostic_actions.is_empty() {
            ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                self.render_diagnostic_target_actions(ui, &inspector.diagnostic_actions);
            });
        }

        ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
            if inspector.related_steps.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedSteps));
                return;
            }
            for step in &inspector.related_steps {
                self.render_solve_step_inspector(ui, step);
            }
        });

        ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
            if inspector.related_diagnostics.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedDiagnostics));
                return;
            }
            for (index, diagnostic) in inspector.related_diagnostics.iter().enumerate() {
                self.render_diagnostic_summary(
                    ui,
                    diagnostic,
                    format!(
                        "result-stream:{}:{}:diagnostic:{index}",
                        inspector.snapshot_id,
                        inspector.selected_stream_id.as_deref().unwrap_or("none")
                    ),
                );
                ui.add_space(4.0);
            }
        });
    }

    pub(in crate::studio_gui_shell) fn render_result_inspector_comparison(
        &mut self,
        ui: &mut egui::Ui,
        comparison: &radishflow_studio::StudioGuiWindowResultInspectorComparisonModel,
    ) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.small(format!(
                "{}: {}",
                self.locale.text(ShellText::BaseStream),
                comparison.base_stream_id
            ));
            let _ = self.render_small_command_action(ui, &comparison.base_stream_focus_action);
            ui.small(format!(
                "{}: {}",
                self.locale.text(ShellText::ComparedStream),
                comparison.compared_stream_id
            ));
            let _ = self.render_small_command_action(ui, &comparison.compared_stream_focus_action);
        });
        egui::Grid::new(format!(
            "result-comparison-summary:{}:{}",
            comparison.base_stream_id, comparison.compared_stream_id
        ))
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            ui.small(self.locale.text(ShellText::StreamSummary));
            ui.small(self.locale.text(ShellText::BaseStream));
            ui.small(self.locale.text(ShellText::ComparedStream));
            ui.small(self.locale.text(ShellText::Delta));
            ui.end_row();
            for row in &comparison.summary_rows {
                ui.small(format!(
                    "{} · {}",
                    row.label,
                    self.locale.runtime_label(row.detail_label)
                ));
                ui.small(&row.base_value);
                ui.small(&row.compared_value);
                ui.small(&row.delta_text);
                ui.end_row();
            }
        });

        if !comparison.composition_rows.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::OverallComposition)).strong());
            egui::Grid::new(format!(
                "result-comparison-composition:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Component));
                ui.small(self.locale.text(ShellText::BaseStream));
                ui.small(self.locale.text(ShellText::ComparedStream));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.composition_rows {
                    ui.small(&row.component_id);
                    ui.small(&row.base_fraction_text);
                    ui.small(&row.compared_fraction_text);
                    ui.small(&row.delta_text);
                    ui.end_row();
                }
            });
        }

        if !comparison.phase_rows.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::PhaseResults)).strong());
            egui::Grid::new(format!(
                "result-comparison-phase-flows:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(7)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Phase));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.text(ShellText::Fraction)
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.text(ShellText::Fraction)
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.runtime_label("Molar flow")
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.runtime_label("Molar flow")
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.phase_rows {
                    ui.small(&row.phase_label);
                    ui.small(&row.base_fraction_text);
                    ui.small(&row.compared_fraction_text);
                    ui.small(&row.fraction_delta_text);
                    ui.small(&row.base_molar_flow_text);
                    ui.small(&row.compared_molar_flow_text);
                    ui.small(&row.molar_flow_delta_text);
                    ui.end_row();
                }
            });

            ui.add_space(4.0);
            egui::Grid::new(format!(
                "result-comparison-phase-enthalpy:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Phase));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.text(ShellText::Enthalpy)
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.text(ShellText::Enthalpy)
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.phase_rows {
                    ui.small(&row.phase_label);
                    ui.small(&row.base_molar_enthalpy_text);
                    ui.small(&row.compared_molar_enthalpy_text);
                    ui.small(&row.molar_enthalpy_delta_text);
                    ui.end_row();
                }
            });
        }
    }
}
