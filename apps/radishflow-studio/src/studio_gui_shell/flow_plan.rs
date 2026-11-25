//! The reviewed run-page plan editor. Formal configuration and raw drafts live
//! in WorkspaceState; the shell owns only connection selections and feedback.
use super::*;
use radishflow_studio::{
    StudioBackgroundRunAction as Action,
    flow_actions::{FlowAction, FlowActionRequest, FlowEditAccess, FlowEditContext},
    modeling_actions::{MaterialConnectionTarget, MaterialPortTarget},
};
use rf_solver::partitioned::{FixedRecycleBoundary, RecycleSolveOptions, TearSpecification};
use rf_solver::single_phase_recycle::SinglePhaseTearBinding;
use rf_types::units::{ALL_UNITS, MeasurementUnit, from_canonical};
use rf_types::{DiagnosticPortTarget, PortDirection, PortKind, StreamId};
use rf_ui::flow_plan::*;
mod connection;
mod equation_oriented;
pub(super) mod equation_results;
mod mixed;
mod mixed_recycle;
pub(super) mod mixed_results;
mod results;
mod simultaneous;
mod single_phase;
use simultaneous::PlanMethod;
pub(super) mod modular_results;

#[derive(Default)]
pub(super) struct FlowPlanPanelState {
    notice: Option<String>,
    pub(super) adoption: Option<radishflow_studio::equation_actions::EquationAdoptionPreview>,
    adoption_focus_first: bool,
    stream: String,
    source: Option<MaterialPortTarget>,
    sink: Option<MaterialPortTarget>,
}
fn tr(zh: bool, cn: &'static str, en: &'static str) -> &'static str {
    if zh { cn } else { en }
}
impl ReadyAppState {
    fn plan_action(&mut self, action: Action) -> bool {
        match self.platform_host.background_run_action(action) {
            Ok(_) => {
                self.flow_plan.notice = None;
                true
            }
            Err(error) => {
                self.flow_plan.notice = Some(error.to_string());
                false
            }
        }
    }
    pub(super) fn render_flow_plan(&mut self, ui: &mut egui::Ui) {
        // egui resolves backward tab focus on the next pass, even in an idle window.
        if ui.input(|input| input.key_pressed(egui::Key::Tab)) {
            ui.ctx().request_repaint();
        }
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.heading(tr(zh, "稳态运行计划", "Steady-state run plan"));
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let access = if self.read_only_project.is_some() {
            FlowEditAccess::ReadOnly
        } else {
            FlowEditAccess::Editable
        };
        let editable = access == FlowEditAccess::Editable;
        let current = workspace.flow_plan.current().clone();
        let expected = FlowPlanEditContext::capture(workspace);
        let mut draft = workspace
            .flow_plan
            .draft()
            .cloned()
            .unwrap_or_else(|| FlowPlanDraft::new(workspace, current.clone()));
        let original = draft.clone();
        let mut candidate = draft.candidate().clone();
        let applied_method = PlanMethod::of(&current);
        let applied_recycle =
            matches!(&current, FlowPlanConfiguration::Mixed(options) if options.recycle.is_some());
        let applied_serial = matches!(&current, FlowPlanConfiguration::Mixed(options) if options.serial_recycle().is_some());
        let saved_serial = workspace
            .flow_plan
            .saved_capabilities()
            .contains_key(rf_store::project_v4::MIXED_SERIAL_RECYCLE_CAPABILITY);
        let saved_recycle = workspace
            .flow_plan
            .saved_capabilities()
            .contains_key(rf_store::project_v4::MIXED_RECYCLE_CAPABILITY);
        let saved_composition = workspace
            .flow_plan
            .saved_capabilities()
            .get(rf_store::project_v4::EQUATION_ORIENTED_CAPABILITY)
            == Some(&rf_store::project_v4::PRODUCT_COMPOSITION_VERSION);
        let applied_composition = matches!(&current,
            FlowPlanConfiguration::EquationOriented(options) if options.requires_product_composition());
        ui.horizontal_wrapped(|ui| {
            let run = ui.add_enabled(
                editable && self.background_command_enabled("run_panel.run_manual"),
                egui::Button::new(tr(zh, "运行 F5", "Run F5")),
            );
            reveal_plan_focus(&run);
            if run.clicked() {
                self.dispatch_ui_command("run_panel.run_manual");
            }
            self.start_paused_button(ui);
            ui.label(format!(
                "{}: {}",
                tr(zh, "已应用", "Applied"),
                if applied_serial {
                    tr(
                        zh,
                        "混合分区 · 两区串联回流",
                        "Mixed partition · Two serial recycle regions",
                    )
                } else if applied_recycle {
                    tr(
                        zh,
                        "混合分区 · 单撕裂回流",
                        "Mixed partition · Single tear recycle",
                    )
                } else {
                    applied_method.label(zh)
                }
            ));
            ui.label(
                if self
                    .platform_host
                    .modeling_inputs()
                    .workspace
                    .flow_plan
                    .is_dirty()
                {
                    tr(zh, "尚未保存", "Unsaved")
                } else {
                    tr(zh, "与保存基线一致", "Matches saved baseline")
                },
            );
        });
        if editable && self.background_runs.is_some() && current == FlowPlanConfiguration::Acyclic {
            self.render_temperature_watch(ui);
        }
        self.render_feed_package_problem_entry(ui);
        if applied_recycle {
            ui.small(tr(
                zh,
                "仅显式运行。Active 下也须点击 Run 或按 F5。",
                "Manual run only, including in Active mode. Select Run or press F5.",
            ));
        }
        if saved_serial && !applied_serial {
            ui.small(tr(
                zh,
                "将当前计划另存到新路径以保留原串联回流工程。",
                "Save this plan to a new path to preserve the original serial recycle project.",
            ));
        } else if saved_recycle && !applied_recycle {
            ui.small(tr(
                zh,
                "使用另存为保存当前计划，保留原混合回流工程。",
                "Use Save As for this plan to keep the original mixed-recycle project.",
            ));
        } else if saved_composition && !applied_composition {
            ui.small(tr(
                zh,
                "使用另存为保存当前计划，保留原产品组成工程。",
                "Use Save As for this plan to keep the original product composition project.",
            ));
        }
        let mut method = PlanMethod::of(&candidate);
        let numeric_pending = draft.has_modified_numbers();
        ui.add_enabled_ui(editable && !numeric_pending, |ui| {
            let method_picker = egui::ComboBox::from_id_salt("flow-method")
                .selected_text(method.label(zh))
                .show_ui(ui, |ui| {
                    for choice in [
                        PlanMethod::Acyclic,
                        PlanMethod::Partitioned,
                        PlanMethod::Simultaneous,
                        PlanMethod::EquationOriented,
                        PlanMethod::Mixed,
                        PlanMethod::SinglePhase,
                        PlanMethod::SinglePhaseRecycle,
                    ] {
                        ui.selectable_value(&mut method, choice, choice.label(zh));
                    }
                });
            reveal_plan_focus(&method_picker.response);
        });
        if numeric_pending {
            ui.small(tr(
                zh,
                "切换方法或尺度规则前，请先应用或放弃数值草稿。",
                "Apply or discard numeric drafts before changing method or scale policy.",
            ));
        }
        if method != PlanMethod::of(&candidate) {
            let mut next = method.default_configuration();
            if let Some(properties) = candidate.single_phase_properties() {
                match &mut next {
                    FlowPlanConfiguration::SinglePhase(options) => *options = properties.clone(),
                    FlowPlanConfiguration::SinglePhaseRecycle(options) => {
                        options.properties = properties.clone()
                    }
                    _ => {}
                }
            }
            candidate = next;
            draft.replace_candidate(candidate.clone());
        }
        let components: Vec<_> = self
            .platform_host
            .modeling_inputs()
            .workspace
            .document
            .flowsheet
            .components
            .keys()
            .cloned()
            .collect();
        let flow = &self
            .platform_host
            .modeling_inputs()
            .workspace
            .document
            .flowsheet;
        let bindings: Vec<_> = flow
            .streams
            .keys()
            .filter_map(|stream| {
                let ports: Vec<_> = flow
                    .units
                    .values()
                    .flat_map(|u| {
                        u.ports
                            .iter()
                            .filter(|p| {
                                p.kind == PortKind::Material && p.stream_id.as_ref() == Some(stream)
                            })
                            .map(move |p| (u.id.clone(), p.name.clone(), p.direction))
                    })
                    .collect();
                let sources: Vec<_> = ports
                    .iter()
                    .filter(|p| p.2 == PortDirection::Outlet)
                    .collect();
                let sinks: Vec<_> = ports
                    .iter()
                    .filter(|p| p.2 == PortDirection::Inlet)
                    .collect();
                (sources.len() == 1 && sinks.len() == 1).then(|| TearSpecification {
                    stream_id: stream.clone(),
                    source: DiagnosticPortTarget::new(sources[0].0.clone(), sources[0].1.clone()),
                    sink: DiagnosticPortTarget::new(sinks[0].0.clone(), sinks[0].1.clone()),
                    initial_component_flows_mol_s: None,
                    component_scales_mol_s: None,
                })
            })
            .collect();
        let mut apply_requested = false;
        let sensible_tears: Vec<_> = bindings
            .iter()
            .filter(|tear| {
                tear.source.port_name == "outlet_a"
                    && flow.units[&tear.source.unit_id].kind == "splitter"
                    && flow.units[&tear.sink.unit_id].kind == "mixer"
                    && matches!(tear.sink.port_name.as_str(), "inlet_a" | "inlet_b")
            })
            .map(|tear| SinglePhaseTearBinding {
                stream_id: tear.stream_id.clone(),
                source: tear.source.clone(),
                sink: tear.sink.clone(),
            })
            .collect();
        ui.add_enabled_ui(editable, |ui| {
            if let FlowPlanConfiguration::Partitioned(options) = &mut candidate {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    for (field, title, value, unit) in [
                        (FlowPlanNumericField::Temperature, tr(zh,"温度","Temperature"), options.boundary.temperature_k, MeasurementUnit::Kelvin),
                        (FlowPlanNumericField::Pressure, tr(zh,"压力","Pressure"), options.boundary.pressure_pa, MeasurementUnit::Pascal),
                        (FlowPlanNumericField::IterationBudget, tr(zh,"迭代预算","Iteration budget"), options.max_iteration_evaluations as f64, MeasurementUnit::One),
                    ] { apply_requested |= number(ui, &mut draft, field, title, value, unit, &mut self.flow_plan.notice).is_some(); }
                    ui.small(tr(zh,"独立最终复核另计 1 次；验收容差随方法固定。","One independent final verification; method acceptance tolerances are fixed."));
                });
                let mut remove = None;
                for (i, tear) in options.tears.iter_mut().enumerate() {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.strong(format!("{} {}", tr(zh,"撕裂绑定","Tear binding"), i + 1));
                        ui.label(format!("{} / {} → {} → {} / {}", tear.source.unit_id, tear.source.port_name, tear.stream_id, tear.sink.unit_id, tear.sink.port_name));
                        ui.horizontal_wrapped(|ui| {
                            if ui.button(tr(zh,"定位源单元","Locate source")).clicked() { self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&rf_ui::InspectorTarget::Unit(tear.source.unit_id.clone()))); }
                            if ui.button(tr(zh,"定位汇单元","Locate sink")).clicked() { self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&rf_ui::InspectorTarget::Unit(tear.sink.unit_id.clone()))); }
                            if ui.button(tr(zh,"移除此绑定","Remove binding")).clicked() { remove = Some(i); }
                        });
                        let mut explicit = tear.initial_component_flows_mol_s.is_some();
                        ui.checkbox(&mut explicit, tr(zh,"显式组分初值（否则零组分流量）","Explicit component initial values (otherwise zero)"));
                        if explicit != tear.initial_component_flows_mol_s.is_some() { tear.initial_component_flows_mol_s = explicit.then(|| components.iter().cloned().map(|c| (c,0.0)).collect()); }
                        let mut scales = tear.component_scales_mol_s.is_some();
                        ui.checkbox(&mut scales, tr(zh,"显式逐组分尺度（否则本次新鲜总量）","Explicit component scales (otherwise evaluated fresh total)"));
                        if scales != tear.component_scales_mol_s.is_some() { tear.component_scales_mol_s = scales.then(|| components.iter().cloned().map(|c| (c,1.0)).collect()); }
                        for (initial, values) in [(true, &tear.initial_component_flows_mol_s),(false, &tear.component_scales_mol_s)] {
                            if let Some(values) = values { for (component, value) in values {
                                let field = if initial { FlowPlanNumericField::InitialComponent { stream: tear.stream_id.clone(), component: component.clone() } } else { FlowPlanNumericField::ResidualScale { stream: tear.stream_id.clone(), component: component.clone() } };
                                apply_requested |= number(ui, &mut draft, field, &format!("{} {component}", if initial { tr(zh,"初值","Initial") } else { tr(zh,"尺度","Scale") }), *value, MeasurementUnit::MolePerSecond, &mut self.flow_plan.notice).is_some();
                            }}
                        }
                    });
                }
                if let Some(i) = remove { options.tears.remove(i); }
                egui::ComboBox::from_id_salt("add-tear").selected_text(tr(zh,"添加撕裂流股绑定…","Add a tear stream binding…")).show_ui(ui, |ui| {
                    for binding in &bindings {
                        if !options.tears.iter().any(|t| t.stream_id == binding.stream_id) && ui.button(format!("{}: {}.{} → {}.{}",binding.stream_id,binding.source.unit_id,binding.source.port_name,binding.sink.unit_id,binding.sink.port_name)).clicked() { options.tears.push(binding.clone()); ui.close_menu(); }
                    }
                });
            }
        });
        if let FlowPlanConfiguration::Simultaneous(options) = &mut candidate {
            ui.add_enabled_ui(editable, |ui| {
                apply_requested |= self.render_simultaneous_plan(ui, &mut draft, options);
            });
        }
        let equation_submission =
            if let FlowPlanConfiguration::EquationOriented(options) = &mut candidate {
                ui.add_enabled_ui(editable, |ui| {
                    self.render_equation_plan(ui, &mut draft, options)
                })
                .inner
            } else {
                None
            };
        if let FlowPlanConfiguration::Mixed(options) = &mut candidate {
            ui.add_enabled_ui(editable, |ui| {
                apply_requested |= self.render_mixed_plan(ui, &mut draft, options, &bindings);
            });
        }
        if candidate.project_model_method().is_some() {
            let thermal = self.single_phase_thermal_units();
            let (submit, clear) = ui
                .add_enabled_ui(editable, |ui| {
                    let (properties, recycle_submit) = match &mut candidate {
                        FlowPlanConfiguration::SinglePhase(options) => (options, false),
                        FlowPlanConfiguration::SinglePhaseRecycle(options) => {
                            let submit = self.render_single_phase_recycle_options(
                                ui,
                                &mut draft,
                                &mut options.recycle,
                                &components,
                                &sensible_tears,
                            );
                            (&mut options.properties, submit)
                        }
                        _ => unreachable!("project property plan"),
                    };
                    let (submit, clear) = self.render_single_phase_plan(
                        ui,
                        &mut draft,
                        properties,
                        &components,
                        &thermal,
                        current.project_model_method().is_some(),
                    );
                    (submit || recycle_submit, clear)
                })
                .inner;
            apply_requested |= submit;
            if let Some(unit) = clear {
                let expected =
                    FlowEditContext::capture(&self.platform_host.modeling_inputs().workspace);
                self.plan_action(Action::EditFlow {
                    access,
                    request: FlowActionRequest {
                        expected,
                        action: FlowAction::ClearHeaterCoolerOutletTemperature(unit),
                    },
                });
            }
        }
        draft.replace_candidate(candidate);
        let configuration = draft.configuration();
        let pending = draft.is_pending(&current);
        if pending
            && (matches!(
                draft.candidate(),
                FlowPlanConfiguration::Simultaneous(_)
                    | FlowPlanConfiguration::EquationOriented(_)
                    | FlowPlanConfiguration::Mixed(_)
                    | FlowPlanConfiguration::SinglePhase(_)
                    | FlowPlanConfiguration::SinglePhaseRecycle(_)
            ) || matches!(
                current,
                FlowPlanConfiguration::Simultaneous(_)
                    | FlowPlanConfiguration::EquationOriented(_)
                    | FlowPlanConfiguration::Mixed(_)
                    | FlowPlanConfiguration::SinglePhase(_)
                    | FlowPlanConfiguration::SinglePhaseRecycle(_)
            ))
        {
            ui.label(tr(zh, "先应用或放弃草稿，才可 Run / F5 或输出当前结果；保存仅写正式配置。", "Apply or discard the draft before Run / F5 or current output. Save writes only formal configuration."));
        } else if pending {
            ui.label(tr(zh,"草稿未应用；Run/F5 使用已应用计划。保存保留草稿，只写正式配置。","Draft is not applied. Run/F5 uses the applied plan. Save writes formal configuration and preserves drafts."));
        }
        if let Err(error) = &configuration {
            ui.colored_label(egui::Color32::DARK_RED, error.to_string());
        }
        if draft != original {
            self.plan_action(Action::FlowPlanDraft {
                access,
                draft: Some(draft.clone()),
            });
        }
        ui.horizontal_wrapped(|ui| {
            let can_apply = editable && pending && configuration.is_ok();
            let apply = ui.add_enabled(
                can_apply,
                egui::Button::new(tr(zh, "应用计划", "Apply plan")),
            );
            reveal_plan_focus(&apply);
            let rejected = if can_apply
                && (apply.clicked() || apply_requested || equation_submission.is_some())
            {
                !self.plan_action(Action::ConfigureFlowPlan {
                    expected: draft.context().clone(),
                    access,
                    command: FlowPlanCommand::Apply(
                        configuration.clone().expect("enabled valid draft"),
                    ),
                })
            } else {
                configuration.is_err()
            };
            if rejected && let Some(field) = equation_submission {
                ui.memory_mut(|memory| memory.request_focus(field));
                if let Some(response) = ui.ctx().read_response(field) {
                    // Enter can restore the same ID without a focus-change event.
                    response.scroll_to_me(None);
                }
            }
            for (title, command) in [
                (
                    tr(zh, "恢复已保存", "Restore saved"),
                    FlowPlanCommand::RestoreSaved,
                ),
                (tr(zh, "撤销计划", "Undo plan"), FlowPlanCommand::Undo),
                (tr(zh, "重做计划", "Redo plan"), FlowPlanCommand::Redo),
            ] {
                let allowed = editable
                    && match command {
                        FlowPlanCommand::Undo => self
                            .platform_host
                            .modeling_inputs()
                            .workspace
                            .flow_plan
                            .can_undo(),
                        FlowPlanCommand::Redo => self
                            .platform_host
                            .modeling_inputs()
                            .workspace
                            .flow_plan
                            .can_redo(),
                        _ => true,
                    };
                let button = ui.add_enabled(allowed, egui::Button::new(title));
                reveal_plan_focus(&button);
                if button.clicked() {
                    self.plan_action(Action::ConfigureFlowPlan {
                        expected: expected.clone(),
                        access,
                        command,
                    });
                }
            }
            let discard = ui.add_enabled(
                editable && pending,
                egui::Button::new(tr(zh, "放弃草稿", "Discard draft")),
            );
            reveal_plan_focus(&discard);
            if discard.clicked() {
                self.plan_action(Action::FlowPlanDraft {
                    access,
                    draft: None,
                });
            }
        });
        let workspace = &self.platform_host.modeling_inputs().workspace;
        match workspace
            .flow_plan
            .current()
            .validate_for(&workspace.document.flowsheet)
        {
            Ok(()) => {
                ui.label(tr(zh,"计划拓扑与绑定已就绪；运行时继续检查输入与物性。","Plan topology and bindings ready; runtime also checks physical inputs and property package."));
            }
            Err(error) => {
                ui.label(format!("{}: {error}", tr(zh, "尚未就绪", "Not ready")));
            }
        }
        self.render_exact_connection(ui, access);
        self.render_flow_results(ui);
        self.render_equation_adoption(ui.ctx());
        if let Some(message) = &self.flow_plan.notice {
            ui.label(message);
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.screen = StudioShellScreen::Workbench;
        }
    }
}
fn reveal_plan_focus(response: &egui::Response) {
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
}

