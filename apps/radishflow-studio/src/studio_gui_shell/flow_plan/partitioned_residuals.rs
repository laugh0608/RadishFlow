//! Fixed-T/P single-tear evidence. All verdicts use the recorded SI values.
use super::*;
use rf_solver::partitioned::{
    EvaluationPhase, PartitionedProgress, PartitionedStage, RecycleBlockReport, RecycleTolerances,
    ResidualSample,
};
use rf_types::units::QuantityKind;

fn phase(phase: EvaluationPhase, zh: bool) -> &'static str {
    match phase {
        EvaluationPhase::Iteration => tr(zh, "迭代", "Iteration"),
        EvaluationPhase::Verification => tr(zh, "独立复核", "Independent verification"),
    }
}
fn flow(value: f64, unit: MeasurementUnit) -> String {
    rf_ui::format_quantity(value, QuantityKind::MolarFlow, unit).unwrap_or_else(|e| e.to_string())
}
fn number(value: f64) -> String {
    rf_ui::format_display_number(value).unwrap_or_else(|e| e.to_string())
}
pub(in crate::studio_gui_shell) fn progress_text(
    p: &PartitionedProgress,
    unit: MeasurementUnit,
    zh: bool,
) -> String {
    use PartitionedStage::*;
    let stage = match p.stage {
        AssemblyStarted => tr(zh, "开始装配", "Assembly started"),
        AssemblyCompleted => tr(zh, "装配完成", "Assembly completed"),
        Initializing => tr(zh, "初始化", "Initializing"),
        BlockStarted => tr(zh, "开始执行块", "Block started"),
        IterationStarted | VerificationStarted => tr(zh, "开始", "started"),
        IterationCompleted | VerificationCompleted => tr(zh, "完成", "completed"),
        UnitStarted => tr(zh, "单元开始", "Unit started"),
        UnitCompleted => tr(zh, "单元完成", "Unit completed"),
        Completing => tr(zh, "准备接收整体结果", "Completing solve"),
    };
    let mut current = Vec::new();
    if let Some(b) = p.block {
        current.push(format!("{} {}", tr(zh, "块", "Block"), b + 1));
    }
    if let (Some(ph), Some(n)) = (p.phase, p.evaluation) {
        current.push(format!("{} {n}", phase(ph, zh)));
    }
    current.push(stage.into());
    if let Some(id) = &p.unit {
        current.push(id.to_string());
    }
    let mut text = format!(
        "{} · {}",
        tr(zh, "当前执行", "Current execution"),
        current.join(" · ")
    );
    if let Some(s) = &p.residual {
        text.push_str(&format!(
            "\n{} · {} {} · {} {} · {} {} ({})",
            tr(zh, "最近完成", "Last completed"),
            phase(s.phase, zh),
            s.evaluation,
            tr(zh, "最大绝对残差", "Max absolute residual"),
            flow(s.max_absolute_mol_s, unit),
            tr(zh, "最大缩放残差", "Max scaled residual"),
            number(s.max_scaled),
            tr(zh, "无量纲", "dimensionless")
        ));
    } else {
        text.push('\n');
        text.push_str(tr(zh, "尚无已完成求值", "No completed evaluation yet"));
    }
    text
}

fn sample(
    ui: &mut egui::Ui,
    s: &ResidualSample,
    tol: RecycleTolerances,
    unit: MeasurementUnit,
    zh: bool,
    role: &str,
) {
    let verdict = |value, limit| {
        if value <= limit {
            tr(zh, "达标", "Met")
        } else {
            tr(zh, "超限", "Exceeded")
        }
    };
    ui.strong(format!(
        "{role}{}{} {}",
        if role.is_empty() { "" } else { " · " },
        phase(s.phase, zh),
        s.evaluation
    ));
    let limit = tr(zh, "容差", "tolerance");
    ui.label(format!(
        "{} {} · {limit} {} · {}",
        tr(zh, "最大绝对残差", "Max absolute residual"),
        flow(s.max_absolute_mol_s, unit),
        flow(tol.component_absolute_flow_mol_s, unit),
        verdict(s.max_absolute_mol_s, tol.component_absolute_flow_mol_s)
    ));
    ui.label(format!(
        "{} {} · {limit} {} ({}) · {}",
        tr(zh, "最大缩放残差", "Max scaled residual"),
        number(s.max_scaled),
        number(tol.scaled_flow),
        tr(zh, "无量纲", "dimensionless"),
        verdict(s.max_scaled, tol.scaled_flow)
    ));
}

