//! Direct solve of the supported fixed-feed material template, with SI verification.
use std::{
    fmt,
    ops::ControlFlow,
    time::{Duration, Instant},
};

use super::*;
use rf_types::UnitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceAlgorithm {
    LowerTriangularSubstitution,
}
impl MaterialBalanceAlgorithm {
    pub const fn implementation(self) -> &'static str {
        match self {
            Self::LowerTriangularSubstitution => concat!(
                "rf-solver::material-balance/lower-triangular-substitution/1@",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceOptions {
    pub algorithm: MaterialBalanceAlgorithm,
    pub absolute_tolerance_mol_s: f64,
    pub scaled_tolerance: f64,
}
impl Default for MaterialBalanceOptions {
    fn default() -> Self {
        Self {
            algorithm: MaterialBalanceAlgorithm::LowerTriangularSubstitution,
            absolute_tolerance_mol_s: COMPONENT_BALANCE_ABSOLUTE_TOLERANCE_MOL_S,
            scaled_tolerance: COMPONENT_BALANCE_TOLERANCE,
        }
    }
}
impl MaterialBalanceOptions {
    pub fn validate(&self) -> Result<(), MaterialBalanceSolveError> {
        for (option, value) in [
            (
                MaterialBalanceTolerance::AbsoluteMolarFlow,
                self.absolute_tolerance_mol_s,
            ),
            (
                MaterialBalanceTolerance::ScaledResidual,
                self.scaled_tolerance,
            ),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(MaterialBalanceSolveError::InvalidTolerance { option });
            }
        }
        Ok(())
    }
}

/// Session input, not a persistent project setting or a complete flowsheet strategy.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalancePlan {
    pub specification: MixerBalanceSpecification,
    pub options: MaterialBalanceOptions,
}
impl MaterialBalancePlan {
    pub fn new(specification: MixerBalanceSpecification) -> Self {
        Self {
            specification,
            options: MaterialBalanceOptions::default(),
        }
    }
    /// Suggested session settings use the actual connected feed totals, never outlet templates.
    /// A zero feed yields a zero proposed scale which the caller must explicitly replace.
    pub fn for_mixer(
        flowsheet: &rf_model::Flowsheet,
        mixer_id: UnitId,
    ) -> Result<Self, ComponentBalanceError> {
        let connections = rf_flowsheet::validate_connections(flowsheet)?;
        let mut values = [0.0; 2];
        for (i, port) in [MixerBalancePort::InletA, MixerBalancePort::InletB]
            .into_iter()
            .enumerate()
        {
            let inlet = connections
                .iter()
                .find(|c| {
                    c.sink
                        .as_ref()
                        .is_some_and(|s| s.unit_id == mixer_id && s.port_name == port.name())
                })
                .ok_or_else(|| ComponentBalanceError::UnsupportedTopology {
                    mixer_id: mixer_id.clone(),
                })?;
            values[i] = flowsheet.stream(&inlet.stream_id)?.total_molar_flow_mol_s;
        }
        Ok(Self::new(MixerBalanceSpecification::new(
            mixer_id,
            MixerBalanceScales {
                inlet_a_mol_s: values[0],
                inlet_b_mol_s: values[1],
                outlet_mol_s: values[0] + values[1],
            },
        )))
    }
    pub fn validate(&self) -> Result<(), MaterialBalanceSolveError> {
        self.options.validate()?;
        for port in MixerBalancePort::ALL {
            let scale = self.specification.scales.for_port(port);
            if !scale.is_finite() || scale <= 0.0 {
                return Err(ComponentBalanceError::InvalidScale { port }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceTolerance {
    AbsoluteMolarFlow,
    ScaledResidual,
}

#[derive(Debug, PartialEq)]
pub enum MaterialBalanceSolveError {
    InvalidTolerance {
        option: MaterialBalanceTolerance,
    },
    UnsupportedStructure {
        equation: Box<ComponentBalanceEquation>,
    },
    NonFiniteSolution {
        binding: ComponentFlowBinding,
    },
    Problem(ComponentBalanceError),
}
impl From<ComponentBalanceError> for MaterialBalanceSolveError {
    fn from(error: ComponentBalanceError) -> Self {
        Self::Problem(error)
    }
}
impl fmt::Display for MaterialBalanceSolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { option } => write!(
                f,
                "material balance tolerance {option:?} must be positive and finite"
            ),
            Self::UnsupportedStructure { equation } => write!(
                f,
                "material balance row {equation:?} is not supported by lower triangular substitution"
            ),
            Self::NonFiniteSolution { binding } => {
                write!(f, "non-finite material balance solution at {binding:?}")
            }
            Self::Problem(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for MaterialBalanceSolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Problem(error) = self {
            Some(error)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceStage {
    Solving,
    Verifying,
    Completed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialBalanceProgress {
    pub stage: MaterialBalanceStage,
    pub completed_rows: usize,
    pub total_rows: usize,
}
/// Safe boundaries are complete linear rows and the final SI verification.
pub trait MaterialBalanceControl {
    fn checkpoint(&mut self, progress: MaterialBalanceProgress) -> ControlFlow<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceSolution {
    pub options: MaterialBalanceOptions,
    pub variables: Vec<ComponentFlowVariable>,
    pub equations: Vec<ComponentBalanceRow>,
    pub point_mol_s: Vec<f64>,
    pub evaluation: ComponentBalanceEvaluation,
    pub values: MixerBalanceValues,
    pub solved_rows: usize,
    pub residual_evaluations: usize,
    pub elapsed: Duration,
}
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialBalanceSolveOutcome {
    Completed(Box<MaterialBalanceSolution>),
    Cancelled,
}

impl MixerComponentBalanceProblem {
    pub fn solve_controlled(
        &self,
        options: &MaterialBalanceOptions,
        control: &mut impl MaterialBalanceControl,
    ) -> Result<MaterialBalanceSolveOutcome, MaterialBalanceSolveError> {
        options.validate()?;
        let started = Instant::now();
        let size = self.variables.len();
        let checkpoint = |control: &mut _, stage, completed_rows| {
            MaterialBalanceControl::checkpoint(
                control,
                MaterialBalanceProgress {
                    stage,
                    completed_rows,
                    total_rows: size,
                },
            )
            .is_break()
        };
        if checkpoint(control, MaterialBalanceStage::Solving, 0) {
            return Ok(MaterialBalanceSolveOutcome::Cancelled);
        }
        let mut scaled_point = vec![0.0; size];
        for (i, row) in self.scaled_jacobian.iter().enumerate() {
            if row[i] == 0.0 || !row[i].is_finite() || row[i + 1..].iter().any(|v| *v != 0.0) {
                return Err(MaterialBalanceSolveError::UnsupportedStructure {
                    equation: Box::new(self.rows[i].equation.clone()),
                });
            }
            let fixed = match &self.rows[i].equation {
                ComponentBalanceEquation::Connection { .. } => self.fixed_inlet_flows_mol_s[i],
                ComponentBalanceEquation::MixerConservation { .. } => 0.0,
            };
            let rhs = fixed / self.rows[i].scale_mol_s;
            let known: f64 = row[..i]
                .iter()
                .zip(&scaled_point[..i])
                .map(|(a, x)| a * x)
                .sum();
            scaled_point[i] = (rhs - known) / row[i];
            if !scaled_point[i].is_finite() {
                return Err(MaterialBalanceSolveError::NonFiniteSolution {
                    binding: self.variables[i].binding.clone(),
                });
            }
            if checkpoint(control, MaterialBalanceStage::Solving, i + 1) {
                return Ok(MaterialBalanceSolveOutcome::Cancelled);
            }
        }
        if checkpoint(control, MaterialBalanceStage::Verifying, size) {
            return Ok(MaterialBalanceSolveOutcome::Cancelled);
        }
        let point_mol_s: Vec<_> = scaled_point
            .iter()
            .zip(&self.variables)
            .map(|(x, v)| x * v.scale_mol_s)
            .collect();
        let evaluation = self.evaluate_si(&point_mol_s)?;
        let values = self.check_evaluated_solution(
            &point_mol_s,
            &evaluation,
            options.absolute_tolerance_mol_s,
            options.scaled_tolerance,
        )?;
        if checkpoint(control, MaterialBalanceStage::Completed, size) {
            return Ok(MaterialBalanceSolveOutcome::Cancelled);
        }
        Ok(MaterialBalanceSolveOutcome::Completed(Box::new(
            MaterialBalanceSolution {
                options: options.clone(),
                variables: self.variables.clone(),
                equations: self.rows.clone(),
                point_mol_s,
                evaluation,
                values,
                solved_rows: size,
                residual_evaluations: 1,
                elapsed: started.elapsed(),
            },
        )))
    }
}

#[cfg(test)]
mod tests;
