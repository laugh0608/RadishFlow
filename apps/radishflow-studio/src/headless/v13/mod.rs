//! CLI v13 reads saved H3 inputs and runs at most one explicitly selected case.

pub(super) mod project;
pub(super) mod records;

use super::{
    project_file::{check_expected, read_only},
    request::JsonObject,
    v3::{self, actions::ExpectedProject, protocol::*},
};
use project::load_project;
use rf_model::equipment::ConcentricTubeRatingTask;
use rf_store::project_v3::StoredEngineeringInputs;
use rf_types::{EngineeringCaseId, EquipmentTaskId};
use rf_ui::engineering::concentric_tube::{RatingRevisions, SavedRatingError, rate_saved_case};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(super) const VERSION: u32 = 13;

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SavedId<T> {
    id: T,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseMember {
    task: JsonObject<SavedId<EquipmentTaskId>>,
    case: JsonObject<SavedId<EngineeringCaseId>>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum CaseTarget {
    Case(JsonObject<CaseMember>),
}

impl CaseTarget {
    pub(super) fn reference(&self) -> ObjectRef {
        let Self::Case(JsonObject(member)) = self;
        ObjectRef::Case {
            task: IdentityRef::id(&member.task.0.id),
            case: IdentityRef::id(&member.case.0.id),
        }
    }

    pub(super) fn find<'a>(
        &self,
        inputs: &'a StoredEngineeringInputs,
        stage: Stage,
    ) -> Result<&'a ConcentricTubeRatingTask, Failure> {
        let Self::Case(JsonObject(member)) = self;
        inputs
            .tasks
            .iter()
            .filter_map(|task| task.as_concentric_tube_rating())
            .find(|task| task.id == member.task.0.id && task.case.id == member.case.0.id)
            .ok_or_else(|| {
                Failure::new(
                    stage,
                    Code::ObjectMissing,
                    "saved concentric-tube case does not exist",
                )
                .target(&self.reference())
            })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum ResultScope {
    Object(CaseTarget),
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum QueryKind {
    Objects {},
    Record { target: CaseTarget },
    Results { scope: ResultScope },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Query {
    pub(super) id: String,
    query: QueryKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct QueryRequest {
    pub(super) schema_version: u32,
    pub(super) project: PathBuf,
    pub(super) queries: Vec<JsonObject<Query>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExpectedRevisions {
    task: u64,
    geometry: u64,
    case: u64,
}

impl From<&ExpectedRevisions> for RatingRevisions {
    fn from(revisions: &ExpectedRevisions) -> Self {
        Self {
            task: revisions.task,
            geometry: revisions.geometry,
            case: revisions.case,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    RateConcentricTube {
        target: CaseTarget,
        expected_revisions: JsonObject<ExpectedRevisions>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    action: Action,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteRequest {
    schema_version: u32,
    project: PathBuf,
    expected_project: JsonObject<ExpectedProject>,
    steps: [JsonObject<Step>; 1],
    queries: Vec<JsonObject<Query>>,
}

pub(super) fn query_file(
    path: &Path,
    bytes: &[u8],
    response: &mut Response,
) -> Result<(), Failure> {
    let JsonObject(request): JsonObject<QueryRequest> =
        serde_json::from_slice(bytes).map_err(v3::invalid_request)?;
    debug_assert_eq!(request.schema_version, VERSION);
    v3::validate_ids(request.queries.iter().map(|q| q.0.id.as_str()))?;
    let (project, inputs) = load_project(path, &request.project)?;
    response.project = Some(project);
    queries(inputs.as_ref(), &request.queries, None, response);
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
    let [JsonObject(step)] = &request.steps;
    v3::validate_ids(std::iter::once(step.id.as_str()))?;
    v3::validate_ids(request.queries.iter().map(|q| q.0.id.as_str()))?;
    let (project, inputs) = load_project(path, &request.project)?;
    let expected = check_expected(&project, &request.expected_project.0);
    response.project = Some(project);
    expected?;
    let inputs = inputs.ok_or_else(|| read_only(Stage::Project))?;

    let run = execute_step(step, &inputs, response);
    queries(Some(&inputs), &request.queries, run.as_ref(), response);
    Ok(())
}

struct Run<'a> {
    task: &'a ConcentricTubeRatingTask,
    result: Value,
}

fn execute_step<'a>(
    step: &Step,
    inputs: &'a StoredEngineeringInputs,
    response: &mut Response,
) -> Option<Run<'a>> {
    let Action::RateConcentricTube {
        target,
        expected_revisions: JsonObject(expected_revisions),
    } = &step.action;
    let expected = RatingRevisions::from(expected_revisions);
    let mut reject = |mut error: Failure| {
        error.step_id = Some(step.id.clone());
        response
            .steps
            .push(json!({"id":step.id,"outcome":"rejected","error":error}));
        response.fail(error);
        None
    };
    let task = match target.find(inputs, Stage::Step) {
        Ok(task) => task,
        Err(error) => return reject(error),
    };
    let result = rate_saved_case(task, &inputs.basis, expected);
    let (outcome, result, error) = match result {
        Ok(value) => ("completed", records::current_result(value), None),
        Err(SavedRatingError::RevisionConflict { actual }) => {
            let mut error = Failure::new(
                Stage::Step,
                Code::RecordConflict,
                "saved H3 input revisions differ",
            )
            .target(&target.reference());
            error.details = Some(Box::new(json!({
                "expected_revisions": records::revisions(expected),
                "actual_revisions": records::revisions(actual),
            })));
            return reject(error);
        }
        Err(SavedRatingError::Calculation(core)) => {
            let diagnostic = json!({
                "code": core.context().diagnostic_code().unwrap_or(core.code().as_str()),
                "message": core.message(),
            });
            let mut error = Failure::new(Stage::Step, Code::RunFailed, core.message())
                .target(&target.reference());
            error.status = Status::Failed;
            error.step_id = Some(step.id.clone());
            error.details = Some(Box::new(json!({"diagnostic":diagnostic})));
            (
                "failed",
                json!({"status":"failed","diagnostic":diagnostic}),
                Some(error),
            )
        }
    };
    let snapshot = response
        .project
        .as_ref()
        .expect("project loaded before execution");
    let mut result = result;
    result["origin"] = records::origin(snapshot, task, &step.id);
    let mut receipt = json!({"id":step.id,"outcome":outcome,"result":result});
    if let Some(error) = error {
        receipt["error"] = json!(error);
        response.fail(error);
    }
    response.steps.push(receipt);
    Some(Run { task, result })
}

fn queries(
    inputs: Option<&StoredEngineeringInputs>,
    queries: &[JsonObject<Query>],
    run: Option<&Run<'_>>,
    response: &mut Response,
) {
    reply_queries(queries, response, |query| match inputs {
        Some(inputs) => query_value(query, inputs, run),
        None => Err(read_only(Stage::Query)),
    });
}

pub(super) fn reply_queries(
    queries: &[JsonObject<Query>],
    response: &mut Response,
    resolve: impl Fn(&QueryKind) -> Result<Value, Failure>,
) {
    for JsonObject(query) in queries {
        let reply = match resolve(&query.query) {
            Ok(value) => QueryReply {
                id: query.id.clone(),
                value: Some(value),
                error: None,
            },
            Err(mut error) => {
                error.query_id = Some(query.id.clone());
                response.fail(error.clone());
                QueryReply {
                    id: query.id.clone(),
                    value: None,
                    error: Some(error),
                }
            }
        };
        response.queries.push(reply);
    }
}

fn query_value(
    query: &QueryKind,
    inputs: &StoredEngineeringInputs,
    run: Option<&Run<'_>>,
) -> Result<Value, Failure> {
    Ok(match query {
        QueryKind::Objects {} => Value::Array(
            inputs
                .tasks
                .iter()
                .filter_map(|task| task.as_concentric_tube_rating())
                .map(records::object)
                .collect(),
        ),
        QueryKind::Record { target } => {
            records::record(target.find(inputs, Stage::Query)?, &inputs.basis)
        }
        QueryKind::Results {
            scope: ResultScope::Object(target),
        } => {
            let task = target.find(inputs, Stage::Query)?;
            run.filter(|run| run.task.id == task.id && run.task.case.id == task.case.id)
                .map_or_else(|| json!({"status":"missing"}), |run| run.result.clone())
        }
    })
}
