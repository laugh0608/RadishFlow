//! Shared ordered execution and response publication, independent of request version.
use super::v3::{protocol::*, references::Bindings};
use serde_json::{Value, json};
pub(super) trait StepIdentity {
    fn id(&self) -> &str;
    fn target(&self) -> Option<ObjectRef>;
}
impl StepIdentity for super::v3::actions::Step {
    fn id(&self) -> &str {
        &self.id
    }
    fn target(&self) -> Option<ObjectRef> {
        self.action.target()
    }
}
pub(super) fn steps<S: StepIdentity>(
    app: &mut rf_ui::AppState,
    steps: &[S],
    response: &mut Response,
    refs: &mut Bindings,
    mut execute: impl FnMut(
        &mut rf_ui::AppState,
        &mut Bindings,
        usize,
        &S,
    ) -> Result<(Value, Option<Failure>), Failure>,
) {
    for (index, step) in steps.iter().enumerate() {
        match execute(app, refs, index, step) {
            Ok((mut receipt, error)) => {
                receipt["id"] = json!(step.id());
                if let Some(mut error) = error {
                    error.step_id = Some(step.id().to_string());
                    if error.target.is_none() {
                        error.target = step.target().map(Box::new);
                    }
                    receipt["error"] = json!(error);
                    response.steps.push(receipt);
                    response.fail(error);
                    break;
                }
                refs.0.entry(step.id().to_string()).or_default();
                response.steps.push(receipt);
            }
            Err(mut error) => {
                error.stage = Stage::Step;
                if error.target.is_none() {
                    error.target = step.target().map(Box::new);
                }
                error.step_id = Some(step.id().to_string());
                response.steps.push(json!({"id":step.id(),"outcome":if error.status == Status::ConfirmationRequired { "confirmation_required" } else { "rejected" },"error":error}));
                response.fail(error);
                break;
            }
        }
    }
}
