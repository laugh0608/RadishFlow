use rf_model::{Composition, MaterialStreamState, PhaseState};
use rf_thermo::{ThermoProvider, ThermoState};
use rf_types::{PhaseLabel, RfError, RfResult, StreamId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashStatus {
    Placeholder,
    Converged,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TpFlashInput {
    pub stream_id: StreamId,
    pub stream_name: String,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub total_molar_flow_mol_s: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl TpFlashInput {
    pub fn new(
        stream_id: impl Into<StreamId>,
        stream_name: impl Into<String>,
        temperature_k: f64,
        pressure_pa: f64,
        total_molar_flow_mol_s: f64,
        overall_mole_fractions: Vec<f64>,
    ) -> Self {
        Self {
            stream_id: stream_id.into(),
            stream_name: stream_name.into(),
            temperature_k,
            pressure_pa,
            total_molar_flow_mol_s,
            overall_mole_fractions,
        }
    }

    pub fn thermo_state(&self) -> ThermoState {
        ThermoState::new(
            self.temperature_k,
            self.pressure_pa,
            self.overall_mole_fractions.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TpFlashResult {
    pub status: FlashStatus,
    pub stream: MaterialStreamState,
    pub vapor_fraction: Option<f64>,
    pub k_values: Option<Vec<f64>>,
}

pub trait TpFlashSolver {
    fn flash(&self, thermo: &dyn ThermoProvider, input: &TpFlashInput) -> RfResult<TpFlashResult>;
}

#[derive(Debug, Default)]
pub struct PlaceholderTpFlashSolver;

impl PlaceholderTpFlashSolver {
    fn build_composition(thermo: &dyn ThermoProvider, mole_fractions: &[f64]) -> Composition {
        thermo
            .system()
            .component_ids()
            .into_iter()
            .zip(mole_fractions.iter().copied())
            .collect()
    }

    fn rachford_rice(beta: f64, overall: &[f64], k_values: &[f64]) -> f64 {
        overall
            .iter()
            .zip(k_values.iter())
            .map(|(z_i, k_i)| {
                let k_minus_one = k_i - 1.0;
                z_i * k_minus_one / (1.0 + beta * k_minus_one)
            })
            .sum()
    }

    fn solve_vapor_fraction(overall: &[f64], k_values: &[f64]) -> RfResult<f64> {
        let all_liquid = k_values.iter().all(|value| *value <= 1.0);
        if all_liquid {
            return Ok(0.0);
        }

        let all_vapor = k_values.iter().all(|value| *value >= 1.0);
        if all_vapor {
            return Ok(1.0);
        }

        let f_zero = Self::rachford_rice(0.0, overall, k_values);
        let f_one = Self::rachford_rice(1.0, overall, k_values);

        if !f_zero.is_finite() || !f_one.is_finite() {
            return Err(RfError::flash(
                "Rachford-Rice evaluation produced a non-finite boundary value",
            ));
        }

        if f_zero < 0.0 {
            return Ok(0.0);
        }

        if f_one > 0.0 {
            return Ok(1.0);
        }

        let mut lower = 0.0;
        let mut upper = 1.0;

        for _ in 0..100 {
            let beta = 0.5 * (lower + upper);
            let value = Self::rachford_rice(beta, overall, k_values);

            if !value.is_finite() {
                return Err(RfError::flash(
                    "Rachford-Rice iteration produced a non-finite value",
                ));
            }

            if value.abs() <= 1e-12 || (upper - lower) <= 1e-12 {
                return Ok(beta.clamp(0.0, 1.0));
            }

            if value > 0.0 {
                lower = beta;
            } else {
                upper = beta;
            }
        }

        Ok((0.5 * (lower + upper)).clamp(0.0, 1.0))
    }

    fn normalize_composition(values: Vec<f64>) -> RfResult<Vec<f64>> {
        if values.iter().any(|value| !value.is_finite() || *value < 0.0) {
            return Err(RfError::flash(
                "phase composition contains a non-finite or negative value",
            ));
        }

        let sum = values.iter().sum::<f64>();
        if !sum.is_finite() || sum <= 0.0 {
            return Err(RfError::flash(
                "phase composition must sum to a positive finite value",
            ));
        }

        Ok(values.into_iter().map(|value| value / sum).collect())
    }
}

impl TpFlashSolver for PlaceholderTpFlashSolver {
    fn flash(&self, thermo: &dyn ThermoProvider, input: &TpFlashInput) -> RfResult<TpFlashResult> {
        let state = input.thermo_state();
        thermo.system().validate_state(&state)?;

        if input.total_molar_flow_mol_s < 0.0 {
            return Err(RfError::invalid_input(
                "total molar flow must be non-negative",
            ));
        }

        let k_values = thermo.estimate_k_values(&state)?;
        if k_values.len() != input.overall_mole_fractions.len() {
            return Err(RfError::flash(format!(
                "expected {} K-values, received {}",
                input.overall_mole_fractions.len(),
                k_values.len()
            )));
        }

        if k_values.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Err(RfError::flash(
                "K-values must be finite numbers greater than zero",
            ));
        }

        let vapor_fraction = Self::solve_vapor_fraction(&input.overall_mole_fractions, &k_values)?;
        let liquid_fraction = 1.0 - vapor_fraction;

        let overall_mole_fractions = Self::build_composition(thermo, &input.overall_mole_fractions);
        let overall_phase =
            PhaseState::new(PhaseLabel::Overall, 1.0, overall_mole_fractions.clone());

        let liquid_mole_fractions = Self::normalize_composition(
            input.overall_mole_fractions
                .iter()
                .zip(k_values.iter())
                .map(|(z_i, k_i)| z_i / (1.0 + vapor_fraction * (k_i - 1.0)))
                .collect(),
        )?;
        let vapor_mole_fractions = Self::normalize_composition(
            liquid_mole_fractions
                .iter()
                .zip(k_values.iter())
                .map(|(x_i, k_i)| k_i * x_i)
                .collect(),
        )?;

        let mut stream = MaterialStreamState::from_tpzf(
            input.stream_id.clone(),
            input.stream_name.clone(),
            input.temperature_k,
            input.pressure_pa,
            input.total_molar_flow_mol_s,
            overall_mole_fractions,
        );
        stream.phases.push(overall_phase);
        if liquid_fraction > 0.0 {
            stream.phases.push(PhaseState::new(
                PhaseLabel::Liquid,
                liquid_fraction,
                Self::build_composition(thermo, &liquid_mole_fractions),
            ));
        }
        if vapor_fraction > 0.0 {
            stream.phases.push(PhaseState::new(
                PhaseLabel::Vapor,
                vapor_fraction,
                Self::build_composition(thermo, &vapor_mole_fractions),
            ));
        }

        Ok(TpFlashResult {
            status: FlashStatus::Converged,
            stream,
            vapor_fraction: Some(vapor_fraction),
            k_values: Some(k_values),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FlashStatus, PlaceholderTpFlashSolver, TpFlashInput, TpFlashSolver};
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::{ComponentId, PhaseLabel};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
        );
    }

    fn build_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
        let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(AntoineCoefficients::new(
            ((k_values[0] * pressure_pa) / 1_000.0).ln(),
            0.0,
            0.0,
        ));

        let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(AntoineCoefficients::new(
            ((k_values[1] * pressure_pa) / 1_000.0).ln(),
            0.0,
            0.0,
        ));

        PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
    }

    #[test]
    fn flash_solver_solves_binary_two_phase_case() {
        let pressure_pa = 100_000.0;
        let provider = build_provider([2.0, 0.5], pressure_pa);
        let solver = PlaceholderTpFlashSolver;
        let input = TpFlashInput::new(
            "stream-1",
            "Feed",
            300.0,
            pressure_pa,
            10.0,
            vec![0.5, 0.5],
        );

        let result = solver.flash(&provider, &input).expect("expected flash result");

        assert_eq!(result.status, FlashStatus::Converged);
        assert_close(result.vapor_fraction.expect("expected vapor fraction"), 0.5, 1e-10);

        let liquid = result
            .stream
            .phases
            .iter()
            .find(|phase| phase.label == PhaseLabel::Liquid)
            .expect("expected liquid phase");
        let vapor = result
            .stream
            .phases
            .iter()
            .find(|phase| phase.label == PhaseLabel::Vapor)
            .expect("expected vapor phase");

        assert_close(liquid.phase_fraction, 0.5, 1e-10);
        assert_close(vapor.phase_fraction, 0.5, 1e-10);
        assert_close(
            *liquid
                .mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected liquid component a"),
            1.0 / 3.0,
            1e-10,
        );
        assert_close(
            *vapor
                .mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected vapor component a"),
            2.0 / 3.0,
            1e-10,
        );
    }

    #[test]
    fn flash_solver_returns_single_liquid_phase_when_all_k_values_below_one() {
        let pressure_pa = 100_000.0;
        let provider = build_provider([0.8, 0.6], pressure_pa);
        let solver = PlaceholderTpFlashSolver;
        let input = TpFlashInput::new(
            "stream-1",
            "Feed",
            300.0,
            pressure_pa,
            10.0,
            vec![0.25, 0.75],
        );

        let result = solver.flash(&provider, &input).expect("expected flash result");

        assert_eq!(result.status, FlashStatus::Converged);
        assert_close(result.vapor_fraction.expect("expected vapor fraction"), 0.0, 1e-12);
        assert_eq!(result.stream.phases.len(), 2);
        assert!(result.stream.phases.iter().any(|phase| phase.label == PhaseLabel::Overall));
        assert!(result.stream.phases.iter().any(|phase| phase.label == PhaseLabel::Liquid));
        assert!(!result.stream.phases.iter().any(|phase| phase.label == PhaseLabel::Vapor));
    }
}
