//! Counts actual attempted service calls, including calls made within flash and
//! calls that fail. No count is inferred from iteration or local evaluation counts.
use rf_flash::{TpFlashInput, TpFlashResult, TpFlashSolver};
use rf_thermo::{
    BubbleDewPressureInput, BubbleDewPressures, BubbleDewTemperatureInput, BubbleDewTemperatures,
    PhaseThermoState, ThermoProvider, ThermoState, ThermoSystem,
};
use rf_types::RfResult;
use std::cell::Cell;

pub(super) struct CountedThermo<'a> {
    pub provider: &'a dyn ThermoProvider,
    pub calls: Cell<usize>,
}
impl ThermoProvider for CountedThermo<'_> {
    fn system(&self) -> &ThermoSystem {
        self.provider.system()
    }
    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>> {
        self.calls.set(self.calls.get() + 1);
        self.provider.estimate_k_values(state)
    }
    fn estimate_bubble_dew_pressures(
        &self,
        input: &BubbleDewPressureInput,
    ) -> RfResult<BubbleDewPressures> {
        self.calls.set(self.calls.get() + 1);
        self.provider.estimate_bubble_dew_pressures(input)
    }
    fn estimate_bubble_dew_temperatures(
        &self,
        input: &BubbleDewTemperatureInput,
    ) -> RfResult<BubbleDewTemperatures> {
        self.calls.set(self.calls.get() + 1);
        self.provider.estimate_bubble_dew_temperatures(input)
    }
    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64> {
        self.calls.set(self.calls.get() + 1);
        self.provider.phase_molar_enthalpy(state)
    }
}
pub(super) struct CountedFlash<'a> {
    pub solver: &'a dyn TpFlashSolver,
    pub calls: Cell<usize>,
}
impl TpFlashSolver for CountedFlash<'_> {
    fn flash(&self, thermo: &dyn ThermoProvider, input: &TpFlashInput) -> RfResult<TpFlashResult> {
        self.calls.set(self.calls.get() + 1);
        self.solver.flash(thermo, input)
    }
}
