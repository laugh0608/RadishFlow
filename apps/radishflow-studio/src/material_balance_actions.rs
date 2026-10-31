//! Formal material task actions shared by desktop and headless consumers.
use crate::{
    StudioAppFacade, StudioRunId, StudioRunOutcome, StudioWorkspaceRunBlocked,
    flow_actions::{FlowEditConflict, FlowEditContext},
    run_actions::{StudioRunAccess, StudioRunCompletion, StudioRunReply},
};
use rf_solver::component_balance::{
    MaterialBalanceControl, MaterialBalancePlan, MaterialBalanceProgress, MaterialBalanceSolveError,
};
use rf_ui::{
    AppState,
    material_balance::{MaterialBalanceDisposition, MaterialBalanceOutcome},
};
use std::ops::ControlFlow;

#[derive(Debug)]
pub enum MaterialBalanceActionError {
    ReadOnly,
    Context(FlowEditConflict),
    PendingDrafts(StudioWorkspaceRunBlocked),
    MissingPlan,
    InvalidPlan(MaterialBalanceSolveError),
}
impl MaterialBalanceActionError {
    pub fn diagnostic(&self) -> rf_types::RfError {
        let error = rf_types::RfError::invalid_input(self.to_string());
        match self {
            Self::InvalidPlan(e) => e.diagnostic(),
            Self::ReadOnly => error.with_diagnostic_code("material.read_only"),
            Self::Context(_) => error.with_diagnostic_code("material.input_context_changed"),
            Self::PendingDrafts(_) => error.with_diagnostic_code("material.pending_drafts"),
            Self::MissingPlan => error.with_diagnostic_code("material.missing_plan"),
        }
    }
}
impl std::fmt::Display for MaterialBalanceActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => f.write_str("read-only projects cannot calculate material balances"),
            Self::Context(error) => write!(f, "material balance input context changed: {error:?}"),
            Self::PendingDrafts(blocked) => f.write_str(&blocked.message),
            Self::MissingPlan => f.write_str("apply material balance settings before calculating"),
            Self::InvalidPlan(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for MaterialBalanceActionError {}

pub(crate) fn material_preflight(app: &AppState) -> Result<(), MaterialBalanceActionError> {
    if let Some(blocked) = crate::app_facade::workspace_pending_inspector_draft_block(app) {
        return Err(MaterialBalanceActionError::PendingDrafts(blocked));
    }
    let plan = app
        .workspace
        .material_balance_plan()
        .ok_or(MaterialBalanceActionError::MissingPlan)?;
    plan.validate()
        .map_err(MaterialBalanceActionError::InvalidPlan)
}
struct InlineControl;
impl MaterialBalanceControl for InlineControl {
    fn checkpoint(&mut self, _: MaterialBalanceProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl StudioAppFacade {
    pub fn configure_material_balance(
        &self,
        app: &mut AppState,
        access: StudioRunAccess,
        expected: &FlowEditContext,
        plan: MaterialBalancePlan,
    ) -> Result<bool, MaterialBalanceActionError> {
        if access == StudioRunAccess::ReadOnly {
            return Err(MaterialBalanceActionError::ReadOnly);
        }
        expected
            .validate(&app.workspace)
            .map_err(MaterialBalanceActionError::Context)?;
        app.workspace
            .set_material_balance_plan(plan)
            .map_err(MaterialBalanceActionError::InvalidPlan)
    }
    pub fn execute_material_balance(
        &self,
        app: &mut AppState,
        access: StudioRunAccess,
        expected: &FlowEditContext,
    ) -> Result<StudioRunReply, MaterialBalanceActionError> {
        if access == StudioRunAccess::ReadOnly {
            return Err(MaterialBalanceActionError::ReadOnly);
        }
        expected
            .validate(&app.workspace)
            .map_err(MaterialBalanceActionError::Context)?;
        material_preflight(app)?;
        if let Some(background) = &self.background {
            return background
                .owner
                .borrow_mut()
                .submit_material_balance(app, std::time::SystemTime::now());
        }
        let (input, mut receipt) = app
            .workspace
            .prepare_material_balance()
            .expect("preflight validated plan");
        let id = StudioRunId::Material(receipt.qualification().run_id().clone());
        let result = input.execute(&mut InlineControl);
        let outcome = material_outcome(&result);
        let diagnostic_code = material_diagnostic_code(&result);
        let applied = app
            .workspace
            .complete_material_balance(&mut receipt, result)
            == MaterialBalanceDisposition::Applied;
        Ok(StudioRunReply::Completed(StudioRunCompletion {
            id,
            outcome,
            applied,
            cancellation: None,
            diagnostic_code,
        }))
    }
}
pub(crate) fn material_outcome(result: &MaterialBalanceOutcome) -> StudioRunOutcome {
    use rf_ui::material_balance::MaterialBalanceFailure;
    match result {
        MaterialBalanceOutcome::Succeeded(_) => StudioRunOutcome::Succeeded,
        MaterialBalanceOutcome::Cancelled => StudioRunOutcome::Cancelled,
        MaterialBalanceOutcome::Failed(error)
            if matches!(error.as_ref(), MaterialBalanceFailure::Infrastructure(_)) =>
        {
            StudioRunOutcome::InfrastructureFailed
        }
        MaterialBalanceOutcome::Failed(_) => StudioRunOutcome::Failed,
    }
}
pub(crate) fn material_diagnostic_code(result: &MaterialBalanceOutcome) -> Option<String> {
    match result {
        MaterialBalanceOutcome::Failed(error) => Some(
            error
                .diagnostic()
                .context()
                .diagnostic_code()
                .unwrap_or("material.infrastructure_failed")
                .into(),
        ),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use rf_solver::component_balance::{MixerBalanceScales, MixerBalanceSpecification};
    use rf_ui::{DocumentMetadata, FlowsheetDocument, material_balance::MaterialBalanceResultView};
    use std::time::UNIX_EPOCH;
    pub(crate) fn app() -> AppState {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
        ))
        .unwrap();
        let mut flow = project.document.flowsheet;
        flow.units.remove(&"flash-1".into());
        flow.streams.remove(&"stream-liquid".into());
        flow.streams.remove(&"stream-vapor".into());
        flow.streams
            .get_mut(&"stream-feed-a".into())
            .unwrap()
            .overall_mole_fractions
            .insert("methane".into(), 0.8);
        flow.streams
            .get_mut(&"stream-feed-a".into())
            .unwrap()
            .overall_mole_fractions
            .insert("ethane".into(), 0.2);
        let mut app = AppState::new(FlowsheetDocument::new(
            flow,
            DocumentMetadata::new("material-doc", "Material", UNIX_EPOCH),
        ));
        app.workspace
            .set_material_balance_plan(MaterialBalancePlan::new(MixerBalanceSpecification::new(
                "mixer-1",
                MixerBalanceScales {
                    inlet_a_mol_s: 2.0,
                    inlet_b_mol_s: 3.0,
                    outlet_mol_s: 5.0,
                },
            )))
            .unwrap();
        app
    }
    #[test]
    fn inline_formal_action_uses_frozen_plan_and_reports_independent_material_result() {
        let mut app = app();
        let before = app.workspace.document.clone();
        let context = FlowEditContext::capture(&app.workspace);
        let reply = StudioAppFacade::new()
            .execute_material_balance(&mut app, StudioRunAccess::Allowed, &context)
            .unwrap();
        assert!(matches!(
            reply,
            StudioRunReply::Completed(StudioRunCompletion {
                id: StudioRunId::Material(_),
                outcome: StudioRunOutcome::Succeeded,
                applied: true,
                ..
            })
        ));
        let MaterialBalanceResultView::Current { solution, .. } =
            app.workspace.material_balance_result()
        else {
            panic!("current")
        };
        assert!((solution.values.mole_fractions[&"methane".into()] - 0.44).abs() < 1e-12);
        assert_eq!(app.workspace.document, before);
        assert!(app.workspace.snapshot_history.is_empty());
    }
    #[test]
    fn formal_actions_recheck_read_only_and_input_context_without_mutation() {
        let mut app = app();
        let context = FlowEditContext::capture(&app.workspace);
        let before = app.clone();
        let facade = StudioAppFacade::new();
        assert!(matches!(
            facade.execute_material_balance(&mut app, StudioRunAccess::ReadOnly, &context),
            Err(MaterialBalanceActionError::ReadOnly)
        ));
        assert_eq!(app, before);
        let mut independent = self::app();
        assert!(matches!(
            facade.execute_material_balance(&mut independent, StudioRunAccess::Allowed, &context),
            Err(MaterialBalanceActionError::Context(_))
        ));
    }
    #[test]
    fn unsupported_problem_publishes_current_failure_with_domain_diagnostics() {
        let mut app = app();
        app.workspace
            .document
            .flowsheet
            .units
            .get_mut(&"feed-a".into())
            .unwrap()
            .kind = "valve".into();
        let context = FlowEditContext::capture(&app.workspace);
        let reply = StudioAppFacade::new()
            .execute_material_balance(&mut app, StudioRunAccess::Allowed, &context)
            .unwrap();
        assert!(
            matches!(reply,StudioRunReply::Completed(StudioRunCompletion {outcome:StudioRunOutcome::Failed,applied:true,diagnostic_code:Some(code),..}) if code=="material.unsupported_model")
        );
        let MaterialBalanceResultView::Failed { failure, .. } =
            app.workspace.material_balance_result()
        else {
            panic!("failed")
        };
        assert_eq!(
            failure.diagnostic().context().related_unit_ids(),
            &["feed-a".into()]
        );
    }
}

#[cfg(test)]
mod draft_tests {
    use super::*;
    #[test]
    fn pending_input_draft_blocks_material_action_instead_of_computing_older_formal_input() {
        let mut app = super::tests::app();
        let mut draft = rf_ui::FieldDraft::new("1".to_string());
        draft.update("2".into());
        draft.is_dirty = true;
        app.workspace
            .drafts
            .fields
            .insert("test.pending".into(), rf_ui::DraftValue::Text(draft));
        let expected = FlowEditContext::capture(&app.workspace);
        let before = app.clone();
        assert!(matches!(
            StudioAppFacade::new().execute_material_balance(
                &mut app,
                StudioRunAccess::Allowed,
                &expected
            ),
            Err(MaterialBalanceActionError::PendingDrafts(_))
        ));
        assert_eq!(app, before);
    }
}
