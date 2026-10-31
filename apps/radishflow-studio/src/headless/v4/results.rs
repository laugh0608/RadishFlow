use super::*;
use rf_solver::partitioned::*;
use rf_ui::result_query::FlowResultView;
pub fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
    // Reject before the common projection could expose new-strategy values.
    if matches!(
        app.workspace.flow_plan.current(),
        rf_ui::flow_plan::FlowPlanConfiguration::Simultaneous(_)
    ) {
        return Err(Failure::new(
            Stage::Query,
            Code::UnsupportedCapability,
            "CLI v4 does not support simultaneous modular results",
        ));
    }
    let mut value = old::results::flow(app, executions);
    match app.workspace.flow_result() {
        FlowResultView::Current { snapshot, origin } => {
            match snapshot {
                rf_ui::FlowResultRecord::Simultaneous(_) => {
                    return Err(Failure::new(
                        Stage::Query,
                        Code::UnsupportedCapability,
                        "CLI v4 does not support simultaneous modular results",
                    ));
                }
                rf_ui::FlowResultRecord::Acyclic(_) => {
                    value["execution_kind"] = json!("acyclic_sm");
                }
                rf_ui::FlowResultRecord::Partitioned(p) => {
                    value["execution_kind"] = json!("partitioned_sm");
                    value["streams"] =
                        json!(p.result.streams.values().map(stream).collect::<Vec<_>>());
                    value["blocks"] = json!(
                        p.result
                            .blocks
                            .iter()
                            .enumerate()
                            .map(|(i, b)| block(i, b))
                            .collect::<Vec<_>>()
                    );
                    value["costs"] = costs(&p.result.costs);
                }
            }
            evidence(&mut value, origin);
        }
        FlowResultView::Failed { origin, .. } => {
            evidence(&mut value, origin);
            if let Some(f) = app.workspace.partitioned_failure_evidence() {
                value["diagnostic_evidence"] = json!({"failure_kind":failure_kind(f.kind),"stage":stage(&f.stage),"costs":costs(&f.costs),"completed_blocks":f.completed_blocks.iter().enumerate().map(|(i,b)|block(i,b)).collect::<Vec<_>>(),"active_units":units(&f.active_units),"active_recycle":f.active_recycle.as_ref().map(recycle)});
            }
        }
        _ => {}
    }
    Ok(value)
}
fn evidence(v: &mut Value, origin: &rf_ui::result_query::FlowResultOrigin) {
    if let Some(e) = origin.execution().and_then(|e| e.plan.as_ref()) {
        v["origin"]["execution"]["plan"] = json!({"method":e.method,"acceptance_contract_version":e.acceptance_contract_version,"configuration_sha256":e.configuration_sha256});
    }
}
fn stream(s: &rf_model::MaterialStreamState) -> Value {
    json!({"id":s.id.as_str(),"name":s.name,"temperature_k":s.temperature_k,"pressure_pa":s.pressure_pa,"total_molar_flow_mol_s":s.total_molar_flow_mol_s,"components":s.overall_mole_fractions.iter().map(|(id,x)|json!({"component_id":id.as_str(),"mole_fraction":x,"flow_mol_s":x*s.total_molar_flow_mol_s})).collect::<Vec<_>>(),"phases":s.phases.iter().map(|p|json!({"label":p.label,"phase_fraction":p.phase_fraction,"composition":p.mole_fractions,"molar_enthalpy_j_per_mol":p.molar_enthalpy_j_per_mol})).collect::<Vec<_>>()})
}
fn units(values: &[LocalUnitEvaluation]) -> Vec<Value> {
    values.iter().map(|u|json!({"unit_id":u.unit_id.as_str(),"consumed":u.consumed_streams.iter().map(stream).collect::<Vec<_>>(),"produced":u.produced_streams.iter().map(stream).collect::<Vec<_>>()})).collect()
}
fn block(i: usize, b: &BlockReport) -> Value {
    json!({"index":i,"kind":match b.kind {BlockKind::Acyclic=>"acyclic",BlockKind::Recycle=>"recycle"},"members":b.members.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"final_units":units(&b.final_units),"recycle":b.recycle.as_ref().map(recycle)})
}
fn sample(v: &ResidualSample) -> Value {
    json!({"phase":match v.phase {EvaluationPhase::Iteration=>"iteration",EvaluationPhase::Verification=>"verification"},"evaluation":v.evaluation,"max_absolute_mol_s":v.max_absolute_mol_s,"max_scaled":v.max_scaled})
}
fn recycle(r: &RecycleBlockReport) -> Value {
    json!({"temperature_k":r.boundary.temperature_k,"pressure_pa":r.boundary.pressure_pa,"tear":{"stream_id":r.tear.stream_id.as_str(),"source":{"unit_id":r.tear.source.unit_id.as_str(),"port":r.tear.source.port_name},"sink":{"unit_id":r.tear.sink.unit_id.as_str(),"port":r.tear.sink.port_name}},"iteration_evaluations":r.iteration_evaluations,"verification_evaluations":r.verification_evaluations,"component_scales_mol_s":r.component_scales_mol_s,"assumed":stream(&r.assumed),"returned":stream(&r.returned),"residuals_mol_s":r.residuals_mol_s,"residual_trace":r.residual_trace.iter().map(sample).collect::<Vec<_>>(),"tolerances":{"component_absolute_flow_mol_s":r.tolerances.component_absolute_flow_mol_s,"scaled_flow":r.tolerances.scaled_flow,"total_absolute_flow_mol_s":r.tolerances.total_absolute_flow_mol_s,"temperature_k":r.tolerances.temperature_k,"pressure_pa":r.tolerances.pressure_pa},"zero_flow_composition_rule":match &r.zero_flow_composition_rule {ZeroFlowCompositionRule::EvaluatedFreshBoundary(id)=>json!({"kind":"evaluated_fresh_boundary","stream_id":id.as_str()})}})
}
fn costs(c: &SolveCosts) -> Value {
    json!({"operation_constructions":c.operation_constructions,"local_unit_calls":c.local_unit_calls,"calls_by_unit":c.calls_by_unit,"thermo_calls":c.thermo_calls,"flash_calls":c.flash_calls,"iteration_evaluations":c.iteration_evaluations,"verification_evaluations":c.verification_evaluations,"assembly_elapsed_s":c.assembly_elapsed.as_secs_f64(),"initialization_elapsed_s":c.initialization_elapsed.as_secs_f64(),"elapsed_s":c.elapsed.as_secs_f64()})
}
fn failure_kind(k: PartitionedFailureKind) -> &'static str {
    match k {
        PartitionedFailureKind::Assembly => "assembly",
        PartitionedFailureKind::Initialization => "initialization",
        PartitionedFailureKind::LocalEvaluation => "local_evaluation",
        PartitionedFailureKind::Boundary => "boundary",
        PartitionedFailureKind::Balance => "balance",
        PartitionedFailureKind::NoFiniteSteadyState => "no_finite_steady_state",
        PartitionedFailureKind::IterationLimit => "iteration_limit",
        PartitionedFailureKind::Verification => "verification",
        PartitionedFailureKind::Cancelled => "cancelled",
    }
}
fn stage(p: &PartitionedProgress) -> Value {
    json!({"stage":match p.stage {PartitionedStage::AssemblyStarted=>"assembly_started",PartitionedStage::AssemblyCompleted=>"assembly_completed",PartitionedStage::Initializing=>"initializing",PartitionedStage::BlockStarted=>"block_started",PartitionedStage::IterationStarted=>"iteration_started",PartitionedStage::IterationCompleted=>"iteration_completed",PartitionedStage::VerificationStarted=>"verification_started",PartitionedStage::VerificationCompleted=>"verification_completed",PartitionedStage::UnitStarted=>"unit_started",PartitionedStage::UnitCompleted=>"unit_completed",PartitionedStage::Completing=>"completing"},"block":p.block,"unit_id":p.unit.as_ref().map(|id|id.as_str()),"evaluation":p.evaluation,"residual":p.residual.as_ref().map(sample)})
}
