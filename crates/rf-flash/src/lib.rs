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

        let overall_mole_fractions = Self::build_composition(thermo, &input.overall_mole_fractions);
        let overall_phase =
            PhaseState::new(PhaseLabel::Overall, 1.0, overall_mole_fractions.clone());

        let mut stream = MaterialStreamState::from_tpzf(
            input.stream_id.clone(),
            input.stream_name.clone(),
            input.temperature_k,
            input.pressure_pa,
            input.total_molar_flow_mol_s,
            overall_mole_fractions,
        );
        stream.phases.push(overall_phase);

        Ok(TpFlashResult {
            status: FlashStatus::Placeholder,
            stream,
            vapor_fraction: None,
            k_values: None,
        })
    }
}
