pub mod component_balance;
mod control;
pub mod partitioned;
pub use control::{ControlledSolveOutcome, SolveProgress, SolveStage, SolverControl};

mod parameter_checks;

use parameter_checks::validate_step_parameters;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rf_flash::TpFlashSolver;
use rf_flowsheet::dependencies::material_dependencies;
use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_thermo::ThermoProvider;
use rf_types::{DiagnosticPortTarget, PortDirection, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{
    BuiltinUnitKind, UnitOperation, UnitOperationInputs, UnitOperationServices, validate_unit_node,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Converged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SolveDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SolverDiagnosticCode {
    ExecutionOrder,
    UnitExecuted,
    ConnectionValidation,
    TopologicalOrdering,
    StepLookup,
    StepSpec,
    StepParameter,
    StepStreamInput,
    StepInstantiation,
    StepInlet,
    StepMaterialization,
    StepExecution,
}

const SOLVER_DIAGNOSTIC_TOPOLOGICAL_SELF_LOOP_CYCLE: &str =
    "solver.topological_ordering.self_loop_cycle";
const SOLVER_DIAGNOSTIC_TOPOLOGICAL_TWO_UNIT_CYCLE: &str =
    "solver.topological_ordering.two_unit_cycle";

impl SolverDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ExecutionOrder => "solver.execution_order",
            Self::UnitExecuted => "solver.unit_executed",
            Self::ConnectionValidation => "solver.connection_validation",
            Self::TopologicalOrdering => "solver.topological_ordering",
            Self::StepLookup => "solver.step.lookup",
            Self::StepSpec => "solver.step.spec",
            Self::StepParameter => "solver.step.parameter",
            Self::StepStreamInput => "solver.step.stream_input",
            Self::StepInstantiation => "solver.step.instantiation",
            Self::StepInlet => "solver.step.inlet",
            Self::StepMaterialization => "solver.step.materialization",
            Self::StepExecution => "solver.step.execution",
        }
    }

    const fn stage_label(self) -> &'static str {
        match self {
            Self::ExecutionOrder => "execution order",
            Self::UnitExecuted => "unit executed",
            Self::ConnectionValidation => "connection validation",
            Self::TopologicalOrdering => "topological ordering",
            Self::StepLookup => "unit lookup",
            Self::StepSpec => "unit spec validation",
            Self::StepParameter => "unit parameter validation",
            Self::StepStreamInput => "stream input validation",
            Self::StepInstantiation => "operation instantiation",
            Self::StepInlet => "inlet resolution",
            Self::StepMaterialization => "output materialization",
            Self::StepExecution => "unit execution",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveDiagnosticSummary {
    pub highest_severity: SolveDiagnosticSeverity,
    pub primary_message: String,
    pub diagnostic_count: usize,
    pub related_unit_ids: Vec<UnitId>,
    pub related_stream_ids: Vec<StreamId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveDiagnostic {
    pub severity: SolveDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub related_unit_ids: Vec<UnitId>,
    pub related_stream_ids: Vec<StreamId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSolveStep {
    pub index: usize,
    pub unit_id: UnitId,
    pub unit_name: String,
    pub unit_kind: String,
    pub consumed_streams: Vec<MaterialStreamState>,
    pub produced_streams: Vec<MaterialStreamState>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveSnapshot {
    pub status: SolveStatus,
    pub summary: SolveDiagnosticSummary,
    pub diagnostics: Vec<SolveDiagnostic>,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub steps: Vec<UnitSolveStep>,
}

impl SolveSnapshot {
    pub fn stream(&self, id: &StreamId) -> Option<&MaterialStreamState> {
        self.streams.get(id)
    }

    pub fn step(&self, index: usize) -> Option<&UnitSolveStep> {
        self.steps.get(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SolveFailureContext {
    pub primary_code: Option<String>,
    pub related_unit_ids: Vec<UnitId>,
    pub related_stream_ids: Vec<StreamId>,
    pub related_port_targets: Vec<DiagnosticPortTarget>,
}

impl SolveFailureContext {
    pub fn from_error(error: &RfError) -> Self {
        let primary_code = error
            .context()
            .diagnostic_code()
            .map(str::to_string)
            .or_else(|| prefixed_solver_diagnostic_code(error.message()));
        let related_unit_ids = if error.context().related_unit_ids().is_empty() {
            related_unit_ids_from_failure_message(error.message())
        } else {
            error.context().related_unit_ids().to_vec()
        };
        let related_stream_ids = if error.context().related_stream_ids().is_empty() {
            related_stream_ids_from_failure_message(error.message())
        } else {
            error.context().related_stream_ids().to_vec()
        };
        let related_port_targets = error.context().related_port_targets().to_vec();

        Self {
            primary_code,
            related_unit_ids,
            related_stream_ids,
            related_port_targets,
        }
    }

    pub fn from_message(message: &str) -> Self {
        Self {
            primary_code: prefixed_solver_diagnostic_code(message),
            related_unit_ids: related_unit_ids_from_failure_message(message),
            related_stream_ids: related_stream_ids_from_failure_message(message),
            related_port_targets: Vec::new(),
        }
    }
}

pub struct SolverServices<'a> {
    pub thermo: &'a dyn ThermoProvider,
    pub flash_solver: &'a dyn TpFlashSolver,
}

pub trait FlowsheetSolver {
    fn solve(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
    ) -> RfResult<SolveSnapshot>;
}

#[derive(Debug, Default)]
pub struct SequentialModularSolver;

impl FlowsheetSolver for SequentialModularSolver {
    fn solve(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
    ) -> RfResult<SolveSnapshot> {
        match self.solve_controlled(services, flowsheet, &mut control::UncontrolledSolve)? {
            ControlledSolveOutcome::Completed(snapshot) => Ok(snapshot),
            ControlledSolveOutcome::Cancelled => unreachable!("uncontrolled solve cannot cancel"),
        }
    }
}

impl SequentialModularSolver {
    pub fn solve_controlled(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
        control: &mut impl SolverControl,
    ) -> RfResult<ControlledSolveOutcome> {
        if control
            .checkpoint(SolveProgress {
                stage: SolveStage::Validating,
                total_units: None,
                completed_units: 0,
                current_unit: None,
            })
            .is_break()
        {
            return Ok(ControlledSolveOutcome::Cancelled);
        }
        let execution_order = sequential_execution_order(flowsheet)?;
        let total_units = execution_order.len();
        if control
            .checkpoint(SolveProgress {
                stage: SolveStage::Planned,
                total_units: Some(total_units),
                completed_units: 0,
                current_unit: None,
            })
            .is_break()
        {
            return Ok(ControlledSolveOutcome::Cancelled);
        }
        let mut solved_streams = BTreeMap::<StreamId, MaterialStreamState>::new();
        let mut steps = Vec::with_capacity(execution_order.len());
        let mut diagnostics = vec![SolveDiagnostic {
            severity: SolveDiagnosticSeverity::Info,
            code: SolverDiagnosticCode::ExecutionOrder.as_str().to_string(),
            message: format!(
                "resolved acyclic execution order for {} unit(s): [{}]",
                execution_order.len(),
                execution_order
                    .iter()
                    .map(|unit_id| unit_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            related_unit_ids: execution_order.clone(),
            related_stream_ids: Vec::new(),
        }];
        let unit_services = UnitOperationServices {
            thermo: Some(services.thermo),
            flash_solver: Some(services.flash_solver),
        };

        for (step_index, unit_id) in execution_order.iter().enumerate() {
            if control
                .checkpoint(SolveProgress {
                    stage: SolveStage::UnitStarted,
                    total_units: Some(total_units),
                    completed_units: step_index,
                    current_unit: Some(unit_id.clone()),
                })
                .is_break()
            {
                return Ok(ControlledSolveOutcome::Cancelled);
            }
            let step_number = step_index + 1;
            let unit = flowsheet
                .unit(unit_id)
                .map_err(|error| solver_step_lookup_error(step_number, unit_id, error))?;
            let spec = validate_unit_node(unit).map_err(|error| {
                solver_step_error(step_number, unit, SolverDiagnosticCode::StepSpec, error)
            })?;
            let operation = instantiate_operation(unit, flowsheet).map_err(|error| {
                solver_step_error(
                    step_number,
                    unit,
                    SolverDiagnosticCode::StepInstantiation,
                    error,
                )
            })?;
            let mut inputs = UnitOperationInputs::new();
            let mut consumed_streams = Vec::new();

            for port in spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Inlet)
            {
                let stream = resolved_stream_for_port(unit, port.name, &solved_streams).map_err(
                    |error| {
                        solver_step_error(step_number, unit, SolverDiagnosticCode::StepInlet, error)
                    },
                )?;
                consumed_streams.push(stream.clone());
                inputs.insert_material_stream(port.name, stream.clone());
            }
            let consumed_stream_ids = consumed_streams
                .iter()
                .map(|stream| stream.id.clone())
                .collect::<Vec<_>>();

            validate_step_stream_inputs(step_number, unit, &consumed_streams)?;
            validate_step_parameters(step_number, unit, &inputs, &consumed_stream_ids)?;

            let outputs = operation.run(&unit_services, &inputs).map_err(|error| {
                solver_step_execution_error(step_number, unit, &consumed_stream_ids, error)
            })?;
            let mut produced_streams = Vec::new();

            for port in spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Outlet)
            {
                let stream = materialized_output_stream(step_number, unit, port.name, &outputs)?;
                produced_streams.push(stream.clone());
                solved_streams.insert(stream.id.clone(), stream.clone());
            }
            let produced_stream_ids = stream_ids(&produced_streams);

            let summary = format!(
                "executed unit `{}` (`{}`) with {} inlet stream(s) [{}] and produced {} outlet stream(s) [{}]",
                unit.id,
                unit.kind,
                consumed_stream_ids.len(),
                format_stream_ids(&consumed_stream_ids),
                produced_stream_ids.len(),
                format_stream_ids(&produced_stream_ids),
            );
            diagnostics.push(SolveDiagnostic {
                severity: SolveDiagnosticSeverity::Info,
                code: SolverDiagnosticCode::UnitExecuted.as_str().to_string(),
                message: format!("step {}: {}", step_index + 1, summary),
                related_unit_ids: vec![unit.id.clone()],
                related_stream_ids: consumed_stream_ids
                    .iter()
                    .cloned()
                    .chain(produced_stream_ids.iter().cloned())
                    .collect(),
            });
            steps.push(UnitSolveStep {
                index: step_index,
                unit_id: unit.id.clone(),
                unit_name: unit.name.clone(),
                unit_kind: unit.kind.clone(),
                consumed_streams,
                produced_streams,
                summary,
            });
            if control
                .checkpoint(SolveProgress {
                    stage: SolveStage::UnitCompleted,
                    total_units: Some(total_units),
                    completed_units: step_index + 1,
                    current_unit: Some(unit_id.clone()),
                })
                .is_break()
            {
                return Ok(ControlledSolveOutcome::Cancelled);
            }
        }

        if control
            .checkpoint(SolveProgress {
                stage: SolveStage::Completing,
                total_units: Some(total_units),
                completed_units: total_units,
                current_unit: None,
            })
            .is_break()
        {
            return Ok(ControlledSolveOutcome::Cancelled);
        }
        let related_unit_ids = steps
            .iter()
            .map(|step| step.unit_id.clone())
            .collect::<Vec<_>>();
        let summary = SolveDiagnosticSummary {
            highest_severity: SolveDiagnosticSeverity::Info,
            primary_message: format!(
                "solved flowsheet with {} unit(s), {} diagnostic entry(ies), and {} resulting stream(s)",
                steps.len(),
                diagnostics.len(),
                solved_streams.len()
            ),
            diagnostic_count: diagnostics.len(),
            related_unit_ids,
            related_stream_ids: solved_streams.keys().cloned().collect(),
        };

        Ok(ControlledSolveOutcome::Completed(SolveSnapshot {
            status: SolveStatus::Converged,
            summary,
            diagnostics,
            streams: solved_streams,
            steps,
        }))
    }
}

/// Validate canonical connections and the current acyclic SM strategy without solving.
/// Returned errors use the same structured codes and targets as execution.
pub fn sequential_execution_order(flowsheet: &Flowsheet) -> RfResult<Vec<UnitId>> {
    let dependencies = material_dependencies(flowsheet)
        .map_err(|error| solver_stage_error(SolverDiagnosticCode::ConnectionValidation, error))?;
    let connections = dependencies.connections;
    let mut incoming_counts = flowsheet
        .units
        .keys()
        .cloned()
        .map(|unit_id| (unit_id, 0usize))
        .collect::<BTreeMap<_, _>>();
    let downstream_units = dependencies.successors;
    let mut processed_connections = Vec::<rf_flowsheet::MaterialConnection>::new();

    for connection in connections {
        if let Some(ref sink) = connection.sink {
            if connection.source.unit_id == sink.unit_id {
                return Err(solver_topological_self_loop_error(
                    &connection.stream_id,
                    &sink.unit_id,
                    &sink.port_name,
                    &connection.source.port_name,
                ));
            }
            if let Some(error) = detect_two_unit_cycle_error(&connection, &processed_connections) {
                return Err(error);
            }
        }
        processed_connections.push(connection);
    }

    for children in downstream_units.values() {
        for child in children {
            *incoming_counts
                .get_mut(child)
                .expect("validated dependency") += 1;
        }
    }
    let mut ready = VecDeque::from(
        incoming_counts
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(unit_id, _)| unit_id.clone())
            .collect::<Vec<_>>(),
    );
    let mut ordered = Vec::with_capacity(incoming_counts.len());

    while let Some(unit_id) = ready.pop_front() {
        ordered.push(unit_id.clone());

        if let Some(children) = downstream_units.get(&unit_id) {
            for child_id in children {
                let count = incoming_counts.get_mut(child_id).ok_or_else(|| {
                    solver_stage_invalid_input_with_related_units(
                        SolverDiagnosticCode::TopologicalOrdering,
                        format!(
                            "internal solver graph missing incoming count for unit `{child_id}`"
                        ),
                        vec![child_id.clone()],
                    )
                })?;
                *count -= 1;
                if *count == 0 {
                    ready.push_back(child_id.clone());
                }
            }
        }
    }

    if ordered.len() != incoming_counts.len() {
        let unresolved_unit_ids = incoming_counts
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(unit_id, _)| unit_id.clone())
            .collect::<Vec<_>>();
        let unresolved_units = unresolved_unit_ids
            .iter()
            .map(|unit_id| unit_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(solver_stage_invalid_input_with_related_units(
            SolverDiagnosticCode::TopologicalOrdering,
            format!(
                "flowsheet contains a cycle or unresolved dependency involving [{}]; only acyclic sequential flowsheets are supported in the current solver",
                unresolved_units
            ),
            unresolved_unit_ids,
        ));
    }

    Ok(ordered)
}

fn solver_stage_error(code: SolverDiagnosticCode, error: RfError) -> RfError {
    let diagnostic_code = solver_stage_diagnostic_code(code, &error);
    RfError::new(
        error.code(),
        format!(
            "{}: solver {} failed: {}",
            diagnostic_code,
            code.stage_label(),
            error.message()
        ),
    )
    .with_diagnostic_code(diagnostic_code)
    .with_related_unit_ids(error.context().related_unit_ids().to_vec())
    .with_related_stream_ids(error.context().related_stream_ids().to_vec())
    .with_related_port_targets(error.context().related_port_targets().to_vec())
}

fn solver_stage_diagnostic_code(code: SolverDiagnosticCode, error: &RfError) -> String {
    match code {
        SolverDiagnosticCode::ConnectionValidation => {
            map_connection_validation_diagnostic_code(error.context().diagnostic_code())
                .unwrap_or_else(|| code.as_str().to_string())
        }
        _ => code.as_str().to_string(),
    }
}

fn map_connection_validation_diagnostic_code(diagnostic_code: Option<&str>) -> Option<String> {
    let diagnostic_code = diagnostic_code?;
    if diagnostic_code == "flowsheet.connection_validation" {
        return Some(
            SolverDiagnosticCode::ConnectionValidation
                .as_str()
                .to_string(),
        );
    }

    diagnostic_code
        .strip_prefix("flowsheet.connection_validation.")
        .map(|suffix| {
            format!(
                "{}.{}",
                SolverDiagnosticCode::ConnectionValidation.as_str(),
                suffix
            )
        })
}

fn solver_stage_invalid_input_with_related_units(
    code: SolverDiagnosticCode,
    message: impl AsRef<str>,
    related_unit_ids: Vec<UnitId>,
) -> RfError {
    solver_stage_invalid_input_with_context(
        code,
        code.as_str(),
        message,
        related_unit_ids,
        Vec::new(),
        Vec::new(),
    )
}

fn solver_stage_invalid_input_with_context(
    code: SolverDiagnosticCode,
    diagnostic_code: impl Into<String>,
    message: impl AsRef<str>,
    related_unit_ids: Vec<UnitId>,
    related_stream_ids: Vec<StreamId>,
    related_port_targets: Vec<DiagnosticPortTarget>,
) -> RfError {
    let diagnostic_code = diagnostic_code.into();
    RfError::invalid_input(format!(
        "{}: solver {} failed: {}",
        diagnostic_code,
        code.stage_label(),
        message.as_ref()
    ))
    .with_diagnostic_code(diagnostic_code)
    .with_related_unit_ids(related_unit_ids)
    .with_related_stream_ids(related_stream_ids)
    .with_related_port_targets(related_port_targets)
}

fn solver_topological_self_loop_error(
    stream_id: &StreamId,
    unit_id: &UnitId,
    inlet_port_name: &str,
    outlet_port_name: &str,
) -> RfError {
    solver_stage_invalid_input_with_context(
        SolverDiagnosticCode::TopologicalOrdering,
        SOLVER_DIAGNOSTIC_TOPOLOGICAL_SELF_LOOP_CYCLE,
        format!(
            "unit `{}` forms a self loop through stream `{}` between inlet `{}` and outlet `{}`; sequential solver requires acyclic unit dependencies",
            unit_id, stream_id, inlet_port_name, outlet_port_name
        ),
        vec![unit_id.clone()],
        vec![stream_id.clone()],
        vec![
            DiagnosticPortTarget::new(unit_id.clone(), inlet_port_name.to_string()),
            DiagnosticPortTarget::new(unit_id.clone(), outlet_port_name.to_string()),
        ],
    )
}

fn detect_two_unit_cycle_error(
    connection: &rf_flowsheet::MaterialConnection,
    processed_connections: &[rf_flowsheet::MaterialConnection],
) -> Option<RfError> {
    let sink = connection.sink.as_ref()?;
    let reverse = processed_connections.iter().find(|previous| {
        let Some(previous_sink) = previous.sink.as_ref() else {
            return false;
        };
        previous.source.unit_id == sink.unit_id
            && previous_sink.unit_id == connection.source.unit_id
    })?;
    let reverse_sink = reverse
        .sink
        .as_ref()
        .expect("reverse cycle connection should retain sink context");

    Some(solver_topological_two_unit_cycle_error(
        reverse,
        reverse_sink,
        connection,
        sink,
    ))
}

fn solver_topological_two_unit_cycle_error(
    reverse_connection: &rf_flowsheet::MaterialConnection,
    reverse_sink: &rf_flowsheet::MaterialPortRef,
    current_connection: &rf_flowsheet::MaterialConnection,
    current_sink: &rf_flowsheet::MaterialPortRef,
) -> RfError {
    solver_stage_invalid_input_with_context(
        SolverDiagnosticCode::TopologicalOrdering,
        SOLVER_DIAGNOSTIC_TOPOLOGICAL_TWO_UNIT_CYCLE,
        format!(
            "units `{}` and `{}` form a two-unit cycle through streams `{}` and `{}`; `{}.{}` and `{}.{}` currently depend on each other in opposite directions",
            reverse_connection.source.unit_id,
            current_connection.source.unit_id,
            reverse_connection.stream_id,
            current_connection.stream_id,
            reverse_sink.unit_id,
            reverse_sink.port_name,
            current_sink.unit_id,
            current_sink.port_name,
        ),
        vec![
            reverse_connection.source.unit_id.clone(),
            current_connection.source.unit_id.clone(),
        ],
        vec![
            reverse_connection.stream_id.clone(),
            current_connection.stream_id.clone(),
        ],
        vec![
            DiagnosticPortTarget::new(reverse_sink.unit_id.clone(), reverse_sink.port_name.clone()),
            DiagnosticPortTarget::new(current_sink.unit_id.clone(), current_sink.port_name.clone()),
        ],
    )
}

fn solver_step_lookup_error(step_number: usize, unit_id: &UnitId, error: RfError) -> RfError {
    RfError::new(
        error.code(),
        format!(
            "{}: solver step {} {} failed for `{}`: {}",
            SolverDiagnosticCode::StepLookup.as_str(),
            step_number,
            SolverDiagnosticCode::StepLookup.stage_label(),
            unit_id.as_str(),
            error.message()
        ),
    )
    .with_diagnostic_code(SolverDiagnosticCode::StepLookup.as_str())
    .with_related_unit_id(unit_id.clone())
}

fn solver_step_error(
    step_number: usize,
    unit: &UnitNode,
    code: SolverDiagnosticCode,
    error: RfError,
) -> RfError {
    RfError::new(
        error.code(),
        format!(
            "{}: solver step {} {} failed for {}: {}",
            code.as_str(),
            step_number,
            code.stage_label(),
            unit_context(unit),
            error.message()
        ),
    )
    .with_diagnostic_code(code.as_str())
    .with_related_unit_id(unit.id.clone())
    .with_related_stream_ids(error.context().related_stream_ids().to_vec())
    .with_related_port_targets(error.context().related_port_targets().to_vec())
}

fn solver_step_invalid_input(
    step_number: usize,
    unit: &UnitNode,
    code: SolverDiagnosticCode,
    message: impl AsRef<str>,
) -> RfError {
    RfError::invalid_input(format!(
        "{}: solver step {} {} failed for {}: {}",
        code.as_str(),
        step_number,
        code.stage_label(),
        unit_context(unit),
        message.as_ref()
    ))
    .with_diagnostic_code(code.as_str())
    .with_related_unit_id(unit.id.clone())
}

fn solver_step_invalid_input_with_context(
    step_number: usize,
    unit: &UnitNode,
    code: SolverDiagnosticCode,
    message: impl AsRef<str>,
    related_stream_ids: Vec<StreamId>,
    related_port_targets: Vec<DiagnosticPortTarget>,
) -> RfError {
    RfError::invalid_input(format!(
        "{}: solver step {} {} failed for {}: {}",
        code.as_str(),
        step_number,
        code.stage_label(),
        unit_context(unit),
        message.as_ref()
    ))
    .with_diagnostic_code(code.as_str())
    .with_related_unit_id(unit.id.clone())
    .with_related_stream_ids(related_stream_ids)
    .with_related_port_targets(related_port_targets)
}

fn solver_step_execution_error(
    step_number: usize,
    unit: &UnitNode,
    consumed_stream_ids: &[StreamId],
    error: RfError,
) -> RfError {
    RfError::new(
        error.code(),
        format!(
            "{}: solver step {} {} failed for {} after consuming [{}]: {}",
            SolverDiagnosticCode::StepExecution.as_str(),
            step_number,
            SolverDiagnosticCode::StepExecution.stage_label(),
            unit_context(unit),
            format_stream_ids(consumed_stream_ids),
            error.message()
        ),
    )
    .with_diagnostic_code(SolverDiagnosticCode::StepExecution.as_str())
    .with_related_unit_id(unit.id.clone())
    .with_related_stream_ids(consumed_stream_ids.to_vec())
}

fn validate_step_stream_inputs(
    step_number: usize,
    unit: &UnitNode,
    consumed_streams: &[MaterialStreamState],
) -> RfResult<()> {
    use rf_unitops::specifications::{CompositionCheck, CompositionPolicy, check_composition};
    for stream in consumed_streams {
        let message = match check_composition(
            &stream.overall_mole_fractions,
            CompositionPolicy::NonnegativeWeights,
        ) {
            CompositionCheck::Empty => "must define at least one overall mole fraction entry",
            CompositionCheck::InvalidFractions(_) => {
                "overall mole fractions must be finite non-negative values"
            }
            CompositionCheck::InvalidTotal { .. } | CompositionCheck::Unnormalized { .. } => {
                "overall mole fractions must sum to a positive finite value"
            }
            CompositionCheck::Valid { .. } => continue,
        };
        return Err(solver_step_invalid_input_with_context(
            step_number,
            unit,
            SolverDiagnosticCode::StepStreamInput,
            format!(
                "stream `{}` {message} before it can be consumed by `{}`",
                stream.id, unit.id
            ),
            vec![stream.id.clone()],
            inlet_port_targets_for_stream(unit, &stream.id),
        ));
    }
    Ok(())
}

fn inlet_port_targets_for_stream(
    unit: &UnitNode,
    stream_id: &StreamId,
) -> Vec<DiagnosticPortTarget> {
    unit.ports
        .iter()
        .filter(|port| port.direction == rf_types::PortDirection::Inlet)
        .filter(|port| port.stream_id.as_ref() == Some(stream_id))
        .map(|port| DiagnosticPortTarget::new(unit.id.clone(), port.name.clone()))
        .collect()
}

fn stream_ids(streams: &[MaterialStreamState]) -> Vec<StreamId> {
    streams.iter().map(|stream| stream.id.clone()).collect()
}

fn dedupe_stream_ids(stream_ids: Vec<StreamId>) -> Vec<StreamId> {
    let mut seen = BTreeSet::new();
    stream_ids
        .into_iter()
        .filter(|stream_id| seen.insert(stream_id.clone()))
        .collect()
}

fn unit_context(unit: &UnitNode) -> String {
    format!("unit `{}` (`{}`)", unit.id, unit.kind)
}

fn unit_port_context(unit: &UnitNode, port_name: &str) -> String {
    format!("{} port `{}`", unit_context(unit), port_name)
}

fn instantiate_operation(
    unit: &UnitNode,
    flowsheet: &Flowsheet,
) -> RfResult<Box<dyn UnitOperation>> {
    if BuiltinUnitKind::parse(&unit.kind).is_none() {
        return Err(RfError::invalid_input(format!(
            "{} uses unsupported solver kind `{}`",
            unit_context(unit),
            unit.kind,
        ))
        .with_related_unit_id(unit.id.clone()));
    }
    rf_unitops::catalog::instantiate_builtin(unit, &flowsheet.streams)
}

fn resolved_stream_for_port<'a>(
    unit: &UnitNode,
    port_name: &str,
    solved_streams: &'a BTreeMap<StreamId, MaterialStreamState>,
) -> RfResult<&'a MaterialStreamState> {
    let stream_id = port_stream_id(unit, port_name)?;
    solved_streams.get(stream_id).ok_or_else(|| {
        RfError::invalid_input(format!(
            "{} requires inlet stream `{}` to be solved before this step",
            unit_port_context(unit, port_name),
            stream_id
        ))
    })
}

fn materialized_output_stream<'a>(
    step_number: usize,
    unit: &UnitNode,
    port_name: &str,
    outputs: &'a rf_unitops::UnitOperationOutputs,
) -> RfResult<&'a MaterialStreamState> {
    outputs.stream(port_name).ok_or_else(|| {
        solver_step_invalid_input(
            step_number,
            unit,
            SolverDiagnosticCode::StepMaterialization,
            format!("missing produced outlet port `{port_name}`"),
        )
    })
}

fn port_stream_id<'a>(unit: &'a UnitNode, port_name: &str) -> RfResult<&'a StreamId> {
    let port = find_port(unit, port_name)?;
    port.stream_id.as_ref().ok_or_else(|| {
        RfError::invalid_input(format!(
            "{} is missing a connected stream id",
            unit_port_context(unit, &port.name)
        ))
    })
}

