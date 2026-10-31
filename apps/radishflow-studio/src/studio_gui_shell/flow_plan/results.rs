use super::*;
use rf_ui::result_query::FlowResultView;
impl ReadyAppState {
    pub(super) fn render_partitioned_results(&mut self, ui: &mut egui::Ui) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let workspace = &self.platform_host.modeling_inputs().workspace;
        ui.separator();
        ui.heading(tr(zh, "流程结果", "Flow results"));
        match workspace.flow_result() {
            FlowResultView::Current { snapshot, origin } => {
                ui.strong("Current");
                if let rf_ui::FlowResultRecord::Partitioned(part) = snapshot {
                    ui.small(tr(
                        zh,
                        "固定温压回流；整体物料结果，不代表完整能量求解。",
                        "Fixed-T/P recycle and material closure; this is not a full energy solve.",
                    ));
                    for (index, block) in part.result.blocks.iter().enumerate() {
                        ui.collapsing(format!("{} {} · {:?} · {}",tr(zh,"依赖块","Block"),index+1,block.kind,block.members.iter().map(ToString::to_string).collect::<Vec<_>>().join(" + ")), |ui| {
                            if let Some(report) = &block.recycle {
                                ui.label(format!("{}: {} · {}: {}",tr(zh,"迭代求值","Iteration evaluations"),report.iteration_evaluations,tr(zh,"独立复核","Independent verification"),report.verification_evaluations));
                                ui.label(format!("T = {} K · P = {} Pa",report.boundary.temperature_k,report.boundary.pressure_pa));
                                ui.label(format!("{}: {}.{} → {} → {}.{}",tr(zh,"撕裂绑定","Tear"),report.tear.source.unit_id,report.tear.source.port_name,report.tear.stream_id,report.tear.sink.unit_id,report.tear.sink.port_name));
                                for (id, residual) in &report.residuals_mol_s { ui.label(format!("{id}: r = {residual:.6e} mol/s · scale = {:.6e} mol/s",report.component_scales_mol_s[id])); }
                                ui.label(format!("tol: {:.3e} mol/s · scaled {:.3e} · total {:.3e} mol/s · T {:.3e} K · P {:.3e} Pa",report.tolerances.component_absolute_flow_mol_s,report.tolerances.scaled_flow,report.tolerances.total_absolute_flow_mol_s,report.tolerances.temperature_k,report.tolerances.pressure_pa));
                                ui.collapsing(tr(zh,"假设值与返回值","Assumed and returned states"), |ui| { state(ui,tr(zh,"假设入口","Assumed inlet"),&report.assumed); state(ui,tr(zh,"返回出口","Returned outlet"),&report.returned); });
                                ui.collapsing(tr(zh,"残差轨迹","Residual trace"), |ui| { for sample in &report.residual_trace { ui.label(format!("{:?} {}: {:.6e} mol/s · scaled {:.6e}",sample.phase,sample.evaluation,sample.max_absolute_mol_s,sample.max_scaled)); } });
                            }
                            for unit in &block.final_units { ui.collapsing(format!("{} · {}",unit.unit_id,tr(zh,"最终实际输入 / 输出","Final actual inputs / outputs")), |ui| { for stream in &unit.consumed_streams { state(ui,tr(zh,"消耗","Consumed"),stream); } for stream in &unit.produced_streams { state(ui,tr(zh,"产出","Produced"),stream); } }); }
                        });
                    }
                    let cost = &part.result.costs;
                    ui.label(format!(
                        "{}: {} · thermo {} · flash {}",
                        tr(zh, "实际局部调用", "Actual local calls"),
                        cost.local_unit_calls,
                        cost.thermo_calls,
                        cost.flash_calls
                    ));
                }
                ui.collapsing(tr(zh, "执行依据", "Execution evidence"), |ui| {
                    for (key, value) in radishflow_studio::flow_origin_lines(Some(origin)) {
                        ui.label(format!("{key}: {value}"));
                    }
                });
            }
            FlowResultView::Stale { .. } => {
                ui.label(tr(
                    zh,
                    "Stale · 输入或计划已改变；重新运行后查看数值。",
                    "Stale · input or plan changed; rerun to read values.",
                ));
            }
            FlowResultView::Missing { .. } => {
                ui.label(tr(
                    zh,
                    "Missing · 尚无本次会话的合格结果。",
                    "Missing · no qualified result in this session.",
                ));
            }
            FlowResultView::Failed { diagnostic, .. } => {
                ui.label(format!("Failed · {}", diagnostic.primary_message));
                if let Some(failure) = workspace.partitioned_failure_evidence() {
                    ui.label(format!(
                        "{:?} · block {:?} · unit {:?} · {} local calls",
                        failure.kind,
                        failure.stage.block.map(|b| b + 1),
                        failure.stage.unit,
                        failure.costs.local_unit_calls
                    ));
                    if let Some(report) = &failure.active_recycle {
                        for sample in &report.residual_trace {
                            ui.label(format!(
                                "{:?} {}: {:.3e} mol/s · {:.3e}",
                                sample.phase,
                                sample.evaluation,
                                sample.max_absolute_mol_s,
                                sample.max_scaled
                            ));
                        }
                    }
                    ui.small(tr(
                        zh,
                        "仅诊断证据；没有发布部分流程结果。",
                        "Diagnostic evidence only; no partial flow result was published.",
                    ));
                }
            }
        }
    }
}
fn state(ui: &mut egui::Ui, title: &str, stream: &rf_model::MaterialStreamState) {
    ui.label(format!(
        "{title} {} · T {} K · P {} Pa · F {:.9e} mol/s",
        stream.id, stream.temperature_k, stream.pressure_pa, stream.total_molar_flow_mol_s
    ));
    ui.label(
        stream
            .overall_mole_fractions
            .iter()
            .map(|(id, value)| format!("{id}: {:.9e} mol/s", value * stream.total_molar_flow_mol_s))
            .collect::<Vec<_>>()
            .join(" · "),
    );
}
