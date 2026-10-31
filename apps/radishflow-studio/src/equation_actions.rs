//! Qualified, read-only EO structure inspection and explicit candidate adoption.
use crate::FlowResultGuard;
use crate::flow_actions::{
    FlowAction, FlowActionReceipt, FlowActionRequest, FlowEditAccess, FlowEditContext,
    execute_flow_action, preflight_flow_action,
};
use rf_solver::equation_oriented::{
    EquationDefinition, EquationOptions, ParameterBinding, SolvedSplitFraction, assemble,
};
use rf_types::{RfError, RfResult};
use rf_ui::flow_plan::{FlowPlanConfiguration, FlowPlanEditContext};
use rf_ui::result_query::FlowResultView;
use rf_ui::variable_browser::{
    ObjectId, VariableField, VariableId, VariableSection, VariableValue,
};
use rf_ui::{AppState, DateTimeUtc, FlowResultRecord, WorkspaceState};

#[derive(Debug, Clone, PartialEq)]
pub struct EquationStructurePreview {
    input: FlowEditContext,
    plan: FlowPlanEditContext,
    pub definition: EquationDefinition,
}
impl EquationStructurePreview {
    pub fn is_current(&self, workspace: &WorkspaceState) -> bool {
        self.input.validate(workspace).is_ok() && self.plan.is_current(workspace)
    }
}
pub fn inspect_equation_structure(
    workspace: &WorkspaceState,
) -> RfResult<EquationStructurePreview> {
    let FlowPlanConfiguration::EquationOriented(options) = workspace.flow_plan.current() else {
        return Err(RfError::invalid_input(
            "the applied plan is not equation-oriented",
        ));
    };
    let problem = assemble(&workspace.document.flowsheet, options).map_err(|e| e.error)?;
    Ok(EquationStructurePreview {
        input: FlowEditContext::capture(workspace),
        plan: FlowPlanEditContext::capture(workspace),
        definition: EquationDefinition::from(&problem),
    })
}

/// Private candidate and execution guards prevent caller-supplied numeric writes.
#[derive(Debug, Clone, PartialEq)]
pub struct EquationAdoptionPreview {
    input: FlowEditContext,
    plan: FlowPlanEditContext,
    result: FlowResultGuard,
    candidate: SolvedSplitFraction,
}
impl EquationAdoptionPreview {
    pub fn binding(&self) -> &ParameterBinding {
        &self.candidate.binding
    }
    pub fn stored_value(&self) -> Option<f64> {
        self.candidate.stored_value
    }
    pub fn solved_value(&self) -> f64 {
        self.candidate.value
    }
    pub fn input_revision(&self) -> u64 {
        self.input.revision()
    }
}
fn eligible(app: &AppState, access: FlowEditAccess, worker_busy: bool) -> RfResult<()> {
    let w = &app.workspace;
    if access != FlowEditAccess::Editable {
        return Err(RfError::invalid_input("project is read-only"));
    }
    if worker_busy || w.pending_flow_run().is_some() {
        return Err(RfError::invalid_input(
            "wait for the running, cancelling or ready worker to settle",
        ));
    }
    if w.drafts.pending_count() > 0
        || w.flow_plan.pending_count() > 0
        || w.canvas_interaction.pending_edit.is_some()
    {
        return Err(RfError::invalid_input(
            "resolve input, plan and canvas drafts before adopting",
        ));
    }
    Ok(())
}
fn candidate(
    app: &AppState,
) -> RfResult<(&SolvedSplitFraction, &EquationOptions, FlowResultGuard)> {
    let w = &app.workspace;
    let FlowResultView::Current {
        snapshot: FlowResultRecord::EquationOriented(s),
        ..
    } = w.flow_result()
    else {
        return Err(RfError::invalid_input(
            "adoption requires the Current EO result",
        ));
    };
    let FlowPlanConfiguration::EquationOriented(o) = w.flow_plan.current() else {
        return Err(RfError::invalid_input("applied plan is not EO"));
    };
    let solved =
        s.result.evidence.solved_parameter.as_ref().ok_or_else(|| {
            RfError::invalid_input("forward EO results have no released parameter")
        })?;
    if o.released
        .as_ref()
        .is_none_or(|r| r.binding != solved.binding)
        || !solved.value.is_finite()
        || !(0.0..=1.0).contains(&solved.value)
    {
        return Err(RfError::invalid_input(
            "result and released parameter do not match",
        ));
    }
    let guard = FlowResultGuard::from_snapshot(&FlowResultRecord::EquationOriented(s.clone()))
        .ok_or_else(|| RfError::invalid_input("execution identity is missing"))?;
    Ok((solved, o, guard))
}
fn request(app: &AppState, preview: &EquationAdoptionPreview) -> FlowActionRequest {
    FlowActionRequest {
        expected: preview.input.clone(),
        action: FlowAction::Write {
            variable: VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object: ObjectId::Unit(preview.candidate.binding.unit.clone()),
                section: VariableSection::Inputs,
                field: VariableField::OutletASplitFraction,
            },
            value: VariableValue::Number(preview.candidate.value),
        },
    }
}
pub fn preview_equation_adoption(
    app: &AppState,
    access: FlowEditAccess,
    worker_busy: bool,
) -> RfResult<EquationAdoptionPreview> {
    eligible(app, access, worker_busy)?;
    let (solved, _, result) = candidate(app)?;
    let preview = EquationAdoptionPreview {
        input: FlowEditContext::capture(&app.workspace),
        plan: FlowPlanEditContext::capture(&app.workspace),
        result,
        candidate: solved.clone(),
    };
    preflight_flow_action(app, access, &request(app, &preview))
        .map_err(|e| RfError::invalid_input(e.to_string()))?;
    Ok(preview)
}
pub fn adopt_equation_candidate(
    app: &mut AppState,
    access: FlowEditAccess,
    worker_busy: bool,
    preview: &EquationAdoptionPreview,
    changed_at: DateTimeUtc,
) -> RfResult<FlowActionReceipt> {
    eligible(app, access, worker_busy)?;
    if preview.input.validate(&app.workspace).is_err()
        || !preview.plan.is_current(&app.workspace)
        || !preview.result.is_current(&app.workspace)
    {
        return Err(RfError::invalid_input(
            "adoption preview is stale; inspect the Current result again",
        ));
    }
    let (solved, _, guard) = candidate(app)?;
    if solved != &preview.candidate || guard != preview.result {
        return Err(RfError::invalid_input("adoption execution changed"));
    }
    let receipt = execute_flow_action(app, access, &request(app, preview), changed_at)
        .map_err(|e| RfError::invalid_input(e.to_string()))?;
    if receipt.invalidates_flow_results() {
        app.workspace.solve_session.pending_reason =
            Some(rf_ui::SolvePendingReason::CandidateAdopted);
        app.refresh_run_panel_state();
    }
    Ok(receipt)
}
