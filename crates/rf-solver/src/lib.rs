use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rf_flash::TpFlashSolver;
use rf_flowsheet::validate_connections;
use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_thermo::ThermoProvider;
use rf_types::{PortDirection, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{
    FEED_KIND, FLASH_DRUM_KIND, FLASH_DRUM_LIQUID_PORT, FLASH_DRUM_VAPOR_PORT,
    FEED_OUTLET_PORT, Feed, FlashDrum, MIXER_KIND, MIXER_OUTLET_PORT, Mixer, StreamTarget,
    UnitOperation, UnitOperationInputs, UnitOperationServices, validate_unit_node,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Converged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSolveStep {
    pub unit_id: UnitId,
    pub unit_name: String,
    pub unit_kind: String,
    pub produced_stream_ids: Vec<StreamId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveSnapshot {
    pub status: SolveStatus,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub steps: Vec<UnitSolveStep>,
}

impl SolveSnapshot {
    pub fn stream(&self, id: &StreamId) -> Option<&MaterialStreamState> {
        self.streams.get(id)
    }
}

pub struct SolverServices<'a> {
    pub thermo: &'a dyn ThermoProvider,
    pub flash_solver: &'a dyn TpFlashSolver,
}

pub trait FlowsheetSolver {
    fn solve(&self, services: &SolverServices<'_>, flowsheet: &Flowsheet)
        -> RfResult<SolveSnapshot>;
}

#[derive(Debug, Default)]
pub struct SequentialModularSolver;

impl FlowsheetSolver for SequentialModularSolver {
    fn solve(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
    ) -> RfResult<SolveSnapshot> {
        let execution_order = topological_unit_order(flowsheet)?;
        let mut solved_streams = BTreeMap::<StreamId, MaterialStreamState>::new();
        let mut steps = Vec::with_capacity(execution_order.len());
        let unit_services = UnitOperationServices {
            thermo: Some(services.thermo),
            flash_solver: Some(services.flash_solver),
        };

        for unit_id in execution_order {
            let unit = flowsheet.unit(&unit_id)?;
            let spec = validate_unit_node(unit)?;
            let operation = instantiate_operation(unit, flowsheet)?;
            let mut inputs = UnitOperationInputs::new();

            for port in spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Inlet)
            {
                let stream = resolved_stream_for_port(unit, port.name, &solved_streams)?;
                inputs.insert_material_stream(port.name, stream.clone());
            }

            let outputs = operation.run(&unit_services, &inputs)?;
            let mut produced_stream_ids = Vec::new();

            for port in spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Outlet)
            {
                let stream = outputs.stream(port.name).ok_or_else(|| {
                    RfError::invalid_input(format!(
                        "unit `{}` did not produce expected outlet port `{}`",
                        unit.id, port.name
                    ))
                })?;
                produced_stream_ids.push(stream.id.clone());
                solved_streams.insert(stream.id.clone(), stream.clone());
            }

            steps.push(UnitSolveStep {
                unit_id: unit.id.clone(),
                unit_name: unit.name.clone(),
                unit_kind: unit.kind.clone(),
                produced_stream_ids,
            });
        }

        Ok(SolveSnapshot {
            status: SolveStatus::Converged,
            streams: solved_streams,
            steps,
        })
    }
}

fn topological_unit_order(flowsheet: &Flowsheet) -> RfResult<Vec<UnitId>> {
    let connections = validate_connections(flowsheet)?;
    let mut incoming_counts = flowsheet
        .units
        .keys()
        .cloned()
        .map(|unit_id| (unit_id, 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut downstream_units = BTreeMap::<UnitId, BTreeSet<UnitId>>::new();

    for connection in connections {
        if let Some(sink) = connection.sink {
            if sink.unit_id != connection.source.unit_id {
                downstream_units
                    .entry(connection.source.unit_id.clone())
                    .or_default()
                    .insert(sink.unit_id.clone());
                *incoming_counts
                    .entry(sink.unit_id.clone())
                    .or_insert(0) += 1;
            }
        }
    }

    let mut ready = VecDeque::from(
        incoming_counts
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(unit_id, _)| unit_id.clone())
            .collect::<Vec<_>>(),
    );
    let mut ordered = Vec::with_capacity(incoming_counts.len());

    while let Some(unit_id) = ready.pop_front() {
        ordered.push(unit_id.clone());

        if let Some(children) = downstream_units.get(&unit_id) {
            for child_id in children {
                let count = incoming_counts.get_mut(child_id).ok_or_else(|| {
                    RfError::invalid_input(format!(
                        "internal solver graph missing incoming count for unit `{child_id}`"
                    ))
                })?;
                *count -= 1;
                if *count == 0 {
                    ready.push_back(child_id.clone());
                }
            }
        }
    }

    if ordered.len() != incoming_counts.len() {
        return Err(RfError::invalid_input(
            "flowsheet contains a cycle or unresolved dependency; only acyclic sequential flowsheets are supported in the current solver",
        ));
    }

    Ok(ordered)
}

fn instantiate_operation(unit: &UnitNode, flowsheet: &Flowsheet) -> RfResult<Box<dyn UnitOperation>> {
    match unit.kind.as_str() {
        FEED_KIND => {
            let outlet = stream_for_port(unit, FEED_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(Feed::new(outlet.clone())))
        }
        MIXER_KIND => {
            let outlet = stream_target_for_port(unit, MIXER_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(Mixer::new(outlet)))
        }
        FLASH_DRUM_KIND => {
            let liquid = stream_target_for_port(unit, FLASH_DRUM_LIQUID_PORT, flowsheet)?;
            let vapor = stream_target_for_port(unit, FLASH_DRUM_VAPOR_PORT, flowsheet)?;
            Ok(Box::new(FlashDrum::new(liquid, vapor)))
        }
        _ => Err(RfError::invalid_input(format!(
            "unit `{}` uses unsupported solver kind `{}`",
            unit.id, unit.kind
        ))),
    }
}

fn stream_for_port<'a>(
    unit: &UnitNode,
    port_name: &str,
    flowsheet: &'a Flowsheet,
) -> RfResult<&'a MaterialStreamState> {
    let stream_id = port_stream_id(unit, port_name)?;
    flowsheet.stream(stream_id)
}

