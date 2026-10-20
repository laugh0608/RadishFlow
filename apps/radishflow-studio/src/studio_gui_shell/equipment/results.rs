use super::*;
use rf_ui::engineering::editor::separator_sizing::*;
use rf_ui::engineering::{EngineeringSession, EquipmentResultStatus};

pub(super) fn format_value(value: f64, q: QuantityKind, units: &DisplayUnitSet) -> String {
    match rf_ui::format_quantity(value, q, units.unit_for(q)) {
        Ok(text) => text,
        Err(error) => format!("⚠ {error}"),
    }
}
pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    id: &rf_types::EquipmentTaskId,
    session: &EngineeringSession,
    units: &DisplayUnitSet,
    evidence_only: bool,
) -> Option<IssueTarget> {
    let status = session.result_status(id);
    ui.heading(status_label(l, status));
    let Some(outcome) = session.historical_outcome(id) else {
        ui.label(tr(
            l,
            "应用输入后，显式点击“计算尺寸”。",
            "Apply inputs, then explicitly select Calculate size.",
        ));
        return None;
    };
    let stale = status == EquipmentResultStatus::Stale;
    if stale {
        ui.colored_label(egui::Color32::from_rgb(160,65,30),tr(l,"下方是历史计算证据，不能作为当前候选使用。请重新计算。","Historical calculation evidence below is not a current candidate. Recalculate explicitly."));
    }
    let mut requested = None;
    ui.scope(|ui| {
        if stale {
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
        }
        match outcome {
            Err(failure) => {
                for issue in &failure.issues {
                    if ui.link(issue_label(l, issue)).clicked() {
                        requested = Some(issue.target.clone());
                    }
                }
            }
            Ok(candidate) => render_candidate(ui, l, candidate, units, evidence_only),
        }
    });
    requested
}
fn render_candidate(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    c: &SeparatorCandidate,
    units: &DisplayUnitSet,
    evidence_only: bool,
) {
    let d = c.dimensions();
    let capture = c.capture();
    ui.label(tr(l,"条件式初步工艺尺寸；未作标准直径圆整或机械设计。","Conditional preliminary process dimensions; no standard diameter rounding or mechanical design."));
    ui.label(format!(
        "{}：{} v{}",
        tr(l, "方法", "Method"),
        capture.task.method.id,
        capture.task.method.version
    ));
    if !evidence_only {
        ui.heading(tr(l, "主尺寸", "Principal dimensions"));
        for (label, value, q) in [
            (
                tr(l, "最小直径 Dmin", "Minimum diameter Dmin"),
                d.minimum_diameter_m,
                QuantityKind::Length,
            ),
            (
                tr(l, "工作容积 · LLL → NLL", "Working volume · LLL → NLL"),
                d.working_volume_m3,
                QuantityKind::Volume,
            ),
            (
                tr(l, "工作液层高度", "Working liquid height"),
                d.working_liquid_height_m,
                QuantityKind::Length,
            ),
        ] {
            ui.label(format!("{label}：{}", format_value(value, q, units)));
        }
        ui.label(format!(
            "{}：{}",
            tr(l, "控制约束", "Controlling constraint"),
            match d.controlling_constraint {
                ControllingConstraint::GasArea => tr(l, "气相面积", "Gas area"),
                ControllingConstraint::LiquidWorkingVolume =>
                    tr(l, "液相工作容积", "Liquid working volume"),
                ControllingConstraint::Both => tr(l, "共同控制", "Both"),
            }
        ));
        ui.label(format!(
            "{}：{}",
            tr(
                l,
                "切线间总高（不含封头 / 裙座）",
                "Tangent-to-tangent height (excludes heads / skirt)"
            ),
            d.tangent_to_tangent_height_m
                .map(|v| format_value(v, QuantityKind::Length, units))
                .unwrap_or_else(|| tr(
                    l,
                    "未知 · 轴向区段未齐全或无效",
                    "Unknown · incomplete or invalid axial sections"
                )
                .into())
        ));
        ui.label(format!(
            "{}：{}",
            tr(l, "缓冲容积 · NLL → HLL", "Buffer volume · NLL → HLL"),
            d.buffer_volume_m3
                .map(|v| format_value(v, QuantityKind::Volume, units))
                .unwrap_or_else(|| tr(l, "未知", "Unknown").into())
        ));
        for section in c.missing_axial_sections() {
            ui.label(format!(
                "{}：{}",
                tr(l, "未填写", "Missing"),
                field_label(l, EquipmentField::Axial(*section))
            ));
        }
        for issue in c.height_issues() {
            ui.label(issue_label(l, issue));
        }
    }
    egui::CollapsingHeader::new(tr(l, "公式与中间值", "Formulas and values"))
        .default_open(true)
        .show(ui, |ui| {
            ui.label("uSB = K × √[(ρL − ρG) / ρG]; uallow = f × uSB");
            ui.label("Ag = Qg / uallow; Vw = QL × tw; AL = Vw / hw,max");
            ui.label("A = max(Ag, AL); Dmin = √(4A / π); hw = Vw / A");
            for (label, value, q) in [
                ("uSB", d.souders_brown_velocity_m_s, QuantityKind::Velocity),
                ("uallow", d.allowed_gas_velocity_m_s, QuantityKind::Velocity),
                ("Ag", d.gas_area_m2, QuantityKind::Area),
                ("AL", d.liquid_area_m2, QuantityKind::Area),
                ("A", d.required_area_m2, QuantityKind::Area),
                (
                    tr(l, "气相负荷比例", "Gas load fraction"),
                    d.gas_load_fraction,
                    QuantityKind::DimensionlessRatio,
                ),
            ] {
                ui.label(format!("{label}：{}", format_value(value, q, units)));
            }
        });
    ui.heading(tr(
        l,
        "假设与待复核来源",
        "Assumptions and sources awaiting review",
    ));
    for (label, ids) in [
        (tr(l, "假设来源", "Assumed source"), c.assumed_sources()),
        (
            tr(l, "待复核来源", "Pending review source"),
            c.pending_review_sources(),
        ),
    ] {
        for id in ids {
            if let Some(source) = capture.sources.get(id) {
                ui.label(format!(
                    "{label}：{} · {}",
                    source.title, source.adoption_note
                ));
            }
        }
    }
    if c.assumed_sources().is_empty() && c.pending_review_sources().is_empty() {
        ui.label(tr(
            l,
            "本次捕获中没有假设或待复核来源标记。",
            "No assumed or pending-review source flags in this capture.",
        ));
    }
    ui.heading(tr(l, "未校核项目", "Unchecked requirements"));
    for requirement in c.unchecked_requirements() {
        ui.label(match requirement {
            UncheckedRequirement::PhaseEquilibrium => tr(
                l,
                "相平衡与物性可信度",
                "Phase equilibrium and property validity",
            ),
            UncheckedRequirement::EntrainmentAndEfficiency => {
                tr(l, "夹带与分离效率", "Entrainment and separation efficiency")
            }
            UncheckedRequirement::PressureDrop => tr(l, "压降", "Pressure drop"),
            UncheckedRequirement::SlugHandling => tr(l, "段塞处理", "Slug handling"),
            UncheckedRequirement::MechanicalDesignAndCertification => {
                tr(l, "机械设计与认证", "Mechanical design and certification")
            }
            UncheckedRequirement::StandardDiameterRounding => {
                tr(l, "标准直径圆整", "Standard diameter rounding")
            }
            UncheckedRequirement::MultipleOperatingCases => {
                tr(l, "多工况包络", "Multiple operating cases")
            }
        });
    }
    egui::CollapsingHeader::new(tr(l, "本次计算输入与来源快照", "Captured evidence"))
        .default_open(evidence_only)
        .show(ui, |ui| {
            ui.label(format!(
                "{} · {} · {}",
                capture.task.tag, capture.task.case.name, capture.task.case.fluid_system
            ));
            for (field, value) in &capture.values_si {
                ui.label(format!(
                    "{}：{}",
                    field_label(l, EquipmentField::Input(*field)),
                    format_value(*value, field.quantity(), units)
                ));
                if let Some(binding) = capture.task.case.inputs.get(field) {
                    render_binding_evidence(ui, l, binding, capture, units);
                }
            }
            for (field, value) in &capture.axial_lengths_m {
                ui.label(format!(
                    "{}：{}",
                    field_label(l, EquipmentField::Axial(*field)),
                    format_value(*value, QuantityKind::Length, units)
                ));
                if let Some(binding) = capture.task.axial_sections.get(field) {
                    render_binding_evidence(ui, l, binding, capture, units);
                }
            }
            for source in capture.sources.values() {
                ui.separator();
                ui.strong(&source.title);
                ui.label(format!(
                    "{} · {}",
                    source_kind(l, source.kind),
                    source_review(l, source.review)
                ));
                for text in [
                    &source.reference,
                    &source.locator,
                    &source.publication_version,
                    &source.adoption_note,
                    &source.applicability.scope,
                ] {
                    if !text.is_empty() {
                        ui.label(text);
                    }
                }
                ui.label(format!(
                    "{}：{}",
                    tr(l, "物系", "Fluid system"),
                    source.applicability.fluid_system
                ));
                if let Some(conditions) = source.applicability.operating_conditions {
                    ui.label(format!(
                        "T = {}; P = {}",
                        format_value(
                            conditions.temperature_k,
                            QuantityKind::AbsoluteTemperature,
                            units
                        ),
                        format_value(
                            conditions.pressure_pa,
                            QuantityKind::AbsolutePressure,
                            units
                        )
                    ));
                }
                if let Some(date) = source.accessed_on {
                    ui.label(date.to_string());
                }
                for observation in &source.observations {
                    ui.label(format!(
                        "{} {} → {} · {}",
                        observation.original_value,
                        observation.original_unit,
                        format_value(observation.canonical_value, observation.quantity, units),
                        observation.conversion_note
                    ));
                }
            }
        });
}

