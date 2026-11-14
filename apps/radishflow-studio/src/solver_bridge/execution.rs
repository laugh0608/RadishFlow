use super::*;
use rf_model::Flowsheet;
use rf_property_package::{PackageProvenance, PropertyPackageDescriptor};
use rf_solver::{
    ControlledSolveOutcome, SolveProgress, SolverServices,
    equation_oriented::{EquationFailure, EquationProgress},
    execution::{
        FlowExecutionControl, FlowExecutionOutcome, FlowExecutionStatus, UncontrolledSolve,
        execute_flow,
    },
    mixed::{MixedFailure, MixedProgress},
    partitioned::{PartitionedProgress, PartitionedSolveFailure},
    simultaneous::{ModularFailure, ModularProgress},
};
#[cfg(test)]
use rf_solver::{
    SolverControl, equation_oriented::EquationControl, mixed::MixedControl,
    partitioned::PartitionedSolverControl, simultaneous::ModularControl,
};
use rf_ui::flow_plan::FlowPlanConfiguration;
use rf_ui::{FlowRunDisposition, FlowRunId, FlowRunReceipt, FrozenFlowRun, SolveSnapshot};
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceSolveProgress {
    Preparing,
    Ready,
    Solving(SolveProgress),
    Partitioned(PartitionedProgress),
    Simultaneous(ModularProgress),
    EquationOriented(EquationProgress),
    Mixed(MixedProgress),
}

pub(crate) trait WorkspaceSolveControl: FlowExecutionControl {
    fn before_load(&mut self) -> ControlFlow<()>;
    fn ready(&mut self) -> RfResult<ControlFlow<()>>;
}

