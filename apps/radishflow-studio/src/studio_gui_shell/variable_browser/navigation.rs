use super::*;
use rf_ui::engineering::editor::{EquipmentEditorCommand, EquipmentEditorTarget};
use rf_ui::project_query::{ProjectObjectId, ProjectQuery, ProjectRecord};
use rf_ui::variable_browser::VariableSection;

impl ReadyAppState {
    pub(super) fn request_browser_navigation(&mut self, target: BrowserNavigation) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self.variable_browser.belongs_to(workspace) || self.read_only_project.is_some() {
            self.variable_browser.notice = Some(
                tr(
                    self.locale,
                    "工程会话已变化，请重新选择。",
                    "Project session changed; select again.",
                )
                .into(),
            );
            return;
        }
        // Selection itself preserves drafts. Make that transition explicit when an
        // editor has pending input; never turn navigation into commit or discard.
        if !rf_ui::InputDiscardScope::All
            .pending_keys(&workspace.drafts.fields)
            .is_empty()
            || !workspace.equipment_editor.pending_targets().is_empty()
        {
            self.variable_browser.pending_navigation = Some(target);
        } else {
            self.navigate_browser(target);
        }
    }

    pub(in crate::studio_gui_shell) fn browser_navigation_pending(&self) -> bool {
        self.variable_browser.pending_navigation.is_some()
    }

    pub(in crate::studio_gui_shell) fn render_browser_navigation_dialog(
        &mut self,
        ctx: &egui::Context,
    ) {
        let Some(target) = self.variable_browser.pending_navigation.clone() else {
            return;
        };
        let mut proceed = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("browser-navigation-drafts")).show(ctx, |ui| {
            ui.set_max_width((ctx.available_rect().width() - 32.0).clamp(240.0, 480.0));
            ui.heading(tr(self.locale,"保留未提交输入","Keep pending inputs"));
            ui.add(egui::Label::new(tr(self.locale,"定位将切换编辑位置，所有草稿仍保留在原编辑器；不会自动应用或丢弃。","Navigation changes the editor location. All drafts remain in their editors; none are applied or discarded.")).wrap());
            ui.horizontal_wrapped(|ui| {
                let back = ui.button(tr(self.locale,"取消导航","Cancel navigation"));
                cancel = back.clicked();
                proceed = ui.button(tr(self.locale,"保留草稿并定位","Keep drafts and locate")).clicked();
            });
        });
        if proceed {
            self.variable_browser.pending_navigation = None;
            self.navigate_browser(target);
        } else if cancel || response.should_close() {
            self.variable_browser.pending_navigation = None;
        }
    }

    pub(super) fn navigate_browser(&mut self, target: BrowserNavigation) -> bool {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self.variable_browser.belongs_to(workspace) || self.read_only_project.is_some() {
            return false;
        }
        let target = match target {
            BrowserNavigation::StableReference(reference) => {
                let variable = match reference.resolve(workspace) {
                    Ok(variable) => variable,
                    Err(error) => {
                        self.variable_browser.notice =
                            Some(references::error_label(error, self.locale).into());
                        return false;
                    }
                };
                let section = variable.section;
                let selection = references::variable_selection(variable);
                self.variable_browser.selected = Some(selection.clone());
                self.variable_browser.query.clear();
                self.variable_browser.scope = None;
                self.variable_browser.section = SectionFilter::All;
                self.variable_browser.detail_page = true;
                self.variable_browser.notice = None;
                self.variable_browser.reference_input = None;
                if section == VariableSection::Results {
                    if self.variable_browser.directory == DirectoryView::WatchList {
                        self.variable_browser.directory = DirectoryView::Project;
                    }
                    self.open_variable_browser();
                    return true;
                }
                BrowserNavigation::Selection(selection)
            }
            target => target,
        };
        let reference = match &target {
            BrowserNavigation::Selection(s) => Some(&s.object),
            BrowserNavigation::EquipmentResult(r) | BrowserNavigation::FlowResults(r) => Some(r),
            BrowserNavigation::Property => None,
            BrowserNavigation::StableReference(_) => unreachable!("resolved above"),
        };
        if let Some(reference) = reference {
            let query = ProjectQuery::new(workspace);
            let check = match &target {
                BrowserNavigation::Selection(BrowserSelection {
                    field: Some(field), ..
                }) => query.read_field(reference, field).map(|_| ()),
                _ => query.record(reference).map(|_| ()),
            };
            if let Err(error) = check {
                self.variable_browser.notice = Some(query_error(&error, self.locale));
                return false;
            }
        }
        match target {
            BrowserNavigation::StableReference(_) => unreachable!("resolved above"),
            BrowserNavigation::Property => self.screen = StudioShellScreen::Property,
            BrowserNavigation::FlowResults(reference) => self.open_browser_flow_result(&reference),
            BrowserNavigation::EquipmentResult(reference) => {
                if !self.open_browser_equipment(&reference, true) {
                    return false;
                }
            }
            BrowserNavigation::Selection(selection) => match &selection.object.object {
                ProjectObjectId::Flow(object) => {
                    if let Some(ProjectField::Flow {
                        section: VariableSection::Results,
                        ..
                    }) = &selection.field
                    {
                        self.open_browser_flow_result(&selection.object);
                    } else {
                        self.focus_browser_object(object.clone());
                        if let Some(ProjectField::Flow { field, section }) = selection.field {
                            self.right_sidebar_tab = if matches!(object, ObjectId::Unit(_)) {
                                StudioShellRightSidebarTab::ModuleSettings
                            } else {
                                StudioShellRightSidebarTab::Inspector
                            };
                            self.variable_browser.pending_focus = Some(VariableId {
                                document: selection.object.document,
                                object: object.clone(),
                                section,
                                field,
                            });
                        }
                    }
                }
                _ => {
                    if !self.open_browser_equipment(&selection.object, false) {
                        return false;
                    }
                }
            },
        }
        self.variable_browser.open = false;
        self.variable_browser.return_available = true;
        true
    }

    pub(in crate::studio_gui_shell) fn focus_browser_object(&mut self, object: ObjectId) {
        let target = match object {
            ObjectId::Unit(id) => rf_ui::InspectorTarget::Unit(id),
            ObjectId::Stream(id) => rf_ui::InspectorTarget::Stream(id),
        };
        self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&target));
        self.screen = StudioShellScreen::Workbench;
        self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
        self.variable_browser.open = false;
    }

    fn open_browser_flow_result(&mut self, reference: &ProjectObjectRef) {
        let ProjectObjectId::Flow(ObjectId::Stream(id)) = &reference.object else {
            return;
        };
        let snapshot = match self.platform_host.modeling_inputs().workspace.flow_result() {
            rf_ui::result_query::FlowResultView::Current { snapshot, .. } => {
                Some(snapshot.id.to_string())
            }
            _ => None,
        };
        self.focus_browser_object(ObjectId::Stream(id.clone()));
        if let Some(snapshot) = snapshot {
            self.focus_result_table_stream(&snapshot, id.as_str());
        }
        self.screen = StudioShellScreen::Results;
    }

    fn open_browser_equipment(&mut self, reference: &ProjectObjectRef, results: bool) -> bool {
        let query = ProjectQuery::new(&self.platform_host.modeling_inputs().workspace);
        let target = match (&reference.object, query.record(reference)) {
            (ProjectObjectId::Equipment(id), Ok(ProjectRecord::Equipment(task))) => {
                if task.as_sizing().is_some() {
                    EquipmentEditorTarget::Task(id.clone())
                } else {
                    EquipmentEditorTarget::RatingTask(id.clone())
                }
            }
            (ProjectObjectId::Case(key), Ok(ProjectRecord::SizingCase(_))) => {
                EquipmentEditorTarget::Task(key.task_id.clone())
            }
            (ProjectObjectId::Case(key), Ok(ProjectRecord::RatingCase(_))) => {
                EquipmentEditorTarget::RatingCase(key.clone())
            }
            (ProjectObjectId::Geometry(id), Ok(_)) => {
                EquipmentEditorTarget::RatingGeometry(id.clone())
            }
            (ProjectObjectId::Basis(id), Ok(_)) => EquipmentEditorTarget::Basis(id.clone()),
            (ProjectObjectId::Source(id), Ok(_)) => EquipmentEditorTarget::Source(id.clone()),
            _ => return false,
        };
        self.equipment_command(EquipmentEditorCommand::Select(target.clone()));
        if self
            .platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active()
            != Some(&target)
        {
            return false;
        }
        self.screen = StudioShellScreen::Equipment;
        self.equipment.section = if results {
            equipment::EquipmentSection::Results
        } else {
            equipment::EquipmentSection::Inputs
        };
        true
    }

    pub(in crate::studio_gui_shell) fn apply_browser_field_focus(
        &mut self,
        response: &egui::Response,
        variable: &VariableId,
    ) -> bool {
        if !self
            .variable_browser
            .belongs_to(&self.platform_host.modeling_inputs().workspace)
        {
            self.variable_browser.pending_focus = None;
            return false;
        }
        if self.variable_browser.pending_focus.as_ref() != Some(variable) {
            return false;
        }
        response.scroll_to_me(Some(egui::Align::Center));
        response.request_focus();
        self.variable_browser.pending_focus = None;
        true
    }

    pub(in crate::studio_gui_shell) fn apply_browser_key_focus(
        &mut self,
        response: &egui::Response,
        key: &str,
    ) {
        if let Some(variable) = self.browser_variable_for_key(key) {
            self.apply_browser_field_focus(response, &variable);
        }
    }

    fn browser_variable_for_key(&self, key: &str) -> Option<VariableId> {
        super::super::modeling_problems::variable_for_draft_key(
            key,
            &self.platform_host.document().metadata.document_id,
        )
    }

    pub(in crate::studio_gui_shell) fn render_browser_field_link(
        &mut self,
        ui: &mut egui::Ui,
        field: &radishflow_studio::StudioGuiWindowInspectorTargetFieldModel,
    ) {
        let Some(variable) = self.browser_variable_for_key(&field.key) else {
            return;
        };
        if ui
            .small_button(tr(
                self.locale,
                "在变量浏览器中查看",
                "View in variable browser",
            ))
            .clicked()
        {
            self.select_browser_variable(variable);
        }
    }

    pub(in crate::studio_gui_shell) fn select_browser_variable(&mut self, variable: VariableId) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        self.variable_browser.synchronize(workspace, self.locale);
        let object = ProjectObjectRef {
            document: variable.document,
            object: ProjectObjectId::Flow(variable.object),
        };
        self.variable_browser.selected = Some(BrowserSelection {
            object: object.clone(),
            field: Some(ProjectField::Flow {
                section: variable.section,
                field: variable.field,
            }),
        });
        self.variable_browser.scope = Some(object);
        self.variable_browser.detail_page = true;
        self.open_variable_browser();
    }

    pub(in crate::studio_gui_shell) fn append_browser_palette_item(
        &self,
        items: &mut Vec<radishflow_studio::StudioGuiWindowCommandPaletteItemModel>,
        query: &str,
    ) {
        let label = tr(self.locale, "变量浏览器", "Variable browser");
        let searchable = "view.variable_browser 变量浏览器 只读 variable browser read only";
        if !query
            .to_lowercase()
            .split_whitespace()
            .all(|q| searchable.contains(q))
        {
            return;
        }
        let detail = tr(
            self.locale,
            "浏览已提交字段、单位、结果来源并定位。",
            "Browse committed fields, units and result origins, then locate their editor.",
        );
        items.push(radishflow_studio::StudioGuiWindowCommandPaletteItemModel {
            command_id: "view.variable_browser".into(),
            enabled: self.read_only_project.is_none(),
            label: label.into(),
            detail: detail.into(),
            menu_path_text: tr(self.locale, "工具 / 变量浏览器", "Tools / Variable browser").into(),
            hover_text: detail.into(),
        });
    }
}
