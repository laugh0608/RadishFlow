//! Structured SI evidence: full equations, conditional solves and actual verification.
use crate::result_evidence::{Numbers, stream, units};
use rf_solver::equation_oriented::*;
use rf_types::{RfError, RfResult};
use serde_json::{Value, json};

pub fn port(p: &PortBinding) -> Value {
    json!({"unit_id":p.unit.as_str(),"port":p.port.name(),"stream_id":p.stream.as_str()})
}
pub fn parameter(p: &ParameterBinding) -> Value {
    json!({"unit_id":p.unit.as_str(),"field":"outlet_a_split_fraction"})
}
pub fn variable(v: &VariableBinding) -> Value {
    match v {
        VariableBinding::Parameter(p) => json!({"kind":"parameter","parameter":parameter(p)}),
        VariableBinding::PortComponent { port: p, component } => {
            json!({"kind":"port_component","port":port(p),"component_id":component.as_str()})
        }
    }
}
pub fn equation(e: &EquationBinding) -> Value {
    match e {
        EquationBinding::Unit {
            unit,
            role,
            component,
        } => {
            json!({"kind":"unit","unit_id":unit.as_str(),"role":match role {MaterialEquationRole::FeedBoundary=>"feed_boundary",MaterialEquationRole::MixerBalance=>"mixer_balance",MaterialEquationRole::SplitA=>"split_a",MaterialEquationRole::SplitB=>"split_b"},"component_id":component.as_str()})
        }
        EquationBinding::Connection {
            source,
            sink,
            component,
        } => {
            json!({"kind":"connection","source":port(source),"sink":port(sink),"component_id":component.as_str()})
        }
        EquationBinding::ProductTotal { source } => {
            json!({"kind":"product_total","source":port(source)})
        }
    }
}
fn region(r: &StructuralRegion) -> Value {
    json!({"variables":r.variables.iter().map(variable).collect::<Vec<_>>(),"equations":r.equations.iter().map(equation).collect::<Vec<_>>()})
}
pub fn structure(s: &StructuralReport) -> Value {
    json!({"variable_count":s.variable_count,"equation_count":s.equation_count,"matching_count":s.matching.len(),"square_matched":s.is_square_matched(),"matching":s.matching.iter().map(|(e,v)|json!({"equation":equation(e),"variable":variable(v)})).collect::<Vec<_>>(),"unmatched_variables":s.unmatched_variables.iter().map(variable).collect::<Vec<_>>(),"unmatched_equations":s.unmatched_equations.iter().map(equation).collect::<Vec<_>>(),"underdetermined":region(&s.underdetermined),"overdetermined":region(&s.overdetermined),"numerical_solvability":"not_proven"})
}
fn definition_with(d: &EquationDefinition, n: Numbers) -> RfResult<Value> {
    let stored = crate::flow_plan_actions::stored_flow_plan(
        &rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(d.options.clone()),
    );
    Ok(
        json!({"variables":d.variables.iter().map(variable).collect::<Vec<_>>(),"equations":d.equations.iter().map(equation).collect::<Vec<_>>(),"variable_scales_si":n.vector(&d.variable_scales_si)?,"residual_scales_mol_s":n.vector(&d.residual_scales_mol_s)?,"structure":structure(&d.structure),"fresh_total_mol_s":n.value(d.fresh_total_mol_s)?,"fixed_split_fractions":d.fixed_split_fractions.iter().map(|(p,v)|Ok(json!({"binding":parameter(p),"value":n.value(*v)?}))).collect::<RfResult<Vec<_>>>()?,"configuration":stored}),
    )
}
pub fn definition(d: &EquationDefinition) -> RfResult<Value> {
    definition_with(d, Numbers::Current)
}
fn linear(l: &LinearEvidence, n: Numbers) -> RfResult<Value> {
    Ok(
        json!({"dimension":l.dimension,"matrix_inf_norm":n.value(l.matrix_inf_norm)?,"pivot_threshold":n.value(l.pivot_threshold)?,"row_swaps":l.row_swaps,"rcond_indicator":n.optional(l.rcond_indicator)?,"backward_error":n.optional(l.backward_error)?}),
    )
}
fn map(v: &std::collections::BTreeMap<rf_types::ComponentId, f64>, n: Numbers) -> RfResult<Value> {
    Ok(Value::Object(
        v.iter()
            .map(|(id, v)| Ok((id.to_string(), n.value(*v)?)))
            .collect::<RfResult<_>>()?,
    ))
}
fn evidence(e: &EquationEvidence, n: Numbers) -> RfResult<Value> {
    let samples=e.conditional_evaluations.iter().map(|s|Ok(json!({"fraction":n.optional(s.fraction)?,"interval":s.interval.map(|(l,u)|n.vector(&[l,u])).transpose()?,"linear":s.linear.as_ref().map(|l|linear(l,n)).transpose()?,"target_residual_mol_s":n.optional(s.target_residual_mol_s)?,"completed":s.completed}))).collect::<RfResult<Vec<_>>>()?;
    let solved=e.solved_parameter.as_ref().map(|s|Ok::<_,RfError>(json!({"binding":parameter(&s.binding),"stored_value":n.optional(s.stored_value)?,"solved_value":n.value(s.value)?,"interval":n.vector(&[s.interval.0,s.interval.1])?}))).transpose()?;
    let target=e.verified_target.as_ref().map(|t|Ok::<_,RfError>(json!({"source":port(&t.target.source),"target_mol_s":n.value(t.target.value_mol_s)?,"actual_mol_s":n.value(t.actual_mol_s)?,"residual_mol_s":n.value(t.residual_mol_s)?,"scaled_residual":n.value(t.scaled_residual)?}))).transpose()?;
    let global=e.global_material.as_ref().map(|g|Ok::<_,RfError>(json!({"fresh_components_mol_s":map(&g.fresh_components_mol_s,n)?,"product_components_mol_s":map(&g.product_components_mol_s,n)?,"component_residuals_mol_s":map(&g.component_residuals_mol_s,n)?,"total_residual_mol_s":n.value(g.total_residual_mol_s)?}))).transpose()?;
    Ok(
        json!({"definition":e.definition.as_ref().map(|d|definition_with(d,n)).transpose()?,"conditional_evaluations":samples,"candidate_values_si":e.candidate_values_si.as_ref().map(|v|n.vector(v)).transpose()?,"candidate_residuals_mol_s":e.candidate_residuals_mol_s.as_ref().map(|v|n.vector(v)).transpose()?,"full_jacobian":e.full_jacobian.as_ref().map(|l|linear(l,n)).transpose()?,"verified_values_si":e.verified_values_si.as_ref().map(|v|n.vector(v)).transpose()?,"verified_residuals_mol_s":e.verified_residuals_mol_s.as_ref().map(|v|n.vector(v)).transpose()?,"solved_parameter":solved,"verified_target":target,"final_units":units(&e.final_units,n)?,"zero_flow_sources":e.zero_flow_sources.iter().map(|(p,s)|json!({"port":port(p),"source":port(s)})).collect::<Vec<_>>(),"global_material":global}),
    )
}
pub fn stage(s: EquationStage) -> &'static str {
    match s {
        EquationStage::Assemble => "assemble",
        EquationStage::StructuralAnalysis => "structural_analysis",
        EquationStage::LinearEvaluation => "linear_evaluation",
        EquationStage::BracketEndpoint => "bracket_endpoint",
        EquationStage::IntervalEvaluation => "interval_evaluation",
        EquationStage::Factorization => "factorization",
        EquationStage::ConditionSolve => "condition_solve",
        EquationStage::FullJacobianCheck => "full_jacobian_check",
        EquationStage::FinalUnitVerification => "final_unit_verification",
        EquationStage::GlobalVerification => "global_verification",
        EquationStage::Completing => "completing",
    }
}
pub fn progress(p: &EquationProgress) -> Value {
    json!({"stage":stage(p.stage),"unit_id":p.unit.as_ref().map(|id|id.as_str()),"row":p.row,"column":p.column,"iteration":p.iteration})
}
pub fn failure_kind(k: EquationFailureKind) -> &'static str {
    match k {
        EquationFailureKind::UnsupportedModel => "unsupported_model",
        EquationFailureKind::InvalidSpecification => "invalid_specification",
        EquationFailureKind::InvalidBinding => "invalid_binding",
        EquationFailureKind::Capacity => "capacity",
        EquationFailureKind::StructuralUnder => "structural_under",
        EquationFailureKind::StructuralOver => "structural_over",
        EquationFailureKind::StructuralSingular => "structural_singular",
        EquationFailureKind::LinearSingular => "linear_singular",
        EquationFailureKind::IllConditioned => "ill_conditioned",
        EquationFailureKind::LinearVerification => "linear_verification",
        EquationFailureKind::TargetNotBracketed => "target_not_bracketed",
        EquationFailureKind::BudgetExceeded => "budget_exceeded",
        EquationFailureKind::Stagnated => "stagnated",
        EquationFailureKind::NonFinite => "non_finite",
        EquationFailureKind::PhysicalDomain => "physical_domain",
        EquationFailureKind::UndefinedComposition => "undefined_composition",
        EquationFailureKind::LocalVerification => "local_verification",
        EquationFailureKind::GlobalVerification => "global_verification",
        EquationFailureKind::Cancelled => "cancelled",
    }
}
pub fn costs(c: &EquationCosts) -> Value {
    json!({"assemblies":c.assemblies,"structural_analyses":c.structural_analyses,"structural_checkpoints":c.structural_checkpoints,"conditional_linear_evaluations":c.conditional_linear_evaluations,"full_jacobian_evaluations":c.full_jacobian_evaluations,"factorizations":c.factorizations,"triangular_solves":c.triangular_solves,"target_evaluations":c.target_evaluations,"interval_updates":c.interval_updates,"operation_constructions":c.operation_constructions,"local_unit_calls":c.local_unit_calls,"calls_by_unit":c.calls_by_unit,"thermo_calls":c.thermo_calls,"flash_calls":c.flash_calls,"stage_elapsed_s":c.stage_elapsed.iter().map(|(s,d)|(stage(*s).to_string(),json!(d.as_secs_f64()))).collect::<serde_json::Map<_,_>>(),"elapsed_s":c.elapsed.as_secs_f64()})
}
pub fn result(r: &EquationResult) -> RfResult<Value> {
    Ok(
        json!({"execution_kind":"equation_oriented","core_method":r.backend,"equation_contract_version":r.equation_contract_version,"acceptance_contract_version":r.acceptance_contract_version,"streams":r.streams.values().map(|s|stream(s,Numbers::Current)).collect::<RfResult<Vec<_>>>()?,"evidence":evidence(&r.evidence,Numbers::Current)?,"costs":costs(&r.costs)}),
    )
}
pub fn failure(f: &EquationFailure) -> RfResult<Value> {
    let c = f.error.context();
    Ok(
        json!({"failure_kind":failure_kind(f.kind),"core_method":f.backend,"code":c.diagnostic_code(),"error_code":f.error.code().as_str(),"message":f.error.to_string(),"unit_ids":c.related_unit_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"stream_ids":c.related_stream_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"component_ids":c.related_component_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"ports":c.related_port_targets().iter().map(crate::modular_result_projection::port).collect::<Vec<_>>(),"progress":progress(&f.progress),"evidence":evidence(&f.evidence,Numbers::Diagnostic)?,"costs":costs(&f.costs)}),
    )
}
