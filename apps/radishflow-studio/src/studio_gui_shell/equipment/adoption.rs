//! Local form state; preparation and publication always use application commands.
use super::*;
use rf_types::EquipmentTaskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdoptionStage {
    Target,
    Geometry,
}

pub(in crate::studio_gui_shell) struct AdoptionForm {
    pub(super) source: EquipmentTaskId,
    pub(in crate::studio_gui_shell) new_target: bool,
    pub(in crate::studio_gui_shell) existing: Option<EquipmentTaskId>,
    pub(in crate::studio_gui_shell) tag: String,
    pub(in crate::studio_gui_shell) name: String,
    pub(in crate::studio_gui_shell) copy_case: bool,
    pub(in crate::studio_gui_shell) diameter: Option<DimensionAdjustmentDraft>,
    pub(in crate::studio_gui_shell) height: Option<DimensionAdjustmentDraft>,
    pub(super) original_diameter: f64,
    pub(super) original_height: f64,
    pub(super) stage: AdoptionStage,
    pub(super) prepared_form: Option<GeometryAdoptionDraft>,
    pub(in crate::studio_gui_shell) error: Option<EquipmentEditError>,
    pub(super) focus_first: bool,
    pub(super) return_focus: Option<egui::Id>,
}
impl AdoptionForm {
    pub(super) fn request(&self) -> Option<GeometryAdoptionDraft> {
        Some(GeometryAdoptionDraft {
            source_task: self.source.clone(),
            target: if self.new_target {
                GeometryAdoptionDestination::New {
                    tag: self.tag.clone(),
                    name: self.name.clone(),
                    copy_case: self.copy_case,
                }
            } else {
                GeometryAdoptionDestination::Existing(self.existing.clone()?)
            },
            diameter: self.diameter.clone(),
            working_liquid_height: self.height.clone(),
        })
    }
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn begin_geometry_adoption(
        &mut self,
        source: &EquipmentTaskId,
    ) {
        if self.equipment_target_busy(source, None) {
            return;
        }
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let session = &workspace.engineering;
        let Some(candidate) = session
            .current_candidate(source)
            .filter(|_| !workspace.equipment_editor.pending_for_task(source, session))
        else {
            return;
        };
        self.equipment.adoption = Some(AdoptionForm {
            source: source.clone(),
            new_target: true,
            existing: None,
            tag: String::new(),
            name: String::new(),
            copy_case: false,
            diameter: None,
            height: None,
            original_diameter: candidate.dimensions().minimum_diameter_m,
            original_height: candidate.dimensions().working_liquid_height_m,
            stage: AdoptionStage::Target,
            prepared_form: None,
            error: None,
            focus_first: true,
            return_focus: None,
        });
        self.equipment_command(EquipmentEditorCommand::CancelGeometryAdoption);
    }

    pub(in crate::studio_gui_shell) fn preview_geometry_adoption_form(&mut self) {
        let Some(request) = self
            .equipment
            .adoption
            .as_ref()
            .and_then(AdoptionForm::request)
        else {
            return;
        };
        self.equipment_command(EquipmentEditorCommand::CancelGeometryAdoption);
        self.equipment_command(EquipmentEditorCommand::PreviewGeometryAdoption(Box::new(
            request.clone(),
        )));
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let form = self.equipment.adoption.as_mut().expect("open form");
        form.error = workspace.equipment_editor.error().cloned();
        if let Some(preview) = workspace.equipment_editor.geometry_adoption_preview() {
            let target = preview.proposal().target().task_id();
            let a = preview
                .proposal()
                .inputs()
                .rating_task(target)
                .expect("prepared rating")
                .geometry
                .adoption
                .as_ref()
                .expect("adoption evidence");
            form.original_diameter = a.diameter.original_m;
            form.original_height = a.working_liquid_height.original_m;
            form.stage = AdoptionStage::Geometry;
            form.prepared_form = Some(request);
            form.focus_first = true;
        }
    }

