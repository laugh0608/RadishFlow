//! Local run-page presentation. Formal inputs and qualified results stay in the app.
use super::*;
use radishflow_studio::{
    StudioBackgroundRunAction, flow_actions::FlowEditContext, run_actions::StudioRunAccess,
};
use rf_solver::component_balance::{MaterialBalancePlan, MixerComponentBalanceProblem};
use rf_ui::material_balance::{
    MaterialBalanceDraft, MaterialBalanceResultView, MaterialBalanceTable,
};

#[derive(Default)]
pub(super) struct MaterialPanelState {
    pub draft: Option<MaterialBalanceDraft>,
    notice: Option<String>,
}
fn label(zh: bool, cn: &'static str, en: &'static str) -> &'static str {
    if zh { cn } else { en }
}
impl ReadyAppState {
    fn material_action(&mut self, action: StudioBackgroundRunAction) {
        match self.platform_host.background_run_action(action) {
            Ok(reply) => {
                self.material.notice = None;
                if let Some(submission) = reply.submission {
                    self.record_run_reply(&submission);
                }
            }
            Err(error) => {
                self.material.notice = Some(error.to_string());
            }
        }
    }
    pub(super) fn render_material_balance(&mut self, ui: &mut egui::Ui) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.heading(label(zh, "物料衡算", "Material balance"));
        ui.label(label(
            zh,
            "两股固定进料连接终端 Mixer，计算组分流量和组成。",
            "Component flows and composition for two fixed feeds into one terminal mixer.",
        ));
        let app = self.platform_host.modeling_inputs();
        let workspace = &app.workspace;
        if self
            .material
            .draft
            .as_ref()
            .is_some_and(|d| !d.belongs_to(workspace))
        {
            self.material = MaterialPanelState::default();
        }
        let targets: Vec<_> = workspace
            .document
            .flowsheet
            .units
            .values()
            .filter(|u| {
                rf_ui::BuiltinUnitKind::parse(&u.kind) == Some(rf_ui::BuiltinUnitKind::Mixer)
            })
            .map(|u| (u.id.clone(), u.name.clone()))
            .collect();
        if self.material.draft.is_none() {
            let proposed = workspace
                .material_balance_plan()
                .cloned()
                .map(Ok)
                .or_else(|| {
                    targets.first().map(|(id, _)| {
                        MaterialBalancePlan::for_mixer(&workspace.document.flowsheet, id.clone())
                    })
                });
            match proposed {
                Some(Ok(plan)) => {
                    self.material.draft = Some(MaterialBalanceDraft::from_plan(workspace, &plan))
                }
                Some(Err(error)) => {
                    ui.label(error.to_string());
                    return;
                }
                None => {
                    ui.label(label(
                        zh,
                        "请先建立 Feed + Feed → Mixer 的正式连接。",
                        "Create formal Feed + Feed → Mixer connections first.",
                    ));
                    return;
                }
            }
        }
        let mut draft = self.material.draft.take().expect("initialized draft");
        let previous = draft.mixer_id.clone();
        egui::ComboBox::from_id_salt("material-target")
            .selected_text(
                targets
                    .iter()
                    .find(|(id, _)| id == &draft.mixer_id)
                    .map(|(_, n)| n.as_str())
                    .unwrap_or(draft.mixer_id.as_str()),
            )
            .show_ui(ui, |ui| {
                for (id, name) in &targets {
                    ui.selectable_value(&mut draft.mixer_id, id.clone(), name);
                }
            });
        if draft.mixer_id != previous {
            match MaterialBalancePlan::for_mixer(
                &workspace.document.flowsheet,
                draft.mixer_id.clone(),
            ) {
                Ok(plan) => draft = MaterialBalanceDraft::from_plan(workspace, &plan),
                Err(error) => self.material.notice = Some(error.to_string()),
            }
        }
        ui.small(label(
            zh,
            "计算设置 · 本次会话 · 尺度固定使用 mol/s",
            "Session settings · scales use mol/s",
        ));
        let labels = if zh {
            [
                "入口 A 尺度 / mol/s",
                "入口 B 尺度 / mol/s",
                "出口尺度 / mol/s",
                "绝对容差 / mol/s",
                "缩放容差",
            ]
        } else {
            [
                "Inlet A scale / mol/s",
                "Inlet B scale / mol/s",
                "Outlet scale / mol/s",
                "Absolute tolerance / mol/s",
                "Scaled tolerance",
            ]
        };
        for (i, title) in labels.into_iter().enumerate() {
            ui.label(title);
            ui.add(
                egui::TextEdit::singleline(&mut draft.text[i])
                    .id(egui::Id::new(("material-setting", i)))
                    .desired_width(ui.available_width().min(300.0)),
            );
        }
        let plan = draft.plan();
        let applied = draft.is_applied(workspace);
        let expected = FlowEditContext::capture(workspace);
        let blocker = plan
            .as_ref()
            .ok()
            .and_then(|plan| {
                MixerComponentBalanceProblem::validate_specification(
                    &workspace.document.flowsheet,
                    &plan.specification,
                )
                .err()
            })
            .map(|e| e.to_string());
        let has_drafts = workspace.drafts.fields.values().any(|d| match d {
            rf_ui::DraftValue::Numeric(n) => {
                n.is_pending() || n.draft_validation() == rf_ui::DraftValidationState::Invalid
            }
            rf_ui::DraftValue::Text(d)
            | rf_ui::DraftValue::Choice(d)
            | rf_ui::DraftValue::Number(d) => {
                d.is_dirty || d.validation == rf_ui::DraftValidationState::Invalid
            }
        });
        let activity = self
            .background_runs
            .as_ref()
            .and_then(|o| o.borrow().activity());
        let access = if self.read_only_project.is_some() {
            StudioRunAccess::ReadOnly
        } else {
            StudioRunAccess::Allowed
        };
        let editable = access == StudioRunAccess::Allowed;
        if plan.is_err() {
            ui.colored_label(
                egui::Color32::from_rgb(180, 40, 40),
                label(
                    zh,
                    "输入未完成或不是正有限数，草稿已保留。",
                    "Enter positive finite values; unfinished drafts are preserved.",
                ),
            );
        }
        if !applied {
            ui.label(label(
                zh,
                "设置尚未应用，请应用后计算。",
                "Apply settings before calculating.",
            ));
        }
        if let Some(reason) = &blocker {
            ui.label(reason);
        }
        if has_drafts {
            ui.label(label(
                zh,
                "请先应用或撤销流程输入草稿。",
                "Apply or revert flow input drafts first.",
            ));
        }
        ui.horizontal_wrapped(|ui| {
            if !applied
                && let Some(formal) = self
                    .platform_host
                    .modeling_inputs()
                    .workspace
                    .material_balance_plan()
                && ui
                    .button(label(zh, "恢复已应用设置", "Restore applied settings"))
                    .clicked()
            {
                draft = MaterialBalanceDraft::from_plan(
                    &self.platform_host.modeling_inputs().workspace,
                    formal,
                );
            }
            if ui
                .add_enabled(
                    editable && plan.is_ok() && !applied,
                    egui::Button::new(label(zh, "应用设置", "Apply settings")),
                )
                .clicked()
            {
                self.material_action(StudioBackgroundRunAction::ConfigureMaterial {
                    expected: expected.clone(),
                    access,
                    plan: Box::new(plan.as_ref().expect("enabled valid plan").clone()),
                });
            }
            if ui
                .add_enabled(
                    editable && applied && blocker.is_none() && !has_drafts && activity.is_none(),
                    egui::Button::new(label(zh, "计算物料衡算", "Calculate material balance")),
                )
                .clicked()
            {
                self.material_action(StudioBackgroundRunAction::RunMaterial { expected, access });
            }
            if let Some(activity) = &activity
                && matches!(activity.id, radishflow_studio::StudioRunId::Material(_))
                && ui
                    .button(label(zh, "取消本次计算", "Cancel this calculation"))
                    .clicked()
            {
                self.material_action(StudioBackgroundRunAction::Cancel(activity.id.clone()));
            }
        });
        if activity.is_some() {
            ui.label(label(
                zh,
                "已有计算在途；取消后等待退出再释放名额。",
                "A calculation is active; cancellation releases the worker after it exits.",
            ));
        }
        self.material.draft = Some(draft);
        if let Some(notice) = &self.material.notice {
            ui.label(notice);
        }
        ui.separator();
        // Re-read after any settings action. A projection captured earlier this frame may be stale.
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self
            .material
            .draft
            .as_ref()
            .is_some_and(|d| d.is_applied(workspace))
        {
            return;
        }
        let table = match MaterialBalanceTable::capture(workspace) {
            Ok(table) => table,
            Err(error) => {
                ui.label(error.to_string());
                return;
            }
        };
        match workspace.material_balance_result() {
            MaterialBalanceResultView::Missing => {
                ui.label(label(
                    zh,
                    "尚无本次物料结果。",
                    "No material result for this calculation.",
                ));
            }
            MaterialBalanceResultView::Stale { .. } => {
                ui.label(label(
                    zh,
                    "结果已过期，请重新计算。",
                    "Result is stale. Calculate again.",
                ));
            }
            MaterialBalanceResultView::Failed { failure, .. } => {
                ui.strong(label(zh, "本次物料计算失败", "Material calculation failed"));
                ui.label(failure.to_string());
                let error = failure.diagnostic();
                let units = error.context().related_unit_ids().to_vec();
                let streams = error.context().related_stream_ids().to_vec();
                for id in units {
                    if ui
                        .button(format!("{} {}", label(zh, "定位单元", "Locate unit"), id))
                        .clicked()
                    {
                        self.dispatch_ui_command(format!("inspector.focus_unit:{id}"));
                    }
                }
                for id in streams {
                    if ui
                        .button(format!("{} {}", label(zh, "定位流股", "Locate stream"), id))
                        .clicked()
                    {
                        self.dispatch_ui_command(format!("inspector.focus_stream:{id}"));
                    }
                }
            }
            MaterialBalanceResultView::Current { solution, origin } => {
                ui.strong(format!(
                    "{} · {} {}",
                    label(zh, "当前物料结果", "Current material result"),
                    label(zh, "输入修订", "Input revision"),
                    origin.qualification().input().revision()
                ));
                let table = table.expect("current table");
                if ui.available_width() < 600.0 {
                    for row in &table.rows {
                        ui.separator();
                        ui.strong(&row[0]);
                        for (i, value) in row.iter().enumerate().skip(1) {
                            ui.label(format!("{}: {value}", label_for_column(zh, i)));
                        }
                    }
                } else {
                    egui::Grid::new("material-result-table")
                        .striped(true)
                        .show(ui, |ui| {
                            for i in 0..5 {
                                ui.strong(label_for_column(zh, i));
                            }
                            ui.end_row();
                            for row in &table.rows {
                                for value in row {
                                    ui.label(value);
                                }
                                ui.end_row();
                            }
                        });
                }
                ui.label(format!(
                    "{}: {}",
                    label(zh, "出口总流量", "Total outlet flow"),
                    table.total_flow
                ));
                if ui
                    .button(label(
                        zh,
                        "复制当前计算表",
                        "Copy current calculation table",
                    ))
                    .clicked()
                    && let Some(text) = table.tsv(&self.platform_host.modeling_inputs().workspace)
                {
                    ui.ctx().copy_text(text);
                }
                let raw = solution
                    .evaluation
                    .residuals_mol_s
                    .iter()
                    .map(|v| v.abs())
                    .fold(0.0_f64, f64::max);
                let scaled = solution
                    .evaluation
                    .scaled_residuals
                    .iter()
                    .map(|v| v.abs())
                    .fold(0.0_f64, f64::max);
                ui.collapsing(label(zh, "计算依据", "Calculation evidence"), |ui| {
                    ui.label(origin.implementation());
                    ui.label(format!(
                        "{}: {} · {}: {}",
                        label(zh, "求解行数", "Solved rows"),
                        solution.solved_rows,
                        label(zh, "残差求值次数", "Residual evaluations"),
                        solution.residual_evaluations
                    ));
                    ui.label(format!(
                        "{}: {raw:.3e} mol/s / {:.3e} mol/s",
                        label(
                            zh,
                            "最大原始残差 / 容差",
                            "Maximum raw residual / tolerance"
                        ),
                        solution.options.absolute_tolerance_mol_s
                    ));
                    ui.label(format!(
                        "{}: {scaled:.3e} / {:.3e}",
                        label(
                            zh,
                            "最大缩放残差 / 容差",
                            "Maximum scaled residual / tolerance"
                        ),
                        solution.options.scaled_tolerance
                    ));
                });
                ui.small(label(zh,"本结果仅包含物料量；温度、焓与相态由对应流程计算提供。","Material quantities only; thermal and phase results come from their respective flow calculations."));
            }
        }
    }
}
fn label_for_column(zh: bool, i: usize) -> &'static str {
    if zh {
        ["组分", "入口 A", "入口 B", "出口", "出口摩尔分率"][i]
    } else {
        [
            "Component",
            "Inlet A",
            "Inlet B",
            "Outlet",
            "Outlet mole fraction",
        ][i]
    }
}
