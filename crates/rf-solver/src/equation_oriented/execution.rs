use super::*;
use crate::dense_linear::{self, DenseCosts, DenseError, DenseFailureKind, DenseStage};
use crate::local_services::{CountedFlash, CountedThermo};
use rf_unitops::{UnitOperation, UnitOperationServices};
use std::{cell::Cell, ops::ControlFlow, time::Instant};

pub(super) struct Execution<'a> {
    pub control: &'a mut dyn EquationControl,
    pub progress: EquationProgress,
    pub evidence: EquationEvidence,
    pub costs: EquationCosts,
    stage_started: Instant,
}
impl EquationControl for Execution<'_> {
    fn checkpoint(&mut self, progress: EquationProgress) -> ControlFlow<()> {
        if progress.stage != self.progress.stage {
            *self
                .costs
                .stage_elapsed
                .entry(self.progress.stage)
                .or_default() += self.stage_started.elapsed();
            self.stage_started = Instant::now();
        }
        if progress.stage == EquationStage::StructuralAnalysis {
            self.costs.structural_analyses = 1;
            self.costs.structural_checkpoints += 1;
        }
        self.progress = progress.clone();
        self.control.checkpoint(progress)
    }
}
pub(super) fn solve(
    services: &crate::SolverServices<'_>,
    flow: &Flowsheet,
    options: &EquationOptions,
    control: &mut impl EquationControl,
    factory: &mut impl FnMut(
        &rf_model::UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> rf_types::RfResult<Box<dyn UnitOperation>>,
) -> Result<EquationResult, Box<EquationFailure>> {
    let start = Instant::now();
    let thermo = CountedThermo {
        provider: services.thermo,
        calls: Cell::new(0),
    };
    let flash = CountedFlash {
        solver: services.flash_solver,
        calls: Cell::new(0),
    };
    let mut run = Execution {
        control,
        progress: EquationProgress::at(EquationStage::Assemble),
        evidence: EquationEvidence::default(),
        costs: EquationCosts {
            assemblies: 1,
            ..Default::default()
        },
        stage_started: Instant::now(),
    };
    let mut backend = None;
    let result: EoResult<BTreeMap<StreamId, MaterialStreamState>> = (|| {
        let problem = assembly::assemble_controlled(flow, options, &mut run)?;
        run.evidence.definition = Some((&problem).into());
        problem.require_ready()?;
        backend = Some(if options.released.is_some() {
            INVERSE_BACKEND
        } else {
            FORWARD_BACKEND
        });
        let values = run.solve_equations(&problem)?;
        run.evidence.candidate_values_si = Some(values.clone());
        let evaluated = problem.evaluate(&values)?;
        run.evidence.candidate_residuals_mol_s = Some(evaluated.residuals_mol_s.clone());
        verify_residuals(&problem, &evaluated.residuals_mol_s)?;
        run.at(EquationStage::FullJacobianCheck, None)?;
        run.costs.full_jacobian_evaluations += 1;
        let matrix = scaled_jacobian(&problem, &evaluated.jacobian_si);
        let mut evidence = None;
        let checked = run.linear(&matrix, &vec![0.0; values.len()], &mut evidence);
        run.evidence.full_jacobian = evidence;
        checked?;
        if let Some(r) = &options.released {
            let i = problem
                .variables
                .iter()
                .position(|v| matches!(v, VariableBinding::Parameter(_)))
                .expect("released index");
            run.evidence.solved_parameter = Some(SolvedSplitFraction {
                binding: r.binding.clone(),
                stored_value: flow.units[&r.binding.unit]
                    .parameters
                    .outlet_a_split_fraction,
                value: values[i],
                interval: (r.lower, r.upper),
            });
        }
        let streams = run.verify(
            &problem,
            &values,
            &UnitOperationServices {
                thermo: Some(&thermo),
                flash_solver: Some(&flash),
            },
            factory,
        )?;
        run.at(EquationStage::Completing, None)?;
        Ok(streams)
    })();
    run.costs.thermo_calls = thermo.calls.get();
    run.costs.flash_calls = flash.calls.get();
    *run.costs
        .stage_elapsed
        .entry(run.progress.stage)
        .or_default() += run.stage_started.elapsed();
    run.costs.elapsed = start.elapsed();
    match result {
        Ok(streams) => Ok(EquationResult {
            backend: backend.expect("executed backend"),
            equation_contract_version: EQUATION_CONTRACT_VERSION,
            acceptance_contract_version: ACCEPTANCE_CONTRACT_VERSION,
            streams,
            evidence: run.evidence,
            costs: run.costs,
        }),
        Err(e) => Err(Box::new(EquationFailure {
            kind: e.kind,
            error: e.error,
            backend,
            evidence: run.evidence,
            progress: run.progress,
            costs: run.costs,
        })),
    }
}
impl Execution<'_> {
    pub(super) fn at(&mut self, stage: EquationStage, iteration: Option<usize>) -> EoResult<()> {
        let mut p = EquationProgress::at(stage);
        p.iteration = iteration;
        checkpoint(self, p)
    }
    fn linear(
        &mut self,
        matrix: &[Vec<f64>],
        rhs: &[f64],
        evidence: &mut Option<LinearEvidence>,
    ) -> EoResult<Vec<f64>> {
        let mut costs = DenseCosts::default();
        let stage = self.progress.stage;
        let iteration = self.progress.iteration;
        let result = dense_linear::solve(matrix, rhs, &mut costs, evidence, &mut |stage, index| {
            let mut p = EquationProgress::at(match stage {
                DenseStage::Factorization => EquationStage::Factorization,
                DenseStage::ConditionSolve => EquationStage::ConditionSolve,
                DenseStage::LinearVerification => EquationStage::LinearEvaluation,
            });
            p.column = Some(index);
            p.iteration = iteration;
            checkpoint(self, p)
        });
        self.costs.factorizations += costs.factorizations;
        self.costs.triangular_solves += costs.triangular_solves;
        let values = result
            .map_err(|e| match e {
                DenseError::Interrupted(e) => e,
                DenseError::Numerical(e) => error(
                    match e.kind {
                        DenseFailureKind::Singular => EquationFailureKind::LinearSingular,
                        DenseFailureKind::IllConditioned => EquationFailureKind::IllConditioned,
                        DenseFailureKind::Verification => EquationFailureKind::LinearVerification,
                    },
                    e.code,
                    e.message,
                ),
            })?
            .values;
        self.at(stage, iteration)?;
        Ok(values)
    }
    fn solve_equations(&mut self, p: &EquationProblem) -> EoResult<Vec<f64>> {
        let Some(released) = &p.options.released else {
            return self
                .conditional(p, None, None, EquationStage::LinearEvaluation, 0)
                .map(|v| v.0);
        };
        let (mut lower, mut upper) = (released.lower, released.upper);
        let (lo_values, mut lo_residual) = self.conditional(
            p,
            Some(lower),
            Some((lower, upper)),
            EquationStage::BracketEndpoint,
            0,
        )?;
        let (hi_values, hi_residual) = self.conditional(
            p,
            Some(upper),
            Some((lower, upper)),
            EquationStage::BracketEndpoint,
            1,
        )?;
        if target_converged(p, lo_residual) {
            return Ok(lo_values);
        }
        if target_converged(p, hi_residual) {
            return Ok(hi_values);
        }
        if lo_residual.is_sign_positive() == hi_residual.is_sign_positive() {
            return Err(error(
                EquationFailureKind::TargetNotBracketed,
                "target_not_bracketed",
                "target residual has no established sign bracket on the declared interval",
            ));
        }
        for iteration in 0..released.max_iterations {
            let midpoint = lower + (upper - lower) / 2.0;
            if midpoint == lower || midpoint == upper {
                return Err(error(
                    EquationFailureKind::Stagnated,
                    "stagnated",
                    "floating-point midpoint cannot shrink the bracket",
                ));
            }
            let (values, residual) = self.conditional(
                p,
                Some(midpoint),
                Some((lower, upper)),
                EquationStage::IntervalEvaluation,
                iteration,
            )?;
            if upper - lower <= FRACTION_INTERVAL_TOLERANCE && target_converged(p, residual) {
                return Ok(values);
            }
            if residual.is_sign_positive() == lo_residual.is_sign_positive() {
                lower = midpoint;
                lo_residual = residual;
            } else {
                upper = midpoint;
            }
            self.costs.interval_updates += 1;
        }
        Err(error(
            EquationFailureKind::BudgetExceeded,
            "budget",
            "bisection budget exhausted before joint width and residual acceptance",
        ))
    }
    fn conditional(
        &mut self,
        p: &EquationProblem,
        fraction: Option<f64>,
        interval: Option<(f64, f64)>,
        stage: EquationStage,
        iteration: usize,
    ) -> EoResult<(Vec<f64>, f64)> {
        self.at(stage, Some(iteration))?;
        self.costs.conditional_linear_evaluations += 1;
        let slot = self.evidence.conditional_evaluations.len();
        self.evidence
            .conditional_evaluations
            .push(ConditionalLinearEvaluation {
                fraction,
                interval,
                linear: None,
                target_residual_mol_s: None,
                completed: false,
            });
        let mut base = vec![0.0; p.variables.len()];
        let columns: Vec<_> = p
            .variables
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if matches!(v, VariableBinding::Parameter(_)) {
                    base[i] = fraction.expect("released evaluation");
                    None
                } else {
                    Some(i)
                }
            })
            .collect();
        let rows: Vec<_> = p
            .equations
            .iter()
            .enumerate()
            .filter_map(|(i, e)| (!matches!(e, EquationBinding::ProductTotal { .. })).then_some(i))
            .collect();
        let e = p.evaluate(&base)?;
        let matrix: Vec<Vec<_>> = rows
            .iter()
            .map(|&i| {
                columns
                    .iter()
                    .map(|&j| e.jacobian_si[i][j] * p.variable_scales[j] / p.residual_scales[i])
                    .collect()
            })
            .collect();
        let rhs: Vec<_> = rows
            .iter()
            .map(|&i| -e.residuals_mol_s[i] / p.residual_scales[i])
            .collect();
        let mut evidence = None;
        let solved = self.linear(&matrix, &rhs, &mut evidence);
        self.evidence.conditional_evaluations[slot].linear = evidence;
        let scaled = solved?;
        for (&j, z) in columns.iter().zip(scaled) {
            base[j] = z * p.variable_scales[j];
        }
        validate_values(p, &base)?;
        let e = p.evaluate(&base)?;
        for &i in &rows {
            require_residual(e.residuals_mol_s[i], p.residual_scales[i])?;
        }
        let target = p
            .equations
            .iter()
            .position(|e| matches!(e, EquationBinding::ProductTotal { .. }))
            .map(|i| e.residuals_mol_s[i]);
        if target.is_some() {
            self.costs.target_evaluations += 1;
        }
        let sample = &mut self.evidence.conditional_evaluations[slot];
        sample.target_residual_mol_s = target;
        sample.completed = true;
        self.at(stage, Some(iteration))?;
        Ok((base, target.unwrap_or(0.0)))
    }
}
fn target_converged(p: &EquationProblem, residual: f64) -> bool {
    let i = p
        .equations
        .iter()
        .position(|e| matches!(e, EquationBinding::ProductTotal { .. }))
        .expect("paired target");
    residual.is_finite()
        && residual.abs() <= ABSOLUTE_TOLERANCE_MOL_S
        && (residual / p.residual_scales[i]).abs() <= SCALED_TOLERANCE
}
pub(super) fn require_residual(value: f64, scale: f64) -> EoResult<()> {
    if !(value.is_finite()
        && scale.is_finite()
        && (value / scale).abs() <= SCALED_TOLERANCE
        && value.abs() <= ABSOLUTE_TOLERANCE_MOL_S)
    {
        Err(error(
            EquationFailureKind::LocalVerification,
            "residual",
            "absolute or scaled material residual exceeds acceptance",
        ))
    } else {
        Ok(())
    }
}
pub(super) fn verify_residuals(p: &EquationProblem, values: &[f64]) -> EoResult<()> {
    for (value, scale) in values.iter().zip(&p.residual_scales) {
        require_residual(*value, *scale)?;
    }
    Ok(())
}
fn validate_values(p: &EquationProblem, values: &[f64]) -> EoResult<()> {
    for (binding, value) in p.variables.iter().zip(values) {
        let valid = match binding {
            VariableBinding::PortComponent { .. } => *value >= 0.0,
            VariableBinding::Parameter(_) => p
                .options
                .released
                .as_ref()
                .is_some_and(|r| *value >= r.lower && *value <= r.upper),
        };
        if !value.is_finite() || !valid {
            return Err(error(
                EquationFailureKind::PhysicalDomain,
                "candidate_domain",
                "candidate is nonfinite, negative or outside the declared fraction interval",
            ));
        }
    }
    Ok(())
}
fn scaled_jacobian(p: &EquationProblem, jacobian: &[Vec<f64>]) -> Vec<Vec<f64>> {
    jacobian
        .iter()
        .enumerate()
        .map(|(i, row)| {
            row.iter()
                .enumerate()
                .map(|(j, v)| v * p.variable_scales[j] / p.residual_scales[i])
                .collect()
        })
        .collect()
}
