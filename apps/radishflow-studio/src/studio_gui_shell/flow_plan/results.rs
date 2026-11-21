use super::*;
use rf_ui::result_query::FlowResultView;
impl ReadyAppState {
    pub(super) fn render_flow_results(&mut self, ui: &mut egui::Ui) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let workspace = &self.platform_host.modeling_inputs().workspace;
        ui.separator();
        ui.heading(tr(zh, "流程结果", "Flow results"));
        let mut adopt = false;
        let mut locate = None;
        match workspace.flow_result() {
            FlowResultView::Current { snapshot, origin } => {
                ui.strong("Current");
                if let Some(snapshot) = snapshot.acyclic().filter(|s| !s.energy_balances.is_empty())
                {
                    ui.strong(tr(zh, "能量衡算", "Energy balance"));
                    for (id, balance) in &snapshot.energy_balances {
                        ui.label(format!(
                            "{id} · {} {:+.6} W",
                            tr(zh, "热负荷", "Heat duty"),
                            balance.heat_duty_w
                        ));
                        for output in snapshot
                            .steps
                            .iter()
                            .filter(|step| &step.unit_id == id)
                            .flat_map(|step| &step.streams)
                        {
                            ui.label(format!(
                                "{} {:.6} K",
                                tr(zh, "出口温度", "Outlet temperature"),
                                output.temperature_k
                            ));
                        }
                        ui.label(format!(
                            "Ḣin {:.6} W · Ḣout {:.6} W",
                            balance.inlet_enthalpy_flow_w, balance.outlet_enthalpy_flow_w
                        ));
                        ui.label(format!(
                            "{} {:.6e} W · {} {:.6e}",
                            tr(zh, "能量残差", "Energy residual"),
                            balance.residual_w,
                            tr(zh, "缩放残差", "Scaled residual"),
                            balance.scaled_residual
                        ));
                    }
                }
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
                if let rf_ui::FlowResultRecord::Mixed(s) = snapshot {
                    locate = self.render_mixed_current(ui, &s.result, origin);
                }
                if let rf_ui::FlowResultRecord::SinglePhaseRecycle(s) = snapshot {
                    single_phase_recycle(ui, &s.result, zh);
                }
                if let rf_ui::FlowResultRecord::EquationOriented(s) = snapshot {
                    adopt = self.render_equation_current(ui, &s.result);
                }
                if let rf_ui::FlowResultRecord::Simultaneous(s) = snapshot {
                    self.render_modular_current(ui, &s.result);
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
                if let Some(failure) = workspace.mixed_failure_evidence() {
                    ui.strong(tr(
                        zh,
                        "仅失败诊断；没有发布部分结果。",
                        "Failure diagnostics only; no partial results published.",
                    ));
                    ui.label(super::mixed_results::progress_text(&failure.progress, zh));
                    if let Some(code) = failure.error.context().diagnostic_code() {
                        ui.label(code);
                    }
                    if let Some(report) = &failure.recycle {
                        let unit = workspace
                            .project_presentation
                            .display_units()
                            .unit_for(rf_types::units::QuantityKind::MolarFlow);
                        if super::mixed_results::render_recycle_report(ui, report, unit, zh, false)
                        {
                            locate = Some(report.tear.stream_id.clone());
                        }
                    } else {
                        for id in failure.error.context().related_stream_ids() {
                            let button = ui.add_enabled(
                                workspace.document.flowsheet.streams.contains_key(id),
                                egui::Button::new(format!("{} {id}", tr(zh, "定位", "Locate"))),
                            );
                            reveal_plan_focus(&button);
                            if button.clicked() {
                                locate = Some(id.clone());
                            }
                        }
                    }
                    for line in radishflow_studio::mixed_evidence::failure_lines(
                        failure,
                        workspace.project_presentation.display_units(),
                    )
                    .lines
                    {
                        ui.label(line);
                    }
                }
                if let Some(failure) = workspace.equation_failure_evidence() {
                    ui.strong(tr(
                        zh,
                        "仅失败诊断；没有发布部分结果。",
                        "Failure diagnostics only; no partial results published.",
                    ));
                    ui.label(super::equation_results::progress_text(
                        &failure.progress,
                        zh,
                    ));
                    for line in radishflow_studio::equation_evidence::failure_lines(
                        failure,
                        workspace.project_presentation.display_units(),
                    )
                    .lines
                    {
                        ui.label(line);
                    }
                }
                if let Some(failure) = workspace.simultaneous_failure_evidence() {
                    ui.strong(tr(
                        zh,
                        "仅失败诊断；没有发布部分结果。",
                        "Failure diagnostics only; no partial results published.",
                    ));
                    ui.label(super::modular_results::progress_text(&failure.progress, zh));
                    for line in radishflow_studio::modular_evidence::failure_lines(
                        failure,
                        workspace.project_presentation.display_units(),
                    )
                    .lines
                    {
                        ui.label(line);
                    }
                }
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
        if let Some(stream) = locate {
            self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(
                &rf_ui::InspectorTarget::Stream(stream),
            ));
        }
        if adopt {
            let access = if self.read_only_project.is_some() {
                FlowEditAccess::ReadOnly
            } else {
                FlowEditAccess::Editable
            };
            match radishflow_studio::equation_actions::preview_equation_adoption(
                self.platform_host.modeling_inputs(),
                access,
                self.background_busy(),
            ) {
                Ok(p) => {
                    self.flow_plan.adoption = Some(p);
                    self.flow_plan.adoption_focus_first = true;
                }
                Err(e) => self.flow_plan.notice = Some(e.to_string()),
            }
        }
    }
}

fn single_phase_recycle(
    ui: &mut egui::Ui,
    result: &rf_solver::single_phase_recycle::SinglePhaseRecycleResult,
    zh: bool,
) {
    use rf_solver::single_phase_recycle::single_phase_enthalpy_flow_w;
    let verification = &result.verification;
    let tear = &result.tear;
    let balance = result.overall_energy_balance;
    ui.strong(tr(
        zh,
        "单相显热回流复核",
        "Single-vapor recycle verification",
    ));
    ui.label(format!(
        "{} · {}.{} → {}.{}",
        tear.stream_id,
        tear.source.unit_id,
        tear.source.port_name,
        tear.sink.unit_id,
        tear.sink.port_name,
    ));
    let product = result
        .final_units
        .iter()
        .find(|u| u.unit_id == tear.source.unit_id)
        .and_then(|u| u.produced_streams.iter().find(|s| s.id != tear.stream_id));
    for (label, stream) in [
        (tr(zh, "产品", "Product"), product),
        (tr(zh, "回流", "Recycle"), Some(&verification.returned)),
    ] {
        if let Some(stream) = stream
            && let Ok(enthalpy) = single_phase_enthalpy_flow_w(stream)
        {
            ui.label(format!(
                "{label} · {} · {:.6} mol/s · {:.6} K · {:.6} W",
                stream.id, stream.total_molar_flow_mol_s, stream.temperature_k, enthalpy
            ));
        }
    }
    ui.label(format!(
        "{} {:+.6} W · {} {:.6e} W · {} {:.6e}",
        tr(zh, "热负荷", "Heat duty"),
        balance.heat_duty_w,
        tr(zh, "整体能量残差", "Overall energy residual"),
        balance.residual_w,
        tr(zh, "缩放残差", "Scaled residual"),
        balance.scaled_residual
    ));
    ui.label(format!(
        "{} {:.6e} mol/s · {} {:.6e} W",
        tr(zh, "最大组分残差", "Maximum component residual"),
        verification.residual.material.max_absolute_mol_s,
        tr(zh, "撕裂焓流残差", "Tear enthalpy-flow residual"),
        verification.residual.enthalpy_flow_residual_w
    ));
    ui.collapsing(
        tr(zh, "撕裂试值与返回值", "Assumed and returned tear states"),
        |ui| {
            state(
                ui,
                tr(zh, "假设入口", "Assumed inlet"),
                &verification.assumed,
            );
            state(
                ui,
                tr(zh, "返回出口", "Returned outlet"),
                &verification.returned,
            );
            for (id, residual) in &verification.component_residuals_mol_s {
                ui.label(format!("{id} · {residual:.6e} mol/s"));
            }
        },
    );
    ui.collapsing(tr(zh, "残差轨迹", "Residual trace"), |ui| {
        for sample in &result.trace {
            ui.label(format!(
                "{:?} {} · {:.6e} mol/s · {:.6e} W",
                sample.material.phase,
                sample.material.evaluation,
                sample.material.max_absolute_mol_s,
                sample.enthalpy_flow_residual_w
            ));
        }
    });
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
