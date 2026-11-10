//! CLI v4 vocabulary remains fixed; versioned execution is shared with v5.
pub(super) mod plan;
pub(super) mod protocol;
pub(super) mod results;
use super::planned::*;
use super::v3 as old;
use old::{
    protocol::{Code, Failure, IdentityRef, Output, Stage},
    references::{Bindings, Executions},
};
use rf_ui::AppState;
use serde_json::{Value, json};
impl PlanProtocol for protocol::Plan {
    const VERSION: u32 = 4;
    fn configuration(
        p: &Self,
        refs: &Bindings,
    ) -> Result<rf_ui::flow_plan::FlowPlanConfiguration, Failure> {
        plan::configuration(p, refs)
    }
    fn wire(p: &rf_ui::flow_plan::FlowPlanConfiguration) -> Result<Value, Failure> {
        serde_json::to_value(plan::wire(p)?).map_err(old::invalid_request)
    }
    fn query(app: &AppState) -> Result<Value, Failure> {
        plan::query(app)
    }
    fn flow(app: &AppState, e: &Executions) -> Result<Value, Failure> {
        results::flow(app, e)
    }
}
pub(super) use super::planned::{execute_file, query_file};
