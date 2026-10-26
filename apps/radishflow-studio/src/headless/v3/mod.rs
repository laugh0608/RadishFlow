//! Version 3 local automation. This module owns wire DTOs and request-local
//! bindings, never independent domain state, solver policy or disk mutation.
mod actions;
mod adoption;
mod confirmation;
mod enums;
mod equipment;
mod errors;
mod execute;
mod flow;
mod inputs;
mod issues;
mod protocol;
mod query;
mod records;
mod references;
mod results;
mod runs;
mod strict;
use crate::document_lifecycle_driver::{StudioProjectOpen, parse_studio_project};
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

fn output(response: &Response, stdout: &mut impl Write, stderr: &mut impl Write) -> i32 {
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
    if version.schema_version != 3 {
        return Err(Failure::new(
            Stage::Request,
            Code::UnsupportedVersion,
            "query/execute require schema_version 3",
        ));
    }
    Ok(bytes)
}
fn invalid_request(error: serde_json::Error) -> Failure {
    Failure::new(Stage::Request, Code::InvalidRequest, error.to_string())
}
fn validate_ids<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), Failure> {
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
    let bytes = std::fs::read(path)
        .map_err(|e| Failure::new(Stage::Project, Code::ProjectIo, e.to_string()))?;
    let file_sha256 = rf_store::sha256_hex(&bytes);
    let contents = std::str::from_utf8(&bytes)
        .map_err(|e| Failure::new(Stage::Project, Code::InvalidProject, e.to_string()))?;
    let opened = parse_studio_project(contents, path)
        .map_err(|e| Failure::new(Stage::Project, Code::InvalidProject, e.to_string()))?;
    let (document_id, flow_revision, project_schema_version, count, read_only) = match &opened {
        StudioProjectOpen::Editable(app) => (
            app.workspace.document.metadata.document_id.to_string(),
            app.workspace.document.revision,
            app.workspace
                .project_presentation
                .source_file_version()
                .expect("loaded project version"),
            app.workspace.engineering.inputs().tasks.len(),
            false,
        ),
        StudioProjectOpen::ReadOnly(project) => (
            project.document().metadata.document_id.to_string(),
            project.document().revision,
            3,
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
        },
    ))
}
fn query_file(path: &Path, response: &mut Response) -> Result<(), Failure> {
    let bytes = request_bytes(path)?;
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
mod tests;
