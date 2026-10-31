use super::*;
use radishflow_studio::{StudioConfirmedInputAction, StudioInputAction};
use rf_ui::{InputDiscardScope, InputEditCheckpoint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectDepartureCheckpoint {
    inputs: InputEditCheckpoint,
    plan: rf_ui::flow_plan::FlowPlanEditContext,
    plan_draft: rf_ui::flow_plan::FlowPlanGeneration,
    presentation: rf_ui::ProjectPresentationState,
    engineering: rf_ui::engineering::EngineeringCheckpoint,
    equipment_editor: rf_ui::engineering::EngineeringCheckpoint,
    saved_revision: Option<u64>,
    project_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PendingInputAction {
    request: StudioConfirmedInputAction,
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
            plan: rf_ui::flow_plan::FlowPlanEditContext::capture(
                &self.platform_host.modeling_inputs().workspace,
            ),
            plan_draft: self
                .platform_host
                .modeling_inputs()
                .workspace
                .flow_plan
                .draft_generation()
                .clone(),
            inputs: InputEditCheckpoint::capture(
                document.document_id,
                document.revision,
                &document.input_edits,
            ),
            presentation: document.presentation,
            engineering: self.platform_host.engineering_checkpoint(),
            equipment_editor: self
                .platform_host
                .modeling_inputs()
                .workspace
                .equipment_editor
                .checkpoint(),
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
        self.project_open.notice = None;
        self.pending_input_action = Some(PendingInputAction {
            request: StudioConfirmedInputAction {
                checkpoint: self.input_checkpoint(),
                action,
            },
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
        let zh = self.locale == StudioShellLocale::ZhCn;
        let title = match &pending.request.action {
            StudioInputAction::History(radishflow_studio::StudioDocumentHistoryCommand::Undo) => {
                if zh {
                    "撤销工程修改"
                } else {
                    "Undo project change"
                }
            }
            StudioInputAction::History(_) => {
                if zh {
                    "重做工程修改"
                } else {
                    "Redo project change"
                }
            }
            StudioInputAction::Delete(_) => {
                if zh {
                    "删除流股及其连接"
                } else {
                    "Delete stream and connections"
                }
            }
            StudioInputAction::Repair(_) => {
                if zh {
                    "修复模型"
                } else {
                    "Repair model"
                }
            }
        };
        let response =
            egui::Modal::new(egui::Id::new("input-departure-confirmation")).show(ctx, |ui| {
                ui.set_width((ctx.screen_rect().width() - 32.0).clamp(240.0, 520.0));
                ui.heading(title);
                egui::ScrollArea::vertical()
                    .max_height((ctx.screen_rect().height() - 160.0).max(120.0))
                    .show(ui, |ui| {
                        ui.label(if zh {
                            "以下未提交输入将被放弃；此操作不会自动应用输入："
                        } else {
                            "The following unsubmitted inputs will be discarded; this action does not apply them:"
                        });
                        render_affected_inputs(ui, &pending.affected);
                        if let Some(notice) = &self.project_open.notice {
                            ui.label(&notice.detail);
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let cancel = ui.button(if zh { "返回编辑" } else { "Return to editing" });
                    let confirm = ui.button(if zh {
                        "放弃受影响编辑后继续"
                    } else {
                        "Discard affected edits and continue"
                    });
                    if !cancel.has_focus() && !confirm.has_focus() && !confirm.lost_focus() {
                        // Let egui finish Tab wrapping before assigning the initial focus.
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        self.pending_input_action = None;
                    }
                    if confirm.clicked() {
                        self.confirm_input_action();
                    }
                });
            });
        if response.should_close() {
            self.pending_input_action = None;
        }
    }

    pub(super) fn render_project_departure_inputs(&self, ui: &mut egui::Ui) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        for target in workspace.equipment_editor.pending_targets() {
            ui.label(format!(
                "{}：{}",
                if self.locale == StudioShellLocale::ZhCn {
                    "将放弃未提交设备输入（保存不会应用）"
                } else {
                    "Unsubmitted equipment input will be discarded (saving does not apply it)"
                },
                equipment::target_label(self.locale, &target, workspace.engineering.inputs())
            ));
        }
        let document = self.platform_host.snapshot().runtime.workspace_document;
        let affected = InputDiscardScope::All.pending_keys(&document.input_edits);
        if !affected.is_empty() {
            ui.label(match self.locale {
                StudioShellLocale::ZhCn => {
                    "继续离开将放弃以下未提交输入；保存不会自动应用这些输入："
                }
                StudioShellLocale::En => {
                    "Continuing will discard these unsubmitted inputs; saving does not apply them:"
                }
            });
            render_affected_inputs(ui, &self.describe_affected_inputs(&affected));
        }
    }

    pub(super) fn describe_affected_inputs(&self, keys: &[String]) -> Vec<String> {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let snapshot = self.platform_host.snapshot();
        let flowsheet = &self.platform_host.document().flowsheet;
        keys.iter()
            .filter_map(|key| {
                let value = snapshot.runtime.workspace_document.input_edits.get(key)?;
                let (object, field) =
                    if let Some((id, field)) = rf_ui::stream_inspector_draft_key_parts(key) {
                        let object = flowsheet
                            .streams
                            .get(&id)
                            .map_or_else(|| id.as_str().to_string(), |stream| stream.name.clone());
                        let field = match field {
                            rf_ui::StreamInspectorDraftField::Name => {
                                if zh { "名称" } else { "Name" }.into()
                            }
                            rf_ui::StreamInspectorDraftField::TemperatureK => {
                                if zh { "温度" } else { "Temperature" }.into()
                            }
                            rf_ui::StreamInspectorDraftField::PressurePa => {
                                if zh { "压力" } else { "Pressure" }.into()
                            }
                            rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS => {
                                if zh { "摩尔流量" } else { "Molar flow" }.into()
                            }
                            rf_ui::StreamInspectorDraftField::OverallMoleFraction(id) => {
                                if zh {
                                    format!("组分 {} 摩尔分数", id.as_str())
                                } else {
                                    format!("Component {} mole fraction", id.as_str())
                                }
                            }
                        };
                        (object, field)
                    } else if let Some((id, field)) = rf_ui::unit_inspector_draft_key_parts(key) {
                        let object = flowsheet
                            .units
                            .get(&id)
                            .map_or_else(|| id.as_str().to_string(), |unit| unit.name.clone());
                        let field = match field {
                            rf_ui::UnitInspectorDraftField::OutletASplitFraction => {
                                if zh {
                                    "出口 A 分流比"
                                } else {
                                    "Outlet A split fraction"
                                }
                            }
                            rf_ui::UnitInspectorDraftField::Name => {
                                if zh {
                                    "名称"
                                } else {
                                    "Name"
                                }
                            }
                            rf_ui::UnitInspectorDraftField::OutletTemperatureK => {
                                if zh {
                                    "出口温度"
                                } else {
                                    "Outlet temperature"
                                }
                            }
                            rf_ui::UnitInspectorDraftField::OutletPressurePa => {
                                if zh {
                                    "出口压力"
                                } else {
                                    "Outlet pressure"
                                }
                            }
                        };
                        (object, field.to_string())
                    } else {
                        return Some(if zh {
                            format!("未识别的待提交输入：{key}")
                        } else {
                            format!("Unrecognized unsubmitted input: {key}")
                        });
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
