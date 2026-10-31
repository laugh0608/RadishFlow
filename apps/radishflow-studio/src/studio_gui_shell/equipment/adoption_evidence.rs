//! Read the stored design decision even when no rating result or source task exists.
use super::*;
use rf_model::{engineering::EngineeringBasis, equipment::*};

pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    geometry: &SeparatorGivenGeometry,
    basis: &EngineeringBasis,
    units: &DisplayUnitSet,
) {
    let Some(a) = &geometry.adoption else {
        return;
    };
    ui.label(tr(
        l,
        "由初步尺寸候选采纳的设计几何",
        "Design geometry adopted from preliminary sizing",
    ));
    ui.label(tr(l, "固定证据保留采纳时的输入、来源与审阅状态。手改当前几何不会修改历史决定。", "Fixed evidence retains inputs, sources and review states at adoption. Editing current geometry does not change that decision."));
    let source = &a.source_task;
    ui.label(format!(
        "{}: {} / {} / {}",
        tr(l, "源工程 / 任务 / 工况", "Source document / task / case"),
        a.source_document_id,
        source.id,
        source.case.id
    ));
    ui.label(format!(
        "{} · {} · {}",
        source.tag, source.name, source.case.name
    ));
    ui.label(format!(
        "{}: {} · {} v{}",
        tr(l, "源修订与方法", "Source revision and method"),
        source.revision,
        source.method.id,
        source.method.version
    ));
    for (field, dimension) in [
        (SeparatorGeometryInput::InternalDiameter, &a.diameter),
        (
            SeparatorGeometryInput::WorkingLiquidHeight,
            &a.working_liquid_height,
        ),
    ] {
        ui.separator();
        ui.strong(geometry_field_label(l, field));
        ui.label(format!(
            "{}: {} m",
            tr(l, "候选原始 SI", "Original candidate SI"),
            dimension.original_m
        ));
        ui.label(format!(
            "{}: {} ({} m)",
            tr(l, "采纳时采用值", "Value at adoption"),
            results::format_value(dimension.adopted_m, QuantityKind::Length, units),
            dimension.adopted_m
        ));
        let current = geometry
            .inputs
            .get(&field)
            .and_then(|b| basis.resolve(QuantityKind::Length, b).ok())
            .map(|v| v.value.value_si);
        compare(ui, l, current, Some(dimension.adopted_m), units);
        ui.label(format!(
            "{}: {}",
            tr(l, "调整理由", "Adjustment reason"),
            nonempty(
                &dimension.reason,
                tr(
                    l,
                    "原样采用（保留未舍入 SI）",
                    "Used unchanged (unrounded SI retained)"
                )
            )
        ));
    }
    egui::CollapsingHeader::new(tr(l, "轴向段对照", "Axial segments"))
        .id_salt("adoption-axial-evidence")
        .show(ui, |ui| {
            for field in SeparatorAxialSection::ALL {
                ui.strong(field_label(l, EquipmentField::Axial(field)));
                let adopted = a.axial_lengths_m.get(&field).copied();
                let current = geometry
                    .axial_sections
                    .get(&field)
                    .and_then(|b| basis.resolve(QuantityKind::Length, b).ok())
                    .map(|v| v.value.value_si);
                ui.label(format!(
                    "{}: {}",
                    tr(l, "采纳时", "At adoption"),
                    length(l, adopted, units)
                ));
                compare(ui, l, current, adopted, units);
            }
        });
    egui::CollapsingHeader::new(tr(l, "原始输入与来源", "Inputs and sources"))
        .id_salt("adoption-original-evidence")
        .show(ui, |ui| original(ui, l, a, units));
}

