use std::path::Path;
use std::time::SystemTime;

use rf_property_package::{
    InMemoryPropertyPackageProvider, PackageFailureKind, PackageLoadError, PropertyPackageManifest,
    PropertyPackageProvider, PropertyPackageSource, load_property_package_files,
    prepare_local_context,
};
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveSnapshot, SolverServices};
use rf_store::{StoredProjectFile, parse_project_file_json};
use rf_thermo::{AntoineCoefficients, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, RfError, RfResult};
use serde::Serialize;

pub const DEMO_PACKAGE_ID: &str = "binary-hydrocarbon-synthetic-demo-v1";
const DEMO_PACKAGE_VERSION: &str = "2026.03.1";
const DEMO_REFERENCE_PRESSURE_PA: f64 = 100_000.0;

#[derive(Debug)]
pub struct Engine {
    package_provider: InMemoryPropertyPackageProvider,
    project: Option<StoredProjectFile>,
    latest_snapshot: Option<SolveSnapshot>,
    last_error: Option<RfError>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            package_provider: build_demo_package_provider(),
            project: None,
            latest_snapshot: None,
            last_error: None,
        }
    }

    pub fn load_flowsheet_json(&mut self, json: &str) -> RfResult<()> {
        let project = parse_project_file_json(json)?;
        self.project = Some(project);
        self.latest_snapshot = None;
        self.clear_last_error();
        Ok(())
    }

    pub fn solve_flowsheet(&mut self, package_id: &str) -> RfResult<()> {
        if package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "ffi flowsheet solve requires a non-empty package id",
            ));
        }

        let project = self.project.as_ref().ok_or_else(|| {
            RfError::invalid_input("ffi engine must load a flowsheet before solving")
                .with_diagnostic_code("ffi.engine_state.flowsheet_not_loaded")
        })?;
        let context = self
            .package_provider
            .load_package(package_id, SystemTime::now())
            .and_then(prepare_local_context)
            .map_err(package_error_for_ffi)?;
        let services = SolverServices {
            thermo: context.thermo(),
            flash_solver: context.tp_flash(),
        };
        let snapshot = SequentialModularSolver.solve(&services, &project.document.flowsheet)?;

        self.latest_snapshot = Some(snapshot);
        self.clear_last_error();
        Ok(())
    }

    pub fn stream_snapshot_json(&self, stream_id: &str) -> RfResult<String> {
        if stream_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "ffi stream snapshot export requires a non-empty stream id",
            ));
        }

        let snapshot = self.latest_snapshot.as_ref().ok_or_else(|| {
            RfError::invalid_input("ffi engine must solve a flowsheet before exporting streams")
                .with_diagnostic_code("ffi.engine_state.snapshot_not_available")
        })?;
        let stream = snapshot
            .stream(&stream_id.into())
            .ok_or_else(|| RfError::missing_entity("solved stream", stream_id))?;

        serde_json::to_string_pretty(stream).map_err(|error| {
            RfError::invalid_input(format!("failed to serialize stream json: {error}"))
        })
    }

    pub fn load_property_package_files(
        &mut self,
        manifest_path: impl AsRef<Path>,
        payload_path: impl AsRef<Path>,
    ) -> RfResult<String> {
        let manifest_path = manifest_path.as_ref();
        let payload_path = payload_path.as_ref();

        if !manifest_path.exists() {
            return Err(RfError::missing_entity(
                "property package manifest file",
                manifest_path.display(),
            )
            .with_diagnostic_code("ffi.property_package.manifest_not_found"));
        }

        if !payload_path.exists() {
            return Err(RfError::missing_entity(
                "property package payload file",
                payload_path.display(),
            )
            .with_diagnostic_code("ffi.property_package.payload_not_found"));
        }

        let package = load_property_package_files(manifest_path, payload_path, SystemTime::now())
            .map_err(package_error_for_ffi)?;
        let package_id = package.manifest().package_id.clone();
        self.package_provider.insert(package);
        Ok(package_id)
    }

    pub fn flowsheet_snapshot_json(&self) -> RfResult<String> {
        let snapshot = self.latest_snapshot.as_ref().ok_or_else(|| {
            RfError::invalid_input("ffi engine must solve a flowsheet before exporting snapshots")
                .with_diagnostic_code("ffi.engine_state.snapshot_not_available")
        })?;

        serde_json::to_string_pretty(&FfiSolveSnapshotJson::from_snapshot(snapshot)).map_err(
            |error| {
                RfError::invalid_input(format!("failed to serialize solve snapshot json: {error}"))
            },
        )
    }

    pub fn property_package_list_json(&self) -> RfResult<String> {
        let manifests = self.package_provider.registered_manifests();
        let json = manifests
            .iter()
            .map(FfiPropertyPackageManifestJson::from_manifest)
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&json).map_err(|error| {
            RfError::invalid_input(format!(
                "failed to serialize property package manifest list json: {error}"
            ))
        })
    }

    pub fn last_error(&self) -> Option<&RfError> {
        self.last_error.as_ref()
    }

    pub fn last_error_message(&self) -> &str {
        self.last_error.as_ref().map_or("", RfError::message)
    }

    pub fn replace_last_error(&mut self, error: RfError) {
        self.last_error = Some(error);
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn build_demo_package_provider() -> InMemoryPropertyPackageProvider {
    InMemoryPropertyPackageProvider::try_new(vec![build_demo_package()])
        .expect("valid built-in demo package")
}

fn package_error_for_ffi(error: PackageLoadError) -> RfError {
    match error.kind {
        PackageFailureKind::Expired | PackageFailureKind::UnsupportedSource => {
            error.into_rf_error()
        }
        _ => error.source,
    }
}

fn build_demo_antoine_coefficients(k_value: f64) -> AntoineCoefficients {
    const DEMO_ANTOINE_BOUNDARY_SLOPE: f64 = 300.0;
    const DEMO_REFERENCE_TEMPERATURE_K: f64 = 300.0;

    AntoineCoefficients::new(
        ((k_value * DEMO_REFERENCE_PRESSURE_PA) / 1_000.0).ln()
            + DEMO_ANTOINE_BOUNDARY_SLOPE / DEMO_REFERENCE_TEMPERATURE_K,
        DEMO_ANTOINE_BOUNDARY_SLOPE,
        0.0,
    )
}

fn build_demo_package() -> (PropertyPackageManifest, ThermoSystem) {
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(2.0));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(0.5));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    (
        PropertyPackageManifest::new(
            DEMO_PACKAGE_ID,
            DEMO_PACKAGE_VERSION,
            PropertyPackageSource::LocalBundled,
            vec![
                ComponentId::new("component-a"),
                ComponentId::new("component-b"),
            ],
        ),
        ThermoSystem::binary([first, second]),
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiSolveSnapshotJson {
    status: &'static str,
    summary: FfiSolveDiagnosticSummaryJson,
    diagnostics: Vec<FfiSolveDiagnosticJson>,
    streams: Vec<rf_model::MaterialStreamState>,
    steps: Vec<FfiUnitSolveStepJson>,
}

impl FfiSolveSnapshotJson {
    fn from_snapshot(snapshot: &SolveSnapshot) -> Self {
        Self {
            status: match snapshot.status {
                rf_solver::SolveStatus::Converged => "converged",
            },
            summary: FfiSolveDiagnosticSummaryJson::from_summary(&snapshot.summary),
            diagnostics: snapshot
                .diagnostics
                .iter()
                .map(FfiSolveDiagnosticJson::from_diagnostic)
                .collect(),
            streams: snapshot.streams.values().cloned().collect(),
            steps: snapshot
                .steps
                .iter()
                .map(FfiUnitSolveStepJson::from_step)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiSolveDiagnosticSummaryJson {
    highest_severity: &'static str,
    primary_message: String,
    diagnostic_count: usize,
    related_unit_ids: Vec<String>,
    related_stream_ids: Vec<String>,
}

impl FfiSolveDiagnosticSummaryJson {
    fn from_summary(summary: &rf_solver::SolveDiagnosticSummary) -> Self {
        Self {
            highest_severity: severity_label(summary.highest_severity),
            primary_message: summary.primary_message.clone(),
            diagnostic_count: summary.diagnostic_count,
            related_unit_ids: summary
                .related_unit_ids
                .iter()
                .map(|unit_id| unit_id.as_str().to_string())
                .collect(),
            related_stream_ids: summary
                .related_stream_ids
                .iter()
                .map(|stream_id| stream_id.as_str().to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiSolveDiagnosticJson {
    severity: &'static str,
    code: String,
    message: String,
    related_unit_ids: Vec<String>,
    related_stream_ids: Vec<String>,
}

impl FfiSolveDiagnosticJson {
    fn from_diagnostic(diagnostic: &rf_solver::SolveDiagnostic) -> Self {
        Self {
            severity: severity_label(diagnostic.severity),
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
            related_unit_ids: diagnostic
                .related_unit_ids
                .iter()
                .map(|unit_id| unit_id.as_str().to_string())
                .collect(),
            related_stream_ids: diagnostic
                .related_stream_ids
                .iter()
                .map(|stream_id| stream_id.as_str().to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiUnitSolveStepJson {
    index: usize,
    unit_id: String,
    unit_name: String,
    unit_kind: String,
    consumed_stream_ids: Vec<String>,
    produced_stream_ids: Vec<String>,
    summary: String,
}

impl FfiUnitSolveStepJson {
    fn from_step(step: &rf_solver::UnitSolveStep) -> Self {
        Self {
            index: step.index,
            unit_id: step.unit_id.as_str().to_string(),
            unit_name: step.unit_name.clone(),
            unit_kind: step.unit_kind.clone(),
            consumed_stream_ids: step
                .consumed_streams
                .iter()
                .map(|stream| stream.id.as_str().to_string())
                .collect(),
            produced_stream_ids: step
                .produced_streams
                .iter()
                .map(|stream| stream.id.as_str().to_string())
                .collect(),
            summary: step.summary.clone(),
        }
    }
}

fn severity_label(severity: rf_solver::SolveDiagnosticSeverity) -> &'static str {
    match severity {
        rf_solver::SolveDiagnosticSeverity::Info => "info",
        rf_solver::SolveDiagnosticSeverity::Warning => "warning",
        rf_solver::SolveDiagnosticSeverity::Error => "error",
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiPropertyPackageManifestJson {
    package_id: String,
    version: String,
    classification: &'static str,
    source: &'static str,
    hash: String,
    size_bytes: u64,
    component_ids: Vec<String>,
    expires_at: Option<String>,
}

impl FfiPropertyPackageManifestJson {
    fn from_manifest(manifest: &PropertyPackageManifest) -> Self {
        Self {
            package_id: manifest.package_id.clone(),
            version: manifest.version.clone(),
            classification: package_classification_label(manifest.classification),
            source: package_source_label(manifest.source),
            hash: manifest.hash.clone(),
            size_bytes: manifest.size_bytes,
            component_ids: manifest
                .component_ids
                .iter()
                .map(|component_id| component_id.as_str().to_string())
                .collect(),
            expires_at: manifest
                .expires_at
                .map(time_format_rfc3339)
                .transpose()
                .unwrap_or(None),
        }
    }
}

fn package_source_label(source: PropertyPackageSource) -> &'static str {
    match source {
        PropertyPackageSource::LocalBundled => "local-bundled",
        PropertyPackageSource::RemoteDerivedPackage => "remote-derived-package",
        PropertyPackageSource::RemoteEvaluationService => "remote-evaluation-service",
    }
}

fn package_classification_label(
    classification: rf_property_package::PropertyPackageClassification,
) -> &'static str {
    match classification {
        rf_property_package::PropertyPackageClassification::Derived => "derived",
        rf_property_package::PropertyPackageClassification::RemoteOnly => "remote-only",
    }
}

fn time_format_rfc3339(time: std::time::SystemTime) -> Result<String, time::error::Format> {
    let offset = time::OffsetDateTime::from(time);
    offset.format(&time::format_description::well_known::Rfc3339)
}