pub(super) fn evidence(
    ui: &mut egui::Ui,
    r: &RecycleBlockReport,
    unit: MeasurementUnit,
    zh: bool,
    verified: bool,
    compact: bool,
) {
    let Some((last, earlier)) = r.residual_trace.split_last() else {
        details(ui, r, unit, zh);
        return;
    };
    let verified = verified && last.phase == EvaluationPhase::Verification;
    let candidates = |ui: &mut egui::Ui| {
        if verified && let Some((candidate, before)) = earlier.split_last() {
            if let Some(previous) = before.last() {
                sample(
                    ui,
                    previous,
                    r.tolerances,
                    unit,
                    zh,
                    tr(zh, "候选前一次", "Before candidate"),
                );
            }
            sample(
                ui,
                candidate,
                r.tolerances,
                unit,
                zh,
                tr(zh, "候选", "Candidate"),
            );
        }
    };
    if !compact {
        candidates(ui);
    }
    let role = if verified {
        tr(zh, "复核通过", "Verified")
    } else {
        tr(zh, "最后完成求值", "Last completed evaluation")
    };
    sample(ui, last, r.tolerances, unit, zh, role);
    if compact {
        ui.collapsing(
            format!(
                "{} · {} ({})",
                tr(zh, "块证据", "Block evidence"),
                r.tear.stream_id,
                r.residual_trace.len()
            ),
            |ui| {
                candidates(ui);
                details(ui, r, unit, zh);
            },
        );
    } else {
        details(ui, r, unit, zh);
    }
}

