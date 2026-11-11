use super::*;
use rf_types::units::DisplayUnitSet;
use rf_ui::project_query::*;
use rf_ui::result_query::{FlowResultView, ResultStatus};
use rf_ui::variable_browser::{
    ValueSource, ValueState, VariableField, VariableSection, VariableValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RowSection {
    Input,
    Template,
    Result,
    Record,
}

impl RowSection {
    pub(super) fn label(self, locale: StudioShellLocale) -> &'static str {
        match self {
            Self::Input => tr(locale, "输入", "Input"),
            Self::Template => tr(locale, "模板", "Template"),
            Self::Result => tr(locale, "结果", "Result"),
            Self::Record => tr(locale, "记录", "Record"),
        }
    }
}

pub(super) struct BrowserEntry {
    pub target: BrowserSelection,
    pub object_index: usize,
    pub label: String,
    pub section: RowSection,
    search: String,
}

/// Only identities and searchable metadata, never a value/snapshot copy.
#[derive(Default)]
pub(super) struct BrowserIndex {
    pub objects: Vec<ProjectObjectDescriptor>,
    pub entries: Vec<BrowserEntry>,
    pub visible: Vec<usize>,
    pub generation: u64,
    input: Option<ProjectInputContext>,
    result: Option<radishflow_studio::FlowResultReview>,
    units: Option<DisplayUnitSet>,
    locale: Option<StudioShellLocale>,
    filter: Option<(String, SectionFilter, Option<ProjectObjectRef>)>,
    #[cfg(test)]
    pub rebuilds: usize,
}

impl BrowserIndex {
    pub(super) fn refresh(&mut self, workspace: &rf_ui::WorkspaceState, locale: StudioShellLocale) {
        let query = ProjectQuery::new(workspace);
        let result = radishflow_studio::FlowResultReview::capture(workspace);
        let units = workspace.project_presentation.display_units();
        if self
            .input
            .as_ref()
            .is_some_and(|c| c.validate(workspace).is_ok())
            && self.result.as_ref() == Some(&result)
            && self.units.as_ref() == Some(units)
            && self.locale == Some(locale)
        {
            return;
        }
        self.objects = query.objects();
        self.entries.clear();
        for (object_index, object) in self.objects.iter().enumerate() {
            let identity = object_identity(&object.reference.object);
            let parent = object
                .parent
                .as_ref()
                .and_then(|p| query.object(p).ok())
                .map(|p| p.label)
                .unwrap_or_default();
            let object_search = format!("{} {} {}", object.label, parent, identity).to_lowercase();
            // Object rows make records and tasks discoverable even without numeric fields.
            self.entries.push(BrowserEntry {
                target: BrowserSelection {
                    object: object.reference.clone(),
                    field: None,
                },
                object_index,
                label: tr(locale, "对象详情", "Object details").into(),
                section: if matches!(object.reference.object, ProjectObjectId::Case(_)) {
                    RowSection::Result
                } else {
                    RowSection::Record
                },
                search: object_search.clone(),
            });
            for field in query
                .fields(&object.reference)
                .expect("enumerated object exists")
            {
                let (section, quantity) = match &field.value {
                    ProjectFieldValue::Flow(v) => (flow_section(v), v.quantity),
                    ProjectFieldValue::Engineering(v) => (RowSection::Input, Some(v.quantity)),
                };
                let label = field_label(&field.field, locale);
                let unit = quantity
                    .map(|q| units.unit_for(q).definition().symbol)
                    .unwrap_or("");
                let canonical = quantity
                    .map(|q| q.definition().canonical_unit.definition().symbol)
                    .unwrap_or("");
                let search = format!(
                    "{object_search} {label} {} {} {unit} {canonical}",
                    field_label(&field.field, StudioShellLocale::En),
                    field_label(&field.field, StudioShellLocale::ZhCn)
                )
                .to_lowercase();
                self.entries.push(BrowserEntry {
                    target: BrowserSelection {
                        object: object.reference.clone(),
                        field: Some(field.field),
                    },
                    object_index,
                    label,
                    section,
                    search,
                });
            }
        }
        self.input = Some(query.input_context());
        self.result = Some(result);
        self.units = Some(units.clone());
        self.locale = Some(locale);
        self.filter = None;
        #[cfg(test)]
        {
            self.rebuilds += 1;
        }
    }

