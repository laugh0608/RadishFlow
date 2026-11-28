//! CLI v12 plan vocabulary with measured partitioned material-balance diagnostics.
use super::{
    planned::PlanProtocol,
    v3::{
        protocol::Failure,
        references::{Bindings, Executions},
    },
    v12,
};
use crate::result_evidence::diagnostic_number;
use rf_solver::partitioned::{
    ComponentBalanceExcess, MaterialBalanceFailure, MaterialBalanceScope,
};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const VERSION: u32 = 16;

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Plan(v12::Plan);

impl PlanProtocol for Plan {
    const VERSION: u32 = VERSION;

    fn configuration(plan: &Self, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        v12::Plan::configuration(&plan.0, refs)
    }

    fn wire(plan: &FlowPlanConfiguration) -> Result<Value, Failure> {
        v12::Plan::wire(plan)
    }

    fn query(app: &AppState) -> Result<Value, Failure> {
        v12::Plan::query(app)
    }

    fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
        let mut result = v12::Plan::flow(app, executions)?;
        if let Some(balance) = app
            .workspace
            .partitioned_failure_evidence()
            .and_then(|failure| failure.material_balance.as_ref())
        {
            result["diagnostic_evidence"]["material_balance"] = material_balance(balance);
        }
        Ok(result)
    }
}

fn material_balance(balance: &MaterialBalanceFailure) -> Value {
    let scope = match &balance.scope {
        MaterialBalanceScope::Unit {
            block_index,
            unit_id,
        } => {
            json!({ "kind": "unit", "block_index": block_index, "unit_id": unit_id })
        }
        MaterialBalanceScope::RecycleRegion {
            block_index,
            tear_stream_id,
        } => {
            json!({ "kind": "recycle_region", "block_index": block_index, "tear_stream_id": tear_stream_id })
        }
        MaterialBalanceScope::Flowsheet => json!({ "kind": "flowsheet" }),
    };
    let readings = &balance.readings;
    let tolerances = &readings.tolerances;
    json!({
        "scope": scope,
        "input_stream_ids": readings.inputs,
        "output_stream_ids": readings.outputs,
        "tolerances": {
            "component_absolute_flow_mol_s": diagnostic_number(tolerances.component_absolute_flow_mol_s),
            "scaled_flow": diagnostic_number(tolerances.scaled_flow),
            "total_absolute_flow_mol_s": diagnostic_number(tolerances.total_absolute_flow_mol_s),
        },
        "exceeded_components": readings.exceeded_components.iter().map(component).collect::<Vec<_>>(),
        "total": {
            "input_mol_s": diagnostic_number(readings.total.input_mol_s),
            "output_mol_s": diagnostic_number(readings.total.output_mol_s),
            "difference_mol_s": diagnostic_number(readings.total.difference_mol_s),
            "exceeded": readings.total_exceeded,
        },
    })
}

fn component(row: &ComponentBalanceExcess) -> Value {
    json!({
        "component_id": row.component_id,
        "input_mol_s": diagnostic_number(row.flow.input_mol_s),
        "output_mol_s": diagnostic_number(row.flow.output_mol_s),
        "difference_mol_s": diagnostic_number(row.flow.difference_mol_s),
        "scale_mol_s": diagnostic_number(row.scale_mol_s),
        "scaled_absolute": diagnostic_number(row.scaled_absolute),
        "absolute_exceeded": row.absolute_exceeded,
        "scaled_exceeded": row.scaled_exceeded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_solver::partitioned::{FlowDifference, MaterialBalanceReadings, RecycleTolerances};

    #[test]
    fn total_only_balance_preserves_nonfinite_values_and_empty_components() {
        let mut failure = MaterialBalanceFailure {
            scope: MaterialBalanceScope::Flowsheet,
            readings: Box::new(MaterialBalanceReadings {
                inputs: vec!["feed".into()],
                outputs: vec!["product".into()],
                exceeded_components: vec![],
                tolerances: RecycleTolerances {
                    component_absolute_flow_mol_s: 1e-10,
                    total_absolute_flow_mol_s: 2e-10,
                    scaled_flow: 1e-10,
                    temperature_k: 1e-9,
                    pressure_pa: 1e-6,
                },
                total: FlowDifference {
                    input_mol_s: f64::INFINITY,
                    output_mol_s: f64::INFINITY,
                    difference_mol_s: f64::NAN,
                },
                total_exceeded: true,
            }),
        };
        let value = material_balance(&failure);
        assert_eq!(value["exceeded_components"], json!([]));
        assert_eq!(
            value["total"],
            json!({
                "input_mol_s": {"non_finite": "positive_infinity"},
                "output_mol_s": {"non_finite": "positive_infinity"},
                "difference_mol_s": {"non_finite": "nan"},
                "exceeded": true,
            })
        );
        failure.readings.total.output_mol_s = 1.0;
        failure.readings.total.difference_mol_s = f64::NEG_INFINITY;
        assert_eq!(
            material_balance(&failure)["total"]["difference_mol_s"],
            json!({"non_finite": "negative_infinity"})
        );
    }
}
