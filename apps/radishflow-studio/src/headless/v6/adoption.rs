use super::super::v4::protocol::FractionTarget;
use super::*;
use crate::equation_actions::{adopt_equation_candidate, preview_equation_adoption};
use crate::flow_actions::{FlowEditAccess, FlowEditOutcome};
use old::protocol::{Code, Stage};
use old::references::BoundOutput;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRef {
    pub step: String,
}
pub(in crate::headless) fn execute(
    app: &mut AppState,
    refs: &mut Bindings,
    executions: &Executions,
    step: &str,
    expected: &old::actions::RevisionRef,
    execution: &ExecutionRef,
    target: &FractionTarget,
) -> Result<Value, Failure> {
    let w = &app.workspace;
    refs.revision(
        expected,
        Output::Document,
        w.document.metadata.document_id.as_str(),
        w.document.revision,
    )?;
    let rf_ui::result_query::FlowResultView::Current { origin, .. } = w.flow_result() else {
        return Err(invalid("adoption requires a Current execution"));
    };
    let source = origin
        .execution()
        .and_then(|e| executions.step(&crate::StudioRunId::Flow(e.run_id.clone())));
    if source != Some(execution.step.as_str()) {
        return Err(Failure::new(
            Stage::Step,
            Code::UnknownReference,
            "execution must reference the accepted Current EO run in this request",
        ));
    }
    let preview =
        preview_equation_adoption(app, FlowEditAccess::Editable, false).map_err(domain)?;
    let object = refs.object(&target.object)?;
    if object
        != rf_ui::project_query::ProjectObjectId::Flow(rf_ui::variable_browser::ObjectId::Unit(
            preview.binding().unit.clone(),
        ))
    {
        return Err(Failure::new(
            Stage::Step,
            Code::ReferenceKindMismatch,
            "adoption field differs from the released result parameter",
        ));
    }
    let stored = preview.stored_value();
    let solved = preview.solved_value();
    let binding = crate::equation_result_projection::parameter(preview.binding());
    let receipt = adopt_equation_candidate(
        app,
        FlowEditAccess::Editable,
        false,
        &preview,
        std::time::SystemTime::now(),
    )
    .map_err(domain)?;
    let outputs = std::collections::BTreeMap::from([(
        Output::Document,
        BoundOutput {
            id: receipt.after.document_id().to_string(),
            revision: receipt.after.revision(),
        },
    )]);
    refs.0.insert(step.into(), outputs.clone());
    Ok(
        json!({"outcome":if receipt.outcome==FlowEditOutcome::Committed {"committed"}else{"unchanged"},"before_revision":receipt.before.revision(),"after_revision":receipt.after.revision(),"outputs":outputs,"binding":binding,"stored_value":stored,"adopted_value":solved,"source_execution":execution.step,"plan_revision":app.workspace.flow_plan.revision(),"result_status":if receipt.outcome==FlowEditOutcome::Committed {"stale"}else{"current"},"persisted":false}),
    )
}