    pub(super) fn filter(
        &mut self,
        query: &str,
        section: SectionFilter,
        scope: Option<&ProjectObjectRef>,
    ) {
        let key = (query.trim().to_lowercase(), section, scope.cloned());
        if self.filter.as_ref() == Some(&key) {
            return;
        }
        self.visible = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let in_scope = scope.is_none_or(|s| {
                    &e.target.object == s || self.objects[e.object_index].parent.as_ref() == Some(s)
                });
                let in_section = match section {
                    SectionFilter::All => true,
                    SectionFilter::Inputs => {
                        matches!(e.section, RowSection::Input | RowSection::Template)
                    }
                    SectionFilter::Results => e.section == RowSection::Result,
                };
                in_scope
                    && in_section
                    && key.0.split_whitespace().all(|word| e.search.contains(word))
            })
            .map(|(i, _)| i)
            .collect();
        self.filter = Some(key);
        self.generation += 1;
    }
}

fn flow_section(v: &rf_ui::variable_browser::VariableDescriptor) -> RowSection {
    if v.id.section == VariableSection::Results {
        RowSection::Result
    } else if v.source == ValueSource::StreamTemplate {
        RowSection::Template
    } else {
        RowSection::Input
    }
}

pub(super) struct BrowserRead {
    pub value: String,
    pub status: String,
    pub canonical: Option<String>,
    pub source: String,
    pub copy: Option<String>,
    pub error: bool,
    pub explanation: String,
}

