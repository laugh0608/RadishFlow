use super::*;
use crate::local_services::{CountedFlash, CountedThermo};
use rf_types::PortDirection;
use rf_unitops::{
    UnitOperation, UnitOperationInputs, UnitOperationServices, catalog::instantiate_builtin,
};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::time::Instant;

struct RunError {
    kind: PartitionedFailureKind,
    error: RfError,
}
type RunResult<T> = Result<T, RunError>;
fn classified<T>(kind: PartitionedFailureKind, result: RfResult<T>) -> RunResult<T> {
    result.map_err(|error| RunError { kind, error })
}

struct TrialExecution<'a, 'b> {
    plan: &'a PartitionedSolvePlan,
    services: UnitOperationServices<'a>,
    operations: BTreeMap<UnitId, Box<dyn UnitOperation>>,
    control: &'b mut dyn PartitionedSolverControl,
    streams: BTreeMap<StreamId, MaterialStreamState>,
    blocks: Vec<BlockReport>,
    active_recycle: Option<RecycleBlockReport>,
    active_units: Vec<LocalUnitEvaluation>,
    costs: SolveCosts,
    block_index: usize,
    progress: PartitionedProgress,
    global_scales: BTreeMap<ComponentId, f64>,
}

pub(super) fn solve(
    services: &SolverServices<'_>,
    flowsheet: &Flowsheet,
    options: &RecycleSolveOptions,
    control: &mut impl PartitionedSolverControl,
) -> Result<PartitionedSolveResult, Box<PartitionedSolveFailure>> {
    let started = Instant::now();
    let initial_progress = PartitionedProgress {
        evaluation: None,
        residual: None,
        phase: None,
        stage: PartitionedStage::AssemblyStarted,
        block: None,
        unit: None,
    };
    let fail_early = |kind, error, assembly_elapsed, stage| {
        Box::new(PartitionedSolveFailure {
            kind,
            error,
            completed_blocks: Vec::new(),
            active_recycle: None,
            active_units: Vec::new(),
            stage: PartitionedProgress {
                evaluation: None,
                residual: None,
                phase: None,
                stage,
                block: None,
                unit: None,
            },
            costs: SolveCosts {
                assembly_elapsed,
                elapsed: started.elapsed(),
                ..Default::default()
            },
        })
    };
    if control.checkpoint(initial_progress).is_break() {
        return Err(fail_early(
            PartitionedFailureKind::Cancelled,
            failure("cancelled", "cancelled before assembly"),
            Duration::ZERO,
            PartitionedStage::AssemblyStarted,
        ));
    }
    let plan = assemble_partitioned_plan(flowsheet, options).map_err(|error| {
        fail_early(
            PartitionedFailureKind::Assembly,
            error,
            started.elapsed(),
            PartitionedStage::AssemblyStarted,
        )
    })?;
    let assembly_elapsed = started.elapsed();
    if control
        .checkpoint(PartitionedProgress {
            evaluation: None,
            residual: None,
            phase: None,
            stage: PartitionedStage::AssemblyCompleted,
            block: None,
            unit: None,
        })
        .is_break()
    {
        return Err(fail_early(
            PartitionedFailureKind::Cancelled,
            failure("cancelled", "cancelled after assembly"),
            assembly_elapsed,
            PartitionedStage::AssemblyCompleted,
        ));
    }
    let thermo = CountedThermo {
        provider: services.thermo,
        calls: Cell::new(0),
    };
    let flash = CountedFlash {
        solver: services.flash_solver,
        calls: Cell::new(0),
    };
    let mut trial = TrialExecution {
        plan: &plan,
        services: UnitOperationServices {
            thermo: Some(&thermo),
            flash_solver: Some(&flash),
        },
        operations: BTreeMap::new(),
        control,
        streams: BTreeMap::new(),
        blocks: Vec::new(),
        active_recycle: None,
        active_units: Vec::new(),
        costs: SolveCosts {
            assembly_elapsed,
            ..Default::default()
        },
        block_index: 0,
        progress: PartitionedProgress {
            evaluation: None,
            residual: None,
            phase: None,
            stage: PartitionedStage::Initializing,
            block: None,
            unit: None,
        },
        global_scales: BTreeMap::new(),
    };
    let initializing = Instant::now();
    let initialization = trial.initialize();
    trial.costs.initialization_elapsed = initializing.elapsed();
    let outcome = initialization.and_then(|()| trial.execute());
    trial.costs.thermo_calls = thermo.calls.get();
    trial.costs.flash_calls = flash.calls.get();
    trial.costs.elapsed = started.elapsed();
    match outcome {
        Ok(()) => Ok(PartitionedSolveResult {
            streams: trial.streams,
            blocks: trial.blocks,
            costs: trial.costs,
        }),
        Err(error) => Err(Box::new(PartitionedSolveFailure {
            kind: error.kind,
            error: error.error,
            completed_blocks: trial.blocks,
            active_recycle: trial.active_recycle,
            active_units: trial.active_units,
            stage: trial.progress,
            costs: trial.costs,
        })),
    }
}

