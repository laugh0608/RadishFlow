use super::{recycle::MixedRecyclePlan, *};
use crate::dependency_graph::strongly_connected_components;
use rf_flowsheet::MaterialConnection;
use rf_types::StreamId;
use rf_unitops::{
    MIXER_OUTLET_PORT, SPLITTER_INLET_PORT, SPLITTER_OUTLET_A_PORT, SPLITTER_OUTLET_B_PORT,
};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SerialRecycleRegion {
    pub(super) equation_block: MixedBlockId,
    pub(super) recycle: MixedRecyclePlan,
    pub(super) outlets: [StreamId; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SerialRecyclePlan {
    pub(super) regions: [SerialRecycleRegion; 2],
}

struct RegionConnections<'a> {
    options: &'a MixedRecycleOptions,
    inlet: &'a MaterialConnection,
    primary_product: &'a MaterialConnection,
    splitter_outlet: &'a MaterialConnection,
}

pub(super) fn assemble(
    flow: &Flowsheet,
    options: &MixedOptions,
    regions: &[MixedRecycleOptions; 2],
    connections: &[MaterialConnection],
    membership: &BTreeMap<UnitId, MixedBlockId>,
    cut: &BTreeMap<MixedBlockId, BTreeSet<MixedBlockId>>,
) -> RfResult<(Vec<MixedBlockId>, SerialRecyclePlan)> {
    let serial_error = |code, message| {
        regions
            .iter()
            .fold(failure(code, message), |error, region| {
                let tear = &region.tear;
                error
                    .with_related_stream_id(tear.stream_id.clone())
                    .with_related_port_target(tear.source.clone())
                    .with_related_port_target(tear.sink.clone())
            })
    };
    if flow.components.len() != 2 {
        return Err(serial_error(
            "serial_components",
            "serial recycle requires exactly two registered components",
        ));
    }
    let [first, second] = regions
        .each_ref()
        .map(|region| region_connections(flow, options, region, connections, membership));
    let first = first?;
    let second = second?;

    let mut uncut = cut.clone();
    for region in regions {
        let tear = &region.tear;
        uncut
            .get_mut(&membership[&tear.source.unit_id])
            .expect("registered source")
            .insert(membership[&tear.sink.unit_id].clone());
    }
    let cycles = strongly_connected_components(&uncut);
    for region in regions {
        let tear = &region.tear;
        let source = &membership[&tear.source.unit_id];
        let sink = &membership[&tear.sink.unit_id];
        if !cycles
            .iter()
            .any(|cycle| cycle.len() == 2 && cycle.contains(source) && cycle.contains(sink))
        {
            return Err(serial_error(
                "serial_regions",
                "each tear must close its own two-block region",
            ));
        }
    }

    let is_feed = |inlet: &MaterialConnection| flow.units[&inlet.source.unit_id].kind == "feed";
    let (upstream, downstream) = match (is_feed(first.inlet), is_feed(second.inlet)) {
        (true, false) => (first, second),
        (false, true) => (second, first),
        _ => {
            return Err(serial_error(
                "serial_feed_order",
                "one region must receive a Local Feed and the other a transfer Mixer",
            ));
        }
    };
    let transfer = &downstream.inlet.source.unit_id;
    let transfer_block = &membership[transfer];
    let linked_transfer = upstream
        .splitter_outlet
        .sink
        .as_ref()
        .is_some_and(|sink| &sink.unit_id == transfer);
    let makeup = connections.iter().find(|c| {
        is_feed(c)
            && c.source.unit_id != upstream.inlet.source.unit_id
            && c.sink
                .as_ref()
                .is_some_and(|sink| &sink.unit_id == transfer)
    });
    let Some(makeup) = makeup.filter(|_| {
        linked_transfer
            && downstream.splitter_outlet.sink.is_none()
            && flow.units[transfer].kind == "mixer"
            && options
                .blocks
                .iter()
                .any(|b| b.id == *transfer_block && matches!(b.method, MixedBlockMethod::Local))
    }) else {
        return Err(serial_error(
            "serial_transfer",
            "the upstream outlet and a separate Feed must supply the Local transfer Mixer",
        )
        .with_related_unit_id(transfer.clone())
        .with_related_stream_id(upstream.splitter_outlet.stream_id.clone())
        .with_related_stream_id(downstream.inlet.stream_id.clone()));
    };

    let upstream_tear = &upstream.options.tear;
    let downstream_tear = &downstream.options.tear;
    let order = [
        &membership[&upstream.inlet.source.unit_id],
        &membership[&upstream_tear.sink.unit_id],
        &membership[&upstream_tear.source.unit_id],
        &membership[&makeup.source.unit_id],
        transfer_block,
        &membership[&downstream_tear.sink.unit_id],
        &membership[&downstream_tear.source.unit_id],
    ];
    let positions: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    if positions.len() != order.len()
        || positions.len() != cut.len()
        || cut.iter().any(|(source, outputs)| {
            outputs
                .iter()
                .any(|sink| positions[source] >= positions[sink])
        })
    {
        return Err(serial_error(
            "serial_order",
            "the cut graph must contain only the supported serial regions and their supply blocks",
        ));
    }
    let region = |connections: RegionConnections<'_>, start| SerialRecycleRegion {
        equation_block: membership[&connections.options.tear.sink.unit_id].clone(),
        recycle: MixedRecyclePlan {
            cycle: start..start + 2,
            fresh_stream: connections.inlet.stream_id.clone(),
            options: connections.options.clone(),
        },
        outlets: [
            connections.primary_product.stream_id.clone(),
            connections.splitter_outlet.stream_id.clone(),
        ],
    };
    Ok((
        order.into_iter().cloned().collect(),
        SerialRecyclePlan {
            regions: [region(upstream, 1), region(downstream, 5)],
        },
    ))
}

