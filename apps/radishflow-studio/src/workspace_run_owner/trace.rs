use std::{
    mem::size_of,
    sync::Arc,
    time::{Duration, Instant},
};

use rf_solver::SequentialExecutionEvent;
use rf_types::{ComponentId, DiagnosticPortTarget, ErrorCode, RfError, StreamId, UnitId};

const PLAN_LIMIT: usize = 512;
const DETAIL_LIMIT: usize = 2_048;
const PAYLOAD_LIMIT: usize = 512 * 1024;
const TERMINAL_RESERVE: usize = 16 * 1024;
const CRITICAL_TEXT_LIMIT: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceTraceRecord {
    Recorded(Arc<WorkspaceExecutionTrace>),
    NotCollected,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTracePhase {
    Accepted,
    Preparing,
    Started,
    Validating,
    Planned,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct WorkspaceTraceCoverage {
    pub first_omitted_sequence: Option<u64>,
    pub omitted_events: usize,
    pub identity_or_details_omitted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceTracePackage {
    pub id: String,
    pub fingerprint: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceTraceStreams {
    pub consumed: Vec<StreamId>,
    pub produced: Vec<StreamId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceTraceDiagnostic {
    pub kind: ErrorCode,
    pub code: Option<String>,
    pub message: Option<String>,
    pub units: Option<Vec<UnitId>>,
    pub streams: Option<Vec<StreamId>>,
    pub compositions: Option<Vec<StreamId>>,
    pub ports: Option<Vec<DiagnosticPortTarget>>,
    pub components: Option<Vec<ComponentId>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceTraceStep {
    pub number: usize,
    pub unit: Option<UnitId>,
    pub started_sequence: u64,
    pub completed_sequence: Option<u64>,
    pub elapsed: Option<Duration>,
    pub streams: Option<WorkspaceTraceStreams>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceTraceStop {
    pub number: usize,
    pub unit: Option<UnitId>,
    pub started_sequence: u64,
    /// A failure has a terminal event. A panic can leave the last start open.
    pub failed_sequence: Option<u64>,
    pub elapsed: Duration,
    pub diagnostic: Option<WorkspaceTraceDiagnostic>,
}

/// Single worker-owned evidence, read only after the terminal payload and join.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceExecutionTrace {
    pub phases: Vec<(u64, WorkspaceTracePhase)>,
    pub package: Option<WorkspaceTracePackage>,
    pub planned_steps: Option<usize>,
    pub plan_prefix: Vec<Option<UnitId>>,
    pub steps: Vec<WorkspaceTraceStep>,
    pub stop: Option<WorkspaceTraceStop>,
    pub run_diagnostic: Option<WorkspaceTraceDiagnostic>,
    pub coverage: WorkspaceTraceCoverage,
    pub last_started: usize,
    pub terminal_sequence: u64,
    pub receipt_sequence: u64,
}

struct InProgress {
    number: usize,
    unit: Option<UnitId>,
    sequence: u64,
    since: Instant,
}

pub(super) struct TraceRecorder {
    trace: WorkspaceExecutionTrace,
    sequence: u64,
    ordinary_events: usize,
    remaining: usize,
    pending: Option<InProgress>,
}

impl TraceRecorder {
    pub fn new() -> Self {
        Self {
            trace: WorkspaceExecutionTrace {
                phases: vec![(1, WorkspaceTracePhase::Accepted)],
                package: None,
                planned_steps: None,
                plan_prefix: Vec::new(),
                steps: Vec::new(),
                stop: None,
                run_diagnostic: None,
                coverage: WorkspaceTraceCoverage::default(),
                last_started: 0,
                terminal_sequence: 0,
                receipt_sequence: 0,
            },
            sequence: 1,
            ordinary_events: 0,
            remaining: PAYLOAD_LIMIT - TERMINAL_RESERVE,
            pending: None,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }

    pub fn phase(&mut self, phase: WorkspaceTracePhase) {
        let sequence = self.next_sequence();
        self.trace.phases.push((sequence, phase));
    }

    pub fn package(&mut self, id: &str, fingerprint: &str) {
        if id.len() <= CRITICAL_TEXT_LIMIT && fingerprint.len() <= CRITICAL_TEXT_LIMIT {
            self.trace.package = Some(WorkspaceTracePackage {
                id: id.to_owned(),
                fingerprint: fingerprint.to_owned(),
            });
        } else {
            self.trace.coverage.identity_or_details_omitted = true;
        }
    }

    fn charge(&mut self, bytes: usize) -> bool {
        if let Some(remaining) = self.remaining.checked_sub(bytes) {
            self.remaining = remaining;
            true
        } else {
            false
        }
    }

    fn omit(&mut self, sequence: u64) {
        self.trace
            .coverage
            .first_omitted_sequence
            .get_or_insert(sequence);
        self.trace.coverage.omitted_events += 1;
    }

    fn record_detail(&mut self, sequence: u64) -> bool {
        if self.trace.coverage.first_omitted_sequence.is_some()
            || self.ordinary_events == DETAIL_LIMIT
        {
            self.omit(sequence);
            false
        } else {
            self.ordinary_events += 1;
            true
        }
    }

    fn plan(&mut self, order: &[UnitId]) {
        self.phase(WorkspaceTracePhase::Planned);
        self.trace.planned_steps = Some(order.len());
        let rows = order.len().min(PLAN_LIMIT);
        let steps = order.len().min(DETAIL_LIMIT / 2);
        let detail_capacity = self.trace.steps.try_reserve_exact(steps);
        let plan_capacity = self.trace.plan_prefix.try_reserve_exact(rows);
        if detail_capacity.is_err() || plan_capacity.is_err() {
            self.trace.coverage.identity_or_details_omitted = true;
        }
        // Both allocations are independently capped by the number of rows.
        let charged = self.charge(
            self.trace.steps.capacity() * size_of::<WorkspaceTraceStep>()
                + self.trace.plan_prefix.capacity() * size_of::<Option<UnitId>>(),
        );
        debug_assert!(
            charged,
            "fixed trace row allocations exceed the trace budget"
        );
        for id in order
            .iter()
            .take(self.trace.plan_prefix.capacity().min(rows))
        {
            let unit = self.capture_unit(id);
            self.trace.plan_prefix.push(unit);
        }
    }

    fn capture_unit(&mut self, id: &UnitId) -> Option<UnitId> {
        if id.as_str().len() <= CRITICAL_TEXT_LIMIT && self.charge(id.as_str().len()) {
            Some(id.clone())
        } else {
            self.trace.coverage.identity_or_details_omitted = true;
            None
        }
    }

    pub fn observe(&mut self, event: SequentialExecutionEvent<'_>) {
        match event {
            SequentialExecutionEvent::Validating => self.phase(WorkspaceTracePhase::Validating),
            SequentialExecutionEvent::Planned(order) => self.plan(order),
            SequentialExecutionEvent::StepStarted { number, unit } => {
                let sequence = self.next_sequence();
                self.trace.last_started = number;
                self.pending = Some(InProgress {
                    number,
                    unit: critical_unit(unit),
                    sequence,
                    since: Instant::now(),
                });
                if self.record_detail(sequence) {
                    if self.trace.steps.len() == self.trace.steps.capacity() {
                        self.omit(sequence);
                        return;
                    }
                    if unit.as_str().len() <= CRITICAL_TEXT_LIMIT
                        && unit.as_str().len() > self.remaining
                    {
                        self.omit(sequence);
                        return;
                    }
                    let unit = self.capture_unit(unit);
                    self.trace.steps.push(WorkspaceTraceStep {
                        number,
                        unit,
                        started_sequence: sequence,
                        completed_sequence: None,
                        elapsed: None,
                        streams: None,
                    });
                }
            }
            SequentialExecutionEvent::StepCompleted { number, step } => {
                let sequence = self.next_sequence();
                let elapsed = self.pending.take().map(|start| start.since.elapsed());
                if self.record_detail(sequence) {
                    let consumed = step.consumed_streams.iter().map(|stream| &stream.id);
                    let produced = step.produced_streams.iter().map(|stream| &stream.id);
                    let bytes = sized_ids(consumed.clone().chain(produced.clone()));
                    if !self.charge(bytes) {
                        self.omit(sequence);
                        return;
                    }
                    if let Some(last) = self
                        .trace
                        .steps
                        .last_mut()
                        .filter(|last| last.number == number)
                    {
                        last.completed_sequence = Some(sequence);
                        last.elapsed = elapsed;
                        last.streams = Some(WorkspaceTraceStreams {
                            consumed: consumed.cloned().collect(),
                            produced: produced.cloned().collect(),
                        });
                    }
                }
            }
            SequentialExecutionEvent::StepFailed {
                number,
                unit,
                error,
            } => {
                let sequence = self.next_sequence();
                let start = self
                    .pending
                    .take()
                    .expect("sequential failure follows a start");
                let (diagnostic, omitted) = capture_diagnostic(error);
                let unit = critical_unit(unit);
                self.trace.coverage.identity_or_details_omitted |= omitted || unit.is_none();
                self.trace.stop = Some(WorkspaceTraceStop {
                    number,
                    unit,
                    started_sequence: start.sequence,
                    failed_sequence: Some(sequence),
                    elapsed: start.since.elapsed(),
                    diagnostic: Some(diagnostic),
                });
            }
        }
    }

    pub fn finish(mut self, error: Option<&RfError>) -> WorkspaceExecutionTrace {
        if let Some(start) = self.pending.take() {
            self.trace.coverage.identity_or_details_omitted |= start.unit.is_none();
            self.trace.stop = Some(WorkspaceTraceStop {
                number: start.number,
                unit: start.unit,
                started_sequence: start.sequence,
                failed_sequence: None,
                elapsed: start.since.elapsed(),
                diagnostic: None,
            });
        }
        if let Some(error) = error
            && self
                .trace
                .stop
                .as_ref()
                .is_none_or(|stop| stop.diagnostic.is_none())
        {
            let (diagnostic, omitted) = capture_diagnostic(error);
            self.trace.coverage.identity_or_details_omitted |= omitted;
            self.trace.run_diagnostic = Some(diagnostic);
        }
        self.trace.terminal_sequence = self.next_sequence();
        self.trace.receipt_sequence = self.next_sequence();
        self.trace
    }
}

fn critical_unit(id: &UnitId) -> Option<UnitId> {
    (id.as_str().len() <= CRITICAL_TEXT_LIMIT).then(|| id.clone())
}

fn sized_ids<'a, Id: AsRef<str> + 'a>(ids: impl Iterator<Item = &'a Id>) -> usize {
    ids.fold(0usize, |bytes, id| {
        bytes
            .saturating_add(size_of::<Id>())
            .saturating_add(id.as_ref().len())
    })
}

fn capture_diagnostic(error: &RfError) -> (WorkspaceTraceDiagnostic, bool) {
    let context = error.context();
    let mut omitted = false;
    let mut copy_allowed = |n: usize| {
        omitted |= n > CRITICAL_TEXT_LIMIT;
        n <= CRITICAL_TEXT_LIMIT
    };
    let code = context
        .diagnostic_code()
        .filter(|code| copy_allowed(code.len()))
        .map(str::to_owned);
    let message = copy_allowed(error.message().len()).then(|| error.message().to_owned());
    let units = copy_allowed(sized_ids(context.related_unit_ids().iter()))
        .then(|| context.related_unit_ids().to_vec());
    let streams = copy_allowed(sized_ids(context.related_stream_ids().iter()))
        .then(|| context.related_stream_ids().to_vec());
    let compositions = copy_allowed(sized_ids(context.related_composition_stream_ids().iter()))
        .then(|| context.related_composition_stream_ids().to_vec());
    let port_bytes = context
        .related_port_targets()
        .iter()
        .fold(0usize, |bytes, port| {
            bytes
                .saturating_add(size_of::<DiagnosticPortTarget>() + port.unit_id.as_str().len())
                .saturating_add(port.port_name.len())
        });
    let ports = copy_allowed(port_bytes).then(|| context.related_port_targets().to_vec());
    let components = copy_allowed(sized_ids(context.related_component_ids().iter()))
        .then(|| context.related_component_ids().to_vec());
    (
        WorkspaceTraceDiagnostic {
            kind: error.code(),
            code,
            message,
            units,
            streams,
            compositions,
            ports,
            components,
        },
        omitted,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_solver::UnitSolveStep;

    #[test]
    fn omitted_prefix_keeps_the_actual_terminal_step_at_the_count_and_byte_limits() {
        for (id_length, expected_prefix) in [(8, DETAIL_LIMIT / 2), (CRITICAL_TEXT_LIMIT, 0)] {
            let unit = UnitId::new("u".repeat(id_length));
            let mut recorder = TraceRecorder::new();
            recorder.observe(SequentialExecutionEvent::Planned(&vec![
                unit.clone();
                4_096
            ]));
            let snapshot = UnitSolveStep {
                index: 0,
                unit_id: unit.clone(),
                unit_name: String::new(),
                unit_kind: String::new(),
                consumed_streams: Vec::new(),
                produced_streams: Vec::new(),
                energy_balance: None,
                summary: String::new(),
            };
            for number in 1..4_096 {
                recorder.observe(SequentialExecutionEvent::StepStarted {
                    number,
                    unit: &unit,
                });
                recorder.observe(SequentialExecutionEvent::StepCompleted {
                    number,
                    step: &snapshot,
                });
            }
            recorder.observe(SequentialExecutionEvent::StepStarted {
                number: 4_096,
                unit: &unit,
            });
            let error = RfError::invalid_input("late failure")
                .with_diagnostic_code("solver.step.execution");
            recorder.observe(SequentialExecutionEvent::StepFailed {
                number: 4_096,
                unit: &unit,
                error: &error,
            });
            let trace = recorder.finish(Some(&error));
            assert_eq!(trace.planned_steps, Some(4_096));
            assert_eq!(trace.plan_prefix.len(), PLAN_LIMIT);
            assert_eq!(trace.steps.len(), expected_prefix);
            assert!(trace.coverage.first_omitted_sequence.is_some());
            assert_eq!(trace.coverage.omitted_events, 8_191 - expected_prefix * 2);
            let stop = trace.stop.unwrap();
            assert_eq!(stop.number, 4_096);
            assert_eq!(stop.unit, Some(unit));
            assert_eq!(
                stop.diagnostic.unwrap().code.as_deref(),
                Some("solver.step.execution")
            );
            assert!(stop.failed_sequence < Some(trace.terminal_sequence));
            assert!(trace.run_diagnostic.is_none());
        }
    }

    #[test]
    fn an_unwound_step_keeps_its_start_without_claiming_numeric_failure_or_completion() {
        let unit = UnitId::new("feed-a");
        let mut recorder = TraceRecorder::new();
        recorder.observe(SequentialExecutionEvent::Planned(std::slice::from_ref(
            &unit,
        )));
        recorder.observe(SequentialExecutionEvent::StepStarted {
            number: 1,
            unit: &unit,
        });
        let error = RfError::invalid_input("worker panicked")
            .with_diagnostic_code("workspace.run.worker_panicked");
        let trace = recorder.finish(Some(&error));
        let stop = trace.stop.unwrap();
        assert_eq!(stop.number, 1);
        assert!(stop.failed_sequence.is_none());
        assert!(stop.diagnostic.is_none());
        assert!(trace.steps[0].completed_sequence.is_none());
        assert_eq!(
            trace.run_diagnostic.unwrap().code.as_deref(),
            Some("workspace.run.worker_panicked")
        );
    }

    #[test]
    #[ignore = "explicit development measurement on a 4,096-feed acyclic flowsheet"]
    fn measure_large_acyclic_trace() {
        use crate::test_support::{
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            build_official_binary_hydrocarbon_in_memory_provider,
        };
        use rf_property_package::{PropertyPackageProvider, prepare_local_context};
        use rf_solver::{
            ControlledSolveOutcome, SequentialModularSolver, SolveProgress, SolverControl,
            SolverServices,
        };
        use std::{ops::ControlFlow, time::SystemTime};

        struct Control(Option<TraceRecorder>);
        impl SolverControl for Control {
            fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
                ControlFlow::Continue(())
            }

            fn observe_sequential(&mut self, event: SequentialExecutionEvent<'_>) {
                if let Some(recorder) = &mut self.0 {
                    recorder.observe(event);
                }
            }
        }

        let package = OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID;
        let context = prepare_local_context(
            build_official_binary_hydrocarbon_in_memory_provider(package)
                .load_package(package, SystemTime::now())
                .unwrap(),
        )
        .unwrap();
        let services = SolverServices {
            thermo: context.thermo(),
            flash_solver: context.tp_flash(),
        };
        let source = rf_store::parse_project_file_json(include_str!(
            "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
        ))
        .unwrap()
        .document
        .flowsheet;
        let feed = &source.units[&UnitId::new("feed-a")];
        let feed_stream = &source.streams[&StreamId::new("stream-feed-a")];
        let mut flow = source.clone();
        flow.units.clear();
        flow.streams.clear();
        for index in 1..=4_096 {
            let mut unit = feed.clone();
            let mut stream = feed_stream.clone();
            unit.id = format!("feed-{index:04}").into();
            stream.id = format!("stream-{index:04}").into();
            unit.ports[0].stream_id = Some(stream.id.clone());
            flow.units.insert(unit.id.clone(), unit);
            flow.streams.insert(stream.id.clone(), stream);
        }
        let mut measurements = [Vec::new(), Vec::new()];
        let mut payload_upper_bound = 0;
        for attempt in 0..=8 {
            for record in [attempt % 2 == 0, attempt % 2 != 0] {
                let mut control = Control(record.then(TraceRecorder::new));
                let started = Instant::now();
                let solved = SequentialModularSolver
                    .solve_controlled(&services, &flow, &mut control)
                    .unwrap();
                let trace = control.0.map(|recorder| {
                    payload_upper_bound = PAYLOAD_LIMIT - recorder.remaining;
                    recorder.finish(None)
                });
                let elapsed = started.elapsed();
                if attempt != 0 {
                    measurements[usize::from(record)].push(elapsed);
                }
                let ControlledSolveOutcome::Completed(snapshot) = solved else {
                    panic!("uncancelled workload")
                };
                assert_eq!(snapshot.steps.len(), 4_096);
                if let Some(trace) = trace {
                    assert_eq!(trace.planned_steps, Some(4_096));
                    assert_eq!(trace.steps.len(), 1_024);
                    assert_eq!(trace.last_started, 4_096);
                    assert_eq!(trace.coverage.omitted_events, 6_144);
                }
            }
        }
        for samples in &mut measurements {
            samples.sort_unstable();
        }
        let medians = measurements.map(|samples| (samples[3] + samples[4]) / 2);
        eprintln!(
            "4,096 steps, 8 measured runs per mode: median without trace {:.2?}, with trace {:.2?}; trace payload upper bound {payload_upper_bound} bytes including terminal reserve",
            medians[0], medians[1]
        );
    }
}