impl TrialExecution<'_, '_> {
    fn checkpoint(&mut self, stage: PartitionedStage, unit: Option<UnitId>) -> RunResult<()> {
        let phase = match stage {
            PartitionedStage::IterationStarted | PartitionedStage::IterationCompleted => {
                Some(EvaluationPhase::Iteration)
            }
            PartitionedStage::VerificationStarted | PartitionedStage::VerificationCompleted => {
                Some(EvaluationPhase::Verification)
            }
            PartitionedStage::UnitStarted | PartitionedStage::UnitCompleted => self.progress.phase,
            _ => None,
        };
        self.progress = PartitionedProgress {
            evaluation: match stage {
                PartitionedStage::IterationStarted => Some(
                    self.active_recycle
                        .as_ref()
                        .map_or(1, |r| r.iteration_evaluations + 1),
                ),
                PartitionedStage::IterationCompleted => self
                    .active_recycle
                    .as_ref()
                    .map(|r| r.iteration_evaluations),
                PartitionedStage::VerificationStarted | PartitionedStage::VerificationCompleted => {
                    Some(1)
                }
                PartitionedStage::UnitStarted | PartitionedStage::UnitCompleted => {
                    self.progress.evaluation
                }
                _ => None,
            },
            residual: self
                .active_recycle
                .as_ref()
                .and_then(|r| r.residual_trace.last().cloned()),
            phase,
            stage,
            block: if matches!(stage, PartitionedStage::Initializing) {
                None
            } else {
                Some(self.block_index)
            },
            unit,
        };
        if self.control.checkpoint(self.progress.clone()).is_break() {
            let mut error = failure("cancelled", format!("cancelled at {stage:?}"));
            if let Some(unit) = &self.progress.unit {
                error = error.with_related_unit_id(unit.clone());
            }
            return Err(RunError {
                kind: PartitionedFailureKind::Cancelled,
                error,
            });
        }
        Ok(())
    }

    fn initialize(&mut self) -> RunResult<()> {
        self.checkpoint(PartitionedStage::Initializing, None)?;
        let service_components: BTreeSet<_> = self
            .services
            .thermo
            .expect("supplied thermo")
            .system()
            .components
            .iter()
            .map(|c| c.id.clone())
            .collect();
        if service_components.iter().ne(self.plan.components.iter())
            || service_components.len()
                != self
                    .services
                    .thermo
                    .expect("supplied thermo")
                    .system()
                    .components
                    .len()
        {
            return Err(RunError {
                kind: PartitionedFailureKind::Initialization,
                error: failure(
                    "thermo_component_binding",
                    "thermo and engineering component registries must match",
                ),
            });
        }
        let mut fresh_total = 0.0;
        let mut templates = self.plan.flowsheet.streams.clone();
        for unit in self.plan.flowsheet.units.values() {
            if unit.kind == "feed" {
                let id = classified(
                    PartitionedFailureKind::Initialization,
                    crate::port_stream_id(unit, "outlet"),
                )?;
                let stream = templates.get_mut(id).expect("validated stream");
                if let Some(t) = unit.parameters.outlet_temperature_k {
                    stream.temperature_k = t;
                }
                if let Some(p) = unit.parameters.outlet_pressure_pa {
                    stream.pressure_pa = p;
                }
                // Derived phases/windows in a project template are not qualified
                // for this execution; Feed will compute its own current intensities.
                stream.phases.clear();
                stream.bubble_dew_window = None;
                classified(
                    PartitionedFailureKind::Initialization,
                    checks::component_flows(
                        &self.plan.components,
                        stream,
                        self.plan.options.boundary,
                    ),
                )?;
                // Expand absent registered components to explicit zeros only in
                // this worker input; legacy local run still receives a full registry.
                for component in &self.plan.components {
                    stream
                        .overall_mole_fractions
                        .entry(component.clone())
                        .or_insert(0.0);
                }
                fresh_total += stream.total_molar_flow_mol_s;
            }
            self.costs.operation_constructions += 1;
            let operation = classified(
                PartitionedFailureKind::Initialization,
                instantiate_builtin(unit, &templates),
            )?;
            self.operations.insert(unit.id.clone(), operation);
        }
        if !fresh_total.is_finite() || fresh_total <= 0.0 {
            return Err(RunError {
                kind: PartitionedFailureKind::Initialization,
                error: failure(
                    "nonpositive_fresh_flow",
                    "fixed-boundary task requires positive finite fresh inflow",
                ),
            });
        }
        self.global_scales = self
            .plan
            .components
            .iter()
            .cloned()
            .map(|id| (id, fresh_total))
            .collect();
        Ok(())
    }

