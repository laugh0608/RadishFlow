use super::*;

#[derive(Debug, Clone)]
pub(super) struct PendingUnitDeletion {
    document_id: String,
    revision: u64,
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
            document_id: document.document_id.clone(),
            revision: document.revision,
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
        let document = &snapshot.runtime.workspace_document;
        if document.document_id != pending.document_id
            || document.revision != pending.revision
            || snapshot.runtime.active_inspector_target
                != Some(rf_ui::InspectorTarget::Unit(pending.unit_id))
        {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "删除未执行".to_string(),
                detail: "文档或选择已变化，请重新选择单元并检查删除影响。".to_string(),
            });
            return;
        }
        self.dispatch_confirmed_ui_command("canvas.delete_selected_unit".to_string());
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
        let response =
            egui::Modal::new(egui::Id::new("delete-unit-confirmation")).show(ctx, |ui| {
                ui.set_max_width(480.0);
                ui.heading("删除单元");
                ui.label(format!("{} ({})", pending.name, pending.unit_id.as_str()));
                ui.label(
                    "删除该单元及其端口绑定；保留关联流股的参数和其他设备的连接。可通过撤销恢复。",
                );
                if pending.bindings.is_empty() {
                    ui.label("该单元没有关联流股。");
                } else {
                    ui.label("以下流股将失去该单元的端口连接：");
                    for (port, stream) in &pending.bindings {
                        ui.label(format!("{port} → {stream}"));
                    }
                    ui.label("缺少源端或变为孤立对象的流股将需要修复或显式删除。");
                }
                ui.horizontal(|ui| {
                    if ui.button("取消删除").clicked() {
                        self.pending_unit_deletion = None;
                    }
                    if ui.button("确认删除单元").clicked() {
                        self.confirm_unit_deletion();
                    }
                });
            });
        if response.should_close() {
            self.pending_unit_deletion = None;
        }
    }
}
