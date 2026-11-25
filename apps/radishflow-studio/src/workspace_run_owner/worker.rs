use std::{
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};

use rf_property_package::LocalThermoContext;
use rf_solver::{SequentialExecutionEvent, SolveProgress, SolverControl};
use rf_types::{RfError, RfResult};
use rf_ui::FlowRunId;

use super::trace::{TraceRecorder, WorkspaceTracePhase};
use super::unit_control::UnitControlGate;
use crate::{
    WorkspaceSolveProgress,
    solver_bridge::execution::{
        WorkspacePackageLoadError, WorkspaceSolveControl, WorkspaceSolveJob, WorkspaceSolveOutput,
    },
};

pub(super) type PackageLoader =
    Box<dyn FnOnce(&str) -> Result<LocalThermoContext, WorkspacePackageLoadError> + Send>;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRunProgress {
    pub run_id: FlowRunId,
    pub sequence: u64,
    pub progress: WorkspaceSolveProgress,
}

#[derive(Debug)]
pub(super) enum WorkerResult {
    Completed(Box<WorkspaceSolveOutput>),
    Panicked,
}

pub(super) struct WorkerTerminal {
    pub result: WorkerResult,
    pub trace: Option<Box<TraceRecorder>>,
}

pub(super) trait WorkerSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>>;
}

pub(super) struct ThreadSpawner;
impl WorkerSpawner for ThreadSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("radishflow-solve".into())
            .spawn(job)
    }
}

pub(super) struct WorkerChannels {
    pub units: Option<Arc<UnitControlGate>>,
    pub cancel: Arc<AtomicBool>,
    pub progress: Arc<Mutex<Option<WorkspaceRunProgress>>>,
    pub ready: Receiver<()>,
    pub start: SyncSender<bool>,
    pub result: Receiver<WorkerTerminal>,
}

pub(super) struct WorkerControl {
    units: Option<Arc<UnitControlGate>>,
    id: FlowRunId,
    watching_temperature: bool,
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<WorkspaceRunProgress>>>,
    sequence: u64,
    ready: SyncSender<()>,
    start: Receiver<bool>,
    trace: Option<Box<TraceRecorder>>,
}

impl WorkerChannels {
    pub fn new(
        id: FlowRunId,
        record_trace: bool,
        unit_control: bool,
    ) -> (Self, WorkerControl, SyncSender<WorkerTerminal>) {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(Mutex::new(None));
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (start, start_rx) = mpsc::sync_channel(1);
        let (result_tx, result) = mpsc::sync_channel(1);
        let units = unit_control.then(|| Arc::new(UnitControlGate::new()));
        (
            Self {
                units: units.clone(),
                cancel: cancel.clone(),
                progress: progress.clone(),
                ready,
                start,
                result,
            },
            WorkerControl {
                units,
                id,
                cancel,
                progress,
                sequence: 0,
                watching_temperature: false,
                ready: ready_tx,
                start: start_rx,
                trace: record_trace.then(|| Box::new(TraceRecorder::new())),
            },
            result_tx,
        )
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(units) = &self.units {
            units.cancel();
        }
        // Wakes a worker waiting at Ready, including a grant/cancel race.
        let _ = self.start.try_send(false);
    }
}

impl Drop for WorkerChannels {
    fn drop(&mut self) {
        // A disappearing owner cannot leave a suspended worker waiting for a command.
        self.request_cancel();
    }
}

impl WorkerControl {
    fn emit(&mut self, progress: WorkspaceSolveProgress) -> ControlFlow<()> {
        if self.cancel.load(Ordering::Acquire) {
            return ControlFlow::Break(());
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("progress sequence exhausted");
        *self.progress.lock().expect("progress slot poisoned") = Some(WorkspaceRunProgress {
            run_id: self.id.clone(),
            sequence: self.sequence,
            progress,
        });
        ControlFlow::Continue(())
    }
}

impl SolverControl for WorkerControl {
    fn checkpoint(&mut self, progress: SolveProgress) -> ControlFlow<()> {
        if progress.stage == rf_solver::SolveStage::UnitStarted
            && let Some(gate) = &self.units
        {
            gate.before_unit(
                progress.completed_units + 1,
                progress.total_units.expect("unit entry has a plan"),
                progress
                    .current_unit
                    .as_ref()
                    .expect("unit entry has a target"),
            )?;
        }
        if self.watching_temperature
            && progress.stage == rf_solver::SolveStage::Completing
            && let Some(gate) = &self.units
        {
            gate.completing()?;
        }
        self.emit(WorkspaceSolveProgress::Solving(progress))
    }

