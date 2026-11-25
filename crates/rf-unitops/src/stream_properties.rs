use rf_flash::{TpFlashInput, estimate_bubble_dew_window};
use rf_model::{MaterialStreamState, PhaseState};
use rf_thermo::ThermoProvider;
use rf_types::{PhaseLabel, RfError, RfResult};

use super::{UnitOperationServices, validated_pressure, validated_temperature};
use crate::component_balance::{normalized_composition, validated_total_flow};

pub(super) fn stream_composition_vector(
    stream: &MaterialStreamState,
    thermo: &dyn ThermoProvider,
) -> RfResult<Vec<f64>> {
    let normalized = normalized_composition(stream)?;
    let component_ids = thermo.system().component_ids();

    if normalized.len() != component_ids.len() {
        return Err(RfError::invalid_input(format!(
            "stream `{}` composition has {} components, but thermo system expects {}",
            stream.id,
            normalized.len(),
            component_ids.len()
        ))
        .with_related_composition_stream_id(stream.id.clone()));
    }

    component_ids
        .into_iter()
        .map(|component_id| {
            normalized.get(&component_id).copied().ok_or_else(|| {
                RfError::invalid_input(format!(
                    "stream `{}` is missing thermo component `{}`",
                    stream.id, component_id
                ))
                .with_related_composition_stream_id(stream.id.clone())
            })
        })
        .collect()
}

pub(super) fn attach_bubble_dew_window(
    thermo: &dyn ThermoProvider,
    stream: &mut MaterialStreamState,
) -> RfResult<()> {
    if stream.total_molar_flow_mol_s <= 0.0 || stream.overall_mole_fractions.is_empty() {
        stream.bubble_dew_window = None;
        return Ok(());
    }

    let composition = stream_composition_vector(stream, thermo)?;
    stream.bubble_dew_window = Some(estimate_bubble_dew_window(
        thermo,
        stream.temperature_k,
        stream.pressure_pa,
        composition,
    )?);
    Ok(())
}

pub(super) fn attach_overall_phase_enthalpy(
    services: &UnitOperationServices<'_>,
    stream: &mut MaterialStreamState,
) -> RfResult<()> {
    if stream.total_molar_flow_mol_s <= 0.0 || stream.overall_mole_fractions.is_empty() {
        return Ok(());
    }

    let Some(flash_solver) = services.flash_solver else {
        return Ok(());
    };

    let thermo = services.require_thermo()?;
    let flash_input = TpFlashInput::new(
        stream.id.clone(),
        stream.name.clone(),
        validated_temperature(stream)?,
        validated_pressure(stream)?,
        validated_total_flow(stream)?,
        stream_composition_vector(stream, thermo)?,
    );
    let flash_result = flash_solver.flash(thermo, &flash_input)?;
    let overall_enthalpy = flash_result
        .stream
        .phases
        .iter()
        .find(|phase| phase.label == PhaseLabel::Overall)
        .and_then(|phase| phase.molar_enthalpy_j_per_mol);
    if let Some(overall_phase) = stream
        .phases
        .iter_mut()
        .find(|phase| phase.label == PhaseLabel::Overall)
    {
        overall_phase.molar_enthalpy_j_per_mol = overall_enthalpy;
    } else {
        let mut overall_phase = PhaseState::new(
            PhaseLabel::Overall,
            1.0,
            stream.overall_mole_fractions.clone(),
        );
        overall_phase.molar_enthalpy_j_per_mol = overall_enthalpy;
        stream.phases.push(overall_phase);
    }
    Ok(())
}