pub(super) fn read_selection(
    workspace: &rf_ui::WorkspaceState,
    selection: &BrowserSelection,
    locale: StudioShellLocale,
) -> Result<BrowserRead, String> {
    let query = ProjectQuery::new(workspace);
    let field = selection.field.as_ref().ok_or_else(|| {
        tr(locale, "此项为对象记录。", "This item is an object record.").to_string()
    })?;
    let row = query
        .read_field(&selection.object, field)
        .map_err(|e| query_error(&e, locale))?;
    let units = workspace.project_presentation.display_units();
    let mut read = BrowserRead {
        value: String::new(),
        status: String::new(),
        canonical: None,
        source: String::new(),
        copy: None,
        error: false,
        explanation: String::new(),
    };
    match row.value {
        ProjectFieldValue::Flow(v) => {
            read.explanation = v.note.into();
            if let Some(c) = v.constraint {
                read.explanation.push_str(&format!(
                    "\n{}: x {} {}{} {}",
                    tr(locale, "约束（SI）", "Constraint (SI)"),
                    if c.minimum_inclusive { ">=" } else { ">" },
                    c.minimum,
                    c.maximum.map(|m| format!(", x <= {m}")).unwrap_or_default(),
                    v.unit_symbol()
                ));
            }
            let result_status = workspace.flow_result().status();
            let is_result = v.id.section == VariableSection::Results;
            let current = !is_result || result_status == ResultStatus::Current;
            read.status = if is_result && !current {
                result_label(result_status, locale).into()
            } else {
                match v.state {
                    ValueState::Valid if is_result => tr(locale, "当前", "Current"),
                    ValueState::Valid => tr(locale, "已指定", "Specified"),
                    ValueState::Invalid => tr(locale, "无效输入", "Invalid input"),
                    ValueState::Unspecified => tr(locale, "未指定", "Unspecified"),
                    ValueState::Missing => tr(locale, "缺少结果", "Missing value"),
                    ValueState::Stale => tr(locale, "已过期", "Stale"),
                }
                .into()
            };
            read.error = v.state == ValueState::Invalid
                || (is_result && result_status == ResultStatus::Failed);
            let display = if current {
                v.display_value(units)
            } else {
                Ok(v.quantity
                    .map(|q| format!("— {}", units.unit_for(q).definition().symbol))
                    .unwrap_or_else(|| "—".into()))
            };
            match display {
                Ok(value) => {
                    if current && v.state == ValueState::Valid && v.value.is_some() {
                        read.copy = Some(value.clone());
                    }
                    read.value = value;
                }
                Err(e) => {
                    read.value = tr(locale, "无法转换", "Conversion failed").into();
                    read.status = e.to_string();
                    read.error = true;
                }
            }
            if current && let Some(value) = &v.value {
                read.canonical = Some(match value {
                    VariableValue::Number(value) => format!("{value} {}", v.unit_symbol()),
                    VariableValue::Text(value) => value.clone(),
                });
            }
            read.source = match &v.source {
                ValueSource::DocumentInput => {
                    tr(locale, "已提交工程输入", "Committed project input").into()
                }
                ValueSource::StreamTemplate => tr(
                    locale,
                    "流股模板；不是计算结果",
                    "Stream template; not a calculation result",
                )
                .into(),
                ValueSource::SolveResult {
                    snapshot,
                    revision,
                    sequence,
                } => format!(
                    "{} {snapshot} · r{revision} · #{sequence}",
                    tr(locale, "快照", "Snapshot")
                ),
                ValueSource::NoResult => tr(locale, "尚无可读快照", "No readable snapshot").into(),
            };
            if is_result && let FlowResultView::Failed { diagnostic, .. } = workspace.flow_result()
            {
                read.source = diagnostic.primary_message.clone();
            }
        }
        ProjectFieldValue::Engineering(v) => {
            let unit = units.unit_for(v.quantity);
            match v.state {
                EngineeringQuantityState::Value {
                    value_si,
                    source,
                    origin,
                } => {
                    read.canonical = Some(format!(
                        "{value_si} {}",
                        v.quantity.definition().canonical_unit.definition().symbol
                    ));
                    match rf_ui::format_quantity(value_si, v.quantity, unit) {
                        Ok(value) => {
                            read.copy = Some(value.clone());
                            read.value = value;
                            read.status = tr(locale, "已指定", "Specified").into();
                        }
                        Err(e) => {
                            read.value = tr(locale, "无法转换", "Conversion failed").into();
                            read.status = e.to_string();
                            read.error = true;
                        }
                    }
                    use rf_model::engineering::BindingOrigin;
                    let origin = match origin {
                        BindingOrigin::Direct => tr(locale, "直接输入", "Direct input").to_string(),
                        BindingOrigin::Basis(b) => {
                            format!("{}: {}", tr(locale, "工程基础", "Basis"), b.name)
                        }
                        BindingOrigin::Override { basis, reason } => format!(
                            "{}: {} · {reason}",
                            tr(locale, "局部覆盖", "Override"),
                            basis.name
                        ),
                    };
                    read.source = format!(
                        "{origin}\n{} · {}\n{}",
                        source.title, source.id, source.reference
                    );
                }
                EngineeringQuantityState::Unspecified => {
                    read.value = format!("— {}", unit.definition().symbol);
                    read.status = tr(locale, "未指定", "Unspecified").into();
                }
                EngineeringQuantityState::Invalid(error) => {
                    read.value = "—".into();
                    read.status = tr(locale, "绑定无效", "Invalid binding").into();
                    read.source = format!("{error:?}");
                    read.error = true;
                }
            }
        }
    }
    Ok(read)
}

