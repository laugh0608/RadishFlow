mod equation_oriented;
mod mixed;
mod single_phase;
use super::*;
use crate::{InputUnitOrigin, NumericParseOutcome, parse_numeric_input};
use rf_types::{
    ComponentId, StreamId, UnitId,
    units::{MeasurementUnit, QuantityKind, from_canonical},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowPlanNumericField {
    Temperature,
    Pressure,
    SinglePhaseMinimumTemperature,
    SinglePhaseMaximumTemperature,
    SinglePhasePressure,
    VaporHeatCapacity(ComponentId),
    HeatDuty(UnitId),
    InitialEnthalpyFlow(StreamId),
    IterationBudget,
    ReleasedLower(rf_solver::equation_oriented::ParameterBinding),
    ReleasedUpper(rf_solver::equation_oriented::ParameterBinding),
    BisectionBudget(rf_solver::equation_oriented::ParameterBinding),
    ProductTarget(rf_solver::equation_oriented::PortBinding),
    ProductMoleFraction {
        source: rf_solver::equation_oriented::PortBinding,
        component: ComponentId,
    },
    EquationVariableScale(rf_solver::equation_oriented::VariableBinding),
    EquationResidualScale(rf_solver::equation_oriented::EquationBinding),
    MixedVariableScale {
        block: rf_solver::mixed::MixedBlockId,
        binding: rf_solver::equation_oriented::VariableBinding,
    },
    MixedResidualScale {
        block: rf_solver::mixed::MixedBlockId,
        binding: rf_solver::equation_oriented::EquationBinding,
    },
    InitialComponent {
        stream: StreamId,
        component: ComponentId,
    },
    ResidualScale {
        stream: StreamId,
        component: ComponentId,
    },
    CouplingVariableScale(rf_solver::simultaneous::CouplingBinding),
    CouplingResidualScale(rf_solver::simultaneous::CouplingBinding),
}
impl FlowPlanNumericField {
    pub fn quantity(&self) -> Option<QuantityKind> {
        match self {
            Self::Temperature
            | Self::SinglePhaseMinimumTemperature
            | Self::SinglePhaseMaximumTemperature => Some(QuantityKind::AbsoluteTemperature),
            Self::Pressure | Self::SinglePhasePressure => Some(QuantityKind::AbsolutePressure),
            Self::IterationBudget
            | Self::BisectionBudget(_)
            | Self::VaporHeatCapacity(_)
            | Self::HeatDuty(_)
            | Self::InitialEnthalpyFlow(_) => None,
            Self::ReleasedLower(_) | Self::ReleasedUpper(_) => {
                Some(QuantityKind::DimensionlessRatio)
            }
            Self::EquationVariableScale(b) => Some(equation_oriented::variable_quantity(b)),
            Self::ProductMoleFraction { .. } => Some(QuantityKind::MoleFraction),
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
impl FlowPlanNumericInput {
    fn value_si(&self, field: &FlowPlanNumericField) -> RfResult<f64> {
        let invalid = || RfError::invalid_input("flow plan numeric draft is incomplete or invalid");
        if self.raw == self.original_raw {
            return Ok(self.original_si);
        }
        if let Some(quantity) = field.quantity() {
            return match parse_numeric_input(
                &self.raw,
                quantity,
                self.unit,
                InputUnitOrigin::Explicit,
            ) {
                NumericParseOutcome::Value { si, .. } => Ok(si),
                _ => Err(invalid()),
            };
        }
        if matches!(
            field,
            FlowPlanNumericField::VaporHeatCapacity(_)
                | FlowPlanNumericField::HeatDuty(_)
                | FlowPlanNumericField::InitialEnthalpyFlow(_)
        ) {
            return self
                .raw
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(invalid);
        }
        self.raw
            .trim()
            .parse::<usize>()
            .map(|v| v as f64)
            .map_err(|_| RfError::invalid_input("iteration budget requires an integer"))
    }
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
        self.numbers.retain(|field, _| match &candidate {
            FlowPlanConfiguration::SinglePhase(options) => single_phase::applies_to(field, options),
            FlowPlanConfiguration::SinglePhaseRecycle(options) => {
                single_phase::applies_to(field, &options.properties)
                    || single_phase::applies_to_recycle(field, &options.recycle)
            }
            _ => equation_oriented::applies_to(field, &candidate),
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
                    return Err(RfError::invalid_input("field uses a fixed input unit"));
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
    pub fn has_modified_numbers(&self) -> bool {
        self.numbers.values().any(|n| n.raw != n.original_raw)
    }
    pub fn is_pending(&self, current: &FlowPlanConfiguration) -> bool {
        self.candidate != *current || self.has_modified_numbers()
    }
    pub fn configuration(&self) -> RfResult<FlowPlanConfiguration> {
        if let FlowPlanConfiguration::SinglePhase(options) = &self.candidate {
            return single_phase::configuration(options, &self.numbers)
                .map(FlowPlanConfiguration::SinglePhase);
        }
        if let FlowPlanConfiguration::SinglePhaseRecycle(options) = &self.candidate {
            let mut options = (**options).clone();
            options.properties = single_phase::configuration(
                &options.properties,
                self.numbers
                    .iter()
                    .filter(|(field, _)| single_phase::applies_to(field, &options.properties)),
            )?;
            single_phase::apply_recycle_numbers(&mut options.recycle, &self.numbers)?;
            let configuration = FlowPlanConfiguration::SinglePhaseRecycle(Box::new(options));
            configuration.validate_configuration()?;
            return Ok(configuration);
        }
        let mut candidate = self.candidate.clone();
        for (field, input) in &self.numbers {
            let value = input.value_si(field)?;
            if let FlowPlanConfiguration::Mixed(options) = &mut candidate {
                mixed::apply(options, field, value)?;
                continue;
            }
            if let FlowPlanConfiguration::EquationOriented(options) = &mut candidate {
                equation_oriented::apply(options, field, value)?;
                continue;
            }
            if let FlowPlanConfiguration::Simultaneous(options) = &mut candidate {
                match field {
                    FlowPlanNumericField::Temperature => options.boundary.temperature_k = value,
                    FlowPlanNumericField::Pressure => options.boundary.pressure_pa = value,
                    FlowPlanNumericField::CouplingVariableScale(binding)
                    | FlowPlanNumericField::CouplingResidualScale(binding) => {
                        let scale = options
                            .scales
                            .as_mut()
                            .and_then(|s| s.get_mut(binding))
                            .ok_or_else(|| {
                                RfError::invalid_input("explicit coupling binding is unavailable")
                            })?;
                        match field {
                            FlowPlanNumericField::CouplingVariableScale(_) => {
                                scale.variable_mol_s = value
                            }
                            _ => scale.residual_mol_s = value,
                        }
                    }
                    _ => {
                        return Err(RfError::invalid_input(
                            "recycle fields are unavailable for a simultaneous plan",
                        ));
                    }
                }
                continue;
            }
            let FlowPlanConfiguration::Partitioned(options) = &mut candidate else {
                return Err(RfError::invalid_input(
                    "numeric recycle fields are unavailable for an acyclic plan",
                ));
            };
            match field {
                FlowPlanNumericField::MixedVariableScale { .. }
                | FlowPlanNumericField::MixedResidualScale { .. }
                | FlowPlanNumericField::CouplingVariableScale(_)
                | FlowPlanNumericField::CouplingResidualScale(_)
                | FlowPlanNumericField::ReleasedLower(_)
                | FlowPlanNumericField::ReleasedUpper(_)
                | FlowPlanNumericField::BisectionBudget(_)
                | FlowPlanNumericField::ProductTarget(_)
                | FlowPlanNumericField::ProductMoleFraction { .. }
                | FlowPlanNumericField::EquationVariableScale(_)
                | FlowPlanNumericField::EquationResidualScale(_)
                | FlowPlanNumericField::SinglePhaseMinimumTemperature
                | FlowPlanNumericField::SinglePhaseMaximumTemperature
                | FlowPlanNumericField::SinglePhasePressure
                | FlowPlanNumericField::VaporHeatCapacity(_)
                | FlowPlanNumericField::HeatDuty(_)
                | FlowPlanNumericField::InitialEnthalpyFlow(_) => {
                    return Err(RfError::invalid_input(
                        "coupling scales are unavailable for a recycle plan",
                    ));
                }
                FlowPlanNumericField::Temperature => options.boundary.temperature_k = value,
                FlowPlanNumericField::Pressure => options.boundary.pressure_pa = value,
                FlowPlanNumericField::IterationBudget => {
                    options.max_iteration_evaluations = iteration_budget(value)?;
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
        validate_workspace_plan(&candidate)?;
        Ok(candidate)
    }
}

fn iteration_budget(value: f64) -> RfResult<usize> {
    if !(1.0..=500.0).contains(&value) || value.fract() != 0.0 {
        return Err(RfError::invalid_input(
            "iteration budget must be in 1..=500",
        ));
    }
    Ok(value as usize)
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
