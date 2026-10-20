use super::{EngineeringDataError, EngineeringDataErrorKind};
use std::{fmt, str::FromStr};

/// ISO calendar date, with no implicit timezone or access-time inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceAccessDate {
    year: u16,
    month: u8,
    day: u8,
}

impl FromStr for SourceAccessDate {
    type Err = EngineeringDataError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid =
            || EngineeringDataError::new("accessedOn", EngineeringDataErrorKind::InvalidAccessDate);
        let bytes = value.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes
                .iter()
                .enumerate()
                .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
        {
            return Err(invalid());
        }
        // Slices are safe after the ASCII/length checks above.
        let year: u16 = value[..4].parse().map_err(|_| invalid())?;
        let month: u8 = value[5..7].parse().map_err(|_| invalid())?;
        let day: u8 = value[8..].parse().map_err(|_| invalid())?;
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return Err(invalid()),
        };
        if day == 0 || day > max_day {
            return Err(invalid());
        }
        Ok(Self { year, month, day })
    }
}

impl fmt::Display for SourceAccessDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
