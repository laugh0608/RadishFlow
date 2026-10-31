//! Explicit v3 material task extension; existing flow/equipment wire records stay unchanged.
use super::{protocol::*, references::Executions, results::measure};
use crate::{StudioRunId, material_balance_actions::MaterialBalanceActionError};
use rf_solver::component_balance::*;
use rf_types::{UnitId, units::QuantityKind};
use rf_ui::material_balance::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    LowerTriangularSubstitution,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scales {
    pub inlet_a: f64,
    pub inlet_b: f64,
    pub outlet: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSettings {
    pub algorithm: Algorithm,
    pub scales_mol_s: Scales,
    pub absolute_tolerance_mol_s: f64,
    pub scaled_tolerance: f64,
    #[serde(
        default,
        deserialize_with = "super::strict::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub outlet_total_molar_flow_mol_s: Option<f64>,
}
impl MaterialSettings {
    pub fn plan(&self, mixer: UnitId) -> MaterialBalancePlan {
        let mut specification = MixerBalanceSpecification::new(
            mixer,
            MixerBalanceScales {
                inlet_a_mol_s: self.scales_mol_s.inlet_a,
                inlet_b_mol_s: self.scales_mol_s.inlet_b,
                outlet_mol_s: self.scales_mol_s.outlet,
            },
        );
        specification.outlet_total_molar_flow_mol_s = self.outlet_total_molar_flow_mol_s;
        MaterialBalancePlan {
            specification,
            options: MaterialBalanceOptions {
                algorithm: match self.algorithm {
                    Algorithm::LowerTriangularSubstitution => {
                        MaterialBalanceAlgorithm::LowerTriangularSubstitution
                    }
                },
                absolute_tolerance_mol_s: self.absolute_tolerance_mol_s,
                scaled_tolerance: self.scaled_tolerance,
            },
        }
    }
}
pub fn action_error(error: MaterialBalanceActionError) -> Failure {
    let details = match &error {
        MaterialBalanceActionError::ReadOnly => json!({"reason":"read_only"}),
        MaterialBalanceActionError::Context(_) => json!({"reason":"input_context_changed"}),
        MaterialBalanceActionError::PendingDrafts(_) => json!({"reason":"pending_drafts"}),
        MaterialBalanceActionError::MissingPlan => json!({"reason":"missing_plan"}),
        MaterialBalanceActionError::InvalidPlan(error) => {
            json!({"reason":"invalid_material_plan","diagnostic":diagnostic(&error.diagnostic())})
        }
    };
    super::errors::blocked(details, error.to_string())
}
fn diagnostic(error: &rf_types::RfError) -> Value {
    let context = error.context();
    json!({"code":context.diagnostic_code(),"message":error.message(),"unit_ids":context.related_unit_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"stream_ids":context.related_stream_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"ports":context.related_port_targets().iter().map(|p|json!({"unit_id":p.unit_id.as_str(),"port":p.port_name})).collect::<Vec<_>>()})
}
fn origin(origin: &MaterialBalanceOrigin, executions: &Executions) -> Value {
    let q = origin.qualification();
    let plan = origin.plan();
    let scales = plan.specification.scales;
    json!({"document_id":q.input().document_id().as_str(),"revision":q.input().revision(),"mixer_id":plan.specification.mixer_id.as_str(),"execution":{"step":executions.step(&StudioRunId::Material(q.run_id().clone())),"scope":"request","solver_implementation":origin.implementation()},"settings":{"algorithm":"lower_triangular_substitution","scales_mol_s":{"inlet_a":scales.inlet_a_mol_s,"inlet_b":scales.inlet_b_mol_s,"outlet":scales.outlet_mol_s},"absolute_tolerance_mol_s":plan.options.absolute_tolerance_mol_s,"scaled_tolerance":plan.options.scaled_tolerance}})
}
fn binding(value: &ComponentFlowBinding) -> Value {
    json!({"unit_id":value.unit_id.as_str(),"port":value.port.name(),"stream_id":value.stream_id.as_str(),"component_id":value.component_id.as_str()})
}
fn equation(value: &ComponentBalanceEquation) -> Value {
    match value {
        ComponentBalanceEquation::Connection { source, target } => {
            json!({"kind":"connection","source":binding(source),"target":binding(target)})
        }
        ComponentBalanceEquation::MixerConservation {
            unit_id,
            component_id,
        } => {
            json!({"kind":"mixer_conservation","unit_id":unit_id.as_str(),"component_id":component_id.as_str()})
        }
    }
}
pub fn result(app: &rf_ui::AppState, executions: &Executions) -> Value {
    match app.workspace.material_balance_result() {
        MaterialBalanceResultView::Missing => json!({"status":"missing"}),
        MaterialBalanceResultView::Stale { origin: value } => {
            json!({"status":"stale","origin":origin(value,executions)})
        }
        MaterialBalanceResultView::Failed {
            origin: value,
            failure,
        } => {
            json!({"status":"failed","origin":origin(value,executions),"diagnostic":diagnostic(&failure.diagnostic())})
        }
        MaterialBalanceResultView::Current {
            origin: value,
            solution,
        } => {
            json!({"status":"current","origin":origin(value,executions),"variables":solution.variables.iter().zip(&solution.point_mol_s).map(|(v,x)|json!({"binding":binding(&v.binding),"value":measure(*x,QuantityKind::MolarFlow),"scale_mol_s":v.scale_mol_s})).collect::<Vec<_>>(),"equations":solution.equations.iter().zip(&solution.evaluation.residuals_mol_s).zip(&solution.evaluation.scaled_residuals).map(|((r,si),scaled)|json!({"equation":equation(&r.equation),"residual":measure(*si,QuantityKind::MolarFlow),"scaled_residual":scaled,"scale_mol_s":r.scale_mol_s})).collect::<Vec<_>>(),"outlet":{"total_molar_flow":measure(solution.values.total_molar_flow_mol_s,QuantityKind::MolarFlow),"components":solution.values.component_molar_flows_mol_s.iter().map(|(id,flow)|json!({"component_id":id.as_str(),"molar_flow":measure(*flow,QuantityKind::MolarFlow),"mole_fraction":measure(solution.values.mole_fractions[id],QuantityKind::MoleFraction)})).collect::<Vec<_>>()},"statistics":{"solved_rows":solution.solved_rows,"residual_evaluations":solution.residual_evaluations,"elapsed_s":solution.elapsed.as_secs_f64()}})
        }
    }
}
