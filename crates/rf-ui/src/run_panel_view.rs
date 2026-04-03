use crate::run::{RunStatus, SimulationMode, SolvePendingReason};
use crate::run_panel::{RunPanelActionId, RunPanelIntent, RunPanelNotice, RunPanelState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPanelActionProminence {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelRenderableAction {
    pub id: RunPanelActionId,
    pub label: &'static str,
    pub intent: RunPanelIntent,
    pub enabled: bool,
    pub prominence: RunPanelActionProminence,
}

impl RunPanelRenderableAction {
    pub fn dispatchable_intent(&self) -> Option<RunPanelIntent> {
        self.enabled.then(|| self.intent.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelViewModel {
    pub mode_label: &'static str,
    pub status_label: &'static str,
    pub pending_label: Option<&'static str>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub latest_log_message: Option<String>,
    pub notice: Option<RunPanelNotice>,
    pub primary_action: RunPanelRenderableAction,
    pub secondary_actions: Vec<RunPanelRenderableAction>,
}

impl RunPanelViewModel {
    pub fn from_state(state: &RunPanelState) -> Self {
        let visible_actions = state
            .commands
            .actions
            .iter()
            .filter(|action| action.visible)
            .collect::<Vec<_>>();
        let primary_action = visible_actions
            .iter()
            .find(|action| action.id == state.commands.primary_action)
            .copied()
            .or_else(|| visible_actions.first().copied())
            .expect("run panel command model must expose at least one visible action");

        Self {
            mode_label: simulation_mode_label(state.simulation_mode),
            status_label: run_status_label(state.run_status),
            pending_label: state.pending_reason.map(solve_pending_reason_label),
            latest_snapshot_id: state.latest_snapshot_id.clone(),
            latest_snapshot_summary: state.latest_snapshot_summary.clone(),
            latest_log_message: state.latest_log_message.clone(),
            notice: state.notice.clone(),
            primary_action: RunPanelRenderableAction {
                id: primary_action.id,
                label: primary_action.label,
                intent: primary_action.intent.clone(),
                enabled: primary_action.enabled,
                prominence: RunPanelActionProminence::Primary,
            },
            secondary_actions: visible_actions
                .into_iter()
                .filter(|action| action.id != primary_action.id)
                .map(|action| RunPanelRenderableAction {
                    id: action.id,
                    label: action.label,
                    intent: action.intent.clone(),
                    enabled: action.enabled,
                    prominence: RunPanelActionProminence::Secondary,
                })
                .collect(),
        }
    }

    pub fn action(&self, id: RunPanelActionId) -> Option<&RunPanelRenderableAction> {
        if self.primary_action.id == id {
            return Some(&self.primary_action);
        }

        self.secondary_actions.iter().find(|action| action.id == id)
    }

    pub fn dispatchable_intent(&self, id: RunPanelActionId) -> Option<RunPanelIntent> {
        self.action(id)
            .and_then(RunPanelRenderableAction::dispatchable_intent)
    }

    pub fn dispatchable_primary_intent(&self) -> Option<RunPanelIntent> {
        self.primary_action.dispatchable_intent()
    }
}

fn simulation_mode_label(mode: SimulationMode) -> &'static str {
    match mode {
        SimulationMode::Active => "Active",
        SimulationMode::Hold => "Hold",
    }
}

fn run_status_label(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Idle => "Idle",
        RunStatus::Dirty => "Dirty",
        RunStatus::Checking => "Checking",
        RunStatus::Runnable => "Runnable",
        RunStatus::Solving => "Solving",
        RunStatus::Converged => "Converged",
        RunStatus::UnderSpecified => "Under-specified",
        RunStatus::OverSpecified => "Over-specified",
        RunStatus::Unconverged => "Unconverged",
        RunStatus::Error => "Error",
    }
}

fn solve_pending_reason_label(reason: SolvePendingReason) -> &'static str {
    match reason {
        SolvePendingReason::DocumentRevisionAdvanced => "Document changed",
        SolvePendingReason::ModeActivated => "Mode activated",
        SolvePendingReason::ManualRunRequested => "Manual run requested",
        SolvePendingReason::SnapshotMissing => "Snapshot missing",
    }
}