pub(super) fn query_error(error: &ProjectQueryError, locale: StudioShellLocale) -> String {
    match error {
        ProjectQueryError::DifferentDocument | ProjectQueryError::InputContextChanged => tr(
            locale,
            "工程上下文已变化，请重新选择。",
            "Project context changed; select again.",
        ),
        ProjectQueryError::ObjectMissing(_) => tr(
            locale,
            "原对象已删除或不存在；不会跳转到同名对象。",
            "The original object was removed; a same-name object will not replace it.",
        ),
        ProjectQueryError::FieldMissing(_) => tr(
            locale,
            "当前对象不支持该字段，或该结果字段已不可用。",
            "This field is unsupported or no longer available in the result.",
        ),
        ProjectQueryError::NotEquipmentCase => tr(
            locale,
            "此记录不是设备工况。",
            "This record is not an equipment case.",
        ),
    }
    .into()
}

pub(super) fn object_identity(id: &ProjectObjectId) -> String {
    match id {
        ProjectObjectId::Flow(ObjectId::Unit(id)) => format!("unit:{id}"),
        ProjectObjectId::Flow(ObjectId::Stream(id)) => format!("stream:{id}"),
        ProjectObjectId::Equipment(id) => format!("task:{id}"),
        ProjectObjectId::Geometry(id) => format!("geometry:{id}"),
        ProjectObjectId::Case(key) => format!("task:{} / case:{}", key.task_id, key.case_id),
        ProjectObjectId::Basis(id) => format!("basis:{id}"),
        ProjectObjectId::Source(id) => format!("source:{id}"),
    }
}

pub(super) fn result_label(status: ResultStatus, locale: StudioShellLocale) -> &'static str {
    match status {
        ResultStatus::Current => tr(locale, "当前", "Current"),
        ResultStatus::Stale => tr(locale, "已过期", "Stale"),
        ResultStatus::Missing => tr(locale, "缺失", "Missing"),
        ResultStatus::Failed => tr(locale, "失败", "Failed"),
    }
}

pub(super) fn field_label(field: &ProjectField, locale: StudioShellLocale) -> String {
    use rf_ui::engineering::editor::EquipmentField;
    match field {
        ProjectField::Flow { field, .. } => match field {
            VariableField::Name => tr(locale, "名称", "Name").into(),
            VariableField::OutletTemperature => tr(locale, "出口温度", "Outlet temperature").into(),
            VariableField::OutletPressure => tr(locale, "出口压力", "Outlet pressure").into(),
            VariableField::OutletASplitFraction => {
                tr(locale, "出口 A 分流比", "Outlet A split fraction").into()
            }
            VariableField::Temperature => tr(locale, "温度", "Temperature").into(),
            VariableField::Pressure => tr(locale, "压力", "Pressure").into(),
            VariableField::MolarFlow => tr(locale, "摩尔流量", "Molar flow").into(),
            VariableField::MoleFraction(c) => {
                format!("{} · {c}", tr(locale, "摩尔分数", "Mole fraction"))
            }
            VariableField::PhaseFraction(p) => {
                format!("{} · {p}", tr(locale, "相分率", "Phase fraction"))
            }
            VariableField::PhaseMoleFraction { phase, component } => format!(
                "{} · {phase} / {component}",
                tr(locale, "相组成", "Phase composition")
            ),
            VariableField::PhaseMolarEnthalpy(p) => {
                format!("{} · {p}", tr(locale, "摩尔焓", "Molar enthalpy"))
            }
        },
        ProjectField::SizingInput(f) => {
            equipment::field_label(locale, EquipmentField::Input(*f)).into()
        }
        ProjectField::RatingInput(f) => equipment::rating_field_label(locale, *f).into(),
        ProjectField::GeometryInput(f) => equipment::geometry_field_label(locale, *f).into(),
        ProjectField::AxialSection(f) => {
            equipment::field_label(locale, EquipmentField::Axial(*f)).into()
        }
        ProjectField::BasisValue => tr(locale, "基础量值", "Basis value").into(),
    }
}