fn details(ui: &mut egui::Ui, r: &RecycleBlockReport, unit: MeasurementUnit, zh: bool) {
    let t = &r.tear;
    ui.collapsing(
        tr(zh, "撕裂绑定与边界", "Tear binding and boundary"),
        |ui| {
            ui.label(format!(
                "{}.{} → {} → {}.{}",
                t.source.unit_id, t.source.port_name, t.stream_id, t.sink.unit_id, t.sink.port_name
            ));
            ui.label(format!(
                "T = {} K · P = {} Pa",
                r.boundary.temperature_k, r.boundary.pressure_pa
            ));
        },
    );
    if let Some(last) = r.residual_trace.last() {
        ui.collapsing(
            format!(
                "{} · {} {}",
                tr(zh, "末次组分", "Final components"),
                phase(last.phase, zh),
                last.evaluation
            ),
            |ui| {
                for (id, residual) in &r.residuals_mol_s {
                    ui.push_id(id, |ui| {
                        ui.strong(id.to_string());
                        let assumed = r
                            .assumed
                            .overall_mole_fractions
                            .get(id)
                            .map(|x| x * r.assumed.total_molar_flow_mol_s);
                        let returned = r
                            .returned
                            .overall_mole_fractions
                            .get(id)
                            .map(|x| x * r.returned.total_molar_flow_mol_s);
                        let scale = r.component_scales_mol_s.get(id).copied();
                        let scaled = scale.filter(|s| *s > 0.0).map(|s| residual.abs() / s);
                        let missing = || tr(zh, "未采集", "Not recorded").to_owned();
                        let fields = [
                            (tr(zh, "Mixer 假设入口", "Mixer assumed inlet"), assumed),
                            (
                                tr(zh, "Splitter 返回出口", "Splitter returned outlet"),
                                returned,
                            ),
                            (tr(zh, "尺度", "Scale"), scale),
                        ];
                        for (label, value) in fields {
                            ui.label(format!(
                                "{label} · {}",
                                value.map(|v| flow(v, unit)).unwrap_or_else(missing)
                            ));
                        }
                        let delta =
                            rf_ui::format_quantity_delta(*residual, QuantityKind::MolarFlow, unit)
                                .unwrap_or_else(|e| e.to_string());
                        ui.label(format!(
                            "{} · {delta}",
                            tr(zh, "返回减假设", "Returned minus assumed")
                        ));
                        ui.label(format!(
                            "{} · {} ({})",
                            tr(zh, "缩放绝对值", "Scaled absolute value"),
                            scaled.map(number).unwrap_or_else(missing),
                            tr(zh, "无量纲", "dimensionless")
                        ));
                        ui.collapsing(
                            format!("{} · {id}", tr(zh, "原始 SI 精度", "Raw SI precision")),
                            |ui| {
                                for (label, value) in fields {
                                    ui.label(format!(
                                        "{label} · {}",
                                        value.map(|v| format!("{v} mol/s")).unwrap_or_else(missing)
                                    ));
                                }
                                ui.label(format!(
                                    "{} · {residual:+e} mol/s",
                                    tr(zh, "返回减假设", "Returned minus assumed")
                                ));
                                ui.label(format!(
                                    "{} · {} ({})",
                                    tr(zh, "缩放绝对值", "Scaled absolute value"),
                                    scaled.map(|s| format!("{s:e}")).unwrap_or_else(missing),
                                    tr(zh, "无量纲", "dimensionless")
                                ));
                            },
                        );
                    });
                }
            },
        );
    }
    ui.collapsing(tr(zh, "其他检查", "Other checks"), |ui| {
        let t = r.tolerances;
        for (label, state) in [
            (tr(zh, "假设入口", "Assumed inlet"), &r.assumed),
            (tr(zh, "返回出口", "Returned outlet"), &r.returned),
        ] {
            ui.label(format!(
                "{label} · {} · T {} K · P {} Pa · F {}",
                state.id,
                state.temperature_k,
                state.pressure_pa,
                flow(state.total_molar_flow_mol_s, unit)
            ));
        }
        ui.label(format!(
            "{} {} · ΔT {} K · ΔP {} Pa",
            tr(zh, "总流量容差", "Total flow tolerance"),
            flow(t.total_absolute_flow_mol_s, unit),
            t.temperature_k,
            t.pressure_pa
        ));
    });
    let title = format!(
        "{} ({})",
        tr(zh, "完整残差轨迹", "Full residual trace"),
        r.residual_trace.len()
    );
    let header = ui.collapsing(title, |ui| render_trace(ui, r, unit, zh));
    reveal_plan_focus(&header.header_response);
}

fn render_trace(ui: &mut egui::Ui, report: &RecycleBlockReport, unit: MeasurementUnit, zh: bool) {
    let hint = tr(
        zh,
        "历史样本仅记录两项最大值，逐组分明细未采集。",
        "Historical samples record both maxima only. Component details were not collected.",
    );
    let focus = ui.add(
        egui::Label::new(hint)
            .wrap()
            .sense(egui::Sense::focusable_noninteractive()),
    );
    reveal_plan_focus(&focus);
    egui::ScrollArea::vertical()
        .id_salt("residual-samples")
        .max_height(250.0)
        .show(ui, |ui| {
            if focus.has_focus() {
                for (key, delta) in [
                    (egui::Key::ArrowDown, -30.0),
                    (egui::Key::ArrowUp, 30.0),
                    (egui::Key::PageDown, -220.0),
                    (egui::Key::PageUp, 220.0),
                ] {
                    if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key)) {
                        ui.scroll_with_delta(egui::vec2(0.0, delta));
                    }
                }
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Home)) {
                    ui.scroll_to_cursor(Some(egui::Align::Min));
                }
            }
            for s in &report.residual_trace {
                sample(ui, s, report.tolerances, unit, zh, "");
            }
            if focus.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::End))
            {
                ui.scroll_to_cursor(Some(egui::Align::Max));
            }
        });
}
