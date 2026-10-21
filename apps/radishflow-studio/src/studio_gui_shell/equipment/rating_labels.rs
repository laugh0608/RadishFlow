use super::*;
use rf_ui::engineering::EquipmentResultStatus;
use rf_ui::engineering::editor::separator_rating::*;

pub(super) fn status_label(l: StudioShellLocale, status: EquipmentResultStatus) -> &'static str {
    match status {
        EquipmentResultStatus::Missing => tr(l, "尚未校核", "Not rated"),
        EquipmentResultStatus::Current => tr(l, "当前有效结果", "Current result"),
        EquipmentResultStatus::Failed => tr(l, "校核计算失败", "Rating calculation failed"),
        EquipmentResultStatus::Stale => {
            tr(l, "结果已过期 · 请重新校核", "Stale result · rate again")
        }
    }
}
pub(super) fn constraint(l: StudioShellLocale, status: ConstraintStatus) -> &'static str {
    match status {
        ConstraintStatus::WithinCriteria => tr(l, "准则内", "Within criteria"),
        ConstraintStatus::BoundaryReview => tr(l, "边界待复核", "Boundary review required"),
        ConstraintStatus::Exceeded => tr(l, "超限", "Exceeded"),
    }
}
pub(super) fn conclusion(l: StudioShellLocale, status: ConstraintStatus) -> &'static str {
    match status {
        ConstraintStatus::WithinCriteria => {
            tr(l, "所选两项准则内", "Within both selected criteria")
        }
        ConstraintStatus::BoundaryReview => constraint(l, status),
        ConstraintStatus::Exceeded => tr(l, "存在超限", "Limits exceeded"),
    }
}
pub(super) fn issue_label(l: StudioShellLocale, issue: &RatingIssue) -> String {
    let target = match &issue.target {
        RatingIssueTarget::GeometryInput(f) => geometry_field_label(l, *f),
        RatingIssueTarget::Input { field, .. } => rating_field_label(l, *field),
        RatingIssueTarget::AxialSection(f) => field_label(l, EquipmentField::Axial(*f)),
        RatingIssueTarget::GeometryApplicability(field) => match field {
            GeometryApplicabilityField::Orientation => tr(l, "方向", "Orientation"),
            GeometryApplicabilityField::GasDirection => tr(l, "气流方向", "Gas direction"),
            GeometryApplicabilityField::Geometry => tr(l, "截面几何", "Section geometry"),
            GeometryApplicabilityField::Internals => tr(l, "内件配置说明", "Internals description"),
        },
        RatingIssueTarget::CaseApplicability { field, .. } => match field {
            CaseApplicabilityField::Service => tr(l, "介质工况", "Service"),
            CaseApplicabilityField::KBasis => tr(l, "K 的适用范围", "K applicability"),
            CaseApplicabilityField::FlowBasis => tr(l, "流量基准", "Flow basis"),
        },
        RatingIssueTarget::Geometry => tr(l, "给定几何与依据", "Given geometry and evidence"),
        RatingIssueTarget::Case(_) => tr(l, "物系与工况", "Fluid system and case"),
        RatingIssueTarget::Source(_) => tr(l, "输入来源", "Input source"),
        RatingIssueTarget::Task => tr(l, "设备身份", "Equipment identity"),
        RatingIssueTarget::Method => tr(l, "校核方法", "Rating method"),
        RatingIssueTarget::Basis => tr(l, "工程基础", "Engineering basis"),
        RatingIssueTarget::Calculation => tr(l, "数值计算", "Numerical calculation"),
    };
    let reason = match &issue.kind {
        RatingIssueKind::MissingInput => tr(l, "尚未填写或确认", "Missing value or confirmation"),
        RatingIssueKind::InvalidIdentity(_) => tr(
            l,
            "设备位号或身份重复",
            "Duplicate equipment tag or identity",
        ),
        RatingIssueKind::UnsupportedMethod => tr(l, "方法版本不支持", "Unsupported method version"),
        RatingIssueKind::UnsupportedCaseCount { .. } => tr(
            l,
            "本方法只支持一个工况",
            "This method supports exactly one case",
        ),
        RatingIssueKind::InvalidBinding(_) => {
            tr(l, "输入引用或量种无效", "Invalid reference or quantity")
        }
        RatingIssueKind::OutsideMethodScope => {
            tr(l, "超出本方法适用范围", "Outside this method’s scope")
        }
        RatingIssueKind::NonPositiveInput => tr(l, "必须大于零", "Must be positive"),
        RatingIssueKind::UtilizationAboveOne => {
            tr(l, "利用系数不能大于 1", "Utilization must not exceed 1")
        }
        RatingIssueKind::AbsentPhase => tr(
            l,
            "要求气液两相且流量均大于零",
            "Both gas and liquid flows must be positive",
        ),
        RatingIssueKind::InvalidDensityOrder => tr(
            l,
            "要求液相密度大于气相密度",
            "Liquid density must exceed gas density",
        ),
        RatingIssueKind::MissingSourceEvidence | RatingIssueKind::MissingSource(_) => tr(
            l,
            "补齐来源引用、理由或适用范围",
            "Complete the source reference, rationale and applicability",
        ),
        RatingIssueKind::SourceNotAdopted => tr(
            l,
            "须明确采用或审阅依据",
            "Explicitly adopt or review this source",
        ),
        RatingIssueKind::SourceConditionMismatch => tr(
            l,
            "来源限定的物系、相别或操作温压不匹配",
            "Source fluid, phase or operating conditions do not match",
        ),
        RatingIssueKind::NumericRange(_) => {
            tr(l, "数值超出可计算范围", "Values exceed the numerical range")
        }
    };
    format!("{target}：{reason}")
}
