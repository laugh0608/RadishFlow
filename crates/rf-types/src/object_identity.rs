use crate::{RfError, StreamId, UnitId};
use std::{collections::BTreeMap, fmt, str::FromStr};

/// Immutable identity of one creation of a flow object, even if its local ID is reused.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectCreationId(String);

impl ObjectCreationId {
    pub fn from_random_bytes(mut bytes: [u8; 16]) -> Self {
        use fmt::Write as _;
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let mut id = String::with_capacity(36);
        for (index, byte) in bytes.iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                id.push('-');
            }
            write!(id, "{byte:02x}").expect("writing to a string is infallible");
        }
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ObjectCreationId {
    type Err = RfError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != 36
            || bytes[14] != b'4'
            || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
            || !bytes.iter().enumerate().all(|(index, byte)| {
                if matches!(index, 8 | 13 | 18 | 23) {
                    *byte == b'-'
                } else {
                    matches!(byte, b'0'..=b'9' | b'a'..=b'f')
                }
            })
        {
            return Err(RfError::invalid_input(
                "expected a canonical lowercase UUID v4",
            ));
        }
        Ok(Self(value.to_owned()))
    }
}

impl fmt::Display for ObjectCreationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlowObjectIdentities {
    pub units: BTreeMap<UnitId, ObjectCreationId>,
    pub streams: BTreeMap<StreamId, ObjectCreationId>,
}
