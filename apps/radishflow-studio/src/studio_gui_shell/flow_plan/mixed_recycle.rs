use super::*;
use rf_model::Flowsheet;
use rf_solver::mixed::{
    MixedBlockMethod, MixedOptions, MixedRecycleConfiguration, MixedRecycleOptions,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Connections {
    Acyclic,
    Single,
    Serial,
}

impl Connections {
    fn label(self, zh: bool) -> &'static str {
        match self {
            Self::Acyclic => tr(zh, "块间无环", "Acyclic blocks"),
            Self::Single => tr(zh, "单撕裂回流", "Single tear recycle"),
            Self::Serial => tr(zh, "两区串联回流", "Two serial recycle regions"),
        }
    }
}

fn unbound_region() -> MixedRecycleOptions {
    MixedRecycleOptions::new(TearSpecification {
        stream_id: "".into(),
        source: DiagnosticPortTarget::new("", ""),
        sink: DiagnosticPortTarget::new("", ""),
        initial_component_flows_mol_s: None,
        component_scales_mol_s: None,
    })
}

impl ReadyAppState {
    pub(super) fn render_mixed_recycle_options(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        options: &mut MixedOptions,
        flow: &Flowsheet,
        bindings: &[TearSpecification],
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let can_change = !draft.has_modified_numbers();
        let current = match options.recycle.as_deref() {
            None => Connections::Acyclic,
            Some(MixedRecycleConfiguration::Single(_)) => Connections::Single,
            Some(MixedRecycleConfiguration::Serial(_)) => Connections::Serial,
        };
        let mut selected = current;
        ui.label(tr(zh, "块间连接", "Block connections"));
        ui.add_enabled_ui(can_change, |ui| {
            let picker = egui::ComboBox::from_id_salt("mixed-block-connections")
                .selected_text(current.label(zh))
                .show_ui(ui, |ui| {
                    for mode in [
                        Connections::Acyclic,
                        Connections::Single,
                        Connections::Serial,
                    ] {
                        ui.selectable_value(&mut selected, mode, mode.label(zh));
                    }
                });
            reveal_plan_focus(&picker.response);
        });
        if selected != current {
            options.recycle = match selected {
                Connections::Acyclic => None,
                Connections::Single => Some(Box::new(MixedRecycleConfiguration::Single(
                    unbound_region(),
                ))),
                Connections::Serial => Some(Box::new(MixedRecycleConfiguration::Serial([
                    options
                        .single_recycle()
                        .cloned()
                        .unwrap_or_else(unbound_region),
                    unbound_region(),
                ]))),
            };
        }
        if selected == Connections::Acyclic {
            return false;
        }
        let eligible = bindings
            .iter()
            .filter(|tear| tear_role_matches(flow, options, tear))
            .collect::<Vec<_>>();
        let serial = selected == Connections::Serial;
        let mut apply = false;
        let regions = options.recycle_regions_mut();
        for index in 0..regions.len() {
            let other = serial.then(|| regions[1 - index].tear.stream_id.clone());
            let choices = eligible
                .iter()
                .copied()
                .filter(|tear| Some(&tear.stream_id) != other.as_ref())
                .collect::<Vec<_>>();
            apply |= ui
                .push_id(("mixed-recycle-region", index), |ui| {
                    if serial {
                        ui.strong(format!(
                            "{} · {}",
                            tr(zh, "回流区", "Recycle region"),
                            index + 1
                        ));
                    }
                    self.render_recycle_region(
                        ui,
                        draft,
                        &mut regions[index],
                        flow,
                        &choices,
                        serial,
                    )
                })
                .inner;
        }
        ui.small(if serial {
            tr(zh, "各区按实际入口缩放，整图按外部进料缩放。运行顺序由流股连接决定。", "Each region scales by its evaluated inlet. The whole flow uses external feeds. Connections determine the execution order.")
        } else {
            tr(zh, "尺度使用本次实际新鲜料总量，独立复核另计。", "Scaling uses the evaluated fresh feed total. Verification is counted separately.")
        });
        apply
    }

