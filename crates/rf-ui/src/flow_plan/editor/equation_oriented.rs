use super::*;
use rf_solver::equation_oriented::{EquationOptions, VariableBinding};
pub(super) fn applies_to(field: &FlowPlanNumericField, plan: &FlowPlanConfiguration) -> bool {
    use FlowPlanNumericField::*;
    match (plan, field) {
        (FlowPlanConfiguration::Acyclic, _) => false,
        (_, Temperature | Pressure) => true,
        (FlowPlanConfiguration::Partitioned(_), IterationBudget) => true,
        (FlowPlanConfiguration::Partitioned(o), InitialComponent { stream, component }) => {
            o.tears.iter().any(|t| {
                &t.stream_id == stream
                    && t.initial_component_flows_mol_s
                        .as_ref()
                        .is_some_and(|v| v.contains_key(component))
            })
        }
        (FlowPlanConfiguration::Partitioned(o), ResidualScale { stream, component }) => {
            o.tears.iter().any(|t| {
                &t.stream_id == stream
                    && t.component_scales_mol_s
                        .as_ref()
                        .is_some_and(|v| v.contains_key(component))
            })
        }
        (
            FlowPlanConfiguration::Simultaneous(o),
            CouplingVariableScale(b) | CouplingResidualScale(b),
        ) => o.scales.as_ref().is_some_and(|s| s.contains_key(b)),
        (
            FlowPlanConfiguration::EquationOriented(o),
            ReleasedLower(b) | ReleasedUpper(b) | BisectionBudget(b),
        ) => o.released.as_ref().is_some_and(|r| &r.binding == b),
        (FlowPlanConfiguration::EquationOriented(o), ProductTarget(b)) => {
            o.target.as_ref().is_some_and(|t| &t.source == b)
        }
        (FlowPlanConfiguration::EquationOriented(o), EquationVariableScale(b)) => o
            .scales
            .as_ref()
            .is_some_and(|s| s.variables.contains_key(b)),
        (FlowPlanConfiguration::EquationOriented(o), EquationResidualScale(b)) => o
            .scales
            .as_ref()
            .is_some_and(|s| s.equations.contains_key(b)),
        _ => false,
    }
}
pub(super) fn apply(
    o: &mut EquationOptions,
    field: &FlowPlanNumericField,
    value: f64,
) -> RfResult<()> {
    use FlowPlanNumericField::*;
    let invalid = || RfError::invalid_input("equation plan numeric binding is unavailable");
    match field {
        Temperature => o.boundary.temperature_k = value,
        Pressure => o.boundary.pressure_pa = value,
        ReleasedLower(b) | ReleasedUpper(b) | BisectionBudget(b) => {
            let r = o
                .released
                .as_mut()
                .filter(|r| &r.binding == b)
                .ok_or_else(invalid)?;
            match field {
                ReleasedLower(_) => r.lower = value,
                ReleasedUpper(_) => r.upper = value,
                BisectionBudget(_) => {
                    if !(1.0..=128.0).contains(&value) || value.fract() != 0.0 {
                        return Err(RfError::invalid_input(
                            "bisection budget must be an integer in 1..=128",
                        ));
                    }
                    r.max_iterations = value as usize;
                }
                _ => unreachable!(),
            }
        }
        ProductTarget(b) => {
            o.target
                .as_mut()
                .filter(|t| &t.source == b)
                .ok_or_else(invalid)?
                .value_mol_s = value
        }
        EquationVariableScale(b) => {
            *o.scales
                .as_mut()
                .and_then(|s| s.variables.get_mut(b))
                .ok_or_else(invalid)? = value
        }
        EquationResidualScale(b) => {
            *o.scales
                .as_mut()
                .and_then(|s| s.equations.get_mut(b))
                .ok_or_else(invalid)? = value
        }
        _ => return Err(invalid()),
    }
    Ok(())
}
pub(super) fn variable_quantity(binding: &VariableBinding) -> QuantityKind {
    match binding {
        VariableBinding::Parameter(_) => QuantityKind::DimensionlessRatio,
        VariableBinding::PortComponent { .. } => QuantityKind::MolarFlow,
    }
}