fn find_port<'a>(unit: &'a UnitNode, port_name: &str) -> RfResult<&'a UnitPort> {
    unit.ports
        .iter()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "{} does not define expected port `{port_name}`",
                unit_context(unit)
            ))
        })
}

fn format_stream_ids(stream_ids: &[StreamId]) -> String {
    if stream_ids.is_empty() {
        return "<none>".to_owned();
    }

    stream_ids
        .iter()
        .map(|stream_id| stream_id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn prefixed_solver_diagnostic_code(message: &str) -> Option<String> {
    message
        .split(": ")
        .find(|segment| is_solver_diagnostic_code(segment))
        .map(str::to_string)
}

fn is_solver_diagnostic_code(candidate: &str) -> bool {
    candidate.starts_with("solver.")
        && candidate.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'_'
        })
}

fn related_unit_ids_from_failure_message(message: &str) -> Vec<UnitId> {
    let mut unit_ids = Vec::new();
    collect_unit_context_ids(message, &mut unit_ids);
    collect_step_lookup_ids(message, &mut unit_ids);
    collect_cycle_unit_ids(message, &mut unit_ids);
    unit_ids
}

fn related_stream_ids_from_failure_message(message: &str) -> Vec<StreamId> {
    let mut stream_ids = Vec::new();
    collect_stream_context_ids(message, &mut stream_ids);
    collect_topological_stream_ids(message, &mut stream_ids);
    stream_ids
}

