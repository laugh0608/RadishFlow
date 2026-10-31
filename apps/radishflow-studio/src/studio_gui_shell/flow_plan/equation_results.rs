use super::*;
use rf_solver::equation_oriented::*;
pub(in crate::studio_gui_shell) fn progress_text(p: &EquationProgress, zh: bool) -> String {
    let stage = match p.stage {
        EquationStage::Assemble => tr(zh, "装配物料方程", "Assembling material equations"),
        EquationStage::StructuralAnalysis => tr(zh, "结构匹配", "Structural matching"),
        EquationStage::LinearEvaluation => tr(zh, "条件线性评价", "Conditional linear evaluation"),
        EquationStage::BracketEndpoint => tr(zh, "区间端点评价", "Bracket endpoint evaluation"),
        EquationStage::IntervalEvaluation => {
            tr(zh, "二分区间评价", "Bisection interval evaluation")
        }
        EquationStage::Factorization => tr(zh, "矩阵分解", "Matrix factorization"),
        EquationStage::ConditionSolve => tr(zh, "条件估计", "Condition estimation"),
        EquationStage::FullJacobianCheck => tr(zh, "完整 Jacobian 检查", "Full Jacobian check"),
        EquationStage::FinalUnitVerification => tr(zh, "真实单元复核", "Actual unit verification"),
        EquationStage::GlobalVerification => tr(zh, "总体物料复核", "Global material verification"),
        EquationStage::Completing => tr(zh, "准备接收结果", "Preparing result reception"),
    };
    let mut text = stage.to_string();
    if let Some(u) = &p.unit {
        text.push_str(&format!(" · {u}"));
    }
    if let Some(i) = p.iteration {
        text.push_str(&format!(" · #{}", i + 1));
    }
    if let Some(c) = p.column {
        text.push_str(&format!(" · {} {}", tr(zh, "列", "column"), c + 1));
    }
    text
}
impl ReadyAppState {
    pub(super) fn render_equation_current(&self, ui: &mut egui::Ui, r: &EquationResult) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.strong(tr(zh, "当前 EO 工况", "Current EO condition"));
        ui.small(tr(zh,"物料方程与真实单元已复核；输入尚未采纳。热量与相平衡不在本次方程范围。","Material equations and actual units verified; inputs are not automatically adopted. Energy and phase equilibrium are outside these equations."));
        if let Some(s) = &r.evidence.solved_parameter {
            ui.label(format!("{}.outlet_a_split_fraction", s.binding.unit));
            ui.label(format!(
                "{}: {} → {}: {:.12}",
                tr(zh, "工程存储值", "Stored input"),
                radishflow_studio::modular_evidence::diagnostic_number(s.stored_value),
                tr(zh, "求得值", "Solved value"),
                s.value
            ));
        }
        let units = self
            .platform_host
            .modeling_inputs()
            .workspace
            .project_presentation
            .display_units();
        let lines = radishflow_studio::equation_evidence::result_lines(r, units);
        ui.collapsing(
            tr(
                zh,
                "目标、方程、括区与最终复核",
                "Target, equations, bracket and final verification",
            ),
            |ui| {
                for line in &lines.lines {
                    ui.label(line);
                }
            },
        );
        let access = if self.read_only_project.is_some() {
            FlowEditAccess::ReadOnly
        } else {
            FlowEditAccess::Editable
        };
        let eligibility = radishflow_studio::equation_actions::preview_equation_adoption(
            self.platform_host.modeling_inputs(),
            access,
            self.background_busy(),
        );
        if r.evidence.solved_parameter.is_some() {
            let response = ui.add_enabled(
                eligibility.is_ok(),
                egui::Button::new(tr(zh, "预览采纳到输入…", "Preview adoption into input…")),
            );
            if let Err(error) = eligibility {
                response.on_hover_text(error.to_string());
                false
            } else {
                response.clicked()
            }
        } else {
            false
        }
    }
    pub(super) fn render_equation_adoption(&mut self, ctx: &egui::Context) {
        let Some(preview) = self.flow_plan.adoption.clone() else {
            return;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let mut confirm = false;
        let mut cancel = false;
        let response=egui::Modal::new(egui::Id::new("eo-adoption")).show(ctx,|ui|{
            ui.set_max_width(400.0f32.min(ctx.screen_rect().width()-32.0));
            ui.heading(tr(zh,"采纳求得比例到工程输入","Adopt solved fraction into input"));
            ui.label(format!("{}.outlet_a_split_fraction",preview.binding().unit));
            ui.label(format!("{} → {:.12}",radishflow_studio::modular_evidence::diagnostic_number(preview.stored_value()),preview.solved_value()));
            ui.label(tr(zh,"只修改这一物理输入；反算计划保留，当前结果转为 Stale。可撤销，但撤销不会恢复旧结果资格；不会自动运行。","Changes this physical input only. The inverse plan is retained and results become Stale. Undo does not restore old result eligibility. No automatic run."));
            ui.horizontal_wrapped(|ui|{
                let button=ui.button(tr(zh,"取消","Cancel"));if std::mem::take(&mut self.flow_plan.adoption_focus_first){button.request_focus();}cancel=button.clicked();
                confirm=ui.button(tr(zh,"确认采纳","Confirm adoption")).clicked();
            });
        });
        if response.should_close() {
            cancel = true;
        }
        if cancel {
            self.flow_plan.adoption = None;
        } else if confirm {
            let access = if self.read_only_project.is_some() {
                FlowEditAccess::ReadOnly
            } else {
                FlowEditAccess::Editable
            };
            self.plan_action(Action::AdoptEquationCandidate { access, preview });
            self.flow_plan.adoption = None;
        }
    }
}
