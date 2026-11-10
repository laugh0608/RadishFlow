use super::*;
use rf_flowsheet::dependencies::material_dependencies;
use rf_unitops::{BuiltinUnitKind, MIXER_INLET_A_PORT, MIXER_INLET_B_PORT, SPLITTER_OUTLET_A_PORT};
use std::collections::{BTreeMap, BTreeSet};

use crate::dependency_graph::{Graph, ordered_components, topological_order};

pub fn assemble_partitioned_plan(
    flowsheet: &Flowsheet,
    options: &RecycleSolveOptions,
) -> RfResult<PartitionedSolvePlan> {
    if flowsheet.units.is_empty() || flowsheet.components.is_empty() {
        return Err(failure(
            "empty_input",
            "partitioned solve requires units and registered components",
        ));
    }
    options.validate_configuration()?;
    for (id, unit) in &flowsheet.units {
        if id != &unit.id {
            return Err(
                failure("identity", "unit map identity differs from node identity")
                    .with_related_unit_id(id.clone()),
            );
        }
        if unit
            .parameters
            .outlet_temperature_k
            .is_some_and(|t| t != options.boundary.temperature_k)
            || unit
                .parameters
                .outlet_pressure_pa
                .is_some_and(|p| p != options.boundary.pressure_pa)
        {
            return Err(failure(
                "unsupported_specification",
                "declared T/P targets must agree with the fixed task boundary",
            )
            .with_related_unit_id(id.clone()));
        }
        if let Some(kind) = BuiltinUnitKind::parse(&unit.kind) {
            use rf_unitops::specifications::{UnitParameter, input_specification};
            let spec = input_specification(kind);
            for parameter in [
                UnitParameter::OutletTemperature,
                UnitParameter::OutletPressure,
                UnitParameter::OutletASplitFraction,
            ] {
                if parameter.value(&unit.parameters).is_some()
                    && !spec.parameters.contains(&parameter)
                {
                    return Err(failure(
                        "unsupported_specification",
                        "physical parameter is not supported by this unit model",
                    )
                    .with_related_unit_id(id.clone()));
                }
            }
        }
    }
    for (id, stream) in &flowsheet.streams {
        if id != &stream.id {
            return Err(failure(
                "identity",
                "stream map identity differs from stream identity",
            )
            .with_related_stream_id(id.clone()));
        }
    }
    for (id, component) in &flowsheet.components {
        if id != &component.id {
            return Err(failure(
                "identity",
                "component map identity differs from component identity",
            ));
        }
    }
    let dependencies = material_dependencies(flowsheet)?;
    let components: Vec<_> = flowsheet.components.keys().cloned().collect();
    let sccs = ordered_components(&dependencies.successors);
    let membership: BTreeMap<_, _> = sccs
        .iter()
        .enumerate()
        .flat_map(|(i, members)| members.iter().cloned().map(move |id| (id, i)))
        .collect();
    let mut tears_by_block = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for tear in &options.tears {
        let target_error = |code, message| {
            failure(code, message)
                .with_related_stream_id(tear.stream_id.clone())
                .with_related_unit_ids(vec![tear.source.unit_id.clone(), tear.sink.unit_id.clone()])
                .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
        };
        if !seen.insert(tear.stream_id.clone()) {
            return Err(target_error(
                "duplicate_tear",
                "tear stream configured more than once",
            ));
        }
        let connection = dependencies
            .connections
            .iter()
            .find(|c| c.stream_id == tear.stream_id)
            .ok_or_else(|| {
                target_error(
                    "missing_tear_stream",
                    "configured tear stream does not exist",
                )
            })?;
        let Some(sink) = &connection.sink else {
            return Err(target_error(
                "outside_tear",
                "terminal stream cannot be a tear",
            ));
        };
        if connection.source.unit_id != tear.source.unit_id
            || connection.source.port_name != tear.source.port_name
            || sink.unit_id != tear.sink.unit_id
            || sink.port_name != tear.sink.port_name
        {
            return Err(target_error(
                "tear_binding",
                "configured tear source/sink binding differs from engineering connection",
            ));
        }
        let index = membership[&connection.source.unit_id];
        let members = &sccs[index];
        let cyclic =
            members.len() > 1 || dependencies.successors[&members[0]].contains(&members[0]);
        if membership[&sink.unit_id] != index || !cyclic {
            return Err(target_error(
                "outside_tear",
                "configured tear is outside a cyclic block",
            ));
        }
        if tears_by_block.insert(index, tear.clone()).is_some() {
            return Err(target_error(
                "multiple_tears",
                "only one explicit tear per supported cyclic block is available",
            ));
        }
        for (vector, positive) in [
            (&tear.initial_component_flows_mol_s, false),
            (&tear.component_scales_mol_s, true),
        ] {
            if let Some(vector) = vector {
                checks::validate_vector(&components, vector, positive).map_err(|e| {
                    e.with_related_stream_id(tear.stream_id.clone())
                        .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
                })?;
            }
        }
    }
    let mut blocks = Vec::with_capacity(sccs.len());
    for (index, members) in sccs.iter().enumerate() {
        let cyclic =
            members.len() > 1 || dependencies.successors[&members[0]].contains(&members[0]);
        let recycle = if cyclic {
            let tear = tears_by_block.remove(&index).ok_or_else(|| {
                failure("missing_tear", "cyclic block requires an explicit tear")
                    .with_related_unit_ids(members.clone())
            })?;
            let cut_graph: Graph = members
                .iter()
                .map(|id| (id.clone(), BTreeSet::new()))
                .collect();
            let mut cut_graph = cut_graph;
            for connection in &dependencies.connections {
                if connection.stream_id == tear.stream_id {
                    continue;
                }
                if let Some(sink) = &connection.sink
                    && membership[&connection.source.unit_id] == index
                    && membership[&sink.unit_id] == index
                {
                    cut_graph
                        .get_mut(&connection.source.unit_id)
                        .expect("SCC source")
                        .insert(sink.unit_id.clone());
                }
            }
            if topological_order(&cut_graph).is_none() {
                return Err(failure(
                    "remaining_cycle",
                    "configured tear does not remove all cyclic dependencies",
                )
                .with_related_unit_ids(members.clone())
                .with_related_stream_id(tear.stream_id.clone()));
            }
            Some(recycle_capability(
                flowsheet,
                &dependencies.connections,
                members,
                tear,
                options.boundary,
            )?)
        } else {
            None
        };
        blocks.push(ExecutionBlock {
            members: members.clone(),
            kind: if cyclic {
                BlockKind::Recycle
            } else {
                BlockKind::Acyclic
            },
            recycle,
        });
    }
    Ok(PartitionedSolvePlan {
        flowsheet: flowsheet.clone(),
        connections: dependencies.connections,
        components,
        options: options.clone(),
        blocks,
    })
}

