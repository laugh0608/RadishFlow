use super::*;
use rf_ui::variable_browser::{
    ActionDescriptor, ObjectId, ValueSource, ValueState, VariableBrowser, VariableDescriptor,
    VariableSection, VariableValue,
};

#[derive(Default)]
pub(super) struct VariableBrowserState {
    pub open: bool,
    query: String,
    selected: Option<(rf_ui::DocumentId, ObjectId)>,
}

impl ReadyAppState {
    pub(super) fn render_variable_browser(&mut self, ctx: &egui::Context) {
        if !self.variable_browser.open {
            return;
        }
        // Query after frame commands: do not combine an old result projection with new inputs.
        let snapshot = self.platform_host.snapshot();
        let browser = VariableBrowser::new(
            self.platform_host.document(),
            snapshot.runtime.latest_solve_snapshot.as_ref(),
            snapshot.runtime.stale_solve_snapshot.as_ref(),
        );
        let mut open = true;
        let mut focus = None;
        let response =
            egui::Modal::new(egui::Id::new("engineering-variable-browser")).show(ctx, |ui| {
                ui.set_width((ctx.screen_rect().width() - 80.).clamp(320., 880.));
                ui.set_min_height((ctx.screen_rect().height() - 180.).clamp(300., 600.));
                ui.horizontal(|ui| {
                    ui.heading("变量浏览器 / Variable Browser");
                    if ui.button("关闭 / Close").clicked() {
                        open = false;
                    }
                });
                ui.label(format!(
                    "{} · 修订 {}",
                    browser.document.metadata.title, browser.document.revision
                ));
                ui.small("只读浏览已提交输入与求解结果；编辑请定位到检查器。");
                ui.add(
                    egui::TextEdit::singleline(&mut self.variable_browser.query)
                        .hint_text("搜索对象、变量或 SI 单位"),
                );
                ui.separator();
                ui.columns(2, |columns| {
                    egui::ScrollArea::vertical()
                        .id_salt("variable-tree")
                        .max_height((ctx.screen_rect().height() - 200.).max(150.))
                        .show(&mut columns[0], |ui| {
                            for (title, units) in
                                [("单元 / Units", true), ("流股 / Streams", false)]
                            {
                                egui::CollapsingHeader::new(title).default_open(true).show(
                                    ui,
                                    |ui| {
                                        for object in browser
                                            .objects()
                                            .into_iter()
                                            .filter(|o| matches!(o.id, ObjectId::Unit(_)) == units)
                                        {
                                            let selected = self
                                                .variable_browser
                                                .selected
                                                .as_ref()
                                                .is_some_and(|(doc, id)| {
                                                    doc == browser.document_id() && id == &object.id
                                                });
                                            if ui
                                                .selectable_label(
                                                    selected,
                                                    format!("{} · {}", object.name, object.kind),
                                                )
                                                .on_hover_text(format!("{:?}", object.id))
                                                .clicked()
                                            {
                                                self.variable_browser.selected = Some((
                                                    browser.document_id().clone(),
                                                    object.id,
                                                ));
                                                self.variable_browser.query.clear();
                                            }
                                        }
                                    },
                                );
                            }
                            egui::CollapsingHeader::new("流程动作 / Flowsheet actions").show(
                                ui,
                                |ui| {
                                    for action in browser
                                        .actions(None)
                                        .expect("flowsheet actions require no object")
                                    {
                                        render_action(ui, &action);
                                    }
                                },
                            );
                        });
                    egui::ScrollArea::vertical()
                        .id_salt("variable-detail")
                        .max_height((ctx.screen_rect().height() - 200.).max(150.))
                        .show(&mut columns[1], |ui| {
                            if !self.variable_browser.query.trim().is_empty() {
                                let rows = browser.search(&self.variable_browser.query);
                                ui.label(format!("{} 项变量", rows.len()));
                                for row in rows {
                                    ui.push_id(format!("{:?}", row.id), |ui| {
                                        if ui
                                            .small_button(format!("查看 {:?}", row.id.object))
                                            .clicked()
                                        {
                                            self.variable_browser.selected = Some((
                                                row.id.document.clone(),
                                                row.id.object.clone(),
                                            ));
                                            self.variable_browser.query.clear();
                                        }
                                        render_variable(ui, &row);
                                    });
                                }
                                return;
                            }
                            let Some((doc, id)) = self.variable_browser.selected.clone() else {
                                ui.label("选择左侧对象，查看输入、结果和动作。");
                                return;
                            };
                            if &doc != browser.document_id() {
                                ui.label("所选对象属于先前项目，请重新选择。");
                                return;
                            }
                            let Ok(object) = browser.object(&id) else {
                                ui.label("所选对象已删除或不存在，请重新选择。");
                                return;
                            };
                            ui.heading(&object.name);
                            if ui.button("定位到检查器").clicked() {
                                focus = Some(id.clone());
                            }
                            let rows = browser.variables(&id).expect("selected object was checked");
                            for (section, title) in [
                                (VariableSection::Inputs, "输入 / Inputs"),
                                (VariableSection::Results, "结果 / Results"),
                            ] {
                                let rows = rows
                                    .iter()
                                    .filter(|r| r.id.section == section)
                                    .collect::<Vec<_>>();
                                if rows.is_empty() {
                                    continue;
                                }
                                egui::CollapsingHeader::new(title)
                                    .id_salt((&doc, format!("{id:?}{section:?}")))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for row in rows {
                                            ui.push_id(format!("{:?}", row.id), |ui| {
                                                render_variable(ui, row)
                                            });
                                        }
                                    });
                            }
                            for (port, stream) in browser
                                .stream_references(&id)
                                .expect("selected object was checked")
                            {
                                let name = browser
                                    .object(&stream)
                                    .map(|o| o.name)
                                    .unwrap_or_else(|_| "流股已不存在".into());
                                if ui
                                    .button(format!("{port} → {name}"))
                                    .on_hover_text(format!("{stream:?}"))
                                    .clicked()
                                {
                                    self.variable_browser.selected = Some((doc.clone(), stream));
                                }
                            }
                            egui::CollapsingHeader::new("动作描述 / Actions").show(ui, |ui| {
                                for action in browser
                                    .actions(Some(&id))
                                    .expect("selected object was checked")
                                {
                                    render_action(ui, &action);
                                }
                            });
                        });
                });
            });
        self.variable_browser.open = open && !response.should_close();
        if let Some(object) = focus {
            self.focus_browser_object(object);
        }
    }

    pub(super) fn focus_browser_object(&mut self, object: ObjectId) {
        let command = match object {
            ObjectId::Unit(id) => format!("inspector.focus_unit:{id}"),
            ObjectId::Stream(id) => format!("inspector.focus_stream:{id}"),
        };
        self.dispatch_ui_command(command);
        self.screen = StudioShellScreen::Workbench;
        self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
        self.variable_browser.open = false;
    }
}

