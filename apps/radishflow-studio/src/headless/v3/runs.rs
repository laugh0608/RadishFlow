//! Synchronous execution adapter. Failed calculations still publish their actual
//! terminal evidence; only input edits use the isolated transaction staging path.
use super::{
    actions::*,
    errors,
    protocol::*,
    query,
    references::{Bindings, Executions},
    results,
};
use crate::{
    StudioAppAuthCacheContext, StudioAppFacade, StudioRunOutcome, WorkspaceRunBlock,
    WorkspaceRunCommand, WorkspaceRunPackageSelection, WorkspaceSolveTrigger,
    run_actions::{
        EquipmentCalculationTarget, StudioRunAccess, StudioRunRejection, StudioRunReply,
        StudioRunRequest, StudioRunStartFailure,
    },
};
use rf_ui::{
    AppState,
    engineering::{EquipmentCaseKey, editor::EquipmentActionContext},
};
use serde_json::{Value, json};
use std::path::Path;

pub struct RunStep {
    pub receipt: Value,
    pub error: Option<Failure>,
}
pub fn is_run(action: &Action) -> bool {
    matches!(
        action,
        Action::RunFlow { .. } | Action::Size { .. } | Action::Rate { .. }
    )
}
pub fn validate_resources(request: &ExecuteRequest) -> Result<(), Failure> {
    let needs = request
        .steps
        .iter()
        .any(|s| matches!(s.action, Action::RunFlow { .. }));
    if needs != request.flow_resources.is_some() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "flow_resources is required exactly when steps contain run_flow",
        ));
    }
    if let Some(r) = &request.flow_resources
        && (r.cache_root.as_os_str().is_empty()
            || r.auth_cache_index.as_os_str().is_empty()
            || matches!(&r.package,PackageSelection::Explicit {package_id} if package_id.trim().is_empty()))
    {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "flow resource paths and explicit package id must not be blank",
        ));
    }
    Ok(())
}
pub fn execute(
    app: &mut AppState,
    step: &Step,
    refs: &Bindings,
    executions: &mut Executions,
    resources: Option<&FlowResources>,
    base: &Path,
) -> Result<RunStep, Failure> {
    let facade = StudioAppFacade::new();
    let (reply, scope) = match &step.action {
        Action::RunFlow { expected_revision } => {
            let doc = &app.workspace.document;
            refs.revision(
                expected_revision,
                Output::Document,
                doc.metadata.document_id.as_str(),
                doc.revision,
            )?;
            let resources = resources.expect("validated flow resources");
            let index = rf_store::read_auth_cache_index(base.join(&resources.auth_cache_index));
            let reply = match index {
                Ok(index) => {
                    let root = base.join(&resources.cache_root);
                    let cache = StudioAppAuthCacheContext::new(&root, &index);
                    facade.execute_run_action(
                        app,
                        &cache,
                        StudioRunAccess::Allowed,
                        StudioRunRequest::Flow {
                            expected: crate::flow_actions::FlowEditContext::capture(&app.workspace),
                            command: WorkspaceRunCommand::new(
                                WorkspaceSolveTrigger::Manual,
                                match &resources.package {
                                    PackageSelection::Preferred {} => {
                                        WorkspaceRunPackageSelection::Preferred
                                    }
                                    PackageSelection::Explicit { package_id } => {
                                        WorkspaceRunPackageSelection::Explicit(package_id.clone())
                                    }
                                },
                            ),
                        },
                    )
                }
                Err(error) => StudioRunReply::StartFailed {
                    id: None,
                    error: StudioRunStartFailure::Infrastructure(error),
                },
            };
            (reply, Scope::Flow)
        }
        Action::Size {
            task,
            expected_revision,
        } => {
            let id = refs.resolve(task, Output::Task)?.into();
            let record = app
                .workspace
                .engineering
                .inputs()
                .task(&id)
                .ok_or_else(super::equipment::missing)?;
            refs.revision(
                expected_revision,
                Output::Task,
                record.id.as_str(),
                record.revision,
            )?;
            let member = ObjectRef::Case {
                task: IdentityRef::id(&id),
                case: IdentityRef::id(&record.case.id),
            };
            let expected = EquipmentActionContext::capture(&app.workspace.engineering);
            (
                facade.execute_equipment_run_action(
                    app,
                    StudioRunAccess::Allowed,
                    expected,
                    EquipmentCalculationTarget::Sizing(id),
                ),
                Scope::Object(member),
            )
        }
        Action::Rate {
            task,
            case,
            expected_case_revision,
            expected_geometry_revision,
        } => {
            let id = refs.resolve(task, Output::Task)?.into();
            let case_id = refs.resolve(case, Output::Case)?.into();
            let record = app
                .workspace
                .engineering
                .inputs()
                .rating_task(&id)
                .ok_or_else(super::equipment::missing)?;
            let row = record
                .cases
                .iter()
                .find(|c| c.id == case_id)
                .ok_or_else(super::equipment::missing)?;
            refs.revision(
                expected_case_revision,
                Output::Case,
                row.id.as_str(),
                row.revision,
            )?;
            refs.revision(
                expected_geometry_revision,
                Output::Geometry,
                record.id.as_str(),
                record.geometry.revision,
            )?;
            let member = EquipmentCaseKey {
                task_id: id,
                case_id,
            };
            let scope = Scope::Object(ObjectRef::Case {
                task: IdentityRef::id(&member.task_id),
                case: IdentityRef::id(&member.case_id),
            });
            let expected = EquipmentActionContext::capture(&app.workspace.engineering);
            (
                facade.execute_equipment_run_action(
                    app,
                    StudioRunAccess::Allowed,
                    expected,
                    EquipmentCalculationTarget::Rating(member),
                ),
                scope,
            )
        }
        _ => unreachable!("run action dispatch"),
    };
    let (mut receipt, error) = reply_value(reply, &step.id, executions);
    receipt["result"] = query::editable(app, &QueryKind::Results { scope }, refs, executions)
        .unwrap_or_else(|e| json!({"query_error":e}));
    if matches!(step.action, Action::RunFlow { .. }) {
        receipt["diagnostic"] = app
            .workspace
            .solve_session
            .latest_diagnostic
            .as_ref()
            .map(results::flow_diagnostic)
            .unwrap_or(Value::Null);
    }
    Ok(RunStep { receipt, error })
}
fn failed(code: Code, reason: Value, message: impl Into<String>) -> Failure {
    let mut e = Failure::new(Stage::Step, code, message);
    e.status = Status::Failed;
    e.details = Some(Box::new(reason));
    e
}
fn reply_value(
    reply: StudioRunReply,
    step: &str,
    executions: &mut Executions,
) -> (Value, Option<Failure>) {
    match reply {
        StudioRunReply::Completed(c) => {
            executions.0.push((c.id, step.to_string()));
            let outcome = match c.outcome {
                StudioRunOutcome::Succeeded => "succeeded",
                StudioRunOutcome::Failed => "failed",
                StudioRunOutcome::Cancelled => "cancelled",
                StudioRunOutcome::InfrastructureFailed => "infrastructure_failed",
            };
            let receipt = json!({"outcome":"completed","execution":{"step":step,"scope":"request"},"terminal":outcome,"reception":if c.applied {"applied"}else{"discarded"},"cancellation":c.cancellation.map(cancellation),"diagnostic_code":c.diagnostic_code});
            let error = (!(c.outcome == StudioRunOutcome::Succeeded && c.applied)).then(|| {
                failed(
                    Code::RunFailed,
                    json!({"terminal":outcome,"applied":c.applied}),
                    "calculation did not produce an applied successful result",
                )
            });
            (receipt, error)
        }
        StudioRunReply::StartFailed { id, error } => {
            let execution = id.map(|id| {
                executions.0.push((id, step.to_string()));
                json!({"step":step,"scope":"request"})
            });
            let reason = match &error {
                StudioRunStartFailure::Infrastructure(e) => {
                    json!({"reason":"infrastructure","code":e.code().as_str()})
                }
                StudioRunStartFailure::Flow(e) => {
                    json!({"reason":match e.reason {crate::StudioWorkspaceRunFailedReason::LocalCacheUnavailable=>"local_cache_unavailable",crate::StudioWorkspaceRunFailedReason::SolveFailed=>"solve_failed"}})
                }
                StudioRunStartFailure::MissingExecutionEvidence => {
                    json!({"reason":"missing_execution_evidence"})
                }
            };
            (
                json!({"outcome":"start_failed","execution":execution,"reason":reason}),
                Some(failed(Code::RunStartFailed, reason, error.to_string())),
            )
        }
        StudioRunReply::Blocked(reason) => {
            let error = rejected(reason);
            (
                json!({"outcome":"blocked","reason":error.details}),
                Some(error),
            )
        }
        StudioRunReply::Skipped(reason) => {
            let reason = match reason {
                crate::WorkspaceSolveSkipReason::HoldMode => "hold_mode",
                crate::WorkspaceSolveSkipReason::NoPendingRequest => "no_pending_request",
            };
            (
                json!({"outcome":"skipped","reason":reason}),
                Some(errors::blocked(json!({"reason":reason}), "run was skipped")),
            )
        }
        StudioRunReply::Closing => (
            json!({"outcome":"blocked","reason":"closing"}),
            Some(errors::blocked(
                json!({"reason":"closing"}),
                "workspace is closing",
            )),
        ),
        StudioRunReply::Accepted(_)
        | StudioRunReply::AlreadyRunning(_)
        | StudioRunReply::Busy(_) => {
            // This adapter constructs an inline facade, so asynchronous receipts
            // cannot be promoted to a completed calculation.
            (
                json!({"outcome":"incomplete"}),
                Some(failed(
                    Code::RunFailed,
                    json!({"reason":"unexpected_async_reply"}),
                    "inline execution did not return a terminal receipt",
                )),
            )
        }
    }
}
fn rejected(reason: StudioRunRejection) -> Failure {
    match reason {
        StudioRunRejection::ReadOnly => {
            errors::blocked(json!({"reason":"read_only"}), "read-only project")
        }
        StudioRunRejection::EquipmentContext(e) => errors::equipment(e),
        StudioRunRejection::Equipment(e) => {
            errors::equipment(rf_ui::engineering::editor::EquipmentActionError::Edit(e))
        }
        StudioRunRejection::FlowContext(e) => {
            errors::flow(crate::flow_actions::FlowActionError::Context(e))
        }
        StudioRunRejection::Flow(WorkspaceRunBlock::Preflight(e)) => errors::blocked(
            json!({"reason":super::super::blocked_code(e.reason)}),
            e.message,
        ),
        StudioRunRejection::Flow(WorkspaceRunBlock::Authorization(e)) => errors::blocked(
            json!({"reason":"authorization","details":authorization(e)}),
            "flow authorization refused",
        ),
        StudioRunRejection::Flow(WorkspaceRunBlock::Closing) => {
            errors::blocked(json!({"reason":"closing"}), "workspace is closing")
        }
    }
}
fn authorization(reason: crate::RunAuthorizationRefusal) -> &'static str {
    match reason {
        crate::RunAuthorizationRefusal::Expired => "expired",
        crate::RunAuthorizationRefusal::Changed => "changed",
    }
}
fn cancellation(reason: crate::WorkspaceRunCancelReason) -> Value {
    use crate::WorkspaceRunCancelReason::*;
    match reason {
        UserRequested => json!({"reason":"user_requested"}),
        QualificationChanged => json!({"reason":"qualification_changed"}),
        Authorization(e) => json!({"reason":"authorization","details":authorization(e)}),
        Detached => json!({"reason":"detached"}),
        Closing => json!({"reason":"closing"}),
    }
}
