use super::*;
use rf_flowsheet::pressure_context::DocumentPressureSource;
use rf_types::units::{DisplayUnitSet, QuantityKind};

pub(super) fn tr(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    if locale == StudioShellLocale::ZhCn {
        zh
    } else {
        en
    }
}

pub(super) fn phase_label(locale: StudioShellLocale, session: &ModelingCheckSession) -> String {
    let last = session.report().map(|report| report.problems.len());
    match session.phase() {
        ModelingCheckPhase::Current => {
            let partial = session.current_report().is_some_and(|report| {
                report.coverage.iter().any(|scope| {
                    !matches!(
                        scope.coverage,
                        SpecificationCoverage::Evaluated | SpecificationCoverage::NotAnalyzed
                    )
                })
            });
            format!(
                "{} · {}{}",
                tr(locale, "当前问题", "Current problems"),
                session.current_count().expect("current phase has a count"),
                if partial {
                    tr(locale, " · 部分范围未检查", " · Coverage incomplete")
                } else {
                    ""
                }
            )
        }
        phase => {
            let label = match phase {
                ModelingCheckPhase::NotChecked => tr(locale, "尚未检查", "Not checked"),
                ModelingCheckPhase::Stale => tr(locale, "待重检", "Needs recheck"),
                ModelingCheckPhase::Checking => tr(locale, "检查中", "Checking"),
                ModelingCheckPhase::Failed(_) => tr(locale, "检查未完成", "Check incomplete"),
                ModelingCheckPhase::Current => unreachable!(),
            };
            match last {
                Some(last) => format!("{label} · {} {last}", tr(locale, "上次", "Previous")),
                None => label.into(),
            }
        }
    }
}

pub(super) fn entry_currency(
    locale: StudioShellLocale,
    entry: &SpecificationEntry,
    phase: &ModelingCheckPhase,
) -> String {
    let label = match entry.state {
        SpecificationEntryState::TargetRemoved => tr(
            locale,
            "目标已移除，非检查通过",
            "Target removed, not a passed check",
        ),
        SpecificationEntryState::Resolved => tr(
            locale,
            "同范围重检已解决",
            "Resolved by recheck in the same scope",
        ),
        SpecificationEntryState::Unconfirmed => tr(
            locale,
            "该范围未检查，问题尚未确认",
            "Scope not checked; problem unconfirmed",
        ),
        SpecificationEntryState::Current if *phase == ModelingCheckPhase::Current => {
            tr(locale, "当前依据", "Current evidence")
        }
        SpecificationEntryState::Current => tr(
            locale,
            "上次依据，非当前结论",
            "Previous evidence, not a current conclusion",
        ),
    };
    format!("{label} · r{}", entry.observed_revision)
}

fn object_label(object: &ObjectId, document: &rf_ui::FlowsheetDocument) -> String {
    let (id, name) = match object {
        ObjectId::Unit(id) => (
            id.as_str(),
            document
                .flowsheet
                .units
                .get(id)
                .map(|unit| unit.name.as_str()),
        ),
        ObjectId::Stream(id) => (
            id.as_str(),
            document
                .flowsheet
                .streams
                .get(id)
                .map(|stream| stream.name.as_str()),
        ),
    };
    name.filter(|name| *name != id)
        .map_or_else(|| id.to_string(), |name| format!("{name} ({id})"))
}

fn field_label(locale: StudioShellLocale, field: &VariableField) -> String {
    match field {
        VariableField::Name => tr(locale, "名称", "Name").into(),
        VariableField::OutletTemperature => tr(locale, "出口温度", "Outlet temperature").into(),
        VariableField::OutletPressure => tr(locale, "出口压力", "Outlet pressure").into(),
        VariableField::Temperature => tr(locale, "温度", "Temperature").into(),
        VariableField::Pressure => tr(locale, "压力", "Pressure").into(),
        VariableField::MolarFlow => tr(locale, "摩尔流量", "Molar flow").into(),
        VariableField::MoleFraction(id) => {
            format!("{} {id}", tr(locale, "摩尔分数", "Mole fraction"))
        }
        _ => unreachable!("specification targets only refer to inputs"),
    }
}

