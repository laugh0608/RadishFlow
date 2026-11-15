//! Read saved project variables by the same stable reference Studio copies.

use super::v3::{
    self,
    protocol::{Code, Failure, Response, Stage},
};
use crate::StudioProjectOpen;
use rf_ui::variable_browser::{StableVariableReference, VariableReferenceError};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(super) const VERSION: u32 = 8;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryRequest {
    schema_version: u32,
    project: PathBuf,
    queries: Vec<Query>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    query: QueryKind,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum QueryKind {
    StableVariable { reference: String },
}

pub(super) fn query_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let request: QueryRequest = serde_json::from_slice(bytes).map_err(v3::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    v3::validate_ids(request.queries.iter().map(|query| query.id.as_str()))?;
    if request.project.as_os_str().is_empty() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "project path must not be empty",
        ));
    }

    let project_path = path
        .parent()
        .unwrap_or(Path::new("."))
        .join(request.project);
    let (opened, snapshot) = v3::load_stable_query_project(&project_path)?;
    response.project = Some(snapshot);
    for query in request.queries {
        let QueryKind::StableVariable { reference } = query.query;
        v3::query::push_reply(&query.id, read_variable(&opened, &reference), response);
    }
    Ok(())
}

fn read_variable(opened: &StudioProjectOpen, reference: &str) -> Result<Value, Failure> {
    let StudioProjectOpen::Editable(app) = opened else {
        return Err(Failure::new(
            Stage::Query,
            Code::UnsupportedCapability,
            "unsupported capability protects the whole project",
        ));
    };
    let reference: StableVariableReference = reference.parse().map_err(reference_error)?;
    let mut field = reference.read(&app.workspace).map_err(reference_error)?;
    field.write_via = None;
    let mut value = v3::query::flow_field(field);
    value["reference"] = json!(reference.to_string());
    Ok(value)
}

fn reference_error(error: VariableReferenceError) -> Failure {
    use VariableReferenceError::*;
    let (code, message) = match error {
        Malformed => (
            Code::InvalidVariableReference,
            "invalid stable variable reference",
        ),
        UnsupportedVersion => (
            Code::UnsupportedReferenceVersion,
            "unsupported stable variable reference version",
        ),
        UnsupportedField => (
            Code::UnsupportedField,
            "field or section is outside the stable reference vocabulary",
        ),
        DifferentDocument => (
            Code::DifferentDocument,
            "stable reference belongs to a different document",
        ),
        IdentitiesUnavailable | ObjectNotSaved => (
            Code::ObjectIdentitiesUnavailable,
            "saved object identities are unavailable",
        ),
        ObjectMissing => (Code::ObjectMissing, "referenced object does not exist"),
        ObjectRecreated => (Code::ObjectRecreated, "object creation identity differs"),
        FieldMissing => (
            Code::FieldMissing,
            "referenced field does not exist on this object",
        ),
    };
    Failure::new(Stage::Query, code, message)
}