    fn execute(&mut self) -> RunResult<()> {
        for (index, block) in self.plan.blocks.iter().enumerate() {
            self.block_index = index;
            self.active_recycle = None;
            self.active_units.clear();
            self.checkpoint(PartitionedStage::BlockStarted, None)?;
            let report = if let Some(recycle) = &block.recycle {
                self.execute_recycle(block, recycle)?
            } else {
                let scales = self.global_scales.clone();
                let evaluated = self.evaluate_unit(&block.members[0], None, &scales)?;
                BlockReport {
                    members: block.members.clone(),
                    kind: BlockKind::Acyclic,
                    final_units: vec![evaluated],
                    recycle: None,
                }
            };
            self.blocks.push(report);
            self.active_recycle = None;
        }
        self.checkpoint(PartitionedStage::Completing, None)?;
        let feed_ids: BTreeSet<_> = self
            .plan
            .flowsheet
            .units
            .values()
            .filter(|u| u.kind == "feed")
            .map(|u| &u.id)
            .collect();
        let feeds: Vec<_> = self
            .plan
            .connections
            .iter()
            .filter(|c| feed_ids.contains(&c.source.unit_id))
            .map(|c| self.streams[&c.stream_id].clone())
            .collect();
        let products: Vec<_> = self
            .plan
            .connections
            .iter()
            .filter(|c| c.sink.is_none())
            .map(|c| self.streams[&c.stream_id].clone())
            .collect();
        classified(
            PartitionedFailureKind::Balance,
            checks::balance(self.plan, &feeds, &products, &self.global_scales),
        )?;
        Ok(())
    }

