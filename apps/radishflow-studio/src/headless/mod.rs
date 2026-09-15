//! One-shot, local, read-only-on-disk automation. No GUI bootstrap or cache seeding.
mod protocol;
use crate::modeling_actions::{
    ModelingAction, ModelingActionEffect, ModelingActionRequest, dispatch_modeling_action,
};
use crate::{
    StudioAppAuthCacheContext, StudioAppFacade, StudioAppResultDispatch, StudioWorkspaceRunOutcome,
    WorkspaceRunPackageSelection, load_project_app_state,
};
use protocol::*;
use rf_ui::variable_browser::{BrowseError, VariableBrowser, VariableValue};
use rf_ui::variable_commands::{VariableWriteError, VariableWriteRequest};
use rf_ui::{AppState, RunStatus, latest_snapshot, stale_snapshot};
use std::{ffi::OsString, io::Write as IoWrite, path::Path, time::SystemTime};

pub const HELP: &str = "Usage: radishflow-studio --headless inspect <project.rfproj.json>\n       radishflow-studio --headless run <request.json>\n       radishflow-studio --headless --help\n\ninspect returns document identity, revision and variables as JSON.\nrun applies SI inputs in memory, solves using the explicit local cache and reads variables.\nPaths in a run request are relative to that request file. No input files are modified.\nExit codes: 0 success, 2 request/load/write/read error, 3 blocked, 4 run failed, 5 output error.\n";

/// Called before the GUI runtime is initialized. stdout contains exactly one JSON response
/// (or help); diagnostics go to stderr only if writing stdout itself fails.
pub fn run_cli(args: &[OsString], stdout: &mut impl IoWrite, stderr: &mut impl IoWrite) -> i32 {
    if args.len() == 1 && args[0] == "--help" {
        return match stdout
            .write_all(HELP.as_bytes())
            .and_then(|_| stdout.flush())
        {
            Ok(()) => 0,
            Err(error) => output_error(stderr, error),
        };
    }
    let mut response = Response::default();
    let result = match args {
        [command, path] if command == "inspect" => inspect(Path::new(path), &mut response),
        [command, path] if command == "run" => run_request_file(Path::new(path), &mut response),
        _ => Err(failure("arguments", "invalid_arguments", HELP)),
    };
    if let Err(error) = result {
        response.status = Status::Error;
        response.error = Some(error);
    }
    let output = match serde_json::to_vec_pretty(&response) {
        Ok(mut bytes) => {
            bytes.push(b'\n');
            bytes
        }
        Err(error) => {
            let _ = writeln!(stderr, "headless JSON serialization failed: {error}");
            return 5;
        }
    };
    match stdout.write_all(&output).and_then(|_| stdout.flush()) {
        Ok(()) => response.status.exit_code(),
        Err(error) => output_error(stderr, error),
    }
}

fn output_error(stderr: &mut impl IoWrite, error: std::io::Error) -> i32 {
    let _ = writeln!(stderr, "headless output failed: {error}");
    5
}

fn failure(stage: &'static str, code: &'static str, message: impl Into<String>) -> Failure {
    Failure {
        stage,
        code,
        message: message.into(),
        index: None,
    }
}
fn domain_failure(stage: &'static str, error: rf_types::RfError) -> Failure {
    failure(stage, error.code().as_str(), error.to_string())
}
fn document(app: &AppState) -> Document {
    Document {
        id: app
            .workspace
            .document
            .metadata
            .document_id
            .as_str()
            .to_string(),
        revision: app.workspace.document.revision,
    }
}
fn browser(app: &AppState) -> VariableBrowser<'_> {
    VariableBrowser::new(
        &app.workspace.document,
        latest_snapshot(&app.workspace),
        stale_snapshot(&app.workspace),
    )
}