pub(super) fn length(l: StudioShellLocale, value: Option<f64>, units: &DisplayUnitSet) -> String {
    value
        .map(|v| results::format_value(v, QuantityKind::Length, units))
        .unwrap_or_else(|| tr(l, "未知", "Unknown").into())
}
fn compare(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    current: Option<f64>,
    adopted: Option<f64>,
    units: &DisplayUnitSet,
) {
    ui.label(format!(
        "{}: {}",
        tr(l, "当前正式值", "Current committed value"),
        length(l, current, units)
    ));
    if current != adopted {
        ui.colored_label(
            egui::Color32::from_rgb(160, 65, 30),
            tr(
                l,
                "当前值与采纳时不同；历史证据保持。",
                "Current value differs from adoption; historical evidence is retained.",
            ),
        );
        if let Some(current) = current {
            ui.small(format!(
                "{}: {current} m",
                tr(l, "当前未舍入 SI", "Current unrounded SI")
            ));
        }
    }
}
fn original(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    a: &SeparatorGeometryAdoption,
    units: &DisplayUnitSet,
) {
    let sources = a
        .source_basis
        .sources
        .iter()
        .map(|s| (s.id.clone(), s.clone()))
        .collect();
    let entries = a
        .source_basis
        .basis_entries
        .iter()
        .map(|b| (b.id.clone(), b.clone()))
        .collect();
    let source = &a.source_task;
    ui.label(format!(
        "{} · {}",
        source.case.purpose, source.case.fluid_system
    ));
    ui.label(&source.case.applicability.internals_description);
    applicability::render_readonly(ui, l, &source.case.applicability);
    for id in &source.case.applicability_source_ids {
        ui.label(format!(
            "{}: {id}",
            tr(l, "适用声明来源", "Applicability source")
        ));
    }
    for (field, binding) in &source.case.inputs {
        ui.strong(field_label(l, EquipmentField::Input(*field)));
        if let Ok(value) = a.source_basis.resolve(field.quantity(), binding) {
            ui.label(results::format_value(
                value.value.value_si,
                field.quantity(),
                units,
            ));
        }
        results::render_binding_evidence(ui, l, binding, &entries, &sources, units);
    }
    for (field, binding) in &source.axial_sections {
        ui.strong(field_label(l, EquipmentField::Axial(*field)));
        if let Ok(value) = a.source_basis.resolve(QuantityKind::Length, binding) {
            ui.label(results::format_value(
                value.value.value_si,
                QuantityKind::Length,
                units,
            ));
        }
        ui.label(if a.axial_lengths_m.contains_key(field) {
            tr(l, "此段已采用", "This segment was adopted")
        } else {
            tr(
                l,
                "此段未采用；原始绑定仅作证据保留",
                "This segment was not adopted; its original binding is retained as evidence",
            )
        });
        results::render_binding_evidence(ui, l, binding, &entries, &sources, units);
    }
    results::render_sources(ui, l, &sources, units);
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn open_adoption_evidence(
        &mut self,
        task: rf_types::EquipmentTaskId,
        return_focus: Option<egui::Id>,
    ) {
        self.equipment.adoption_evidence = Some((task, return_focus));
        self.equipment.adoption_evidence_first = true;
    }
    pub(super) fn render_adoption_evidence(&mut self, ctx: &egui::Context) {
        let Some((task, return_focus)) = self.equipment.adoption_evidence.clone() else {
            return;
        };
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let inputs = workspace.engineering.inputs();
        let Some(geometry) = inputs.rating_task(&task).map(|t| &t.geometry) else {
            self.equipment.adoption_evidence = None;
            return;
        };
        let l = self.locale;
        let mut close = false;
        let first = std::mem::take(&mut self.equipment.adoption_evidence_first);
        if first {
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
        }
        let response = egui::Modal::new(egui::Id::new("fixed-adoption-evidence")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 48.0).clamp(220.0, 720.0));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.heading(tr(l, "固定采纳证据", "Fixed adoption evidence"));
            let back = ui.button(tr(l, "返回共同几何", "Return to common geometry"));
            if first { back.request_focus(); }
            close = back.clicked();
            egui::ScrollArea::vertical().max_height((ctx.screen_rect().height() - 180.0).max(100.0)).show(ui, |ui| {
                if let Some(a) = &geometry.adoption {
                    if inputs.task(&a.source_task.id).is_none() {
                        ui.label(tr(l, "源任务已不存在；采纳值和固定证据仍保留。", "The source task is no longer present; adopted values and fixed evidence remain."));
                    } else {
                        ui.label(tr(l, "源任务仍在工程中；下方展示采纳时的副本。再次采纳请从当前尺寸候选发起。", "The source task is still in the project; the copy below is from adoption. Start another adoption from a current sizing candidate."));
                    }
                }
                render(ui, l, geometry, &inputs.basis, workspace.project_presentation.display_units());
            });
        });
        if close || response.should_close() {
            self.equipment.adoption_evidence = None;
            self.equipment.adoption_return_focus = return_focus;
        }
    }
}
