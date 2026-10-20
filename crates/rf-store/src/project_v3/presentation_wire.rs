use rf_types::{
    RfError, RfResult,
    units::{ALL_QUANTITIES, ALL_UNITS, DisplayUnitSet},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PresentationWire {
    display_units: Vec<DisplayUnitChoice>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DisplayUnitChoice {
    quantity_id: String,
    unit_id: String,
}
impl PresentationWire {
    pub(super) fn into_display_units(self) -> RfResult<DisplayUnitSet> {
        let entries = self
            .display_units
            .into_iter()
            .enumerate()
            .map(|(i, choice)| {
                let quantity = ALL_QUANTITIES
                    .iter()
                    .copied()
                    .find(|q| q.definition().id == choice.quantity_id)
                    .ok_or_else(|| {
                        RfError::invalid_input(format!(
                            "presentation.displayUnits[{i}].quantityId: unknown quantity {}",
                            choice.quantity_id
                        ))
                    })?;
                let unit = ALL_UNITS
                    .iter()
                    .copied()
                    .find(|u| u.definition().id == choice.unit_id)
                    .ok_or_else(|| {
                        RfError::invalid_input(format!(
                            "presentation.displayUnits[{i}].unitId: unknown unit {}",
                            choice.unit_id
                        ))
                    })?;
                if !quantity.definition().units.contains(&unit) {
                    return Err(RfError::invalid_input(format!(
                        "presentation.displayUnits[{i}].unitId: incompatible unit {}",
                        choice.unit_id
                    )));
                }
                Ok((quantity, unit))
            })
            .collect::<RfResult<Vec<_>>>()?;
        DisplayUnitSet::try_from_entries(entries)
            .map_err(|e| RfError::invalid_input(format!("presentation.displayUnits: {e}")))
    }
}
impl From<&DisplayUnitSet> for PresentationWire {
    fn from(units: &DisplayUnitSet) -> Self {
        Self {
            display_units: units
                .entries()
                .map(|(quantity, unit)| DisplayUnitChoice {
                    quantity_id: quantity.definition().id.into(),
                    unit_id: unit.definition().id.into(),
                })
                .collect(),
        }
    }
}
