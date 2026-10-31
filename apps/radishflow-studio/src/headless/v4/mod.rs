//! CLI v4 increments. Ordering, historical actions, domain impact review and
//! resource authorization remain shared with existing automation consumers.
mod plan;
mod protocol;
mod results;
use super::v3 as old;
use crate::{
    StudioProjectOpen,
    flow_actions::{FlowAction, FlowEditAccess},
    modeling_actions::{MaterialConnectionTarget, MaterialPortTarget},
};
use old::{
    confirmation::Aliases,
    protocol::{Code, Failure, IdentityRef, Output, Response, Scope, Stage, Status},
    references::{Bindings, Executions},
};
use protocol::*;
use rf_ui::{
    AppState,
    flow_plan::{FlowPlanCommand, FlowPlanEditContext},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
fn invalid(message: impl Into<String>) -> Failure {
    Failure::new(Stage::Request, Code::InvalidRequest, message)
}
fn domain(error: rf_types::RfError) -> Failure {
    old::errors::blocked(
        json!({"code":error.context().diagnostic_code()}),
        error.to_string(),
    )
}
fn project_path(base: &Path, path: &Path) -> Result<std::path::PathBuf, Failure> {
    if path.as_os_str().is_empty() {
        Err(invalid("project path must not be empty"))
    } else {
        Ok(base.parent().unwrap_or(Path::new(".")).join(path))
    }
}
pub(super) fn query_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let request: QueryRequest = serde_json::from_slice(bytes).map_err(old::invalid_request)?;
    debug_assert_eq!(request.schema_version, 4);
    old::validate_ids(request.queries.iter().map(|q| q.id.as_str()))?;
    let (opened, snapshot) = old::load_project_version(&project_path(path, &request.project)?, 4)?;
    response.project = Some(snapshot);
    let refs = Bindings::default();
    let executions = Executions::default();
    queries(&opened, &request.queries, &refs, &executions, response);
    if let StudioProjectOpen::Editable(app) = &opened {
        required_version(app, response);
    }
    Ok(())
}
pub(super) fn execute_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let request: ExecuteRequest = serde_json::from_slice(bytes).map_err(old::invalid_request)?;
    debug_assert_eq!(request.schema_version, 4);
    old::validate_ids(request.steps.iter().map(|s| s.id.as_str()))?;
    old::validate_ids(request.queries.iter().map(|q| q.id.as_str()))?;
    // Validate the exact same resource vocabulary before reading any project or cache.
    let validation = old::actions::ExecuteRequest {
        schema_version: 3,
        project: request.project.clone(),
        expected_project: request.expected_project.clone(),
        flow_resources: request.flow_resources.clone(),
        queries: Vec::new(),
        steps: request
            .steps
            .iter()
            .filter_map(|s| match &s.action {
                Action::New(NewAction::RunFlow {
                    expected_revision, ..
                }) => Some(old::actions::Step {
                    id: s.id.clone(),
                    action: old::actions::Action::RunFlow {
                        expected_revision: expected_revision.clone(),
                    },
                    confirmation: s.confirmation.clone(),
                }),
                _ => None,
            })
            .collect(),
    };
    old::runs::validate_resources(&validation)?;
    let (opened, snapshot) = old::load_project_version(&project_path(path, &request.project)?, 4)?;
    let file_hash = snapshot.file_sha256.clone();
    let matches_id = request.expected_project.document_id == snapshot.document_id;
    let matches_hash = request.expected_project.file_sha256 == snapshot.file_sha256;
    response.project = Some(snapshot);
    if !matches_id {
        return Err(Failure::new(
            Stage::Project,
            Code::DifferentDocument,
            "document id differs",
        ));
    }
    if !matches_hash {
        return Err(Failure::new(
            Stage::Project,
            Code::ProjectConflict,
            "project file snapshot differs",
        ));
    }
    let StudioProjectOpen::Editable(mut app) = opened else {
        let mut e = Failure::new(
            Stage::Project,
            Code::ReadOnlyProject,
            "unsupported capability protects the whole project",
        );
        e.status = Status::Blocked;
        return Err(e);
    };
    let mut refs = Bindings::default();
    let mut executions = Executions::default();
    let mut aliases = Aliases::default();
    let mut plans = BTreeMap::new();
    super::engine::steps(
        &mut app,
        &request.steps,
        response,
        &mut refs,
        |app, refs, index, step| match &step.action {
            Action::New(action) => new_action(
                app,
                step,
                action,
                refs,
                &mut executions,
                &mut plans,
                request.flow_resources.as_ref(),
                path,
            ),
            Action::Legacy(action) => {
                let legacy = old::actions::Step {
                    id: step.id.clone(),
                    action: action.clone(),
                    confirmation: step.confirmation.clone(),
                };
                if old::runs::is_run(action) {
                    no_confirmation(step)?;
                    old::runs::execute(
                        app,
                        &legacy,
                        refs,
                        &mut executions,
                        request.flow_resources.as_ref(),
                        path.parent().unwrap_or(Path::new(".")),
                    )
                    .map(|r| (r.receipt, r.error))
                } else {
                    let prefix: Vec<_> = request.steps[..=index]
                        .iter()
                        .map(|s| json!({"id":s.id,"action":s.action}))
                        .collect();
                    let review = json!({"steps":prefix,"flow_plan":plan::wire(app.workspace.flow_plan.current()),"in_memory_required_project_schema_version":crate::project_save_version(app)});
                    old::execute::step_action_review(
                        app,
                        &legacy,
                        refs,
                        &mut aliases,
                        &file_hash,
                        &[],
                        request.flow_resources.as_ref(),
                        Some(&review),
                    )
                    .map(|r| (r, None))
                }
            }
        },
    );
    required_version(&app, response);
    queries(
        &StudioProjectOpen::Editable(app),
        &request.queries,
        &refs,
        &executions,
        response,
    );
    Ok(())
}
fn required_version(app: &AppState, response: &mut Response) {
    if let Some(project) = &mut response.project {
        project.in_memory_required_project_schema_version = Some(crate::project_save_version(app));
    }
}
fn no_confirmation(step: &Step) -> Result<(), Failure> {
    if step.confirmation.is_some() {
        Err(invalid("confirmation is not applicable to this action"))
    } else {
        Ok(())
    }
}
fn plan_revision(
    reference: &PlanRevision,
    actual: u64,
    plans: &BTreeMap<String, u64>,
) -> Result<(), Failure> {
    let expected = match reference {
        PlanRevision::Number(n) => *n,
        PlanRevision::Step { step, .. } => *plans.get(step).ok_or_else(|| {
            Failure::new(
                Stage::Step,
                Code::UnknownReference,
                "step has no successful plan output",
            )
        })?,
    };
    if expected != actual {
        let mut e = Failure::new(Stage::Step, Code::RecordConflict, "plan revision differs");
        e.details = Some(Box::new(
            json!({"owner":"plan","expected":expected,"actual":actual}),
        ));
        Err(e)
    } else {
        Ok(())
    }
}
#[allow(clippy::too_many_arguments)]
fn new_action(
    app: &mut AppState,
    step: &Step,
    action: &NewAction,
    refs: &mut Bindings,
    executions: &mut Executions,
    plans: &mut BTreeMap<String, u64>,
    resources: Option<&old::actions::FlowResources>,
    path: &Path,
) -> Result<(Value, Option<Failure>), Failure> {
    no_confirmation(step)?;
    let (expected, action) = match action {
        NewAction::ConfigureFlowPlan {
            expected_plan_revision,
            plan: config,
        } => {
            plan_revision(
                expected_plan_revision,
                app.workspace.flow_plan.revision(),
                plans,
            )?;
            let configuration = plan::configuration(config, refs)?;
            let context = FlowPlanEditContext::capture(&app.workspace);
            let receipt = crate::flow_plan_actions::execute_flow_plan_action(
                app,
                FlowEditAccess::Editable,
                &context,
                FlowPlanCommand::Apply(configuration),
            )
            .map_err(domain)?;
            plans.insert(step.id.clone(), receipt.revision);
            return Ok((
                json!({"outcome":if receipt.changed {"committed"} else {"unchanged"},"before_plan_revision":receipt.before_revision,"plan_revision":receipt.revision,"outputs":{"plan":{"revision":receipt.revision}},"plan":plan::query(app)}),
                None,
            ));
        }
        NewAction::RunFlow {
            expected_revision,
            expected_plan_revision,
        } => {
            plan_revision(
                expected_plan_revision,
                app.workspace.flow_plan.revision(),
                plans,
            )?;
            let legacy = old::actions::Step {
                id: step.id.clone(),
                action: old::actions::Action::RunFlow {
                    expected_revision: expected_revision.clone(),
                },
                confirmation: None,
            };
            let mut r = old::runs::execute(
                app,
                &legacy,
                refs,
                executions,
                resources,
                path.parent().unwrap_or(Path::new(".")),
            )?;
            r.receipt["plan_revision"] = json!(app.workspace.flow_plan.revision());
            r.receipt["result"] = results::flow(app, executions);
            return Ok((r.receipt, r.error));
        }
        NewAction::CreateUnit {
            expected_revision,
            unit_kind,
        } => (
            expected_revision,
            FlowAction::CreateUnit((*unit_kind).into()),
        ),
        NewAction::Write {
            expected_revision,
            target,
            value,
        } => {
            let object = match refs.object(&target.object)? {
                rf_ui::project_query::ProjectObjectId::Flow(
                    rf_ui::variable_browser::ObjectId::Unit(id),
                ) => rf_ui::variable_browser::ObjectId::Unit(id),
                _ => return Err(invalid("split fraction requires a unit input")),
            };
            (
                expected_revision,
                FlowAction::Write {
                    variable: rf_ui::variable_browser::VariableId {
                        document: app.workspace.document.metadata.document_id.clone(),
                        object,
                        section: rf_ui::variable_browser::VariableSection::Inputs,
                        field: rf_ui::variable_browser::VariableField::OutletASplitFraction,
                    },
                    value: rf_ui::variable_browser::VariableValue::Number(*value),
                },
            )
        }
        NewAction::ConnectExact {
            expected_revision,
            source,
            sink,
            stream,
        } => {
            let port = |p: &old::actions::Port| -> Result<MaterialPortTarget, Failure> {
                Ok(MaterialPortTarget {
                    unit_id: refs.resolve(&p.unit, Output::Unit)?.into(),
                    port: p.port.clone(),
                })
            };
            (
                expected_revision,
                FlowAction::ConnectExact(MaterialConnectionTarget {
                    stream_id: refs.resolve(stream, Output::Stream)?.into(),
                    source: port(source)?,
                    sink: Some(port(sink)?),
                }),
            )
        }
    };
    let reply = old::flow::apply(app, refs, expected, action)?;
    refs.0.insert(step.id.clone(), reply.outputs.clone());
    let mut receipt = reply.receipt;
    receipt["outputs"] = json!(reply.outputs);
    Ok((receipt, None))
}
fn queries(
    opened: &StudioProjectOpen,
    queries: &[Query],
    refs: &Bindings,
    executions: &Executions,
    response: &mut Response,
) {
    for query in queries {
        let result = match opened {
            StudioProjectOpen::Editable(app) => match &query.query {
                QueryKind::Plan(PlanQuery::Results { .. }) => Ok(plan::query(app)),
                QueryKind::Plan(PlanQuery::Actions { .. }) => Ok(
                    json!([{"kind":"configure_flow_plan","expected_plan_revision":true,"requires_resources":false},{"kind":"run_flow","expected_revision":true,"expected_plan_revision":true,"requires_resources":true}]),
                ),
                QueryKind::Legacy(old::protocol::QueryKind::Results { scope: Scope::Flow }) => {
                    Ok(results::flow(app, executions))
                }
                QueryKind::Legacy(old::protocol::QueryKind::Actions { scope }) => {
                    old::query::editable(app, &old::protocol::QueryKind::Actions { scope: scope.clone() }, refs, executions).map(|mut value| {
                    if matches!(scope, Scope::Project | Scope::Flow) && let Some(actions) = value.as_array_mut() {
                        actions.push(json!({"kind":"connect_exact","execution":"inline","modifies_inputs":true,"undoable":true,"callable":true}));
                        for action in actions.iter_mut() {
                            if action["kind"] == "create_unit" {
                                action["unit_kinds"] = json!(["feed","mixer","heater","cooler","valve","flash_drum","splitter"]);
                            } else if action["kind"] == "run_flow" {
                                action["expected_plan_revision"] = json!(true);
                            }
                        }
                    }
                    value
                    })
                }
                QueryKind::Legacy(kind) => old::query::editable(app, kind, refs, executions),
            },
            StudioProjectOpen::ReadOnly(project) => Ok(
                json!({"read_only":true,"document_id":project.metadata().document_id.as_str(),"project_schema_version":project.schema_version(),"reasons":project.reason_summary()}),
            ),
        };
        let (value, error) = match result {
            Ok(v) => (Some(v), None),
            Err(mut e) => {
                e.stage = Stage::Query;
                e.query_id = Some(query.id.clone());
                response.fail(e.clone());
                (None, Some(e))
            }
        };
        response.queries.push(old::protocol::QueryReply {
            id: query.id.clone(),
            value,
            error,
        });
    }
}
