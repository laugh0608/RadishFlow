//! Desktop application lifetime, one numeric worker across flowsheet and equipment.
//! Rc/RefCell is confined to the UI thread. Workers receive frozen values only.
mod equipment;
mod material;
mod numeric_worker;
use crate::{
    PropertyPackageCacheRoot, WorkspaceExecutionOutcome, WorkspaceRunCommand, WorkspaceRunOwner,
    WorkspaceRunPhase, WorkspaceRunSubmission,
};
use equipment::EquipmentRun;
use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::engineering::CalculationId;
use rf_ui::engineering::editor::EquipmentEditorCommand;
use rf_ui::{AppState, FlowRunId, FlowRunQualification, SimulationMode};
use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::SystemTime};

pub type SharedStudioRunOwner = Rc<RefCell<StudioRunOwner>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunId {
    Flow(FlowRunId),
    Equipment(CalculationId),
    Material(rf_ui::material_balance::MaterialBalanceRunId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunSubmission {
    Flow(WorkspaceRunSubmission),
    Accepted(StudioRunId),
    AlreadyRunning(StudioRunId),
    Busy(StudioRunId),
    Closing,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioRunOutcome {
    Succeeded,
    Failed,
    Cancelled,
    InfrastructureFailed,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRunActivity {
    pub id: StudioRunId,
    pub number: u64,
    pub target: String,
    pub phase: WorkspaceRunPhase,
    pub submitted_at: SystemTime,
    pub cancel_requested_at: Option<SystemTime>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRunSummary {
    pub activity: StudioRunActivity,
    pub outcome: StudioRunOutcome,
    pub applied: bool,
    pub finished_at: SystemTime,
    pub cancellation: Option<crate::WorkspaceRunCancelReason>,
    pub diagnostic_code: Option<String>,
}

struct StudioRunTerminal {
    outcome: StudioRunOutcome,
    applied: bool,
    cancellation: Option<crate::WorkspaceRunCancelReason>,
    diagnostic_code: Option<String>,
}

/// Retain across document replacement and poll until idle before application exit.
/// This owner never blocks in Drop and cannot forcibly stop a numeric call.
#[must_use]
pub struct StudioRunOwner {
    flow: WorkspaceRunOwner,
    equipment: Option<EquipmentRun>,
    material: Option<material::MaterialRun>,
    activity: Option<StudioRunActivity>,
    history: VecDeque<StudioRunSummary>,
    next_number: u64,
    accepting: bool,
    last_auto_attempt: Option<(FlowRunQualification, Option<rf_ui::SolvePendingReason>)>,
}
impl std::fmt::Debug for StudioRunOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StudioRunOwner")
            .field("activity", &self.activity)
            .finish_non_exhaustive()
    }
}
impl Default for StudioRunOwner {
    fn default() -> Self {
        Self {
            flow: WorkspaceRunOwner::new(),
            equipment: None,
            material: None,
            activity: None,
            history: VecDeque::new(),
            next_number: 0,
            accepting: true,
            last_auto_attempt: None,
        }
    }
}
impl StudioRunOwner {
    pub fn is_idle(&self) -> bool {
        self.flow.is_idle() && self.equipment.is_none() && self.material.is_none()
    }
    pub fn activity(&self) -> Option<StudioRunActivity> {
        let mut activity = self.activity.clone()?;
        activity.phase = if let Some(run) = &self.material {
            if run.cancel_reason.is_some() {
                WorkspaceRunPhase::CancelRequested
            } else if run.started() {
                WorkspaceRunPhase::Running
            } else {
                WorkspaceRunPhase::Preparing
            }
        } else if let Some(run) = &self.equipment {
            if run.cancel_reason.is_some() {
                WorkspaceRunPhase::CancelRequested
            } else if run.started {
                WorkspaceRunPhase::Running
            } else {
                WorkspaceRunPhase::Preparing
            }
        } else {
            self.flow.state()?.phase
        };
        Some(activity)
    }
    pub fn history(&self) -> &VecDeque<StudioRunSummary> {
        &self.history
    }
    fn set_activity(&mut self, id: StudioRunId, target: String, now: SystemTime) {
        self.next_number = self
            .next_number
            .checked_add(1)
            .expect("run display counter exhausted");
        self.activity = Some(StudioRunActivity {
            id,
            number: self.next_number,
            target,
            phase: WorkspaceRunPhase::Preparing,
            submitted_at: now,
            cancel_requested_at: None,
        });
    }
    pub fn submit_flow(
        &mut self,
        app: &mut AppState,
        root: PropertyPackageCacheRoot,
        index: &StoredAuthCacheIndex,
        command: &WorkspaceRunCommand,
        now: SystemTime,
    ) -> RfResult<StudioRunSubmission> {
        if !self.accepting {
            return Ok(StudioRunSubmission::Closing);
        }
        if self.equipment.is_some() || self.material.is_some() {
            match crate::app_facade::preflight_workspace_run(app, index, command) {
                crate::app_facade::WorkspaceRunPreflight::Skipped(reason) => {
                    return Ok(StudioRunSubmission::Flow(WorkspaceRunSubmission::Skipped(
                        reason,
                    )));
                }
                crate::app_facade::WorkspaceRunPreflight::Blocked { blocked, .. } => {
                    return Ok(StudioRunSubmission::Flow(WorkspaceRunSubmission::Blocked(
                        crate::WorkspaceRunBlock::Preflight(blocked),
                    )));
                }
                crate::app_facade::WorkspaceRunPreflight::Ready(_) => {}
            }
            return Ok(StudioRunSubmission::Busy(
                self.activity.as_ref().expect("occupied slot").id.clone(),
            ));
        }
        let submission = self.flow.submit_cached(app, root, index, command, now)?;
        if let WorkspaceRunSubmission::Accepted(id) = &submission {
            self.set_activity(
                StudioRunId::Flow(id.clone()),
                app.workspace.document.metadata.title.clone(),
                now,
            );
        }
        if !matches!(submission, WorkspaceRunSubmission::Busy(_)) {
            self.last_auto_attempt = Some((
                app.flow_run_qualification(),
                app.workspace.solve_session.pending_reason,
            ));
        }
        Ok(StudioRunSubmission::Flow(submission))
    }
    pub fn submit_equipment(
        &mut self,
        app: &mut AppState,
        command: &EquipmentEditorCommand,
        now: SystemTime,
    ) -> RfResult<StudioRunSubmission> {
        self.submit_equipment_typed(app, command, now)
            .map_err(crate::run_actions::EquipmentRunError::legacy_error)
    }
    pub fn submit_equipment_typed(
        &mut self,
        app: &mut AppState,
        command: &EquipmentEditorCommand,
        now: SystemTime,
    ) -> Result<StudioRunSubmission, crate::run_actions::EquipmentRunError> {
        self.submit_equipment_with(app, command, now, EquipmentRun::spawn)
    }
    fn submit_equipment_with(
        &mut self,
        app: &mut AppState,
        command: &EquipmentEditorCommand,
        now: SystemTime,
        spawn: impl FnOnce(rf_ui::engineering::CalculationRequest) -> std::io::Result<EquipmentRun>,
    ) -> Result<StudioRunSubmission, crate::run_actions::EquipmentRunError> {
        use crate::run_actions::EquipmentRunError;
        use rf_ui::engineering::editor::EquipmentEditError;
        if !self.accepting {
            return Ok(StudioRunSubmission::Closing);
        }
        let workspace = &app.workspace;
        let pending_drafts = match command {
            EquipmentEditorCommand::Calculate(task) => workspace
                .equipment_editor
                .pending_for_task(task, &workspace.engineering),
            EquipmentEditorCommand::CalculateRating(member) => workspace
                .equipment_editor
                .pending_for_case(member, &workspace.engineering),
            _ => {
                return Err(EquipmentRunError::Rejected(
                    EquipmentEditError::UnsupportedEditorAction,
                ));
            }
        };
        if pending_drafts {
            return Err(EquipmentRunError::Rejected(
                EquipmentEditError::PendingDrafts,
            ));
        }
        if let Some(activity) = self.activity() {
            let same = self.equipment.as_ref().is_some_and(|run| {
                run.cancel_reason.is_none()
                    && app
                        .workspace
                        .engineering
                        .calculation_is_current(&run.qualification)
                    && match command {
                        EquipmentEditorCommand::Calculate(id) => {
                            run.qualification.member().task_id == *id
                        }
                        EquipmentEditorCommand::CalculateRating(member) => {
                            run.qualification.member() == member
                        }
                        _ => false,
                    }
            });
            return Ok(if same {
                StudioRunSubmission::AlreadyRunning(activity.id)
            } else {
                StudioRunSubmission::Busy(activity.id)
            });
        }
        let workspace = &mut app.workspace;
        let request = workspace
            .equipment_editor
            .prepare_calculation(&mut workspace.engineering, command)
            .map_err(EquipmentRunError::Rejected)?;
        let qualification = request.qualification();
        let id = StudioRunId::Equipment(qualification.id().clone());
        let target = format!(
            "{} · {} {} / {}",
            workspace.document.metadata.title,
            request.task().tag(),
            qualification.member().task_id.as_str(),
            qualification.member().case_id.as_str()
        );
        match spawn(request) {
            Ok(run) => {
                self.equipment = Some(run);
                self.set_activity(id.clone(), target, now);
            }
            Err(error) => {
                workspace.engineering.revoke_calculation(&qualification);
                return Err(EquipmentRunError::StartFailed {
                    id,
                    error: RfError::invalid_input(format!(
                        "Could not start equipment worker: {error}"
                    )),
                });
            }
        }
        Ok(StudioRunSubmission::Accepted(id))
    }
    pub fn cancel(
        &mut self,
        app: &mut AppState,
        id: &StudioRunId,
    ) -> crate::WorkspaceRunCancelReply {
        use crate::WorkspaceRunCancelReply as Reply;
        if let Some(activity) = &mut self.activity
            && &activity.id == id
        {
            activity
                .cancel_requested_at
                .get_or_insert_with(SystemTime::now);
        }
        match id {
            StudioRunId::Flow(id) => self.flow.cancel(app, id),
            StudioRunId::Material(id) => {
                if let Some(run) = &mut self.material
                    && run.receipt.qualification().run_id() == id
                {
                    if run.cancel_reason.is_some() {
                        return Reply::AlreadyRequested;
                    }
                    run.cancel(Some(app), crate::WorkspaceRunCancelReason::UserRequested);
                    Reply::Requested
                } else if self
                    .history
                    .iter()
                    .any(|summary| summary.activity.id == StudioRunId::Material(id.clone()))
                {
                    Reply::AlreadyFinished
                } else {
                    Reply::UnknownRun
                }
            }
            StudioRunId::Equipment(id) => {
                if let Some(run) = &mut self.equipment
                    && run.qualification.id() == id
                {
                    if run.cancel_reason.is_some() {
                        return Reply::AlreadyRequested;
                    }
                    run.cancel(Some(app), crate::WorkspaceRunCancelReason::UserRequested);
                    Reply::Requested
                } else if self
                    .history
                    .iter()
                    .any(|summary| summary.activity.id == StudioRunId::Equipment(id.clone()))
                {
                    Reply::AlreadyFinished
                } else {
                    Reply::UnknownRun
                }
            }
        }
    }
    pub fn detach(&mut self, app: &mut AppState) {
        if let Some(run) = &mut self.material {
            run.cancel(Some(app), crate::WorkspaceRunCancelReason::Detached);
        }
        self.flow.detach(app);
        if let Some(run) = &mut self.equipment {
            run.cancel(Some(app), crate::WorkspaceRunCancelReason::Detached);
        }
        self.last_auto_attempt = Some((
            app.flow_run_qualification(),
            app.workspace.solve_session.pending_reason,
        ));
        if let Some(activity) = &mut self.activity {
            activity
                .cancel_requested_at
                .get_or_insert_with(SystemTime::now);
        }
    }
    pub fn begin_shutdown(&mut self, app: &mut AppState) {
        self.accepting = false;
        self.last_auto_attempt = Some((
            app.flow_run_qualification(),
            app.workspace.solve_session.pending_reason,
        ));
        if let Some(activity) = &mut self.activity {
            activity
                .cancel_requested_at
                .get_or_insert_with(SystemTime::now);
        }
        if let Some(run) = &mut self.equipment {
            run.cancel(Some(app), crate::WorkspaceRunCancelReason::Closing);
        }
        if let Some(run) = &mut self.material {
            run.cancel(Some(app), crate::WorkspaceRunCancelReason::Closing);
        }
        self.flow.begin_shutdown(Some(app));
    }
    pub fn resume_submissions(&mut self) {
        self.accepting = true;
        self.flow.resume_submissions();
    }
    pub fn poll(&mut self, mut app: Option<&mut AppState>, now: SystemTime) -> bool {
        let flow = match app.as_deref_mut() {
            Some(app) => self.flow.poll(app, now),
            None => self.flow.poll_detached(now),
        };
        let terminal = if let Some(summary) = flow {
            Some(StudioRunTerminal {
                outcome: match summary.outcome {
                    WorkspaceExecutionOutcome::Succeeded => StudioRunOutcome::Succeeded,
                    WorkspaceExecutionOutcome::Cancelled => StudioRunOutcome::Cancelled,
                    WorkspaceExecutionOutcome::Failed => StudioRunOutcome::Failed,
                    WorkspaceExecutionOutcome::InfrastructureFailed => {
                        StudioRunOutcome::InfrastructureFailed
                    }
                },
                applied: matches!(summary.disposition, rf_ui::FlowRunDisposition::Applied),
                cancellation: summary.cancellation,
                diagnostic_code: summary.diagnostic_code,
            })
        } else if self
            .material
            .as_mut()
            .is_some_and(|run| run.poll(app.as_deref_mut()))
        {
            Some(
                self.material
                    .take()
                    .expect("finished worker")
                    .finish(app.as_deref_mut()),
            )
        } else if self
            .equipment
            .as_mut()
            .is_some_and(|run| run.poll(app.as_deref_mut()))
        {
            Some(
                self.equipment
                    .take()
                    .expect("finished worker")
                    .finish(app.as_deref_mut()),
            )
        } else {
            None
        };
        if self
            .activity()
            .is_some_and(|activity| activity.phase == WorkspaceRunPhase::CancelRequested)
            && let Some(activity) = &mut self.activity
        {
            activity.cancel_requested_at.get_or_insert(now);
        }
        if let Some(terminal) = terminal {
            let StudioRunTerminal {
                outcome,
                applied,
                cancellation,
                diagnostic_code,
            } = terminal;
            let mut activity = self.activity.take().expect("terminal has activity");
            activity.phase = if cancellation.is_some() {
                WorkspaceRunPhase::CancelRequested
            } else {
                WorkspaceRunPhase::Running
            };
            if cancellation.is_some() {
                activity.cancel_requested_at.get_or_insert(now);
            }
            if let Some(app) = app
                && matches!(activity.id, StudioRunId::Equipment(_))
                && applied
            {
                app.push_log(
                    rf_ui::AppLogLevel::Info,
                    format!("Equipment {}: {outcome:?}", activity.target),
                );
            }
            self.history.push_back(StudioRunSummary {
                activity,
                outcome,
                applied,
                finished_at: now,
                cancellation,
                diagnostic_code,
            });
            while self.history.len() > 8 {
                self.history.pop_front();
            }
            true
        } else {
            false
        }
    }
    pub fn should_attempt_automatic(&self, app: &AppState) -> bool {
        self.accepting
            && self.is_idle()
            && app.workspace.solve_session.mode == SimulationMode::Active
            && matches!(
                app.workspace.solve_session.pending_reason,
                Some(
                    rf_ui::SolvePendingReason::DocumentRevisionAdvanced
                        | rf_ui::SolvePendingReason::ModeActivated
                )
            )
            && self.last_auto_attempt.as_ref()
                != Some(&(
                    app.flow_run_qualification(),
                    app.workspace.solve_session.pending_reason,
                ))
    }
    pub fn equipment_target_busy(
        &self,
        app: &AppState,
        task: &rf_types::EquipmentTaskId,
        case: Option<&rf_types::EngineeringCaseId>,
    ) -> bool {
        self.equipment.as_ref().is_some_and(|run| {
            let member = run.qualification.member();
            app.workspace
                .engineering
                .calculation_belongs_to_session(&run.qualification)
                && &member.task_id == task
                && case.is_none_or(|case| &member.case_id == case)
        })
    }
    pub fn flow_progress(&self) -> Option<crate::WorkspaceRunProgress> {
        self.flow.state()?.progress
    }
    pub fn try_close(self) -> Result<(), Box<Self>> {
        if self.is_idle() {
            Ok(())
        } else {
            Err(Box::new(self))
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BackgroundRunSession {
    pub owner: SharedStudioRunOwner,
    pub root: PropertyPackageCacheRoot,
}

#[derive(Debug, Clone)]
pub enum StudioBackgroundRunAction {
    ConfigureFlowPlan {
        expected: rf_ui::flow_plan::FlowPlanEditContext,
        access: crate::flow_actions::FlowEditAccess,
        command: rf_ui::flow_plan::FlowPlanCommand,
    },
    FlowPlanDraft {
        access: crate::flow_actions::FlowEditAccess,
        draft: Option<rf_ui::flow_plan::FlowPlanDraft>,
    },
    EditFlow {
        access: crate::flow_actions::FlowEditAccess,
        request: crate::flow_actions::FlowActionRequest,
    },
    ConfigureMaterial {
        expected: crate::flow_actions::FlowEditContext,
        access: crate::run_actions::StudioRunAccess,
        plan: Box<rf_solver::component_balance::MaterialBalancePlan>,
    },
    RunMaterial {
        expected: crate::flow_actions::FlowEditContext,
        access: crate::run_actions::StudioRunAccess,
    },
    Install(SharedStudioRunOwner),
    Poll {
        attached: bool,
    },
    Cancel(StudioRunId),
    Detach,
    BeginShutdown,
    ResumeSubmissions,
}

#[cfg(test)]
mod tests;

/// State refresh and a control receipt travel independently through host adapters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StudioBackgroundRunReply {
    pub changed: bool,
    pub cancellation: Option<crate::WorkspaceRunCancelReply>,
    pub submission: Option<crate::run_actions::StudioRunReply>,
}