fn collect_unit_context_ids(message: &str, unit_ids: &mut Vec<UnitId>) {
    let needle = "unit `";
    let mut remaining = message;

    while let Some(start) = remaining.find(needle) {
        let after = &remaining[start + needle.len()..];
        let Some(end) = after.find("` (`") else {
            break;
        };
        push_related_unit_id(unit_ids, &after[..end]);
        remaining = &after[end + 1..];
    }
}

fn collect_step_lookup_ids(message: &str, unit_ids: &mut Vec<UnitId>) {
    let Some(code) = prefixed_solver_diagnostic_code(message) else {
        return;
    };
    if code != SolverDiagnosticCode::StepLookup.as_str() {
        return;
    }

    let needle = "failed for `";
    let mut remaining = message;
    while let Some(start) = remaining.find(needle) {
        let after = &remaining[start + needle.len()..];
        let Some(end) = after.find("`:") else {
            break;
        };
        let candidate = &after[..end];
        if !candidate.contains(' ') {
            push_related_unit_id(unit_ids, candidate);
        }
        remaining = &after[end + 2..];
    }
}

fn collect_cycle_unit_ids(message: &str, unit_ids: &mut Vec<UnitId>) {
    let Some(code) = prefixed_solver_diagnostic_code(message) else {
        return;
    };
    if !code.starts_with(SolverDiagnosticCode::TopologicalOrdering.as_str()) {
        return;
    }

    if let Some(start) = message.find("involving [") {
        let after = &message[start + "involving [".len()..];
        if let Some(end) = after.find(']') {
            for candidate in after[..end].split(',') {
                push_related_unit_id(unit_ids, candidate.trim());
            }
        }
    }

    collect_backticked_topological_units(message, unit_ids);
}

