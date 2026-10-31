//! Application-owned background flow execution. Keep this owner outside the
//! replaceable document host; poll until idle before dropping it or exiting.
//! The desktop StudioRunOwner shares its numeric slot with device calculations.
mod authorization;
#[cfg(test)]
mod tests;
mod worker;

use std::{collections::VecDeque, sync::mpsc::TryRecvError, thread::JoinHandle, time::SystemTime};

use rf_store::StoredAuthCacheIndex;
use rf_types::RfResult;
use rf_ui::{
    AppState, FlowRunDiscardReason, FlowRunDisposition, FlowRunId, FlowRunReceipt, SimulationMode,
};

use crate::{
    CompletedWorkspaceSolve, PropertyPackageCacheRoot, StudioSolveRequest,
    StudioWorkspaceRunBlocked, WorkspaceExecutionOutcome, WorkspaceRunCommand,
    WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    app_facade::{WorkspaceRunPreflight, preflight_workspace_run},
    build_workspace_solve_request, prepare_workspace_solve,
    solver_bridge::execution::{complete_background_workspace_solve, load_cached_context},
};
use authorization::RunAuthorization;
pub use authorization::RunAuthorizationRefusal;
pub use worker::WorkspaceRunProgress;
use worker::{
    PackageLoader, ThreadSpawner, WorkerChannels, WorkerResult, WorkerSpawner,
    infrastructure_error, spawn_worker,
};

