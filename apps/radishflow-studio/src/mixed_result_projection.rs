//! Structured SI projection; partial blocks are diagnostics, never current streams.
use crate::result_evidence::{Numbers, stream, units};
use rf_solver::mixed::*;
use rf_solver::partitioned::EvaluationPhase;
use rf_types::{RfError, RfResult};
use serde_json::{Value, json};
fn costs(c: &MixedCosts) -> Value {
    json!({"operation_constructions":c.operation_constructions,"local_unit_calls":c.local_unit_calls,"calls_by_unit":c.calls_by_unit,"thermo_calls":c.thermo_calls,"flash_calls":c.flash_calls,"assembly_elapsed_s":c.assembly_elapsed.as_secs_f64(),"elapsed_s":c.elapsed.as_secs_f64()})
}
fn blocks(values: &[MixedBlockReport], n: Numbers) -> RfResult<Vec<Value>> {
    values.iter().map(|b|Ok(json!({"id":b.id.as_str(),"members":b.members.iter().map(|u|u.as_str()).collect::<Vec<_>>(),"elapsed_s":b.elapsed.as_secs_f64(),"evidence":match &b.evidence {
        MixedBlockEvidence::Local(u)=>json!({"kind":"local","final_units":units(std::slice::from_ref(u),n)?}),
        MixedBlockEvidence::EquationOriented(r)=>json!({"kind":"equation_oriented","result":crate::equation_result_projection::result(r)?}),
    }}))).collect()
}
fn boundaries(values: &[MixedBoundaryEvidence], n: Numbers) -> RfResult<Vec<Value>> {
    values.iter().map(|b|Ok(json!({"stream_id":b.connection.stream_id.as_str(),"source":{"unit_id":b.connection.source.unit_id.as_str(),"port":b.connection.source.port_name},"sink":b.connection.sink.as_ref().map(|s|json!({"unit_id":s.unit_id.as_str(),"port":s.port_name})),"producer":stream(&b.producer,n)?,"consumed":stream(&b.consumed,n)?,"residuals_mol_s":b.residuals_mol_s.iter().map(|(id,v)|Ok((id.to_string(),n.value(*v)?))).collect::<RfResult<serde_json::Map<_,_>>>()?,"scaled_residuals":b.scaled_residuals.iter().map(|(id,v)|Ok((id.to_string(),n.value(*v)?))).collect::<RfResult<serde_json::Map<_,_>>>()?}))).collect()
}
fn component_values(
    v: &std::collections::BTreeMap<rf_types::ComponentId, f64>,
    numbers: Numbers,
) -> RfResult<Value> {
    Ok(Value::Object(
        v.iter()
            .map(|(id, v)| Ok((id.to_string(), numbers.value(*v)?)))
            .collect::<RfResult<_>>()?,
    ))
}
fn unsupported_report() -> RfError {
    RfError::invalid_input("the DAG result projection cannot represent an in-memory recycle result")
        .with_diagnostic_code("workspace.plan.in_memory_only")
}
fn failure_kind(k: MixedFailureKind) -> &'static str {
    match k {
        MixedFailureKind::Assembly => "assembly",
        MixedFailureKind::LocalEvaluation => "local_evaluation",
        MixedFailureKind::Equation => "equation",
        MixedFailureKind::Boundary => "boundary",
        MixedFailureKind::IterationLimit => "iteration_limit",
        MixedFailureKind::RecycleVerification => "recycle_verification",
        MixedFailureKind::GlobalVerification => "global_verification",
        MixedFailureKind::Cancelled => "cancelled",
    }
}
fn stage(s: MixedStage) -> &'static str {
    match s {
        MixedStage::Assembly => "assembly",
        MixedStage::BlockStarted => "block_started",
        MixedStage::Equation => "equation",
        MixedStage::Boundary => "boundary",
        MixedStage::BlockCompleted => "block_completed",
        MixedStage::GlobalVerification => "global_verification",
        MixedStage::Completing => "completing",
    }
}
pub fn result(r: &MixedResult) -> RfResult<Value> {
    if r.method != METHOD {
        return Err(unsupported_report());
    }
    result_payload(r, "mixed_material")
}
pub fn failure(f: &MixedFailure) -> RfResult<Value> {
    if f.method != METHOD
        || matches!(
            f.kind,
            MixedFailureKind::IterationLimit | MixedFailureKind::RecycleVerification
        )
    {
        return Err(unsupported_report());
    }
    failure_payload(f)
}

