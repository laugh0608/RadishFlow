use super::*;
impl EquationProblem {
    pub fn evaluate(&self, values: &[f64]) -> EoResult<EquationEvaluation> {
        if values.len() != self.variables.len() || values.iter().any(|v| !v.is_finite()) {
            return Err(error(
                EquationFailureKind::NonFinite,
                "evaluation",
                "finite value for every declared variable required",
            ));
        }
        let mut residuals = Vec::with_capacity(self.rows.len());
        let mut jacobian = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let mut derivatives = vec![0.0; values.len()];
            let residual = match row {
                AssembledEquation::Unit {
                    model,
                    ports,
                    fraction,
                } => {
                    let v = ports.iter().map(|(p, i)| (*p, values[*i])).collect();
                    let result = model
                        .evaluate(&v, fraction.map(|i| values[i]))
                        .map_err(|e| mapped(EquationFailureKind::PhysicalDomain, e))?;
                    for (port, d) in result.port_derivatives {
                        derivatives[ports[&port]] += d;
                    }
                    if let (Some(i), Some(d)) = (fraction, result.fraction_derivative_mol_s) {
                        derivatives[*i] += d;
                    }
                    result.residual_mol_s
                }
                AssembledEquation::Connection { source, sink } => {
                    derivatives[*sink] += 1.0;
                    derivatives[*source] -= 1.0;
                    values[*sink] - values[*source]
                }
                AssembledEquation::ProductTotal { components, target } => {
                    for i in components {
                        derivatives[*i] += 1.0;
                    }
                    components.iter().map(|i| values[*i]).sum::<f64>() - target
                }
            };
            if !residual.is_finite() || derivatives.iter().any(|v| !v.is_finite()) {
                return Err(error(
                    EquationFailureKind::NonFinite,
                    "residual",
                    "equation evaluation overflowed",
                ));
            }
            residuals.push(residual);
            jacobian.push(derivatives);
        }
        Ok(EquationEvaluation {
            residuals_mol_s: residuals,
            jacobian_si: jacobian,
        })
    }
    pub(super) fn incidence(&self) -> EoResult<Vec<Vec<usize>>> {
        self.rows
            .iter()
            .map(|row| {
                let mut indices = match row {
                    AssembledEquation::Unit {
                        model,
                        ports,
                        fraction,
                    } => {
                        let (dependencies, released) = model
                            .dependencies()
                            .map_err(|e| mapped(EquationFailureKind::InvalidSpecification, e))?;
                        let mut v: Vec<_> = dependencies.iter().map(|p| ports[p]).collect();
                        if released {
                            v.push(fraction.expect("released binding assembled"));
                        }
                        v
                    }
                    AssembledEquation::Connection { source, sink } => vec![*source, *sink],
                    AssembledEquation::ProductTotal { components, .. } => components.clone(),
                };
                indices.sort_unstable();
                indices.dedup();
                Ok(indices)
            })
            .collect()
    }
}
