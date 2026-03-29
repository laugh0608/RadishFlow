use rf_types::{ComponentId, PhaseLabel, RfError, RfResult};

#[derive(Debug, Clone, PartialEq)]
pub struct AntoineCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl AntoineCoefficients {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoComponent {
    pub id: ComponentId,
    pub name: String,
    pub antoine: Option<AntoineCoefficients>,
    pub liquid_heat_capacity_j_per_mol_k: Option<f64>,
    pub vapor_heat_capacity_j_per_mol_k: Option<f64>,
}

impl ThermoComponent {
    pub fn new(id: impl Into<ComponentId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            antoine: None,
            liquid_heat_capacity_j_per_mol_k: None,
            vapor_heat_capacity_j_per_mol_k: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidPhaseModel {
    IdealSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporPhaseModel {
    IdealGas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermoMethod {
    pub liquid_phase_model: LiquidPhaseModel,
    pub vapor_phase_model: VaporPhaseModel,
}

impl Default for ThermoMethod {
    fn default() -> Self {
        Self {
            liquid_phase_model: LiquidPhaseModel::IdealSolution,
            vapor_phase_model: VaporPhaseModel::IdealGas,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoSystem {
    pub components: Vec<ThermoComponent>,
    pub method: ThermoMethod,
}

impl ThermoSystem {
    pub fn new(components: Vec<ThermoComponent>) -> Self {
        Self {
            components,
            method: ThermoMethod::default(),
        }
    }

    pub fn binary(components: [ThermoComponent; 2]) -> Self {
        Self::new(Vec::from(components))
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .map(|component| component.id.clone())
            .collect()
    }

    pub fn validate_mole_fractions(&self, mole_fractions: &[f64]) -> RfResult<()> {
        if self.components.is_empty() {
            return Err(RfError::thermo(
                "thermo system must contain at least one component",
            ));
        }

        if mole_fractions.len() != self.component_count() {
            return Err(RfError::invalid_input(format!(
                "expected {} mole fractions, received {}",
                self.component_count(),
                mole_fractions.len()
            )));
        }

        if mole_fractions.iter().any(|value| *value < 0.0) {
            return Err(RfError::invalid_input(
                "mole fractions must be non-negative",
            ));
        }

        let sum = mole_fractions.iter().sum::<f64>();
        if sum <= 0.0 {
            return Err(RfError::invalid_input(
                "mole fractions must sum to a positive value",
            ));
        }

        Ok(())
    }

    pub fn validate_state(&self, state: &ThermoState) -> RfResult<()> {
        if state.temperature_k <= 0.0 {
            return Err(RfError::invalid_input(
                "temperature must be greater than zero kelvin",
            ));
        }

        if state.pressure_pa <= 0.0 {
            return Err(RfError::invalid_input(
                "pressure must be greater than zero pascal",
            ));
        }

        self.validate_mole_fractions(&state.overall_mole_fractions)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoState {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl ThermoState {
    pub fn new(temperature_k: f64, pressure_pa: f64, overall_mole_fractions: Vec<f64>) -> Self {
        Self {
            temperature_k,
            pressure_pa,
            overall_mole_fractions,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseThermoState {
    pub label: PhaseLabel,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub mole_fractions: Vec<f64>,
}

impl PhaseThermoState {
    pub fn new(
        label: PhaseLabel,
        temperature_k: f64,
        pressure_pa: f64,
        mole_fractions: Vec<f64>,
    ) -> Self {
        Self {
            label,
            temperature_k,
            pressure_pa,
            mole_fractions,
        }
    }
}

pub trait ThermoProvider {
    fn system(&self) -> &ThermoSystem;

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>>;

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64>;
}

#[derive(Debug, Clone)]
pub struct PlaceholderThermoProvider {
    system: ThermoSystem,
}

impl PlaceholderThermoProvider {
    pub fn new(system: ThermoSystem) -> Self {
        Self { system }
    }
}

impl ThermoProvider for PlaceholderThermoProvider {
    fn system(&self) -> &ThermoSystem {
        &self.system
    }

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>> {
        self.system.validate_state(state)?;

        Err(RfError::not_implemented(
            "K-value estimation is not implemented yet",
        ))
    }

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64> {
        if state.temperature_k <= 0.0 {
            return Err(RfError::invalid_input(
                "temperature must be greater than zero kelvin",
            ));
        }

        if state.pressure_pa <= 0.0 {
            return Err(RfError::invalid_input(
                "pressure must be greater than zero pascal",
            ));
        }

        self.system.validate_mole_fractions(&state.mole_fractions)?;

        Err(RfError::not_implemented(
            "phase enthalpy evaluation is not implemented yet",
        ))
    }
}
