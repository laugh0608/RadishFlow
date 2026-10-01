use std::fmt;

use super::{ALL_QUANTITIES, ConversionError, MeasurementUnit, QuantityKind, affine_rule};

/// Complete, context-free display choices in the catalog's stable quantity order.
/// The private array prevents missing, duplicate or incompatible choices after construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayUnitSet {
    units: [MeasurementUnit; ALL_QUANTITIES.len()],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayUnitSetError {
    MissingQuantity(QuantityKind),
    DuplicateQuantity(QuantityKind),
    InvalidUnit {
        quantity: QuantityKind,
        cause: ConversionError,
    },
}

impl fmt::Display for DisplayUnitSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingQuantity(quantity) => {
                write!(f, "missing display unit for {}", quantity.definition().id)
            }
            Self::DuplicateQuantity(quantity) => {
                write!(f, "duplicate display unit for {}", quantity.definition().id)
            }
            Self::InvalidUnit { quantity, cause } => {
                write!(f, "display unit for {}: {cause}", quantity.definition().id)
            }
        }
    }
}
impl std::error::Error for DisplayUnitSetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUnit { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

impl Default for DisplayUnitSet {
    fn default() -> Self {
        Self::si()
    }
}

impl DisplayUnitSet {
    pub fn si() -> Self {
        Self {
            units: std::array::from_fn(|index| ALL_QUANTITIES[index].definition().canonical_unit),
        }
    }

    pub fn engineering() -> Self {
        Self {
            units: std::array::from_fn(|index| match ALL_QUANTITIES[index] {
                QuantityKind::AbsoluteTemperature => MeasurementUnit::Celsius,
                QuantityKind::AbsolutePressure => MeasurementUnit::Bar,
                QuantityKind::MolarFlow => MeasurementUnit::KilomolePerHour,
                quantity => quantity.definition().canonical_unit,
            }),
        }
    }

    pub fn try_from_entries(
        entries: impl IntoIterator<Item = (QuantityKind, MeasurementUnit)>,
    ) -> Result<Self, DisplayUnitSetError> {
        let mut units = [None; ALL_QUANTITIES.len()];
        for (quantity, unit) in entries {
            let index = quantity_index(quantity);
            if units[index].is_some() {
                return Err(DisplayUnitSetError::DuplicateQuantity(quantity));
            }
            validate_choice(quantity, unit)?;
            units[index] = Some(unit);
        }
        for (quantity, unit) in ALL_QUANTITIES.iter().zip(&units) {
            if unit.is_none() {
                return Err(DisplayUnitSetError::MissingQuantity(*quantity));
            }
        }
        Ok(Self {
            units: units.map(|unit| unit.expect("all quantity choices were checked")),
        })
    }

    pub fn unit_for(&self, quantity: QuantityKind) -> MeasurementUnit {
        self.units[quantity_index(quantity)]
    }

    /// Validate before mutation so a rejected edit leaves the complete set unchanged.
    pub fn set_unit(
        &mut self,
        quantity: QuantityKind,
        unit: MeasurementUnit,
    ) -> Result<(), DisplayUnitSetError> {
        validate_choice(quantity, unit)?;
        self.units[quantity_index(quantity)] = unit;
        Ok(())
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = (QuantityKind, MeasurementUnit)> + '_ {
        ALL_QUANTITIES
            .iter()
            .copied()
            .zip(self.units.iter().copied())
    }
}

fn quantity_index(quantity: QuantityKind) -> usize {
    ALL_QUANTITIES
        .iter()
        .position(|candidate| *candidate == quantity)
        .expect("every QuantityKind must be listed in ALL_QUANTITIES")
}

fn validate_choice(
    quantity: QuantityKind,
    unit: MeasurementUnit,
) -> Result<(), DisplayUnitSetError> {
    affine_rule(quantity, unit)
        .map(|_| ())
        .map_err(|cause| DisplayUnitSetError::InvalidUnit { quantity, cause })
}

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementUnit::*;
    use QuantityKind::*;

    #[test]
    fn presets_are_complete_and_keep_temperature_difference_semantics() {
        let si = DisplayUnitSet::si();
        let engineering = DisplayUnitSet::engineering();
        assert_eq!(si.entries().len(), 7);
        for quantity in ALL_QUANTITIES {
            assert_eq!(si.unit_for(*quantity), quantity.definition().canonical_unit);
        }
        assert_eq!(
            engineering.entries().collect::<Vec<_>>(),
            vec![
                (AbsoluteTemperature, Celsius),
                (TemperatureDifference, KelvinDifference),
                (AbsolutePressure, Bar),
                (MolarFlow, KilomolePerHour),
                (MoleFraction, MolePerMole),
                (MolarPhaseFraction, MolePerMole),
                (MolarEnthalpy, JoulePerMole),
            ]
        );
        assert_eq!(
            DisplayUnitSet::try_from_entries(engineering.entries()).unwrap(),
            engineering
        );
    }

    #[test]
    fn construction_requires_exactly_one_choice_per_quantity_and_normalizes_order() {
        let si = DisplayUnitSet::si();
        let mut entries = si.entries().collect::<Vec<_>>();
        entries.reverse();
        assert_eq!(
            DisplayUnitSet::try_from_entries(entries.clone()).unwrap(),
            si
        );
        entries.push((AbsolutePressure, Bar));
        assert_eq!(
            DisplayUnitSet::try_from_entries(entries),
            Err(DisplayUnitSetError::DuplicateQuantity(AbsolutePressure))
        );
        assert_eq!(
            DisplayUnitSet::try_from_entries(si.entries().filter(|(q, _)| *q != AbsolutePressure)),
            Err(DisplayUnitSetError::MissingQuantity(AbsolutePressure))
        );
    }

    #[test]
    fn invalid_replacement_is_atomic_and_never_accepts_contextual_units() {
        let mut units = DisplayUnitSet::si();
        for (quantity, unit) in [
            (AbsoluteTemperature, CelsiusDifference),
            (AbsolutePressure, BarGauge),
            (MolarFlow, KilogramPerSecond),
            (MolarFlow, StandardCubicMetrePerHour),
        ] {
            assert!(units.set_unit(quantity, unit).is_err());
            assert_eq!(units, DisplayUnitSet::si());
            let entries = units
                .entries()
                .map(|(q, u)| (q, if q == quantity { unit } else { u }));
            assert!(DisplayUnitSet::try_from_entries(entries).is_err());
        }
        units.set_unit(AbsolutePressure, Kilopascal).unwrap();
        assert_eq!(units.unit_for(AbsolutePressure), Kilopascal);
        assert_eq!(units.unit_for(AbsoluteTemperature), Kelvin);
    }
}
