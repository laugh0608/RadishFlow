use super::*;
use rf_model::equipment::*;
use rf_ui::engineering::editor::separator_sizing::*;

pub(super) fn tr<'a>(locale: StudioShellLocale, zh: &'a str, en: &'a str) -> &'a str {
    if locale == StudioShellLocale::ZhCn {
        zh
    } else {
        en
    }
}
pub(super) fn field_label(locale: StudioShellLocale, field: EquipmentField) -> &'static str {
    let (zh, en) = match field {
        EquipmentField::Input(i) => match i {
            SeparatorInput::OperatingTemperature => ("操作温度", "Operating temperature"),
            SeparatorInput::OperatingPressure => ("操作绝对压力", "Operating absolute pressure"),
            SeparatorInput::GasVolumeFlow => ("气相实际体积流量", "Actual gas volume flow"),
            SeparatorInput::LiquidVolumeFlow => ("液相实际体积流量", "Actual liquid volume flow"),
            SeparatorInput::GasDensity => ("气相密度", "Gas density"),
            SeparatorInput::LiquidDensity => ("液相密度", "Liquid density"),
            SeparatorInput::EffectiveK => ("整罐有效 K", "Effective whole-vessel K"),
            SeparatorInput::GasVelocityUtilization => ("气速利用系数", "Gas velocity utilization"),
            SeparatorInput::WorkingVolumeTime => {
                ("工作容积时间 · LLL → NLL", "Working duration · LLL → NLL")
            }
            SeparatorInput::MaximumWorkingLiquidHeight => {
                ("最大工作液层高度", "Maximum working liquid height")
            }
        },
        EquipmentField::Axial(s) => match s {
            SeparatorAxialSection::LowerTangentToLowLiquidLevel => {
                ("下切线 → LLL", "Lower tangent → LLL")
            }
            SeparatorAxialSection::NormalToHighLiquidLevel => {
                ("NLL → HLL 缓冲段", "NLL → HLL buffer")
            }
            SeparatorAxialSection::HighLiquidLevelToInlet => {
                ("HLL → 入口中心", "HLL → inlet center")
            }
            SeparatorAxialSection::InletToInternals => {
                ("入口中心 → 内件下缘", "Inlet center → internals bottom")
            }
            SeparatorAxialSection::InternalsThickness => ("内件厚度", "Internals thickness"),
            SeparatorAxialSection::InternalsToUpperTangent => {
                ("内件上缘 → 上切线", "Internals top → upper tangent")
            }
        },
    };
    tr(locale, zh, en)
}
pub(in super::super) fn target_label(
    locale: StudioShellLocale,
    target: &EquipmentEditorTarget,
    inputs: &rf_ui::engineering::EngineeringInputs,
) -> String {
    match target {
        EquipmentEditorTarget::Task(id)
        | EquipmentEditorTarget::RatingTask(id)
        | EquipmentEditorTarget::RatingGeometry(id) => inputs
            .equipment_task(id)
            .map(|t| task_label(locale, t))
            .unwrap_or_else(|| tr(locale, "设备任务", "Equipment task").into()),
        EquipmentEditorTarget::RatingCase(key) => inputs
            .rating_task(&key.task_id)
            .and_then(|t| t.cases.iter().find(|c| c.id == key.case_id))
            .map(|c| {
                nonempty(
                    &c.name,
                    tr(locale, "未命名校核工况", "Untitled rating case"),
                )
                .to_owned()
            })
            .unwrap_or_else(|| tr(locale, "校核工况", "Rating case").into()),
        EquipmentEditorTarget::Source(id) => inputs
            .basis
            .sources
            .iter()
            .find(|s| s.id == *id)
            .map(|s| nonempty(&s.title, tr(locale, "未命名来源", "Untitled source")).to_owned())
            .unwrap_or_else(|| tr(locale, "来源", "Source").into()),
        EquipmentEditorTarget::Basis(id) => inputs
            .basis
            .basis_entries
            .iter()
            .find(|b| b.id == *id)
            .map(|b| {
                nonempty(
                    &b.name,
                    tr(locale, "未命名基础条目", "Untitled basis entry"),
                )
                .to_owned()
            })
            .unwrap_or_else(|| tr(locale, "新基础条目", "New basis entry").into()),
    }
}
pub(super) fn nonempty<'a>(value: &'a str, empty: &'a str) -> &'a str {
    if value.trim().is_empty() {
        empty
    } else {
        value
    }
}
pub(super) fn task_label(locale: StudioShellLocale, task: &EquipmentTask) -> String {
    let name = nonempty(
        task.name(),
        tr(locale, "立式气液分离器", "Vertical gas–liquid separator"),
    );
    if task.tag().trim().is_empty() {
        name.to_owned()
    } else {
        format!("{} · {name}", task.tag())
    }
}
pub(super) fn status_label(
    locale: StudioShellLocale,
    status: rf_ui::engineering::EquipmentResultStatus,
) -> &'static str {
    use rf_ui::engineering::EquipmentResultStatus::*;
    match status {
        Missing => tr(locale, "尚未计算", "Not calculated"),
        Current => tr(
            locale,
            "当前 · 条件式候选",
            "Current · conditional candidate",
        ),
        Failed => tr(locale, "计算未通过", "Calculation failed"),
        Stale => tr(
            locale,
            "已过期 · 请重新计算",
            "Stale · recalculate explicitly",
        ),
    }
}
pub(super) fn issue_label(locale: StudioShellLocale, issue: &SizingIssue) -> String {
    let target = match &issue.target {
        IssueTarget::Input(f) => field_label(locale, EquipmentField::Input(*f)).to_owned(),
        IssueTarget::AxialSection(f) => field_label(locale, EquipmentField::Axial(*f)).to_owned(),
        IssueTarget::Applicability(field) => match field {
            ApplicabilityField::Orientation => tr(locale, "方向", "Orientation"),
            ApplicabilityField::GasDirection => tr(locale, "气流方向", "Gas direction"),
            ApplicabilityField::Service => tr(locale, "介质工况", "Service"),
            ApplicabilityField::Geometry => tr(locale, "截面几何", "Geometry"),
            ApplicabilityField::KBasis => tr(locale, "K 的适用范围", "K applicability"),
            ApplicabilityField::FlowBasis => tr(locale, "流量基准", "Flow basis"),
            ApplicabilityField::Internals => tr(locale, "内件配置说明", "Internals description"),
        }
        .into(),
        IssueTarget::Source(_) => tr(locale, "输入来源", "Input source").into(),
        IssueTarget::Case => tr(locale, "物系与工况", "Fluid system and case").into(),
        IssueTarget::Method => tr(locale, "计算方法", "Calculation method").into(),
        IssueTarget::Task => tr(locale, "设备身份", "Equipment identity").into(),
        IssueTarget::Basis => tr(locale, "工程基础", "Engineering basis").into(),
        IssueTarget::Calculation => tr(locale, "数值计算", "Numerical calculation").into(),
    };
    let reason = match &issue.kind {
        SizingIssueKind::MissingInput => {
            tr(locale, "尚未填写或确认", "Missing value or confirmation")
        }
        SizingIssueKind::UnsupportedMethod => {
            tr(locale, "方法版本不支持", "Unsupported method version")
        }
        SizingIssueKind::OutsideMethodScope => {
            tr(locale, "超出本方法适用范围", "Outside this method’s scope")
        }
        SizingIssueKind::NonPositiveInput => tr(locale, "必须大于零", "Must be positive"),
        SizingIssueKind::UtilizationAboveOne => tr(
            locale,
            "利用系数不能大于 1",
            "Utilization must not exceed 1",
        ),
        SizingIssueKind::AbsentPhase => tr(
            locale,
            "要求气液两相且流量均大于零",
            "Both gas and liquid flows must be positive",
        ),
        SizingIssueKind::InvalidDensityOrder => tr(
            locale,
            "要求液相密度大于气相密度",
            "Liquid density must exceed gas density",
        ),
        SizingIssueKind::MissingSourceEvidence | SizingIssueKind::MissingSource(_) => tr(
            locale,
            "补齐来源引用、理由或适用范围",
            "Complete the source reference, rationale and applicability",
        ),
        SizingIssueKind::SourceNotAdopted => tr(
            locale,
            "须明确采用或审阅依据",
            "Explicitly adopt or review this source",
        ),
        SizingIssueKind::SourceConditionMismatch => tr(
            locale,
            "来源的物系、相别或操作温压不匹配",
            "Source fluid, phase or operating conditions do not match",
        ),
        SizingIssueKind::NumericRange(_) => tr(
            locale,
            "数值超出可计算范围",
            "Values exceed the numerical range",
        ),
        SizingIssueKind::InvalidBinding(_) => tr(
            locale,
            "输入引用或量种无效",
            "Invalid reference or quantity",
        ),
        SizingIssueKind::InvalidIdentity(_) => tr(
            locale,
            "设备位号或身份重复",
            "Duplicate equipment tag or identity",
        ),
    };
    format!("{target}：{reason}")
}
pub(super) fn edit_error(locale: StudioShellLocale, error: &EquipmentEditError) -> String {
    use EquipmentEditError::*;
    match error {
        Field(field, inner) => format!(
            "{}：{}",
            field_label(locale, *field),
            edit_error(locale, inner)
        ),
        RatingGeometryField(field, inner) => format!(
            "{}：{}",
            match field {
                rf_ui::engineering::editor::RatingGeometryField::Input(i) =>
                    geometry_field_label(locale, *i),
                rf_ui::engineering::editor::RatingGeometryField::Axial(i) =>
                    field_label(locale, EquipmentField::Axial(*i)),
            },
            edit_error(locale, inner)
        ),
        RatingInput(field, inner) => format!(
            "{}：{}",
            rating_field_label(locale, *field),
            edit_error(locale, inner)
        ),
        UnsupportedEditorAction => tr(
            locale,
            "此记录不支持该操作。",
            "This action is not supported for this record.",
        )
        .into(),
        Number(e) => locale.numeric_issue(&rf_ui::NumericFieldIssue::Rejected(
            rf_ui::NumericEditError::Invalid(e.clone()),
        )),
        Conversion(e) => locale.numeric_issue(&rf_ui::NumericFieldIssue::Rejected(
            rf_ui::NumericEditError::Conversion(e.clone()),
        )),
        IncompleteNumber => tr(
            locale,
            "请完成数值输入；未提交内容已保留。",
            "Complete the number; unsubmitted input is preserved.",
        )
        .into(),
        SelectSource => tr(locale, "请选择已记录的来源。", "Select a recorded source.").into(),
        SelectBasis => tr(
            locale,
            "请选择同量种的工程基础条目。",
            "Select a basis entry with the same quantity.",
        )
        .into(),
        OverrideReason => tr(
            locale,
            "请填写局部覆盖理由。",
            "Provide a reason for the local override.",
        )
        .into(),
        AccessDate => tr(
            locale,
            "访问日期须为空或 YYYY-MM-DD。",
            "Access date must be empty or YYYY-MM-DD.",
        )
        .into(),
        ObservationValue => tr(
            locale,
            "原始观测值须为有限数值。",
            "Original observation must be a finite number.",
        )
        .into(),
        PendingDrafts => tr(
            locale,
            "先应用或取消本任务及共享基础 / 来源的未提交输入。",
            "Apply or discard pending inputs for this task and shared basis / sources first.",
        )
        .into(),
        StaleEditor | MissingEditor | MissingTask => tr(
            locale,
            "记录或草稿已变化；输入已保留，请返回重新检查。",
            "The record or draft changed. Your input is preserved; return and review it again.",
        )
        .into(),
        DraftTypeMismatch => tr(
            locale,
            "编辑目标不匹配，请重新选择记录。",
            "The editor target does not match. Select the record again.",
        )
        .into(),
        CounterExhausted => tr(
            locale,
            "编辑计数已达上限，请保存并重新打开工程。",
            "The edit counter is exhausted. Save and reopen the project.",
        )
        .into(),
        Domain(error) => match error {
            rf_ui::engineering::EngineeringError::Referenced { references } => format!(
                "{} ({})",
                tr(
                    locale,
                    "仍被引用，不能删除；请先解除下方引用",
                    "Still referenced; remove these references before deleting"
                ),
                references.len()
            ),
            rf_ui::engineering::EngineeringError::InvalidIdentity(_) => tr(
                locale,
                "设备位号必须在工程内唯一；修改名称不会改变引用。",
                "Equipment tags must be unique in this project; renaming preserves references.",
            )
            .into(),
            rf_ui::engineering::EngineeringError::InvalidBasis(error) => format!(
                "{} {}",
                tr(
                    locale,
                    "来源 / 基础结构无效，请检查日期、成对温压、观察依赖与数值：",
                    "Invalid source / basis data; check dates, paired T/P, observation dependencies and values:"
                ),
                error.path
            ),
            rf_ui::engineering::EngineeringError::InvalidBinding { .. } => tr(
                locale,
                "绑定的量种、来源或基础引用无效。",
                "The binding quantity, source or basis reference is invalid.",
            )
            .into(),
            rf_ui::engineering::EngineeringError::MissingApplicabilitySource { .. } => tr(
                locale,
                "适用性依据引用的来源不存在。",
                "The applicability source no longer exists.",
            )
            .into(),
            _ => tr(
                locale,
                "记录已变化，操作未执行；请返回检查输入。",
                "The record changed; nothing was applied. Return and review the input.",
            )
            .into(),
        },
    }
}