pub(super) fn target_label(
    locale: StudioShellLocale,
    target: &SpecificationTarget,
    document: &rf_ui::FlowsheetDocument,
) -> String {
    match target {
        SpecificationTarget::Palette => tr(locale, "组件库", "Module palette").into(),
        SpecificationTarget::ProjectComponents => {
            tr(locale, "工程组分", "Project components").into()
        }
        SpecificationTarget::Unit(id) => object_label(&ObjectId::Unit(id.clone()), document),
        SpecificationTarget::Stream(id) => object_label(&ObjectId::Stream(id.clone()), document),
        SpecificationTarget::Composition(id) => format!(
            "{} · {}",
            object_label(&ObjectId::Stream(id.clone()), document),
            tr(locale, "组成", "Composition")
        ),
        SpecificationTarget::Field { object, field } => format!(
            "{} · {}",
            object_label(object, document),
            field_label(locale, field)
        ),
        SpecificationTarget::Port(port) => format!(
            "{} · {}",
            object_label(&ObjectId::Unit(port.unit_id.clone()), document),
            port.port_name
        ),
    }
}

pub(super) fn problem_title(
    locale: StudioShellLocale,
    problem: &SpecificationProblem,
    document: &rf_ui::FlowsheetDocument,
) -> String {
    let reason = match problem {
        SpecificationProblem::EmptyFlowsheet => tr(locale, "放置首个单元", "Place the first unit"),
        SpecificationProblem::Topology(_) => {
            tr(locale, "连接或支持范围错误", "Connection or support error")
        }
        SpecificationProblem::MissingProjectComponents => {
            tr(locale, "尚未选择组分", "No components selected")
        }
        SpecificationProblem::ComponentOutsideProject { .. } => {
            tr(locale, "引用了工程外组分", "Component outside project")
        }
        SpecificationProblem::InvalidFeedInput { .. } => {
            tr(locale, "源输入非法", "Invalid feed input")
        }
        SpecificationProblem::MissingFeedComposition { .. } => {
            tr(locale, "缺少源组成", "Missing feed composition")
        }
        SpecificationProblem::InvalidMoleFraction { .. }
        | SpecificationProblem::InvalidCompositionTotal { .. } => {
            tr(locale, "组成数值非法", "Invalid composition value")
        }
        SpecificationProblem::UnnormalizedComposition { .. } => {
            tr(locale, "组成未归一", "Composition not normalized")
        }
        SpecificationProblem::MissingParameter { .. } => {
            tr(locale, "缺少显式输入", "Missing explicit input")
        }
        SpecificationProblem::InvalidParameter { .. } => {
            tr(locale, "参数值非法", "Invalid parameter value")
        }
        SpecificationProblem::OutletPressureAboveInlet { .. } => {
            tr(locale, "出口压力高于入口", "Outlet pressure exceeds inlet")
        }
    };
    let targets = problem
        .targets()
        .iter()
        .map(|target| target_label(locale, target, document))
        .collect::<Vec<_>>()
        .join(" / ");
    format!("{reason} · {targets}")
}

fn quantity(value: f64, kind: QuantityKind, units: &DisplayUnitSet) -> String {
    rf_ui::format_quantity(value, kind, units.unit_for(kind)).unwrap_or_else(|_| {
        format!(
            "{value} {}",
            kind.definition().canonical_unit.definition().symbol
        )
    })
}

