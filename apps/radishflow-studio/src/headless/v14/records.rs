use super::{Action, CaseTarget, Step, h3_records};
use crate::document_lifecycle_driver::h3_flow_review::{H3FlowPreview, H3PureWaterQualification};
use crate::headless::v3::protocol::{Code, Failure, ProjectSnapshot, Stage, Status};
use rf_model::{
    engineering::EngineeringBasis,
    equipment::{ConcentricTubeFlowStream, ConcentricTubeLiquidInlet, ConcentricTubeRatingTask},
};
use rf_types::{RfError, SourceId};
use serde_json::{Value, json};

pub(super) fn record(
    task: &ConcentricTubeRatingTask,
    basis: &EngineeringBasis,
    package_id: Option<&str>,
) -> Value {
    let mut record = h3_records::record(task, basis);
    let association = task
        .case
        .flow_association
        .as_ref()
        .expect("only saved associations are projected");
    record["case"]["flow_association"] =
        json!({"hot":stream(&association.hot), "cold":stream(&association.cold)});
    record["property_package_id"] = json!(package_id);
    record
}

fn stream(stream: &ConcentricTubeFlowStream) -> Value {
    json!({"stream_id":stream.id,"creation_id":stream.creation_id.as_str()})
}

fn effective_inlet(
    (reference, inlet): (&ConcentricTubeFlowStream, ConcentricTubeLiquidInlet),
) -> Value {
    json!({"stream":stream(reference),"inlet":{
        "temperature_k":inlet.temperature_k,"pressure_pa":inlet.pressure_pa,
        "molar_flow_mol_s":inlet.molar_flow_mol_s,
    }})
}

pub(super) fn origin(
    project: &ProjectSnapshot,
    task: &ConcentricTubeRatingTask,
    step: &Step,
    basis: &EngineeringBasis,
    preview: &H3FlowPreview,
    approved: &H3PureWaterQualification,
) -> Value {
    let mut origin = h3_records::origin(project, task, &step.id);
    let Action::RateConcentricTubeFromFlow { qualification, .. } = &step.action;
    let execution = preview
        .origin()
        .execution()
        .expect("H3 preview requires execution evidence");
    let package = execution
        .package
        .as_ref()
        .expect("H3 preview requires a qualified package");
    origin["flow_run"] = json!({
        "run_id":step.id,"scope":"request","snapshot_id":preview.snapshot_id().as_str(),
        "sequence":preview.sequence(),"flow_revision":preview.origin().revision(),
        "package":{"id":package.package_id,"version":package.version,"payload_fingerprint":package.payload_fingerprint},
    });
    origin["hot"] = effective_inlet(preview.hot());
    origin["cold"] = effective_inlet(preview.cold());
    origin["qualification"] = json!({
        "id":qualification.0.id,"revision":qualification.0.revision,
        "phase_reference":approved.phase_reference,
    });
    let source = |id: &SourceId| {
        basis
            .sources
            .iter()
            .find(|entry| &entry.id == id)
            .map(|entry| json!({"id":entry.id,"revision":entry.revision}))
    };
    origin["sources"] = json!({
        "geometry":task.geometry.source_id.as_ref().and_then(source),
        "hot_properties":source(&task.case.hot.properties.source_id),
        "cold_properties":source(&task.case.cold.properties.source_id),
    });
    origin
}

pub(super) fn diagnostic_failure(
    target: &CaseTarget,
    task: &ConcentricTubeRatingTask,
    status: Status,
    cause: RfError,
) -> Failure {
    let context = cause.context();
    let association = task
        .case
        .flow_association
        .as_ref()
        .expect("target has a saved association");
    let streams: Vec<_> = [("hot", &association.hot), ("cold", &association.cold)]
        .into_iter()
        .filter(|(_, reference)| context.related_stream_ids().contains(&reference.id))
        .map(|(side, reference)| json!({"side":side,"stream":stream(reference)}))
        .collect();
    let diagnostic = json!({
        "code":context.diagnostic_code().unwrap_or(cause.code().as_str()),
        "message":cause.message(),
        "streams":streams,
        "related_stream_ids":context.related_stream_ids(),
        "related_unit_ids":context.related_unit_ids(),
    });
    let code = if status == Status::Failed {
        Code::RunFailed
    } else {
        Code::ActionRejected
    };
    let mut failure = Failure::new(Stage::Step, code, cause.message()).target(&target.reference());
    failure.status = status;
    failure.details = Some(Box::new(json!({"diagnostic":diagnostic})));
    failure
}
