//! Structured SI projection; partial blocks are diagnostics, never current streams.
use crate::result_evidence::{Numbers, stream, units};
use rf_solver::mixed::*;
use rf_types::RfResult;
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
fn component_values(v: &std::collections::BTreeMap<rf_types::ComponentId, f64>) -> RfResult<Value> {
    Ok(Value::Object(
        v.iter()
            .map(|(id, v)| Ok((id.to_string(), Numbers::Current.value(*v)?)))
            .collect::<RfResult<_>>()?,
    ))
}
fn failure_kind(k: MixedFailureKind) -> &'static str {
    match k {
        MixedFailureKind::Assembly => "assembly",
        MixedFailureKind::LocalEvaluation => "local_evaluation",
        MixedFailureKind::Equation => "equation",
        MixedFailureKind::Boundary => "boundary",
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
    Ok(
        json!({"execution_kind":"mixed_material","streams":r.streams.values().map(|s|stream(s,Numbers::Current)).collect::<RfResult<Vec<_>>>()?,"blocks":blocks(&r.blocks,Numbers::Current)?,"boundaries":boundaries(&r.boundaries,Numbers::Current)?,"global_material":{"fresh_components_mol_s":component_values(&r.global_material.fresh_components_mol_s)?,"product_components_mol_s":component_values(&r.global_material.product_components_mol_s)?,"component_residuals_mol_s":component_values(&r.global_material.component_residuals_mol_s)?,"total_residual_mol_s":Numbers::Current.value(r.global_material.total_residual_mol_s)?},"costs":costs(&r.costs)}),
    )
}
pub fn failure(f: &MixedFailure) -> RfResult<Value> {
    Ok(
        json!({"failure_kind":failure_kind(f.kind),"error_code":f.error.context().diagnostic_code(),"message":f.error.to_string(),"block_id":f.progress.block.as_ref().map(|b|b.as_str()),"unit_id":f.progress.unit.as_ref().map(|u|u.as_str()),"stage":stage(f.progress.stage),"completed_blocks":blocks(&f.completed_blocks,Numbers::Diagnostic)?,"boundaries":boundaries(&f.boundaries,Numbers::Diagnostic)?,"active_units":units(&f.active_units,Numbers::Diagnostic)?,"equation_failure":f.equation_failure.as_ref().map(|e|crate::equation_result_projection::failure(e)).transpose()?,"costs":costs(&f.costs)}),
    )
}
