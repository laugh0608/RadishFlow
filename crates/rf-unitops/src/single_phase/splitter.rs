use super::{COMPONENT_FLOW_TOLERANCE_MOL_S, enthalpy_flow_w, verify_energy_balance};
use crate::{
    BuiltinUnitKind, SPLITTER_INLET_PORT, SPLITTER_OUTLET_A_PORT, SPLITTER_OUTLET_B_PORT, Splitter,
    UnitOperation, UnitOperationInputs, UnitOperationOutputs, UnitOperationServices,
    component_balance::stream_component_molar_flows,
};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{RfError, RfResult};

pub(crate) struct SinglePhaseSplitter<'a> {
    pub(crate) splitter: Splitter,
    pub(crate) model: &'a SensibleEnthalpyModel,
}

impl UnitOperation for SinglePhaseSplitter<'_> {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Splitter
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        let inlet = inputs.require_stream(SPLITTER_INLET_PORT)?;
        self.model
            .validate_domain(inlet.temperature_k, inlet.pressure_pa)
            .map_err(|error| error.with_related_stream_id(inlet.id.clone()))?;
        let mut outputs = self.splitter.run(services, inputs)?;
        for output in outputs.material_streams.values_mut() {
            if output.total_molar_flow_mol_s == 0.0 {
                output.phases.clear();
            }
        }
        let a = &outputs.material_streams[SPLITTER_OUTLET_A_PORT];
        let b = &outputs.material_streams[SPLITTER_OUTLET_B_PORT];
        let a_flows = stream_component_molar_flows(a)?;
        let b_flows = stream_component_molar_flows(b)?;
        for (component, incoming) in stream_component_molar_flows(inlet)? {
            let residual = (a_flows[&component] + b_flows[&component]) - incoming;
            if !residual.is_finite() || residual.abs() > COMPONENT_FLOW_TOLERANCE_MOL_S {
                return Err(RfError::thermo(format!(
                    "splitter component `{component}` residual is {residual} mol/s"
                ))
                .with_diagnostic_code("solver.single_phase.component_balance")
                .with_related_component_id(component)
                .with_related_stream_ids(vec![
                    inlet.id.clone(),
                    a.id.clone(),
                    b.id.clone(),
                ]));
            }
        }
        outputs.energy_balance = Some(verify_energy_balance(
            &[enthalpy_flow_w(inlet)?],
            0.0,
            &[a, b],
        )?);
        Ok(outputs)
    }
}
