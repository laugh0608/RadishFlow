use super::*;
use rf_solver::simultaneous::{CouplingScale, ModularOptions, assemble};
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanMethod {
    Acyclic,
    Partitioned,
    Simultaneous,
    EquationOriented,
}
impl PlanMethod {
    pub(super) fn of(plan: &FlowPlanConfiguration) -> Self {
        match plan {
            FlowPlanConfiguration::Acyclic => Self::Acyclic,
            FlowPlanConfiguration::Partitioned(_) => Self::Partitioned,
            FlowPlanConfiguration::Simultaneous(_) => Self::Simultaneous,
            FlowPlanConfiguration::EquationOriented(_) => Self::EquationOriented,
        }
    }
    pub(super) fn label(self, zh: bool) -> &'static str {
        match self {
            Self::EquationOriented => tr(
                zh,
                "固定温压物料方程 / 规格反算",
                "Fixed-T/P equations / specification solve",
            ),
            Self::Acyclic => tr(zh, "无环序贯", "Acyclic sequential"),
            Self::Partitioned => tr(
                zh,
                "分块回流（固定温压）",
                "Partitioned recycle (fixed T/P)",
            ),
            Self::Simultaneous => tr(zh, "固定温压仿射联立模块", "Fixed-T/P affine modular solve"),
        }
    }
    pub(super) fn default_configuration(self) -> FlowPlanConfiguration {
        let boundary = FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650000.0,
        };
        match self {
            Self::EquationOriented => FlowPlanConfiguration::EquationOriented(
                rf_solver::equation_oriented::EquationOptions::new(boundary),
            ),
            Self::Acyclic => FlowPlanConfiguration::Acyclic,
            Self::Partitioned => FlowPlanConfiguration::Partitioned(RecycleSolveOptions::new(
                FixedRecycleBoundary {
                    temperature_k: 298.15,
                    pressure_pa: 101325.0,
                },
                vec![],
            )),
            Self::Simultaneous => {
                FlowPlanConfiguration::Simultaneous(ModularOptions::new(boundary))
            }
        }
    }
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn modular_plan_draft_pending(&self) -> bool {
        let plan = &self.platform_host.modeling_inputs().workspace.flow_plan;
        plan.has_pending_modular_draft() || plan.has_pending_equation_draft()
    }

    pub(super) fn render_simultaneous_plan(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        options: &mut ModularOptions,
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.small(tr(zh, "仅固定温压 Mixer / Splitter 仿射物料响应；不代表通用非线性、能量或相平衡求解。", "Only fixed-T/P affine Mixer / Splitter material responses; not a general nonlinear, energy or phase-equilibrium solve."));
        let mut apply = false;
        for (field, title, value, unit) in [
            (
                FlowPlanNumericField::Temperature,
                tr(zh, "温度", "Temperature"),
                options.boundary.temperature_k,
                MeasurementUnit::Kelvin,
            ),
            (
                FlowPlanNumericField::Pressure,
                tr(zh, "压力", "Pressure"),
                options.boundary.pressure_pa,
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
        // Discovery ignores the explicit map so invalid saved references remain
        // editable beside the current graph. It never repairs them implicitly.
        let mut discovery = options.clone();
        if let Ok(FlowPlanConfiguration::Simultaneous(parsed)) = draft.configuration() {
            discovery.boundary = parsed.boundary;
        }
        discovery.scales = None;
        let preview = assemble(&self.platform_host.document().flowsheet, &discovery);
        let mut bindings = BTreeSet::new();
        ui.strong(tr(
            zh,
            "自动耦合预览（只读）",
            "Automatic coupling preview (read-only)",
        ));
        match &preview {
            Ok(plan) => {
                for (index, block) in plan.blocks().iter().enumerate() {
                    ui.label(format!(
                        "{} {} · {} · {} {}",
                        tr(zh, "块", "Block"),
                        index + 1,
                        block
                            .members
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" + "),
                        block.couplings.len(),
                        tr(zh, "未知量", "unknowns")
                    ));
                    for binding in &block.couplings {
                        bindings.insert(binding.clone());
                        ui.label(format!(
                            "{} / {}: {}.{} → {}.{}",
                            binding.stream,
                            binding.component,
                            binding.source.unit_id,
                            binding.source.port_name,
                            binding.sink.unit_id,
                            binding.sink.port_name
                        ));
                    }
                }
            }
            Err(error) => {
                ui.label(format!(
                    "{}: {error}",
                    tr(zh, "预览不可用", "Preview unavailable")
                ));
            }
        }
        ui.small(tr(zh, "静态预览不证明数值可解。分流比例仅在单元设置编辑。", "Static preview does not prove numerical solvability. Edit split fractions in unit settings."));
        let explicit = options.scales.is_some();
        let can_change = !draft.has_modified_numbers();
        if !explicit {
            ui.label(tr(
                zh,
                "尺度：本次新鲜总量（运行时求值并冻结）",
                "Scales: evaluated fresh total (frozen during this run)",
            ));
            if ui
                .add_enabled(
                    can_change && preview.is_ok(),
                    egui::Button::new(tr(zh, "生成显式双尺度草稿", "Create explicit scale draft")),
                )
                .clicked()
            {
                options.scales = Some(
                    bindings
                        .iter()
                        .cloned()
                        .map(|b| {
                            (
                                b,
                                CouplingScale {
                                    variable_mol_s: 1.0,
                                    residual_mol_s: 1.0,
                                },
                            )
                        })
                        .collect(),
                );
            }
        } else if ui
            .add_enabled(
                can_change,
                egui::Button::new(tr(zh, "改为本次新鲜总量", "Use evaluated fresh total")),
            )
            .clicked()
        {
            options.scales = None;
        }
        if let Some(scales) = &mut options.scales {
            ui.small(tr(zh,"显式草稿新项以 1 mol/s 起始，须审阅并应用；两种尺度独立且必须正有限。", "New explicit entries start at 1 mol/s; review and Apply. Both scales are independent and must be positive finite."));
            let missing: Vec<_> = bindings
                .iter()
                .filter(|b| !scales.contains_key(*b))
                .cloned()
                .collect();
            if !missing.is_empty()
                && ui
                    .button(tr(
                        zh,
                        "向草稿添加缺少的绑定",
                        "Add missing bindings to draft",
                    ))
                    .clicked()
            {
                for binding in missing {
                    scales.insert(
                        binding,
                        CouplingScale {
                            variable_mol_s: 1.0,
                            residual_mol_s: 1.0,
                        },
                    );
                }
            }
            let mut remove = None;
            for (binding, scale) in scales.iter() {
                ui.push_id(binding, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.label(format!("{} / {}", binding.stream, binding.component));
                        ui.label(format!("{}.{} → {}.{}", binding.source.unit_id, binding.source.port_name, binding.sink.unit_id, binding.sink.port_name));
                        if !bindings.contains(binding) {
                            ui.colored_label(egui::Color32::DARK_RED, tr(zh,"绑定不在当前预览；保留原文，不自动重绑。", "Binding absent from preview; text retained, no automatic rebinding."));
                        }
                        ui.horizontal_wrapped(|ui| {
                            for (title, id) in [(tr(zh,"定位源单元","Locate source"), &binding.source.unit_id), (tr(zh,"定位汇单元","Locate sink"), &binding.sink.unit_id)] {
                                if ui.add_enabled(self.platform_host.document().flowsheet.units.contains_key(id), egui::Button::new(title)).clicked() {
                                    self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&rf_ui::InspectorTarget::Unit(id.clone())));
                                }
                            }
                            if ui.add_enabled(can_change, egui::Button::new(tr(zh,"移除此尺度绑定","Remove scale binding"))).clicked() { remove = Some(binding.clone()); }
                        });
                        apply |= number(ui, draft, FlowPlanNumericField::CouplingVariableScale(binding.clone()), tr(zh,"变量尺度","Variable scale"), scale.variable_mol_s, MeasurementUnit::MolePerSecond, &mut self.flow_plan.notice);
                        apply |= number(ui, draft, FlowPlanNumericField::CouplingResidualScale(binding.clone()), tr(zh,"残差尺度","Residual scale"), scale.residual_mol_s, MeasurementUnit::MolePerSecond, &mut self.flow_plan.notice);
                    });
                });
            }
            if let Some(binding) = remove {
                scales.remove(&binding);
            }
        }
        apply
    }
}
