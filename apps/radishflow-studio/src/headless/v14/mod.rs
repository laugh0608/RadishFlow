//! One explicitly rated H3 case using two saved associations and one fresh flow result.

mod project;
mod records;

use super::{
    project_file::{check_expected, read_only},
    request::JsonObject,
    v3::{self, actions::ExpectedProject, protocol::*},
    v13::{
        CaseTarget, ExpectedRevisions, Query, QueryKind, QueryRequest, ResultScope,
        records as h3_records, reply_queries,
    },
};
use crate::document_lifecycle_driver::h3_flow_review::{H3FlowReview, bundled_water};
use project::{ReviewProject, load_project};
use rf_model::equipment::ConcentricTubeRatingTask;
use rf_store::project_v3::StoredEngineeringInputs;
use rf_ui::engineering::{EquipmentCaseKey, concentric_tube::RatingRevisions};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(super) const VERSION: u32 = 14;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QualificationRef {
    id: String,
    revision: u64,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    RateConcentricTubeFromFlow {
        target: CaseTarget,
        expected_revisions: JsonObject<ExpectedRevisions>,
        qualification: JsonObject<QualificationRef>,
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
    let (snapshot, project) = load_project(path, &request.project)?;
    response.project = Some(snapshot);
    let empty = StoredEngineeringInputs::default();
    reply_queries(&request.queries, response, |query| {
        let project = project.as_ref().ok_or_else(|| read_only(Stage::Query))?;
        query_value(
            query,
            project.data.engineering.as_ref().unwrap_or(&empty),
            project.data.document.flowsheet.property_package_id(),
            None,
        )
    });
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
    let (snapshot, project) = load_project(path, &request.project)?;
    let expected = check_expected(&snapshot, &request.expected_project.0);
    response.project = Some(snapshot);
    expected?;
    let project = project.ok_or_else(|| read_only(Stage::Project))?;
    // Keep saved engineering records available for same-request queries after the review owns the project.
    let inputs = project.data.engineering.clone().unwrap_or_default();
    let package_id = project
        .data
        .document
        .flowsheet
        .property_package_id()
        .map(str::to_owned);
    let Action::RateConcentricTubeFromFlow { target, .. } = &step.action;
    let rated = rate(
        step,
        project,
        &inputs,
        package_id.as_deref(),
        response
            .project
            .as_ref()
            .expect("project snapshot set before execution"),
    );
    let result = reply_step(step, rated, response);
    reply_queries(&request.queries, response, |query| {
        query_value(
            query,
            &inputs,
            package_id.as_deref(),
            result.as_ref().map(|value| (target, value)),
        )
    });
    Ok(())
}

fn associated_case<'a>(
    target: &CaseTarget,
    inputs: &'a StoredEngineeringInputs,
    stage: Stage,
) -> Result<&'a ConcentricTubeRatingTask, Failure> {
    let task = target.find(inputs, stage)?;
    if task.case.flow_association.is_none() {
        return Err(Failure::new(
            stage,
            Code::ObjectMissing,
            "saved H3 case has no flow association",
        )
        .target(&target.reference()));
    }
    Ok(task)
}

fn query_value(
    query: &QueryKind,
    inputs: &StoredEngineeringInputs,
    package_id: Option<&str>,
    run: Option<(&CaseTarget, &Value)>,
) -> Result<Value, Failure> {
    Ok(match query {
        QueryKind::Objects {} => Value::Array(
            inputs
                .tasks
                .iter()
                .filter_map(|task| task.as_concentric_tube_rating())
                .filter(|task| task.case.flow_association.is_some())
                .map(h3_records::object)
                .collect(),
        ),
        QueryKind::Record { target } => records::record(
            associated_case(target, inputs, Stage::Query)?,
            &inputs.basis,
            package_id,
        ),
        QueryKind::Results {
            scope: ResultScope::Object(target),
        } => {
            associated_case(target, inputs, Stage::Query)?;
            run.filter(|(rated, _)| *rated == target)
                .map_or_else(|| json!({"status":"missing"}), |(_, value)| value.clone())
        }
    })
}

fn rate(
    step: &Step,
    project: ReviewProject,
    inputs: &StoredEngineeringInputs,
    package_id: Option<&str>,
    snapshot: &ProjectSnapshot,
) -> Result<Value, Failure> {
    let Action::RateConcentricTubeFromFlow {
        target,
        expected_revisions,
        qualification,
    } = &step.action;
    let task = associated_case(target, inputs, Stage::Step)?;
    let expected = RatingRevisions::from(&expected_revisions.0);
    let actual = RatingRevisions::from(task);
    if actual != expected {
        let mut error = Failure::new(
            Stage::Step,
            Code::RecordConflict,
            "saved H3 input revisions differ",
        )
        .target(&target.reference());
        error.details = Some(Box::new(json!({
            "expected_revisions": h3_records::revisions(expected),
            "actual_revisions": h3_records::revisions(actual),
        })));
        return Err(error);
    }
    let reject = |error| records::diagnostic_failure(target, task, Status::Blocked, error);
    let failed = |error| records::diagnostic_failure(target, task, Status::Failed, error);
    let reference = &qualification.0;
    let approved =
        bundled_water::qualification(&reference.id, reference.revision).map_err(reject)?;
    bundled_water::check_case(&approved, package_id, &inputs.basis, &task.case).map_err(reject)?;
    let provider = bundled_water::provider(&approved).map_err(reject)?;
    let mut review = H3FlowReview::from_saved_project(
        &project.path,
        *project.data,
        &project.contents,
        approved.clone(),
    )
    .map_err(failed)?;
    review.solve_flow(&provider).map_err(failed)?;
    let member = EquipmentCaseKey {
        task_id: task.id.clone(),
        case_id: task.case.id.clone(),
    };
    let preview = review.preview_saved(member).map_err(reject)?;
    let origin = records::origin(snapshot, task, step, &inputs.basis, &preview, &approved);
    let rated = review.rate(preview).map_err(failed)?;
    let value = review.current_rating(&rated).map_err(failed)?;
    let mut result = h3_records::current_result(*value);
    result["origin"] = origin;
    Ok(result)
}

fn reply_step(
    step: &Step,
    rated: Result<Value, Failure>,
    response: &mut Response,
) -> Option<Value> {
    match rated {
        Ok(result) => {
            response
                .steps
                .push(json!({"id":step.id,"outcome":"completed","result":result}));
            Some(result)
        }
        Err(mut error) => {
            error.step_id = Some(step.id.clone());
            let result = (error.status == Status::Failed).then(|| {
                json!({"status":"failed","diagnostic":error.details.as_ref().and_then(|d| d.get("diagnostic"))})
            });
            let mut receipt = json!({"id":step.id,"outcome":"rejected","error":error});
            if let Some(result) = &result {
                receipt["outcome"] = json!("failed");
                receipt["result"] = result.clone();
            }
            response.steps.push(receipt);
            response.fail(error);
            result
        }
    }
}
