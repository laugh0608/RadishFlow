use super::*;
use crate::{
    dependency_graph::strongly_connected_components, fixed_tp_material as material,
    partitioned::EvaluationPhase,
};
use rf_flowsheet::MaterialConnection;
use rf_model::MaterialStreamState;
use rf_types::{ComponentId, StreamId};
use rf_unitops::{MIXER_INLET_A_PORT, MIXER_INLET_B_PORT, SPLITTER_OUTLET_A_PORT};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct MixedRecyclePlan {
    pub(super) cycle: Range<usize>,
    pub(super) fresh_stream: StreamId,
}

pub(super) fn tear_error(tear: &TearSpecification, code: &str, message: &str) -> RfError {
    failure(code, message)
        .with_related_stream_id(tear.stream_id.clone())
        .with_related_unit_ids(vec![tear.source.unit_id.clone(), tear.sink.unit_id.clone()])
        .with_related_port_targets(vec![tear.source.clone(), tear.sink.clone()])
}

pub(super) fn prepare_tear(
    flow: &Flowsheet,
    options: &MixedOptions,
    recycle: &MixedRecycleOptions,
    connections: &[MaterialConnection],
    membership: &BTreeMap<UnitId, MixedBlockId>,
) -> RfResult<StreamId> {
    let tear = &recycle.tear;
    let connection = connections
        .iter()
        .find(|c| c.stream_id == tear.stream_id)
        .ok_or_else(|| tear_error(tear, "missing_tear", "tear stream is absent"))?;
    if connection.source.unit_id != tear.source.unit_id
        || connection.source.port_name != tear.source.port_name
        || !connection.sink.as_ref().is_some_and(|sink| {
            sink.unit_id == tear.sink.unit_id && sink.port_name == tear.sink.port_name
        })
    {
        return Err(tear_error(
            tear,
            "tear_binding",
            "tear bindings differ from the engineering connection",
        ));
    }
    let source = &flow.units[&tear.source.unit_id];
    let sink = &flow.units[&tear.sink.unit_id];
    let block = |unit: &UnitId| {
        options
            .blocks
            .iter()
            .find(|b| b.id == membership[unit])
            .expect("validated membership")
    };
    if let Some(feed) = flow
        .units
        .values()
        .find(|u| u.kind == "feed" && !matches!(block(&u.id).method, MixedBlockMethod::Local))
    {
        return Err(tear_error(
            tear,
            "recycle_fresh_feed",
            "Feeds in the recycle plan must be Local blocks evaluated before the loop",
        )
        .with_related_unit_id(feed.id.clone()));
    }
    if source.kind != "splitter"
        || tear.source.port_name != SPLITTER_OUTLET_A_PORT
        || !matches!(block(&source.id).method, MixedBlockMethod::Local)
        || sink.kind != "mixer"
        || tear.sink.port_name != MIXER_INLET_B_PORT
        || !matches!(
            block(&sink.id).method,
            MixedBlockMethod::EquationOriented { .. }
        )
    {
        return Err(tear_error(
            tear,
            "tear_role",
            "tear must connect a Local Splitter outlet_a to an EO Mixer inlet_b",
        ));
    }
    if let Some(values) = &tear.initial_component_flows_mol_s {
        material::validate_vector(
            &flow.components.keys().cloned().collect::<Vec<_>>(),
            values,
            false,
        )
        .map_err(|e| e.with_related_stream_id(tear.stream_id.clone()))?;
        if !values.values().sum::<f64>().is_finite() {
            return Err(tear_error(
                tear,
                "tear_initial_flow",
                "initial component sum overflows",
            ));
        }
    }
    connections
        .iter()
        .find(|c| {
            c.sink
                .as_ref()
                .is_some_and(|p| p.unit_id == sink.id && p.port_name == MIXER_INLET_A_PORT)
                && flow.units[&c.source.unit_id].kind == "feed"
                && matches!(block(&c.source.unit_id).method, MixedBlockMethod::Local)
        })
        .map(|c| c.stream_id.clone())
        .ok_or_else(|| {
            tear_error(
                tear,
                "tear_fresh_feed",
                "the EO tear consumer requires a Local Feed at inlet_a",
            )
        })
}

pub(super) fn order_recycle(
    flow: &Flowsheet,
    tear: &TearSpecification,
    cut: &BTreeMap<MixedBlockId, BTreeSet<MixedBlockId>>,
    membership: &BTreeMap<UnitId, MixedBlockId>,
    ordered: &mut [MixedBlock],
    fresh_stream: StreamId,
) -> RfResult<MixedRecyclePlan> {
    let source = &membership[&tear.source.unit_id];
    let sink = &membership[&tear.sink.unit_id];
    let mut graph = cut.clone();
    graph
        .get_mut(source)
        .expect("registered source")
        .insert(sink.clone());
    let cycle: BTreeSet<_> = strongly_connected_components(&graph)
        .into_iter()
        .find(|scc| scc.contains(sink) && scc.contains(source))
        .ok_or_else(|| {
            tear_error(
                tear,
                "tear_outside_cycle",
                "tear does not close a cross-block cycle",
            )
        })?
        .into_iter()
        .collect();
    if cycle
        .iter()
        .any(|id| cut[id].intersection(&cycle).count() > 1)
    {
        return Err(tear_error(
            tear,
            "recycle_capability",
            "the first cross-block recycle requires a single dependency path from sink to source",
        ));
    }
    for block in ordered.iter().filter(|b| cycle.contains(&b.id)) {
        if &block.id != sink
            && (!matches!(block.method, MixedBlockMethod::Local)
                || block
                    .members
                    .iter()
                    .any(|u| !matches!(flow.units[u].kind.as_str(), "mixer" | "splitter")))
        {
            return Err(tear_error(
                tear,
                "recycle_capability",
                "the cross-block cycle supports one EO group and Local Mixer/Splitter blocks",
            )
            .with_related_unit_ids(block.members.iter().cloned().collect()));
        }
    }
    let mut downstream = BTreeSet::new();
    let mut pending: Vec<_> = cycle.iter().cloned().collect();
    while let Some(id) = pending.pop() {
        if downstream.insert(id.clone()) {
            pending.extend(cut[&id].iter().cloned());
        }
    }
    // Stable sort: preserve the cut DAG order within each evaluation phase.
    ordered.sort_by_key(|block| {
        if cycle.contains(&block.id) {
            1
        } else {
            usize::from(downstream.contains(&block.id)) * 2
        }
    });
    let start = ordered
        .iter()
        .position(|b| cycle.contains(&b.id))
        .expect("cycle");
    Ok(MixedRecyclePlan {
        cycle: start..start + cycle.len(),
        fresh_stream,
    })
}

pub(super) fn record_sweep(
    report: &mut MixedRecycleReport,
    assumed: MaterialStreamState,
    returned: MaterialStreamState,
    residuals: BTreeMap<ComponentId, f64>,
    phase: EvaluationPhase,
    index: usize,
) -> RfResult<bool> {
    let sample = material::sample(&residuals, &report.component_scales_mol_s, phase, index)?;
    let converged = material::converged(&sample);
    report.residual_trace.push(sample);
    report.last_sweep = Some(MixedRecycleSweep {
        assumed,
        returned,
        residuals_mol_s: residuals,
    });
    Ok(converged)
}