pub(super) fn problem_evidence(
    locale: StudioShellLocale,
    problem: &SpecificationProblem,
    units: &DisplayUnitSet,
) -> String {
    let zh = locale == StudioShellLocale::ZhCn;
    match problem {
        SpecificationProblem::EmptyFlowsheet => tr(locale, "当前流程没有单元；请从组件库放置。", "The flowsheet has no units; place one from the palette.").into(),
        SpecificationProblem::Topology(error) => topology_evidence(locale, error),
        SpecificationProblem::MissingProjectComponents => tr(locale, "在物性页选择工程组分；检查不会替你添加组分。", "Select project components on the Property page; checking does not add them.").into(),
        SpecificationProblem::ComponentOutsideProject { component_id, .. } => if zh { format!("组成引用 {component_id}，但工程未选择该组分。请检查组成或工程组分。") } else { format!("Composition references {component_id}, which is not selected in the project. Review composition or project components.") },
        SpecificationProblem::MissingFeedComposition { .. } => tr(locale, "源流股组成为空；需要明确各组分摩尔分数。", "Feed composition is empty; specify component mole fractions.").into(),
        SpecificationProblem::MissingParameter { .. } => tr(locale, "需要显式指定此参数；界面默认显示和未提交草稿不能补齐规格。", "Specify this parameter explicitly; displayed defaults and uncommitted drafts do not complete the specification.").into(),
        SpecificationProblem::InvalidParameter { parameter, value, .. } => {
            let kind = match parameter { rf_flowsheet::specifications::UnitParameter::OutletTemperature => QuantityKind::AbsoluteTemperature, rf_flowsheet::specifications::UnitParameter::OutletPressure => QuantityKind::AbsolutePressure };
            let value = quantity(*value, kind, units);
            if zh { format!("已提交 {value}；要求有限且严格大于零（SI）。") } else { format!("Committed {value}; must be finite and strictly positive in SI.") }
        }
        SpecificationProblem::InvalidFeedInput { field, value, .. } => {
            let kind = match field { rf_flowsheet::specifications::FeedInputField::Temperature => QuantityKind::AbsoluteTemperature, rf_flowsheet::specifications::FeedInputField::Pressure => QuantityKind::AbsolutePressure, rf_flowsheet::specifications::FeedInputField::MolarFlow => QuantityKind::MolarFlow };
            let value = quantity(*value, kind, units);
            if zh { format!("有效源输入 {value}；要求有限且严格大于零（SI）。") } else { format!("Effective feed input {value}; must be finite and strictly positive in SI.") }
        }
        SpecificationProblem::InvalidMoleFraction { value, .. } => if zh { format!("已提交摩尔分数 {value}；要求有限且在 0—1 内。") } else { format!("Committed mole fraction {value}; must be finite and within 0–1.") },
        SpecificationProblem::InvalidCompositionTotal { sum, .. } => if zh { format!("组成总和 {sum} 非法；检查不会自动修复。") } else { format!("Composition total {sum} is invalid; checks do not repair it.") },
        SpecificationProblem::UnnormalizedComposition { sum, .. } => if zh { format!("组成总和 {sum} 偏离 1；容差 1e-9。需要用户显式处理。") } else { format!("Composition total {sum} differs from 1; tolerance 1e-9. Explicit user action is required.") },
        SpecificationProblem::OutletPressureAboveInlet { pressure_pa, context, .. } => {
            let outlet = quantity(*pressure_pa, QuantityKind::AbsolutePressure, units);
            let limit = context.limit().map(|limit| quantity(limit.pressure_pa, QuantityKind::AbsolutePressure, units)).unwrap_or_else(|| "—".into());
            let mut lines = vec![if zh { format!("已提交出口 {outlet} > 最低入口 {limit}。") } else { format!("Committed outlet {outlet} > lowest inlet {limit}.") }];
            for input in &context.inputs {
                let source = match &input.source { DocumentPressureSource::FeedStream { unit_id, stream_id } => format!("{unit_id} / {stream_id}"), DocumentPressureSource::UnitParameter { unit_id } => format!("{unit_id} · {}", tr(locale, "显式出口压力", "explicit outlet pressure")) };
                let limiting = context.limit().is_some_and(|limit| limit.limiting_stream_id == input.input.stream_id);
                lines.push(format!("{} ← {source}: {}{}", input.input.port, quantity(input.input.pressure_pa, QuantityKind::AbsolutePressure, units), if limiting { tr(locale, "（限制入口）", " (limiting inlet)") } else { "" }));
            }
            lines.join("\n")
        }
    }
}

fn topology_evidence(locale: StudioShellLocale, error: &rf_types::RfError) -> String {
    let code = error
        .context()
        .diagnostic_code()
        .unwrap_or(error.code().as_str());
    let reason = match code {
        "solver.connection_validation.unsupported_unit_kind" => "当前顺序求解不支持此单元类型。",
        "solver.connection_validation.invalid_port_signature" => "单元端口与其类型要求不一致。",
        "solver.connection_validation.unbound_inlet_port" => "入口端口尚未连接流股。",
        "solver.connection_validation.unbound_outlet_port" => "出口端口尚未连接流股。",
        "solver.connection_validation.missing_stream_reference" => "连接引用的流股不存在。",
        "solver.connection_validation.duplicate_upstream_source" => "同一流股存在多个上游来源。",
        "solver.connection_validation.duplicate_downstream_sink" => "同一流股连接了多个下游入口。",
        "solver.connection_validation.orphan_stream" => "流股没有连接到任何单元。",
        "solver.connection_validation.missing_upstream_source" => "流股缺少上游来源。",
        "solver.topological_ordering.self_loop_cycle"
        | "solver.topological_ordering.two_unit_cycle"
        | "solver.topological_ordering" => "流程存在环路，当前顺序求解无法确定执行顺序。",
        _ => "连接或单元支持范围未通过检查，详情见技术依据。",
    };
    if locale == StudioShellLocale::ZhCn {
        format!("{reason}\n{code}")
    } else {
        format!("{code}: {}", error.message())
    }
}