fn number(
    ui: &mut egui::Ui,
    draft: &mut FlowPlanDraft,
    field: FlowPlanNumericField,
    label: &str,
    value: f64,
    unit: MeasurementUnit,
    notice: &mut Option<String>,
) -> Option<egui::Id> {
    ui.push_id(&field, |ui| {
        ui.label(label);
        let input = draft.edit_number(field.clone(), value, unit).ok()?;
        let response = ui.add(
            egui::TextEdit::singleline(&mut input.raw)
                .id_salt("plan-number")
                .desired_width(ui.available_width().min(260.0)),
        );
        reveal_plan_focus(&response);
        let submitted = response.lost_focus()
            && ui.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    && !i.events.iter().any(|e| matches!(e, egui::Event::Ime(_)))
            });
        let mut selected = input.unit;
        if let Some(quantity) = field.quantity() {
            let unit_picker = egui::ComboBox::from_id_salt("unit")
                .selected_text(selected.definition().symbol)
                .show_ui(ui, |ui| {
                    for unit in ALL_UNITS
                        .iter()
                        .filter(|u| from_canonical(1.0, quantity, **u).is_ok())
                    {
                        ui.selectable_value(&mut selected, *unit, unit.definition().symbol);
                    }
                });
            reveal_plan_focus(&unit_picker.response);
            if selected != draft.numbers()[&field].unit
                && let Err(error) = draft.change_input_unit(&field, selected)
            {
                *notice = Some(error.to_string());
            }
        }
        submitted.then_some(response.id)
    })
    .inner
}