fn stream_target_for_port(
    unit: &UnitNode,
    port_name: &str,
    flowsheet: &Flowsheet,
) -> RfResult<StreamTarget> {
    let stream = stream_for_port(unit, port_name, flowsheet)?;
    Ok(StreamTarget::new(stream.id.clone(), stream.name.clone()))
}

fn resolved_stream_for_port<'a>(
    unit: &UnitNode,
    port_name: &str,
    solved_streams: &'a BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<&'a MaterialStreamState> {
    let stream_id = port_stream_id(unit, port_name)?;
    solved_streams.get(stream_id).ok_or_else(|| {
        RfError::invalid_input(format!(
            "unit `{}` requires inlet stream `{}` on port `{}` before it has been solved",
            unit.id, stream_id, port_name
        ))
    })
}

fn port_stream_id<'a>(unit: &'a UnitNode, port_name: &str) -> RfResult<&'a StreamId> {
    let port = find_port(unit, port_name)?;
    port.stream_id.as_ref().ok_or_else(|| {
        RfError::invalid_input(format!(
            "unit `{}` port `{}` is missing a connected stream id",
            unit.id, port.name
        ))
    })
}

fn find_port<'a>(unit: &'a UnitNode, port_name: &str) -> RfResult<&'a UnitPort> {
    unit.ports
        .iter()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not define port `{port_name}`",
                unit.id
            ))
        })
}

#[cfg(test)]
mod tests {
    use rf_flash::PlaceholderTpFlashSolver;
    use rf_model::{Composition, Component, Flowsheet, MaterialStreamState};
    use rf_store::parse_project_file_json;
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::{ComponentId, PhaseLabel};
    use rf_unitops::{build_feed_node, build_flash_drum_node, build_mixer_node};

    use super::{FlowsheetSolver, SequentialModularSolver, SolveStatus, SolverServices};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
        );
    }

    fn binary_composition(first: f64, second: f64) -> Composition {
        [
            (ComponentId::new("component-a"), first),
            (ComponentId::new("component-b"), second),
        ]
        .into_iter()
        .collect()
    }

    fn build_stream(
        id: &str,
        name: &str,
        temperature_k: f64,
        pressure_pa: f64,
        total_molar_flow_mol_s: f64,
        composition: Composition,
    ) -> MaterialStreamState {
        MaterialStreamState::from_tpzf(
            id,
            name,
            temperature_k,
            pressure_pa,
            total_molar_flow_mol_s,
            composition,
        )
    }

    fn build_provider() -> PlaceholderThermoProvider {
        let pressure_pa = 100_000.0_f64;
        let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(AntoineCoefficients::new(
            ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(AntoineCoefficients::new(
            ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
    }

    fn build_demo_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-mixer-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed-a",
                "Feed A",
                300.0,
                120_000.0,
                2.0,
                binary_composition(0.25, 0.75),
            ),
            build_stream(
                "stream-feed-b",
                "Feed B",
                360.0,
                100_000.0,
                3.0,
                binary_composition(0.60, 0.40),
            ),
            build_stream(
                "stream-mix-out",
                "Mixer Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet.insert_stream(stream).expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-a", "Feed A", "stream-feed-a"),
            build_feed_node("feed-b", "Feed B", "stream-feed-b"),
            build_mixer_node(
                "mixer-1",
                "Mixer",
                "stream-feed-a",
                "stream-feed-b",
                "stream-mix-out",
            ),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-mix-out",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    #[test]
    fn sequential_solver_solves_feed_mixer_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_demo_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.steps.len(), 4);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-a");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "feed-b");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "mixer-1");
        assert_eq!(snapshot.steps[3].unit_id.as_str(), "flash-1");

        let mixer_out = snapshot
            .stream(&"stream-mix-out".into())
            .expect("expected mixer outlet");
        assert_close(mixer_out.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(mixer_out.temperature_k, 336.0, 1e-12);
        assert_close(
            *mixer_out
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.46,
            1e-12,
        );

        let liquid = snapshot
            .stream(&"stream-liquid".into())
            .expect("expected liquid outlet");
        let vapor = snapshot
            .stream(&"stream-vapor".into())
            .expect("expected vapor outlet");
        assert_close(liquid.total_molar_flow_mol_s, 3.099999999994907, 1e-9);
        assert_close(vapor.total_molar_flow_mol_s, 1.900000000005093, 1e-9);
        assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
        assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    }

    #[test]
    fn sequential_solver_runs_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-mixer-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.steps.len(), 4);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }
}
