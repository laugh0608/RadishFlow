use super::*;
use crate::dependency_graph::ordered_components;
use rf_flowsheet::dependencies::material_dependencies;
use rf_unitops::{BuiltinUnitKind, validate_unit_node};
use std::collections::BTreeSet;

pub fn assemble(flow: &Flowsheet, options: &ModularOptions) -> RfResult<ModularPlan> {
    let b = options.boundary;
    if flow.units.is_empty()
        || flow.components.is_empty()
        || !b.temperature_k.is_finite()
        || b.temperature_k <= 0.0
        || !b.pressure_pa.is_finite()
        || b.pressure_pa <= 0.0
    {
        return Err(failure(
            "invalid_configuration",
            "nonempty flow and positive finite fixed T/P required",
        ));
    }
    for (id, c) in &flow.components {
        if id != &c.id {
            return Err(failure(
                "component_identity",
                "component map key differs from identity",
            )
            .with_related_component_id(id.clone()));
        }
    }
    for (id, s) in &flow.streams {
        if id != &s.id {
            return Err(
                failure("stream_identity", "stream map key differs from identity")
                    .with_related_stream_id(id.clone()),
            );
        }
    }
    for (id, u) in &flow.units {
        if id != &u.id {
            return Err(
                failure("unit_identity", "unit map key differs from identity")
                    .with_related_unit_id(id.clone()),
            );
        }
        let spec = validate_unit_node(u)?;
        let contract = rf_unitops::specifications::input_specification(spec.kind);
        use rf_unitops::specifications::UnitParameter;
        for p in [
            UnitParameter::OutletTemperature,
            UnitParameter::OutletPressure,
            UnitParameter::OutletASplitFraction,
        ] {
            if p.value(&u.parameters).is_some() && !contract.parameters.contains(&p) {
                return Err(failure(
                    "unsupported_specification",
                    "physical parameter is unsupported by unit",
                )
                .with_related_unit_id(id.clone()));
            }
        }
        if u.parameters
            .outlet_temperature_k
            .is_some_and(|t| t != b.temperature_k)
            || u.parameters
                .outlet_pressure_pa
                .is_some_and(|p| p != b.pressure_pa)
        {
            return Err(
                failure("fixed_boundary", "declared T/P differ from task boundary")
                    .with_related_unit_id(id.clone()),
            );
        }
    }
    let dependencies = material_dependencies(flow)?;
    let components: Vec<_> = flow.components.keys().cloned().collect();
    let sccs = ordered_components(&dependencies.successors);
    let mut blocks = Vec::new();
    for members in sccs {
        let cyclic =
            members.len() > 1 || dependencies.successors[&members[0]].contains(&members[0]);
        let mut block = ModularBlock {
            members,
            couplings: Vec::new(),
            responses: Vec::new(),
            fresh_by_mixer: BTreeMap::new(),
            products: Vec::new(),
        };
        if cyclic {
            coupled_block(flow, &dependencies.connections, &components, b, &mut block)?;
        }
        blocks.push(block);
    }
    let ordered = blocks;
    if let Some(scales) = &options.scales {
        let bindings: BTreeSet<_> = ordered
            .iter()
            .flat_map(|b| b.couplings.iter().cloned())
            .collect();
        if scales.keys().ne(bindings.iter())
            || scales.values().any(|s| {
                !s.variable_mol_s.is_finite()
                    || s.variable_mol_s <= 0.0
                    || !s.residual_mol_s.is_finite()
                    || s.residual_mol_s <= 0.0
            })
        {
            return Err(failure(
                "scales",
                "scales must exactly bind all couplings with positive finite values",
            ));
        }
    }
    Ok(ModularPlan {
        flowsheet: flow.clone(),
        connections: dependencies.connections,
        components,
        options: options.clone(),
        blocks: ordered,
    })
}

fn coupled_block(
    flow: &Flowsheet,
    connections: &[MaterialConnection],
    components: &[ComponentId],
    boundary: FixedBoundary,
    block: &mut ModularBlock,
) -> RfResult<()> {
    let unsupported = || {
        failure(
            "unsupported_block",
            "requires a two-member recycle or four-member alternating cross-recycle block",
        )
        .with_related_unit_ids(block.members.clone())
    };
    if ![2, 4].contains(&block.members.len()) {
        return Err(unsupported());
    }
    // Reject capacity before constructing dense local response matrices.
    if components.len() > MAX_UNKNOWNS / block.members.len() {
        return Err(failure("capacity", "coupled block exceeds 128 unknowns")
            .with_related_unit_ids(block.members.clone()));
    }
    for id in &block.members {
        let u = &flow.units[id];
        // Material response capability independently rejects unsupported kinds/specs.
        let response = AffineComponentResponse::for_unit(
            u,
            boundary.temperature_k,
            boundary.pressure_pa,
            components,
        )?;
        if u.kind == BuiltinUnitKind::Mixer.as_str() {
            let fresh = connections
                .iter()
                .find(|c| {
                    c.sink
                        .as_ref()
                        .is_some_and(|s| s.unit_id == *id && s.port_name == "inlet_a")
                })
                .ok_or_else(unsupported)?;
            if block.members.contains(&fresh.source.unit_id) {
                return Err(unsupported());
            }
            block
                .fresh_by_mixer
                .insert(id.clone(), fresh.stream_id.clone());
            let output = connections
                .iter()
                .find(|c| c.source.unit_id == *id && c.source.port_name == "outlet")
                .ok_or_else(unsupported)?;
            if output.sink.as_ref().is_none_or(|s| {
                !block.members.contains(&s.unit_id)
                    || s.port_name != "inlet"
                    || flow.units[&s.unit_id].kind != "splitter"
            }) {
                return Err(unsupported());
            }
        } else if u.kind == "splitter" {
            let back = connections
                .iter()
                .find(|c| c.source.unit_id == *id && c.source.port_name == "outlet_a")
                .ok_or_else(unsupported)?;
            if back.sink.as_ref().is_none_or(|s| {
                !block.members.contains(&s.unit_id)
                    || s.port_name != "inlet_b"
                    || flow.units[&s.unit_id].kind != "mixer"
            }) {
                return Err(unsupported());
            }
            let product = connections
                .iter()
                .find(|c| c.source.unit_id == *id && c.source.port_name == "outlet_b")
                .ok_or_else(unsupported)?;
            if product
                .sink
                .as_ref()
                .is_some_and(|s| block.members.contains(&s.unit_id))
            {
                return Err(unsupported());
            }
            block.products.push(product.stream_id.clone());
        } else {
            return Err(unsupported());
        }
        block.responses.push(response);
    }
    if block.fresh_by_mixer.len() * 2 != block.members.len()
        || block.products.len() != block.fresh_by_mixer.len()
    {
        return Err(unsupported());
    }
    for c in connections {
        if let Some(sink) = &c.sink
            && block.members.contains(&c.source.unit_id)
            && block.members.contains(&sink.unit_id)
        {
            for component in components {
                block.couplings.push(CouplingBinding {
                    stream: c.stream_id.clone(),
                    source: DiagnosticPortTarget::new(
                        c.source.unit_id.clone(),
                        c.source.port_name.clone(),
                    ),
                    sink: DiagnosticPortTarget::new(sink.unit_id.clone(), sink.port_name.clone()),
                    component: component.clone(),
                });
            }
        }
    }
    block.couplings.sort();
    block.products.sort();
    if block.couplings.len() != block.members.len() * components.len() {
        return Err(unsupported());
    }
    Ok(())
}