fn inspect(path: &Path, response: &mut Response) -> Result<(), Failure> {
    let app = load_project_app_state(path).map_err(|e| domain_failure("project", e))?;
    response.document = Some(document(&app));
    response.variables = browser(&app)
        .search("")
        .into_iter()
        .map(Variable::from)
        .collect();
    Ok(())
}

fn run_request_file(path: &Path, response: &mut Response) -> Result<(), Failure> {
    let bytes = std::fs::read(path).map_err(|e| failure("request", "request_io", e.to_string()))?;
    let request: RunRequest = serde_json::from_slice(&bytes)
        .map_err(|e| failure("request", "invalid_json", e.to_string()))?;
    if request.schema_version != SCHEMA_VERSION {
        return Err(failure(
            "request",
            "unsupported_version",
            "supported schema_version is 1",
        ));
    }
    for (name, value) in [
        ("project", &request.project),
        ("cache_root", &request.cache_root),
        ("auth_cache_index", &request.auth_cache_index),
    ] {
        if value.as_os_str().is_empty() {
            return Err(failure(
                "request",
                "empty_path",
                format!("{name} must not be empty"),
            ));
        }
    }
    let base = path.parent().unwrap_or(Path::new("."));
    let mut app = load_project_app_state(&base.join(&request.project))
        .map_err(|e| domain_failure("project", e))?;
    response.document = Some(document(&app));
    if request.document_id != app.workspace.document.metadata.document_id.as_str() {
        return Err(failure(
            "project",
            "different_document",
            "request document_id does not match the project",
        ));
    }
    if request.expected_revision != app.workspace.document.revision {
        return Err(failure(
            "project",
            "revision_conflict",
            format!(
                "expected revision {}, found {}",
                request.expected_revision, app.workspace.document.revision
            ),
        ));
    }
    let index = rf_store::read_auth_cache_index(base.join(&request.auth_cache_index))
        .map_err(|e| domain_failure("cache", e))?;
    let root = base.join(&request.cache_root);
    let context = StudioAppAuthCacheContext::new(&root, &index);
    for (index, write) in request.writes.into_iter().enumerate() {
        let receipt = app
            .write_variable(
                VariableWriteRequest {
                    variable: write
                        .target
                        .variable_id(&app.workspace.document.metadata.document_id),
                    expected_revision: app.workspace.document.revision,
                    value: match write.value {
                        Value::Number(n) => VariableValue::Number(n),
                        Value::Text(t) => VariableValue::Text(t),
                    },
                },
                SystemTime::now(),
            )
            .map_err(|error| write_failure(index, error))?;
        response.writes.push(WriteReceipt {
            target: write.target,
            revision: receipt.revision,
            changed: receipt.command.is_some(),
        });
        response.document = Some(document(&app));
    }
    let action_request = ModelingActionRequest {
        document_id: app.workspace.document.metadata.document_id.clone(),
        expected_revision: app.workspace.document.revision,
        action: ModelingAction::Run(match request.package_id {
            Some(id) => WorkspaceRunPackageSelection::Explicit(id),
            None => WorkspaceRunPackageSelection::Preferred,
        }),
    };
    let receipt = dispatch_modeling_action(
        &StudioAppFacade::new(),
        &mut app,
        &context,
        action_request,
        SystemTime::now(),
    )
    .map_err(|e| failure("run", "action_rejected", e.to_string()))?;
    let ModelingActionEffect::Run(outcome) = receipt.effect else {
        return Err(failure("run", "unexpected_action", "expected run outcome"));
    };
    let StudioAppResultDispatch::WorkspaceRun(run) = outcome.dispatch else {
        return Err(failure(
            "run",
            "unexpected_dispatch",
            "expected workspace run dispatch",
        ));
    };
    response.package_id = run.package_id;
    response.diagnostic = app
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .map(|d| Diagnostic {
            code: d.primary_code.clone(),
            message: d.primary_message.clone(),
            unit_ids: d.related_unit_ids.iter().map(ToString::to_string).collect(),
            stream_ids: d
                .related_stream_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            ports: d
                .related_port_targets
                .iter()
                .map(|p| Port {
                    unit_id: p.unit_id.to_string(),
                    port: p.port_name.clone(),
                })
                .collect(),
        });
    match run.outcome {
        StudioWorkspaceRunOutcome::Blocked(blocked) => {
            response.status = Status::Blocked;
            response.error = Some(failure(
                "run",
                blocked_code(blocked.reason),
                blocked.message,
            ));
            return Ok(());
        }
        StudioWorkspaceRunOutcome::Failed(failed) => {
            response.status = Status::Failed;
            response.error = Some(failure(
                "run",
                match failed.reason {
                    crate::StudioWorkspaceRunFailedReason::LocalCacheUnavailable => {
                        "local_cache_unavailable"
                    }
                    crate::StudioWorkspaceRunFailedReason::SolveFailed => "solve_failed",
                },
                failed.message,
            ));
            return Ok(());
        }
        StudioWorkspaceRunOutcome::Skipped(reason) => {
            response.status = Status::Blocked;
            response.error = Some(failure(
                "run",
                "run_skipped",
                match reason {
                    crate::WorkspaceSolveSkipReason::HoldMode => "workspace is in Hold mode",
                    crate::WorkspaceSolveSkipReason::NoPendingRequest => {
                        "workspace has no pending request"
                    }
                },
            ));
            return Ok(());
        }
        StudioWorkspaceRunOutcome::Started(_) => {}
    }
    if !latest_snapshot(&app.workspace).is_some_and(|s| {
        s.status == RunStatus::Converged && s.document_revision == app.workspace.document.revision
    }) {
        response.status = Status::Failed;
        response.error = Some(failure(
            "run",
            "no_current_result",
            "run did not produce a converged snapshot for the current revision",
        ));
        return Ok(());
    }
    // Collect before publishing so an invalid later query never produces a partial value list.
    response.variables = request
        .reads
        .into_iter()
        .enumerate()
        .map(|(index, target)| {
            browser(&app)
                .read(&target.variable_id(&app.workspace.document.metadata.document_id))
                .map(Variable::from)
                .map_err(|e| Failure {
                    index: Some(index),
                    ..failure(
                        "read",
                        browse_code(&e),
                        format!("variable query failed: {e:?}"),
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    Ok(())
}

fn browse_code(error: &BrowseError) -> &'static str {
    match error {
        BrowseError::DifferentDocument => "different_document",
        BrowseError::ObjectMissing(_) => "object_missing",
        BrowseError::VariableMissing(_) => "variable_missing",
    }
}
fn write_failure(index: usize, error: VariableWriteError) -> Failure {
    let code = match &error {
        VariableWriteError::Lookup(e) => browse_code(e),
        VariableWriteError::ReadOnly => "read_only",
        VariableWriteError::RevisionConflict { .. } => "revision_conflict",
        VariableWriteError::PendingDrafts => "pending_drafts",
        VariableWriteError::TypeMismatch { .. } => "type_mismatch",
        VariableWriteError::Rejected(e) => e.code().as_str(),
    };
    Failure {
        index: Some(index),
        ..failure("write", code, error.to_string())
    }
}
fn blocked_code(reason: crate::StudioWorkspaceRunBlockedReason) -> &'static str {
    use crate::StudioWorkspaceRunBlockedReason::*;
    match reason {
        CachedPackageMissing => "cached_package_missing",
        ExplicitPackageSelectionRequired => "explicit_package_selection_required",
        EntitlementMismatch => "entitlement_mismatch",
        InvalidSelection => "invalid_selection",
        ModelingInputsNotReady => "modeling_inputs_not_ready",
        MissingProjectComponents => "missing_project_components",
        PendingInspectorDrafts => "pending_inspector_drafts",
        UnnormalizedStreamComposition => "unnormalized_stream_composition",
    }
}

#[cfg(test)]
mod tests;
