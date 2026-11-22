use super::*;
use rf_solver::flow_plan::SinglePhaseOptions;
use rf_solver::single_phase_recycle::SinglePhaseRecycleOptions;

pub(super) fn applies_to(field: &FlowPlanNumericField, options: &SinglePhaseOptions) -> bool {
    match field {
        FlowPlanNumericField::SinglePhaseMinimumTemperature
        | FlowPlanNumericField::SinglePhaseMaximumTemperature
        | FlowPlanNumericField::SinglePhasePressure => true,
        FlowPlanNumericField::VaporHeatCapacity(id) => {
            options.model.heat_capacities_j_per_mol_k().contains_key(id)
        }
        FlowPlanNumericField::HeatDuty(id) => options.heat_duties_w.contains_key(id),
        _ => false,
    }
}

pub(super) fn configuration<'a>(
    options: &SinglePhaseOptions,
    numbers: impl IntoIterator<Item = (&'a FlowPlanNumericField, &'a FlowPlanNumericInput)>,
) -> RfResult<SinglePhaseOptions> {
    let (mut min_k, mut max_k) = options.model.temperature_range_k().into_inner();
    let mut pressure_pa = options.model.pressure_pa();
    let mut capacities = options.model.heat_capacities_j_per_mol_k().clone();
    let mut duties = options.heat_duties_w.clone();
    let invalid = || RfError::invalid_input("single-vapor numeric draft is incomplete or invalid");
    for (field, input) in numbers {
        let v = input.value_si(field)?;
        match field {
            FlowPlanNumericField::SinglePhaseMinimumTemperature => min_k = v,
            FlowPlanNumericField::SinglePhaseMaximumTemperature => max_k = v,
            FlowPlanNumericField::SinglePhasePressure => pressure_pa = v,
            FlowPlanNumericField::VaporHeatCapacity(id) => {
                *capacities.get_mut(id).ok_or_else(invalid)? = v
            }
            FlowPlanNumericField::HeatDuty(id) => *duties.get_mut(id).ok_or_else(invalid)? = v,
            _ => return Err(invalid()),
        }
    }
    SinglePhaseOptions::new(capacities, min_k..=max_k, pressure_pa, duties)
}

pub(super) fn applies_to_recycle(
    field: &FlowPlanNumericField,
    options: &SinglePhaseRecycleOptions,
) -> bool {
    match field {
        FlowPlanNumericField::IterationBudget => true,
        FlowPlanNumericField::InitialEnthalpyFlow(stream) => {
            options.initial_state.is_some() && stream == &options.tear.stream_id
        }
        FlowPlanNumericField::InitialComponent { stream, component } => {
            stream == &options.tear.stream_id
                && options
                    .initial_state
                    .as_ref()
                    .is_some_and(|s| s.component_flows_mol_s.contains_key(component))
        }
        _ => false,
    }
}

pub(super) fn apply_recycle_numbers(
    options: &mut SinglePhaseRecycleOptions,
    numbers: &BTreeMap<FlowPlanNumericField, FlowPlanNumericInput>,
) -> RfResult<()> {
    if let Some(input) = numbers.get(&FlowPlanNumericField::IterationBudget) {
        options.max_iteration_evaluations =
            iteration_budget(input.value_si(&FlowPlanNumericField::IterationBudget)?)?;
    }
    let Some(state) = &mut options.initial_state else {
        return Ok(());
    };
    let stream = &options.tear.stream_id;
    let field = FlowPlanNumericField::InitialEnthalpyFlow(stream.clone());
    if let Some(input) = numbers.get(&field) {
        state.enthalpy_flow_w = input.value_si(&field)?;
    }
    for (component, flow) in &mut state.component_flows_mol_s {
        let field = FlowPlanNumericField::InitialComponent {
            stream: stream.clone(),
            component: component.clone(),
        };
        if let Some(input) = numbers.get(&field) {
            *flow = input.value_si(&field)?;
        }
    }
    Ok(())
}
