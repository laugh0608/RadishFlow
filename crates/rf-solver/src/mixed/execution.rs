use super::*;
use crate::{
    equation_oriented::{
        EquationBlockInput, EquationControl, EquationFailureKind, EquationProgress,
        GlobalMaterialEvidence,
    },
    fixed_tp_material as material,
    local_services::{CountedFlash, CountedThermo},
    partitioned::{EvaluationPhase, LocalUnitEvaluation},
};
use rf_model::{MaterialStreamState, UnitNode};
use rf_types::{ComponentId, PortDirection, StreamId};
use rf_unitops::{UnitOperation, UnitOperationInputs, UnitOperationServices};
use std::{cell::Cell, ops::ControlFlow, time::Instant};

struct Run<'a> {
    control: &'a mut dyn MixedControl,
    progress: MixedProgress,
    streams: BTreeMap<StreamId, MaterialStreamState>,
    blocks: Vec<MixedBlockReport>,
    boundaries: Vec<MixedBoundaryEvidence>,
    active_units: Vec<LocalUnitEvaluation>,
    equation_failure: Option<Box<crate::equation_oriented::EquationFailure>>,
    costs: MixedCosts,
    kind: MixedFailureKind,
    assumed_tear: Option<MaterialStreamState>,
    recycle: Option<MixedRecycleReport>,
    serial_recycles: Vec<MixedRecycleRegionReport>,
}
impl Run<'_> {
    fn at(&mut self, stage: MixedStage) -> RfResult<()> {
        self.progress.stage = stage;
        self.progress.equation = None;
        if self.control.checkpoint(self.progress.clone()).is_break() {
            self.kind = MixedFailureKind::Cancelled;
            return Err(failure("cancelled", "mixed execution cancelled"));
        }
        Ok(())
    }
}
impl EquationControl for Run<'_> {
    fn checkpoint(&mut self, p: EquationProgress) -> ControlFlow<()> {
        self.progress.stage = MixedStage::Equation;
        self.progress.unit = p.unit.clone();
        self.progress.equation = Some(p);
        self.control.checkpoint(self.progress.clone())
    }
}
pub(super) fn solve(
    services: &crate::SolverServices<'_>,
    flow: &Flowsheet,
    options: &MixedOptions,
    control: &mut impl MixedControl,
    factory: &mut impl FnMut(
        &UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> RfResult<Box<dyn UnitOperation>>,
) -> Result<MixedResult, Box<MixedFailure>> {
    let start = Instant::now();
    let thermo = CountedThermo {
        provider: services.thermo,
        calls: Cell::new(0),
    };
    let flash = CountedFlash {
        solver: services.flash_solver,
        calls: Cell::new(0),
    };
    let counted = crate::SolverServices {
        thermo: &thermo,
        flash_solver: &flash,
    };
    let mut run = Run {
        control,
        progress: MixedProgress {
            stage: MixedStage::Assembly,
            block: None,
            unit: None,
            equation: None,
            recycle: None,
            recycle_region: None,
        },
        streams: BTreeMap::new(),
        blocks: vec![],
        boundaries: vec![],
        active_units: vec![],
        equation_failure: None,
        costs: MixedCosts::default(),
        kind: MixedFailureKind::Assembly,
        assumed_tear: None,
        recycle: None,
        serial_recycles: vec![],
    };
    let outcome = (|| {
        run.at(MixedStage::Assembly)?;
        let plan = assemble(flow, options)?;
        run.costs.assembly_elapsed = start.elapsed();
        let components: Vec<_> = flow.components.keys().cloned().collect();
        let expected_feeds: Vec<_> = flow
            .units
            .values()
            .filter(|u| u.kind == "feed")
            .map(|u| rf_unitops::catalog::effective_feed_boundary(u, &flow.streams))
            .collect::<RfResult<_>>()?;
        let fresh_scale = expected_feeds
            .iter()
            .map(|s| s.total_molar_flow_mol_s)
            .sum::<f64>();
        if !fresh_scale.is_finite() || fresh_scale <= 0.0 {
            return Err(failure(
                "fresh_total",
                "positive finite Feed total required",
            ));
        }
        for s in &expected_feeds {
            material::component_flows(&components, s, options.boundary)?;
        }

        if let Some(serial) = &plan.serial_recycle {
            execute_serial(
                &mut run,
                &plan,
                &serial.regions,
                &components,
                fresh_scale,
                &counted,
                factory,
            )?;
        } else {
            let before = plan
                .recycle
                .as_ref()
                .map_or(plan.ordered.len(), |r| r.cycle.start);
            for block in &plan.ordered[..before] {
                execute_block(
                    &mut run,
                    &plan,
                    block,
                    &components,
                    MaterialScale::Legacy(fresh_scale),
                    &counted,
                    factory,
                )?;
            }
            if let Some(recycle) = &plan.recycle {
                execute_recycle(
                    &mut run,
                    &plan,
                    recycle,
                    &components,
                    None,
                    &counted,
                    factory,
                )?;
                for block in &plan.ordered[recycle.cycle.end..] {
                    execute_block(
                        &mut run,
                        &plan,
                        block,
                        &components,
                        MaterialScale::Legacy(fresh_scale),
                        &counted,
                        factory,
                    )?;
                }
            }
        }
        run.kind = MixedFailureKind::GlobalVerification;
        run.progress.block = None;
        run.progress.unit = None;
        run.progress.recycle_region = None;
        run.at(MixedStage::GlobalVerification)?;
        let fresh: Vec<_> = flow
            .units
            .values()
            .filter(|u| u.kind == "feed")
            .flat_map(|u| {
                u.ports
                    .iter()
                    .filter_map(|p| p.stream_id.as_ref())
                    .map(|id| run.streams[id].clone())
            })
            .collect();
        let products: Vec<_> = plan
            .connections
            .iter()
            .filter(|c| c.sink.is_none())
            .map(|c| run.streams[&c.stream_id].clone())
            .collect();
        let total = fresh.iter().map(|s| s.total_molar_flow_mol_s).sum::<f64>();
        if !total.is_finite() || total <= 0.0 {
            return Err(failure("fresh_total", "positive total Feed required"));
        }
        let scales = components.iter().cloned().map(|c| (c, total)).collect();
        material::balance(&components, options.boundary, &fresh, &products, &scales)?;
        let global = material_evidence(&components, options.boundary, &fresh, &products)?;
        if !global.total_residual_mol_s.is_finite()
            || (global.total_residual_mol_s / total).abs()
                > crate::equation_oriented::SCALED_TOLERANCE
        {
            return Err(failure(
                "global_total",
                "global scaled total balance failed",
            ));
        }
        run.at(MixedStage::Completing)?;
        Ok(global)
    })();
    run.costs.thermo_calls = thermo.calls.get();
    run.costs.flash_calls = flash.calls.get();
    run.costs.elapsed = start.elapsed();
    match outcome {
        Ok(global_material) => Ok(MixedResult {
            method: options.method(),
            streams: run.streams,
            blocks: run.blocks,
            boundaries: run.boundaries,
            global_material,
            costs: run.costs,
            recycle: run.recycle,
            serial_recycles: run.serial_recycles,
        }),
        Err(error) => Err(Box::new(MixedFailure {
            method: options.method(),
            kind: run.kind,
            error,
            progress: run.progress,
            completed_blocks: run.blocks,
            boundaries: run.boundaries,
            active_units: run.active_units,
            equation_failure: run.equation_failure,
            costs: run.costs,
            recycle: run.recycle,
            serial_recycles: run.serial_recycles,
        })),
    }
}

#[derive(Clone, Copy)]
enum MaterialScale<'a> {
    Legacy(f64),
    Serial(f64),
    Region {
        global: f64,
        total: f64,
        inlet: &'a StreamId,
    },
}

