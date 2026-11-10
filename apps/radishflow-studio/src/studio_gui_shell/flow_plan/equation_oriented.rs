use super::*;
use rf_solver::equation_oriented::*;
use std::collections::BTreeSet;
impl ReadyAppState {
    pub(super) fn render_equation_plan(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        o: &mut EquationOptions,
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.small(tr(zh,"Feed / Mixer / Splitter 固定温压物料方程；反算仅支持一个分流比例和一个终端产品总量。","Fixed-T/P Feed / Mixer / Splitter material equations; one released split fraction and one terminal product target."));
        let mut apply = false;
        for (field, title, value, unit) in [
            (
                FlowPlanNumericField::Temperature,
                tr(zh, "温度", "Temperature"),
                o.boundary.temperature_k,
                MeasurementUnit::Kelvin,
            ),
            (
                FlowPlanNumericField::Pressure,
                tr(zh, "压力", "Pressure"),
                o.boundary.pressure_pa,
                MeasurementUnit::Pascal,
            ),
        ] {
            apply |= number(
                ui,
                draft,
                field,
                title,
                value,
                unit,
                &mut self.flow_plan.notice,
            );
        }
        let can_change = !draft.has_modified_numbers();
        let flow = &self.platform_host.document().flowsheet;
        let splitters: Vec<_> = flow
            .units
            .values()
            .filter(|u| u.kind == "splitter")
            .map(|u| u.id.clone())
            .collect();
        let mut products = Vec::new();
        let consumed: BTreeSet<_> = flow
            .units
            .values()
            .flat_map(|u| u.ports.iter())
            .filter(|p| p.direction == PortDirection::Inlet)
            .filter_map(|p| p.stream_id.as_ref())
            .collect();
        for u in flow.units.values() {
            for p in &u.ports {
                if p.direction == PortDirection::Outlet
                    && let Some(s) = &p.stream_id
                    && !consumed.contains(s)
                {
                    let port = match p.name.as_str() {
                        "outlet" => MaterialPort::Outlet,
                        "outlet_a" => MaterialPort::OutletA,
                        "outlet_b" => MaterialPort::OutletB,
                        _ => continue,
                    };
                    products.push(PortBinding {
                        unit: u.id.clone(),
                        port,
                        stream: s.clone(),
                    });
                }
            }
        }
        ui.strong(tr(
            zh,
            "释放参数与产品规格",
            "Released parameter and product specification",
        ));
        let mut selected = o.released.as_ref().map(|r| r.binding.unit.clone());
        let before = selected.clone();
        ui.add_enabled_ui(can_change, |ui| {
            egui::ComboBox::from_id_salt("eo-release")
                .width(ui.available_width().min(300.0))
                .selected_text(
                    selected
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| {
                            tr(zh, "不释放比例（正向）", "No released fraction (forward)").into()
                        }),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut selected,
                        None,
                        tr(zh, "不释放比例", "No released fraction"),
                    );
                    for id in &splitters {
                        ui.selectable_value(&mut selected, Some(id.clone()), id.to_string());
                    }
                });
        });
        if selected != before {
            o.released = selected.map(|unit| ReleasedSplitFraction {
                binding: ParameterBinding {
                    unit,
                    field: ParameterField::OutletASplitFraction,
                },
                lower: 0.0,
                upper: 0.95,
                max_iterations: 64,
            });
        }
        if let Some(r) = &o.released {
            ui.label(format!("{}.outlet_a_split_fraction", r.binding.unit));
            let stored = self
                .platform_host
                .document()
                .flowsheet
                .units
                .get(&r.binding.unit)
                .and_then(|u| u.parameters.outlet_a_split_fraction);
            ui.small(format!(
                "{}: {} · {}",
                tr(zh, "工程输入", "Stored input"),
                radishflow_studio::modular_evidence::diagnostic_number(stored),
                tr(
                    zh,
                    "本任务由规格求解；求解不会修改输入。",
                    "Derived by this specification; solving does not change the input."
                )
            ));
            if !splitters.contains(&r.binding.unit) {
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    tr(
                        zh,
                        "绑定已失效，保留原身份；请选择或移除。",
                        "Binding unavailable; original identity retained. Select or remove it.",
                    ),
                );
            }
            if ui
                .button(tr(zh, "定位释放字段", "Locate released field"))
                .clicked()
            {
                self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(
                    &rf_ui::InspectorTarget::Unit(r.binding.unit.clone()),
                ));
            }
            for (field, title, value) in [
                (
                    FlowPlanNumericField::ReleasedLower(r.binding.clone()),
                    tr(zh, "区间下限", "Interval lower"),
                    r.lower,
                ),
                (
                    FlowPlanNumericField::ReleasedUpper(r.binding.clone()),
                    tr(zh, "区间上限", "Interval upper"),
                    r.upper,
                ),
                (
                    FlowPlanNumericField::BisectionBudget(r.binding.clone()),
                    tr(zh, "二分预算（1—128）", "Bisection budget (1–128)"),
                    r.max_iterations as f64,
                ),
            ] {
                apply |= number(
                    ui,
                    draft,
                    field,
                    title,
                    value,
                    MeasurementUnit::One,
                    &mut self.flow_plan.notice,
                );
            }
        }
        let mut target = o.target.as_ref().map(|t| t.source.clone());
        let before = target.clone();
        ui.add_enabled_ui(can_change, |ui| {
            egui::ComboBox::from_id_salt("eo-product")
                .width(ui.available_width().min(300.0))
                .selected_text(
                    target
                        .as_ref()
                        .map(|p| format!("{} / {}.{}", p.stream, p.unit, p.port.name()))
                        .unwrap_or_else(|| tr(zh, "无产品目标", "No product target").into()),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut target,
                        None,
                        tr(zh, "无产品目标", "No product target"),
                    );
                    for p in &products {
                        ui.selectable_value(
                            &mut target,
                            Some(p.clone()),
                            format!("{} / {}.{}", p.stream, p.unit, p.port.name()),
                        );
                    }
                });
        });
        if target != before {
            o.target = target.map(|source| ProductTotalTarget {
                source,
                value_mol_s: 1.0,
            });
        }
        if let Some(t) = &o.target {
            if !products.contains(&t.source) {
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    tr(
                        zh,
                        "目标不是当前终端出口；未自动重绑。",
                        "Target is not a current terminal outlet; no automatic rebinding.",
                    ),
                );
            }
            apply |= number(
                ui,
                draft,
                FlowPlanNumericField::ProductTarget(t.source.clone()),
                tr(zh, "产品总摩尔流量目标", "Product total molar flow target"),
                t.value_mol_s,
                MeasurementUnit::MolePerSecond,
                &mut self.flow_plan.notice,
            );
        }
        // A draft preview is explicitly separate from the applied qualified query.
        let mut preview_draft = draft.clone();
        preview_draft.replace_candidate(FlowPlanConfiguration::EquationOriented(o.clone()));
        let mut discovered = match preview_draft.configuration() {
            Ok(FlowPlanConfiguration::EquationOriented(p)) => p,
            _ => o.clone(),
        };
        discovered.scales = None;
        let preview = assemble(&self.platform_host.document().flowsheet, &discovered);
        ui.strong(tr(
            zh,
            "草稿方程与结构预览",
            "Draft equations and structure preview",
        ));
        match &preview {
            Ok(p) => {
                let s = p.structure();
                ui.label(format!(
                    "{} {} · {} {} · {} {}",
                    tr(zh, "变量", "Variables"),
                    s.variable_count,
                    tr(zh, "方程", "Equations"),
                    s.equation_count,
                    tr(zh, "匹配", "Matched"),
                    s.matching.len()
                ));
                ui.label(if s.is_square_matched() {
                    tr(
                        zh,
                        "结构匹配通过；数值可解性尚未证明。",
                        "Structure matched; numerical solvability is not proven.",
                    )
                } else {
                    tr(
                        zh,
                        "存在欠 / 过规格区域；正式运行被阻挡。",
                        "Under/over-specified regions exist; running is blocked.",
                    )
                });
                ui.collapsing(
                    tr(zh, "未匹配与关联区域", "Unmatched and associated regions"),
                    |ui| {
                        for (label, region) in [
                            (tr(zh, "欠定区域", "Underdetermined"), &s.underdetermined),
                            (tr(zh, "过定区域", "Overdetermined"), &s.overdetermined),
                        ] {
                            ui.strong(label);
                            for v in &region.variables {
                                self.equation_variable_row(ui, v, zh);
                            }
                            for e in &region.equations {
                                ui.label(format!("{e:?}"));
                            }
                        }
                    },
                );
                ui.collapsing(
                    tr(
                        zh,
                        "完整变量 / 方程（只读）",
                        "All variables / equations (read-only)",
                    ),
                    |ui| {
                        for v in p.variables() {
                            self.equation_variable_row(ui, v, zh);
                        }
                        ui.separator();
                        for e in p.equations() {
                            ui.label(format!("{e:?}"));
                        }
                    },
                );
            }
            Err(e) => {
                ui.label(format!(
                    "{}: {e}",
                    tr(zh, "预览不可用", "Preview unavailable")
                ));
            }
        }
        ui.small(tr(zh,"未完成数值文本使用上次合法配置预览；Apply 前须解决草稿。结构分析不调用物性。","Incomplete numeric text previews the last valid configuration. Resolve drafts before Apply. Structure analysis calls no property services."));
        if o.scales.is_none() {
            ui.label(tr(
                zh,
                "流量 / 残差尺度：本次新鲜总量；比例尺度：1。",
                "Flow / residual scales: evaluated fresh total; fraction scale: 1.",
            ));
            if ui
                .add_enabled(
                    can_change && preview.is_ok(),
                    egui::Button::new(tr(
                        zh,
                        "生成完整显式双尺度草稿",
                        "Create complete explicit scale draft",
                    )),
                )
                .clicked()
            {
                let p = preview.as_ref().expect("enabled");
                o.scales = Some(ExplicitEquationScales {
                    variables: p
                        .variables()
                        .iter()
                        .cloned()
                        .zip(p.variable_scales().iter().copied())
                        .collect(),
                    equations: p
                        .equations()
                        .iter()
                        .cloned()
                        .zip(p.residual_scales().iter().copied())
                        .collect(),
                });
            }
        } else if ui
            .add_enabled(
                can_change,
                egui::Button::new(tr(zh, "恢复默认尺度规则", "Use default scale rule")),
            )
            .clicked()
        {
            o.scales = None;
        }
        if let Some(scales) = &mut o.scales {
            ui.collapsing(
                tr(
                    zh,
                    "显式变量 / 残差尺度",
                    "Explicit variable / residual scales",
                ),
                |ui| {
                    let mut remove_v = None;
                    let mut remove_e = None;
                    for (b, v) in &scales.variables {
                        ui.push_id(b, |ui| {
                            ui.label(format!("{b:?}"));
                            let unit = if matches!(b, VariableBinding::Parameter(_)) {
                                MeasurementUnit::One
                            } else {
                                MeasurementUnit::MolePerSecond
                            };
                            apply |= number(
                                ui,
                                draft,
                                FlowPlanNumericField::EquationVariableScale(b.clone()),
                                tr(zh, "变量尺度", "Variable scale"),
                                *v,
                                unit,
                                &mut self.flow_plan.notice,
                            );
                            if ui
                                .add_enabled(
                                    can_change,
                                    egui::Button::new(tr(zh, "移除绑定", "Remove binding")),
                                )
                                .clicked()
                            {
                                remove_v = Some(b.clone());
                            }
                        });
                    }
                    for (b, v) in &scales.equations {
                        ui.push_id(b, |ui| {
                            ui.label(format!("{b:?}"));
                            apply |= number(
                                ui,
                                draft,
                                FlowPlanNumericField::EquationResidualScale(b.clone()),
                                tr(zh, "残差尺度", "Residual scale"),
                                *v,
                                MeasurementUnit::MolePerSecond,
                                &mut self.flow_plan.notice,
                            );
                            if ui
                                .add_enabled(
                                    can_change,
                                    egui::Button::new(tr(zh, "移除绑定", "Remove binding")),
                                )
                                .clicked()
                            {
                                remove_e = Some(b.clone());
                            }
                        });
                    }
                    if let Some(b) = remove_v {
                        scales.variables.remove(&b);
                    }
                    if let Some(b) = remove_e {
                        scales.equations.remove(&b);
                    }
                    if let Ok(p) = &preview
                        && ui
                            .add_enabled(
                                can_change,
                                egui::Button::new(tr(
                                    zh,
                                    "向草稿添加缺少的尺度绑定",
                                    "Add missing scale bindings to draft",
                                )),
                            )
                            .clicked()
                    {
                        for (b, v) in p.variables().iter().zip(p.variable_scales()) {
                            scales.variables.entry(b.clone()).or_insert(*v);
                        }
                        for (b, v) in p.equations().iter().zip(p.residual_scales()) {
                            scales.equations.entry(b.clone()).or_insert(*v);
                        }
                    }
                },
            );
        }
        apply
    }
    fn equation_variable_row(&mut self, ui: &mut egui::Ui, b: &VariableBinding, zh: bool) {
        ui.label(format!("{b:?}"));
        let unit = match b {
            VariableBinding::PortComponent { port, .. } => &port.unit,
            VariableBinding::Parameter(p) => &p.unit,
        };
        ui.push_id(b, |ui| {
            if ui.small_button(tr(zh, "定位单元", "Locate unit")).clicked() {
                self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(
                    &rf_ui::InspectorTarget::Unit(unit.clone()),
                ));
            }
        });
    }
}