pub(super) fn geometry_field_label(
    locale: StudioShellLocale,
    field: SeparatorGeometryInput,
) -> &'static str {
    match field {
        SeparatorGeometryInput::InternalDiameter => {
            tr(locale, "给定内径", "Given internal diameter")
        }
        SeparatorGeometryInput::WorkingLiquidHeight => {
            tr(locale, "给定工作液层高度", "Given working liquid height")
        }
    }
}
pub(super) fn rating_field_label(
    locale: StudioShellLocale,
    field: SeparatorRatingInput,
) -> &'static str {
    use SeparatorRatingInput::*;
    let sizing = match field {
        OperatingTemperature => SeparatorInput::OperatingTemperature,
        OperatingPressure => SeparatorInput::OperatingPressure,
        GasVolumeFlow => SeparatorInput::GasVolumeFlow,
        LiquidVolumeFlow => SeparatorInput::LiquidVolumeFlow,
        GasDensity => SeparatorInput::GasDensity,
        LiquidDensity => SeparatorInput::LiquidDensity,
        EffectiveK => SeparatorInput::EffectiveK,
        GasVelocityUtilization => SeparatorInput::GasVelocityUtilization,
        RequiredWorkingTime => return tr(locale, "要求工作时间", "Required working time"),
    };
    field_label(locale, EquipmentField::Input(sizing))
}