fn collect_backticked_topological_units(message: &str, unit_ids: &mut Vec<UnitId>) {
    let needle = "unit";
    let mut remaining = message;

    while let Some(start) = remaining.find(needle) {
        let after_keyword = &remaining[start + needle.len()..];
        let after_keyword = after_keyword.trim_start();
        if !after_keyword.starts_with('`') && !after_keyword.starts_with('s') {
            remaining = after_keyword;
            continue;
        }

        let mut cursor = after_keyword;
        if cursor.starts_with('s') {
            cursor = cursor[1..].trim_start();
        }

        while let Some(rest) = cursor.strip_prefix('`') {
            let Some(end) = rest.find('`') else {
                return;
            };
            push_related_unit_id(unit_ids, &rest[..end]);
            cursor = rest[end + 1..].trim_start();
            if let Some(next) = cursor.strip_prefix("and") {
                cursor = next.trim_start();
                continue;
            }
            break;
        }

        remaining = cursor;
    }
}

fn collect_stream_context_ids(message: &str, stream_ids: &mut Vec<StreamId>) {
    let needle = "stream `";
    let mut remaining = message;

    while let Some(start) = remaining.find(needle) {
        let after = &remaining[start + needle.len()..];
        let Some(end) = after.find('`') else {
            break;
        };
        let candidate = &after[..end];
        if !candidate.is_empty() {
            let stream_id = StreamId::new(candidate);
            if !stream_ids.iter().any(|existing| existing == &stream_id) {
                stream_ids.push(stream_id);
            }
        }
        remaining = &after[end + 1..];
    }
}

