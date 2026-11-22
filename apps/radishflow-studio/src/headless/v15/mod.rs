//! Read and explicitly run one saved v4 serial recycle plan per CLI request.

mod results;

use super::{
    engine,
    planned::{invalid, validate_saved_flow_run},
    project_file::{check_expected, read_only},
    request::JsonObject,
    v3::{
        self,
        actions::{ExpectedProject, FlowResources},
        protocol::{Code, Failure, ObjectRef, Response, Stage, Status},
        references::{Bindings, Executions},
    },
};
use crate::StudioProjectOpen;
use rf_store::project_v4::{self, FlowPlanSupport, ObjectIdentitySupport};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};

pub(super) const VERSION: u32 = 15;
const PLAN_SUPPORT: FlowPlanSupport = FlowPlanSupport::MixedSerialRecycle;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryRequest {
    schema_version: u32,
    project: PathBuf,
    queries: Vec<JsonObject<Query>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteRequest {
    schema_version: u32,
    project: PathBuf,
    expected_project: JsonObject<ExpectedProject>,
    flow_resources: JsonObject<FlowResources>,
    steps: [JsonObject<RunStep>; 1],
    queries: Vec<JsonObject<Query>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    query: JsonObject<QueryKind>,
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
    action: JsonObject<RunAction>,
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
    let JsonObject(request): JsonObject<QueryRequest> =
        serde_json::from_slice(bytes).map_err(v3::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    v3::validate_ids(request.queries.iter().map(|query| query.0.id.as_str()))?;
    match open(path, &request.project, response)? {
        StudioProjectOpen::Editable(app) => {
            require_m3(&app)?;
            queries(&app, &request.queries, &Executions::default(), response);
        }
        StudioProjectOpen::ReadOnly(project) => {
            let metadata = json!({
                "read_only": true,
                "document_id": project.metadata().document_id,
                "project_schema_version": project.schema_version(),
                "reasons": project.reason_summary(),
            });
            for JsonObject(query) in &request.queries {
                v3::query::push_reply(&query.id, Ok(metadata.clone()), response);
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
    let JsonObject(request): JsonObject<ExecuteRequest> =
        serde_json::from_slice(bytes).map_err(v3::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    let [JsonObject(step)] = request.steps;
    v3::validate_ids(std::iter::once(step.id.as_str()))?;
    v3::validate_ids(request.queries.iter().map(|query| query.0.id.as_str()))?;
    let resources = &request.flow_resources.0;
    v3::runs::validate_flow_resources(resources)?;
    if [&resources.cache_root, &resources.auth_cache_index]
        .iter()
        .any(|path| blank_path(path))
    {
        return Err(invalid("flow resource paths must not be blank"));
    }

    let opened = open(path, &request.project, response)?;
    check_expected(
        response.project.as_ref().expect("loaded project snapshot"),
        &request.expected_project.0,
    )?;
    let StudioProjectOpen::Editable(mut app) = opened else {
        return Err(read_only(Stage::Project));
    };
    require_m3(&app)?;

    let mut executions = Executions::default();
    engine::steps(
        &mut app,
        &[step],
        response,
        &mut Bindings::default(),
        |app, refs, _, step| {
            let RunAction::RunFlow {
                expected_revision,
                expected_plan_revision,
            } = step.action.0;
            validate_saved_flow_run(app, refs, expected_revision, expected_plan_revision)?;
            let mut run = v3::runs::execute_flow(
                app,
                &step.id,
                &mut executions,
                resources,
                request_directory(path),
            );
            let result = results::flow(app, &executions);
            if run.error.is_none() && !matches!(&result, Ok(value) if value["status"] == "current")
            {
                let mut error = Failure::new(
                    Stage::Step,
                    Code::RunFailed,
                    "run did not produce a qualified serial recycle result",
                );
                error.status = Status::Failed;
                run.error = Some(error);
            }
            run.receipt["plan_revision"] = json!(app.workspace.flow_plan.revision());
            run.receipt["result"] = result.unwrap_or_else(|error| json!({ "query_error": error }));
            Ok((run.receipt, run.error))
        },
    );
    queries(&app, &request.queries, &executions, response);
    Ok(())
}

fn blank_path(path: &Path) -> bool {
    path.as_os_str().to_string_lossy().trim().is_empty()
}

fn request_directory(path: &Path) -> &Path {
    path.parent().unwrap_or(Path::new("."))
}

fn open(
    path: &Path,
    project: &Path,
    response: &mut Response,
) -> Result<StudioProjectOpen, Failure> {
    if blank_path(project) {
        return Err(invalid("project path must not be blank"));
    }
    let (opened, snapshot) = v3::load_project_with_support(
        &request_directory(path).join(project),
        project_v4::VERSION,
        PLAN_SUPPORT,
        ObjectIdentitySupport::Supported,
    )?;
    response.project = Some(snapshot);
    Ok(opened)
}

fn require_m3(app: &AppState) -> Result<(), Failure> {
    if matches!(app.workspace.flow_plan.current(), FlowPlanConfiguration::Mixed(plan) if plan.serial_recycle().is_some())
    {
        return Ok(());
    }
    Err(Failure::new(
        Stage::Project,
        Code::UnsupportedCapability,
        "v15 requires a saved v4 mixed serial recycle plan",
    ))
}

fn queries(
    app: &AppState,
    queries: &[JsonObject<Query>],
    executions: &Executions,
    response: &mut Response,
) {
    for JsonObject(query) in queries {
        let QueryKind::Results { scope } = &query.query.0;
        let result = match scope {
            ResultScope::FlowPlan => results::plan(app),
            ResultScope::Flow => results::flow(app, executions),
        };
        v3::query::push_reply(&query.id, result, response);
    }
}