fn region_connections<'a>(
    flow: &Flowsheet,
    options: &MixedOptions,
    region: &'a MixedRecycleOptions,
    connections: &'a [MaterialConnection],
    membership: &BTreeMap<UnitId, MixedBlockId>,
) -> RfResult<RegionConnections<'a>> {
    let tear = &region.tear;
    let inlet = recycle::validate_tear(flow, options, region, connections, membership)?;
    let equation_block = options
        .blocks
        .iter()
        .find(|b| b.id == membership[&tear.sink.unit_id])
        .expect("validated membership");
    let outlet = |unit: &UnitId, port| {
        connections
            .iter()
            .find(|c| c.source.unit_id == *unit && c.source.port_name == port)
    };
    let invalid = || {
        recycle::tear_error(
            tear,
            "serial_region_shape",
            "expected an EO Mixer and primary Splitter feeding a Local recycle Splitter",
        )
    };
    let primary = outlet(&tear.sink.unit_id, MIXER_OUTLET_PORT)
        .and_then(|c| c.sink.as_ref())
        .filter(|p| {
            p.port_name == SPLITTER_INLET_PORT
                && equation_block.members.contains(&p.unit_id)
                && flow.units[&p.unit_id].kind == "splitter"
        })
        .filter(|_| equation_block.members.len() == 2)
        .ok_or_else(invalid)?;
    if !outlet(&primary.unit_id, SPLITTER_OUTLET_A_PORT).is_some_and(|c| {
        c.sink
            .as_ref()
            .is_some_and(|p| p.unit_id == tear.source.unit_id && p.port_name == SPLITTER_INLET_PORT)
    }) {
        return Err(invalid());
    }
    for id in [&primary.unit_id, &tear.source.unit_id] {
        if !flow.units[id]
            .parameters
            .outlet_a_split_fraction
            .is_some_and(|p| p > 0.0 && p < 1.0)
        {
            return Err(recycle::tear_error(
                tear,
                "serial_fraction",
                "serial recycle split fractions must be strictly inside (0, 1)",
            )
            .with_related_unit_id(id.clone()));
        }
    }
    Ok(RegionConnections {
        options: region,
        inlet,
        primary_product: outlet(&primary.unit_id, SPLITTER_OUTLET_B_PORT)
            .filter(|c| c.sink.is_none())
            .ok_or_else(invalid)?,
        splitter_outlet: outlet(&tear.source.unit_id, SPLITTER_OUTLET_B_PORT)
            .ok_or_else(invalid)?,
    })
}