fn recycle_capability(
    flowsheet: &Flowsheet,
    connections: &[MaterialConnection],
    members: &[UnitId],
    tear: TearSpecification,
    boundary: FixedRecycleBoundary,
) -> RfResult<RecyclePlan> {
    let unsupported = || {
        failure("unsupported_recycle", "supported cyclic block requires Mixer -> Splitter with Splitter.outlet_a returning to a Mixer inlet")
        .with_related_unit_ids(members.to_vec()).with_related_stream_id(tear.stream_id.clone())
        .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
    };
    if members.len() != 2
        || flowsheet.units[&tear.source.unit_id].kind != BuiltinUnitKind::Splitter.as_str()
        || flowsheet.units[&tear.sink.unit_id].kind != BuiltinUnitKind::Mixer.as_str()
        || tear.source.port_name != SPLITTER_OUTLET_A_PORT
        || ![MIXER_INLET_A_PORT, MIXER_INLET_B_PORT].contains(&tear.sink.port_name.as_str())
    {
        return Err(unsupported());
    }
    let mixer = &flowsheet.units[&tear.sink.unit_id];
    let splitter = &flowsheet.units[&tear.source.unit_id];
    if mixer.parameters.outlet_pressure_pa != Some(boundary.pressure_pa)
        || mixer.parameters.outlet_temperature_k.is_some()
        || splitter.parameters.outlet_pressure_pa.is_some()
        || splitter.parameters.outlet_temperature_k.is_some()
    {
        return Err(failure(
            "unsupported_specification",
            "cyclic Mixer requires explicit fixed pressure; Splitter cannot carry T/P targets",
        )
        .with_related_unit_ids(members.to_vec()));
    }
    let fraction = splitter.parameters.outlet_a_split_fraction.ok_or_else(|| {
        failure(
            "missing_fraction",
            "Splitter requires an explicit outlet_a_split_fraction",
        )
        .with_related_unit_id(splitter.id.clone())
    })?;
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(failure(
            "invalid_fraction",
            "split fraction must be finite in [0, 1]",
        )
        .with_related_unit_id(splitter.id.clone()));
    }
    let fresh = connections
        .iter()
        .find(|c| {
            c.sink
                .as_ref()
                .is_some_and(|s| s.unit_id == mixer.id && s.port_name != tear.sink.port_name)
        })
        .ok_or_else(unsupported)?;
    if members.contains(&fresh.source.unit_id) {
        return Err(unsupported());
    }
    let forward = connections
        .iter()
        .find(|c| c.source.unit_id == mixer.id)
        .ok_or_else(unsupported)?;
    if forward
        .sink
        .as_ref()
        .is_none_or(|s| s.unit_id != splitter.id)
    {
        return Err(unsupported());
    }
    let product = connections
        .iter()
        .find(|c| c.source.unit_id == splitter.id && c.source.port_name == "outlet_b")
        .ok_or_else(unsupported)?;
    if product
        .sink
        .as_ref()
        .is_some_and(|s| members.contains(&s.unit_id))
    {
        return Err(unsupported());
    }
    Ok(RecyclePlan {
        mixer: mixer.id.clone(),
        splitter: splitter.id.clone(),
        fresh_stream: fresh.stream_id.clone(),
        product_stream: product.stream_id.clone(),
        tear,
    })
}
