//! Validate the original JSON stream before typed deserialization. In particular,
//! map fields and internally tagged enums must not silently discard duplicate keys.
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::{collections::BTreeSet, fmt};

struct JsonValue;
impl<'de> Deserialize<'de> for JsonValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = JsonValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON with unique object keys and finite numbers")
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<JsonValue, E> {
                Ok(JsonValue)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<JsonValue, E> {
                Ok(JsonValue)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<JsonValue, E> {
                Ok(JsonValue)
            }
            fn visit_f64<E: de::Error>(self, n: f64) -> Result<JsonValue, E> {
                if n.is_finite() {
                    Ok(JsonValue)
                } else {
                    Err(E::custom("non-finite number"))
                }
            }
            fn visit_str<E: de::Error>(self, _: &str) -> Result<JsonValue, E> {
                Ok(JsonValue)
            }
            fn visit_unit<E: de::Error>(self) -> Result<JsonValue, E> {
                Ok(JsonValue)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut values: A) -> Result<JsonValue, A::Error> {
                while values.next_element::<JsonValue>()?.is_some() {}
                Ok(JsonValue)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut values: A) -> Result<JsonValue, A::Error> {
                let mut keys = BTreeSet::new();
                while let Some(key) = values.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(de::Error::custom(format!("duplicate key {key:?}")));
                    }
                    values.next_value::<JsonValue>()?;
                }
                Ok(JsonValue)
            }
        }
        d.deserialize_any(JsonVisitor)
    }
}

pub fn validate(bytes: &[u8]) -> Result<(), serde_json::Error> {
    serde_json::from_slice::<JsonValue>(bytes).map(|_| ())
}
