use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rf_flash::TpFlashSolver;
use rf_flowsheet::validate_connections;
use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_thermo::ThermoProvider;
use rf_types::{DiagnosticPortTarget, PortDirection, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{
    BuiltinUnitKind, COOLER_KIND, FEED_KIND, FEED_OUTLET_PORT, FLASH_DRUM_KIND,
    FLASH_DRUM_LIQUID_PORT, FLASH_DRUM_VAPOR_PORT, Feed, FlashDrum, HEATER_COOLER_OUTLET_PORT,
    HEATER_KIND, HeaterCooler, MIXER_KIND, MIXER_OUTLET_PORT, Mixer, StreamTarget, UnitOperation,
    UnitOperationInputs, UnitOperationServices, VALVE_KIND, Valve, validate_unit_node,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSolveStep {
    pub index: usize,
    pub unit_id: UnitId,
    pub unit_name: String,
    pub unit_kind: String,
    pub consumed_stream_ids: Vec<StreamId>,
    pub produced_stream_ids: Vec<StreamId>,
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
        let execution_order = topological_unit_order(flowsheet)?;
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
            let mut consumed_stream_ids = Vec::new();

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
                consumed_stream_ids.push(stream.id.clone());
                inputs.insert_material_stream(port.name, stream.clone());
            }

            let outputs = operation.run(&unit_services, &inputs).map_err(|error| {
                solver_step_execution_error(step_number, unit, &consumed_stream_ids, error)
            })?;
            let mut produced_stream_ids = Vec::new();

            for port in spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Outlet)
            {
                let stream = materialized_output_stream(step_number, unit, port.name, &outputs)?;
                produced_stream_ids.push(stream.id.clone());
                solved_streams.insert(stream.id.clone(), stream.clone());
            }

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
                consumed_stream_ids,
                produced_stream_ids,
                summary,
            });
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

        Ok(SolveSnapshot {
            status: SolveStatus::Converged,
            summary,
            diagnostics,
            streams: solved_streams,
            steps,
        })
    }
}

fn topological_unit_order(flowsheet: &Flowsheet) -> RfResult<Vec<UnitId>> {
    let connections = validate_connections(flowsheet)
        .map_err(|error| solver_stage_error(SolverDiagnosticCode::ConnectionValidation, error))?;
    let mut incoming_counts = flowsheet
        .units
        .keys()
        .cloned()
        .map(|unit_id| (unit_id, 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut downstream_units = BTreeMap::<UnitId, BTreeSet<UnitId>>::new();
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
            downstream_units
                .entry(connection.source.unit_id.clone())
                .or_default()
                .insert(sink.unit_id.clone());
            *incoming_counts.entry(sink.unit_id.clone()).or_insert(0) += 1;
        }
        processed_connections.push(connection);
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

fn solver_context_error(context: impl AsRef<str>, error: RfError) -> RfError {
    let mut wrapped = RfError::new(
        error.code(),
        format!("{}: {}", context.as_ref(), error.message()),
    )
    .with_related_unit_ids(error.context().related_unit_ids().to_vec())
    .with_related_stream_ids(error.context().related_stream_ids().to_vec())
    .with_related_port_targets(error.context().related_port_targets().to_vec());
    if let Some(code) = error.context().diagnostic_code() {
        wrapped = wrapped.with_diagnostic_code(code.to_string());
    }
    wrapped
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
    match unit.kind.as_str() {
        FEED_KIND => {
            let outlet = stream_for_port(unit, FEED_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(Feed::new(outlet.clone())))
        }
        MIXER_KIND => {
            let outlet = stream_target_for_port(unit, MIXER_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(Mixer::new(outlet)))
        }
        HEATER_KIND => {
            let outlet = stream_for_port(unit, HEATER_COOLER_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(HeaterCooler::new(
                BuiltinUnitKind::Heater,
                outlet.clone(),
            )?))
        }
        COOLER_KIND => {
            let outlet = stream_for_port(unit, HEATER_COOLER_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(HeaterCooler::new(
                BuiltinUnitKind::Cooler,
                outlet.clone(),
            )?))
        }
        VALVE_KIND => {
            let outlet = stream_for_port(unit, HEATER_COOLER_OUTLET_PORT, flowsheet)?;
            Ok(Box::new(Valve::new(outlet.clone())))
        }
        FLASH_DRUM_KIND => {
            let liquid = stream_target_for_port(unit, FLASH_DRUM_LIQUID_PORT, flowsheet)?;
            let vapor = stream_target_for_port(unit, FLASH_DRUM_VAPOR_PORT, flowsheet)?;
            Ok(Box::new(FlashDrum::new(liquid, vapor)))
        }
        _ => Err(RfError::invalid_input(format!(
            "{} uses unsupported solver kind `{}`",
            unit_context(unit),
            unit.kind
        ))
        .with_related_unit_id(unit.id.clone())),
    }
}

fn stream_for_port<'a>(
    unit: &UnitNode,
    port_name: &str,
    flowsheet: &'a Flowsheet,
) -> RfResult<&'a MaterialStreamState> {
    let stream_id = port_stream_id(unit, port_name)?;
    flowsheet.stream(stream_id).map_err(|error| {
        solver_context_error(
            format!(
                "{} references missing stream `{}`",
                unit_port_context(unit, port_name),
                stream_id
            ),
            error,
        )
    })
}