pub(super) fn coverage_label(
    locale: StudioShellLocale,
    item: &SpecificationCheckCoverage,
) -> String {
    let reason = match &item.coverage {
        SpecificationCoverage::NotCheckedTopology => tr(
            locale,
            "未检查：拓扑前置条件未满足",
            "Not checked: topology prerequisite unmet",
        ),
        SpecificationCoverage::NotCheckedMissingOrInvalidParameter => tr(
            locale,
            "未检查：出口参数缺失或非法",
            "Not checked: missing or invalid outlet parameter",
        ),
        SpecificationCoverage::NotCheckedInputPressure(_) => tr(
            locale,
            "未检查：缺少有来源的有效入口压力",
            "Not checked: no valid sourced inlet pressure",
        ),
        SpecificationCoverage::AwaitingExecution(_) => tr(
            locale,
            "待运行检查：需要实际执行上下文",
            "Awaiting execution context",
        ),
        SpecificationCoverage::UnsupportedUnitKind(_) => {
            tr(locale, "不支持该单元类型", "Unsupported unit kind")
        }
        SpecificationCoverage::NotAnalyzed => tr(locale, "未分析", "Not analyzed"),
        SpecificationCoverage::Evaluated => tr(locale, "已检查", "Evaluated"),
    };
    let scope = match &item.scope {
        SpecificationCheckScope::Topology => tr(locale, "拓扑", "Topology").into(),
        SpecificationCheckScope::ProjectComponents => {
            tr(locale, "工程组分", "Project components").into()
        }
        SpecificationCheckScope::FeedInputs => tr(locale, "源输入", "Feed inputs").into(),
        SpecificationCheckScope::StreamCompositions => {
            tr(locale, "流股组成", "Stream compositions").into()
        }
        SpecificationCheckScope::UnitParameters(id) => {
            format!("{id} · {}", tr(locale, "单元参数", "Unit parameters"))
        }
        SpecificationCheckScope::InletPressure(id) => {
            format!("{id} · {}", tr(locale, "入口压力", "Inlet pressure"))
        }
        SpecificationCheckScope::StructuralDegreesOfFreedom => "DOF".into(),
        SpecificationCheckScope::NumericalSolvability => {
            tr(locale, "数值可解性", "Numerical solvability").into()
        }
    };
    format!("— {scope}: {reason}")
}

pub(super) fn draft_target(key: &str) -> Option<SpecificationTarget> {
    if let Some((unit, field)) = rf_ui::unit_inspector_draft_key_parts(key) {
        let field = match field {
            rf_ui::UnitInspectorDraftField::Name => VariableField::Name,
            rf_ui::UnitInspectorDraftField::OutletTemperatureK => VariableField::OutletTemperature,
            rf_ui::UnitInspectorDraftField::OutletPressurePa => VariableField::OutletPressure,
        };
        return Some(SpecificationTarget::Field {
            object: ObjectId::Unit(unit),
            field,
        });
    }
    rf_ui::stream_inspector_draft_key_parts(key).map(|(stream, field)| SpecificationTarget::Field {
        object: ObjectId::Stream(stream),
        field: match field {
            rf_ui::StreamInspectorDraftField::Name => VariableField::Name,
            rf_ui::StreamInspectorDraftField::TemperatureK => VariableField::Temperature,
            rf_ui::StreamInspectorDraftField::PressurePa => VariableField::Pressure,
            rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS => VariableField::MolarFlow,
            rf_ui::StreamInspectorDraftField::OverallMoleFraction(id) => {
                VariableField::MoleFraction(id)
            }
        },
    })
}