const HISTORY_LIMIT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRunBlock {
    Preflight(StudioWorkspaceRunBlocked),
    Authorization(RunAuthorizationRefusal),
    Closing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRunSubmission {
    Accepted(FlowRunId),
    AlreadyRunning(FlowRunId),
    Busy(FlowRunId),
    Blocked(WorkspaceRunBlock),
    Skipped(WorkspaceSolveSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRunPhase {
    Preparing,
    Running,
    CancelRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRunCancelReason {
    UserRequested,
    QualificationChanged,
    Authorization(RunAuthorizationRefusal),
    Detached,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRunCancelReply {
    Requested,
    AlreadyRequested,
    AlreadyFinished,
    UnknownRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRunState {
    pub run_id: FlowRunId,
    pub phase: WorkspaceRunPhase,
    pub submitted_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub progress: Option<WorkspaceRunProgress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRunSummary {
    pub run_id: FlowRunId,
    pub submitted_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub finished_at: SystemTime,
    pub outcome: WorkspaceExecutionOutcome,
    pub disposition: FlowRunDisposition,
    pub cancellation: Option<WorkspaceRunCancelReason>,
    pub diagnostic_code: Option<String>,
}

struct ActiveRun {
    receipt: FlowRunReceipt,
    request: StudioSolveRequest,
    authorization: RunAuthorization,
    submitted_at: SystemTime,
    started_at: Option<SystemTime>,
    cancel_reason: Option<WorkspaceRunCancelReason>,
    channels: WorkerChannels,
    handle: JoinHandle<()>,
    received: Option<WorkerResult>,
    progress: Option<WorkspaceRunProgress>,
    progress_sequence: u64,
}

/// Owns at most one worker and no queue. `begin_shutdown` rejects new work;
/// continue polling and use `try_close` to prove all handles have been reaped.
/// There is deliberately no blocking Drop or forced thread termination.
#[must_use = "keep the run owner alive and drain it before closing its application"]
pub struct WorkspaceRunOwner {
    active: Option<Box<ActiveRun>>,
    history: VecDeque<WorkspaceRunSummary>,
    accepting: bool,
}

impl Default for WorkspaceRunOwner {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRunOwner {
    pub fn new() -> Self {
        Self {
            active: None,
            history: VecDeque::new(),
            accepting: true,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.active.is_none()
    }
    pub fn history(&self) -> &VecDeque<WorkspaceRunSummary> {
        &self.history
    }

    pub fn state(&self) -> Option<WorkspaceRunState> {
        self.active.as_ref().map(|active| WorkspaceRunState {
            run_id: active.receipt.run_id().clone(),
            phase: if active.cancel_reason.is_some() {
                WorkspaceRunPhase::CancelRequested
            } else if active.started_at.is_some() {
                WorkspaceRunPhase::Running
            } else {
                WorkspaceRunPhase::Preparing
            },
            submitted_at: active.submitted_at,
            started_at: active.started_at,
            progress: active.progress.clone(),
        })
    }

    /// This method performs existing read-only input preflight, reserves the one
    /// slot, then freezes inputs. Cache IO and all numeric work occur on worker.
    pub fn submit_cached(
        &mut self,
        app: &mut AppState,
        root: PropertyPackageCacheRoot,
        index: &StoredAuthCacheIndex,
        command: &WorkspaceRunCommand,
        now: SystemTime,
    ) -> RfResult<WorkspaceRunSubmission> {
        if !self.accepting {
            return Ok(WorkspaceRunSubmission::Blocked(WorkspaceRunBlock::Closing));
        }
        let package_id = match preflight_workspace_run(app, index, command) {
            WorkspaceRunPreflight::Ready(id) => id,
            WorkspaceRunPreflight::Skipped(reason) => {
                return Ok(WorkspaceRunSubmission::Skipped(reason));
            }
            WorkspaceRunPreflight::Blocked { blocked, .. } => {
                return Ok(WorkspaceRunSubmission::Blocked(
                    WorkspaceRunBlock::Preflight(blocked),
                ));
            }
        };
        let authorization = match RunAuthorization::capture(app, &package_id, now) {
            Ok(authorization) => authorization,
            Err(reason) => {
                return Ok(WorkspaceRunSubmission::Blocked(
                    WorkspaceRunBlock::Authorization(reason),
                ));
            }
        };
        if let Some(reply) = self.existing_submission(app, &package_id) {
            return Ok(reply);
        }
        let request = build_workspace_solve_request(app, package_id)?;
        let index = index.clone();
        let load: PackageLoader = Box::new(move |id| {
            // Consuming this closure releases the root lease immediately after
            // loading, before Ready or a potentially long numeric calculation.
            load_cached_context(root.path(), &index, id)
        });
        if command.trigger == WorkspaceSolveTrigger::Manual {
            app.request_manual_run();
        }
        self.spawn(app, request, authorization, load, true, now, &ThreadSpawner)
    }

    fn existing_submission(&self, app: &AppState, package: &str) -> Option<WorkspaceRunSubmission> {
        self.active.as_ref().map(|active| {
            let id = active.receipt.run_id().clone();
            if active.cancel_reason.is_none()
                && active.request.package_id == package
                && active.receipt.qualification() == &app.flow_run_qualification()
                && app.workspace.pending_flow_run() == Some(&id)
            {
                WorkspaceRunSubmission::AlreadyRunning(id)
            } else {
                WorkspaceRunSubmission::Busy(id)
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn(
        &mut self,
        app: &mut AppState,
        request: StudioSolveRequest,
        authorization: RunAuthorization,
        load: PackageLoader,
        cached: bool,
        now: SystemTime,
        spawner: &impl WorkerSpawner,
    ) -> RfResult<WorkspaceRunSubmission> {
        let prepared = prepare_workspace_solve(app, &request)?;
        let (receipt, job) = prepared.into_job();
        let id = receipt.run_id().clone();
        let (channels, control, result) = WorkerChannels::new(id.clone());
        let handle = match spawn_worker(spawner, job, load, cached, control, result) {
            Ok(handle) => handle,
            Err(error) => {
                let error = infrastructure_error(
                    "spawn_failed",
                    &format!("failed to start solve worker: {error}"),
                );
                let mut completion = CompletedWorkspaceSolve::infrastructure_failure(
                    receipt,
                    request,
                    error.clone(),
                );
                let disposition = complete_background_workspace_solve(app, &mut completion);
                self.record_summary(WorkspaceRunSummary {
                    run_id: id,
                    submitted_at: now,
                    started_at: None,
                    finished_at: now,
                    outcome: WorkspaceExecutionOutcome::InfrastructureFailed,
                    disposition,
                    cancellation: None,
                    diagnostic_code: error.context().diagnostic_code().map(str::to_owned),
                });
                return Err(error);
            }
        };
        self.active = Some(Box::new(ActiveRun {
            receipt,
            request,
            authorization,
            submitted_at: now,
            started_at: None,
            cancel_reason: None,
            channels,
            handle,
            received: None,
            progress: None,
            progress_sequence: 0,
        }));
        Ok(WorkspaceRunSubmission::Accepted(id))
    }

    pub fn cancel(&mut self, app: &mut AppState, id: &FlowRunId) -> WorkspaceRunCancelReply {
        let Some(active) = self
            .active
            .as_mut()
            .filter(|active| active.receipt.run_id() == id)
        else {
            return if self.history.iter().any(|item| &item.run_id == id) {
                WorkspaceRunCancelReply::AlreadyFinished
            } else {
                WorkspaceRunCancelReply::UnknownRun
            };
        };
        if active
            .receipt
            .qualification()
            .same_session(&app.flow_run_qualification())
        {
            app.set_simulation_mode(SimulationMode::Hold);
        }
        if active.cancel_reason.is_some() {
            return WorkspaceRunCancelReply::AlreadyRequested;
        }
        Self::cancel_active(active, Some(app), WorkspaceRunCancelReason::UserRequested);
        WorkspaceRunCancelReply::Requested
    }

    pub fn detach(&mut self, app: &mut AppState) {
        if let Some(active) = &mut self.active {
            Self::cancel_active(active, Some(app), WorkspaceRunCancelReason::Detached);
        }
    }

    pub fn begin_shutdown(&mut self, mut app: Option<&mut AppState>) {
        self.accepting = false;
        if let Some(active) = &mut self.active {
            if let Some(app) = app.as_deref_mut()
                && active
                    .receipt
                    .qualification()
                    .same_session(&app.flow_run_qualification())
            {
                app.set_simulation_mode(SimulationMode::Hold);
            }
            Self::cancel_active(active, app, WorkspaceRunCancelReason::Closing);
        }
    }

    /// Cancelling application exit never uncancels a task already asked to stop.
    pub fn resume_submissions(&mut self) {
        self.accepting = true;
    }

    pub fn try_close(self) -> Result<(), Self> {
        if self.active.is_none() {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn cancel_active(
        active: &mut ActiveRun,
        app: Option<&mut AppState>,
        reason: WorkspaceRunCancelReason,
    ) {
        if let Some(app) = app {
            app.workspace.revoke_flow_run(active.receipt.run_id());
        }
        active.cancel_reason.get_or_insert(reason);
        active.progress = None;
        active.channels.request_cancel();
    }

    pub fn poll(&mut self, app: &mut AppState, now: SystemTime) -> Option<WorkspaceRunSummary> {
        self.poll_inner(Some(app), now)
    }

    pub fn poll_detached(&mut self, now: SystemTime) -> Option<WorkspaceRunSummary> {
        self.poll_inner(None, now)
    }

    fn poll_inner(
        &mut self,
        mut app: Option<&mut AppState>,
        now: SystemTime,
    ) -> Option<WorkspaceRunSummary> {
        let active = self.active.as_mut()?;
        if active.cancel_reason.is_none() {
            let reason = match app.as_deref() {
                None => Some(WorkspaceRunCancelReason::Detached),
                Some(app)
                    if active.receipt.qualification() != &app.flow_run_qualification()
                        || app.workspace.pending_flow_run() != Some(active.receipt.run_id()) =>
                {
                    Some(WorkspaceRunCancelReason::QualificationChanged)
                }
                Some(app) => (if active.started_at.is_none() {
                    active.authorization.allow_start(app, now)
                } else {
                    active.authorization.check_revocation(app)
                })
                .err()
                .map(WorkspaceRunCancelReason::Authorization),
            };
            if let Some(reason) = reason {
                Self::cancel_active(active, app.as_deref_mut(), reason);
            }
        }
        if active.channels.ready.try_recv().is_ok() && active.cancel_reason.is_none() {
            let authorization = app.as_deref().expect("attached run has an application");
            match active.authorization.allow_start(authorization, now) {
                Ok(()) => {
                    // No wait on the owner thread. A disconnected start receiver
                    // is handled through its terminal channel / joined worker.
                    if active.channels.start.try_send(true).is_ok() {
                        active.started_at = Some(now);
                    }
                }
                Err(reason) => Self::cancel_active(
                    active,
                    app.as_deref_mut(),
                    WorkspaceRunCancelReason::Authorization(reason),
                ),
            }
        }
        let progress = active
            .channels
            .progress
            .lock()
            .expect("progress slot poisoned")
            .take();
        if let Some(progress) = progress
            && active.cancel_reason.is_none()
            && progress.run_id == *active.receipt.run_id()
            && progress.sequence > active.progress_sequence
        {
            active.progress_sequence = progress.sequence;
            active.progress = Some(progress);
        }
        if active.received.is_none() {
            match active.channels.result.try_recv() {
                Ok(result) => active.received = Some(result),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => {}
            }
        }
        // Never join a running worker, including after a terminal payload arrives.
        if !active.handle.is_finished() {
            return None;
        }
        let mut active = self.active.take().expect("finished active worker");
        let joined = active.handle.join();
        // A send immediately before thread exit can race the preceding try_recv.
        if active.received.is_none() {
            active.received = active.channels.result.try_recv().ok();
        }
        let mut completion = match (joined, active.received) {
            (Ok(()), Some(WorkerResult::Completed(output))) => {
                CompletedWorkspaceSolve::from_output(active.receipt, *output)
            }
            (Err(_), _) | (_, Some(WorkerResult::Panicked)) => {
                CompletedWorkspaceSolve::infrastructure_failure(
                    active.receipt,
                    active.request,
                    infrastructure_error("worker_panicked", "solve worker panicked"),
                )
            }
            (Ok(()), None) => CompletedWorkspaceSolve::infrastructure_failure(
                active.receipt,
                active.request,
                infrastructure_error(
                    "completion_disconnected",
                    "solve worker exited without a terminal payload",
                ),
            ),
        };
        let disposition = match app {
            Some(app) => complete_background_workspace_solve(app, &mut completion),
            None => FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession),
        };
        let summary = WorkspaceRunSummary {
            run_id: completion.run_id().clone(),
            submitted_at: active.submitted_at,
            started_at: active.started_at,
            finished_at: now,
            outcome: completion.execution_outcome(),
            disposition,
            cancellation: active.cancel_reason,
            diagnostic_code: completion
                .result()
                .err()
                .and_then(|error| error.context().diagnostic_code().map(str::to_owned)),
        };
        self.record_summary(summary.clone());
        Some(summary)
    }

    fn record_summary(&mut self, summary: WorkspaceRunSummary) {
        self.history.push_back(summary);
        while self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
    }
}