    fn observe_sequential(&mut self, event: SequentialExecutionEvent<'_>) {
        if self.watching_temperature
            && let Some(gate) = &self.units
            && gate.observe(&event)
        {
            self.cancel.store(true, Ordering::Release);
        }
        if let Some(trace) = &mut self.trace {
            trace.observe(event);
        }
    }
}

impl rf_solver::partitioned::PartitionedSolverControl for WorkerControl {
    fn checkpoint(
        &mut self,
        progress: rf_solver::partitioned::PartitionedProgress,
    ) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::Partitioned(progress))
    }
}
impl rf_solver::simultaneous::ModularControl for WorkerControl {
    fn checkpoint(
        &mut self,
        progress: rf_solver::simultaneous::ModularProgress,
    ) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::Simultaneous(progress))
    }
}
impl rf_solver::equation_oriented::EquationControl for WorkerControl {
    fn checkpoint(
        &mut self,
        progress: rf_solver::equation_oriented::EquationProgress,
    ) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::EquationOriented(progress))
    }
}
impl WorkspaceSolveControl for WorkerControl {
    fn before_load(&mut self) -> ControlFlow<()> {
        let state = self.emit(WorkspaceSolveProgress::Preparing);
        if state.is_continue()
            && let Some(trace) = &mut self.trace
        {
            trace.phase(WorkspaceTracePhase::Preparing);
        }
        state
    }

    fn package_loaded(&mut self, id: &str, fingerprint: &str) {
        if let Some(trace) = &mut self.trace {
            trace.package(id, fingerprint);
        }
    }

    fn ready(&mut self) -> RfResult<ControlFlow<()>> {
        if self.emit(WorkspaceSolveProgress::Ready).is_break() {
            return Ok(ControlFlow::Break(()));
        }
        self.ready.send(()).map_err(|_| {
            infrastructure_error("ready_disconnected", "run owner disconnected before Ready")
        })?;
        let allowed = self.start.recv().map_err(|_| {
            infrastructure_error("start_disconnected", "run owner disconnected during Ready")
        })?;
        Ok(if allowed && !self.cancel.load(Ordering::Acquire) {
            self.watching_temperature = self
                .units
                .as_ref()
                .is_some_and(|gate| gate.snapshot().observation.is_some());
            if let Some(trace) = &mut self.trace {
                trace.phase(WorkspaceTracePhase::Started);
            }
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        })
    }
}

pub(super) fn spawn_worker(
    spawner: &impl WorkerSpawner,
    job: WorkspaceSolveJob,
    load: PackageLoader,
    cached: bool,
    mut control: WorkerControl,
    result: SyncSender<WorkerTerminal>,
) -> std::io::Result<JoinHandle<()>> {
    spawner.spawn(Box::new(move || {
        let outcome =
            match catch_unwind(AssertUnwindSafe(|| job.execute(load, cached, &mut control))) {
                Ok(output) => WorkerResult::Completed(Box::new(output)),
                Err(_) => WorkerResult::Panicked,
            };
        if let Some(units) = &control.units {
            units.finish();
        }
        // Exactly one terminal payload; progress uses a separate replaceable slot.
        let _ = result.send(WorkerTerminal {
            result: outcome,
            trace: control.trace,
        });
    }))
}

pub(super) fn infrastructure_error(code: &str, message: &str) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("workspace.run.{code}"))
}

impl rf_solver::mixed::MixedControl for WorkerControl {
    fn checkpoint(&mut self, p: rf_solver::mixed::MixedProgress) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::Mixed(p))
    }
}
