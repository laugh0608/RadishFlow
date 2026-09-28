use rf_types::units::{ConversionError, MeasurementUnit, QuantityKind, resolve_unit, to_canonical};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputUnitOrigin {
    Display,
    Explicit,
    Suffix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericParseError {
    Syntax,
    Conversion(ConversionError),
    UnitConflict {
        selected: MeasurementUnit,
        suffix: MeasurementUnit,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericParseOutcome {
    Incomplete,
    Invalid(NumericParseError),
    Value {
        si: f64,
        unit: MeasurementUnit,
        origin: InputUnitOrigin,
    },
}

/// Locale-independent scalar grammar. A suffix is resolved by the shared quantity catalog.
pub fn parse_numeric_input(
    raw: &str,
    quantity: QuantityKind,
    unit: MeasurementUnit,
    origin: InputUnitOrigin,
) -> NumericParseOutcome {
    use NumericParseOutcome::*;
    let text = raw.trim();
    if text.parse::<f64>().is_ok_and(|value| !value.is_finite()) {
        return Invalid(NumericParseError::Conversion(
            ConversionError::NonFiniteValue,
        ));
    }
    let bytes = text.as_bytes();
    let mut i = usize::from(bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')));
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - start;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - start;
    }
    if digits == 0 {
        return if i == bytes.len() {
            Incomplete
        } else {
            Invalid(NumericParseError::Syntax)
        };
    }
    if bytes.get(i).is_some_and(|b| matches!(b, b'e' | b'E')) {
        i += 1;
        if bytes.get(i).is_some_and(|b| matches!(b, b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return if i == bytes.len() {
                Incomplete
            } else {
                Invalid(NumericParseError::Syntax)
            };
        }
    }
    let Ok(value) = text[..i].parse::<f64>() else {
        return Invalid(NumericParseError::Syntax);
    };
    let suffix = text[i..].trim();
    let (unit, origin) = if suffix.is_empty() {
        (unit, origin)
    } else {
        let suffix = match resolve_unit(quantity, suffix) {
            Ok(unit) => unit,
            Err(error) => return Invalid(NumericParseError::Conversion(error)),
        };
        if origin == InputUnitOrigin::Explicit && suffix != unit {
            return Invalid(NumericParseError::UnitConflict {
                selected: unit,
                suffix,
            });
        }
        (
            suffix,
            if origin == InputUnitOrigin::Explicit {
                origin
            } else {
                InputUnitOrigin::Suffix
            },
        )
    };
    match to_canonical(value, quantity, unit) {
        Ok(si) => Value { si, unit, origin },
        Err(error) => Invalid(NumericParseError::Conversion(error)),
    }
}
