pub use rf_store::strict_json::validate;
use serde::{Deserialize, Deserializer};

/// Optional means omitted. An explicit null must not silently choose a default.
pub fn optional<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
