//! Fixed-pressure single-vapor recycle with one explicit material tear.
mod plan;

use crate::{
    SequentialProperties, UnitEnergyBalance, UnitSolveStep, evaluate_unit_step,
    fixed_tp_material::{converged, residuals, sample},
    flow_plan::SinglePhaseOptions,
    partitioned::{
        ABSOLUTE_FLOW_TOLERANCE, EvaluationPhase, MAX_ITERATION_EVALUATIONS, PartitionedProgress,
        PartitionedSolverControl, PartitionedStage, ResidualSample,
    },
    validate_heat_duty_targets,
};
use plan::Plan;
use rf_model::{Flowsheet, MaterialStreamState};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{ComponentId, DiagnosticPortTarget, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{
    BuiltinUnitKind, ENERGY_ABSOLUTE_TOLERANCE_W, ENERGY_SCALED_TOLERANCE, UnitOperation,
    catalog::{
        instantiate_single_phase_recycle_heat_duty, instantiate_single_phase_with_heat_duty,
    },
    component_balance::stream_component_molar_flows,
    materialize_single_phase_vapor, single_phase_enthalpy_flow_w,
    verify_single_phase_energy_balance,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinglePhaseTearBinding {
    pub stream_id: StreamId,
    pub source: DiagnosticPortTarget,
    pub sink: DiagnosticPortTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseTearState {
    pub component_flows_mol_s: BTreeMap<ComponentId, f64>,
    pub enthalpy_flow_w: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseRecycleOptions {
    pub tear: SinglePhaseTearBinding,
    /// None starts with zero component flows and zero enthalpy flow.
    pub initial_state: Option<SinglePhaseTearState>,
    pub max_iteration_evaluations: usize,
}

impl SinglePhaseRecycleOptions {
    pub fn new(tear: SinglePhaseTearBinding) -> Self {
        Self {
            tear,
            initial_state: None,
            max_iteration_evaluations: MAX_ITERATION_EVALUATIONS,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseRecycleResidual {
    pub material: ResidualSample,
    pub enthalpy_flow_residual_w: f64,
    pub scaled_enthalpy_flow_residual: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseTearEvaluation {
    pub assumed: MaterialStreamState,
    pub returned: MaterialStreamState,
    pub component_residuals_mol_s: BTreeMap<ComponentId, f64>,
    pub residual: SinglePhaseRecycleResidual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseRecycleResult {
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub final_units: Vec<UnitSolveStep>,
    pub tear: SinglePhaseTearBinding,
    pub verification: SinglePhaseTearEvaluation,
    pub trace: Vec<SinglePhaseRecycleResidual>,
    pub component_scale_mol_s: f64,
    pub tear_enthalpy_scale_w: f64,
    pub overall_energy_balance: UnitEnergyBalance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePhaseRecycleFailure {
    pub error: RfError,
    pub stage: PartitionedProgress,
    pub trace: Vec<SinglePhaseRecycleResidual>,
    pub last_evaluation: Option<SinglePhaseTearEvaluation>,
}

#[derive(Debug, Default)]
pub struct SinglePhaseRecycleSolver;

impl SinglePhaseRecycleSolver {
    pub fn solve_controlled(
        &self,
        flowsheet: &Flowsheet,
        properties: &SinglePhaseOptions,
        options: &SinglePhaseRecycleOptions,
        control: &mut impl PartitionedSolverControl,
    ) -> Result<SinglePhaseRecycleResult, Box<SinglePhaseRecycleFailure>> {
        let mut run = RecycleRun {
            flowsheet,
            options,
            model: &properties.model,
            heat_duties_w: &properties.heat_duties_w,
            control,
            stage: PartitionedProgress {
                stage: PartitionedStage::AssemblyStarted,
                evaluation: None,
                residual: None,
                phase: None,
                block: None,
                unit: None,
            },
            operations: BTreeMap::new(),
            streams: BTreeMap::new(),
            trace: Vec::new(),
            last_evaluation: None,
        };
        match run.solve() {
            Ok((final_units, overall_energy_balance, scales)) => Ok(SinglePhaseRecycleResult {
                streams: run.streams,
                final_units,
                tear: options.tear.clone(),
                verification: run.last_evaluation.expect("verified execution"),
                trace: run.trace,
                component_scale_mol_s: scales.component_mol_s,
                tear_enthalpy_scale_w: scales.enthalpy_w,
                overall_energy_balance,
            }),
            Err(error) => Err(Box::new(SinglePhaseRecycleFailure {
                error,
                stage: run.stage,
                trace: run.trace,
                last_evaluation: run.last_evaluation,
            })),
        }
    }
}

struct RecycleScales {
    component_mol_s: f64,
    component_flows_mol_s: BTreeMap<ComponentId, f64>,
    enthalpy_w: f64,
}

struct RecycleRun<'a, 'c, C> {
    flowsheet: &'a Flowsheet,
    options: &'a SinglePhaseRecycleOptions,
    model: &'a SensibleEnthalpyModel,
    heat_duties_w: &'a BTreeMap<UnitId, f64>,
    control: &'c mut C,
    stage: PartitionedProgress,
    operations: BTreeMap<UnitId, Box<dyn UnitOperation + 'a>>,
    streams: BTreeMap<StreamId, MaterialStreamState>,
    trace: Vec<SinglePhaseRecycleResidual>,
    last_evaluation: Option<SinglePhaseTearEvaluation>,
}

impl<C: PartitionedSolverControl> RecycleRun<'_, '_, C> {
    fn solve(&mut self) -> RfResult<(Vec<UnitSolveStep>, UnitEnergyBalance, RecycleScales)> {
        self.checkpoint(PartitionedStage::AssemblyStarted, None)?;
        let plan = plan::assemble(self.flowsheet, self.options)?;
        validate_heat_duty_targets(self.flowsheet, self.heat_duties_w)?;
        for unit in self.flowsheet.units.values() {
            self.operations.insert(
                unit.id.clone(),
                instantiate_single_phase_with_heat_duty(
                    unit,
                    &self.flowsheet.streams,
                    self.model,
                    self.heat_duties_w.get(&unit.id).copied(),
                )
                .map_err(|error| error.with_related_unit_id(unit.id.clone()))?,
            );
        }
        self.checkpoint(PartitionedStage::AssemblyCompleted, None)?;
        let mut upstream = Vec::with_capacity(plan.upstream.len());
        for id in &plan.upstream {
            upstream.push(self.evaluate_unit(upstream.len(), id)?);
        }
        self.checkpoint(PartitionedStage::Initializing, None)?;
        let fresh = self.streams[&plan.fresh].clone();
        let fresh_flow = fresh.total_molar_flow_mol_s;
        if fresh_flow <= 0.0 {
            return Err(recycle_error(
                "nonpositive_fresh_flow",
                "single-vapor recycle requires positive fresh inflow",
            )
            .with_related_stream_id(fresh.id));
        }
        let splitter = &self.flowsheet.units[&plan.cycle[2]];
        if splitter.parameters.outlet_a_split_fraction == Some(1.0) {
            return Err(recycle_error(
                "no_finite_steady_state",
                "full recycle with positive fresh inflow has no finite steady state",
            )
            .with_related_unit_id(splitter.id.clone())
            .with_related_stream_id(self.options.tear.stream_id.clone()));
        }
        let heat_unit = &self.flowsheet.units[&plan.cycle[1]];
        if let Some(&duty_w) = self.heat_duties_w.get(&heat_unit.id) {
            self.stage.unit = Some(heat_unit.id.clone());
            let operation = instantiate_single_phase_recycle_heat_duty(
                heat_unit,
                &self.flowsheet.streams,
                self.model,
                duty_w,
                &fresh,
            )?;
            self.operations.insert(heat_unit.id.clone(), operation);
            self.stage.unit = None;
        }
        let scales = RecycleScales {
            component_mol_s: fresh_flow,
            component_flows_mol_s: plan
                .components
                .iter()
                .cloned()
                .map(|id| (id, fresh_flow))
                .collect(),
            enthalpy_w: single_phase_enthalpy_flow_w(&fresh)?.abs().max(1.0),
        };
        let zero_flow_guess = self.materialize_tear(
            &fresh,
            &SinglePhaseTearState {
                component_flows_mol_s: plan
                    .components
                    .iter()
                    .cloned()
                    .map(|id| (id, 0.0))
                    .collect(),
                enthalpy_flow_w: 0.0,
            },
        )?;
        let mut assumed = match &self.options.initial_state {
            Some(initial) => self.materialize_tear(&fresh, initial)?,
            None => zero_flow_guess.clone(),
        };

        for iteration in 1..=self.options.max_iteration_evaluations {
            self.stage.evaluation = Some(iteration);
            self.stage.phase = Some(EvaluationPhase::Iteration);
            self.stage.residual = None;
            self.checkpoint(PartitionedStage::IterationStarted, None)?;
            self.evaluate_cycle(&plan, &scales, assumed)?;
            self.checkpoint(PartitionedStage::IterationCompleted, None)?;
            let evaluation = self.last_evaluation.as_ref().expect("completed evaluation");
            assumed = if evaluation.returned.total_molar_flow_mol_s == 0.0 {
                zero_flow_guess.clone()
            } else {
                evaluation.returned.clone()
            };
            if accepted(&evaluation.residual) {
                self.stage.evaluation = Some(1);
                self.stage.phase = Some(EvaluationPhase::Verification);
                self.stage.residual = None;
                self.checkpoint(PartitionedStage::VerificationStarted, None)?;
                let cycle_units = self.evaluate_cycle(&plan, &scales, assumed)?;
                if !accepted(
                    &self
                        .last_evaluation
                        .as_ref()
                        .expect("verification evaluated")
                        .residual,
                ) {
                    return Err(self.tear_error(
                        "verification_failed",
                        "fresh recycle evaluation exceeds closure tolerances",
                    ));
                }
                upstream.extend(cycle_units);
                self.checkpoint(PartitionedStage::VerificationCompleted, None)?;
                let overall = self.verify_overall(&plan, &scales, &upstream)?;
                self.checkpoint(PartitionedStage::Completing, None)?;
                return Ok((upstream, overall, scales));
            }
        }
        Err(self.tear_error(
            "iteration_limit",
            "single-vapor recycle exhausted its evaluation budget",
        ))
    }

    fn materialize_tear(
        &self,
        fresh: &MaterialStreamState,
        state: &SinglePhaseTearState,
    ) -> RfResult<MaterialStreamState> {
        let total: f64 = state.component_flows_mol_s.values().sum();
        if !total.is_finite()
            || !state.enthalpy_flow_w.is_finite()
            || (total == 0.0 && state.enthalpy_flow_w != 0.0)
        {
            return Err(self.tear_error(
                "invalid_initial_state",
                "tear requires finite flows and zero enthalpy flow at zero total flow",
            ));
        }
        let pressure = self.model.pressure_pa();
        let (temperature, fractions) = if total == 0.0 {
            (fresh.temperature_k, fresh.overall_mole_fractions.clone())
        } else {
            let fractions = state
                .component_flows_mol_s
                .iter()
                .map(|(id, value)| (id.clone(), value / total))
                .collect();
            let temperature = self
                .model
                .temperature_for_molar_enthalpy(state.enthalpy_flow_w / total, pressure, &fractions)
                .map_err(|error| {
                    error.with_related_stream_id(self.options.tear.stream_id.clone())
                })?;
            (temperature, fractions)
        };
        let template = &self.flowsheet.streams[&self.options.tear.stream_id];
        materialize_single_phase_vapor(
            self.model,
            MaterialStreamState::from_tpzf(
                template.id.clone(),
                template.name.clone(),
                temperature,
                pressure,
                total,
                fractions,
            ),
        )
        .map_err(|error| error.with_related_stream_id(template.id.clone()))
    }

    fn evaluate_unit(&mut self, index: usize, id: &UnitId) -> RfResult<UnitSolveStep> {
        self.checkpoint(PartitionedStage::UnitStarted, Some(id.clone()))?;
        let unit = &self.flowsheet.units[id];
        let operation = self.operations[id].as_ref();
        let properties = SequentialProperties::SinglePhase {
            model: self.model,
            heat_duties_w: self.heat_duties_w,
        };
        let step = evaluate_unit_step(
            index,
            unit,
            operation.spec(),
            operation,
            &self.streams,
            properties,
        )?;
        for stream in &step.produced_streams {
            self.streams.insert(stream.id.clone(), stream.clone());
        }
        self.checkpoint(PartitionedStage::UnitCompleted, Some(id.clone()))?;
        Ok(step)
    }

    fn evaluate_cycle(
        &mut self,
        plan: &Plan,
        scales: &RecycleScales,
        assumed: MaterialStreamState,
    ) -> RfResult<Vec<UnitSolveStep>> {
        self.streams.insert(assumed.id.clone(), assumed.clone());
        let units = plan
            .cycle
            .iter()
            .enumerate()
            .map(|(index, id)| self.evaluate_unit(plan.upstream.len() + index, id))
            .collect::<RfResult<_>>()?;
        let returned = self.streams[&self.options.tear.stream_id].clone();
        let component_residuals_mol_s = residuals(
            &plan.components,
            &component_flows(&plan.components, &assumed)?,
            &component_flows(&plan.components, &returned)?,
        );
        let enthalpy_flow_residual_w =
            single_phase_enthalpy_flow_w(&returned)? - single_phase_enthalpy_flow_w(&assumed)?;
        let residual = SinglePhaseRecycleResidual {
            material: sample(
                &component_residuals_mol_s,
                &scales.component_flows_mol_s,
                self.stage.phase.expect("evaluation phase"),
                self.stage.evaluation.expect("evaluation index"),
            )?,
            enthalpy_flow_residual_w,
            scaled_enthalpy_flow_residual: enthalpy_flow_residual_w / scales.enthalpy_w,
        };
        if !residual.enthalpy_flow_residual_w.is_finite() {
            return Err(self.tear_error(
                "nonfinite_residual",
                "tear enthalpy flow residual is nonfinite",
            ));
        }
        self.stage.residual = Some(residual.material.clone());
        self.trace.push(residual.clone());
        self.last_evaluation = Some(SinglePhaseTearEvaluation {
            assumed,
            returned,
            component_residuals_mol_s,
            residual,
        });
        Ok(units)
    }

    fn verify_overall(
        &self,
        plan: &Plan,
        scales: &RecycleScales,
        steps: &[UnitSolveStep],
    ) -> RfResult<UnitEnergyBalance> {
        let mut feed_flows: BTreeMap<_, _> = plan
            .components
            .iter()
            .cloned()
            .map(|id| (id, 0.0))
            .collect();
        let mut feed_rates_w = Vec::new();
        let mut feed_total_mol_s = 0.0;
        for feed in steps
            .iter()
            .filter(|step| step.unit_kind == BuiltinUnitKind::Feed.as_str())
            .flat_map(|step| &step.produced_streams)
        {
            for (id, flow) in component_flows(&plan.components, feed)? {
                *feed_flows.get_mut(&id).expect("registered component") += flow;
            }
            feed_total_mol_s += feed.total_molar_flow_mol_s;
            feed_rates_w.push(single_phase_enthalpy_flow_w(feed)?);
        }
        let product = &self.streams[&plan.product];
        let defects = residuals(
            &plan.components,
            &feed_flows,
            &component_flows(&plan.components, product)?,
        );
        let material = sample(
            &defects,
            &scales.component_flows_mol_s,
            EvaluationPhase::Verification,
            1,
        )?;
        let total_defect = (product.total_molar_flow_mol_s - feed_total_mol_s).abs();
        if !converged(&material)
            || !total_defect.is_finite()
            || total_defect > plan.components.len() as f64 * ABSOLUTE_FLOW_TOLERANCE
        {
            return Err(recycle_error(
                "overall_component_balance",
                "product and fresh feed do not close the material balance",
            )
            .with_related_stream_id(product.id.clone()));
        }
        let heat_duty_w = steps
            .iter()
            .filter_map(|step| step.energy_balance)
            .map(|balance| balance.heat_duty_w)
            .sum();
        verify_single_phase_energy_balance(&feed_rates_w, heat_duty_w, &[product])
    }

    fn checkpoint(&mut self, stage: PartitionedStage, unit: Option<UnitId>) -> RfResult<()> {
        self.stage.stage = stage;
        self.stage.unit = unit;
        if self.control.checkpoint(self.stage.clone()).is_break() {
            return Err(self.tear_error("cancelled", "single-vapor recycle was cancelled"));
        }
        Ok(())
    }

    fn tear_error(&self, code: &str, message: &str) -> RfError {
        recycle_error(code, message)
            .with_related_stream_id(self.options.tear.stream_id.clone())
            .with_related_port_targets(vec![
                self.options.tear.source.clone(),
                self.options.tear.sink.clone(),
            ])
    }
}

fn component_flows(
    components: &[ComponentId],
    stream: &MaterialStreamState,
) -> RfResult<BTreeMap<ComponentId, f64>> {
    let mut flows = stream_component_molar_flows(stream)?;
    for id in flows.keys() {
        if components.binary_search(id).is_err() {
            return Err(recycle_error(
                "unregistered_component",
                "stream composition references an unregistered component",
            )
            .with_related_stream_id(stream.id.clone())
            .with_related_component_id(id.clone()));
        }
    }
    for id in components {
        flows.entry(id.clone()).or_insert(0.0);
    }
    Ok(flows)
}

fn accepted(residual: &SinglePhaseRecycleResidual) -> bool {
    converged(&residual.material)
        && residual.enthalpy_flow_residual_w.abs() <= ENERGY_ABSOLUTE_TOLERANCE_W
        && residual.scaled_enthalpy_flow_residual.abs() <= ENERGY_SCALED_TOLERANCE
}

fn recycle_error(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message)
        .with_diagnostic_code(format!("solver.single_phase_recycle.{code}"))
}
