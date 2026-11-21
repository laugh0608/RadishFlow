use super::{SinglePhaseRecycleOptions, recycle_error};
use crate::{
    dependency_graph::topological_order, fixed_tp_material::validate_vector,
    partitioned::MAX_ITERATION_EVALUATIONS,
};
use rf_flowsheet::dependencies::material_dependencies;
use rf_model::Flowsheet;
use rf_types::{ComponentId, RfResult, StreamId, UnitId};
use rf_unitops::{
    BuiltinUnitKind, SPLITTER_OUTLET_A_PORT, SPLITTER_OUTLET_B_PORT,
    specifications::{UnitParameter, input_specification},
};

pub(super) struct Plan {
    pub(super) upstream: Vec<UnitId>,
    pub(super) cycle: [UnitId; 3],
    pub(super) fresh: StreamId,
    pub(super) product: StreamId,
    pub(super) components: Vec<ComponentId>,
}

pub(super) fn assemble(
    flowsheet: &Flowsheet,
    options: &SinglePhaseRecycleOptions,
) -> RfResult<Plan> {
    if !(1..=MAX_ITERATION_EVALUATIONS).contains(&options.max_iteration_evaluations) {
        return Err(recycle_error(
            "invalid_budget",
            format!("recycle evaluation budget must be in 1..={MAX_ITERATION_EVALUATIONS}"),
        ));
    }
    let components: Vec<_> = flowsheet.components.keys().cloned().collect();
    if components.is_empty() {
        return Err(recycle_error(
            "empty_components",
            "single-vapor recycle requires registered components",
        ));
    }
    for (id, unit) in &flowsheet.units {
        if id != &unit.id {
            return Err(
                recycle_error("identity", "unit map key differs from node identity")
                    .with_related_unit_id(id.clone()),
            );
        }
        if let Some(kind) = BuiltinUnitKind::parse(&unit.kind) {
            let spec = input_specification(kind);
            for parameter in [
                UnitParameter::OutletTemperature,
                UnitParameter::OutletPressure,
                UnitParameter::OutletASplitFraction,
            ] {
                if parameter.value(&unit.parameters).is_some()
                    && !spec.parameters.contains(&parameter)
                {
                    return Err(recycle_error(
                        "unsupported_specification",
                        "physical parameter is unavailable for this unit",
                    )
                    .with_related_unit_id(id.clone()));
                }
            }
        }
    }
    for (id, stream) in &flowsheet.streams {
        if id != &stream.id {
            return Err(
                recycle_error("identity", "stream map key differs from stream identity")
                    .with_related_stream_id(id.clone()),
            );
        }
    }
    for (id, component) in &flowsheet.components {
        if id != &component.id {
            return Err(recycle_error(
                "identity",
                "component map key differs from component identity",
            )
            .with_related_component_id(id.clone()));
        }
    }

    let tear = &options.tear;
    if let Some(initial) = &options.initial_state {
        validate_vector(&components, &initial.component_flows_mol_s, false).map_err(|error| {
            error
                .with_related_stream_id(tear.stream_id.clone())
                .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
        })?;
    }
    let dependencies = material_dependencies(flowsheet)?;
    let invalid_tear = || {
        recycle_error(
            "invalid_tear",
            "tear must bind the actual Splitter.outlet_a to the recycle Mixer inlet",
        )
        .with_related_stream_id(tear.stream_id.clone())
        .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
    };
    let connection = dependencies
        .connections
        .iter()
        .find(|c| c.stream_id == tear.stream_id)
        .ok_or_else(invalid_tear)?;
    let sink = connection.sink.as_ref().ok_or_else(invalid_tear)?;
    let mixer = &sink.unit_id;
    let splitter = &connection.source.unit_id;
    let kind = |id: &UnitId| BuiltinUnitKind::parse(&flowsheet.units[id].kind);
    if splitter != &tear.source.unit_id
        || connection.source.port_name != tear.source.port_name
        || mixer != &tear.sink.unit_id
        || sink.port_name != tear.sink.port_name
        || kind(mixer) != Some(BuiltinUnitKind::Mixer)
        || kind(splitter) != Some(BuiltinUnitKind::Splitter)
        || tear.source.port_name != SPLITTER_OUTLET_A_PORT
    {
        return Err(invalid_tear());
    }

    let unsupported = || {
        recycle_error("unsupported_topology", "single-vapor recycle requires one Mixer → Heater/Cooler → Splitter cycle and one terminal product")
            .with_related_unit_ids(vec![mixer.clone(), splitter.clone()])
            .with_related_stream_id(tear.stream_id.clone())
    };
    let next = |from: &UnitId| {
        dependencies
            .connections
            .iter()
            .find(|c| &c.source.unit_id == from)
            .and_then(|c| c.sink.as_ref())
            .map(|p| &p.unit_id)
            .ok_or_else(unsupported)
    };
    let heat_unit = next(mixer)?;
    if !matches!(
        kind(heat_unit),
        Some(BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler)
    ) || next(heat_unit)? != splitter
    {
        return Err(unsupported());
    }
    let mut terminal = dependencies.connections.iter().filter(|c| c.sink.is_none());
    let product = terminal.next().ok_or_else(unsupported)?;
    if &product.source.unit_id != splitter
        || product.source.port_name != SPLITTER_OUTLET_B_PORT
        || terminal.next().is_some()
    {
        return Err(unsupported());
    }
    let fresh = dependencies
        .connections
        .iter()
        .find(|c| {
            c.stream_id != tear.stream_id
                && c.sink.as_ref().is_some_and(|sink| &sink.unit_id == mixer)
        })
        .ok_or_else(unsupported)?;

    let mut cut_graph = dependencies.successors;
    cut_graph
        .get_mut(splitter)
        .expect("validated splitter")
        .remove(mixer);
    let mut order = topological_order(&cut_graph).ok_or_else(unsupported)?;
    let cycle = [mixer.clone(), heat_unit.clone(), splitter.clone()];
    if !order.ends_with(&cycle) {
        return Err(unsupported());
    }
    order.truncate(order.len() - cycle.len());

    Ok(Plan {
        upstream: order,
        cycle,
        fresh: fresh.stream_id.clone(),
        product: product.stream_id.clone(),
        components,
    })
}