fn render_binding_evidence(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    binding: &rf_model::engineering::InputBinding,
    capture: &SizingInputCapture,
    units: &DisplayUnitSet,
) {
    use rf_model::engineering::InputBinding;
    let (basis_id, source_id, reason) = match binding {
        InputBinding::Direct { source_id, .. } => (None, Some(source_id), None),
        InputBinding::BasisRef { basis_entry_id } => (Some(basis_entry_id), None, None),
        InputBinding::Override {
            basis_entry_id,
            source_id,
            reason,
            ..
        } => (Some(basis_entry_id), Some(source_id), Some(reason)),
    };
    if let Some(basis) = basis_id.and_then(|id| capture.basis_entries.get(id)) {
        ui.label(format!(
            "{}：{} = {}",
            tr(l, "原始基础", "Original basis"),
            basis.name,
            format_value(basis.value.value_si, basis.value.quantity, units)
        ));
        if let Some(source) = capture.sources.get(&basis.source_id) {
            ui.label(format!(
                "{}：{}",
                tr(l, "基础来源", "Basis source"),
                source.title
            ));
        }
    }
    if let Some(source) = source_id.and_then(|id| capture.sources.get(id)) {
        ui.label(format!(
            "{}：{}",
            tr(l, "有效输入来源", "Effective input source"),
            source.title
        ));
    }
    if let Some(reason) = reason {
        ui.label(format!(
            "{}：{reason}",
            tr(l, "局部覆盖理由", "Local override reason")
        ));
    }
}