pub(crate) fn recycle_result(r: &MixedResult) -> RfResult<Value> {
    let report = r
        .recycle
        .as_ref()
        .filter(|_| r.method == RECYCLE_METHOD)
        .ok_or_else(|| {
            RfError::invalid_input("verified mixed recycle result requires its recycle report")
                .with_diagnostic_code("workspace.result.missing_recycle_report")
        })?;
    let mut value = result_payload(r, "mixed_recycle")?;
    value["recycle"] = recycle(report, Numbers::Current)?;
    Ok(value)
}

pub(crate) fn recycle_failure(f: &MixedFailure) -> RfResult<Value> {
    if f.method != RECYCLE_METHOD {
        return Err(
            RfError::invalid_input("failure method does not match mixed recycle")
                .with_diagnostic_code("workspace.result.method_mismatch"),
        );
    }
    let mut value = failure_payload(f)?;
    if let Some(report) = &f.recycle {
        value["recycle"] = recycle(report, Numbers::Diagnostic)?;
    }
    Ok(value)
}

fn result_payload(r: &MixedResult, kind: &str) -> RfResult<Value> {
    let numbers = Numbers::Current;
    let material = &r.global_material;
    Ok(json!({
        "execution_kind": kind,
        "streams": r.streams.values().map(|s| stream(s, numbers)).collect::<RfResult<Vec<_>>>()?,
        "blocks": blocks(&r.blocks, numbers)?,
        "boundaries": boundaries(&r.boundaries, numbers)?,
        "global_material": {
            "fresh_components_mol_s": component_values(&material.fresh_components_mol_s, numbers)?,
            "product_components_mol_s": component_values(&material.product_components_mol_s, numbers)?,
            "component_residuals_mol_s": component_values(&material.component_residuals_mol_s, numbers)?,
            "total_residual_mol_s": numbers.value(material.total_residual_mol_s)?,
        },
        "costs": costs(&r.costs),
    }))
}

fn failure_payload(f: &MixedFailure) -> RfResult<Value> {
    Ok(
        json!({"failure_kind":failure_kind(f.kind),"error_code":f.error.context().diagnostic_code(),"message":f.error.to_string(),"block_id":f.progress.block.as_ref().map(|b|b.as_str()),"unit_id":f.progress.unit.as_ref().map(|u|u.as_str()),"stage":stage(f.progress.stage),"completed_blocks":blocks(&f.completed_blocks,Numbers::Diagnostic)?,"boundaries":boundaries(&f.boundaries,Numbers::Diagnostic)?,"active_units":units(&f.active_units,Numbers::Diagnostic)?,"equation_failure":f.equation_failure.as_ref().map(|e|crate::equation_result_projection::failure(e)).transpose()?,"costs":costs(&f.costs)}),
    )
}

fn recycle(report: &MixedRecycleReport, numbers: Numbers) -> RfResult<Value> {
    let residual_trace = report
        .residual_trace
        .iter()
        .map(|sample| {
            Ok(json!({
                "phase": match sample.phase {
                    EvaluationPhase::Iteration => "iteration",
                    EvaluationPhase::Verification => "verification",
                },
                "evaluation": sample.evaluation,
                "max_absolute_mol_s": numbers.value(sample.max_absolute_mol_s)?,
                "max_scaled": numbers.value(sample.max_scaled)?,
            }))
        })
        .collect::<RfResult<Vec<_>>>()?;
    let last_sweep = report
        .last_sweep
        .as_ref()
        .map(|sweep| -> RfResult<Value> {
            Ok(json!({
                "assumed": stream(&sweep.assumed, numbers)?,
                "returned": stream(&sweep.returned, numbers)?,
                "residuals_mol_s": component_values(&sweep.residuals_mol_s, numbers)?,
            }))
        })
        .transpose()?;
    Ok(json!({
        "stream_id": report.tear.stream_id,
        "fresh_stream_id": report.fresh_stream,
        "max_iteration_evaluations": report.max_iteration_evaluations,
        "component_scales_mol_s": component_values(&report.component_scales_mol_s, numbers)?,
        "residual_trace": residual_trace,
        "last_sweep": last_sweep,
    }))
}
