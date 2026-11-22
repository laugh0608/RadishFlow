use super::*;
use rf_solver::mixed::*;
use rf_solver::partitioned::EvaluationPhase;
use rf_types::units::QuantityKind;
pub(in crate::studio_gui_shell) fn progress_text(p: &MixedProgress, zh: bool) -> String {
    let stage = match p.stage {
        MixedStage::Assembly => tr(zh, "装配分区 DAG", "Assembling partition DAG"),
        MixedStage::BlockStarted => tr(zh, "执行分区块", "Executing block"),
        MixedStage::Equation => tr(zh, "块内 EO 求解与复核", "Block EO solve and verification"),
        MixedStage::Boundary => tr(zh, "复核真实跨块边界", "Verifying actual block boundary"),
        MixedStage::BlockCompleted => {
            tr(zh, "块完成（仅诊断）", "Block completed (diagnostic only)")
        }
        MixedStage::GlobalVerification => tr(zh, "总体物料复核", "Global material verification"),
        MixedStage::Completing => tr(
            zh,
            "准备接收整体结果",
            "Preparing complete result reception",
        ),
    };
    let mut text = stage.to_owned();
    if let Some(recycle) = &p.recycle {
        text.push_str(&format!(
            " · {} {}",
            phase_label(recycle.phase, zh),
            recycle.evaluation
        ));
    }
    if let Some(b) = &p.block {
        text.push_str(&format!(" · {}", b.as_str()));
    }
    if let Some(u) = &p.unit {
        text.push_str(&format!(" · {u}"));
    }
    if let Some(e) = &p.equation {
        text.push_str(&format!(
            " · {}",
            super::equation_results::progress_text(e, zh)
        ));
    }
    text
}
impl ReadyAppState {
    pub(super) fn render_mixed_current(
        &self,
        ui: &mut egui::Ui,
        r: &MixedResult,
        origin: &rf_ui::result_query::FlowResultOrigin,
    ) -> Option<StreamId> {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let units = self
            .platform_host
            .modeling_inputs()
            .workspace
            .project_presentation
            .display_units();
        let mut locate = None;
        if let Some(report) = &r.recycle {
            ui.strong(tr(
                zh,
                "混合分区 · 单撕裂回流",
                "Mixed partition · Single tear recycle",
            ));
            if let Some(verification) = report
                .residual_trace
                .last()
                .filter(|sample| sample.phase == EvaluationPhase::Verification)
            {
                let count = report
                    .residual_trace
                    .iter()
                    .filter(|sample| sample.phase == EvaluationPhase::Iteration)
                    .count();
                ui.label(format!(
                    "{} · {count}",
                    tr(zh, "候选评价", "Candidate evaluations")
                ));
                ui.label(format!(
                    "{} · {}",
                    tr(zh, "独立复核", "Independent verification"),
                    tr(zh, "通过", "Passed")
                ));
                ui.label(format!(
                    "{} · {}",
                    tr(zh, "最大绝对残差", "Max absolute residual"),
                    displayed_molar(
                        verification.max_absolute_mol_s,
                        units.unit_for(QuantityKind::MolarFlow)
                    )
                ));
                ui.label(format!(
                    "{} · {:.6e}",
                    tr(zh, "最大缩放残差", "Max scaled residual"),
                    verification.max_scaled
                ));
            }
            ui.label(tr(zh, "分量守恒 · 通过", "Component balance · Passed"));
            if let Some(package) = origin
                .execution()
                .and_then(|execution| execution.package.as_ref())
            {
                ui.label(format!(
                    "{} · {} / {}",
                    tr(zh, "本次物性包", "Property package"),
                    package.package_id,
                    package.version
                ));
            }
            let rendered = render_recycle_report(
                ui,
                report,
                units.unit_for(QuantityKind::MolarFlow),
                zh,
                true,
            );
            if rendered {
                locate = Some(report.tear.stream_id.clone());
            }
        } else {
            ui.strong(tr(
                zh,
                "混合分区整体结果",
                "Complete mixed-partition result",
            ));
        }
        ui.label(format!(
            "{} {} · {} {}",
            r.blocks.len(),
            tr(zh, "块", "blocks"),
            r.boundaries.len(),
            tr(zh, "真实跨块边界", "actual block boundaries")
        ));
        ui.small(tr(zh,"全部局部单元、EO、跨块边界及总体物料已复核；热量和相平衡不在 EO 方程范围。","All local units, EO blocks, boundaries and global material closure verified. EO equations cover material only."));
        for b in &r.blocks {
            ui.collapsing(
                format!(
                    "{} · {} · {}",
                    b.id.as_str(),
                    match b.evidence {
                        MixedBlockEvidence::Local(_) => "Local",
                        MixedBlockEvidence::EquationOriented(_) => "EO",
                    },
                    b.members
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" + ")
                ),
                |ui| {
                    for line in radishflow_studio::mixed_evidence::block_lines(b, units).lines {
                        ui.label(line);
                    }
                },
            );
        }
        ui.collapsing(
            tr(
                zh,
                "跨块实际产出 / 消耗、残差及总体成本",
                "Actual boundary production / consumption, residuals and total costs",
            ),
            |ui| {
                for line in radishflow_studio::mixed_evidence::boundary_lines(r, units).lines {
                    ui.label(line);
                }
            },
        );
        locate
    }
}