fn stream_target_for_port(
    unit: &UnitNode,
    port_name: &str,
    flowsheet: &Flowsheet,
) -> RfResult<StreamTarget> {
    let stream = stream_for_port(unit, port_name, flowsheet)?;
    Ok(StreamTarget::new(stream.id.clone(), stream.name.clone()))
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
mod tests {
    use std::collections::BTreeMap;

    use rf_flash::PlaceholderTpFlashSolver;
    use rf_model::{Component, Composition, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
    use rf_store::parse_project_file_json;
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::{
        ComponentId, DiagnosticPortTarget, PhaseLabel, PortDirection, PortKind, RfError, StreamId,
        UnitId,
    };
    use rf_unitops::{
        UnitOperationOutputs, build_cooler_node, build_feed_node, build_flash_drum_node,
        build_heater_node, build_mixer_node, build_valve_node,
    };

    use super::{
        FlowsheetSolver, SequentialModularSolver, SolveDiagnosticSeverity, SolveFailureContext,
        SolveStatus, SolverDiagnosticCode, SolverServices, find_port, instantiate_operation,
        materialized_output_stream, port_stream_id, resolved_stream_for_port, solver_step_error,
        solver_step_execution_error, solver_step_lookup_error, stream_for_port,
    };

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
        );
    }

    fn binary_composition(first: f64, second: f64) -> Composition {
        [
            (ComponentId::new("component-a"), first),
            (ComponentId::new("component-b"), second),
        ]
        .into_iter()
        .collect()
    }

    fn build_stream(
        id: &str,
        name: &str,
        temperature_k: f64,
        pressure_pa: f64,
        total_molar_flow_mol_s: f64,
        composition: Composition,
    ) -> MaterialStreamState {
        MaterialStreamState::from_tpzf(
            id,
            name,
            temperature_k,
            pressure_pa,
            total_molar_flow_mol_s,
            composition,
        )
    }

    fn build_provider() -> PlaceholderThermoProvider {
        let pressure_pa = 100_000.0_f64;
        let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(AntoineCoefficients::new(
            ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(AntoineCoefficients::new(
            ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
    }

    fn build_demo_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-mixer-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed-a",
                "Feed A",
                300.0,
                120_000.0,
                2.0,
                binary_composition(0.25, 0.75),
            ),
            build_stream(
                "stream-feed-b",
                "Feed B",
                360.0,
                100_000.0,
                3.0,
                binary_composition(0.60, 0.40),
            ),
            build_stream(
                "stream-mix-out",
                "Mixer Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-a", "Feed A", "stream-feed-a"),
            build_feed_node("feed-b", "Feed B", "stream-feed-b"),
            build_mixer_node(
                "mixer-1",
                "Mixer",
                "stream-feed-a",
                "stream-feed-b",
                "stream-mix-out",
            ),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-mix-out",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    fn build_feed_heater_flash_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-heater-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed",
                "Feed",
                300.0,
                120_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-heated",
                "Heated Outlet",
                345.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                345.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                345.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-1", "Feed", "stream-feed"),
            build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated"),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-heated",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    fn build_feed_mixer_heater_flash_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-mixer-heater-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed-a",
                "Feed A",
                300.0,
                120_000.0,
                2.0,
                binary_composition(0.25, 0.75),
            ),
            build_stream(
                "stream-feed-b",
                "Feed B",
                360.0,
                100_000.0,
                3.0,
                binary_composition(0.60, 0.40),
            ),
            build_stream(
                "stream-mix-out",
                "Mixer Outlet",
                330.0,
                100_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-heated",
                "Heated Outlet",
                350.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                350.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                350.0,
                95_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-a", "Feed A", "stream-feed-a"),
            build_feed_node("feed-b", "Feed B", "stream-feed-b"),
            build_mixer_node(
                "mixer-1",
                "Mixer",
                "stream-feed-a",
                "stream-feed-b",
                "stream-mix-out",
            ),
            build_heater_node("heater-1", "Heater", "stream-mix-out", "stream-heated"),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-heated",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    fn build_feed_valve_flash_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-valve-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed",
                "Feed",
                315.0,
                120_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-throttled",
                "Valve Outlet",
                300.0,
                90_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                315.0,
                90_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                315.0,
                90_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-1", "Feed", "stream-feed"),
            build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled"),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-throttled",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    fn build_feed_cooler_flash_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("feed-cooler-flash");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }

        for stream in [
            build_stream(
                "stream-feed",
                "Feed",
                360.0,
                120_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-cooled",
                "Cooled Outlet",
                305.0,
                98_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-liquid",
                "Liquid Outlet",
                305.0,
                98_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
            build_stream(
                "stream-vapor",
                "Vapor Outlet",
                305.0,
                98_000.0,
                0.0,
                binary_composition(0.5, 0.5),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }

        for unit in [
            build_feed_node("feed-1", "Feed", "stream-feed"),
            build_cooler_node("cooler-1", "Cooler", "stream-feed", "stream-cooled"),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-cooled",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        flowsheet
    }

    #[test]
    fn sequential_solver_solves_feed_mixer_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_demo_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(
            snapshot.summary.highest_severity,
            SolveDiagnosticSeverity::Info
        );
        assert_eq!(snapshot.summary.diagnostic_count, 5);
        assert_eq!(snapshot.steps.len(), 4);
        assert_eq!(snapshot.steps[0].index, 0);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-a");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "feed-b");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "mixer-1");
        assert_eq!(snapshot.steps[3].unit_id.as_str(), "flash-1");
        assert_eq!(snapshot.steps[2].consumed_stream_ids.len(), 2);
        assert!(
            snapshot.steps[2]
                .summary
                .contains("produced 1 outlet stream")
        );
        assert_eq!(snapshot.diagnostics[0].code, "solver.execution_order");
        assert_eq!(snapshot.diagnostics[1].code, "solver.unit_executed");

        let mixer_out = snapshot
            .stream(&"stream-mix-out".into())
            .expect("expected mixer outlet");
        assert_close(mixer_out.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(mixer_out.temperature_k, 336.0, 1e-12);
        assert_close(
            *mixer_out
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.46,
            1e-12,
        );

        let liquid = snapshot
            .stream(&"stream-liquid".into())
            .expect("expected liquid outlet");
        let vapor = snapshot
            .stream(&"stream-vapor".into())
            .expect("expected vapor outlet");
        assert_close(liquid.total_molar_flow_mol_s, 3.099999999994907, 1e-9);
        assert_close(vapor.total_molar_flow_mol_s, 1.900000000005093, 1e-9);
        assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
        assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    }

    #[test]
    fn sequential_solver_runs_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-mixer-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.summary.related_unit_ids.len(), 4);
        assert_eq!(snapshot.steps.len(), 4);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }

    #[test]
    fn sequential_solver_solves_feed_heater_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_feed_heater_flash_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.summary.diagnostic_count, 4);
        assert_eq!(snapshot.steps.len(), 3);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "heater-1");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
        assert_eq!(
            snapshot.steps[1].consumed_stream_ids,
            vec!["stream-feed".into()]
        );
        assert!(snapshot.steps[1].summary.contains("heater-1"));

        let heated = snapshot
            .stream(&"stream-heated".into())
            .expect("expected heated outlet");
        assert_close(heated.temperature_k, 345.0, 1e-12);
        assert_close(heated.pressure_pa, 95_000.0, 1e-12);
        assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *heated
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.35,
            1e-12,
        );

        let liquid = snapshot
            .stream(&"stream-liquid".into())
            .expect("expected liquid outlet");
        let vapor = snapshot
            .stream(&"stream-vapor".into())
            .expect("expected vapor outlet");
        assert!(liquid.total_molar_flow_mol_s > 0.0);
        assert!(vapor.total_molar_flow_mol_s > 0.0);
        assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
        assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    }

    #[test]
    fn sequential_solver_runs_feed_heater_flash_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        let heated = snapshot
            .stream(&"stream-heated".into())
            .expect("expected heated outlet");
        assert_close(heated.temperature_k, 345.0, 1e-12);
        assert_close(heated.pressure_pa, 95_000.0, 1e-12);
        assert_eq!(snapshot.steps.len(), 3);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }

    #[test]
    fn sequential_solver_solves_feed_mixer_heater_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_feed_mixer_heater_flash_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.summary.diagnostic_count, 6);
        assert_eq!(snapshot.steps.len(), 5);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-a");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "feed-b");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "mixer-1");
        assert_eq!(snapshot.steps[3].unit_id.as_str(), "heater-1");
        assert_eq!(snapshot.steps[4].unit_id.as_str(), "flash-1");

        let mixed = snapshot
            .stream(&"stream-mix-out".into())
            .expect("expected mixer outlet");
        assert_close(mixed.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(mixed.temperature_k, 336.0, 1e-12);
        assert_close(
            *mixed
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.46,
            1e-12,
        );

        let heated = snapshot
            .stream(&"stream-heated".into())
            .expect("expected heated outlet");
        assert_close(heated.temperature_k, 350.0, 1e-12);
        assert_close(heated.pressure_pa, 95_000.0, 1e-12);
        assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *heated
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.46,
            1e-12,
        );
    }

    #[test]
    fn sequential_solver_runs_feed_mixer_heater_flash_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-mixer-heater-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        let heated = snapshot
            .stream(&"stream-heated".into())
            .expect("expected heated outlet");
        assert_close(heated.temperature_k, 350.0, 1e-12);
        assert_close(heated.pressure_pa, 95_000.0, 1e-12);
        assert_eq!(snapshot.steps.len(), 5);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }

    #[test]
    fn sequential_solver_solves_feed_cooler_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_feed_cooler_flash_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.summary.diagnostic_count, 4);
        assert_eq!(snapshot.steps.len(), 3);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "cooler-1");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
        assert_eq!(
            snapshot.steps[1].consumed_stream_ids,
            vec!["stream-feed".into()]
        );
        assert!(snapshot.steps[1].summary.contains("cooler-1"));

        let cooled = snapshot
            .stream(&"stream-cooled".into())
            .expect("expected cooled outlet");
        assert_close(cooled.temperature_k, 305.0, 1e-12);
        assert_close(cooled.pressure_pa, 98_000.0, 1e-12);
        assert_close(cooled.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *cooled
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.35,
            1e-12,
        );

        let liquid = snapshot
            .stream(&"stream-liquid".into())
            .expect("expected liquid outlet");
        let vapor = snapshot
            .stream(&"stream-vapor".into())
            .expect("expected vapor outlet");
        assert!(liquid.total_molar_flow_mol_s > 0.0);
        assert!(vapor.total_molar_flow_mol_s > 0.0);
        assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
        assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
    }

    #[test]
    fn sequential_solver_runs_feed_cooler_flash_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-cooler-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        let cooled = snapshot
            .stream(&"stream-cooled".into())
            .expect("expected cooled outlet");
        assert_close(cooled.temperature_k, 305.0, 1e-12);
        assert_close(cooled.pressure_pa, 98_000.0, 1e-12);
        assert_eq!(snapshot.steps.len(), 3);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }

    #[test]
    fn sequential_solver_solves_feed_valve_flash_chain() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };

        let snapshot = SequentialModularSolver
            .solve(&services, &build_feed_valve_flash_flowsheet())
            .expect("expected solve snapshot");

        assert_eq!(snapshot.status, SolveStatus::Converged);
        assert_eq!(snapshot.summary.diagnostic_count, 4);
        assert_eq!(snapshot.steps.len(), 3);
        assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
        assert_eq!(snapshot.steps[1].unit_id.as_str(), "valve-1");
        assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
        assert_eq!(
            snapshot.steps[1].consumed_stream_ids,
            vec!["stream-feed".into()]
        );
        assert!(snapshot.steps[1].summary.contains("valve-1"));

        let throttled = snapshot
            .stream(&"stream-throttled".into())
            .expect("expected valve outlet");
        assert_close(throttled.temperature_k, 315.0, 1e-12);
        assert_close(throttled.pressure_pa, 90_000.0, 1e-12);
        assert_close(throttled.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *throttled
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.35,
            1e-12,
        );

        let liquid = snapshot
            .stream(&"stream-liquid".into())
            .expect("expected liquid outlet");
        let vapor = snapshot
            .stream(&"stream-vapor".into())
            .expect("expected vapor outlet");
        assert!(liquid.total_molar_flow_mol_s > 0.0);
        assert!(vapor.total_molar_flow_mol_s > 0.0);
    }

    #[test]
    fn sequential_solver_runs_feed_valve_flash_example_project_file() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
        ))
        .expect("expected example project parse");

        let snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        let throttled = snapshot
            .stream(&"stream-throttled".into())
            .expect("expected valve outlet");
        assert_close(throttled.temperature_k, 315.0, 1e-12);
        assert_close(throttled.pressure_pa, 90_000.0, 1e-12);
        assert_eq!(snapshot.steps.len(), 3);
        assert!(snapshot.stream(&"stream-liquid".into()).is_some());
        assert!(snapshot.stream(&"stream-vapor".into()).is_some());
    }

    #[test]
    fn sequential_solver_reports_step_context_for_unit_execution_failures() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = build_feed_valve_flash_flowsheet();
        flowsheet
            .streams
            .get_mut(&"stream-throttled".into())
            .expect("expected throttled stream")
            .pressure_pa = 130_000.0;

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected valve execution failure");

        assert!(error.message().contains("solver.step.execution:"));
        assert!(
            error
                .message()
                .contains("step 2 unit execution failed for unit `valve-1` (`valve`)")
        );
        assert!(error.message().contains("after consuming [stream-feed]"));
    }

    #[test]
    fn sequential_solver_reports_connection_validation_stage_context() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = build_feed_valve_flash_flowsheet();
        flowsheet
            .units
            .get_mut(&"flash-1".into())
            .expect("expected flash unit")
            .ports
            .iter_mut()
            .find(|port| port.name == "inlet")
            .expect("expected inlet port")
            .stream_id = Some("stream-feed".into());

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected connection validation failure");

        assert_eq!(error.code().as_str(), "invalid_connection");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.connection_validation.duplicate_downstream_sink")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            &[UnitId::new("flash-1"), UnitId::new("valve-1")]
        );
        assert_eq!(
            error.context().related_stream_ids(),
            &[StreamId::new("stream-feed")]
        );
        assert_eq!(
            error.context().related_port_targets(),
            &[
                DiagnosticPortTarget::new("flash-1", "inlet"),
                DiagnosticPortTarget::new("valve-1", "inlet"),
            ]
        );
        assert!(
            error
                .message()
                .contains("solver connection validation failed")
        );
        assert!(error.message().contains("consumed by both"));
    }

    #[test]
    fn sequential_solver_reports_unsupported_unit_kind_during_connection_validation() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = Flowsheet::new("unsupported-unit-kind");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }
        flowsheet
            .insert_stream(build_stream(
                "stream-feed",
                "Feed",
                320.0,
                100_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ))
            .expect("expected stream insert");
        flowsheet
            .insert_unit(UnitNode::new(
                "mystery-1",
                "Mystery Unit",
                "pump",
                vec![UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                )],
            ))
            .expect("expected unit insert");

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected unsupported unit kind failure");

        assert_eq!(error.code().as_str(), "invalid_connection");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.connection_validation.unsupported_unit_kind")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            &[UnitId::new("mystery-1")]
        );
        assert!(
            error
                .message()
                .contains("solver.connection_validation.unsupported_unit_kind: solver connection validation failed")
        );
        assert!(
            error
                .message()
                .contains("canonical built-in port signature")
        );
        assert!(error.message().contains("unsupported kind `pump`"));
    }

    #[test]
    fn sequential_solver_reports_topological_ordering_stage_context() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = Flowsheet::new("heater-valve-cycle");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }
        for stream in [
            build_stream(
                "stream-a",
                "Cycle Stream A",
                320.0,
                100_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-b",
                "Cycle Stream B",
                300.0,
                95_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }
        for unit in [
            build_heater_node("heater-1", "Heater", "stream-b", "stream-a"),
            build_valve_node("valve-1", "Valve", "stream-a", "stream-b"),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected cycle detection failure");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.topological_ordering.two_unit_cycle")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            &[UnitId::new("heater-1"), UnitId::new("valve-1")]
        );
        assert_eq!(
            error.context().related_stream_ids(),
            &[StreamId::new("stream-a"), StreamId::new("stream-b")]
        );
        assert_eq!(
            error.context().related_port_targets(),
            &[
                DiagnosticPortTarget::new("valve-1", "inlet"),
                DiagnosticPortTarget::new("heater-1", "inlet"),
            ]
        );
        assert!(
            error
                .message()
                .contains("solver topological ordering failed")
        );
        assert!(error.message().contains("form a two-unit cycle"));
        assert!(
            error
                .message()
                .contains("streams `stream-a` and `stream-b`")
        );
    }

    #[test]
    fn sequential_solver_reports_self_loop_as_topological_cycle() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = Flowsheet::new("self-loop");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }
        for stream in [
            build_stream(
                "stream-loop",
                "Loop Stream",
                320.0,
                100_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-out",
                "Outlet Stream",
                300.0,
                95_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }
        flowsheet
            .insert_unit(build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-loop",
                "stream-loop",
                "stream-out",
            ))
            .expect("expected unit insert");

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected self-loop cycle failure");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.topological_ordering.self_loop_cycle")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            &[UnitId::new("flash-1")]
        );
        assert_eq!(
            error.context().related_stream_ids(),
            &[StreamId::new("stream-loop")]
        );
        assert_eq!(
            error.context().related_port_targets(),
            &[
                DiagnosticPortTarget::new("flash-1", "inlet"),
                DiagnosticPortTarget::new("flash-1", "liquid"),
            ]
        );
        assert!(
            error
                .message()
                .contains("solver topological ordering failed")
        );
        assert!(error.message().contains("forms a self loop"));
        assert!(error.message().contains("stream `stream-loop`"));
    }

    #[test]
    fn instantiate_operation_reports_unit_context_for_unsupported_kind() {
        let flowsheet = Flowsheet::new("unsupported-kind-helper");
        let unit = UnitNode::new("mystery-1", "Mystery Unit", "pump", Vec::new());

        let error = match instantiate_operation(&unit, &flowsheet) {
            Ok(_) => panic!("expected unsupported kind"),
            Err(error) => error,
        };

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("unit `mystery-1` (`pump`) uses unsupported solver kind `pump`")
        );
    }

    #[test]
    fn find_port_reports_unit_context_for_missing_port() {
        let unit = build_feed_node("feed-1", "Feed", "stream-feed");

        let error = find_port(&unit, "missing-port").expect_err("expected missing port");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("unit `feed-1` (`feed`) does not define expected port `missing-port`")
        );
    }

    #[test]
    fn port_stream_id_reports_unit_port_context_for_missing_stream_id() {
        let unit = UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![UnitPort::new(
                "outlet",
                PortDirection::Outlet,
                PortKind::Material,
                None,
            )],
        );

        let error =
            port_stream_id(&unit, "outlet").expect_err("expected missing connected stream id");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error.message().contains(
                "unit `heater-1` (`heater`) port `outlet` is missing a connected stream id"
            )
        );
    }

    #[test]
    fn stream_for_port_reports_unit_port_context_for_missing_stream_reference() {
        let mut flowsheet = Flowsheet::new("missing-stream-helper");
        let unit = build_feed_node("feed-1", "Feed", "stream-missing");

        flowsheet
            .insert_unit(unit.clone())
            .expect("expected unit insert");

        let error =
            stream_for_port(&unit, "outlet", &flowsheet).expect_err("expected missing stream");

        assert_eq!(error.code().as_str(), "missing_entity");
        assert!(error.message().contains(
            "unit `feed-1` (`feed`) port `outlet` references missing stream `stream-missing`"
        ));
    }

    #[test]
    fn resolved_stream_for_port_reports_unit_port_context_for_unsolved_inlet() {
        let unit = build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated");
        let solved_streams = BTreeMap::new();

        let error = resolved_stream_for_port(&unit, "inlet", &solved_streams)
            .expect_err("expected unsolved inlet stream");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("unit `heater-1` (`heater`) port `inlet` requires inlet stream `stream-feed` to be solved before this step")
        );
    }

    #[test]
    fn solver_step_lookup_error_uses_stable_template() {
        let error = solver_step_lookup_error(
            3,
            &UnitId::new("flash-1"),
            RfError::missing_entity("unit", "flash-1"),
        );

        assert_eq!(error.code().as_str(), "missing_entity");
        assert_eq!(
            error.message(),
            "solver.step.lookup: solver step 3 unit lookup failed for `flash-1`: missing unit `flash-1`"
        );
    }

    #[test]
    fn solver_step_error_uses_stable_template() {
        let unit = build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated");
        let error = solver_step_error(
            2,
            &unit,
            SolverDiagnosticCode::StepSpec,
            RfError::invalid_input("port mismatch"),
        );

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.message(),
            "solver.step.spec: solver step 2 unit spec validation failed for unit `heater-1` (`heater`): port mismatch"
        );
    }

    #[test]
    fn solver_step_execution_error_uses_stable_code_and_template() {
        let unit = build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled");
        let error = solver_step_execution_error(
            2,
            &unit,
            &[StreamId::new("stream-feed")],
            RfError::invalid_input("valve outlet pressure cannot exceed inlet pressure"),
        );

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.message(),
            "solver.step.execution: solver step 2 unit execution failed for unit `valve-1` (`valve`) after consuming [stream-feed]: valve outlet pressure cannot exceed inlet pressure"
        );
        assert_eq!(
            error.context().related_stream_ids(),
            &[StreamId::new("stream-feed")]
        );
    }

    #[test]
    fn materialized_output_stream_reports_step_context_for_missing_port() {
        let unit = build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-feed",
            "stream-liquid",
            "stream-vapor",
        );
        let outputs = UnitOperationOutputs::new();

        let error = materialized_output_stream(4, &unit, "liquid", &outputs)
            .expect_err("expected missing produced outlet port");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.message(),
            "solver.step.materialization: solver step 4 output materialization failed for unit `flash-1` (`flash_drum`): missing produced outlet port `liquid`"
        );
    }

    #[test]
    fn solve_failure_context_extracts_step_execution_code_and_unit() {
        let error = solver_step_execution_error(
            2,
            &build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled"),
            &[StreamId::new("stream-feed")],
            RfError::invalid_input("valve outlet pressure cannot exceed inlet pressure"),
        );

        let context = SolveFailureContext::from_error(&error);

        assert_eq!(
            context.primary_code.as_deref(),
            Some("solver.step.execution")
        );
        assert_eq!(context.related_unit_ids, vec![UnitId::new("valve-1")]);
        assert_eq!(
            context.related_stream_ids,
            vec![StreamId::new("stream-feed")]
        );
        assert!(context.related_port_targets.is_empty());
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.step.execution")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            [UnitId::new("valve-1")].as_slice()
        );
        assert_eq!(
            error.context().related_stream_ids(),
            [StreamId::new("stream-feed")].as_slice()
        );
    }

    #[test]
    fn solve_failure_context_extracts_step_lookup_unit_id() {
        let error = solver_step_lookup_error(
            3,
            &UnitId::new("flash-1"),
            RfError::missing_entity("unit", "flash-1"),
        );

        let context = SolveFailureContext::from_error(&error);

        assert_eq!(context.primary_code.as_deref(), Some("solver.step.lookup"));
        assert_eq!(context.related_unit_ids, vec![UnitId::new("flash-1")]);
        assert!(context.related_stream_ids.is_empty());
        assert!(context.related_port_targets.is_empty());
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.step.lookup")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            [UnitId::new("flash-1")].as_slice()
        );
        assert!(error.context().related_stream_ids().is_empty());
    }

    #[test]
    fn solve_failure_context_extracts_cycle_unit_ids_from_wrapped_message() {
        let context = SolveFailureContext::from_message(
            "flowsheet solve failed with package `binary-hydrocarbon-lite-v1`: solver.topological_ordering.two_unit_cycle: solver topological ordering failed: units `heater-1` and `valve-1` form a two-unit cycle through streams `stream-a` and `stream-b`; `valve-1.inlet` and `heater-1.inlet` currently depend on each other in opposite directions",
        );

        assert_eq!(
            context.primary_code.as_deref(),
            Some("solver.topological_ordering.two_unit_cycle")
        );
        assert_eq!(
            context.related_unit_ids,
            vec![UnitId::new("heater-1"), UnitId::new("valve-1")]
        );
        assert_eq!(
            context.related_stream_ids,
            vec![StreamId::new("stream-a"), StreamId::new("stream-b")]
        );
        assert!(context.related_port_targets.is_empty());
    }

    #[test]
    fn solve_failure_context_extracts_stream_ids_from_wrapped_message() {
        let context = SolveFailureContext::from_message(
            "flowsheet solve failed with package `binary-hydrocarbon-lite-v1`: solver.connection_validation.duplicate_downstream_sink: solver connection validation failed: stream `shared-stream` is consumed by both `mixer-1.inlet_a` and `flash-1.inlet`",
        );

        assert_eq!(
            context.primary_code.as_deref(),
            Some("solver.connection_validation.duplicate_downstream_sink")
        );
        assert_eq!(
            context.related_stream_ids,
            vec![StreamId::new("shared-stream")]
        );
    }

    #[test]
    fn cycle_detection_error_carries_related_units_in_error_context() {
        let provider = build_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let mut flowsheet = Flowsheet::new("heater-valve-cycle");
        for component in [
            Component::new("component-a", "Component A"),
            Component::new("component-b", "Component B"),
        ] {
            flowsheet
                .insert_component(component)
                .expect("expected component insert");
        }
        for stream in [
            build_stream(
                "stream-a",
                "Cycle Stream A",
                320.0,
                100_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
            build_stream(
                "stream-b",
                "Cycle Stream B",
                300.0,
                95_000.0,
                5.0,
                binary_composition(0.35, 0.65),
            ),
        ] {
            flowsheet
                .insert_stream(stream)
                .expect("expected stream insert");
        }
        for unit in [
            build_heater_node("heater-1", "Heater", "stream-b", "stream-a"),
            build_valve_node("valve-1", "Valve", "stream-a", "stream-b"),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        let error = SequentialModularSolver
            .solve(&services, &flowsheet)
            .expect_err("expected cycle detection failure");

        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.topological_ordering.two_unit_cycle")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            [UnitId::new("heater-1"), UnitId::new("valve-1")].as_slice()
        );
    }
}
