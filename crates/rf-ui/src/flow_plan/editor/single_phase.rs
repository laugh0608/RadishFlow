use super::*;
use rf_solver::flow_plan::SinglePhaseOptions;

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

pub(super) fn configuration(
    options: &SinglePhaseOptions,
    numbers: &BTreeMap<FlowPlanNumericField, FlowPlanNumericInput>,
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
