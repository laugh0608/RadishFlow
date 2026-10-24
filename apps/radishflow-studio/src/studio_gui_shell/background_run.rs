use super::*;
use radishflow_studio::{StudioBackgroundRunAction as Action, StudioRunId, WorkspaceRunPhase};

pub(super) struct PendingRunExit {
    window_id: StudioWindowHostId,
    requested_at: std::time::Instant,
    checkpoint: input_departure::ProjectDepartureCheckpoint,
}

impl ReadyAppState {
    pub(super) fn enable_background_runs(&mut self) {
        if self.background_runs.is_some() {
            return;
        }
        let owner = std::rc::Rc::new(std::cell::RefCell::new(
            radishflow_studio::StudioRunOwner::default(),
        ));
        self.background_action(Action::Install(owner.clone()));
        self.background_runs = Some(owner);
    }
    pub(super) fn background_action(&mut self, action: Action) {
        if let Err(error) = self.platform_host.background_run_action(action) {
            self.platform_host
                .record_activity_line(format!("Background calculation: {error}"));
        }
    }
    pub(super) fn background_busy(&self) -> bool {
        self.background_runs
            .as_ref()
            .is_some_and(|owner| !owner.borrow().is_idle())
    }
    pub(super) fn replace_background_host(&mut self, host: StudioGuiPlatformHost) {
        self.background_action(Action::Detach);
        self.platform_host = host;
        if let Some(owner) = self.background_runs.clone() {
            self.background_action(Action::Install(owner));
        }
    }
    pub(super) fn poll_background_runs(&mut self, ctx: &egui::Context) {
        if self.background_runs.is_none() {
            return;
        }
        self.background_action(Action::Poll {
            attached: self.read_only_project.is_none(),
        });
        if self.background_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    pub(super) fn render_background_activity(&mut self, ctx: &egui::Context) {
        let activity = self
            .background_runs
            .as_ref()
            .and_then(|owner| owner.borrow().activity());
        let Some(activity) = activity else {
            return;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let cancelling = activity.phase == WorkspaceRunPhase::CancelRequested;
        let kind = match activity.id {
            StudioRunId::Flow(_) => {
                if zh {
                    "流程"
                } else {
                    "Flowsheet"
                }
            }
            StudioRunId::Equipment(_) => {
                if zh {
                    "设备"
                } else {
                    "Equipment"
                }
            }
        };
        let phase = match activity.phase {
            WorkspaceRunPhase::Preparing => {
                if zh {
                    "准备中"
                } else {
                    "Preparing"
                }
            }
            WorkspaceRunPhase::Running => {
                if zh {
                    "运行中"
                } else {
                    "Running"
                }
            }
            WorkspaceRunPhase::CancelRequested => {
                if zh {
                    "正在取消 · 等待当前计算结束"
                } else {
                    "Cancelling · waiting for the current calculation"
                }
            }
        };
        egui::TopBottomPanel::top("background-run-activity").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label(format!("{kind} #{} · {} · {phase}", activity.number, activity.target));
                if ui.add_enabled(!cancelling, egui::Button::new(if zh { "取消本次运行" } else { "Cancel this run" })).clicked() {
                    self.background_action(Action::Cancel(activity.id.clone()));
                }
                if ui.button(if zh { "运行日志" } else { "Run log" }).clicked() {
                    self.show_background_run_details();
                }
            });
            if let Some(owner) = &self.background_runs
                && let Some(progress) = owner.borrow().flow_progress()
                && let radishflow_studio::WorkspaceSolveProgress::Solving(progress) = progress.progress {
                if let Some(unit) = progress.current_unit {
                    ui.label(if zh { format!("当前单元：{unit}") } else { format!("Current unit: {unit}") });
                }
                if let Some(total) = progress.total_units {
                    ui.label(if zh { format!("已完成 {} / {total} 个单元", progress.completed_units) }
                        else { format!("{} / {total} units completed", progress.completed_units) });
                }
            }
            if let Ok(elapsed) = activity.submitted_at.elapsed() {
                ui.small(if zh { format!("已用时 {} 秒", elapsed.as_secs()) } else { format!("Elapsed {} s", elapsed.as_secs()) });
            }
            if cancelling && activity.cancel_requested_at.is_some_and(|at| at.elapsed().is_ok_and(|time| time.as_secs() >= 5)) {
                ui.label(if zh { "任务仍在处理当前计算；结果已禁止回写，可以继续查看和编辑。" }
                    else { "The current calculation is still finishing. Its result cannot be applied; you can continue viewing and editing." });
            }
        });
    }
    pub(super) fn show_background_run_details(&mut self) {
        self.background_run_details_open = true;
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
    }
    pub(super) fn equipment_target_busy(
        &self,
        task: &rf_types::EquipmentTaskId,
        case: Option<&rf_types::EngineeringCaseId>,
    ) -> bool {
        self.background_runs.as_ref().is_some_and(|owner| {
            owner
                .borrow()
                .equipment_target_busy(self.platform_host.modeling_inputs(), task, case)
        })
    }
    pub(super) fn background_command_enabled(&self, command: &str) -> bool {
        !matches!(
            command,
            "run_panel.run_manual" | "run_panel.resume_workspace"
        ) || !self.background_busy()
    }
    pub(super) fn run_panel_background_enabled(&self, action: rf_ui::RunPanelActionId) -> bool {
        !matches!(
            action,
            rf_ui::RunPanelActionId::RunManual | rf_ui::RunPanelActionId::Resume
        ) || !self.background_busy()
    }
    pub(super) fn background_command_label<'a>(&self, command: &str, fallback: &'a str) -> &'a str {
        let fallback = self.recommendation_command_label(command, fallback);
        if !matches!(
            command,
            "run_panel.run_manual" | "run_panel.resume_workspace"
        ) {
            return fallback;
        }
        let Some(activity) = self
            .background_runs
            .as_ref()
            .and_then(|owner| owner.borrow().activity())
        else {
            return fallback;
        };
        let current = matches!(&activity.id, StudioRunId::Flow(id)
            if self.platform_host.modeling_inputs().workspace.pending_flow_run() == Some(id));
        match (self.locale, current, activity.phase) {
            (StudioShellLocale::ZhCn, _, WorkspaceRunPhase::CancelRequested) => "等待计算结束",
            (StudioShellLocale::En, _, WorkspaceRunPhase::CancelRequested) => {
                "Waiting for calculation"
            }
            (StudioShellLocale::ZhCn, true, WorkspaceRunPhase::Preparing) => "准备运行",
            (StudioShellLocale::En, true, WorkspaceRunPhase::Preparing) => "Preparing run",
            (StudioShellLocale::ZhCn, true, WorkspaceRunPhase::Running) => "运行中",
            (StudioShellLocale::En, true, WorkspaceRunPhase::Running) => "Running",
            (StudioShellLocale::ZhCn, false, _) => "计算名额被占用",
            (StudioShellLocale::En, false, _) => "Calculation busy",
        }
    }
    pub(super) fn background_run_panel_label<'a>(
        &self,
        action: rf_ui::RunPanelActionId,
        fallback: &'a str,
    ) -> &'a str {
        let command = match action {
            rf_ui::RunPanelActionId::RunManual => "run_panel.run_manual",
            rf_ui::RunPanelActionId::Resume => "run_panel.resume_workspace",
            _ => return fallback,
        };
        self.background_command_label(command, fallback)
    }
    pub(super) fn equipment_execution_label(
        &self,
        member: &rf_ui::engineering::EquipmentCaseKey,
    ) -> &'static str {
        if !self.equipment_target_busy(&member.task_id, Some(&member.case_id)) {
            return "—";
        }
        let phase = self
            .background_runs
            .as_ref()
            .and_then(|owner| owner.borrow().activity())
            .map(|a| a.phase);
        match (self.locale, phase) {
            (StudioShellLocale::ZhCn, Some(WorkspaceRunPhase::Preparing)) => "准备校核",
            (StudioShellLocale::ZhCn, Some(WorkspaceRunPhase::Running)) => "校核中",
            (StudioShellLocale::ZhCn, Some(WorkspaceRunPhase::CancelRequested)) => "正在取消",
            (_, Some(WorkspaceRunPhase::Preparing)) => "Preparing",
            (_, Some(WorkspaceRunPhase::Running)) => "Calculating",
            (_, Some(WorkspaceRunPhase::CancelRequested)) => "Cancelling",
            (_, None) => "—",
        }
    }
    pub(super) fn capture_background_palette_target(&mut self) {
        if self.command_palette.open && !self.command_palette.background_target_captured {
            self.command_palette.background_target_captured = true;
            self.command_palette.background_target = self
                .background_runs
                .as_ref()
                .and_then(|owner| owner.borrow().activity());
        }
    }
    pub(super) fn append_background_palette_items(
        &self,
        items: &mut Vec<radishflow_studio::StudioGuiWindowCommandPaletteItemModel>,
        query: &str,
    ) {
        for item in items.iter_mut() {
            item.enabled &= self.background_command_enabled(&item.command_id);
        }
        let Some(target) = &self.command_palette.background_target else {
            return;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let label = if zh {
            format!("取消运行 #{} · {}", target.number, target.target)
        } else {
            format!("Cancel run #{} · {}", target.number, target.target)
        };
        let search = format!("{label} cancel run 取消运行").to_lowercase();
        if !query
            .to_lowercase()
            .split_whitespace()
            .all(|word| search.contains(word))
        {
            return;
        }
        let enabled = self
            .background_runs
            .as_ref()
            .and_then(|owner| owner.borrow().activity())
            .is_some_and(|current| {
                current.id == target.id && current.phase != WorkspaceRunPhase::CancelRequested
            });
        let detail = if zh { "只取消这次计算，等待当前调用结束；不启动其他任务。" }
            else { "Cancel this calculation and wait for its current call to finish. No other task is started." }.to_owned();
        items.insert(
            0,
            radishflow_studio::StudioGuiWindowCommandPaletteItemModel {
                command_id: format!("background.cancel.{}", target.number),
                enabled,
                label,
                detail: detail.clone(),
                hover_text: detail,
                menu_path_text: if zh {
                    "运行 / 取消"
                } else {
                    "Run / Cancel"
                }
                .into(),
            },
        );
    }
    pub(super) fn dispatch_background_cancel_command(&mut self, command: &str) -> bool {
        if !command.starts_with("background.cancel.") {
            return false;
        }
        if let Some(target) = &self.command_palette.background_target
            && command == format!("background.cancel.{}", target.number)
        {
            self.background_action(Action::Cancel(target.id.clone()));
        }
        true
    }
    pub(super) fn render_background_history(&self, ui: &mut egui::Ui) {
        use radishflow_studio::StudioRunOutcome;
        let Some(owner) = &self.background_runs else {
            return;
        };
        let owner = owner.borrow();
        if owner.history().is_empty() {
            return;
        }
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.label(if zh { "最近运行" } else { "Recent runs" });
        for summary in owner.history().iter().rev() {
            let outcome = match summary.outcome {
                StudioRunOutcome::Succeeded => {
                    if zh {
                        "计算已返回"
                    } else {
                        "Calculation returned"
                    }
                }
                StudioRunOutcome::Failed => {
                    if zh {
                        "计算失败"
                    } else {
                        "Calculation failed"
                    }
                }
                StudioRunOutcome::Cancelled => {
                    if zh {
                        "已取消"
                    } else {
                        "Cancelled"
                    }
                }
                StudioRunOutcome::InfrastructureFailed => {
                    if zh {
                        "执行失败"
                    } else {
                        "Execution failed"
                    }
                }
            };
            let acceptance = if summary.applied {
                if zh { "已接收" } else { "Accepted" }
            } else if zh {
                "结果未接收"
            } else {
                "Result not accepted"
            };
            render_wrapped_small(
                ui,
                format!(
                    "#{} · {} · {outcome} · {acceptance}",
                    summary.activity.number, summary.activity.target
                ),
            );
            if let Some(reason) = summary.cancellation {
                use radishflow_studio::{
                    RunAuthorizationRefusal, WorkspaceRunCancelReason as Reason,
                };
                ui.small(match (zh, reason) {
                    (true, Reason::UserRequested) => "用户取消",
                    (false, Reason::UserRequested) => "Cancelled by user",
                    (true, Reason::QualificationChanged) => "输入或结果资格已变化",
                    (false, Reason::QualificationChanged) => "Inputs or result eligibility changed",
                    (true, Reason::Detached) => "已离开原工作区",
                    (false, Reason::Detached) => "Original workspace detached",
                    (true, Reason::Closing) => "关闭工作区",
                    (false, Reason::Closing) => "Workspace closing",
                    (true, Reason::Authorization(RunAuthorizationRefusal::Expired)) => {
                        "开始前授权已过期"
                    }
                    (false, Reason::Authorization(RunAuthorizationRefusal::Expired)) => {
                        "Authorization expired before start"
                    }
                    (true, Reason::Authorization(RunAuthorizationRefusal::Changed)) => {
                        "运行授权已变化"
                    }
                    (false, Reason::Authorization(RunAuthorizationRefusal::Changed)) => {
                        "Run authorization changed"
                    }
                });
            }
            if let Ok(duration) = summary
                .finished_at
                .duration_since(summary.activity.submitted_at)
            {
                ui.small(if zh {
                    format!("用时 {:.2} 秒", duration.as_secs_f64())
                } else {
                    format!("Elapsed {:.2} s", duration.as_secs_f64())
                });
            }
            if let Some(code) = &summary.diagnostic_code {
                ui.small(code);
            }
        }
        ui.separator();
    }
    pub(super) fn begin_background_exit(&mut self, window_id: StudioWindowHostId) {
        if self.pending_run_exit.is_some() {
            return;
        }
        self.background_action(Action::BeginShutdown);
        self.pending_run_exit = Some(PendingRunExit {
            window_id,
            requested_at: std::time::Instant::now(),
            checkpoint: self.project_departure_checkpoint(),
        });
    }
    pub(super) fn return_from_background_exit(&mut self) {
        self.pending_run_exit = None;
        self.project_open.departure_checkpoint = None;
        self.project_open.pending_close_window_confirmation = None;
        self.background_action(Action::ResumeSubmissions);
    }
    pub(super) fn render_background_exit(&mut self, ctx: &egui::Context) -> bool {
        let Some(wait) = &self.pending_run_exit else {
            return false;
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        if !self.background_busy() {
            let wait = self.pending_run_exit.take().expect("pending exit");
            self.project_open.departure_checkpoint = Some(wait.checkpoint);
            if self.validate_project_departure()
                && self.close_window_without_confirmation(wait.window_id)
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                self.background_action(Action::ResumeSubmissions);
            }
            return true;
        }
        let waited = wait.requested_at.elapsed().as_secs() >= 5;
        let zh = self.locale == StudioShellLocale::ZhCn;
        let mut return_to_workspace =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.platform_host.document().metadata.title);
        });
        egui::Modal::new(egui::Id::new("background-exit-wait")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 480.0));
            ui.heading(if zh { "正在结束计算" } else { "Finishing calculation" });
            ui.spinner();
            ui.label(if zh { "已请求取消。当前计算结束后将关闭工作区。" } else { "Cancellation requested. The workspace will close when the current calculation finishes." });
            if waited { ui.label(if zh { "仍在等待当前计算；不会强制终止线程。" } else { "Still waiting for the current calculation to finish." }); }
            ui.label(if zh { "返回会保留当前内容与未提交输入，已取消的任务不会自动恢复。" } else { "Returning preserves your content and drafts. The cancelled task will not restart automatically." });
            if ui.button(if zh { "返回工作区" } else { "Return to workspace" }).clicked() { return_to_workspace = true; }
        });
        if return_to_workspace {
            self.return_from_background_exit();
        }
        true
    }
}
