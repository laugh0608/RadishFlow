//! Fixed-T/P material equations, independent of global indices and solver policy.
use crate::component_balance::{mixer_component_residual, splitter_component_outlet};
use rf_types::{RfError, RfResult};
use std::collections::BTreeMap;

pub const EQUATION_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentEquationCapability {
    Feed,
    Mixer,
    Splitter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialPort {
    Inlet,
    InletA,
    InletB,
    Outlet,
    OutletA,
    OutletB,
}
impl MaterialPort {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Inlet => "inlet",
            Self::InletA => "inlet_a",
            Self::InletB => "inlet_b",
            Self::Outlet => "outlet",
            Self::OutletA => "outlet_a",
            Self::OutletB => "outlet_b",
        }
    }
}
impl ComponentEquationCapability {
    pub const fn ports(self) -> &'static [MaterialPort] {
        use MaterialPort::*;
        match self {
            Self::Feed => &[Outlet],
            Self::Mixer => &[InletA, InletB, Outlet],
            Self::Splitter => &[Inlet, OutletA, OutletB],
        }
    }
    pub const fn rows(self) -> &'static [MaterialEquationRole] {
        use MaterialEquationRole::*;
        match self {
            Self::Feed => &[FeedBoundary],
            Self::Mixer => &[MixerBalance],
            Self::Splitter => &[SplitA, SplitB],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialEquationRole {
    FeedBoundary,
    MixerBalance,
    SplitA,
    SplitB,
}

/// Whether the parameter is fixed or contributes its own Jacobian column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitFraction {
    Fixed(f64),
    Released,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentEquation {
    pub role: MaterialEquationRole,
    pub feed_mol_s: Option<f64>,
    pub split: Option<SplitFraction>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentEquationEvaluation {
    pub residual_mol_s: f64,
    pub port_derivatives: BTreeMap<MaterialPort, f64>,
    /// d(residual mol/s)/d(fraction), with units mol/s.
    pub fraction_derivative_mol_s: Option<f64>,
}
impl ComponentEquation {
    /// Structural dependencies, including possible nonzeros over a released interval.
    pub fn dependencies(&self) -> RfResult<(Vec<MaterialPort>, bool)> {
        use MaterialEquationRole::*;
        use MaterialPort::*;
        self.validate()?;
        Ok(match self.role {
            FeedBoundary => (vec![Outlet], false),
            MixerBalance => (vec![InletA, InletB, Outlet], false),
            SplitA | SplitB => {
                let a = self.role == SplitA;
                let mut ports = vec![if a { OutletA } else { OutletB }];
                if !matches!(self.split, Some(SplitFraction::Fixed(f)) if f == if a {0.0} else {1.0})
                {
                    ports.push(Inlet);
                }
                (ports, self.split == Some(SplitFraction::Released))
            }
        })
    }
    pub fn validate(&self) -> RfResult<()> {
        use MaterialEquationRole::*;
        let valid = match self.role {
            FeedBoundary => {
                self.split.is_none() && self.feed_mol_s.is_some_and(|v| v.is_finite() && v >= 0.0)
            }
            MixerBalance => self.split.is_none() && self.feed_mol_s.is_none(),
            SplitA | SplitB => {
                self.feed_mol_s.is_none()
                    && match self.split {
                        Some(SplitFraction::Fixed(f)) => f.is_finite() && (0.0..=1.0).contains(&f),
                        Some(SplitFraction::Released) => true,
                        None => false,
                    }
            }
        };
        if valid { Ok(()) } else { Err(invalid()) }
    }
    /// Signed trial port values are allowed for derivative evaluation; the solver
    /// separately enforces the nonnegative physical domain before publication.
    pub fn evaluate(
        &self,
        ports: &BTreeMap<MaterialPort, f64>,
        fraction: Option<f64>,
    ) -> RfResult<ComponentEquationEvaluation> {
        self.validate()?;
        use MaterialEquationRole::*;
        use MaterialPort::*;
        let value = |p| {
            ports
                .get(&p)
                .copied()
                .filter(|v| v.is_finite())
                .ok_or_else(invalid)
        };
        if (self.split == Some(SplitFraction::Released)) != fraction.is_some() {
            return Err(invalid());
        }
        let mut derivatives = BTreeMap::new();
        let mut parameter = None;
        let residual = match self.role {
            FeedBoundary => {
                derivatives.insert(Outlet, 1.0);
                value(Outlet)? - self.feed_mol_s.expect("validated feed")
            }
            MixerBalance => {
                derivatives.extend([(InletA, -1.0), (InletB, -1.0), (Outlet, 1.0)]);
                mixer_component_residual(value(InletA)?, value(InletB)?, value(Outlet)?)
            }
            SplitA | SplitB => {
                let alpha = match self.split.expect("validated split") {
                    SplitFraction::Fixed(f) => f,
                    SplitFraction::Released => fraction.ok_or_else(invalid)?,
                };
                if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
                    return Err(invalid());
                }
                let a = self.role == SplitA;
                let out = if a { OutletA } else { OutletB };
                let factor = if a { alpha } else { 1.0 - alpha };
                derivatives.extend([(out, 1.0), (Inlet, -factor)]);
                if self.split == Some(SplitFraction::Released) {
                    parameter = Some(if a { -value(Inlet)? } else { value(Inlet)? });
                }
                value(out)? - splitter_component_outlet(value(Inlet)?, factor)
            }
        };
        if !residual.is_finite() {
            return Err(invalid());
        }
        Ok(ComponentEquationEvaluation {
            residual_mol_s: residual,
            port_derivatives: derivatives,
            fraction_derivative_mol_s: parameter,
        })
    }
}
fn invalid() -> RfError {
    RfError::invalid_input("invalid fixed-T/P component equation or evaluation domain")
        .with_diagnostic_code("unitops.component_equations.invalid")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_zero_dependencies_and_released_derivatives_have_distinct_meanings() {
        use MaterialEquationRole::*;
        use MaterialPort::*;
        for (role, zero) in [(SplitA, 0.0), (SplitB, 1.0)] {
            let mut row = ComponentEquation {
                role,
                feed_mol_s: None,
                split: Some(SplitFraction::Fixed(zero)),
            };
            assert!(!row.dependencies().unwrap().0.contains(&Inlet));
            row.split = Some(SplitFraction::Released);
            assert!(row.dependencies().unwrap().0.contains(&Inlet));
            for alpha in [0.0, 0.2, 0.8, 1.0] {
                let values = [
                    (Inlet, 7.0),
                    (OutletA, 7.0 * alpha),
                    (OutletB, 7.0 * (1.0 - alpha)),
                ]
                .into();
                let e = row.evaluate(&values, Some(alpha)).unwrap();
                assert_eq!(e.residual_mol_s, 0.0);
                assert_eq!(
                    e.fraction_derivative_mol_s,
                    Some(if role == SplitA { -7.0 } else { 7.0 })
                );
            }
        }
    }
    #[test]
    fn catalog_does_not_claim_equations_for_unsupported_models() {
        for model in crate::catalog::builtin_models() {
            let expected = matches!(
                model.spec.kind,
                crate::BuiltinUnitKind::Feed
                    | crate::BuiltinUnitKind::Mixer
                    | crate::BuiltinUnitKind::Splitter
            );
            assert_eq!(model.component_equation_capability().is_some(), expected);
        }
    }
}
