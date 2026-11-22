//! Strict versioned wire requests, normalized without changing v1's contract.
use super::{Failure, failure, protocol::*, workflow::*};
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{fmt, marker::PhantomData, path::PathBuf};

pub struct ExecutionRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub document_id: String,
    pub expected_revision: u64,
    pub cache_root: PathBuf,
    pub auth_cache_index: PathBuf,
    pub package_id: Option<String>,
    pub edits: Edits,
    pub reads: Vec<ReferenceTarget>,
}
pub enum Edits {
    Writes(Vec<Write>),
    Steps(Vec<Step>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRequest {
    schema_version: u32,
    project: PathBuf,
    document_id: String,
    expected_revision: u64,
    cache_root: PathBuf,
    auth_cache_index: PathBuf,
    package_id: Option<String>,
    steps: Vec<Step>,
    reads: Vec<ReferenceTarget>,
}

pub fn parse(bytes: &[u8]) -> Result<ExecutionRequest, Failure> {
    // Probe only the version, then deserialize the original bytes with a strict schema.
    // Do not round-trip through Value: that would discard duplicate object keys.
    #[derive(Deserialize)]
    struct Version {
        schema_version: u32,
    }
    let version: Version = serde_json::from_slice(bytes).map_err(invalid_json)?;
    match version.schema_version {
        1 => {
            let r: RunRequest = serde_json::from_slice(bytes).map_err(invalid_json)?;
            Ok(ExecutionRequest {
                schema_version: r.schema_version,
                project: r.project,
                document_id: r.document_id,
                expected_revision: r.expected_revision,
                cache_root: r.cache_root,
                auth_cache_index: r.auth_cache_index,
                package_id: r.package_id,
                edits: Edits::Writes(r.writes),
                reads: r.reads.into_iter().map(ReferenceTarget::from).collect(),
            })
        }
        2 => {
            let r: WorkflowRequest = serde_json::from_slice(bytes).map_err(invalid_json)?;
            Ok(ExecutionRequest {
                schema_version: r.schema_version,
                project: r.project,
                document_id: r.document_id,
                expected_revision: r.expected_revision,
                cache_root: r.cache_root,
                auth_cache_index: r.auth_cache_index,
                package_id: r.package_id,
                edits: Edits::Steps(r.steps),
                reads: r.reads,
            })
        }
        _ => Err(failure(
            "request",
            "unsupported_version",
            "supported schema_version values are 1 and 2",
        )),
    }
}
fn invalid_json(error: serde_json::Error) -> Failure {
    failure("request", "invalid_json", error.to_string())
}

// Derived struct decoders also accept positional sequences. This protocol
// requires named JSON objects for request records and identity references.
#[derive(PartialEq, Eq)]
pub(super) struct JsonObject<T>(pub(super) T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for JsonObject<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = JsonObject<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(JsonObject)
            }
        }
        de.deserialize_map(ObjectVisitor(PhantomData))
    }
}
