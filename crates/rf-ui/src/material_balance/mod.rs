//! Qualified, session-only material tasks. No project serialization or full flow snapshot.
mod editor;
mod query;
mod table;
use crate::{FlowRunQualification, WorkspaceState};
pub use editor::*;
pub use query::*;
use rf_model::Flowsheet;
use rf_solver::component_balance::*;
use std::sync::Arc;
pub use table::*;

#[derive(Debug, Clone)]
struct Identity(Arc<()>);
impl Identity {
    fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Identity {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialBalanceRunId(Identity);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialBalanceQualification {
    flow: FlowRunQualification,
    configuration: Identity,
    run_id: MaterialBalanceRunId,
}
impl MaterialBalanceQualification {
    pub fn run_id(&self) -> &MaterialBalanceRunId {
        &self.run_id
    }
    pub fn input(&self) -> &FlowRunQualification {
        &self.flow
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MaterialBalanceState {
    plan: Option<MaterialBalancePlan>,
    configuration: Identity,
    pending: Option<MaterialBalanceRunId>,
    record: Option<Arc<MaterialBalanceRecord>>,
}
impl Default for MaterialBalanceState {
    fn default() -> Self {
        Self {
            plan: None,
            configuration: Identity::fresh(),
            pending: None,
            record: None,
        }
    }
}

/// Application owns this single-use receipt while numeric input moves to a worker.
#[derive(Debug)]
pub struct MaterialBalanceReceipt {
    origin: MaterialBalanceOrigin,
    settled: bool,
}
impl MaterialBalanceReceipt {
    pub fn qualification(&self) -> &MaterialBalanceQualification {
        &self.origin.qualification
    }
    pub fn origin(&self) -> &MaterialBalanceOrigin {
        &self.origin
    }
}
#[derive(Debug)]
pub struct MaterialBalanceInput {
    flowsheet: Flowsheet,
    plan: MaterialBalancePlan,
}
impl MaterialBalanceInput {
    pub fn plan(&self) -> &MaterialBalancePlan {
        &self.plan
    }
    pub fn execute(self, control: &mut impl MaterialBalanceControl) -> MaterialBalanceOutcome {
        let problem =
            match MixerComponentBalanceProblem::assemble(&self.flowsheet, &self.plan.specification)
            {
                Ok(problem) => problem,
                Err(error) => {
                    return MaterialBalanceOutcome::Failed(Arc::new(
                        MaterialBalanceFailure::Assembly(error),
                    ));
                }
            };
        match problem.solve_controlled(&self.plan.options, control) {
            Ok(MaterialBalanceSolveOutcome::Completed(solution)) => {
                MaterialBalanceOutcome::Succeeded(Arc::from(solution))
            }
            Ok(MaterialBalanceSolveOutcome::Cancelled) => MaterialBalanceOutcome::Cancelled,
            Err(error) => {
                MaterialBalanceOutcome::Failed(Arc::new(MaterialBalanceFailure::Numerical(error)))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MaterialBalanceFailure {
    Assembly(ComponentBalanceError),
    Numerical(MaterialBalanceSolveError),
    Infrastructure(rf_types::RfError),
}
impl MaterialBalanceFailure {
    pub fn diagnostic(&self) -> rf_types::RfError {
        match self {
            Self::Assembly(error) => error.diagnostic(),
            Self::Numerical(error) => error.diagnostic(),
            Self::Infrastructure(error) => error.clone(),
        }
    }
}
impl std::fmt::Display for MaterialBalanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assembly(e) => e.fmt(f),
            Self::Numerical(e) => e.fmt(f),
            Self::Infrastructure(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for MaterialBalanceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Assembly(e) => e,
            Self::Numerical(e) => e,
            Self::Infrastructure(e) => e,
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialBalanceOutcome {
    Succeeded(Arc<MaterialBalanceSolution>),
    Failed(Arc<MaterialBalanceFailure>),
    Cancelled,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceOrigin {
    qualification: MaterialBalanceQualification,
    plan: MaterialBalancePlan,
}
impl MaterialBalanceOrigin {
    pub fn qualification(&self) -> &MaterialBalanceQualification {
        &self.qualification
    }
    pub fn plan(&self) -> &MaterialBalancePlan {
        &self.plan
    }
    pub fn implementation(&self) -> &'static str {
        self.plan.options.algorithm.implementation()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct MaterialBalanceRecord {
    origin: MaterialBalanceOrigin,
    outcome: MaterialBalanceOutcome,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceDiscardReason {
    AlreadySettled,
    DifferentSession,
    InputChanged,
    ConfigurationChanged,
    RevokedOrSuperseded,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceDisposition {
    Applied,
    Discarded(MaterialBalanceDiscardReason),
}

impl WorkspaceState {
    pub fn material_balance_plan(&self) -> Option<&MaterialBalancePlan> {
        self.material_balance.plan.as_ref()
    }
    /// A plan edit is not a document edit. A new identity prevents change-back from reviving receipts.
    pub fn set_material_balance_plan(
        &mut self,
        plan: MaterialBalancePlan,
    ) -> Result<bool, MaterialBalanceSolveError> {
        plan.validate()?;
        if self.material_balance.plan.as_ref() == Some(&plan) {
            return Ok(false);
        }
        self.material_balance.plan = Some(plan);
        self.material_balance.configuration = Identity::fresh();
        self.material_balance.pending = None;
        Ok(true)
    }
    pub fn prepare_material_balance(
        &mut self,
    ) -> Option<(MaterialBalanceInput, MaterialBalanceReceipt)> {
        let plan = self.material_balance.plan.clone()?;
        let id = MaterialBalanceRunId(Identity::fresh());
        let origin = MaterialBalanceOrigin {
            qualification: MaterialBalanceQualification {
                flow: self.flow_run_qualification(),
                configuration: self.material_balance.configuration.clone(),
                run_id: id.clone(),
            },
            plan: plan.clone(),
        };
        self.material_balance.pending = Some(id);
        self.material_balance.record = None;
        Some((
            MaterialBalanceInput {
                flowsheet: self.document.flowsheet.clone(),
                plan,
            },
            MaterialBalanceReceipt {
                origin,
                settled: false,
            },
        ))
    }
    pub fn material_balance_is_current(
        &self,
        qualification: &MaterialBalanceQualification,
    ) -> bool {
        qualification
            .flow
            .same_input(&self.flow_run_qualification())
            && qualification.configuration == self.material_balance.configuration
            && self.material_balance.pending.as_ref() == Some(&qualification.run_id)
    }
    pub fn revoke_material_balance(&mut self, id: &MaterialBalanceRunId) -> bool {
        if self.material_balance.pending.as_ref() != Some(id) {
            return false;
        }
        self.material_balance.pending = None;
        true
    }
    pub fn clear_material_balance_results(&mut self) {
        self.material_balance.pending = None;
        self.material_balance.record = None;
    }
    pub fn complete_material_balance(
        &mut self,
        receipt: &mut MaterialBalanceReceipt,
        outcome: MaterialBalanceOutcome,
    ) -> MaterialBalanceDisposition {
        use MaterialBalanceDiscardReason as R;
        if receipt.settled {
            return MaterialBalanceDisposition::Discarded(R::AlreadySettled);
        }
        receipt.settled = true;
        let q = &receipt.origin.qualification;
        let current = self.flow_run_qualification();
        let reason = if !q.flow.same_session(&current) {
            Some(R::DifferentSession)
        } else if !q.flow.same_input(&current) {
            Some(R::InputChanged)
        } else if q.configuration != self.material_balance.configuration {
            Some(R::ConfigurationChanged)
        } else if self.material_balance.pending.as_ref() != Some(&q.run_id) {
            Some(R::RevokedOrSuperseded)
        } else {
            None
        };
        if let Some(reason) = reason {
            return MaterialBalanceDisposition::Discarded(reason);
        }
        self.material_balance.pending = None;
        if matches!(outcome, MaterialBalanceOutcome::Cancelled) {
            // Once another run starts, never revive a previously accepted value on cancellation.
            self.material_balance.record = None;
            return MaterialBalanceDisposition::Discarded(R::Cancelled);
        }
        self.material_balance.record = Some(Arc::new(MaterialBalanceRecord {
            origin: receipt.origin.clone(),
            outcome,
        }));
        MaterialBalanceDisposition::Applied
    }
}

#[cfg(test)]
mod tests;
