use super::*;

impl ReadyAppState {
    pub(super) fn render_equipment_source_snapshot(
        &mut self,
        ui: &mut egui::Ui,
        target: &EquipmentEditorTarget,
        inputs: &EngineeringInputs,
        units: &DisplayUnitSet,
        commands: &mut Vec<EquipmentEditorCommand>,
    ) {
        let EquipmentEditorTarget::Source(id) = target else {
            return;
        };
        ui.heading(tr(
            self.locale,
            "来源详情 · 正式记录",
            "Source details · committed record",
        ));
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(tr(self.locale, "返回输入字段", "Return to input field"))
                .clicked()
                && let Some((target, field)) = self.equipment.source_return.take()
            {
                self.equipment.source_readonly = false;
                self.equipment.focus_field = field;
                self.equipment.section = EquipmentSection::Inputs;
                commands.push(EquipmentEditorCommand::Select(target));
            }
            if ui
                .button(tr(self.locale, "编辑来源", "Edit source"))
                .clicked()
            {
                self.equipment.source_readonly = false;
            }
        });
        if let Some(source) = inputs.basis.sources.iter().find(|s| s.id == *id) {
            egui::ScrollArea::vertical()
                .id_salt(("rating-source", id))
                .show(ui, |ui| {
                    results::render_sources(
                        ui,
                        self.locale,
                        &std::collections::BTreeMap::from([(id.clone(), source.clone())]),
                        units,
                    );
                });
        }
    }
}
use rf_model::engineering::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    draft: &mut SourceDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: bool,
) {
    ui.label(tr(l,"工程来源由用户记录与采用；软件不自动认可外部数值。","Sources are recorded and adopted by the user; external values are not endorsed automatically."));
    let title = text_row(ui, tr(l, "来源标题", "Source title"), &mut draft.title);
    if focus {
        title.request_focus();
    }
    egui::ComboBox::from_id_salt("source-kind")
        .selected_text(source_kind(l, draft.kind))
        .show_ui(ui, |ui| {
            for kind in [
                SourceKind::Unknown,
                SourceKind::Measured,
                SourceKind::Vendor,
                SourceKind::Literature,
                SourceKind::Estimated,
                SourceKind::Assumed,
            ] {
                ui.selectable_value(&mut draft.kind, kind, source_kind(l, kind));
            }
        });
    text_row(
        ui,
        tr(l, "引用 / URL / 文献", "Reference / URL / publication"),
        &mut draft.reference,
    );
    text_row(
        ui,
        tr(l, "页码 / 表号 / 数据定位", "Page / table / data locator"),
        &mut draft.locator,
    );
    text_row(
        ui,
        tr(l, "版本 / 发布信息", "Version / publication"),
        &mut draft.publication_version,
    );
    text_row(
        ui,
        tr(l, "访问日期 · YYYY-MM-DD", "Access date · YYYY-MM-DD"),
        &mut draft.accessed_on,
    );
    multiline(
        ui,
        tr(l, "采用理由 / 假设说明", "Adoption rationale / assumption"),
        &mut draft.adoption_note,
    );
    ui.label(tr(l, "审阅与采用状态", "Review and adoption status"));
    egui::ComboBox::from_id_salt("source-review")
        .selected_text(source_review(l, draft.review))
        .show_ui(ui, |ui| {
            for review in [
                SourceReview::Unreviewed,
                SourceReview::Adopted,
                SourceReview::Reviewed,
            ] {
                ui.selectable_value(&mut draft.review, review, source_review(l, review));
            }
        });
    ui.separator();
    ui.heading(tr(l, "来源适用条件", "Source applicability"));
    text_row(
        ui,
        tr(l, "物系（须与工况一致）", "Fluid system (match the case)"),
        &mut draft.fluid_system,
    );
    let phases = [
        (
            SourcePhase::NotApplicable,
            tr(l, "不适用", "Not applicable"),
        ),
        (SourcePhase::Gas, tr(l, "气相", "Gas")),
        (SourcePhase::Liquid, tr(l, "液相", "Liquid")),
        (SourcePhase::Both, tr(l, "气液两相", "Both gas and liquid")),
    ];
    egui::ComboBox::from_id_salt("source-phase")
        .selected_text(phases.iter().find(|(p, _)| *p == draft.phase).unwrap().1)
        .show_ui(ui, |ui| {
            for (phase, label) in phases {
                ui.selectable_value(&mut draft.phase, phase, label);
            }
        });
    ui.label(tr(
        l,
        "操作温度（与压力成对填写或同时留空）",
        "Operating temperature (enter both T/P or leave both empty)",
    ));
    quantity(
        ui,
        l,
        "source-temperature",
        QuantityKind::AbsoluteTemperature,
        &mut draft.temperature,
        false,
        tr(l, "来源操作温度", "Source operating temperature"),
    );
    ui.label(tr(l, "操作绝对压力", "Operating absolute pressure"));
    quantity(
        ui,
        l,
        "source-pressure",
        QuantityKind::AbsolutePressure,
        &mut draft.pressure,
        false,
        tr(l, "来源操作绝对压力", "Source operating absolute pressure"),
    );
    multiline(
        ui,
        tr(l, "适用范围", "Applicability scope"),
        &mut draft.scope,
    );
    ui.separator();
    ui.heading(tr(
        l,
        "原始观测与换算记录",
        "Original observations and conversions",
    ));
    let mut remove = None;
    for (index, o) in draft.observations.iter_mut().enumerate() {
        ui.push_id(&o.id, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.strong(format!("{} {}", tr(l, "观测", "Observation"), index + 1));
                quantity_selector(ui, l, &mut o.quantity, &mut o.canonical, units);
                text_row(
                    ui,
                    tr(l, "原始数值", "Original value"),
                    &mut o.original_value,
                );
                text_row(
                    ui,
                    tr(l, "原始单位原文", "Original unit as recorded"),
                    &mut o.original_unit,
                );
                ui.label(tr(l, "换算后数值", "Converted value"));
                quantity(
                    ui,
                    l,
                    "observation-value",
                    o.quantity,
                    &mut o.canonical,
                    false,
                    tr(l, "换算后数值", "Converted value"),
                );
                multiline(
                    ui,
                    tr(l, "换算 / 推导说明", "Conversion / derivation note"),
                    &mut o.conversion_note,
                );
                egui::CollapsingHeader::new(tr(l, "依赖的原始观测", "Underlying observations"))
                    .show(ui, |ui| {
                        for source in &inputs.basis.sources {
                            for (ordinal, observation) in source.observations.iter().enumerate() {
                                let dependency = SourceObservationRef {
                                    source_id: source.id.clone(),
                                    observation_id: observation.id.clone(),
                                };
                                let mut checked = o.depends_on.contains(&dependency);
                                if ui
                                    .checkbox(
                                        &mut checked,
                                        format!(
                                            "{} · {} {} · {} {}",
                                            source.title,
                                            tr(l, "观测", "Observation"),
                                            ordinal + 1,
                                            observation.original_value,
                                            observation.original_unit
                                        ),
                                    )
                                    .changed()
                                {
                                    if checked {
                                        o.depends_on.push(dependency);
                                    } else {
                                        o.depends_on.retain(|d| d != &dependency);
                                    }
                                }
                            }
                        }
                    });
                if ui
                    .button(tr(l, "移除此观测", "Remove observation"))
                    .clicked()
                {
                    remove = Some(index);
                }
            });
        });
    }
    if let Some(index) = remove {
        draft.observations.remove(index);
    }
    if ui.button(tr(l, "添加观测", "Add observation")).clicked() {
        let mut n = draft.observations.len() + 1;
        let id = loop {
            let id = rf_types::SourceObservationId::new(format!("observation-{n}"));
            if !draft.observations.iter().any(|o| o.id == id) {
                break id;
            }
            n += 1;
        };
        draft.observations.push(ObservationDraft {
            id,
            quantity: QuantityKind::Duration,
            original_value: String::new(),
            original_unit: String::new(),
            canonical: QuantityDraft::empty(units.unit_for(QuantityKind::Duration)),
            conversion_note: String::new(),
            depends_on: Vec::new(),
        });
    }
}
fn quantity_selector(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    q: &mut QuantityKind,
    value: &mut QuantityDraft,
    units: &DisplayUnitSet,
) {
    let before = *q;
    egui::ComboBox::from_id_salt("quantity-kind")
        .selected_text(quantity_name(l, *q))
        .show_ui(ui, |ui| {
            for candidate in rf_types::units::ALL_QUANTITIES {
                ui.selectable_value(q, *candidate, quantity_name(l, *candidate));
            }
        });
    if *q != before {
        *value = QuantityDraft::empty(units.unit_for(*q));
    }
}
pub(super) fn render_basis(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    draft: &mut BasisDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
) {
    text_row(
        ui,
        tr(l, "基础条目名称", "Basis entry name"),
        &mut draft.name,
    );
    ui.label(tr(
        l,
        "量种（更改量种后需重新填写数值）",
        "Quantity (changing it clears the value)",
    ));
    quantity_selector(ui, l, &mut draft.quantity, &mut draft.value, units);
    quantity(
        ui,
        l,
        "basis-value",
        draft.quantity,
        &mut draft.value,
        false,
        tr(l, "基础条目数值", "Basis entry value"),
    );
    source_picker(ui, l, "basis-source", &mut draft.source, inputs);
}
pub(super) fn render_references(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    error: &EquipmentEditError,
    inputs: &EngineeringInputs,
) {
    use rf_ui::engineering::EngineeringReference::*;
    if let EquipmentEditError::Domain(rf_ui::engineering::EngineeringError::Referenced {
        references,
    }) = error
    {
        for reference in references {
            let label = match reference {
                BasisSource(id) => {
                    target_label(l, &EquipmentEditorTarget::Basis(id.clone()), inputs)
                }
                SourceObservation {
                    source,
                    observation,
                } => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::Source(source.clone()), inputs),
                    observation.as_str()
                ),
                TaskInput { task, field } => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::Task(task.clone()), inputs),
                    field_label(l, EquipmentField::Input(*field))
                ),
                TaskAxialSection { task, section } => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::Task(task.clone()), inputs),
                    field_label(l, EquipmentField::Axial(*section))
                ),
                RatingGeometryInput { task, field } => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::RatingTask(task.clone()), inputs),
                    geometry_field_label(l, *field)
                ),
                RatingCaseInput { task, case, field } => format!(
                    "{} · {} · {}",
                    target_label(l, &EquipmentEditorTarget::RatingTask(task.clone()), inputs),
                    target_label(
                        l,
                        &EquipmentEditorTarget::RatingCase(rf_ui::engineering::EquipmentCaseKey {
                            task_id: task.clone(),
                            case_id: case.clone()
                        }),
                        inputs
                    ),
                    rating_field_label(l, *field)
                ),
                RatingGeometryApplicability(id) => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::RatingTask(id.clone()), inputs),
                    tr(l, "结构适用性依据", "Geometry applicability evidence")
                ),
                RatingCaseApplicability { task, case } => format!(
                    "{} · {}",
                    target_label(
                        l,
                        &EquipmentEditorTarget::RatingCase(rf_ui::engineering::EquipmentCaseKey {
                            task_id: task.clone(),
                            case_id: case.clone()
                        }),
                        inputs
                    ),
                    tr(l, "工况适用性依据", "Case applicability evidence")
                ),
                TaskApplicability(id) => format!(
                    "{} · {}",
                    target_label(l, &EquipmentEditorTarget::Task(id.clone()), inputs),
                    tr(l, "适用性依据", "Applicability evidence")
                ),
            };
            ui.label(label);
        }
    }
}
