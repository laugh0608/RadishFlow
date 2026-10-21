use super::*;
use rf_ui::engineering::editor::separator_rating::*;
use rf_ui::engineering::{EngineeringSession, EquipmentCaseKey, EquipmentResultStatus};
pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    key: &EquipmentCaseKey,
    session: &EngineeringSession,
    units: &DisplayUnitSet,
    evidence_only: bool,
) -> Option<RatingIssueTarget> {
    let status = session.case_result_status(key);
    ui.heading(rating_labels::status_label(l, status));
    let Some(outcome) = session.historical_rating(key) else {
        ui.label(tr(
            l,
            "应用正式输入后，显式点击“校核当前工况”。",
            "Apply inputs, then explicitly select Rate current case.",
        ));
        return None;
    };
    if status == EquipmentResultStatus::Stale {
        ui.colored_label(
            egui::Color32::from_rgb(160, 65, 30),
            tr(
                l,
                "下方为过期证据，不代表当前几何与工况；请重新校核。",
                "Historical evidence below does not represent the current geometry and case. Rate again.",
            ),
        );
    }
    let mut requested = None;
    ui.scope(|ui| {
        if status == EquipmentResultStatus::Stale {
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
        }
        match outcome {
            Err(failure) => {
                for issue in &failure.issues {
                    if ui.link(rating_labels::issue_label(l, issue)).clicked() {
                        requested = Some(issue.target.clone());
                    }
                }
            }
            Ok(result) => requested = render_result(ui, l, result, units, evidence_only),
        }
    });
    requested
}
fn render_result(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    result: &SeparatorRatingResult,
    units: &DisplayUnitSet,
    evidence_only: bool,
) -> Option<RatingIssueTarget> {
    let p = result.performance();
    let capture = result.capture();
    let task = &capture.task;
    let case = task.cases.first().expect("single-case rating result");
    let mut requested = None;
    ui.heading(rating_labels::conclusion(l, p.conclusion()));
    ui.label(tr(
        l,
        "只校核所选气相与工作容积两项准则。超限是有效结论；边界待复核不等于通过。",
        "Only gas capacity and working volume criteria are checked. Exceedance is a valid result; boundary review is not a pass.",
    ));
    ui.label(tr(
        l,
        "流量限值仅在本次物性、K、f 与要求时间不变时成立，不是最大操作包络。",
        "Flow limits hold only with the captured properties, K, f and required time unchanged; they are not an operating envelope.",
    ));
    if !evidence_only {
        for (label, constraint) in [
            (tr(l, "气相处理能力", "Gas capacity"), &p.gas),
            (tr(l, "液体工作容积", "Liquid working volume"), &p.liquid),
        ] {
            ui.strong(format!(
                "{label} · {}",
                rating_labels::constraint(l, constraint.status)
            ));
            ui.label(format!(
                "{}：{}",
                tr(l, "负荷", "Load",),
                results::format_value(
                    constraint.load_fraction,
                    QuantityKind::DimensionlessRatio,
                    units
                )
            ));
            ui.label(format!(
                "{}：{}",
                tr(l, "余量（带符号）", "Signed margin",),
                signed_margin(constraint.margin_fraction, units)
            ));
        }
        use QuantityKind::*;
        for (label, value, q) in [
            (tr(l, "给定截面积", "Given area"), p.given_area_m2, Area),
            (
                tr(l, "可用工作容积", "Available working volume"),
                p.available_working_volume_m3,
                Volume,
            ),
            (
                tr(l, "要求工作容积", "Required working volume"),
                p.required_working_volume_m3,
                Volume,
            ),
            (
                tr(l, "允许气速", "Allowed gas velocity"),
                p.allowed_gas_velocity_m_s,
                Velocity,
            ),
            (
                tr(l, "实际表观气速", "Actual superficial gas velocity"),
                p.actual_gas_velocity_m_s,
                Velocity,
            ),
            (
                tr(l, "条件气相流量限值", "Conditional gas flow limit"),
                p.gas_flow_limit_m3_s,
                ActualVolumeFlow,
            ),
            (
                tr(l, "条件液相流量限值", "Conditional liquid flow limit"),
                p.liquid_flow_limit_m3_s,
                ActualVolumeFlow,
            ),
            (
                tr(l, "可用工作时间", "Available working time"),
                p.available_working_time_s,
                Duration,
            ),
        ] {
            ui.label(format!(
                "{label}：{}",
                results::format_value(value, q, units)
            ));
        }
    }
    for (label, value, q) in [
        (
            tr(
                l,
                "切线间总高（不含封头 / 裙座）",
                "Tangent-to-tangent height (excludes heads / skirt)",
            ),
            p.tangent_to_tangent_height_m,
            QuantityKind::Length,
        ),
        (
            tr(l, "缓冲容积 · NLL → HLL", "Buffer volume · NLL → HLL"),
            p.buffer_volume_m3,
            QuantityKind::Volume,
        ),
    ] {
        ui.label(format!(
            "{label}：{}",
            value
                .map(|v| results::format_value(v, q, units))
                .unwrap_or_else(|| tr(l, "未确定", "Undetermined",).into())
        ));
    }
    for section in result.missing_axial_sections() {
        if ui
            .link(format!(
                "{}：{}",
                tr(l, "未填写轴向段", "Missing axial section",),
                field_label(l, EquipmentField::Axial(*section))
            ))
            .clicked()
        {
            requested = Some(RatingIssueTarget::AxialSection(*section));
        }
    }
    for issue in result.height_issues() {
        if ui.link(rating_labels::issue_label(l, issue)).clicked() {
            requested = Some(issue.target.clone());
        }
    }
    ui.heading(tr(
        l,
        "假设与待复核来源",
        "Assumptions and sources awaiting review",
    ));
    for (label, ids) in [
        (
            tr(l, "假设来源", "Assumed source"),
            result.assumed_sources(),
        ),
        (
            tr(l, "待复核来源", "Pending review source"),
            result.pending_review_sources(),
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
    results::render_unchecked(ui, l, result.unchecked_requirements());
    egui::CollapsingHeader::new(tr(l, "本次校核输入与来源捕获", "Captured rating inputs and evidence"))
        .default_open(evidence_only)
        .show(ui, |ui| {
            ui.label(format!("{} · {} · {}", task.tag, case.name, case.fluid_system));
            ui.label(format!("{} v{}", task.method.id, task.method.version));
            ui.label(format!("{} / {} / {}", task.id, case.id, task.geometry.revision));
            for (field, value) in &capture.geometry_values_m {
                ui.label(format!("{}：{}", geometry_field_label(l, *field), results::format_value(*value, field.quantity(), units)));
                if let Some(binding) = task.geometry.inputs.get(field) {
                    results::render_binding_evidence(ui, l, binding, &capture.basis_entries, &capture.sources, units);
                }
            }
            for (field, value) in &capture.values_si {
                ui.label(format!("{}：{}", rating_field_label(l, *field), results::format_value(*value, field.quantity(), units)));
                if let Some(binding) = case.inputs.get(field) {
                    results::render_binding_evidence(ui, l, binding, &capture.basis_entries, &capture.sources, units);
                }
            }
            for (field, value) in &capture.axial_lengths_m {
                ui.label(format!(
                    "{}：{}",
                    field_label(l, EquipmentField::Axial(*field)),
                    results::format_value(*value, QuantityKind::Length, units)
                ));
                if let Some(binding) = task.geometry.axial_sections.get(field) {
                    results::render_binding_evidence(ui, l, binding, &capture.basis_entries, &capture.sources, units);
                }
            }
            ui.label(tr(
                l,
                "给定圆筒：A = πD²/4；V = A·h；气相负荷 = Qg/(A·uallow)；液体负荷 = Ql·t/V；余量 = 1 − 负荷。",
                "Given cylinder: A = πD²/4; V = A·h; gas load = Qg/(A·uallow); liquid load = Ql·t/V; margin = 1 − load.",
            ));
            ui.label(format!(
                "{}: {} / {}; {} / {}",
                tr(l, "未舍入负荷 / 余量（比值）", "Unrounded load / margin (ratio)",),
                p.gas.load_fraction,
                p.gas.margin_fraction,
                p.liquid.load_fraction,
                p.liquid.margin_fraction
            ));
            ui.label(tr(
                l,
                "边界比较阈值为 1e-10，仅处理浮点比较，不是工程容差；判定不随显示舍入。",
                "The 1e-10 boundary threshold handles numerical comparison, not engineering tolerance. Display rounding never determines the conclusion.",
            ));
            results::render_sources(ui, l, &capture.sources, units);
        });
    requested
}
fn signed_margin(value: f64, units: &DisplayUnitSet) -> String {
    let text = results::format_value(value, QuantityKind::DimensionlessRatio, units);
    if value > 0.0 {
        format!("+{text}")
    } else {
        text
    }
}
