use super::*;
use crate::result_query::ResultStatus;

#[derive(Debug)]
pub enum MaterialBalanceResultView<'a> {
    Current {
        solution: &'a MaterialBalanceSolution,
        origin: &'a MaterialBalanceOrigin,
    },
    Failed {
        failure: &'a MaterialBalanceFailure,
        origin: &'a MaterialBalanceOrigin,
    },
    Stale {
        origin: &'a MaterialBalanceOrigin,
    },
    Missing,
}
impl MaterialBalanceResultView<'_> {
    pub fn status(&self) -> ResultStatus {
        match self {
            Self::Current { .. } => ResultStatus::Current,
            Self::Failed { .. } => ResultStatus::Failed,
            Self::Stale { .. } => ResultStatus::Stale,
            Self::Missing => ResultStatus::Missing,
        }
    }
}
impl WorkspaceState {
    pub fn material_balance_result(&self) -> MaterialBalanceResultView<'_> {
        // Pending input owns the next result, so old success is never this attempt's result.
        if self.material_balance.pending.is_some() {
            return MaterialBalanceResultView::Missing;
        }
        let Some(record) = &self.material_balance.record else {
            return MaterialBalanceResultView::Missing;
        };
        let origin = &record.origin;
        if !origin
            .qualification
            .flow
            .same_session(&self.flow_run_qualification())
        {
            return MaterialBalanceResultView::Missing;
        }
        if !origin
            .qualification
            .flow
            .same_input(&self.flow_run_qualification())
            || origin.qualification.configuration != self.material_balance.configuration
        {
            return MaterialBalanceResultView::Stale { origin };
        }
        match &record.outcome {
            MaterialBalanceOutcome::Succeeded(solution) => {
                MaterialBalanceResultView::Current { solution, origin }
            }
            MaterialBalanceOutcome::Failed(failure) => {
                MaterialBalanceResultView::Failed { failure, origin }
            }
            MaterialBalanceOutcome::Cancelled => MaterialBalanceResultView::Missing,
        }
    }
}

/// Output/navigation consumers revalidate the same accepted execution immediately before use.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceResultGuard(MaterialBalanceOrigin);
impl MaterialBalanceResultGuard {
    pub fn capture(workspace: &WorkspaceState) -> Option<Self> {
        match workspace.material_balance_result() {
            MaterialBalanceResultView::Current { origin, .. } => Some(Self(origin.clone())),
            _ => None,
        }
    }
    pub fn read<'a>(&self, workspace: &'a WorkspaceState) -> Option<&'a MaterialBalanceSolution> {
        match workspace.material_balance_result() {
            MaterialBalanceResultView::Current { solution, origin } if origin == &self.0 => {
                Some(solution)
            }
            _ => None,
        }
    }
}
