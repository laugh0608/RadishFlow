//! Immutable, material-only QS0 problem. No solver, application state or IO.
mod assembly;
mod diagnostics;
mod solve;
mod types;
pub use solve::*;
pub use types::*;

use rf_model::{Composition, Flowsheet};
use rf_types::{ComponentId, UnitId};
pub use rf_unitops::component_balance::MixerBalancePort;
use rf_unitops::component_balance::mixer_component_residual;

/// Numerical acceptance for the approved bounded arithmetic case, not physical accuracy.
pub const COMPONENT_BALANCE_TOLERANCE: f64 = 1e-10;
pub const COMPONENT_BALANCE_ABSOLUTE_TOLERANCE_MOL_S: f64 = 1e-10;

#[derive(Debug)]
pub struct MixerComponentBalanceProblem {
    mixer_id: UnitId,
    components: Vec<ComponentId>,
    variables: Vec<ComponentFlowVariable>,
    rows: Vec<ComponentBalanceRow>,
    fixed_inlet_flows_mol_s: Vec<f64>,
    jacobian_si: Vec<Vec<f64>>,
    scaled_jacobian: Vec<Vec<f64>>,
}

impl MixerComponentBalanceProblem {
    pub fn assemble(
        flowsheet: &Flowsheet,
        specification: &MixerBalanceSpecification,
    ) -> Result<Self, ComponentBalanceError> {
        assembly::assemble(flowsheet, specification)
    }

    /// Readiness uses exactly the assembly checks without constructing numeric matrices.
    pub fn validate_specification(
        flowsheet: &Flowsheet,
        specification: &MixerBalanceSpecification,
    ) -> Result<(), ComponentBalanceError> {
        assembly::validate(flowsheet, specification).map(|_| ())
    }

    pub fn variables(&self) -> &[ComponentFlowVariable] {
        &self.variables
    }
    pub fn equations(&self) -> &[ComponentBalanceRow] {
        &self.rows
    }
    /// Positions belong only to this assembled problem; bindings carry domain identity.
    pub fn variable_index(&self, port: MixerBalancePort, component: &ComponentId) -> Option<usize> {
        self.variables.iter().position(|v| {
            v.binding.port == ComponentBalancePort::Mixer(port)
                && &v.binding.component_id == component
        })
    }
    /// dr_SI / du_SI. Both quantities are mol/s, so coefficients are dimensionless.
    pub fn jacobian_si(&self) -> &[Vec<f64>] {
        &self.jacobian_si
    }
    /// diag(1 / residual_scale) * J_SI * diag(variable_scale).
    pub fn scaled_jacobian(&self) -> &[Vec<f64>] {
        &self.scaled_jacobian
    }

    pub fn evaluate_si(
        &self,
        point_mol_s: &[f64],
    ) -> Result<ComponentBalanceEvaluation, ComponentBalanceError> {
        self.validate_point(point_mol_s)?;
        let n = self.components.len();
        let mut residuals = Vec::with_capacity(3 * n);
        for (value, fixed) in point_mol_s.iter().zip(&self.fixed_inlet_flows_mol_s) {
            residuals.push(value - fixed);
        }
        for i in 0..n {
            residuals.push(mixer_component_residual(
                point_mol_s[i],
                point_mol_s[n + i],
                point_mol_s[2 * n + i],
            ));
        }
        let mut scaled = Vec::with_capacity(residuals.len());
        for (row, residual) in self.rows.iter().zip(&residuals) {
            if !residual.is_finite() {
                return Err(ComponentBalanceError::NonFiniteEquation {
                    equation: Box::new(row.equation.clone()),
                });
            }
            let value = residual / row.scale_mol_s;
            if !value.is_finite() {
                return Err(ComponentBalanceError::NonFiniteScaling {
                    equation: Box::new(row.equation.clone()),
                });
            }
            scaled.push(value);
        }
        Ok(ComponentBalanceEvaluation {
            residuals_mol_s: residuals,
            scaled_residuals: scaled,
        })
    }

    pub fn evaluate_scaled(
        &self,
        dimensionless_point: &[f64],
    ) -> Result<ComponentBalanceEvaluation, ComponentBalanceError> {
        self.check_size(dimensionless_point)?;
        let point = dimensionless_point
            .iter()
            .zip(&self.variables)
            .map(|(value, variable)| value * variable.scale_mol_s)
            .collect::<Vec<_>>();
        self.evaluate_si(&point)
    }

    /// Checks the supplied point; never computes a solution or publishes a run result.
    pub fn check_solution(
        &self,
        point_mol_s: &[f64],
    ) -> Result<MixerBalanceValues, ComponentBalanceError> {
        let evaluated = self.evaluate_si(point_mol_s)?;
        self.check_evaluated_solution(
            point_mol_s,
            &evaluated,
            COMPONENT_BALANCE_ABSOLUTE_TOLERANCE_MOL_S,
            COMPONENT_BALANCE_TOLERANCE,
        )
    }

    fn check_evaluated_solution(
        &self,
        point_mol_s: &[f64],
        evaluated: &ComponentBalanceEvaluation,
        absolute_tolerance_mol_s: f64,
        scaled_tolerance: f64,
    ) -> Result<MixerBalanceValues, ComponentBalanceError> {
        for ((row, value), residual) in self
            .rows
            .iter()
            .zip(&evaluated.scaled_residuals)
            .zip(&evaluated.residuals_mol_s)
        {
            if value.abs() > scaled_tolerance || residual.abs() > absolute_tolerance_mol_s {
                return Err(ComponentBalanceError::UnsatisfiedEquation {
                    equation: Box::new(row.equation.clone()),
                    scaled_residual: *value,
                    residual_mol_s: *residual,
                });
            }
        }
        let n = self.components.len();
        let flows: Composition = self
            .components
            .iter()
            .cloned()
            .zip(point_mol_s[2 * n..].iter().copied())
            .collect();
        let total: f64 = flows.values().sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(ComponentBalanceError::InvalidOutletTotal {
                mixer_id: self.mixer_id.clone(),
            });
        }
        let fractions = flows
            .iter()
            .map(|(id, flow)| (id.clone(), flow / total))
            .collect();
        Ok(MixerBalanceValues {
            component_molar_flows_mol_s: flows,
            total_molar_flow_mol_s: total,
            mole_fractions: fractions,
        })
    }

    fn check_size(&self, point: &[f64]) -> Result<(), ComponentBalanceError> {
        if point.len() != self.variables.len() {
            return Err(ComponentBalanceError::PointSize {
                expected: self.variables.len(),
                actual: point.len(),
            });
        }
        Ok(())
    }
    fn validate_point(&self, point: &[f64]) -> Result<(), ComponentBalanceError> {
        self.check_size(point)?;
        for (variable, value) in self.variables.iter().zip(point) {
            if !value.is_finite() || *value < 0.0 {
                return Err(ComponentBalanceError::InvalidTrialValue {
                    binding: variable.binding.clone(),
                    value: *value,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
