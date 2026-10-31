use super::*;
use rf_model::Flowsheet;
use rf_property_package::{PackageProvenance, PropertyPackageDescriptor};
use rf_solver::{ControlledSolveOutcome, SolveProgress, SolverControl};
use rf_ui::{FlowRunDisposition, FlowRunId, FlowRunReceipt, FrozenFlowRun, SolveSnapshot};
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSolveProgress {
    Preparing,
    Ready,
    Solving(SolveProgress),
}

pub(crate) trait WorkspaceSolveControl: SolverControl {
    fn before_load(&mut self) -> ControlFlow<()>;
    fn ready(&mut self) -> RfResult<ControlFlow<()>>;
}

struct InlineControl;
impl SolverControl for InlineControl {
    fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl WorkspaceSolveControl for InlineControl {
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
    request: StudioSolveRequest,
}

pub(crate) enum WorkspacePackageLoadError {
    Provider(RfError),
    Package(PackageLoadError),
}

#[derive(Debug)]
pub(crate) struct WorkspaceSolveOutput {
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
    Cancelled,
    InfrastructureFailed(RfError),
    Succeeded(Box<rf_solver::SolveSnapshot>),
    Failed {
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

#[derive(Debug)]
struct ExecutedPackage {
    descriptor: PropertyPackageDescriptor,
    provenance: PackageProvenance,
}

/// Execution fact remains inspectable even when publication is discarded.
/// The non-cloneable receipt makes repeated delivery explicitly one-shot.
#[derive(Debug)]
pub struct CompletedWorkspaceSolve {
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
    Ok(PreparedWorkspaceSolve {
        input: app_state.prepare_flow_run(),
        request: request.clone(),
    })
}

impl PreparedWorkspaceSolve {
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
        let (flowsheet, receipt) = self.input.into_parts();
        (
            receipt,
            WorkspaceSolveJob {
                flowsheet,
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
            &mut InlineControl,
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
            &mut InlineControl,
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
            return self.output(SolveOutcome::Cancelled, None);
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
                    SolveOutcome::Failed {
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
            Ok(ControlFlow::Break(())) => return self.output(SolveOutcome::Cancelled, package),
            Err(error) => return self.output(SolveOutcome::InfrastructureFailed(error), package),
        }
        let services = SolverServices {
            thermo: context.thermo(),
            flash_solver: context.tp_flash(),
        };
        let outcome =
            match SequentialModularSolver.solve_controlled(&services, &self.flowsheet, control) {
                Ok(ControlledSolveOutcome::Completed(snapshot)) => {
                    SolveOutcome::Succeeded(Box::new(snapshot))
                }
                Ok(ControlledSolveOutcome::Cancelled) => SolveOutcome::Cancelled,
                Err(error) => SolveOutcome::Failed {
                    message: format!(
                        "flowsheet solve failed with package `{}`: {}",
                        self.request.package_id,
                        error.message()
                    ),
                    error,
                    package_log: None,
                },
            };
        self.output(outcome, package)
    }

    fn output(
        self,
        outcome: SolveOutcome,
        package: Option<ExecutedPackage>,
    ) -> WorkspaceSolveOutput {
        WorkspaceSolveOutput {
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
            request: output.request,
            outcome: output.outcome,
            package: output.package,
        }
    }

    pub(crate) fn infrastructure_failure(
        receipt: FlowRunReceipt,
        request: StudioSolveRequest,
        error: RfError,
    ) -> Self {
        Self {
            receipt,
            request,
            outcome: SolveOutcome::InfrastructureFailed(error),
            package: None,
        }
    }

    pub fn execution_outcome(&self) -> WorkspaceExecutionOutcome {
        match self.outcome {
            SolveOutcome::Succeeded(_) => WorkspaceExecutionOutcome::Succeeded,
            SolveOutcome::Cancelled => WorkspaceExecutionOutcome::Cancelled,
            SolveOutcome::InfrastructureFailed(_) => {
                WorkspaceExecutionOutcome::InfrastructureFailed
            }
            _ => WorkspaceExecutionOutcome::Failed,
        }
    }

    pub fn run_id(&self) -> &FlowRunId {
        self.receipt.run_id()
    }

    pub fn result(&self) -> RfResult<&rf_solver::SolveSnapshot> {
        match &self.outcome {
            SolveOutcome::Cancelled => Err(RfError::invalid_input("workspace solve cancelled")
                .with_diagnostic_code("workspace.run.cancelled")),
            SolveOutcome::InfrastructureFailed(error) => Err(error.clone()),
            SolveOutcome::Succeeded(snapshot) => Ok(snapshot),
            SolveOutcome::Failed { error, .. } | SolveOutcome::PreparationFailed { error, .. } => {
                Err(error.clone())
            }
        }
    }

    pub fn package_descriptor(&self) -> Option<&PropertyPackageDescriptor> {
        self.package.as_ref().map(|package| &package.descriptor)
    }

    pub fn provenance(&self) -> Option<&PackageProvenance> {
        self.package.as_ref().map(|package| &package.provenance)
    }
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
    let revision = completion.receipt.revision();
    let origin = completion.receipt.result_origin(
        concat!(
            "rf-solver::SequentialModularSolver@",
            env!("CARGO_PKG_VERSION")
        ),
        completion.package.as_ref().map(executed_package_evidence),
    );
    match &completion.outcome {
        SolveOutcome::Cancelled => {}
        SolveOutcome::InfrastructureFailed(error) => record_solve_failure(
            app_state,
            revision,
            error.message().into(),
            SolveFailureContext::from_error(error),
        ),
        SolveOutcome::Succeeded(snapshot) => {
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
        SolveOutcome::Failed {
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
    if matches!(
        completion.outcome,
        SolveOutcome::Failed { .. } | SolveOutcome::InfrastructureFailed(_)
    ) || (publish_preparation_failure
        && matches!(completion.outcome, SolveOutcome::PreparationFailed { .. }))
    {
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
