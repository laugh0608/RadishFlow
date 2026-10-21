use super::*;
use rf_model::equipment::*;

fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    key: &str,
    value: &mut Option<T>,
    options: &[(T, &str)],
) {
    ui.label(key);
    let label = options
        .iter()
        .find(|(v, _)| Some(*v) == *value)
        .map(|(_, label)| *label)
        .unwrap_or(tr(locale, "未声明", "Not declared"));
    egui::ComboBox::from_id_salt(key)
        .selected_text(label)
        .width(ui.available_width().min(360.0))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, tr(locale, "未声明", "Not declared"));
            for (v, label) in options {
                ui.selectable_value(value, Some(*v), *label);
            }
        });
}
pub(super) fn orientation(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorOrientation>,
) {
    choice(
        ui,
        l,
        tr(l, "方向", "Orientation"),
        value,
        &[
            (SeparatorOrientation::Vertical, tr(l, "立式", "Vertical")),
            (
                SeparatorOrientation::Horizontal,
                tr(l, "卧式", "Horizontal"),
            ),
        ],
    );
}

pub(super) fn gas_direction(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorGasDirection>,
) {
    choice(
        ui,
        l,
        tr(l, "气流方向", "Gas direction"),
        value,
        &[
            (SeparatorGasDirection::Upward, tr(l, "向上", "Upward")),
            (SeparatorGasDirection::Other, tr(l, "其他", "Other")),
        ],
    );
}

pub(super) fn service(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorService>,
) {
    choice(
        ui,
        l,
        tr(l, "介质工况", "Service"),
        value,
        &[
            (
                SeparatorService::CleanNonFoamingTwoPhase,
                tr(
                    l,
                    "洁净、不起泡气液两相",
                    "Clean nonfoaming gas–liquid two phase",
                ),
            ),
            (SeparatorService::SinglePhase, tr(l, "单相", "Single phase")),
            (SeparatorService::Foaming, tr(l, "起泡", "Foaming")),
            (SeparatorService::Solids, tr(l, "含固体", "Solids")),
            (SeparatorService::Slugging, tr(l, "段塞", "Slugging")),
            (SeparatorService::ThreePhase, tr(l, "三相", "Three phase")),
        ],
    );
}

pub(super) fn geometry(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorSectionGeometry>,
) {
    choice(
        ui,
        l,
        tr(l, "截面几何", "Section geometry"),
        value,
        &[
            (
                SeparatorSectionGeometry::FullEqualDiameterCylinder,
                tr(l, "完整等径圆柱", "Full equal-diameter cylinder"),
            ),
            (
                SeparatorSectionGeometry::ReducedEffectiveArea,
                tr(l, "有效面积缩减", "Reduced effective area"),
            ),
            (
                SeparatorSectionGeometry::NonCylindrical,
                tr(l, "非圆柱", "Noncylindrical"),
            ),
        ],
    );
}

pub(super) fn k_basis(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorKBasis>,
) {
    choice(
        ui,
        l,
        tr(l, "K 的适用范围", "K applicability"),
        value,
        &[
            (
                SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals,
                tr(
                    l,
                    "匹配所述内件的整罐有效 K",
                    "Effective whole-vessel K for declared internals",
                ),
            ),
            (
                SeparatorKBasis::MistEliminatorOnly,
                tr(l, "仅除沫器 K", "Mist eliminator only"),
            ),
            (
                SeparatorKBasis::UnmatchedInternals,
                tr(l, "内件不匹配", "Unmatched internals"),
            ),
        ],
    );
}

pub(super) fn flow_basis(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    value: &mut Option<SeparatorFlowBasis>,
) {
    choice(
        ui,
        l,
        tr(l, "流量基准", "Flow basis"),
        value,
        &[
            (
                SeparatorFlowBasis::ActualAtOperatingConditions,
                tr(l, "操作条件实际流量", "Actual at operating conditions"),
            ),
            (
                SeparatorFlowBasis::StandardConditions,
                tr(l, "标准状态流量", "Standard conditions"),
            ),
        ],
    );
}
