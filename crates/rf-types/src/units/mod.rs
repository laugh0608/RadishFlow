//! Shared quantity semantics and measurement units. Engineering values remain canonical SI.
//! Equipment identity is `crate::UnitId`; `MeasurementUnit` is a different domain concept.
mod catalog;
pub use catalog::{
    ALL_QUANTITIES, ALL_UNITS, ConversionRule, Dimension, MeasurementUnit, QuantityDefinition,
    QuantityKind, UnitDefinition,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRequirement {
    ReferencePressure,
    MixtureMolarMass,
    StandardVolumeBasisAndStateRelation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    UnknownUnit {
        token: String,
    },
    AmbiguousUnit {
        token: String,
    },
    IncompatibleUnit {
        quantity: QuantityKind,
        unit: MeasurementUnit,
    },
    ContextRequired {
        unit: MeasurementUnit,
        requirement: ContextRequirement,
    },
    NonFiniteValue,
    Overflow,
}
impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUnit { token } => write!(f, "unknown measurement unit {token:?}"),
            Self::AmbiguousUnit { token } => {
                write!(f, "ambiguous measurement unit {token:?} for this quantity")
            }
            Self::IncompatibleUnit { quantity, unit } => write!(
                f,
                "unit {} is incompatible with {}",
                unit.definition().id,
                quantity.definition().id
            ),
            Self::ContextRequired { unit, requirement } => write!(
                f,
                "unit {} requires {requirement:?}; contextual conversion is not implemented",
                unit.definition().id
            ),
            Self::NonFiniteValue => f.write_str("conversion requires a finite value"),
            Self::Overflow => f.write_str("unit conversion exceeds the finite numeric range"),
        }
    }
}
impl std::error::Error for ConversionError {}

/// Resolve a stable ID, symbol or documented alias within a quantity's semantics.
/// Matching is case sensitive; surrounding whitespace is ignored. No heuristic parsing.
/// Known contextual conversions return ContextRequired rather than an assumed coefficient.
pub fn resolve_unit(
    quantity: QuantityKind,
    token: &str,
) -> Result<MeasurementUnit, ConversionError> {
    let token = token.trim();
    let matches = || {
        ALL_UNITS.iter().copied().filter(|unit| {
            let d = unit.definition();
            d.id == token || d.symbol == token || d.aliases.contains(&token)
        })
    };
    let mut compatible = matches().filter(|unit| quantity.definition().units.contains(unit));
    if let Some(unit) = compatible.next() {
        return if compatible.next().is_none() {
            Ok(unit)
        } else {
            Err(ConversionError::AmbiguousUnit {
                token: token.into(),
            })
        };
    }
    let mut known = matches();
    match known.next() {
        None => Err(ConversionError::UnknownUnit {
            token: token.into(),
        }),
        Some(unit) if known.next().is_none() => affine_rule(quantity, unit).map(|_| unit),
        Some(_) => Err(ConversionError::AmbiguousUnit {
            token: token.into(),
        }),
    }
}

fn affine_rule(
    quantity: QuantityKind,
    unit: MeasurementUnit,
) -> Result<(f64, f64), ConversionError> {
    match unit.definition().conversion {
        ConversionRule::Affine { scale, offset } if quantity.definition().units.contains(&unit) => {
            Ok((scale, offset))
        }
        ConversionRule::Contextual {
            target,
            requirement,
        } if target == quantity => Err(ConversionError::ContextRequired { unit, requirement }),
        _ => Err(ConversionError::IncompatibleUnit { quantity, unit }),
    }
}

/// Convert through the quantity's canonical SI unit, retaining full computation precision.
/// This checks unit semantics and numeric range, not physical constraints (e.g. positive T).
/// No document, preferences or solve state are read or changed.
pub fn convert(
    value: f64,
    quantity: QuantityKind,
    from: MeasurementUnit,
    to: MeasurementUnit,
) -> Result<f64, ConversionError> {
    let (from_scale, from_offset) = affine_rule(quantity, from)?;
    let (to_scale, to_offset) = affine_rule(quantity, to)?;
    if !value.is_finite() {
        return Err(ConversionError::NonFiniteValue);
    }
    // Preserve canonical/no-op values bit-for-bit; unnecessary offset round trips lose precision.
    if from == to {
        return Ok(value);
    }
    let canonical = value.mul_add(from_scale, from_offset);
    let result = (canonical - to_offset) / to_scale;
    if !canonical.is_finite() || !result.is_finite() {
        return Err(ConversionError::Overflow);
    }
    Ok(result)
}

pub fn to_canonical(
    value: f64,
    quantity: QuantityKind,
    from: MeasurementUnit,
) -> Result<f64, ConversionError> {
    convert(value, quantity, from, quantity.definition().canonical_unit)
}
pub fn from_canonical(
    value: f64,
    quantity: QuantityKind,
    to: MeasurementUnit,
) -> Result<f64, ConversionError> {
    convert(value, quantity, quantity.definition().canonical_unit, to)
}

#[cfg(test)]
mod tests;
