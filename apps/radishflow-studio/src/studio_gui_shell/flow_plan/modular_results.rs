use super::*;
use radishflow_studio::modular_evidence;
use rf_solver::simultaneous::{ModularProgress, ModularStage};
use rf_types::units::{DisplayUnitSet, QuantityKind};

pub(crate) fn progress_text(progress: &ModularProgress, zh: bool) -> String {
    let stage = match progress.stage {
        ModularStage::AssemblyStarted | ModularStage::AssemblyCompleted => {
            tr(zh, "装配", "Assembly")
        }
        ModularStage::Initializing => tr(zh, "初始化", "Initialization"),
        ModularStage::BlockStarted | ModularStage::BlockCompleted => tr(zh, "依赖块", "Block"),
        ModularStage::Anchor => tr(zh, "锚点求值", "Anchor evaluation"),
        ModularStage::UnitStarted | ModularStage::UnitCompleted => {
            tr(zh, "局部单元求值", "Local unit evaluation")
        }
        ModularStage::Response => tr(zh, "仿射响应", "Affine response"),
        ModularStage::Factorization => tr(zh, "LU 分解", "LU factorization"),
        ModularStage::ConditionSolve => tr(zh, "条件估计", "Condition estimation"),
        ModularStage::LinearVerification => tr(zh, "线性复核", "Linear verification"),
        ModularStage::Verification => tr(zh, "独立最终复核", "Independent final verification"),
        ModularStage::Completing => tr(zh, "完成接收前检查", "Completion checks"),
    };
    let mut parts = vec![stage.to_string()];
    if let Some(block) = progress.block {
        parts.push(format!("{} {}", tr(zh, "块", "Block"), block + 1));
    }
    if let Some(unit) = &progress.unit {
        parts.push(format!("{} {unit}", tr(zh, "单元", "Unit")));
    }
    if let Some(column) = progress.column {
        parts.push(format!(
            "{} {}",
            tr(zh, "列 / RHS", "Column / RHS"),
            column + 1
        ));
    }
    parts.join(" · ")
}
fn flow_value(units: &DisplayUnitSet, value: f64) -> String {
    rf_ui::format_quantity(
        value,
        QuantityKind::MolarFlow,
        units.unit_for(QuantityKind::MolarFlow),
    )
    .unwrap_or_else(|e| format!("Conversion failed: {e}"))
}
impl ReadyAppState {
    pub(super) fn render_modular_current(
        &self,
        ui: &mut egui::Ui,
        result: &rf_solver::simultaneous::ModularResult,
    ) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let units = self
            .platform_host
            .modeling_inputs()
            .workspace
            .project_presentation
            .display_units();
        ui.small(tr(zh,"固定温压物料边界已复核；不代表通用能量或相平衡闭合。", "Fixed-T/P material boundary verified; not general energy or phase-equilibrium closure."));
        for (index, block) in result.blocks.iter().enumerate() {
            ui.push_id(("modular-block", index), |ui| {
                egui::CollapsingHeader::new(format!("{} {}", tr(zh, "依赖块", "Block"), index + 1))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(
                            block
                                .members
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(" + "),
                        );
                        if let Some(e) = &block.coupling {
                            ui.label(format!(
                                "{} {}",
                                e.connections.len(),
                                tr(zh, "耦合未知量", "coupled unknowns")
                            ));
                            for c in &e.connections {
                                ui.label(format!(
                                    "{} / {} · {}.{} → {}.{}",
                                    c.binding.stream,
                                    c.binding.component,
                                    c.binding.source.unit_id,
                                    c.binding.source.port_name,
                                    c.binding.sink.unit_id,
                                    c.binding.sink.port_name
                                ));
                                let assumed = format!(
                                    "{}: {}",
                                    tr(zh, "消费假设", "Assumed"),
                                    flow_value(units, c.assumed_mol_s)
                                );
                                let produced = format!(
                                    "{}: {}",
                                    tr(zh, "生产者值", "Produced"),
                                    flow_value(units, c.produced_mol_s)
                                );
                                if ui.available_width() >= 600.0 {
                                    ui.columns(2, |columns| {
                                        columns[0].label(&assumed);
                                        columns[1].label(&produced);
                                    });
                                } else {
                                    ui.label(assumed);
                                    ui.label(produced);
                                }
                                ui.label(format!(
                                    "{}: {} · {}: {}",
                                    tr(zh, "变量尺度", "Variable scale"),
                                    flow_value(units, c.scale.variable_mol_s),
                                    tr(zh, "残差尺度", "Residual scale"),
                                    flow_value(units, c.scale.residual_mol_s)
                                ));
                                // Scientific residuals retain small nonzero differences.
                                let raw = rf_types::units::from_canonical(
                                    c.residual_mol_s,
                                    QuantityKind::MolarFlow,
                                    units.unit_for(QuantityKind::MolarFlow),
                                );
                                ui.label(format!(
                                    "{}: {} {} · {}: {}",
                                    tr(zh, "原始残差", "Residual"),
                                    raw.map(|v| format!("{v:.9e}"))
                                        .unwrap_or_else(|e| e.to_string()),
                                    units.unit_for(QuantityKind::MolarFlow).definition().symbol,
                                    tr(zh, "缩放残差（无量纲）", "Scaled residual (dimensionless)"),
                                    modular_evidence::diagnostic_number(Some(c.scaled_residual))
                                ));
                            }
                            if let Some(linear) = &e.linear {
                                ui.label(format!(
                                    "rcond_indicator: {} · backward_error: {} ({})",
                                    modular_evidence::diagnostic_number(linear.rcond_indicator),
                                    modular_evidence::diagnostic_number(linear.backward_error),
                                    tr(zh, "无量纲", "dimensionless")
                                ));
                            }
                            ui.collapsing(tr(zh, "数值证据", "Numerical evidence"), |ui| {
                                for line in modular_evidence::coupling_lines(e, units).lines {
                                    ui.label(line);
                                }
                            });
                        } else {
                            ui.label(tr(
                                zh,
                                "无环块 · 无耦合未知量",
                                "Acyclic block · no coupling unknowns",
                            ));
                        }
                        ui.collapsing(tr(zh, "局部输入 / 输出", "Local I/O"), |ui| {
                            for line in
                                modular_evidence::local_unit_lines(&block.final_units, units).lines
                            {
                                ui.label(line);
                            }
                        });
                    });
            });
        }
        ui.strong(tr(
            zh,
            "真实成本（无外层迭代）",
            "Actual costs (no outer iterations)",
        ));
        for line in modular_evidence::cost_lines(&result.costs) {
            ui.label(line);
        }
    }
}