impl WorkspaceSolveControl for UncontrolledSolve {
    fn before_load(&mut self) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
    fn ready(&mut self) -> RfResult<ControlFlow<()>> {
        Ok(ControlFlow::Continue(()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceExecutionOutcome {
    Succeeded,
    Failed,
    Cancelled,
    InfrastructureFailed,
}

/// Worker-owned numeric input. The acceptance receipt stays on the owner thread.
#[derive(Debug)]
pub(crate) struct WorkspaceSolveJob {
    flowsheet: Flowsheet,
    plan: FlowPlanConfiguration,
    request: StudioSolveRequest,
}

pub(crate) enum WorkspacePackageLoadError {
    Provider(RfError),
    Package(PackageLoadError),
}

#[derive(Debug)]
pub(crate) struct WorkspaceSolveOutput {
    plan: FlowPlanConfiguration,
    request: StudioSolveRequest,
    outcome: SolveOutcome,
    package: Option<ExecutedPackage>,
}

/// Immutable formal inputs and the actual selected package ID. This value is
/// intentionally not Clone and never retains a mutable application reference.
#[derive(Debug)]
pub struct PreparedWorkspaceSolve {
    input: FrozenFlowRun,
    request: StudioSolveRequest,
}

#[derive(Debug)]
enum SolveOutcome {
    CancelledBeforeExecution,
    Executed(FlowExecutionOutcome),
    InfrastructureFailed(RfError),
    PackageFailed {
        error: RfError,
        message: String,
        package_log: Option<String>,
    },
    // Preserve the cached-resource path's existing diagnostic/status semantics.
    PreparationFailed {
        error: RfError,
        package_log: Option<String>,
    },
}

impl SolveOutcome {
    fn matches_plan(&self, plan: &FlowPlanConfiguration) -> bool {
        match self {
            Self::Executed(execution) => execution.matches_plan(plan),
            _ => true,
        }
    }

    fn mixed_method(&self) -> Option<&'static str> {
        matches!(self, Self::Executed(FlowExecutionOutcome::Mixed(_)))
            .then_some(rf_solver::mixed::METHOD)
    }

    fn equation_invoked(&self) -> bool {
        matches!(
            self,
            Self::Executed(FlowExecutionOutcome::EquationOriented(_))
        )
    }

    fn equation_backend(&self) -> Option<&'static str> {
        match self {
            Self::Executed(FlowExecutionOutcome::EquationOriented(Ok(result))) => {
                Some(result.backend)
            }
            Self::Executed(FlowExecutionOutcome::EquationOriented(Err(failure))) => failure.backend,
            _ => None,
        }
    }

    fn modular_method(&self) -> Option<&'static str> {
        match self {
            Self::Executed(FlowExecutionOutcome::Simultaneous(Ok(result))) => Some(result.method),
            Self::Executed(FlowExecutionOutcome::Simultaneous(Err(_))) => {
                Some(rf_solver::simultaneous::METHOD)
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct ExecutedPackage {
    descriptor: PropertyPackageDescriptor,
    provenance: PackageProvenance,
}

/// Execution fact remains inspectable even when publication is discarded.
/// The non-cloneable receipt makes repeated delivery explicitly one-shot.
#[derive(Debug)]
pub struct CompletedWorkspaceSolve {
    plan: FlowPlanConfiguration,
    receipt: FlowRunReceipt,
    request: StudioSolveRequest,
    outcome: SolveOutcome,
    package: Option<ExecutedPackage>,
}

pub fn prepare_workspace_solve(
    app_state: &mut AppState,
    request: &StudioSolveRequest,
) -> RfResult<PreparedWorkspaceSolve> {
    request.validate()?;
    match &request.expected_plan {
        FlowPlanExpectation::LegacyAcyclic
            if app_state.workspace.flow_plan.current() != &FlowPlanConfiguration::Acyclic =>
        {
            return Err(
                RfError::invalid_input("legacy run requires the acyclic plan")
                    .with_diagnostic_code("workspace.run.unsupported_plan"),
            );
        }
        FlowPlanExpectation::Selected(expected) if !expected.is_current(&app_state.workspace) => {
            return Err(RfError::invalid_input("run plan expectation changed")
                .with_diagnostic_code("workspace.run.plan_changed"));
        }
        _ => {}
    }
    app_state
        .workspace
        .flow_plan
        .current()
        .validate_configuration()?;
    if matches!(
        app_state.workspace.flow_plan.current(),
        FlowPlanConfiguration::EquationOriented(_) | FlowPlanConfiguration::Mixed(_)
    ) {
        app_state
            .workspace
            .flow_plan
            .current()
            .validate_for(&app_state.workspace.document.flowsheet)?;
    }
    Ok(PreparedWorkspaceSolve {
        input: app_state.prepare_flow_run(),
        request: request.clone(),
    })
}

impl PreparedWorkspaceSolve {
    pub fn plan(&self) -> &FlowPlanConfiguration {
        self.input.plan()
    }
    pub fn run_id(&self) -> &FlowRunId {
        self.input.receipt().run_id()
    }

    pub fn revision(&self) -> u64 {
        self.input.receipt().revision()
    }

    pub fn flowsheet(&self) -> &Flowsheet {
        self.input.flowsheet()
    }

    pub fn request(&self) -> &StudioSolveRequest {
        &self.request
    }

    pub(crate) fn into_job(self) -> (FlowRunReceipt, WorkspaceSolveJob) {
        let (flowsheet, plan, receipt) = self.input.into_planned_parts();
        (
            receipt,
            WorkspaceSolveJob {
                flowsheet,
                plan,
                request: self.request,
            },
        )
    }

    pub fn execute_with_property_package<P: PropertyPackageProvider>(
        self,
        provider: &P,
    ) -> CompletedWorkspaceSolve {
        let (receipt, job) = self.into_job();
        let output = job.execute(
            |id| {
                provider
                    .load_package(id, SystemTime::now())
                    .and_then(prepare_local_context)
                    .map_err(WorkspacePackageLoadError::Package)
            },
            false,
            &mut UncontrolledSolve,
        );
        CompletedWorkspaceSolve::from_output(receipt, output)
    }

    pub fn execute_from_auth_cache(
        self,
        cache_root: impl AsRef<Path>,
        index: &StoredAuthCacheIndex,
    ) -> CompletedWorkspaceSolve {
        let (receipt, job) = self.into_job();
        let output = job.execute(
            |id| load_cached_context(cache_root.as_ref(), index, id),
            true,
            &mut UncontrolledSolve,
        );
        CompletedWorkspaceSolve::from_output(receipt, output)
    }
}

pub(crate) fn load_cached_context(
    root: &Path,
    index: &StoredAuthCacheIndex,
    id: &str,
) -> Result<LocalThermoContext, WorkspacePackageLoadError> {
    let provider = CachedPropertyPackageProvider::new(root, index).map_err(|error| {
        WorkspacePackageLoadError::Provider(
            error
                .source
                .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE),
        )
    })?;
    provider
        .load_package(id, SystemTime::now())
        .and_then(prepare_local_context)
        .map_err(WorkspacePackageLoadError::Package)
}

impl WorkspaceSolveJob {
    pub(crate) fn execute(
        self,
        load: impl FnOnce(&str) -> Result<LocalThermoContext, WorkspacePackageLoadError>,
        cached: bool,
        control: &mut impl WorkspaceSolveControl,
    ) -> WorkspaceSolveOutput {
        if control.before_load().is_break() {
            return self.output(SolveOutcome::CancelledBeforeExecution, None);
        }
        let context = match load(&self.request.package_id) {
            Ok(context) => context,
            Err(WorkspacePackageLoadError::Provider(error)) => {
                return self.output(
                    SolveOutcome::PreparationFailed {
                        error,
                        package_log: None,
                    },
                    None,
                );
            }
            Err(WorkspacePackageLoadError::Package(error)) => {
                let package_log = (error.kind != PackageFailureKind::Missing)
                    .then(|| package_failure_message(&error));
                let preparation_failure = cached
                    && !matches!(
                        error.kind,
                        PackageFailureKind::Missing
                            | PackageFailureKind::Expired
                            | PackageFailureKind::UnsupportedSource
                    );
                let error = workspace_property_package_load_error(error);
                let outcome = if preparation_failure {
                    SolveOutcome::PreparationFailed { error, package_log }
                } else {
                    SolveOutcome::PackageFailed {
                        message: format!(
                            "failed to load property package `{}`: {}",
                            self.request.package_id,
                            error.message()
                        ),
                        error,
                        package_log,
                    }
                };
                return self.output(outcome, None);
            }
        };
        let package = Some(ExecutedPackage {
            descriptor: context.descriptor().clone(),
            provenance: context.provenance().clone(),
        });
        match control.ready() {
            Ok(ControlFlow::Continue(())) => {}
            Ok(ControlFlow::Break(())) => {
                return self.output(SolveOutcome::CancelledBeforeExecution, package);
            }
            Err(error) => return self.output(SolveOutcome::InfrastructureFailed(error), package),
        }
        let services = SolverServices {
            thermo: context.thermo(),
            flash_solver: context.tp_flash(),
        };
        let execution = execute_flow(&self.flowsheet, &self.plan, &services, control);
        self.output(SolveOutcome::Executed(execution), package)
    }

    fn output(
        self,
        outcome: SolveOutcome,
        package: Option<ExecutedPackage>,
    ) -> WorkspaceSolveOutput {
        WorkspaceSolveOutput {
            plan: self.plan,
            request: self.request,
            outcome,
            package,
        }
    }
}

impl CompletedWorkspaceSolve {
    pub(crate) fn from_output(receipt: FlowRunReceipt, output: WorkspaceSolveOutput) -> Self {
        Self {
            receipt,
            plan: output.plan,
            request: output.request,
            outcome: output.outcome,
            package: output.package,
        }
    }

    pub(crate) fn infrastructure_failure(
        receipt: FlowRunReceipt,
        request: StudioSolveRequest,
        plan: FlowPlanConfiguration,
        error: RfError,
    ) -> Self {
        Self {
            plan,
            receipt,
            request,
            outcome: SolveOutcome::InfrastructureFailed(error),
            package: None,
        }
    }

    pub fn execution_outcome(&self) -> WorkspaceExecutionOutcome {
        match &self.outcome {
            SolveOutcome::Executed(execution) => match execution.status() {
                FlowExecutionStatus::Succeeded => WorkspaceExecutionOutcome::Succeeded,
                FlowExecutionStatus::Failed => WorkspaceExecutionOutcome::Failed,
                FlowExecutionStatus::Cancelled => WorkspaceExecutionOutcome::Cancelled,
            },
            SolveOutcome::CancelledBeforeExecution => WorkspaceExecutionOutcome::Cancelled,
            SolveOutcome::InfrastructureFailed(_) => {
                WorkspaceExecutionOutcome::InfrastructureFailed
            }
            SolveOutcome::PreparationFailed { .. } | SolveOutcome::PackageFailed { .. } => {
                WorkspaceExecutionOutcome::Failed
            }
        }
    }

    pub fn run_id(&self) -> &FlowRunId {
        self.receipt.run_id()
    }

    pub fn partitioned_failure_evidence(&self) -> Option<&PartitionedSolveFailure> {
        match &self.outcome {
            SolveOutcome::Executed(FlowExecutionOutcome::Partitioned(Err(failure))) => {
                Some(failure)
            }
            _ => None,
        }
    }

    pub fn simultaneous_failure_evidence(&self) -> Option<&ModularFailure> {
        match &self.outcome {
            SolveOutcome::Executed(FlowExecutionOutcome::Simultaneous(Err(failure))) => {
                Some(failure)
            }
            _ => None,
        }
    }

    pub fn mixed_failure_evidence(&self) -> Option<&MixedFailure> {
        match &self.outcome {
            SolveOutcome::Executed(FlowExecutionOutcome::Mixed(Err(failure))) => Some(failure),
            _ => None,
        }
    }

    pub fn equation_failure_evidence(&self) -> Option<&EquationFailure> {
        match &self.outcome {
            SolveOutcome::Executed(FlowExecutionOutcome::EquationOriented(Err(failure))) => {
                Some(failure)
            }
            _ => None,
        }
    }

    pub fn completion_result(&self) -> RfResult<()> {
        match self.execution_outcome() {
            WorkspaceExecutionOutcome::Succeeded => Ok(()),
            WorkspaceExecutionOutcome::Cancelled => Err(cancelled_error()),
            _ => self.result().map(|_| ()),
        }
    }

    pub fn result(&self) -> RfResult<&rf_solver::SolveSnapshot> {
        match &self.outcome {
            SolveOutcome::Executed(execution) => {
                use FlowExecutionOutcome::*;
                if let Some(error) = execution.error() {
                    return Err(error.clone());
                }
                match execution {
                    Acyclic(Ok(ControlledSolveOutcome::Completed(snapshot))) => Ok(snapshot),
                    Acyclic(Ok(ControlledSolveOutcome::Cancelled)) => Err(cancelled_error()),
                    Partitioned(Ok(_)) => Err(RfError::invalid_input(
                        "partitioned results require the typed flow result query",
                    )),
                    Mixed(Ok(_)) => Err(RfError::invalid_input(
                        "mixed results require typed flow result query",
                    )),
                    EquationOriented(Ok(_)) => Err(RfError::invalid_input(
                        "equation results require the typed flow result query",
                    )),
                    Simultaneous(Ok(_)) => Err(RfError::invalid_input(
                        "simultaneous results require the typed flow result query",
                    )),
                    _ => unreachable!("execution errors are handled above"),
                }
            }
            SolveOutcome::CancelledBeforeExecution => Err(cancelled_error()),
            SolveOutcome::InfrastructureFailed(error)
            | SolveOutcome::PackageFailed { error, .. }
            | SolveOutcome::PreparationFailed { error, .. } => Err(error.clone()),
        }
    }

    pub fn package_descriptor(&self) -> Option<&PropertyPackageDescriptor> {
        self.package.as_ref().map(|package| &package.descriptor)
    }

    pub fn provenance(&self) -> Option<&PackageProvenance> {
        self.package.as_ref().map(|package| &package.provenance)
    }
}

fn cancelled_error() -> RfError {
    RfError::invalid_input("workspace solve cancelled")
        .with_diagnostic_code("workspace.run.cancelled")
}

/// Guard and publication share one exclusive application borrow. No logs,
/// failure state, or snapshot history are changed for a rejected completion.
pub fn complete_workspace_solve(
    app_state: &mut AppState,
    completion: &mut CompletedWorkspaceSolve,
) -> FlowRunDisposition {
    complete_with_publication(app_state, completion, false)
}

/// Background completion has no later synchronous Facade error-dispatch pass.
/// It therefore publishes a current preparation failure through the same guard.
pub(crate) fn complete_background_workspace_solve(
    app_state: &mut AppState,
    completion: &mut CompletedWorkspaceSolve,
) -> FlowRunDisposition {
    complete_with_publication(app_state, completion, true)
}

fn complete_with_publication(
    app_state: &mut AppState,
    completion: &mut CompletedWorkspaceSolve,
    publish_preparation_failure: bool,
) -> FlowRunDisposition {
    let disposition = app_state.accept_flow_run(&mut completion.receipt);
    if disposition != FlowRunDisposition::Applied {
        return disposition;
    }
    let mismatched = !completion.outcome.matches_plan(&completion.plan);
    if mismatched {
        completion.outcome = SolveOutcome::InfrastructureFailed(
            RfError::invalid_input("result payload does not match frozen run plan")
                .with_diagnostic_code("workspace.run.plan_payload_mismatch"),
        );
    }
    let revision = completion.receipt.revision();
    let implementation = match &completion.plan {
        FlowPlanConfiguration::Mixed(_) if completion.outcome.mixed_method().is_some() => {
            concat!("rf-solver::MixedSolver@", env!("CARGO_PKG_VERSION"))
        }
        FlowPlanConfiguration::Mixed(_) => "not-invoked",
        FlowPlanConfiguration::Acyclic => concat!(
            "rf-solver::SequentialModularSolver@",
            env!("CARGO_PKG_VERSION")
        ),
        FlowPlanConfiguration::Partitioned(_) => concat!(
            "rf-solver::PartitionedSequentialSolver@",
            env!("CARGO_PKG_VERSION")
        ),
        FlowPlanConfiguration::Simultaneous(_) if completion.outcome.modular_method().is_some() => {
            concat!(
                "rf-solver::SimultaneousModularSolver@",
                env!("CARGO_PKG_VERSION")
            )
        }
        FlowPlanConfiguration::Simultaneous(_) => "not-invoked",
        FlowPlanConfiguration::EquationOriented(_) if completion.outcome.equation_invoked() => {
            concat!(
                "rf-solver::EquationOrientedSolver@",
                env!("CARGO_PKG_VERSION")
            )
        }
        FlowPlanConfiguration::EquationOriented(_) => "not-invoked",
    };
    let origin = completion
        .receipt
        .result_origin(
            implementation,
            completion.package.as_ref().map(executed_package_evidence),
        )
        .with_plan_evidence(rf_ui::result_query::ExecutedFlowPlan {
            method: match &completion.plan {
                FlowPlanConfiguration::Acyclic => "radishflow.acyclic-sm/1",
                FlowPlanConfiguration::Mixed(_) => rf_solver::mixed::METHOD,
                FlowPlanConfiguration::Partitioned(_) => "radishflow.partitioned-sm/1",
                FlowPlanConfiguration::Simultaneous(_) => "radishflow.fixed-tp-affine-modular-lu/1",
                FlowPlanConfiguration::EquationOriented(_) => "radishflow.fixed-tp-material-eo/1",
            },
            acceptance_contract_version: 1,
            configuration_sha256: rf_store::sha256_hex(
                &serde_json::to_vec(&crate::flow_plan_actions::stored_flow_plan(
                    &completion.plan,
                ))
                .expect("validated frozen plan is serializable"),
            ),
            equation_oriented: matches!(
                completion.plan,
                FlowPlanConfiguration::EquationOriented(_)
            )
            .then(|| rf_ui::result_query::EquationExecutionEvidence {
                capability: rf_store::project_v4::EQUATION_ORIENTED_CAPABILITY,
                capability_version: 1,
                equation_contract_version: rf_solver::equation_oriented::EQUATION_CONTRACT_VERSION,
                core_backend: completion.outcome.equation_backend(),
            }),
            mixed: matches!(completion.plan, FlowPlanConfiguration::Mixed(_)).then(|| {
                rf_ui::result_query::MixedExecutionEvidence {
                    capability: rf_store::project_v4::MIXED_CAPABILITY,
                    capability_version: 1,
                    core_method: completion.outcome.mixed_method(),
                }
            }),
            modular: matches!(completion.plan, FlowPlanConfiguration::Simultaneous(_)).then(|| {
                rf_ui::result_query::ModularExecutionEvidence {
                    capability: rf_store::project_v4::SIMULTANEOUS_CAPABILITY,
                    capability_version: 1,
                    response_contract_version: rf_solver::simultaneous::RESPONSE_VERSION,
                    core_method: completion.outcome.modular_method(),
                }
            }),
        });
    match &completion.outcome {
        SolveOutcome::CancelledBeforeExecution => {}
        SolveOutcome::Executed(execution)
            if execution.status() == FlowExecutionStatus::Cancelled => {}
        SolveOutcome::Executed(FlowExecutionOutcome::Mixed(Ok(result))) => {
            let mut accepted = rf_ui::MixedFlowSnapshot::new(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                (**result).clone(),
            );
            accepted.origin = Some(origin.clone());
            app_state.store_snapshot(accepted);
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Mixed(Err(f))) => {
            record_solve_failure(
                app_state,
                revision,
                f.error.to_string(),
                SolveFailureContext::from_error(&f.error),
            );
            app_state.workspace.attach_mixed_failure((**f).clone());
        }
        SolveOutcome::Executed(FlowExecutionOutcome::EquationOriented(Ok(result))) => {
            let mut accepted = rf_ui::EquationFlowSnapshot::new(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                (**result).clone(),
            );
            accepted.origin = Some(origin.clone());
            app_state.store_snapshot(accepted);
        }
        SolveOutcome::Executed(FlowExecutionOutcome::EquationOriented(Err(failure))) => {
            record_solve_failure(
                app_state,
                revision,
                failure.error.to_string(),
                SolveFailureContext::from_error(&failure.error),
            );
            app_state
                .workspace
                .attach_equation_failure((**failure).clone());
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Simultaneous(Ok(result))) => {
            let mut accepted = rf_ui::SimultaneousFlowSnapshot::new(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                (**result).clone(),
            );
            accepted.origin = Some(origin.clone());
            app_state.store_snapshot(accepted);
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Simultaneous(Err(failure))) => {
            record_solve_failure(
                app_state,
                revision,
                failure.error.to_string(),
                SolveFailureContext::from_error(&failure.error),
            );
            app_state
                .workspace
                .attach_simultaneous_failure((**failure).clone());
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Partitioned(Ok(result))) => {
            let mut accepted = rf_ui::PartitionedFlowSnapshot::new(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                (**result).clone(),
            );
            accepted.origin = Some(origin.clone());
            app_state.store_snapshot(accepted);
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Partitioned(Err(failure))) => {
            record_solve_failure(
                app_state,
                revision,
                failure.error.to_string(),
                SolveFailureContext::from_error(&failure.error),
            );
            app_state
                .workspace
                .attach_partitioned_failure((**failure).clone());
        }
        SolveOutcome::InfrastructureFailed(error) => record_solve_failure(
            app_state,
            revision,
            error.message().into(),
            SolveFailureContext::from_error(error),
        ),
        SolveOutcome::Executed(FlowExecutionOutcome::Acyclic(Ok(
            ControlledSolveOutcome::Completed(snapshot),
        ))) => {
            let mut accepted = SolveSnapshot::from_solver_snapshot(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                snapshot,
            );
            accepted.origin = Some(origin.clone());
            app_state.store_snapshot(accepted);
            app_state.push_log(
                AppLogLevel::Info,
                format!(
                    "Solved document revision {} with property package `{}` into snapshot `{}`",
                    revision, completion.request.package_id, completion.request.snapshot_id
                ),
            );
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Acyclic(Err(error))) => {
            record_solve_failure(
                app_state,
                revision,
                format!(
                    "flowsheet solve failed with package `{}`: {}",
                    completion.request.package_id,
                    error.message()
                ),
                SolveFailureContext::from_error(error),
            );
        }
        SolveOutcome::Executed(FlowExecutionOutcome::Acyclic(Ok(
            ControlledSolveOutcome::Cancelled,
        ))) => {}
        SolveOutcome::PackageFailed {
            error,
            message,
            package_log,
        } => {
            if let Some(message) = package_log {
                app_state.push_log(AppLogLevel::Error, message);
            }
            record_solve_failure(
                app_state,
                revision,
                message.clone(),
                SolveFailureContext::from_error(error),
            );
        }
        SolveOutcome::PreparationFailed { error, package_log } => {
            if let Some(message) = package_log {
                app_state.push_log(AppLogLevel::Error, message);
            }
            if publish_preparation_failure {
                record_solve_failure(
                    app_state,
                    revision,
                    format!(
                        "failed to prepare local property package cache for `{}`: {}",
                        completion.request.package_id,
                        error.message()
                    ),
                    SolveFailureContext::from_error(error),
                );
            }
        }
    }
    let publish_failure_origin = match &completion.outcome {
        SolveOutcome::CancelledBeforeExecution
        | SolveOutcome::Executed(FlowExecutionOutcome::Mixed(_)) => false,
        SolveOutcome::Executed(execution) => execution.status() == FlowExecutionStatus::Failed,
        SolveOutcome::PackageFailed { .. } | SolveOutcome::InfrastructureFailed(_) => true,
        SolveOutcome::PreparationFailed { .. } => publish_preparation_failure,
    };
    if publish_failure_origin {
        app_state.attach_flow_failure_origin(origin);
    }
    disposition
}

fn executed_package_evidence(
    package: &ExecutedPackage,
) -> rf_ui::result_query::ExecutedPropertyPackage {
    use rf_property_package::PackageDataOrigin;
    use rf_ui::result_query::{ExecutedPropertyPackage, ResultDataOrigin};
    let descriptor = &package.descriptor;
    let source = &package.provenance;
    ExecutedPropertyPackage {
        package_id: descriptor.manifest.package_id.clone(),
        version: descriptor.manifest.version.clone(),
        origin: match source.origin {
            PackageDataOrigin::InMemory => ResultDataOrigin::InMemory,
            PackageDataOrigin::Cache => ResultDataOrigin::Cache,
            PackageDataOrigin::LocalFiles => ResultDataOrigin::LocalFiles,
            PackageDataOrigin::Bundled => ResultDataOrigin::Bundled,
        },
        manifest_schema_version: source.manifest_schema_version,
        payload_schema_version: source.payload_schema_version,
        payload_fingerprint: source.payload_fingerprint.clone(),
        canonical_payload_size_bytes: source.canonical_payload_size_bytes,
        liquid_model: match descriptor.method.liquid_phase_model {
            rf_thermo::LiquidPhaseModel::IdealSolution => "ideal-solution",
        },
        vapor_model: match descriptor.method.vapor_phase_model {
            rf_thermo::VaporPhaseModel::IdealGas => "ideal-gas",
        },
        thermo_implementation: source.thermo_implementation,
        flash_implementation: source.flash_implementation,
    }
}

fn package_failure_message(error: &PackageLoadError) -> String {
    format!(
        "{}: {}",
        error.kind.diagnostic_code(),
        error.source.message()
    )
}

#[cfg(test)]
mod plan_contract_tests;

#[cfg(test)]
mod equation_tests;
#[cfg(test)]
mod simultaneous_tests;

#[cfg(test)]
mod mixed_tests;
