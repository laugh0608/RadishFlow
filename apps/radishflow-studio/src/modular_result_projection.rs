//! Lossless SI evidence projection shared by structured consumers and text output.
//! Diagnostic non-finite numbers are explicit; they can never become success data.
use rf_solver::simultaneous::*;
use rf_types::{DiagnosticPortTarget, RfError, RfResult};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
enum Numbers {
    Current,
    Diagnostic,
}
impl Numbers {
    fn value(self, value: f64) -> RfResult<Value> {
        if value.is_finite() {
            return Ok(json!(value));
        }
        match self {
            Self::Current => Err(RfError::invalid_input(
                "nonfinite value in formal modular result",
            )
            .with_diagnostic_code("workspace.result.nonfinite")),
            Self::Diagnostic => Ok(
                json!({"non_finite": if value.is_nan() {"nan"} else if value.is_sign_positive() {"positive_infinity"} else {"negative_infinity"}}),
            ),
        }
    }
    fn optional(self, value: Option<f64>) -> RfResult<Value> {
        value.map(|v| self.value(v)).unwrap_or(Ok(Value::Null))
    }
    fn vector(self, values: &[f64]) -> RfResult<Vec<Value>> {
        values.iter().map(|v| self.value(*v)).collect()
    }
}

pub(crate) fn port(p: &DiagnosticPortTarget) -> Value {
    json!({"unit_id":p.unit_id.as_str(),"port":p.port_name})
}
pub(crate) fn binding(b: &CouplingBinding) -> Value {
    json!({"stream_id":b.stream.as_str(),"source":port(&b.source),"sink":port(&b.sink),"component_id":b.component.as_str()})
}
fn stream(s: &rf_model::MaterialStreamState, n: Numbers) -> RfResult<Value> {
    let components = s.overall_mole_fractions.iter().map(|(id,x)| Ok(json!({"component_id":id.as_str(),"mole_fraction":n.value(*x)?,"flow_mol_s":n.value(*x*s.total_molar_flow_mol_s)?}))).collect::<RfResult<Vec<_>>>()?;
    let phases = s.phases.iter().map(|p| {
        let composition = p.mole_fractions.iter().map(|(id,x)| Ok((id.to_string(),n.value(*x)?))).collect::<RfResult<serde_json::Map<_,_>>>()?;
        Ok(json!({"label":p.label,"phase_fraction":n.value(p.phase_fraction)?,"composition":composition,"molar_enthalpy_j_per_mol":n.optional(p.molar_enthalpy_j_per_mol)?}))
    }).collect::<RfResult<Vec<_>>>()?;
    Ok(
        json!({"id":s.id.as_str(),"name":s.name,"temperature_k":n.value(s.temperature_k)?,"pressure_pa":n.value(s.pressure_pa)?,"total_molar_flow_mol_s":n.value(s.total_molar_flow_mol_s)?,"components":components,"phases":phases}),
    )
}
fn units(units: &[LocalUnitEvaluation], n: Numbers) -> RfResult<Vec<Value>> {
    units.iter().map(|u| Ok(json!({"unit_id":u.unit_id.as_str(),
        "consumed":u.consumed_streams.iter().map(|s|stream(s,n)).collect::<RfResult<Vec<_>>>()?,
        "produced":u.produced_streams.iter().map(|s|stream(s,n)).collect::<RfResult<Vec<_>>>()?
    }))).collect()
}
fn response(r: &AffineComponentResponse, n: Numbers) -> RfResult<Value> {
    let derivatives = r
        .outputs()
        .iter()
        .map(|o| {
            r.inputs()
                .iter()
                .map(|i| n.value(r.derivative(o, i)?))
                .collect::<RfResult<Vec<_>>>()
        })
        .collect::<RfResult<Vec<_>>>()?;
    Ok(
        json!({"unit_id":r.unit_id().as_str(),"capability":match r.capability(){ AffineComponentCapability::TwoInletMixing=>"two_inlet_mixing",AffineComponentCapability::NonselectiveSplitting=>"nonselective_splitting"},
        "temperature_k":n.value(r.temperature_k())?,"pressure_pa":n.value(r.pressure_pa())?,
        "inputs":r.inputs().iter().map(|p|json!({"port":p.port,"component_id":p.component.as_str()})).collect::<Vec<_>>(),
        "outputs":r.outputs().iter().map(|p|json!({"port":p.port,"component_id":p.component.as_str()})).collect::<Vec<_>>(),
        "derivatives_dimensionless":derivatives}),
    )
}
fn coupling(c: &CoupledEvidence, n: Numbers) -> RfResult<Value> {
    let linear = c.linear.as_ref().map(|l| Ok::<_,RfError>(json!({"dimension":l.dimension,"matrix_inf_norm":n.value(l.matrix_inf_norm)?,"pivot_threshold":n.value(l.pivot_threshold)?,"row_swaps":l.row_swaps,"rcond_indicator":n.optional(l.rcond_indicator)?,"backward_error":n.optional(l.backward_error)?}))).transpose()?;
    let connections = c.connections.iter().map(|e| Ok(json!({"binding":binding(&e.binding),"scale":{"variable_mol_s":n.value(e.scale.variable_mol_s)?,"residual_mol_s":n.value(e.scale.residual_mol_s)?},"assumed_mol_s":n.value(e.assumed_mol_s)?,"produced_mol_s":n.value(e.produced_mol_s)?,"residual_mol_s":n.value(e.residual_mol_s)?,"scaled_residual":n.value(e.scaled_residual)?}))).collect::<RfResult<Vec<_>>>()?;
    Ok(
        json!({"temperature_k":n.value(c.boundary.temperature_k)?,"pressure_pa":n.value(c.boundary.pressure_pa)?,"response_version":c.response_version,"acceptance_version":c.acceptance_version,"responses":c.responses.iter().map(|r|response(r,n)).collect::<RfResult<Vec<_>>>()?,"zero_flow_fresh_boundaries":c.zero_flow_fresh_boundaries,"anchor_residuals_mol_s":n.vector(&c.anchor_residuals_mol_s)?,"connections":connections,"linear":linear}),
    )
}
fn blocks(blocks: &[ModularBlockReport], n: Numbers) -> RfResult<Vec<Value>> {
    blocks.iter().enumerate().map(|(index,b)| Ok(json!({"index":index,"kind":if b.coupling.is_some(){"coupled"}else{"acyclic"},"members":b.members.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"final_units":units(&b.final_units,n)?,"coupling":b.coupling.as_ref().map(|c|coupling(c,n)).transpose()?}))).collect()
}
pub(crate) fn costs(c: &ModularCosts) -> Value {
    json!({"operation_constructions":c.operation_constructions,"local_unit_calls":c.local_unit_calls,"calls_by_unit":c.calls_by_unit,"thermo_calls":c.thermo_calls,"flash_calls":c.flash_calls,"response_evaluations":c.response_evaluations,"anchor_evaluations":c.anchor_evaluations,"verification_evaluations":c.verification_evaluations,"factorizations":c.factorizations,"triangular_solves":c.triangular_solves,"assembly_elapsed_s":c.assembly_elapsed.as_secs_f64(),"initialization_elapsed_s":c.initialization_elapsed.as_secs_f64(),"anchor_elapsed_s":c.anchor_elapsed.as_secs_f64(),"response_elapsed_s":c.response_elapsed.as_secs_f64(),"linear_elapsed_s":c.linear_elapsed.as_secs_f64(),"verification_elapsed_s":c.verification_elapsed.as_secs_f64(),"elapsed_s":c.elapsed.as_secs_f64()})
}
pub(crate) fn result(r: &ModularResult) -> RfResult<Value> {
    Ok(
        json!({"execution_kind":"simultaneous_modular","core_method":r.method,"streams":r.streams.values().map(|s|stream(s,Numbers::Current)).collect::<RfResult<Vec<_>>>()?,"blocks":blocks(&r.blocks,Numbers::Current)?,"costs":costs(&r.costs)}),
    )
}
pub(crate) fn failure(f: &ModularFailure) -> RfResult<Value> {
    let context = f.error.context();
    Ok(
        json!({"failure_kind":failure_kind(f.kind),"code":context.diagnostic_code(),"error_code":f.error.code().as_str(),"message":f.error.to_string(),
        "unit_ids":context.related_unit_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "stream_ids":context.related_stream_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "component_ids":context.related_component_ids().iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "ports":context.related_port_targets().iter().map(port).collect::<Vec<_>>(),
        "stage":progress(&f.progress),"completed_blocks":blocks(&f.completed_blocks,Numbers::Diagnostic)?,"active_units":units(&f.active_units,Numbers::Diagnostic)?,"active_coupling":f.active_coupling.as_ref().map(|c|coupling(c,Numbers::Diagnostic)).transpose()?,"costs":costs(&f.costs)}),
    )
}
pub(crate) fn progress(p: &ModularProgress) -> Value {
    json!({"stage":match p.stage {
        ModularStage::AssemblyStarted=>"assembly_started",ModularStage::AssemblyCompleted=>"assembly_completed",ModularStage::Initializing=>"initializing",ModularStage::BlockStarted=>"block_started",ModularStage::Anchor=>"anchor",ModularStage::UnitStarted=>"unit_started",ModularStage::UnitCompleted=>"unit_completed",ModularStage::Response=>"response",ModularStage::Factorization=>"factorization",ModularStage::ConditionSolve=>"condition_solve",ModularStage::LinearVerification=>"linear_verification",ModularStage::Verification=>"verification",ModularStage::BlockCompleted=>"block_completed",ModularStage::Completing=>"completing"
    },"block":p.block,"unit_id":p.unit.as_ref().map(|id|id.as_str()),"column":p.column})
}
fn failure_kind(k: ModularFailureKind) -> &'static str {
    match k {
        ModularFailureKind::Assembly => "assembly",
        ModularFailureKind::Initialization => "initialization",
        ModularFailureKind::LocalEvaluation => "local_evaluation",
        ModularFailureKind::Boundary => "boundary",
        ModularFailureKind::Balance => "balance",
        ModularFailureKind::Response => "response",
        ModularFailureKind::NoFiniteSteadyState => "no_finite_steady_state",
        ModularFailureKind::Singular => "singular",
        ModularFailureKind::IllConditioned => "ill_conditioned",
        ModularFailureKind::LinearVerification => "linear_verification",
        ModularFailureKind::Verification => "verification",
        ModularFailureKind::Cancelled => "cancelled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_nonfinite_numbers_never_become_missing_or_formal_values() {
        for (v, label) in [
            (f64::NAN, "nan"),
            (f64::INFINITY, "positive_infinity"),
            (f64::NEG_INFINITY, "negative_infinity"),
        ] {
            let value = Numbers::Diagnostic.optional(Some(v)).unwrap();
            assert_eq!(value, json!({"non_finite":label}));
            assert_eq!(
                serde_json::from_str::<Value>(&serde_json::to_string(&value).unwrap()).unwrap(),
                value
            );
            assert!(Numbers::Current.value(v).is_err());
        }
        assert_eq!(Numbers::Diagnostic.optional(None).unwrap(), Value::Null);
        assert_eq!(Numbers::Diagnostic.value(0.0).unwrap(), json!(0.0));
    }
}
