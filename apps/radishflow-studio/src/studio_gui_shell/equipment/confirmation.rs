use super::*;

impl ReadyAppState {
    pub(in super::super) fn render_equipment_confirmation(&mut self, ctx: &egui::Context) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let Some(pending) = workspace.equipment_editor.confirmation().cloned() else {
            return;
        };
        let inputs = workspace.engineering.inputs().clone();
        let error = workspace.equipment_editor.error().cloned();
        let locale = self.locale;
        let mut command = None;
        let response = egui::Modal::new(egui::Id::new("equipment-confirmation")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
            ui.heading(if pending.upgrades_rating_method() {
                tr(locale, "升级校核方法并新增工况？", "Upgrade rating method and add case?")
            } else if pending.deleted_rating_case().is_some() {
                tr(locale, "删除工况并缩小覆盖范围？", "Delete case and reduce coverage?")
            } else if pending.is_history() {
                tr(
                    locale,
                    "确认设备历史操作",
                    "Confirm equipment history change",
                )
            } else if pending.is_delete() {
                tr(locale, "确认删除记录", "Confirm record deletion")
            } else {
                tr(
                    locale,
                    "确认几何或共享记录变更",
                    "Confirm geometry or shared record change",
                )
            });
            egui::ScrollArea::vertical()
                .max_height((ctx.screen_rect().height() - 180.0).max(100.0))
                .show(ui, |ui| {
                    if pending.upgrades_rating_method() {
                        ui.label(tr(locale, "此任务将从校核 v1 升级为 v2，保留原输入并使原结果过期。仅支持 v1 的旧程序将整份工程只读打开；保存时可另存为保留原件。", "This task upgrades from rating v1 to v2, preserving its inputs and making previous results stale. Programs supporting only v1 open the entire project read-only; Save As can preserve the original."));
                    }
                    if let Some(member) = pending.deleted_rating_case() {
                        ui.label(format!("{}: {}", tr(locale, "删除工况", "Delete case"), member.case_id));
                        ui.label(tr(locale, "该行将移出必要工况集合；未改行的结果保留。撤销可恢复输入，但不会复活旧计算或待输出资格。", "This row leaves the required case set; unchanged rows retain their results. Undo restores inputs without reviving old executions or pending outputs."));
                    }
                    if let Some(error) = &error {
                        ui.label(edit_error(locale, error));
                    }
                    for impact in pending.impacts() {
                        let case_name = inputs
                            .equipment_task(&impact.task_id)
                            .and_then(|task| match task {
                                rf_model::equipment::EquipmentTask::SeparatorSizing(task) => {
                                    Some(task.case.name.as_str())
                                }
                                rf_model::equipment::EquipmentTask::SeparatorRating(task) => task
                                    .cases
                                    .iter()
                                    .find(|case| case.id == impact.case_id)
                                    .map(|case| case.name.as_str()),
                            })
                            .filter(|name| !name.trim().is_empty())
                            .unwrap_or(impact.case_id.as_str());
                        let change = match (impact.input_changed, impact.evidence_changed) {
                            (true, true) => {
                                tr(locale, "输入与证据变化", "Inputs and evidence change")
                            }
                            (true, false) => tr(locale, "输入变化", "Inputs change"),
                            (false, true) => tr(locale, "证据变化", "Evidence changes"),
                            (false, false) => tr(locale, "记录变化", "Record changes"),
                        };
                        ui.label(format!(
                            "{} / {} · {change} · {}",
                            target_label(
                                locale,
                                &EquipmentEditorTarget::Task(impact.task_id.clone()),
                                &inputs
                            ),
                            case_name,
                            tr(locale, "已有结果将过期", "Existing results become stale")
                        ));
                    }
                    for target in pending.discarded_drafts() {
                        ui.label(format!(
                            "{}：{}",
                            tr(
                                locale,
                                "将放弃未提交输入",
                                "Unsubmitted input will be discarded"
                            ),
                            target_label(locale, target, &inputs)
                        ));
                    }
                    ui.label(tr(
                        locale,
                        "确认不会自动重新计算。",
                        "Confirmation does not recalculate automatically.",
                    ));
                });
            ui.horizontal_wrapped(|ui| {
                let cancel = ui.button(tr(locale, "返回编辑", "Return to editing"));
                let confirm = ui.button(tr(locale, "确认继续", "Confirm and continue"));
                if !cancel.has_focus() && !confirm.has_focus() && !confirm.lost_focus() {
                    cancel.request_focus();
                }
                if cancel.clicked() {
                    command = Some(EquipmentEditorCommand::CancelConfirmation);
                }
                if confirm.clicked() {
                    command = Some(EquipmentEditorCommand::Confirm(pending.key()));
                }
            });
        });
        if response.should_close() {
            command = Some(EquipmentEditorCommand::CancelConfirmation);
        }
        if let Some(command) = command {
            self.equipment_command(command);
        }
    }
}
