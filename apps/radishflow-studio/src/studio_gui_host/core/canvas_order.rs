use std::collections::{BTreeMap, BTreeSet};

use rf_model::{Flowsheet, UnitNode};
use rf_types::{PortDirection, PortKind};

/// Presentation order only: invalid or cyclic drafts still need to be drawn.
pub(super) fn canvas_units_in_layout_order(flowsheet: &Flowsheet) -> Vec<&UnitNode> {
    let mut source_unit_by_stream = BTreeMap::new();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != PortKind::Material || port.direction != PortDirection::Outlet {
                continue;
            }
            if let Some(stream_id) = port.stream_id.as_ref() {
                source_unit_by_stream
                    .entry(stream_id.clone())
                    .or_insert_with(|| unit.id.clone());
            }
        }
    }

    let mut dependencies = flowsheet
        .units
        .keys()
        .map(|unit_id| (unit_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != PortKind::Material || port.direction != PortDirection::Inlet {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            let Some(source_unit_id) = source_unit_by_stream.get(stream_id) else {
                continue;
            };
            if source_unit_id != &unit.id {
                dependencies
                    .entry(unit.id.clone())
                    .or_default()
                    .insert(source_unit_id.clone());
            }
        }
    }

    let mut downstream_by_unit = BTreeMap::<_, Vec<_>>::new();
    let mut ready = BTreeSet::new();
    for (unit_id, upstream) in &dependencies {
        if upstream.is_empty() {
            ready.insert(unit_id.clone());
        }
        for source_unit_id in upstream {
            downstream_by_unit
                .entry(source_unit_id.clone())
                .or_default()
                .push(unit_id.clone());
        }
    }

    let mut ordered_unit_ids = Vec::with_capacity(dependencies.len());
    while let Some(unit_id) = ready.pop_first() {
        dependencies.remove(&unit_id);
        if let Some(downstream) = downstream_by_unit.get(&unit_id) {
            for downstream_id in downstream {
                if let Some(upstream) = dependencies.get_mut(downstream_id) {
                    upstream.remove(&unit_id);
                    if upstream.is_empty() {
                        ready.insert(downstream_id.clone());
                    }
                }
            }
        }
        ordered_unit_ids.push(unit_id);
    }

    // Preserve the ID order of cycles and all nodes blocked behind them.
    ordered_unit_ids.extend(dependencies.into_keys());
    ordered_unit_ids
        .into_iter()
        .filter_map(|unit_id| flowsheet.units.get(&unit_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_model::UnitPort;
    use rf_types::{StreamId, UnitId};

    fn graph(unit_ids: &[&str], edges: &[(&str, &str, &str)]) -> Flowsheet {
        let mut flowsheet = Flowsheet::new("Canvas ordering");
        for id in unit_ids {
            let unit = UnitNode::new(*id, *id, "layout-fixture", Vec::new());
            flowsheet.units.insert(unit.id.clone(), unit);
        }
        for (source, sink, stream) in edges {
            for (unit, direction) in [
                (source, PortDirection::Outlet),
                (sink, PortDirection::Inlet),
            ] {
                flowsheet
                    .units
                    .get_mut(&UnitId::new(*unit))
                    .expect("fixture unit")
                    .ports
                    .push(UnitPort::new(
                        *stream,
                        direction,
                        PortKind::Material,
                        Some(StreamId::new(*stream)),
                    ));
            }
        }
        flowsheet
    }

    fn order(flowsheet: &Flowsheet) -> Vec<&str> {
        canvas_units_in_layout_order(flowsheet)
            .into_iter()
            .map(|unit| unit.id.as_str())
            .collect()
    }

    #[test]
    fn canvas_units_default_to_material_process_order_for_heater_flash_example() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("example project");
        assert_eq!(
            order(&project.document.flowsheet),
            ["feed-1", "heater-1", "flash-1"]
        );
    }

    #[test]
    fn newly_ready_units_compete_with_existing_roots_in_id_order() {
        let flowsheet = graph(
            &["z-independent", "c-branch", "b-join", "m-feed", "a-release"],
            &[
                ("m-feed", "c-branch", "mc"),
                ("m-feed", "a-release", "ma"),
                ("a-release", "b-join", "ab"),
                ("c-branch", "b-join", "cb"),
            ],
        );
        assert_eq!(
            order(&flowsheet),
            ["m-feed", "a-release", "c-branch", "b-join", "z-independent"]
        );
    }

    #[test]
    fn cycles_and_blocked_descendants_follow_all_ready_nodes_in_id_order() {
        let flowsheet = graph(
            &["a-downstream", "b-cycle", "c-cycle", "z-independent"],
            &[
                ("b-cycle", "c-cycle", "bc"),
                ("c-cycle", "b-cycle", "cb"),
                ("c-cycle", "a-downstream", "ca"),
            ],
        );
        assert_eq!(
            order(&flowsheet),
            ["z-independent", "a-downstream", "b-cycle", "c-cycle"]
        );
    }

    #[test]
    fn repeated_dependencies_and_self_connections_do_not_block_ready_units() {
        let flowsheet = graph(
            &["a-sink", "z-source"],
            &[
                ("z-source", "a-sink", "one"),
                ("z-source", "a-sink", "two"),
                ("a-sink", "a-sink", "self"),
            ],
        );
        assert_eq!(order(&flowsheet), ["z-source", "a-sink"]);
    }

    #[test]
    fn incomplete_and_non_material_connections_do_not_create_dependencies() {
        let mut flowsheet = graph(&["a", "b", "c"], &[("c", "a", "energy")]);
        for unit in flowsheet.units.values_mut() {
            for port in &mut unit.ports {
                port.kind = PortKind::Energy;
            }
        }
        let ports = &mut flowsheet.units.get_mut(&UnitId::new("a")).unwrap().ports;
        ports.push(UnitPort::new(
            "unbound",
            PortDirection::Inlet,
            PortKind::Material,
            None,
        ));
        ports.push(UnitPort::new(
            "missing-source",
            PortDirection::Inlet,
            PortKind::Material,
            Some(StreamId::new("orphan")),
        ));
        assert_eq!(order(&flowsheet), ["a", "b", "c"]);
        assert!(order(&Flowsheet::new("Empty")).is_empty());
    }

    #[test]
    fn multiply_bound_stream_uses_first_source_in_id_order_for_presentation() {
        let flowsheet = graph(
            &["a-sink", "b-source", "c-source"],
            &[
                ("c-source", "a-sink", "shared"),
                ("b-source", "a-sink", "shared"),
            ],
        );
        assert_eq!(order(&flowsheet), ["b-source", "a-sink", "c-source"]);
    }
}
