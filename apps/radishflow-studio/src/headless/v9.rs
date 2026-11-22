//! Read-only-on-disk automation for a saved v4 mixed-recycle plan.
pub(super) mod results;

use super::{engine, planned::invalid, v3 as old};
use crate::StudioProjectOpen;
use old::{
    actions::{ExpectedProject, FlowResources, RevisionRef},
    protocol::{Code, Failure, ObjectRef, Output, Response, Stage, Status},
    references::{Bindings, Executions},
};
use rf_store::project_v4::{self, FlowPlanSupport, ObjectIdentitySupport};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};

pub(super) const VERSION: u32 = 9;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryRequest {
    schema_version: u32,
    project: PathBuf,
    queries: Vec<Query>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteRequest {
    schema_version: u32,
    project: PathBuf,
    expected_project: ExpectedProject,
    flow_resources: FlowResources,
    steps: Vec<RunStep>,
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
    Results { scope: ResultScope },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResultScope {
    FlowPlan,
    Flow,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunStep {
    id: String,
    action: RunAction,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RunAction {
    RunFlow {
        expected_revision: u64,
        expected_plan_revision: u64,
    },
}

impl engine::StepIdentity for RunStep {
    fn id(&self) -> &str {
        &self.id
    }

    fn target(&self) -> Option<ObjectRef> {
        None
    }
}

pub(super) fn query_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let request: QueryRequest = serde_json::from_slice(bytes).map_err(old::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    old::validate_ids(request.queries.iter().map(|query| query.id.as_str()))?;
    let opened = open(path, &request.project, response)?;
    match &opened {
        StudioProjectOpen::Editable(app) => {
            require_m2(app)?;
            queries(app, &request.queries, &Executions::default(), response);
        }
        StudioProjectOpen::ReadOnly(project) => {
            let metadata = json!({
                "read_only": true,
                "document_id": project.metadata().document_id,
                "project_schema_version": project.schema_version(),
                "reasons": project.reason_summary(),
            });
            for query in &request.queries {
                old::query::push_reply(&query.id, Ok(metadata.clone()), response);
            }
        }
    }
    Ok(())
}

pub(super) fn execute_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let request: ExecuteRequest = serde_json::from_slice(bytes).map_err(old::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    if request.steps.is_empty() {
        return Err(invalid("at least one run_flow step is required"));
    }
    old::validate_ids(request.steps.iter().map(|step| step.id.as_str()))?;
    old::validate_ids(request.queries.iter().map(|query| query.id.as_str()))?;
    old::runs::validate_flow_resources(&request.flow_resources)?;

    let opened = open(path, &request.project, response)?;
    let snapshot = response.project.as_ref().expect("loaded project snapshot");
    if request.expected_project.document_id != snapshot.document_id {
        return Err(Failure::new(
            Stage::Project,
            Code::DifferentDocument,
            "document id differs",
        ));
    }
    if request.expected_project.file_sha256 != snapshot.file_sha256 {
        return Err(Failure::new(
            Stage::Project,
            Code::ProjectConflict,
            "project file snapshot differs",
        ));
    }
    let StudioProjectOpen::Editable(mut app) = opened else {
        let mut error = Failure::new(
            Stage::Project,
            Code::ReadOnlyProject,
            "unsupported capability protects the whole project",
        );
        error.status = Status::Blocked;
        return Err(error);
    };
    require_m2(&app)?;

    let mut executions = Executions::default();
    engine::steps(
        &mut app,
        &request.steps,
        response,
        &mut Bindings::default(),
        |app, refs, _, step| {
            validate_run(app, refs, &step.action)?;
            let mut run = old::runs::execute_flow(
                app,
                &step.id,
                &mut executions,
                &request.flow_resources,
                request_directory(path),
            );
            run.receipt["plan_revision"] = json!(app.workspace.flow_plan.revision());
            run.receipt["result"] = match results::flow(app, &executions) {
                Ok(value) => value,
                Err(error) if run.error.is_some() => json!({ "query_error": error }),
                Err(error) => return Err(error),
            };
            Ok((run.receipt, run.error))
        },
    );
    queries(&app, &request.queries, &executions, response);
    Ok(())
}

fn request_directory(path: &Path) -> &Path {
    path.parent().unwrap_or(Path::new("."))
}

fn open(
    path: &Path,
    project: &Path,
    response: &mut Response,
) -> Result<StudioProjectOpen, Failure> {
    if project.as_os_str().is_empty() {
        return Err(invalid("project path must not be empty"));
    }
    let (opened, snapshot) = old::load_project_with_support(
        &request_directory(path).join(project),
        project_v4::VERSION,
        FlowPlanSupport::MixedRecycle,
        ObjectIdentitySupport::Supported,
    )?;
    response.project = Some(snapshot);
    Ok(opened)
}

fn require_m2(app: &AppState) -> Result<(), Failure> {
    if matches!(app.workspace.flow_plan.current(), FlowPlanConfiguration::Mixed(p) if p.single_recycle().is_some())
    {
        Ok(())
    } else {
        Err(Failure::new(
            Stage::Project,
            Code::UnsupportedCapability,
            "v9 requires a saved mixed-recycle plan",
        ))
    }
}

fn validate_run(app: &AppState, refs: &Bindings, action: &RunAction) -> Result<(), Failure> {
    let RunAction::RunFlow {
        expected_revision,
        expected_plan_revision,
    } = *action;
    let workspace = &app.workspace;
    refs.revision(
        &RevisionRef::Number(expected_revision),
        Output::Document,
        workspace.document.metadata.document_id.as_str(),
        workspace.document.revision,
    )?;
    let actual = workspace.flow_plan.revision();
    if expected_plan_revision != actual {
        let mut error = Failure::new(Stage::Step, Code::RecordConflict, "plan revision differs");
        error.details = Some(Box::new(json!({
            "owner": "plan", "expected": expected_plan_revision, "actual": actual,
        })));
        return Err(error);
    }
    workspace
        .flow_plan
        .current()
        .validate_for(&workspace.document.flowsheet)
        .map_err(|error| old::errors::blocked(results::plan_diagnostic(&error), error.to_string()))
}

fn queries(app: &AppState, queries: &[Query], executions: &Executions, response: &mut Response) {
    for query in queries {
        let QueryKind::Results { scope } = &query.query;
        let result = match scope {
            ResultScope::FlowPlan => results::plan(app),
            ResultScope::Flow => results::flow(app, executions),
        };
        old::query::push_reply(&query.id, result, response);
    }
}
