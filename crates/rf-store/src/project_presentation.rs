use rf_types::units::{ALL_QUANTITIES, ALL_UNITS, DisplayUnitSet};
use serde::{Deserialize, Serialize};

/// Persistence projection; only stable catalog IDs cross the file boundary.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "PresentationWire", into = "PresentationWire")]
pub struct StoredProjectPresentation {
    pub display_units: DisplayUnitSet,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PresentationWire {
    display_units: Vec<DisplayUnitChoice>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DisplayUnitChoice {
    quantity_id: String,
    unit_id: String,
}

impl TryFrom<PresentationWire> for StoredProjectPresentation {
    type Error = String;

    fn try_from(wire: PresentationWire) -> Result<Self, Self::Error> {
        let mut entries = Vec::with_capacity(wire.display_units.len());
        for (index, choice) in wire.display_units.into_iter().enumerate() {
            let quantity = ALL_QUANTITIES
                .iter()
                .copied()
                .find(|q| q.definition().id == choice.quantity_id)
                .ok_or_else(|| {
                    format!(
                        "presentation.displayUnits[{index}]: unknown quantity ID {:?}",
                        choice.quantity_id
                    )
                })?;
            let unit = ALL_UNITS
                .iter()
                .copied()
                .find(|u| u.definition().id == choice.unit_id)
                .ok_or_else(|| {
                    format!(
                        "presentation.displayUnits[{index}]: unknown unit ID {:?} for {}",
                        choice.unit_id, choice.quantity_id
                    )
                })?;
            entries.push((quantity, unit));
        }
        let display_units = DisplayUnitSet::try_from_entries(entries)
            .map_err(|error| format!("presentation.displayUnits: {error}"))?;
        Ok(Self { display_units })
    }
}

impl From<StoredProjectPresentation> for PresentationWire {
    fn from(presentation: StoredProjectPresentation) -> Self {
        Self {
            display_units: presentation
                .display_units
                .entries()
                .map(|(quantity, unit)| DisplayUnitChoice {
                    quantity_id: quantity.definition().id.to_string(),
                    unit_id: unit.definition().id.to_string(),
                })
                .collect(),
        }
    }
}