fn render_variable(ui: &mut egui::Ui, row: &VariableDescriptor) {
    let state = match row.state {
        ValueState::Valid => "单值有效",
        ValueState::Invalid => "无效",
        ValueState::Unspecified => "未指定",
        ValueState::Missing => "缺少结果",
        ValueState::Stale => "结果已过期",
    };
    let value = match &row.value {
        Some(VariableValue::Number(v)) => v.to_string(),
        Some(VariableValue::Text(v)) => v.clone(),
        None => "—".into(),
    };
    ui.label(egui::RichText::new(&row.label).strong());
    ui.label(format!("{value} {} · {state}", row.unit_symbol()));
    ui.small(match row.source {
        ValueSource::DocumentInput => "来源：已提交文档输入",
        ValueSource::StreamTemplate => "来源：流股模板（不是求解结果）",
        ValueSource::SolveResult { .. } => "来源：求解快照（只读）",
        ValueSource::NoResult => "来源：尚无结果",
    });
    egui::CollapsingHeader::new("变量信息").show(ui, |ui| {
        ui.label(format!("身份：{:?}", row.id));
        ui.label(format!("类型：{:?} · 标量", row.value_type));
        if let Some(c) = row.constraint {
            ui.label(format!(
                "有限值；{} {}{}",
                if c.minimum_inclusive { "≥" } else { ">" },
                c.minimum,
                c.maximum.map(|m| format!("；≤ {m}")).unwrap_or_default()
            ));
        }
        ui.label(row.note);
        ui.label(
            row.write_via
                .map(|s| format!("编辑入口：检查器 / {s:?}"))
                .unwrap_or_else(|| "只读结果，不能写入".into()),
        );
        if let ValueSource::SolveResult {
            snapshot,
            revision,
            sequence,
        } = &row.source
        {
            ui.label(format!(
                "快照 {snapshot} · 修订 {revision} · 序号 {sequence}"
            ));
        }
    });
    ui.separator();
}

fn render_action(ui: &mut egui::Ui, action: &ActionDescriptor) {
    egui::CollapsingHeader::new(action.label).show(ui, |ui| {
        ui.label(action.preconditions);
        for parameter in &action.parameters {
            ui.label(format!(
                "{}: {:?}{}",
                parameter.name,
                parameter.kind,
                if parameter.optional {
                    "（可选）"
                } else {
                    ""
                }
            ));
        }
        if !action.fields.is_empty() {
            ui.label(format!("字段：{:?}", action.fields));
        }
        ui.label(format!(
            "修改文档：{}；可撤销：{}；同步执行",
            action.modifies_document, action.undoable
        ));
        ui.label(format!("返回：{}", action.returns));
        ui.small("此处仅描述动作；可用性由正式操作入口实时校验。");
    });
}
