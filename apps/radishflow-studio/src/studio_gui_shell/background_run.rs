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
        match self.platform_host.background_run_action(action) {
            Ok(reply) => {
                if let Some(submission) = reply.submission {
                    self.record_run_reply(&submission);
                }
                let zh = self.locale == StudioShellLocale::ZhCn;
                let message = match reply.cancellation {
                    Some(radishflow_studio::WorkspaceRunCancelReply::AlreadyFinished) => {
                        Some(if zh {
                            "该次运行已结束。"
                        } else {
                            "This run has already finished."
                        })
                    }
                    Some(radishflow_studio::WorkspaceRunCancelReply::UnknownRun) => Some(if zh {
                        "目标运行不在当前运行记录中。"
                    } else {
                        "The target run is not in the current run history."
                    }),
                    _ => None,
                };
                if let Some(message) = message {
                    self.platform_host.record_activity_line(message);
                }
            }
            Err(error) => self
                .platform_host
                .record_activity_line(format!("Background calculation: {error}")),
        }
    }
    pub(super) fn record_run_reply(
        &mut self,
        reply: &radishflow_studio::run_actions::StudioRunReply,
    ) {
        use radishflow_studio::run_actions::{
            StudioRunRejection as Rejection, StudioRunReply as Reply,
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let message: String = match reply {
            Reply::Accepted(_) => {
                self.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
                if zh {
                    "计算已受理。"
                } else {
                    "Calculation accepted."
                }
                .into()
            }
            Reply::AlreadyRunning(_) => if zh {
                "本次输入已有计算正在运行。"
            } else {
                "A calculation for these inputs is already running."
            }
            .into(),
            Reply::Busy(_) => if zh {
                "已有计算占用运行名额，本次未启动。"
            } else {
                "Another calculation occupies the worker. This request did not start."
            }
            .into(),
            Reply::Closing => if zh {
                "正在关闭工作区，本次计算未启动。"
            } else {
                "The workspace is closing. This calculation did not start."
            }
            .into(),
            Reply::Skipped(reason) => match reason {
                radishflow_studio::WorkspaceSolveSkipReason::HoldMode => {
                    if zh {
                        "当前为保持模式，未启动计算。"
                    } else {
                        "Calculation skipped in Hold mode."
                    }
                }
                radishflow_studio::WorkspaceSolveSkipReason::NoPendingRequest => {
                    if zh {
                        "没有待处理的计算请求。"
                    } else {
                        "No calculation request is pending."
                    }
                }
            }
            .into(),
            Reply::Blocked(Rejection::Equipment(error)) => self.equipment_edit_error(error),
            Reply::Blocked(Rejection::Flow(radishflow_studio::WorkspaceRunBlock::Preflight(
                blocked,
            ))) => blocked.message.clone(),
            Reply::Blocked(Rejection::ReadOnly) => if zh {
                "只读工程不能启动计算。"
            } else {
                "Read-only projects cannot start calculations."
            }
            .into(),
            Reply::Blocked(Rejection::FlowContext(_) | Rejection::EquipmentContext(_)) => if zh {
                "输入上下文已变化，请重新发起计算。"
            } else {
                "The input context changed. Submit the calculation again."
            }
            .into(),
            Reply::Blocked(Rejection::Flow(_)) => if zh {
                "当前运行资格不允许启动计算。"
            } else {
                "The current run authorization does not allow this calculation."
            }
            .into(),
            Reply::StartFailed { error, .. } => format!(
                "{} {error}",
                if zh {
                    "计算未能启动："
                } else {
                    "Calculation could not start:"
                }
            ),
            Reply::Completed(completed) => if !completed.applied {
                if zh {
                    "计算已结束，结果未接收。"
                } else {
                    "Calculation finished; the result was not accepted."
                }
            } else if completed.outcome == radishflow_studio::StudioRunOutcome::Succeeded {
                if zh {
                    "计算完成，结果已接收。"
                } else {
                    "Calculation completed and its result was accepted."
                }
            } else {
                if zh {
                    "计算失败，请查看检查与诊断。"
                } else {
                    "Calculation failed. Review the checks and diagnostics."
                }
            }
            .into(),
        };
        self.platform_host.record_activity_line(message);
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
            StudioRunId::Material(_) => {
                if zh {
                    "物料衡算"
                } else {
                    "Material balance"
                }
            }
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
                && let Some(progress) = owner.borrow().flow_progress() {
                match progress.progress {
                    radishflow_studio::WorkspaceSolveProgress::Solving(progress) => {
                        if let Some(unit) = progress.current_unit { ui.label(if zh { format!("当前单元：{unit}") } else { format!("Current unit: {unit}") }); }
                        if let Some(total) = progress.total_units { ui.label(if zh { format!("已完成 {} / {total} 个单元",progress.completed_units) } else { format!("{} / {total} units completed",progress.completed_units) }); }
                    }
                    radishflow_studio::WorkspaceSolveProgress::Partitioned(progress) => {
                        if let Some(block) = progress.block { ui.label(format!("{} {}",if zh {"依赖块"} else {"Block"},block+1)); }
                        if let Some(unit) = progress.unit { ui.label(format!("{}: {unit}",if zh {"当前单元"} else {"Current unit"})); }
                        if let Some(evaluation) = progress.evaluation { ui.label(format!("{} #{evaluation}",match progress.phase {Some(rf_solver::partitioned::EvaluationPhase::Verification)=>if zh {"独立复核"} else {"Independent verification"},_=>if zh {"迭代求值"} else {"Iteration evaluation"}})); }
                        if let Some(sample) = progress.residual { ui.label(format!("{}: {:.3e} mol/s · scaled {:.3e}",if zh {"最近完成残差"} else {"Last completed residual"},sample.max_absolute_mol_s,sample.max_scaled)); }
                    }
                    radishflow_studio::WorkspaceSolveProgress::Simultaneous(progress) => {
                        ui.label(flow_plan::modular_results::progress_text(&progress, zh));
                    }
                    _ => {}
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
        ) || (!self.background_busy() && !self.modular_plan_draft_pending())
    }
    pub(super) fn run_panel_background_enabled(&self, action: rf_ui::RunPanelActionId) -> bool {
        !matches!(
            action,
            rf_ui::RunPanelActionId::RunManual | rf_ui::RunPanelActionId::Resume
        ) || (!self.background_busy() && !self.modular_plan_draft_pending())
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
            if let StudioRunId::Flow(id) = &summary.activity.id
                && let Some(failure) = owner.flow_diagnostic(id)
            {
                ui.push_id(summary.activity.number, |ui| {
                    ui.collapsing(
                        if zh {
                            "本次执行诊断（非当前结果）"
                        } else {
                            "Execution diagnostics (not current results)"
                        },
                        |ui| {
                            for line in radishflow_studio::modular_evidence::failure_lines(
                                failure,
                                self.platform_host
                                    .modeling_inputs()
                                    .workspace
                                    .project_presentation
                                    .display_units(),
                            )
                            .lines
                            {
                                ui.label(line);
                            }
                        },
                    );
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
