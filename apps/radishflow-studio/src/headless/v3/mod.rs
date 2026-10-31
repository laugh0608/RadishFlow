//! Version 3 local automation. This module owns wire DTOs and request-local
//! bindings, never independent domain state, solver policy or disk mutation.
pub(super) mod actions;
pub(super) mod adoption;
pub(super) mod confirmation;
pub(super) mod enums;
pub(super) mod equipment;
pub(super) mod errors;
pub(super) mod execute;
pub(super) mod flow;
pub(super) mod inputs;
pub(super) mod issues;
pub(super) mod material;
pub(super) mod protocol;
pub(super) mod query;
pub(super) mod records;
pub(super) mod references;
pub(super) mod results;
pub(super) mod runs;
pub(super) mod strict;
#[cfg(test)]
use crate::document_lifecycle_driver::parse_studio_project;
use crate::document_lifecycle_driver::{StudioProjectOpen, parse_studio_project_with_support};
use protocol::*;
use std::{collections::BTreeSet, io::Write, path::Path};

pub(super) fn query_args(
    args: &[std::ffi::OsString],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if let [path] = args {
        return query_cli(Path::new(path), stdout, stderr);
    }
    let mut response = Response::default();
    response.fail(Failure::new(
        Stage::Request,
        Code::InvalidRequest,
        "query requires exactly one request file",
    ));
    output(&response, stdout, stderr)
}

pub(super) fn execute_args(
    args: &[std::ffi::OsString],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let mut response = Response::default();
    let result = match args {
        [path] => execute::file(Path::new(path), &mut response),
        _ => Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "execute requires exactly one request file",
        )),
    };
    if let Err(error) = result {
        response.fail(error);
    }
    output(&response, stdout, stderr)
}

pub(super) fn query_cli(path: &Path, stdout: &mut impl Write, stderr: &mut impl Write) -> i32 {
    let mut response = Response::default();
    if let Err(error) = query_file(path, &mut response) {
        response.fail(error);
    }
    output(&response, stdout, stderr)
}

pub(super) fn output(response: &Response, stdout: &mut impl Write, stderr: &mut impl Write) -> i32 {
    let bytes = serde_json::to_vec_pretty(response).map(|mut bytes| {
        bytes.push(b'\n');
        bytes
    });
    match bytes {
        Ok(bytes) => match stdout.write_all(&bytes).and_then(|_| stdout.flush()) {
            Ok(()) => response.status.exit_code(),
            Err(error) => super::output_error(stderr, error),
        },
        Err(error) => {
            let _ = writeln!(stderr, "headless v3 serialization failed: {error}");
            5
        }
    }
}

fn request_bytes(path: &Path) -> Result<Vec<u8>, Failure> {
    let bytes = std::fs::read(path)
        .map_err(|e| Failure::new(Stage::Request, Code::InvalidRequest, e.to_string()))?;
    strict::validate(&bytes).map_err(invalid_request)?;
    #[derive(serde::Deserialize)]
    struct Version {
        schema_version: u32,
    }
    let version: Version = serde_json::from_slice(&bytes).map_err(invalid_request)?;
    if !matches!(version.schema_version, 3..=6) {
        return Err(Failure::new(
            Stage::Request,
            Code::UnsupportedVersion,
            "query/execute require schema_version 3, 4, 5 or 6",
        ));
    }
    Ok(bytes)
}
pub(super) fn invalid_request(error: serde_json::Error) -> Failure {
    Failure::new(Stage::Request, Code::InvalidRequest, error.to_string())
}
pub(super) fn validate_ids<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), Failure> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() || !seen.insert(id) {
            return Err(Failure::new(
                Stage::Request,
                Code::InvalidRequest,
                "ids must be nonblank and unique within steps/queries",
            ));
        }
    }
    Ok(())
}
fn load_project(path: &Path) -> Result<(StudioProjectOpen, ProjectSnapshot), Failure> {
    load_project_version(path, 3)
}
pub(super) fn load_project_version(
    path: &Path,
    protocol_version: u32,
) -> Result<(StudioProjectOpen, ProjectSnapshot), Failure> {
    let bytes = std::fs::read(path)
        .map_err(|e| Failure::new(Stage::Project, Code::ProjectIo, e.to_string()))?;
    let file_sha256 = rf_store::sha256_hex(&bytes);
    let contents = std::str::from_utf8(&bytes)
        .map_err(|e| Failure::new(Stage::Project, Code::InvalidProject, e.to_string()))?;
    if rf_store::project_v3::probe_version(contents)
        .map_err(|e| Failure::new(Stage::Project, Code::InvalidProject, e.to_string()))?
        > protocol_version.min(4)
    {
        return Err(Failure::new(
            Stage::Project,
            Code::UnsupportedCapability,
            "project schema version is not supported by this CLI protocol",
        ));
    }
    let support = if protocol_version <= 4 {
        rf_store::project_v4::FlowPlanSupport::Sequential
    } else if protocol_version == 5 {
        rf_store::project_v4::FlowPlanSupport::Simultaneous
    } else {
        rf_store::project_v4::FlowPlanSupport::EquationOriented
    };
    let opened = parse_studio_project_with_support(contents, path, support)
        .map_err(|e| Failure::new(Stage::Project, Code::InvalidProject, e.to_string()))?;
    let (document_id, flow_revision, project_schema_version, count, read_only) = match &opened {
        StudioProjectOpen::Editable(app) => (
            app.workspace.document.metadata.document_id.to_string(),
            app.workspace.document.revision,
            app.workspace
                .project_presentation
                .source_file_version()
                .expect("loaded project version"),
            Some(app.workspace.engineering.inputs().tasks.len()),
            false,
        ),
        StudioProjectOpen::ReadOnly(project) => (
            project.metadata().document_id.to_string(),
            project.revision(),
            project.schema_version(),
            project.task_count(),
            true,
        ),
    };
    Ok((
        opened,
        ProjectSnapshot {
            document_id,
            project_schema_version,
            flow_revision,
            file_sha256,
            read_only,
            read_only_reason: read_only.then_some(Code::UnsupportedCapability),
            equipment_task_count: count,
            in_memory_required_project_schema_version: None,
            in_memory_required_capabilities: None,
        },
    ))
}
fn query_file(path: &Path, response: &mut Response) -> Result<(), Failure> {
    let bytes = request_bytes(path)?;
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(invalid_request)?["schema_version"]
        == 6
    {
        response.schema_version = 6;
        return super::planned::query_file::<super::v6::Plan>(path, &bytes, response);
    }
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(invalid_request)?["schema_version"]
        == 5
    {
        response.schema_version = 5;
        return super::planned::query_file::<super::v5::Plan>(path, &bytes, response);
    }
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(invalid_request)?["schema_version"]
        == 4
    {
        response.schema_version = 4;
        return super::v4::query_file::<super::v4::protocol::Plan>(path, &bytes, response);
    }
    let request: QueryRequest = serde_json::from_slice(&bytes).map_err(invalid_request)?;
    debug_assert_eq!(request.schema_version, 3);
    validate_ids(request.queries.iter().map(|q| q.id.as_str()))?;
    if request.project.as_os_str().is_empty() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "project path must not be empty",
        ));
    }
    let (opened, snapshot) = load_project(
        &path
            .parent()
            .unwrap_or(Path::new("."))
            .join(&request.project),
    )?;
    response.project = Some(snapshot);
    query::queries(&opened, &request.queries, response);
    Ok(())
}

#[cfg(test)]
pub(super) mod tests;