    fn evaluate_unit(
        &mut self,
        id: &UnitId,
        injection: Option<(&TearSpecification, &MaterialStreamState)>,
        scales: &BTreeMap<ComponentId, f64>,
    ) -> RunResult<LocalUnitEvaluation> {
        self.checkpoint(PartitionedStage::UnitStarted, Some(id.clone()))?;
        let unit = &self.plan.flowsheet.units[id];
        let mut inputs = UnitOperationInputs::new();
        let mut consumed = Vec::new();
        for port in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Inlet)
        {
            let stream_id = port.stream_id.as_ref().expect("validated binding");
            let stream = if let Some((tear, guess)) = injection
                && tear.sink.unit_id == *id
                && tear.sink.port_name == port.name
            {
                guess
            } else {
                classified(
                    PartitionedFailureKind::LocalEvaluation,
                    self.streams.get(stream_id).ok_or_else(|| {
                        failure(
                            "missing_upstream",
                            "input is not a source-owned output of this execution",
                        )
                        .with_related_stream_id(stream_id.clone())
                        .with_related_port_target(
                            DiagnosticPortTarget::new(id.clone(), port.name.clone()),
                        )
                    }),
                )?
            };
            classified(
                PartitionedFailureKind::Boundary,
                checks::component_flows(&self.plan.components, stream, self.plan.options.boundary)
                    .map_err(|e| {
                        e.with_related_unit_id(id.clone()).with_related_port_target(
                            DiagnosticPortTarget::new(id.clone(), port.name.clone()),
                        )
                    }),
            )?;
            consumed.push(stream.clone());
            inputs.insert_material_stream(&port.name, stream.clone());
        }
        let ids: Vec<_> = consumed.iter().map(|s| s.id.clone()).collect();
        classified(
            PartitionedFailureKind::LocalEvaluation,
            crate::parameter_checks::validate_step_parameters(1, unit, &inputs, &ids),
        )?;
        self.costs.local_unit_calls += 1;
        *self.costs.calls_by_unit.entry(id.clone()).or_default() += 1;
        let outputs = classified(
            PartitionedFailureKind::LocalEvaluation,
            self.operations[id]
                .run(&self.services, &inputs)
                .map_err(|mut error| {
                    error = error.with_related_unit_id(id.clone());
                    for port in &unit.ports {
                        error = error.with_related_port_target(DiagnosticPortTarget::new(
                            id.clone(),
                            port.name.clone(),
                        ));
                        if let Some(stream_id) = &port.stream_id {
                            error = error.with_related_stream_id(stream_id.clone());
                        }
                    }
                    error
                }),
        )?;
        let mut produced = Vec::new();
        for port in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Outlet)
        {
            let output = classified(
                PartitionedFailureKind::LocalEvaluation,
                outputs.stream(&port.name).ok_or_else(|| {
                    failure(
                        "missing_output",
                        "unit did not produce its canonical outlet",
                    )
                    .with_related_unit_id(id.clone())
                    .with_related_port_target(DiagnosticPortTarget::new(
                        id.clone(),
                        port.name.clone(),
                    ))
                }),
            )?;
            if Some(&output.id) != port.stream_id.as_ref() {
                return Err(RunError {
                    kind: PartitionedFailureKind::LocalEvaluation,
                    error: failure(
                        "output_identity",
                        "local output does not match its source port binding",
                    )
                    .with_related_unit_id(id.clone())
                    .with_related_stream_id(output.id.clone()),
                });
            }
            classified(
                PartitionedFailureKind::Boundary,
                checks::component_flows(&self.plan.components, output, self.plan.options.boundary)
                    .map_err(|e| {
                        e.with_related_unit_id(id.clone()).with_related_port_target(
                            DiagnosticPortTarget::new(id.clone(), port.name.clone()),
                        )
                    }),
            )?;
            produced.push(output.clone());
        }
        if unit.kind != "feed" {
            classified(
                PartitionedFailureKind::Balance,
                checks::balance(self.plan, &consumed, &produced, scales)
                    .map_err(|e| e.with_related_unit_id(id.clone())),
            )?;
        }
        for output in &produced {
            self.streams.insert(output.id.clone(), output.clone());
        }
        let evaluation = LocalUnitEvaluation {
            unit_id: id.clone(),
            consumed_streams: consumed,
            produced_streams: produced,
        };
        self.active_units.push(evaluation.clone());
        self.checkpoint(PartitionedStage::UnitCompleted, Some(id.clone()))?;
        Ok(evaluation)
    }

    fn execute_recycle(
        &mut self,
        block: &ExecutionBlock,
        recycle: &RecyclePlan,
    ) -> RunResult<BlockReport> {
        let fresh = classified(
            PartitionedFailureKind::Initialization,
            self.streams
                .get(&recycle.fresh_stream)
                .cloned()
                .ok_or_else(|| {
                    failure(
                        "missing_fresh",
                        "cyclic block has no evaluated fresh boundary",
                    )
                    .with_related_stream_id(recycle.fresh_stream.clone())
                }),
        )?;
        classified(
            PartitionedFailureKind::Boundary,
            checks::component_flows(&self.plan.components, &fresh, self.plan.options.boundary),
        )?;
        if fresh.total_molar_flow_mol_s <= 0.0 {
            return Err(RunError {
                kind: PartitionedFailureKind::Initialization,
                error: failure(
                    "nonpositive_fresh_flow",
                    "zero-flow guess requires positive fresh inflow",
                )
                .with_related_stream_id(fresh.id.clone()),
            });
        }
        let fraction = self.plan.flowsheet.units[&recycle.splitter]
            .parameters
            .outlet_a_split_fraction
            .expect("assembled fraction");
        if fraction == 1.0 {
            return Err(RunError {
                kind: PartitionedFailureKind::NoFiniteSteadyState,
                error: failure(
                    "no_finite_steady_state",
                    "unit split fraction with positive fresh inflow has no finite steady state",
                )
                .with_related_unit_id(recycle.splitter.clone())
                .with_related_stream_id(recycle.tear.stream_id.clone()),
            });
        }
        let scales = recycle
            .tear
            .component_scales_mol_s
            .clone()
            .unwrap_or_else(|| {
                self.plan
                    .components
                    .iter()
                    .cloned()
                    .map(|id| (id, fresh.total_molar_flow_mol_s))
                    .collect()
            });
        let mut values = recycle
            .tear
            .initial_component_flows_mol_s
            .clone()
            .unwrap_or_else(|| {
                self.plan
                    .components
                    .iter()
                    .cloned()
                    .map(|id| (id, 0.0))
                    .collect()
            });
        let mut trace = Vec::new();
        for iteration in 1..=self.plan.options.max_iteration_evaluations {
            self.checkpoint(PartitionedStage::IterationStarted, None)?;
            self.costs.iteration_evaluations += 1;
            let assumed = classified(
                PartitionedFailureKind::Boundary,
                checks::guess(self.plan, recycle, &fresh, &values),
            )?;
            let units = self.evaluate_loop(recycle, &assumed, &scales)?;
            let returned = self.streams[&recycle.tear.stream_id].clone();
            let returned_values = classified(
                PartitionedFailureKind::Boundary,
                checks::component_flows(
                    &self.plan.components,
                    &returned,
                    self.plan.options.boundary,
                ),
            )?;
            let residuals = checks::residuals(&self.plan.components, &values, &returned_values);
            let sample = classified(
                PartitionedFailureKind::Boundary,
                checks::sample(&residuals, &scales, EvaluationPhase::Iteration, iteration),
            )?;
            let candidate = checks::converged(&sample);
            trace.push(sample);
            self.active_recycle = Some(RecycleBlockReport {
                boundary: self.plan.options.boundary,
                zero_flow_composition_rule: ZeroFlowCompositionRule::EvaluatedFreshBoundary(
                    recycle.fresh_stream.clone(),
                ),
                tolerances: RecycleTolerances::for_components(self.plan.components.len()),
                tear: recycle.tear.clone(),
                iteration_evaluations: iteration,
                verification_evaluations: 0,
                component_scales_mol_s: scales.clone(),
                assumed,
                returned,
                residuals_mol_s: residuals,
                residual_trace: trace.clone(),
            });
            self.checkpoint(PartitionedStage::IterationCompleted, None)?;
            values = returned_values;
            if candidate {
                // Never use the candidate's states as the final result. Inject its
                // returned value and perform a separately counted real loop sweep.
                self.checkpoint(PartitionedStage::VerificationStarted, None)?;
                self.costs.verification_evaluations += 1;
                let assumed = classified(
                    PartitionedFailureKind::Verification,
                    checks::guess(self.plan, recycle, &fresh, &values),
                )?;
                let final_units = self.evaluate_loop(recycle, &assumed, &scales)?;
                let returned = self.streams[&recycle.tear.stream_id].clone();
                let returned_values = classified(
                    PartitionedFailureKind::Verification,
                    checks::component_flows(
                        &self.plan.components,
                        &returned,
                        self.plan.options.boundary,
                    ),
                )?;
                let residuals = checks::residuals(&self.plan.components, &values, &returned_values);
                let sample = classified(
                    PartitionedFailureKind::Verification,
                    checks::sample(&residuals, &scales, EvaluationPhase::Verification, 1),
                )?;
                let verified = checks::converged(&sample);
                trace.push(sample);
                self.active_recycle = Some(RecycleBlockReport {
                    boundary: self.plan.options.boundary,
                    zero_flow_composition_rule: ZeroFlowCompositionRule::EvaluatedFreshBoundary(
                        recycle.fresh_stream.clone(),
                    ),
                    tolerances: RecycleTolerances::for_components(self.plan.components.len()),
                    tear: recycle.tear.clone(),
                    iteration_evaluations: iteration,
                    verification_evaluations: 1,
                    component_scales_mol_s: scales.clone(),
                    assumed,
                    returned,
                    residuals_mol_s: residuals,
                    residual_trace: trace,
                });
                self.checkpoint(PartitionedStage::VerificationCompleted, None)?;
                if !verified {
                    return Err(RunError { kind: PartitionedFailureKind::Verification, error: failure("verification_failed", "independent final loop verification did not meet both residual tolerances").with_related_stream_id(recycle.tear.stream_id.clone()) });
                }
                let product = self.streams[&recycle.product_stream].clone();
                classified(
                    PartitionedFailureKind::Balance,
                    checks::balance(self.plan, &[fresh], &[product], &scales),
                )?;
                return Ok(BlockReport {
                    members: block.members.clone(),
                    kind: BlockKind::Recycle,
                    final_units,
                    recycle: self.active_recycle.take(),
                });
            }
            // Keep no full intermediate local-state history.
            drop(units);
        }
        Err(RunError {
            kind: PartitionedFailureKind::IterationLimit,
            error: failure(
                "iteration_limit",
                "direct substitution exhausted its evaluation budget",
            )
            .with_related_stream_id(recycle.tear.stream_id.clone())
            .with_related_unit_ids(block.members.clone()),
        })
    }

    fn evaluate_loop(
        &mut self,
        recycle: &RecyclePlan,
        assumed: &MaterialStreamState,
        scales: &BTreeMap<ComponentId, f64>,
    ) -> RunResult<Vec<LocalUnitEvaluation>> {
        self.active_units.clear();
        let mixer = self.evaluate_unit(&recycle.mixer, Some((&recycle.tear, assumed)), scales)?;
        let splitter = self.evaluate_unit(&recycle.splitter, None, scales)?;
        Ok(vec![mixer, splitter])
    }
}

#[cfg(test)]
mod faults;