    fn render_recycle_region(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        recycle: &mut MixedRecycleOptions,
        flow: &Flowsheet,
        eligible: &[&TearSpecification],
        serial: bool,
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let can_change = !draft.has_modified_numbers();
        let tear = &mut recycle.tear;
        let mut apply = false;
        ui.label(tr(zh, "撕裂流股", "Tear stream"));
        ui.add_enabled_ui(can_change, |ui| {
            let binding = egui::ComboBox::from_id_salt("mixed-recycle-tear")
                .selected_text(if tear.stream_id.as_str().is_empty() {
                    tr(zh, "选择撕裂流股…", "Choose a tear stream…").to_owned()
                } else {
                    stream_label(flow, &tear.stream_id)
                })
                .show_ui(ui, |ui| {
                    if eligible.is_empty() {
                        ui.label(tr(
                            zh,
                            "需要从 Local Splitter 的 outlet_a 连到 EO Mixer 的 inlet_b。",
                            "Connect a Local Splitter outlet_a to an EO Mixer inlet_b.",
                        ));
                    }
                    for &candidate in eligible {
                        let selected = same_binding(candidate, tear);
                        let label = format!(
                            "{} · {} → {}",
                            stream_label(flow, &candidate.stream_id),
                            port_label(&candidate.source),
                            port_label(&candidate.sink)
                        );
                        if ui.selectable_label(selected, label).clicked() && !selected {
                            *tear = candidate.clone();
                        }
                    }
                });
            reveal_plan_focus(&binding.response);
        });
        if tear.stream_id.as_str().is_empty() {
            return false;
        }
        ui.label(format!(
            "{} · {}",
            tr(zh, "源端", "Source"),
            port_label(&tear.source)
        ));
        ui.label(format!(
            "{} · {}",
            tr(zh, "汇端", "Sink"),
            port_label(&tear.sink)
        ));
        if !eligible
            .iter()
            .any(|candidate| same_binding(candidate, tear))
        {
            ui.strong(tr(
                zh,
                "撕裂绑定失效，保留原身份。",
                "The tear binding is unavailable. Its original identity is retained.",
            ));
        }
        let locate = ui.add_enabled(
            flow.streams.contains_key(&tear.stream_id),
            egui::Button::new(format!("{} {}", tr(zh, "定位", "Locate"), tear.stream_id)),
        );
        reveal_plan_focus(&locate);
        if locate.clicked() {
            self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(
                &rf_ui::InspectorTarget::Stream(tear.stream_id.clone()),
            ));
        }
        let mut explicit = tear.initial_component_flows_mol_s.is_some();
        let response = ui.add_enabled(
            can_change,
            egui::Checkbox::new(
                &mut explicit,
                tr(
                    zh,
                    "显式组分流量初值",
                    "Explicit component flow initialization",
                ),
            ),
        );
        reveal_plan_focus(&response);
        if response.changed() {
            tear.initial_component_flows_mol_s = explicit.then(|| {
                flow.components
                    .keys()
                    .cloned()
                    .map(|component| (component, 0.0))
                    .collect()
            });
        }
        if let Some(values) = &mut tear.initial_component_flows_mol_s {
            if values.keys().ne(flow.components.keys()) {
                ui.label(tr(
                    zh,
                    "初值组分集合与工程不符，原值保留。",
                    "Initial components differ from the project. Original values are retained.",
                ));
                let reset = ui.add_enabled(
                    can_change,
                    egui::Button::new(tr(
                        zh,
                        "按工程组分重建初值",
                        "Reset initial values to project components",
                    )),
                );
                reveal_plan_focus(&reset);
                if reset.clicked() {
                    *values = flow
                        .components
                        .keys()
                        .map(|id| (id.clone(), values.get(id).copied().unwrap_or(0.0)))
                        .collect();
                }
            }
            for (component, value) in values {
                apply |= number(
                    ui,
                    draft,
                    FlowPlanNumericField::InitialComponent {
                        stream: tear.stream_id.clone(),
                        component: component.clone(),
                    },
                    &format!("{} {component}", tr(zh, "初值", "Initial flow")),
                    *value,
                    MeasurementUnit::MolePerSecond,
                    &mut self.flow_plan.notice,
                )
                .is_some();
            }
        } else {
            ui.label(tr(
                zh,
                "零组分流量初值",
                "Zero component flow initialization",
            ));
        }
        apply |= number(
            ui,
            draft,
            if serial {
                FlowPlanNumericField::RegionIterationBudget(tear.stream_id.clone())
            } else {
                FlowPlanNumericField::IterationBudget
            },
            tr(zh, "候选评价上限", "Candidate evaluation limit"),
            recycle.max_iteration_evaluations as f64,
            MeasurementUnit::One,
            &mut self.flow_plan.notice,
        )
        .is_some();
        apply
    }
}

fn stream_label(flow: &Flowsheet, stream: &StreamId) -> String {
    match flow.streams.get(stream) {
        Some(value) if value.name != stream.as_str() => format!("{} · {stream}", value.name),
        _ => stream.to_string(),
    }
}

fn port_label(port: &DiagnosticPortTarget) -> String {
    format!("{}.{}", port.unit_id, port.port_name)
}

fn same_binding(left: &TearSpecification, right: &TearSpecification) -> bool {
    left.stream_id == right.stream_id && left.source == right.source && left.sink == right.sink
}

fn tear_role_matches(flow: &Flowsheet, plan: &MixedOptions, tear: &TearSpecification) -> bool {
    tear.source.port_name == "outlet_a"
        && tear.sink.port_name == "inlet_b"
        && flow
            .units
            .get(&tear.source.unit_id)
            .is_some_and(|u| u.kind == "splitter")
        && flow
            .units
            .get(&tear.sink.unit_id)
            .is_some_and(|u| u.kind == "mixer")
        && plan.blocks.iter().any(|block| {
            block.members.contains(&tear.source.unit_id)
                && matches!(block.method, MixedBlockMethod::Local)
        })
        && plan.blocks.iter().any(|block| {
            block.members.contains(&tear.sink.unit_id)
                && matches!(block.method, MixedBlockMethod::EquationOriented { .. })
        })
}