impl MaterialScale<'_> {
    fn unit(self, incoming: f64) -> f64 {
        match self {
            Self::Legacy(fresh) => {
                if incoming > 0.0 {
                    incoming
                } else {
                    fresh
                }
            }
            Self::Serial(global) => global,
            Self::Region { total, .. } => total,
        }
    }

    fn boundary(self, stream: &StreamId, incoming: f64) -> f64 {
        match self {
            Self::Region { inlet, global, .. } if stream == inlet => global,
            _ => self.unit(incoming),
        }
    }
}

fn evaluated_feed_total(
    plan: &MixedPlan,
    streams: &BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<f64> {
    let total = plan
        .connections
        .iter()
        .filter(|c| plan.flow.units[&c.source.unit_id].kind == "feed")
        .try_fold(0.0, |total, c| {
            let stream = streams.get(&c.stream_id).ok_or_else(|| {
                failure("missing_boundary", "evaluated Feed output is missing")
                    .with_related_stream_id(c.stream_id.clone())
            })?;
            Ok(total + stream.total_molar_flow_mol_s)
        })?;
    if !total.is_finite() || total <= 0.0 {
        return Err(failure(
            "fresh_total",
            "positive finite evaluated Feed total required",
        ));
    }
    Ok(total)
}

fn material_evidence(
    components: &[ComponentId],
    boundary: FixedRecycleBoundary,
    inputs: &[MaterialStreamState],
    outputs: &[MaterialStreamState],
) -> RfResult<GlobalMaterialEvidence> {
    let totals = |states: &[MaterialStreamState]| -> RfResult<BTreeMap<ComponentId, f64>> {
        let mut values: BTreeMap<_, _> = components.iter().cloned().map(|c| (c, 0.0)).collect();
        for s in states {
            for (c, v) in material::component_flows(components, s, boundary)? {
                *values.get_mut(&c).expect("component") += v;
            }
        }
        Ok(values)
    };
    let inputs_total = totals(inputs)?;
    let outputs_total = totals(outputs)?;
    let total_flow = |states: &[MaterialStreamState]| {
        states.iter().map(|s| s.total_molar_flow_mol_s).sum::<f64>()
    };
    Ok(GlobalMaterialEvidence {
        component_residuals_mol_s: material::residuals(components, &inputs_total, &outputs_total),
        fresh_components_mol_s: inputs_total,
        product_components_mol_s: outputs_total,
        total_residual_mol_s: total_flow(outputs) - total_flow(inputs),
    })
}

fn execute_serial(
    run: &mut Run<'_>,
    plan: &MixedPlan,
    regions: &[serial_recycle::SerialRecycleRegion; 2],
    components: &[ComponentId],
    fresh_scale: f64,
    services: &crate::SolverServices<'_>,
    factory: &mut impl FnMut(
        &UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> RfResult<Box<dyn UnitOperation>>,
) -> RfResult<()> {
    let mut cursor = 0;
    let mut global_scale = fresh_scale;
    for region in regions {
        let recycle = &region.recycle;
        for block in &plan.ordered[cursor..recycle.cycle.start] {
            execute_block(
                run,
                plan,
                block,
                components,
                MaterialScale::Serial(global_scale),
                services,
                factory,
            )?;
            // The first Feed alone cannot establish the global scale. Once both
            // are evaluated, the transfer and second region use their real total.
            if cursor != 0
                && plan.flow.units[block.members.first().expect("local supply block")].kind
                    == "feed"
            {
                global_scale = evaluated_feed_total(plan, &run.streams)?;
            }
        }
        run.progress.recycle_region = Some(region.equation_block.clone());
        run.progress.block = Some(region.equation_block.clone());
        run.progress.unit = None;
        let outcome = execute_recycle(
            run,
            plan,
            recycle,
            components,
            Some(global_scale),
            services,
            factory,
        );
        if let Some(report) = run.recycle.take() {
            run.serial_recycles.push(MixedRecycleRegionReport {
                equation_block: region.equation_block.clone(),
                recycle: report,
                balance: None,
            });
        }
        outcome?;
        run.kind = MixedFailureKind::RecycleVerification;
        run.progress.block = Some(region.equation_block.clone());
        run.progress.unit = None;
        run.at(MixedStage::RegionVerification)?;
        let report = run
            .serial_recycles
            .last_mut()
            .expect("verified tear evidence");
        let input = std::slice::from_ref(&run.streams[&recycle.fresh_stream]);
        let outputs = region
            .outlets
            .each_ref()
            .map(|stream| run.streams[stream].clone());
        let boundary = plan.options.boundary;
        report.balance = Some(material_evidence(components, boundary, input, &outputs)?);
        material::balance(
            components,
            boundary,
            input,
            &outputs,
            &report.recycle.component_scales_mol_s,
        )?;
        run.progress.recycle_region = None;
        cursor = recycle.cycle.end;
    }
    Ok(())
}

fn execute_recycle(
    run: &mut Run<'_>,
    plan: &MixedPlan,
    recycle: &recycle::MixedRecyclePlan,
    components: &[ComponentId],
    serial_global_scale: Option<f64>,
    services: &crate::SolverServices<'_>,
    factory: &mut impl FnMut(
        &UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> RfResult<Box<dyn UnitOperation>>,
) -> RfResult<()> {
    let options = &recycle.options;
    let tear = &options.tear;
    let boundary = plan.options.boundary;
    run.kind = MixedFailureKind::Boundary;
    let inlet_error = || {
        let (code, message) = if serial_global_scale.is_some() {
            (
                "serial_inlet",
                "serial recycle requires a positive evaluated region inlet",
            )
        } else {
            (
                "tear_fresh_feed",
                "tear initialization requires a positive evaluated Feed at inlet_a",
            )
        };
        recycle::tear_error(tear, code, message)
            .with_related_stream_id(recycle.fresh_stream.clone())
    };
    let fresh = run
        .streams
        .get(&recycle.fresh_stream)
        .filter(|state| {
            state.total_molar_flow_mol_s.is_finite() && state.total_molar_flow_mol_s > 0.0
        })
        .cloned()
        .ok_or_else(inlet_error)?;
    let (fresh_total, block_scale) = if let Some(global) = serial_global_scale {
        let total = fresh.total_molar_flow_mol_s;
        (
            total,
            MaterialScale::Region {
                global,
                total,
                inlet: &recycle.fresh_stream,
            },
        )
    } else {
        let total = evaluated_feed_total(plan, &run.streams)?;
        (total, MaterialScale::Legacy(total))
    };
    let scales: BTreeMap<_, _> = components
        .iter()
        .cloned()
        .map(|id| (id, fresh_total))
        .collect();
    let mut values = tear
        .initial_component_flows_mol_s
        .clone()
        .unwrap_or_else(|| components.iter().cloned().map(|id| (id, 0.0)).collect());
    run.recycle = Some(MixedRecycleReport {
        tear: tear.clone(),
        fresh_stream: fresh.id.clone(),
        max_iteration_evaluations: options.max_iteration_evaluations,
        component_scales_mol_s: scales.clone(),
        residual_trace: Vec::new(),
        last_sweep: None,
    });
    let cycle = &plan.ordered[recycle.cycle.clone()];
    let cycle_members: BTreeSet<_> = cycle.iter().flat_map(|b| b.members.iter()).collect();
    let cycle_outputs: BTreeSet<_> = plan
        .connections
        .iter()
        .filter(|c| cycle_members.contains(&c.source.unit_id))
        .map(|c| &c.stream_id)
        .collect();
    let completed_before = run.blocks.len();
    let boundaries_before = run.boundaries.len();
    let mut sweep = |run: &mut Run<'_>, values: &BTreeMap<ComponentId, f64>, phase, evaluation| {
        run.progress.block = None;
        run.progress.unit = None;
        run.progress.recycle = Some(MixedRecycleProgress { phase, evaluation });
        run.at(MixedStage::BlockStarted)?;
        run.kind = MixedFailureKind::Boundary;
        run.blocks.truncate(completed_before);
        run.boundaries.truncate(boundaries_before);
        run.active_units.clear();
        run.streams.retain(|id, _| !cycle_outputs.contains(id));
        let assumed = material::tear_stream(
            &plan.flow.streams[&tear.stream_id],
            &fresh,
            boundary,
            values,
        )?;
        run.assumed_tear = Some(assumed);
        for block in cycle {
            execute_block(run, plan, block, components, block_scale, services, factory)?;
        }
        let injected = run.assumed_tear.take().expect("sweep input");
        let consumed = run.blocks[completed_before..]
            .iter()
            .flat_map(|b| b.evidence.final_units())
            .find(|u| u.unit_id == tear.sink.unit_id)
            .and_then(|u| u.consumed_streams.iter().find(|s| s.id == tear.stream_id))
            .ok_or_else(|| {
                recycle::tear_error(
                    tear,
                    "tear_consumed",
                    "actual tear consumer input is missing",
                )
            })?
            .clone();
        let returned = run
            .streams
            .get(&tear.stream_id)
            .ok_or_else(|| {
                recycle::tear_error(
                    tear,
                    "tear_returned",
                    "actual tear producer output is missing",
                )
            })?
            .clone();
        let consumed_values = material::component_flows(components, &consumed, boundary)?;
        let injected_values = material::component_flows(components, &injected, boundary)?;
        if consumed_values != injected_values {
            return Err(recycle::tear_error(
                tear,
                "tear_consumed",
                "tear consumer did not use the injected boundary",
            ));
        }
        let returned_values = material::component_flows(components, &returned, boundary)?;
        let residuals = material::residuals(components, &consumed_values, &returned_values);
        let converged = recycle::record_sweep(
            run.recycle.as_mut().expect("initialized report"),
            consumed,
            returned,
            residuals,
            phase,
            evaluation,
        )?;
        run.at(MixedStage::Boundary)?;
        Ok::<_, RfError>((converged, returned_values))
    };
    for evaluation in 1..=options.max_iteration_evaluations {
        let (candidate, returned) = sweep(run, &values, EvaluationPhase::Iteration, evaluation)?;
        values = returned;
        if !candidate {
            continue;
        }
        let (verified, _) = sweep(run, &values, EvaluationPhase::Verification, 1)?;
        if !verified {
            run.kind = MixedFailureKind::RecycleVerification;
            return Err(recycle::tear_error(
                tear,
                "tear_verification",
                "final sweep did not meet both closure tolerances",
            ));
        }
        let connection = plan
            .connections
            .iter()
            .find(|c| c.stream_id == tear.stream_id)
            .expect("validated tear");
        let final_sweep = run
            .recycle
            .as_ref()
            .and_then(|r| r.last_sweep.as_ref())
            .expect("verified sweep");
        run.boundaries.push(MixedBoundaryEvidence {
            connection: connection.clone(),
            producer: final_sweep.returned.clone(),
            consumed: final_sweep.assumed.clone(),
            residuals_mol_s: final_sweep.residuals_mol_s.clone(),
            scaled_residuals: final_sweep
                .residuals_mol_s
                .iter()
                .map(|(id, value)| (id.clone(), value / scales[id]))
                .collect(),
        });
        run.progress.recycle = None;
        return Ok(());
    }
    run.kind = MixedFailureKind::IterationLimit;
    Err(recycle::tear_error(
        tear,
        "iteration_limit",
        "direct substitution exhausted its evaluation budget",
    ))
}

fn execute_block(
    run: &mut Run<'_>,
    plan: &MixedPlan,
    block: &MixedBlock,
    components: &[ComponentId],
    material_scale: MaterialScale<'_>,
    services: &crate::SolverServices<'_>,
    factory: &mut impl FnMut(
        &UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> RfResult<Box<dyn UnitOperation>>,
) -> RfResult<()> {
    let flow = &plan.flow;
    let options = &plan.options;
    let block_start = Instant::now();
    run.progress.block = Some(block.id.clone());
    run.progress.unit = None;
    run.active_units.clear();
    run.at(MixedStage::BlockStarted)?;
    run.kind = MixedFailureKind::Boundary;
    let incoming: BTreeMap<_, _> = plan
        .connections
        .iter()
        .filter(|c| {
            !block.members.contains(&c.source.unit_id)
                && c.sink
                    .as_ref()
                    .is_some_and(|s| block.members.contains(&s.unit_id))
        })
        .map(|c| {
            run.assumed_tear
                .as_ref()
                .filter(|s| s.id == c.stream_id)
                .or_else(|| run.streams.get(&c.stream_id))
                .cloned()
                .map(|s| (c.stream_id.clone(), s))
                .ok_or_else(|| {
                    failure("missing_boundary", "upstream block produced no boundary")
                        .with_related_stream_id(c.stream_id.clone())
                })
        })
        .collect::<RfResult<_>>()?;
    let evidence = if let Some(o) = block.equation_options(options.boundary) {
        run.kind = MixedFailureKind::Equation;
        let input = EquationBlockInput {
            flowsheet: flow,
            members: &block.members,
            inputs: &incoming,
        };
        // The same factory hook serves actual Local and EO verification calls.
        let result =
            crate::equation_oriented::solve_block_with_factory(services, &input, &o, run, factory);
        let result = match result {
            Ok(r) => r,
            Err(e) => {
                run.costs.operation_constructions += e.costs.operation_constructions;
                run.costs.local_unit_calls += e.costs.local_unit_calls;
                for (id, n) in &e.costs.calls_by_unit {
                    *run.costs.calls_by_unit.entry(id.clone()).or_default() += n;
                }
                if e.kind == EquationFailureKind::Cancelled {
                    run.kind = MixedFailureKind::Cancelled;
                }
                run.active_units = e.evidence.final_units.clone();
                let error = e.error.clone();
                run.equation_failure = Some(e);
                return Err(error);
            }
        };
        run.costs.operation_constructions += result.costs.operation_constructions;
        run.costs.local_unit_calls += result.costs.local_unit_calls;
        for (id, n) in &result.costs.calls_by_unit {
            *run.costs.calls_by_unit.entry(id.clone()).or_default() += n;
        }
        run.active_units = result.evidence.final_units.clone();
        MixedBlockEvidence::EquationOriented(Box::new(result))
    } else {
        run.kind = MixedFailureKind::LocalEvaluation;
        let unit = &flow.units[block.members.first().expect("local member")];
        run.progress.unit = Some(unit.id.clone());
        let mut inputs = UnitOperationInputs::new();
        let mut consumed = vec![];
        for port in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Inlet)
        {
            let id = port.stream_id.as_ref().expect("validated connection");
            let input = incoming
                .get(id)
                .ok_or_else(|| failure("missing_boundary", "local inlet has no upstream value"))?
                .clone();
            material::component_flows(components, &input, options.boundary)?;
            inputs.insert_material_stream(port.name.clone(), input.clone());
            consumed.push(input);
        }
        crate::parameter_checks::validate_step_parameters(
            1,
            unit,
            &inputs,
            &consumed.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        )?;
        run.costs.operation_constructions += 1;
        let operation = factory(unit, &flow.streams)?;
        run.costs.local_unit_calls += 1;
        *run.costs.calls_by_unit.entry(unit.id.clone()).or_default() += 1;
        let output = operation.run(
            &UnitOperationServices {
                thermo: Some(services.thermo),
                flash_solver: Some(services.flash_solver),
            },
            &inputs,
        )?;
        let mut produced = vec![];
        for port in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Outlet)
        {
            let s = output
                .stream(&port.name)
                .ok_or_else(|| {
                    failure("missing_output", "local output missing")
                        .with_related_unit_id(unit.id.clone())
                })?
                .clone();
            if Some(&s.id) != port.stream_id.as_ref() {
                return Err(failure(
                    "output_identity",
                    "output identity differs from engineering port",
                )
                .with_related_unit_id(unit.id.clone()));
            }
            material::component_flows(components, &s, options.boundary)?;
            produced.push(s);
        }
        let local = LocalUnitEvaluation {
            unit_id: unit.id.clone(),
            consumed_streams: consumed,
            produced_streams: produced,
        };
        run.active_units.push(local.clone());
        let input_total = local
            .consumed_streams
            .iter()
            .map(|s| s.total_molar_flow_mol_s)
            .sum::<f64>();
        let scale = material_scale.unit(input_total);
        let scales = components.iter().cloned().map(|c| (c, scale)).collect();
        if unit.kind == "feed" {
            let expected = rf_unitops::catalog::effective_feed_boundary(unit, &flow.streams)?;
            material::balance(
                components,
                options.boundary,
                &[expected],
                &local.produced_streams,
                &scales,
            )?;
        } else {
            material::balance(
                components,
                options.boundary,
                &local.consumed_streams,
                &local.produced_streams,
                &scales,
            )?;
        }
        MixedBlockEvidence::Local(local)
    };
    run.kind = MixedFailureKind::Boundary;
    run.at(MixedStage::Boundary)?;
    for connection in plan.connections.iter().filter(|c| {
        c.sink
            .as_ref()
            .is_some_and(|s| block.members.contains(&s.unit_id))
            && !block.members.contains(&c.source.unit_id)
    }) {
        if run
            .assumed_tear
            .as_ref()
            .is_some_and(|s| s.id == connection.stream_id)
        {
            continue;
        }
        let producer = &incoming[&connection.stream_id];
        let sink = connection.sink.as_ref().expect("incoming sink");
        let consumed = run
            .active_units
            .iter()
            .find(|u| u.unit_id == sink.unit_id)
            .and_then(|u| {
                u.consumed_streams
                    .iter()
                    .find(|s| s.id == connection.stream_id)
            })
            .ok_or_else(|| failure("consumed_boundary", "actual consumer boundary missing"))?;
        let produced = material::component_flows(components, producer, options.boundary)?;
        let used = material::component_flows(components, consumed, options.boundary)?;
        let incoming_total = incoming
            .values()
            .map(|s| s.total_molar_flow_mol_s)
            .sum::<f64>();
        let total = material_scale.boundary(&connection.stream_id, incoming_total);
        if !total.is_finite() || total <= 0.0 {
            return Err(failure("zero_scale", "boundary total must be positive"));
        }
        let residuals = material::residuals(components, &produced, &used);
        let scaled = residuals
            .iter()
            .map(|(c, r)| (c.clone(), r / total))
            .collect();
        run.boundaries.push(MixedBoundaryEvidence {
            connection: connection.clone(),
            producer: producer.clone(),
            consumed: consumed.clone(),
            residuals_mol_s: residuals.clone(),
            scaled_residuals: scaled,
        });
        let scales = components.iter().cloned().map(|c| (c, total)).collect();
        if !material::converged(&material::sample(
            &residuals,
            &scales,
            crate::partitioned::EvaluationPhase::Verification,
            0,
        )?) {
            return Err(failure(
                "boundary_residual",
                "cross-block residual exceeds tolerance",
            ));
        }
    }
    for local in &run.active_units {
        for s in &local.produced_streams {
            if run.streams.insert(s.id.clone(), s.clone()).is_some() {
                return Err(failure("duplicate_producer", "stream produced twice")
                    .with_related_stream_id(s.id.clone()));
            }
        }
    }
    run.blocks.push(MixedBlockReport {
        id: block.id.clone(),
        members: block.members.clone(),
        evidence,
        elapsed: block_start.elapsed(),
    });
    run.at(MixedStage::BlockCompleted)?;
    Ok(())
}