pub(super) fn render_recycle_report(
    ui: &mut egui::Ui,
    report: &MixedRecycleReport,
    unit: MeasurementUnit,
    zh: bool,
    verified: bool,
) -> bool {
    let tear = &report.tear;
    ui.label(format!(
        "{} · {}.{} → {}.{}",
        tear.stream_id,
        tear.source.unit_id,
        tear.source.port_name,
        tear.sink.unit_id,
        tear.sink.port_name
    ));
    let locate = ui.button(format!("{} {}", tr(zh, "定位", "Locate"), tear.stream_id));
    reveal_plan_focus(&locate);
    if let Some(last) = &report.last_sweep {
        ui.collapsing(
            if verified {
                tr(zh, "最后一轮实际撕裂值", "Final evaluated tear boundary")
            } else {
                tr(
                    zh,
                    "最后一轮试值与闭合差",
                    "Last trial and closure residual",
                )
            },
            |ui| {
                for (id, residual) in &last.residuals_mol_s {
                    ui.strong(id.as_str());
                    for (name, state) in [
                        (tr(zh, "汇端消费", "Sink consumption"), &last.assumed),
                        (tr(zh, "源端返回", "Source return"), &last.returned),
                    ] {
                        if let Some(fraction) = state.overall_mole_fractions.get(id) {
                            ui.label(format!(
                                "{name} · {}",
                                displayed_molar(state.total_molar_flow_mol_s * fraction, unit)
                            ));
                        }
                    }
                    ui.label(format!(
                        "{} · {}",
                        tr(zh, "闭合差", "Closure residual"),
                        displayed_molar(*residual, unit)
                    ));
                    if let Some(scale) = report.component_scales_mol_s.get(id) {
                        ui.label(format!(
                            "{} · {}",
                            tr(zh, "尺度", "Scale"),
                            displayed_molar(*scale, unit)
                        ));
                    }
                }
            },
        );
    }
    ui.collapsing(
        tr(zh, "候选及复核轨迹", "Candidate and verification trace"),
        |ui| {
            for sample in &report.residual_trace {
                ui.label(format!(
                    "{} {} · {} · {:.6e}",
                    phase_label(sample.phase, zh),
                    sample.evaluation,
                    displayed_molar(sample.max_absolute_mol_s, unit),
                    sample.max_scaled
                ));
            }
        },
    );
    locate.clicked()
}

fn phase_label(phase: EvaluationPhase, zh: bool) -> &'static str {
    match phase {
        EvaluationPhase::Iteration => tr(zh, "候选评价", "Candidate"),
        EvaluationPhase::Verification => tr(zh, "独立复核", "Verification"),
    }
}

fn displayed_molar(value: f64, unit: MeasurementUnit) -> String {
    match from_canonical(value, QuantityKind::MolarFlow, unit) {
        Ok(shown) => format!("{shown:.9e} {}", unit.definition().symbol),
        Err(_) if value.is_finite() => format!("{value:.9e} mol/s"),
        Err(_) => "—".into(),
    }
}
