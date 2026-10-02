use super::*;
use radishflow_studio::{StudioConfirmedInputAction, StudioInputAction};
use rf_ui::{InputDiscardScope, InputEditCheckpoint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectDepartureCheckpoint {
    inputs: InputEditCheckpoint,
    presentation: rf_ui::ProjectPresentationState,
    saved_revision: Option<u64>,
    project_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PendingInputAction {
    request: StudioConfirmedInputAction,
    title: String,
    affected: Vec<String>,
}

impl ReadyAppState {
    pub(super) fn input_checkpoint(&self) -> InputEditCheckpoint {
        let document = self.platform_host.snapshot().runtime.workspace_document;
        InputEditCheckpoint::capture(
            document.document_id,
            document.revision,
            &document.input_edits,
        )
    }

    pub(super) fn project_departure_checkpoint(&self) -> ProjectDepartureCheckpoint {
        let document = self.platform_host.snapshot().runtime.workspace_document;
        ProjectDepartureCheckpoint {
            inputs: InputEditCheckpoint::capture(
                document.document_id,
                document.revision,
                &document.input_edits,
            ),
            presentation: document.presentation,
            saved_revision: document.last_saved_revision,
            project_path: document.project_path,
        }
    }

    pub(super) fn validate_project_departure(&mut self) -> bool {
        if self.unit_settings.is_open() {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "请先处理单位设置".into(),
                detail: "应用或取消尚未关闭的设置后，再继续工程操作。".into(),
            });
            return false;
        }
        let current = self.project_departure_checkpoint();
        if self.project_open.departure_checkpoint.as_ref() != Some(&current) {
            self.project_open.departure_checkpoint = Some(current);
            self.report_stale_departure();
            return false;
        }
        true
    }

    pub(super) fn report_stale_departure(&mut self) {
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Warning,
            title: "操作尚未执行".into(),
            detail: "工程、显示设置或未提交输入已变化；已保留编辑，请重新检查影响后确认。".into(),
        });
    }

    pub(super) fn intercept_input_action(&mut self, command_id: &str) -> bool {
        let snapshot = self.platform_host.snapshot();
        let action = if let Some(command) =
            radishflow_studio::document_history_command_from_id(command_id)
        {
            StudioInputAction::History(command)
        } else if command_id == "canvas.delete_selected_stream" {
            let Some(target @ rf_ui::InspectorTarget::Stream(_)) =
                snapshot.runtime.active_inspector_target
            else {
                return false;
            };
            StudioInputAction::Delete(target)
        } else if command_id == "run_panel.recover_failure" {
            let Some(action) = snapshot
                .runtime
                .run_panel
                .recovery_action()
                .filter(|action| action.mutation.is_some())
            else {
                return false;
            };
            StudioInputAction::Repair(action.clone())
        } else {
            return false;
        };
        let affected_keys = action
            .discard_scope()
            .pending_keys(&snapshot.runtime.workspace_document.input_edits);
        if affected_keys.is_empty() {
            if matches!(action, StudioInputAction::Delete(_)) {
                self.execute_confirmed_input_action(StudioConfirmedInputAction {
                    checkpoint: self.input_checkpoint(),
                    action,
                });
                return true;
            }
            return false;
        }
        let title = match &action {
            StudioInputAction::History(radishflow_studio::StudioDocumentHistoryCommand::Undo) => {
                "撤销工程修改"
            }
            StudioInputAction::History(_) => "重做工程修改",
            StudioInputAction::Delete(_) => "删除流股及其连接",
            StudioInputAction::Repair(_) => "修复模型",
        };
        self.project_open.notice = None;
        self.pending_input_action = Some(PendingInputAction {
            request: StudioConfirmedInputAction {
                checkpoint: self.input_checkpoint(),
                action,
            },
            title: title.into(),
            affected: self.describe_affected_inputs(&affected_keys),
        });
        true
    }

    pub(super) fn execute_confirmed_input_action(
        &mut self,
        request: StudioConfirmedInputAction,
    ) -> bool {
        let Some(window_id) = self.current_window_id() else {
            return false;
        };
        match self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::ConfirmedInputAction(Box::new(request)),
        }) {
            Ok(_) => {
                self.refresh_active_modeling_readiness_notice();
                true
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "操作未完成，输入已保留".into(),
                    detail: error.message().to_string(),
                });
                false
            }
        }
    }

    pub(super) fn confirm_input_action(&mut self) {
        let Some(mut pending) = self.pending_input_action.take() else {
            return;
        };
        let current = self.input_checkpoint();
        if pending.request.checkpoint != current {
            pending.request.checkpoint = current;
            let keys = pending.request.action.discard_scope().pending_keys(
                &self
                    .platform_host
                    .snapshot()
                    .runtime
                    .workspace_document
                    .input_edits,
            );
            pending.affected = self.describe_affected_inputs(&keys);
            self.pending_input_action = Some(pending);
            self.report_stale_departure();
            return;
        }
        self.execute_confirmed_input_action(pending.request);
    }

    pub(super) fn render_input_action_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_input_action.clone() else {
            return;
        };
        let response =
            egui::Modal::new(egui::Id::new("input-departure-confirmation")).show(ctx, |ui| {
                ui.set_max_width(520.0);
                ui.heading(&pending.title);
                ui.label("以下未提交输入将被放弃；此操作不会自动应用输入：");
                render_affected_inputs(ui, &pending.affected);
                if let Some(notice) = &self.project_open.notice {
                    ui.label(&notice.detail);
                }
                ui.horizontal(|ui| {
                    if ui.button("返回编辑").clicked() {
                        self.pending_input_action = None;
                    }
                    if ui.button("放弃受影响编辑后继续").clicked() {
                        self.confirm_input_action();
                    }
                });
            });
        if response.should_close() {
            self.pending_input_action = None;
        }
    }

    pub(super) fn render_project_departure_inputs(&self, ui: &mut egui::Ui) {
        let document = self.platform_host.snapshot().runtime.workspace_document;
        let affected = InputDiscardScope::All.pending_keys(&document.input_edits);
        if !affected.is_empty() {
            ui.label("继续离开将放弃以下未提交输入；保存不会自动应用这些输入：");
            render_affected_inputs(ui, &self.describe_affected_inputs(&affected));
        }
    }

    pub(super) fn describe_affected_inputs(&self, keys: &[String]) -> Vec<String> {
        let snapshot = self.platform_host.snapshot();
        let flowsheet = &self.platform_host.document().flowsheet;
        keys.iter()
            .filter_map(|key| {
                let value = snapshot.runtime.workspace_document.input_edits.get(key)?;
                let (object, field) = if let Some((id, field)) =
                    rf_ui::stream_inspector_draft_key_parts(key)
                {
                    let object = flowsheet
                        .streams
                        .get(&id)
                        .map_or_else(|| id.as_str().to_string(), |stream| stream.name.clone());
                    let field = match field {
                        rf_ui::StreamInspectorDraftField::Name => "名称".into(),
                        rf_ui::StreamInspectorDraftField::TemperatureK => "温度".into(),
                        rf_ui::StreamInspectorDraftField::PressurePa => "压力".into(),
                        rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS => "摩尔流量".into(),
                        rf_ui::StreamInspectorDraftField::OverallMoleFraction(id) => {
                            format!("组分 {} 摩尔分数", id.as_str())
                        }
                    };
                    (object, field)
                } else if let Some((id, field)) = rf_ui::unit_inspector_draft_key_parts(key) {
                    let object = flowsheet
                        .units
                        .get(&id)
                        .map_or_else(|| id.as_str().to_string(), |unit| unit.name.clone());
                    let field = match field {
                        rf_ui::UnitInspectorDraftField::Name => "名称",
                        rf_ui::UnitInspectorDraftField::OutletTemperatureK => "出口温度",
                        rf_ui::UnitInspectorDraftField::OutletPressurePa => "出口压力",
                    };
                    (object, field.to_string())
                } else {
                    return Some(format!("未识别的待提交输入：{key}"));
                };
                let text = match value {
                    rf_ui::DraftValue::Numeric(session) => format!(
                        "{} {}",
                        session.raw_text(),
                        session.input_unit().definition().symbol
                    ),
                    rf_ui::DraftValue::Text(draft)
                    | rf_ui::DraftValue::Number(draft)
                    | rf_ui::DraftValue::Choice(draft) => draft.current.clone(),
                };
                Some(format!("{object} · {field}：{text}"))
            })
            .collect()
    }
}

pub(super) fn render_affected_inputs(ui: &mut egui::Ui, keys: &[String]) {
    egui::ScrollArea::vertical()
        .max_height(180.0)
        .show(ui, |ui| {
            for key in keys {
                ui.label(key);
            }
        });
}
