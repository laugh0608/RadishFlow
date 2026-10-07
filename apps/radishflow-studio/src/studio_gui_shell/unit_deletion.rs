use super::*;

#[derive(Debug, Clone)]
pub(super) struct PendingUnitDeletion {
    checkpoint: rf_ui::InputEditCheckpoint,
    revision: u64,
    affected: Vec<String>,
    unit_id: rf_types::UnitId,
    name: String,
    bindings: Vec<(String, String)>,
}

impl ReadyAppState {
    pub(super) fn request_unit_deletion(&mut self) {
        let snapshot = self.platform_host.snapshot();
        let Some(detail) = snapshot.runtime.active_inspector_detail.as_ref() else {
            return;
        };
        let rf_ui::InspectorTarget::Unit(unit_id) = &detail.target else {
            return;
        };
        let document = &snapshot.runtime.workspace_document;
        self.pending_unit_deletion = Some(PendingUnitDeletion {
            checkpoint: self.input_checkpoint(),
            revision: document.revision,
            affected: self.describe_affected_inputs(
                &rf_ui::InputDiscardScope::Object(rf_ui::InspectorTarget::Unit(unit_id.clone()))
                    .pending_keys(&document.input_edits),
            ),
            unit_id: unit_id.clone(),
            name: detail.title.clone(),
            bindings: detail
                .unit_ports
                .iter()
                .filter_map(|port| {
                    port.stream_id
                        .as_ref()
                        .map(|stream_id| (port.name.clone(), stream_id.clone()))
                })
                .collect(),
        });
    }

    pub(super) fn confirm_unit_deletion(&mut self) {
        let Some(pending) = self.pending_unit_deletion.take() else {
            return;
        };
        let snapshot = self.platform_host.snapshot();
        if self.input_checkpoint() != pending.checkpoint
            || snapshot.runtime.active_inspector_target
                != Some(rf_ui::InspectorTarget::Unit(pending.unit_id.clone()))
        {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "删除未执行".to_string(),
                detail: "文档、选择或未提交输入已变化，请重新检查删除影响。".to_string(),
            });
            return;
        }
        self.execute_confirmed_input_action(radishflow_studio::StudioConfirmedInputAction {
            checkpoint: pending.checkpoint,
            action: radishflow_studio::StudioInputAction::Delete(rf_ui::InspectorTarget::Unit(
                pending.unit_id,
            )),
        });
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .revision
            > pending.revision
        {
            self.canvas_command_result = None;
            self.canvas_viewport_navigation.active_anchor = None;
            self.canvas_unit_drag = None;
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Info,
                title: "单元已删除".to_string(),
                detail: format!(
                    "已删除 {}；关联流股及其他设备的连接已保留，可撤销恢复。",
                    pending.name
                ),
            });
        }
    }

    pub(super) fn render_unit_deletion_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_unit_deletion.clone() else {
            return;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let response =
            egui::Modal::new(egui::Id::new("delete-unit-confirmation")).show(ctx, |ui| {
                ui.set_width((ctx.screen_rect().width() - 32.0).clamp(240.0, 480.0));
                ui.heading(if zh { "删除单元" } else { "Delete unit" });
                egui::ScrollArea::vertical()
                    .max_height((ctx.screen_rect().height() - 160.0).max(120.0))
                    .show(ui, |ui| {
                        ui.label(format!("{} ({})", pending.name, pending.unit_id.as_str()));
                        ui.label(if zh {
                            "删除该单元及其端口绑定；保留关联流股的参数和其他设备的连接。可通过撤销恢复。"
                        } else {
                            "Delete this unit and its port bindings. Connected stream parameters and other units' connections are preserved. Undo can restore the unit."
                        });
                        if !pending.affected.is_empty() {
                            ui.label(if zh {
                                "确认后放弃以下未提交输入；其他对象的编辑保留，数值会话重新校验。"
                            } else {
                                "Confirming discards the following unsubmitted inputs. Other objects' edits are preserved and numeric sessions are revalidated."
                            });
                            super::input_departure::render_affected_inputs(ui, &pending.affected);
                        }
                        if pending.bindings.is_empty() {
                            ui.label(if zh {
                                "该单元没有关联流股。"
                            } else {
                                "This unit has no connected streams."
                            });
                        } else {
                            ui.label(if zh {
                                "以下流股将失去该单元的端口连接："
                            } else {
                                "These streams will lose their port connections to this unit:"
                            });
                            for (port, stream) in &pending.bindings {
                                ui.label(format!("{port} → {stream}"));
                            }
                            ui.label(if zh {
                                "缺少源端或变为孤立对象的流股将需要修复或显式删除。"
                            } else {
                                "Streams without a source or any remaining connections will need repair or explicit deletion."
                            });
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let cancel = ui.button(if zh { "取消删除" } else { "Cancel deletion" });
                    let confirm = ui.button(if zh { "确认删除单元" } else { "Delete unit" });
                    if !cancel.has_focus() && !confirm.has_focus() && !confirm.lost_focus() {
                        // Let egui finish Tab wrapping before assigning the initial focus.
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        self.pending_unit_deletion = None;
                    }
                    if confirm.clicked() {
                        self.confirm_unit_deletion();
                    }
                });
            });
        if response.should_close() {
            self.pending_unit_deletion = None;
        }
    }
}
