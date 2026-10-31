use super::*;
use crate::{InputUnitOrigin, NumericParseOutcome, parse_numeric_input};
use rf_types::{
    ComponentId,
    units::{MeasurementUnit, QuantityKind, from_canonical},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowPlanNumericField {
    Temperature,
    Pressure,
    IterationBudget,
    InitialComponent {
        stream: StreamId,
        component: ComponentId,
    },
    ResidualScale {
        stream: StreamId,
        component: ComponentId,
    },
}
impl FlowPlanNumericField {
    pub fn quantity(&self) -> Option<QuantityKind> {
        match self {
            Self::Temperature => Some(QuantityKind::AbsoluteTemperature),
            Self::Pressure => Some(QuantityKind::AbsolutePressure),
            Self::IterationBudget => None,
            _ => Some(QuantityKind::MolarFlow),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlowPlanNumericInput {
    pub raw: String,
    pub unit: MeasurementUnit,
    original_si: f64,
    original_raw: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlowPlanDraft {
    context: FlowPlanEditContext,
    candidate: FlowPlanConfiguration,
    numbers: BTreeMap<FlowPlanNumericField, FlowPlanNumericInput>,
}
impl FlowPlanDraft {
    pub fn new(workspace: &WorkspaceState, candidate: FlowPlanConfiguration) -> Self {
        Self {
            context: FlowPlanEditContext::capture(workspace),
            candidate,
            numbers: BTreeMap::new(),
        }
    }
    pub fn context(&self) -> &FlowPlanEditContext {
        &self.context
    }
    pub fn replace_candidate(&mut self, candidate: FlowPlanConfiguration) {
        self.numbers.retain(|field, _| match (&candidate, field) {
            (FlowPlanConfiguration::Acyclic, _) => false,
            (
                FlowPlanConfiguration::Partitioned(o),
                FlowPlanNumericField::InitialComponent { stream, component },
            ) => o.tears.iter().any(|t| {
                &t.stream_id == stream
                    && t.initial_component_flows_mol_s
                        .as_ref()
                        .is_some_and(|v| v.contains_key(component))
            }),
            (
                FlowPlanConfiguration::Partitioned(o),
                FlowPlanNumericField::ResidualScale { stream, component },
            ) => o.tears.iter().any(|t| {
                &t.stream_id == stream
                    && t.component_scales_mol_s
                        .as_ref()
                        .is_some_and(|v| v.contains_key(component))
            }),
            _ => true,
        });
        self.candidate = candidate;
    }
    pub fn candidate(&self) -> &FlowPlanConfiguration {
        &self.candidate
    }
    pub fn numbers(&self) -> &BTreeMap<FlowPlanNumericField, FlowPlanNumericInput> {
        &self.numbers
    }
    /// Starts a field with original SI precision and an explicit captured input unit.
    pub fn edit_number(
        &mut self,
        field: FlowPlanNumericField,
        value_si: f64,
        unit: MeasurementUnit,
    ) -> RfResult<&mut FlowPlanNumericInput> {
        if !value_si.is_finite() {
            return Err(RfError::invalid_input("flow plan number must be finite"));
        }
        if !self.numbers.contains_key(&field) {
            let displayed = if let Some(quantity) = field.quantity() {
                from_canonical(value_si, quantity, unit)
                    .map_err(|e| RfError::invalid_input(e.to_string()))?
            } else {
                if unit != MeasurementUnit::One {
                    return Err(RfError::invalid_input("iteration budget has no input unit"));
                }
                value_si
            };
            let raw = displayed.to_string();
            self.numbers.insert(
                field.clone(),
                FlowPlanNumericInput {
                    raw: raw.clone(),
                    original_raw: raw,
                    unit,
                    original_si: value_si,
                },
            );
        }
        Ok(self.numbers.get_mut(&field).expect("inserted field"))
    }
    pub fn change_input_unit(
        &mut self,
        field: &FlowPlanNumericField,
        unit: MeasurementUnit,
    ) -> RfResult<()> {
        let quantity = field
            .quantity()
            .ok_or_else(|| RfError::invalid_input("field has no physical unit"))?;
        let number = self
            .numbers
            .get_mut(field)
            .ok_or_else(|| RfError::invalid_input("numeric draft not started"))?;
        let value = if number.raw == number.original_raw {
            number.original_si
        } else {
            match parse_numeric_input(
                &number.raw,
                quantity,
                number.unit,
                InputUnitOrigin::Explicit,
            ) {
                NumericParseOutcome::Value { si, .. } => si,
                _ => {
                    return Err(RfError::invalid_input(
                        "finish the number before changing its input unit",
                    ));
                }
            }
        };
        let raw = from_canonical(value, quantity, unit)
            .map_err(|e| RfError::invalid_input(e.to_string()))?
            .to_string();
        let original = from_canonical(number.original_si, quantity, unit)
            .map_err(|e| RfError::invalid_input(e.to_string()))?
            .to_string();
        number.unit = unit;
        number.raw = raw;
        number.original_raw = original;
        Ok(())
    }
    pub fn is_pending(&self, current: &FlowPlanConfiguration) -> bool {
        self.candidate != *current || self.numbers.values().any(|n| n.raw != n.original_raw)
    }
    pub fn configuration(&self) -> RfResult<FlowPlanConfiguration> {
        let mut candidate = self.candidate.clone();
        for (field, input) in &self.numbers {
            let value = if input.raw == input.original_raw {
                input.original_si
            } else if let Some(quantity) = field.quantity() {
                match parse_numeric_input(
                    &input.raw,
                    quantity,
                    input.unit,
                    InputUnitOrigin::Explicit,
                ) {
                    NumericParseOutcome::Value { si, .. } => si,
                    _ => {
                        return Err(RfError::invalid_input(
                            "flow plan numeric draft is incomplete or invalid",
                        ));
                    }
                }
            } else {
                input
                    .raw
                    .trim()
                    .parse::<usize>()
                    .map(|v| v as f64)
                    .map_err(|_| RfError::invalid_input("iteration budget requires an integer"))?
            };
            let FlowPlanConfiguration::Partitioned(options) = &mut candidate else {
                return Err(RfError::invalid_input(
                    "numeric recycle fields are unavailable for an acyclic plan",
                ));
            };
            match field {
                FlowPlanNumericField::Temperature => options.boundary.temperature_k = value,
                FlowPlanNumericField::Pressure => options.boundary.pressure_pa = value,
                FlowPlanNumericField::IterationBudget => {
                    if !(1.0..=500.0).contains(&value) || value.fract() != 0.0 {
                        return Err(RfError::invalid_input(
                            "iteration budget must be in 1..=500",
                        ));
                    }
                    options.max_iteration_evaluations = value as usize;
                }
                FlowPlanNumericField::InitialComponent { stream, component }
                | FlowPlanNumericField::ResidualScale { stream, component } => {
                    let tear = options
                        .tears
                        .iter_mut()
                        .find(|t| &t.stream_id == stream)
                        .ok_or_else(|| RfError::missing_entity("tear", stream))?;
                    let vector = match field {
                        FlowPlanNumericField::InitialComponent { .. } => {
                            &mut tear.initial_component_flows_mol_s
                        }
                        _ => &mut tear.component_scales_mol_s,
                    };
                    let values = vector.as_mut().ok_or_else(|| {
                        RfError::invalid_input(
                            "select an explicit component vector before editing its values",
                        )
                    })?;
                    let number = values.get_mut(component).ok_or_else(|| {
                        RfError::missing_entity("component vector entry", component)
                    })?;
                    *number = value;
                }
            }
        }
        candidate.validate_configuration()?;
        Ok(candidate)
    }
}
impl WorkspaceState {
    pub fn apply_flow_plan_draft(&mut self) -> RfResult<bool> {
        let draft = self
            .flow_plan
            .draft
            .as_ref()
            .ok_or_else(|| RfError::invalid_input("no flow plan draft"))?;
        let context = draft.context.clone();
        let configuration = draft.configuration()?;
        let changed = self.apply_flow_plan(&context, FlowPlanCommand::Apply(configuration))?;
        self.flow_plan.discard_draft();
        Ok(changed)
    }
}
