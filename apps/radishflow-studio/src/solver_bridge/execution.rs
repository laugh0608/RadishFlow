use super::*;
use rf_model::Flowsheet;
use rf_property_package::{PackageProvenance, PropertyPackageDescriptor};
use rf_ui::{FlowRunDisposition, FlowRunId, FlowRunReceipt, FrozenFlowRun, SolveSnapshot};

/// Immutable formal inputs and the actual selected package ID. This value is
/// intentionally not Clone and never retains a mutable application reference.
#[derive(Debug)]
pub struct PreparedWorkspaceSolve {
    input: FrozenFlowRun,
    request: StudioSolveRequest,
}

#[derive(Debug)]
enum SolveOutcome {
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

    pub fn execute_with_property_package<P: PropertyPackageProvider>(
        self,
        provider: &P,
    ) -> CompletedWorkspaceSolve {
        let context = provider
            .load_package(&self.request.package_id, SystemTime::now())
            .and_then(prepare_local_context);
        self.execute_with_context(context, false)
    }

    pub fn execute_from_auth_cache(
        self,
        cache_root: impl AsRef<Path>,
        auth_cache_index: &StoredAuthCacheIndex,
    ) -> CompletedWorkspaceSolve {
        let provider = match CachedPropertyPackageProvider::new(cache_root, auth_cache_index) {
            Ok(provider) => provider,
            Err(error) => {
                return self.completed(
                    SolveOutcome::PreparationFailed {
                        error: error
                            .source
                            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE),
                        package_log: None,
                    },
                    None,
                );
            }
        };
        let context = provider
            .load_package(&self.request.package_id, SystemTime::now())
            .and_then(prepare_local_context);
        self.execute_with_context(context, true)
    }

    fn execute_with_context(
        self,
        context: PackageResult<LocalThermoContext>,
        cached: bool,
    ) -> CompletedWorkspaceSolve {
        let context = match context {
            Ok(context) => context,
            Err(error) => {
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
                return self.completed(outcome, None);
            }
        };
        let services = SolverServices {
            thermo: context.thermo(),
            flash_solver: context.tp_flash(),
        };
        let outcome = match SequentialModularSolver.solve(&services, self.flowsheet()) {
            Ok(snapshot) => SolveOutcome::Succeeded(Box::new(snapshot)),
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
        self.completed(
            outcome,
            Some(ExecutedPackage {
                descriptor: context.descriptor().clone(),
                provenance: context.provenance().clone(),
            }),
        )
    }

    fn completed(
        self,
        outcome: SolveOutcome,
        package: Option<ExecutedPackage>,
    ) -> CompletedWorkspaceSolve {
        CompletedWorkspaceSolve {
            receipt: self.input.into_receipt(),
            request: self.request,
            outcome,
            package,
        }
    }
}

impl CompletedWorkspaceSolve {
    pub fn run_id(&self) -> &FlowRunId {
        self.receipt.run_id()
    }

    pub fn result(&self) -> RfResult<&rf_solver::SolveSnapshot> {
        match &self.outcome {
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
    let disposition = app_state.accept_flow_run(&mut completion.receipt);
    if disposition != FlowRunDisposition::Applied {
        return disposition;
    }
    let revision = completion.receipt.revision();
    match &completion.outcome {
        SolveOutcome::Succeeded(snapshot) => {
            app_state.store_snapshot(SolveSnapshot::from_solver_snapshot(
                completion.request.snapshot_id.as_str(),
                revision,
                completion.request.sequence,
                snapshot,
            ));
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
        SolveOutcome::PreparationFailed { package_log, .. } => {
            if let Some(message) = package_log {
                app_state.push_log(AppLogLevel::Error, message);
            }
        }
    }
    disposition
}

fn package_failure_message(error: &PackageLoadError) -> String {
    format!(
        "{}: {}",
        error.kind.diagnostic_code(),
        error.source.message()
    )
}
