use super::*;
use crate::state::unit_inspector::{
    inlet_pressure_limit, is_valid_unit_parameter_value_for_unit, unit_inspector_draft_fields,
    unit_outlet_pressure_cannot_exceed_inlet,
};
use crate::{SolveSnapshotId, UnitInspectorDraftField};
use rf_types::ComponentId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableField {
    Name,
    OutletTemperature,
    OutletPressure,
    Temperature,
    Pressure,
    MolarFlow,
    MoleFraction(ComponentId),
    PhaseFraction(String),
    PhaseMoleFraction {
        phase: String,
        component: ComponentId,
    },
    PhaseMolarEnthalpy(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableSection {
    Inputs,
    Results,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableId {
    pub document: DocumentId,
    pub object: ObjectId,
    pub section: VariableSection,
    pub field: VariableField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableType {
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueState {
    Valid,
    Invalid,
    Unspecified,
    Missing,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSource {
    DocumentInput,
    StreamTemplate,
    SolveResult {
        snapshot: SolveSnapshotId,
        revision: u64,
        sequence: u64,
    },
    NoResult,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericConstraint {
    pub minimum: f64,
    pub minimum_inclusive: bool,
    pub maximum: Option<f64>,
}
impl NumericConstraint {
    pub fn accepts(self, value: f64) -> bool {
        value.is_finite()
            && (if self.minimum_inclusive {
                value >= self.minimum
            } else {
                value > self.minimum
            })
            && self.maximum.is_none_or(|max| value <= max)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDescriptor {
    pub id: VariableId,
    pub label: String,
    pub value_type: VariableType,
    /// SI unit encodes quantity and basis; composition is mol/mol, not mass fraction.
    pub unit: &'static str,
    pub value: Option<VariableValue>,
    pub state: ValueState,
    pub source: ValueSource,
    pub constraint: Option<NumericConstraint>,
    /// Writes are only available through the existing document command / Inspector.
    pub write_via: Option<ActionKind>,
    pub note: &'static str,
}

impl VariableBrowser<'_> {
    pub fn variables(&self, object: &ObjectId) -> Result<Vec<VariableDescriptor>, BrowseError> {
        let descriptor = self.object(object)?;
        let mut rows = vec![self.row(
            object,
            VariableSection::Inputs,
            VariableField::Name,
            Some(VariableValue::Text(descriptor.name)),
            ValueSource::DocumentInput,
        )];
        rows[0].write_via = Some(match object {
            ObjectId::Unit(_) => ActionKind::RenameUnit,
            ObjectId::Stream(_) => ActionKind::SetStreamSpecification,
        });
        rows[0].note = "名称不能为空白；允许重名，身份保持不变。";
        if let Some(VariableValue::Text(name)) = &rows[0].value
            && name.trim().is_empty()
        {
            rows[0].state = ValueState::Invalid;
        }
        match object {
            ObjectId::Unit(id) => {
                let unit = &self.document.flowsheet.units[id];
                for field in unit_inspector_draft_fields(unit) {
                    let (metric, value) = match field {
                        UnitInspectorDraftField::Name => continue,
                        UnitInspectorDraftField::OutletTemperatureK => (
                            VariableField::OutletTemperature,
                            unit.parameters.outlet_temperature_k,
                        ),
                        UnitInspectorDraftField::OutletPressurePa => (
                            VariableField::OutletPressure,
                            unit.parameters.outlet_pressure_pa,
                        ),
                    };
                    let mut row = self.row(
                        object,
                        VariableSection::Inputs,
                        metric,
                        value.map(VariableValue::Number),
                        ValueSource::DocumentInput,
                    );
                    row.write_via = Some(ActionKind::SetUnitParameter);
                    if field == UnitInspectorDraftField::OutletPressurePa
                        && unit_outlet_pressure_cannot_exceed_inlet(unit)
                    {
                        row.constraint.as_mut().unwrap().maximum =
                            inlet_pressure_limit(&self.document.flowsheet, unit);
                        row.note = "不能高于已连接入口压力；提交同步出口模板。";
                    }
                    if value.is_some_and(|v| {
                        !is_valid_unit_parameter_value_for_unit(
                            &self.document.flowsheet,
                            unit,
                            &field,
                            v,
                        )
                    }) {
                        row.state = ValueState::Invalid;
                    }
                    rows.push(row);
                }
            }
            ObjectId::Stream(id) => {
                let stream = &self.document.flowsheet.streams[id];
                let source = if self.document.flowsheet.units.values().any(|u| {
                    u.kind != "feed"
                        && u.ports.iter().any(|p| {
                            p.direction == rf_types::PortDirection::Outlet
                                && p.stream_id.as_ref() == Some(id)
                        })
                }) {
                    ValueSource::StreamTemplate
                } else {
                    ValueSource::DocumentInput
                };
                for (field, value) in [
                    (VariableField::Temperature, stream.temperature_k),
                    (VariableField::Pressure, stream.pressure_pa),
                    (VariableField::MolarFlow, stream.total_molar_flow_mol_s),
                ] {
                    rows.push(self.row(
                        object,
                        VariableSection::Inputs,
                        field,
                        Some(VariableValue::Number(value)),
                        source.clone(),
                    ));
                }
                for component in self
                    .document
                    .flowsheet
                    .components
                    .keys()
                    .chain(stream.overall_mole_fractions.keys())
                {
                    let field = VariableField::MoleFraction(component.clone());
                    if rows.iter().any(|r| r.id.field == field) {
                        continue;
                    }
                    rows.push(
                        self.row(
                            object,
                            VariableSection::Inputs,
                            field,
                            stream
                                .overall_mole_fractions
                                .get(component)
                                .copied()
                                .map(VariableValue::Number),
                            source.clone(),
                        ),
                    );
                }
                for row in &mut rows[1..] {
                    row.write_via = Some(ActionKind::SetStreamSpecification);
                    row.note =
                        "组成使用摩尔分数；运行要求项目组分齐全且总和为 1。模板值不是求解结果。";
                }
                self.result_rows(object, id, &mut rows);
            }
        }
        Ok(rows)
    }

    fn row(
        &self,
        object: &ObjectId,
        section: VariableSection,
        field: VariableField,
        value: Option<VariableValue>,
        source: ValueSource,
    ) -> VariableDescriptor {
        let (label, unit, constraint) = field.metadata();
        let value_type = if field == VariableField::Name {
            VariableType::Text
        } else {
            VariableType::Number
        };
        let state = match value.as_ref() {
            None => ValueState::Unspecified,
            Some(VariableValue::Number(v))
                if !v.is_finite() || constraint.is_some_and(|c| !c.accepts(*v)) =>
            {
                ValueState::Invalid
            }
            _ => ValueState::Valid,
        };
        VariableDescriptor {
            id: VariableId {
                document: self.document_id().clone(),
                object: object.clone(),
                section,
                field,
            },
            label,
            value_type,
            unit,
            value,
            state,
            source,
            constraint,
            write_via: None,
            note: "只读查询；未提交草稿不作为正式值。",
        }
    }

    fn result_rows(&self, object: &ObjectId, id: &StreamId, rows: &mut Vec<VariableDescriptor>) {
        let snapshot = self.current.or(self.stale);
        let result = snapshot.and_then(|s| s.streams.iter().find(|s| &s.stream_id == id));
        let mut values = vec![
            (VariableField::Temperature, result.map(|s| s.temperature_k)),
            (VariableField::Pressure, result.map(|s| s.pressure_pa)),
            (
                VariableField::MolarFlow,
                result.map(|s| s.total_molar_flow_mol_s),
            ),
        ];
        for component in self.document.flowsheet.components.keys() {
            values.push((
                VariableField::MoleFraction(component.clone()),
                result.and_then(|s| {
                    s.overall_mole_fractions
                        .iter()
                        .find(|(c, _)| c == component.as_str())
                        .map(|(_, v)| *v)
                }),
            ));
        }
        if let Some(stream) = result {
            for phase in &stream.phases {
                values.push((
                    VariableField::PhaseFraction(phase.label.clone()),
                    Some(phase.phase_fraction),
                ));
                values.push((
                    VariableField::PhaseMolarEnthalpy(phase.label.clone()),
                    phase.molar_enthalpy_j_per_mol,
                ));
                for (component, value) in &phase.composition {
                    values.push((
                        VariableField::PhaseMoleFraction {
                            phase: phase.label.clone(),
                            component: ComponentId::new(component),
                        },
                        Some(*value),
                    ));
                }
            }
        }
        for (field, value) in values {
            let source = snapshot
                .map(|s| ValueSource::SolveResult {
                    snapshot: s.id.clone(),
                    revision: s.document_revision,
                    sequence: s.sequence,
                })
                .unwrap_or(ValueSource::NoResult);
            let mut row = self.row(
                object,
                VariableSection::Results,
                field,
                value.map(VariableValue::Number),
                source,
            );
            if self.current.is_none() {
                row.value = None;
                row.state = if value.is_some() {
                    ValueState::Stale
                } else {
                    ValueState::Missing
                };
            } else if value.is_none() {
                row.state = ValueState::Missing;
            }
            rows.push(row);
        }
    }
}

impl VariableField {
    fn metadata(&self) -> (String, &'static str, Option<NumericConstraint>) {
        let positive = Some(NumericConstraint {
            minimum: 0.,
            minimum_inclusive: false,
            maximum: None,
        });
        let fraction = Some(NumericConstraint {
            minimum: 0.,
            minimum_inclusive: true,
            maximum: Some(1.),
        });
        match self {
            Self::Name => ("名称 / Name".into(), "", None),
            Self::OutletTemperature => ("出口温度 / Outlet temperature".into(), "K", positive),
            Self::OutletPressure => ("出口压力 / Outlet pressure".into(), "Pa", positive),
            Self::Temperature => ("温度 / Temperature".into(), "K", positive),
            Self::Pressure => ("压力 / Pressure".into(), "Pa", positive),
            Self::MolarFlow => (
                "摩尔流量 / Molar flow".into(),
                "mol/s",
                Some(NumericConstraint {
                    minimum: 0.,
                    minimum_inclusive: true,
                    maximum: None,
                }),
            ),
            Self::MoleFraction(c) => (format!("摩尔分数 / Mole fraction {c}"), "mol/mol", fraction),
            Self::PhaseFraction(p) => (format!("相分率 / Phase fraction {p}"), "mol/mol", fraction),
            Self::PhaseMoleFraction { phase, component } => (
                format!("相组成 / Phase composition {phase} {component}"),
                "mol/mol",
                fraction,
            ),
            Self::PhaseMolarEnthalpy(p) => (format!("摩尔焓 / Molar enthalpy {p}"), "J/mol", None),
        }
    }
}