    pub(in crate::studio_gui_shell) fn confirm_geometry_adoption_form(&mut self) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let Some(form) = self.equipment.adoption.as_ref() else {
            return;
        };
        if self.equipment_target_busy(&form.source, None) {
            return;
        }
        // An old preview never approves subsequently edited local fields.
        if form.prepared_form.is_none() || form.prepared_form != form.request() {
            return;
        }
        let Some(preview) = workspace.equipment_editor.geometry_adoption_preview() else {
            return;
        };
        let target = preview.proposal().target().task_id().clone();
        let key = preview.key();
        self.equipment_command(EquipmentEditorCommand::ConfirmGeometryAdoption(key));
        let editor = &self
            .platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor;
        if let Some(error) = editor.error() {
            self.equipment.adoption.as_mut().expect("open form").error = Some(error.clone());
        } else {
            self.equipment.adoption = None;
            self.equipment.section = EquipmentSection::Inputs;
            self.equipment.rating_overview.insert(target, false);
        }
    }

    pub(in crate::studio_gui_shell) fn cancel_geometry_adoption_form(&mut self) {
        self.equipment.adoption = None;
        self.equipment_command(EquipmentEditorCommand::CancelGeometryAdoption);
    }

    pub(super) fn render_geometry_adoption(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.equipment.adoption.take() else {
            return;
        };
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let session = &workspace.engineering;
        let editor = &workspace.equipment_editor;
        let units = workspace.project_presentation.display_units();
        let l = self.locale;
        let preview = editor.geometry_adoption_preview();
        let valid = !self.equipment_target_busy(&form.source, None)
            && preview.is_some()
            && editor.validate_geometry_adoption_preview(session).is_ok();
        let mut cancel = false;
        let mut prepare = false;
        let mut confirm = false;
        let mut back = false;
        let before = form.request();
        if form.focus_first {
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
        }
        let response = egui::Modal::new(egui::Id::new("equipment-geometry-adoption")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 48.0).clamp(220.0, 760.0));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.heading(tr(l, "采纳为设计几何", "Adopt as design geometry"));
            ui.horizontal_wrapped(|ui| {
                let button = ui.button(tr(l, "取消", "Cancel"));
                if std::mem::take(&mut form.focus_first) { button.request_focus(); }
                cancel = button.clicked();
                if form.stage == AdoptionStage::Geometry {
                    back = ui.button(tr(l, "返回目标", "Back to target")).clicked();
                }
            });
            egui::ScrollArea::vertical().id_salt("adoption-body")
                .max_height((ctx.screen_rect().height() - 250.0).max(100.0))
                .show(ui, |ui| {
                    if let Some(source) = session.inputs().task(&form.source) {
                        ui.label(format!("{} · {} · {}", source.tag, source.name, source.case.name));
                    }
                    if form.stage == AdoptionStage::Target {
                        render_target(ui, l, &mut form, session.inputs());
                    } else {
                        adoption_preview::dimensions(ui, l, &mut form, units);
                        if let Some(preview) = preview {
                            adoption_preview::render(ui, l, preview.proposal(), session.inputs(), units);
                        }
                    }
                    if let Some(error) = &form.error {
                        ui.colored_label(egui::Color32::from_rgb(160,65,30), edit_error(l, error));
                    }
                });
            if preview.is_some() && !valid {
                ui.colored_label(egui::Color32::from_rgb(160,65,30), tr(l,
                    "预览已失效：源重算、相关输入或草稿已变化。编辑内容保留，请重新预览。",
                    "Preview expired: source execution, inputs or drafts changed. Edits are retained; preview again."));
            }
            ui.label(tr(l, "采纳只写设计输入，不自动校核。原样最小尺寸可能得到“边界待复核”。", "Adoption writes design inputs without running rating. An unchanged minimum size may require boundary review."));
            let unchanged = form.prepared_form.is_some() && form.prepared_form == form.request();
            ui.horizontal_wrapped(|ui| {
                prepare = ui.add_enabled(!self.equipment_target_busy(&form.source, None) && form.request().is_some(), egui::Button::new(tr(l, "预览采用几何", "Preview adopted geometry"))).clicked();
                if form.stage == AdoptionStage::Geometry {
                    confirm = ui.add_enabled(valid && unchanged, egui::Button::new(tr(l, "确认采纳", "Confirm adoption"))).clicked();
                }
            });
        });
        let changed = before != form.request();
        if changed {
            form.error = None;
            form.prepared_form = None;
        }
        if back {
            form.stage = AdoptionStage::Target;
            form.focus_first = true;
        }
        let return_focus = form.return_focus;
        self.equipment.adoption = Some(form);
        if changed || back {
            self.equipment_command(EquipmentEditorCommand::CancelGeometryAdoption);
        }
        if cancel || response.should_close() {
            self.cancel_geometry_adoption_form();
            self.equipment.adoption_return_focus = return_focus;
        } else if prepare {
            self.preview_geometry_adoption_form();
        } else if confirm && !changed {
            self.confirm_geometry_adoption_form();
        }
    }
}

fn render_target(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    form: &mut AdoptionForm,
    inputs: &EngineeringInputs,
) {
    ui.radio_value(
        &mut form.new_target,
        true,
        tr(
            l,
            "新建已有几何校核任务",
            "Create a given-geometry rating task",
        ),
    );
    if form.new_target {
        text_row(ui, tr(l, "位号", "Tag"), &mut form.tag);
        text_row(ui, tr(l, "名称", "Name"), &mut form.name);
        ui.checkbox(
            &mut form.copy_case,
            tr(
                l,
                "复制源工况的九项有效输入与来源",
                "Copy the nine effective case inputs and sources",
            ),
        );
        ui.label(tr(l, "不勾选则保留一条空工况。复制后的工况与来源独立保存。", "Leave unchecked for one empty case. Copied conditions and sources are saved independently."));
    }
    ui.radio_value(
        &mut form.new_target,
        false,
        tr(
            l,
            "更新指定的已有校核任务",
            "Update a specified existing rating task",
        ),
    );
    if !form.new_target {
        let label = form
            .existing
            .as_ref()
            .and_then(|id| inputs.equipment_task(id))
            .map(|t| task_label(l, t))
            .unwrap_or_else(|| tr(l, "选择目标…", "Choose target…").into());
        egui::ComboBox::from_id_salt("adoption-target")
            .selected_text(label)
            .width(ui.available_width().min(380.0))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut form.existing, None, tr(l, "未选择", "Not selected"));
                for task in inputs.tasks.iter().filter(|t| t.as_rating().is_some()) {
                    ui.selectable_value(
                        &mut form.existing,
                        Some(task.id().clone()),
                        task_label(l, task),
                    );
                }
            });
        ui.label(tr(l, "只替换共同几何；保留全部工况、输入和顺序。所有已有结果将过期。", "Replace common geometry only; preserve all cases, inputs and order. All existing results become stale."));
    }
}