fn collect_topological_stream_ids(message: &str, stream_ids: &mut Vec<StreamId>) {
    let Some(code) = prefixed_solver_diagnostic_code(message) else {
        return;
    };
    if !code.starts_with(SolverDiagnosticCode::TopologicalOrdering.as_str()) {
        return;
    }

    let needle = "stream";
    let mut remaining = message;

    while let Some(start) = remaining.find(needle) {
        let after_keyword = &remaining[start + needle.len()..];
        let after_keyword = after_keyword.trim_start();
        if !after_keyword.starts_with('`') && !after_keyword.starts_with('s') {
            remaining = after_keyword;
            continue;
        }

        let mut cursor = after_keyword;
        if cursor.starts_with('s') {
            cursor = cursor[1..].trim_start();
        }

        while let Some(rest) = cursor.strip_prefix('`') {
            let Some(end) = rest.find('`') else {
                return;
            };
            push_related_stream_id(stream_ids, &rest[..end]);
            cursor = rest[end + 1..].trim_start();
            if let Some(next) = cursor.strip_prefix("and") {
                cursor = next.trim_start();
                continue;
            }
            break;
        }

        remaining = cursor;
    }
}

fn push_related_unit_id(unit_ids: &mut Vec<UnitId>, candidate: &str) {
    if candidate.is_empty() || unit_ids.iter().any(|unit_id| unit_id.as_str() == candidate) {
        return;
    }
    unit_ids.push(UnitId::new(candidate));
}

fn push_related_stream_id(stream_ids: &mut Vec<StreamId>, candidate: &str) {
    if candidate.is_empty()
        || stream_ids
            .iter()
            .any(|stream_id| stream_id.as_str() == candidate)
    {
        return;
    }
    stream_ids.push(StreamId::new(candidate));
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_fixtures;
