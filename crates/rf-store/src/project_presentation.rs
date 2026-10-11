use rf_types::units::{ALL_UNITS, DisplayUnitSet, LEGACY_QUANTITIES};
use serde::{Deserialize, Serialize, Serializer};

/// Project v2 / personal-default v1 projection. The in-memory set also contains
/// newer quantities, but this wire format must remain the original seven entries.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(try_from = "PresentationWire")]
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

impl StoredProjectPresentation {
    /// Never silently discard a choice that requires project v3.
    pub(crate) fn validate_legacy_projection(&self) -> Result<(), String> {
        for (quantity, unit) in self.display_units.entries() {
            if !LEGACY_QUANTITIES.contains(&quantity)
                && unit != quantity.definition().canonical_unit
            {
                return Err(format!(
                    "presentation.displayUnits: {} requires project v3; project v2 and personal defaults v1 cannot preserve this choice",
                    quantity.definition().id
                ));
            }
        }
        Ok(())
    }
}

impl TryFrom<PresentationWire> for StoredProjectPresentation {
    type Error = String;

    fn try_from(wire: PresentationWire) -> Result<Self, Self::Error> {
        let mut entries = Vec::with_capacity(wire.display_units.len());
        for (index, choice) in wire.display_units.into_iter().enumerate() {
            let quantity = LEGACY_QUANTITIES
                .iter()
                .copied()
                .find(|q| q.definition().id == choice.quantity_id)
                .ok_or_else(|| {
                    format!(
                        "presentation.displayUnits[{index}]: unknown quantity ID {:?} for this file version",
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
        // Only the version migration may fill new quantities. Missing original
        // entries and duplicates still fail the complete-set constructor below.
        entries.extend(
            DisplayUnitSet::si()
                .entries()
                .filter(|(q, _)| !LEGACY_QUANTITIES.contains(q)),
        );
        let display_units = DisplayUnitSet::try_from_entries(entries)
            .map_err(|error| format!("presentation.displayUnits: {error}"))?;
        Ok(Self { display_units })
    }
}

impl Serialize for StoredProjectPresentation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validate_legacy_projection()
            .map_err(serde::ser::Error::custom)?;
        PresentationWire {
            display_units: LEGACY_QUANTITIES
                .iter()
                .map(|quantity| DisplayUnitChoice {
                    quantity_id: quantity.definition().id.to_string(),
                    unit_id: self
                        .display_units
                        .unit_for(*quantity)
                        .definition()
                        .id
                        .to_string(),
                })
                .collect(),
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_types::units::{MeasurementUnit, QuantityKind};

    #[test]
    fn legacy_projection_preserves_seven_entries_and_rejects_lossy_writes() {
        let mut presentation = StoredProjectPresentation {
            display_units: DisplayUnitSet::engineering(),
        };
        let json = serde_json::to_string(&presentation).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["displayUnits"].as_array().unwrap().len(), 7);
        let restored: StoredProjectPresentation = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, presentation);
        assert_eq!(
            restored.display_units.unit_for(QuantityKind::Duration),
            MeasurementUnit::Second
        );
        presentation
            .display_units
            .set_unit(QuantityKind::Duration, MeasurementUnit::Minute)
            .unwrap();
        assert!(
            serde_json::to_string(&presentation)
                .unwrap_err()
                .to_string()
                .contains("v3")
        );
        let mut invalid = value.clone();
        invalid["displayUnits"].as_array_mut().unwrap().pop();
        assert!(serde_json::from_value::<StoredProjectPresentation>(invalid).is_err());
        let mut invalid = value;
        invalid["displayUnits"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"quantityId":"duration", "unitId":"second"}));
        assert!(serde_json::from_value::<StoredProjectPresentation>(invalid).is_err());
    }
}
