use std::collections::BTreeMap;
use std::path::Path;

use rf_flash::PlaceholderTpFlashSolver;
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveSnapshot, SolverServices};
use rf_store::{
    StoredAntoineCoefficients, StoredLiquidPhaseModel, StoredProjectFile,
    StoredPropertyPackageClassification, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageSource, StoredThermoComponent,
    StoredThermoMethod, StoredVaporPhaseModel, parse_project_file_json,
    read_property_package_manifest, read_property_package_payload,
};
use rf_thermo::{
    AntoineCoefficients, LiquidPhaseModel, PlaceholderThermoProvider, PropertyPackageManifest,
    PropertyPackageProvider, PropertyPackageSource, ThermoComponent, ThermoMethod, ThermoSystem,
    VaporPhaseModel,
};
use rf_types::{ComponentId, RfError, RfResult};
use serde::Serialize;

pub const DEMO_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
const DEMO_PACKAGE_VERSION: &str = "2026.03.1";
const DEMO_REFERENCE_PRESSURE_PA: f64 = 100_000.0;

#[derive(Debug)]
pub struct Engine {
    package_provider: EnginePropertyPackageRegistry,
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
        let thermo_system = self.package_provider.load_system(package_id)?;
        let thermo_provider = PlaceholderThermoProvider::new(thermo_system);
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &thermo_provider,
            flash_solver: &flash_solver,
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

        let stored_manifest = read_property_package_manifest(manifest_path)?;
        let stored_payload = read_property_package_payload(payload_path)?;
        stored_manifest.validate()?;
        stored_payload.validate_against_manifest(&stored_manifest)?;

        let package_id = stored_manifest.package_id.clone();
        let runtime_manifest = property_package_manifest_from_stored(stored_manifest)?;
        let runtime_system = thermo_system_from_stored_payload(stored_payload);
        self.package_provider
            .insert(runtime_manifest, runtime_system);
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
        let manifests = self.package_provider.list_manifests();
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

#[derive(Debug, Clone, Default)]
struct EnginePropertyPackageRegistry {
    packages: BTreeMap<String, (PropertyPackageManifest, ThermoSystem)>,
}

impl EnginePropertyPackageRegistry {
    fn with_demo_package() -> Self {
        let mut registry = Self::default();
        let (manifest, system) = build_demo_package();
        registry.insert(manifest, system);
        registry
    }

    fn insert(&mut self, manifest: PropertyPackageManifest, system: ThermoSystem) {
        self.packages
            .insert(manifest.package_id.clone(), (manifest, system));
    }
}

impl PropertyPackageProvider for EnginePropertyPackageRegistry {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|(manifest, _)| manifest.clone())
            .collect()
    }

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem> {
        self.packages
            .get(package_id)
            .map(|(_, system)| system.clone())
            .ok_or_else(|| RfError::missing_entity("property package", package_id))
    }
}

fn build_demo_package_provider() -> EnginePropertyPackageRegistry {
    EnginePropertyPackageRegistry::with_demo_package()
}

fn build_demo_package() -> (PropertyPackageManifest, ThermoSystem) {
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(AntoineCoefficients::new(
        ((2.0_f64 * DEMO_REFERENCE_PRESSURE_PA) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(AntoineCoefficients::new(
        ((0.5_f64 * DEMO_REFERENCE_PRESSURE_PA) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

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

fn property_package_manifest_from_stored(
    stored_manifest: StoredPropertyPackageManifest,
) -> RfResult<PropertyPackageManifest> {
    let source = property_package_source_from_stored(stored_manifest.source);
    let classification =
        property_package_classification_from_stored(stored_manifest.classification);

    let mut manifest = PropertyPackageManifest::new(
        stored_manifest.package_id,
        stored_manifest.version,
        source,
        stored_manifest.component_ids,
    );
    manifest.classification = classification;
    manifest.hash = stored_manifest.hash;
    manifest.size_bytes = stored_manifest.size_bytes;
    manifest.expires_at = stored_manifest.expires_at;
    Ok(manifest)
}

fn thermo_system_from_stored_payload(payload: StoredPropertyPackagePayload) -> ThermoSystem {
    let components = payload
        .components
        .into_iter()
        .map(thermo_component_from_stored)
        .collect();

    ThermoSystem {
        components,
        method: thermo_method_from_stored(payload.method),
    }
}

fn thermo_component_from_stored(component: StoredThermoComponent) -> ThermoComponent {
    ThermoComponent {
        id: component.id,
        name: component.name,
        antoine: component.antoine.map(antoine_coefficients_from_stored),
        liquid_heat_capacity_j_per_mol_k: component.liquid_heat_capacity_j_per_mol_k,
        vapor_heat_capacity_j_per_mol_k: component.vapor_heat_capacity_j_per_mol_k,
    }
}

fn antoine_coefficients_from_stored(
    coefficients: StoredAntoineCoefficients,
) -> AntoineCoefficients {
    AntoineCoefficients::new(coefficients.a, coefficients.b, coefficients.c)
}

fn thermo_method_from_stored(method: StoredThermoMethod) -> ThermoMethod {
    ThermoMethod {
        liquid_phase_model: liquid_phase_model_from_stored(method.liquid_phase_model),
        vapor_phase_model: vapor_phase_model_from_stored(method.vapor_phase_model),
    }
}

fn liquid_phase_model_from_stored(model: StoredLiquidPhaseModel) -> LiquidPhaseModel {
    match model {
        StoredLiquidPhaseModel::IdealSolution => LiquidPhaseModel::IdealSolution,
    }
}

fn vapor_phase_model_from_stored(model: StoredVaporPhaseModel) -> VaporPhaseModel {
    match model {
        StoredVaporPhaseModel::IdealGas => VaporPhaseModel::IdealGas,
    }
}

fn property_package_source_from_stored(
    source: StoredPropertyPackageSource,
) -> PropertyPackageSource {
    match source {
        StoredPropertyPackageSource::LocalBundled => PropertyPackageSource::LocalBundled,
        StoredPropertyPackageSource::RemoteDerivedPackage => {
            PropertyPackageSource::RemoteDerivedPackage
        }
        StoredPropertyPackageSource::RemoteEvaluationService => {
            PropertyPackageSource::RemoteEvaluationService
        }
    }
}

fn property_package_classification_from_stored(
    classification: StoredPropertyPackageClassification,
) -> rf_thermo::PropertyPackageClassification {
    match classification {
        StoredPropertyPackageClassification::Derived => {
            rf_thermo::PropertyPackageClassification::Derived
        }
        StoredPropertyPackageClassification::RemoteOnly => {
            rf_thermo::PropertyPackageClassification::RemoteOnly
        }
    }
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
                .consumed_stream_ids
                .iter()
                .map(|stream_id| stream_id.as_str().to_string())
                .collect(),
            produced_stream_ids: step
                .produced_stream_ids
                .iter()
                .map(|stream_id| stream_id.as_str().to_string())
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
    classification: rf_thermo::PropertyPackageClassification,
) -> &'static str {
    match classification {
        rf_thermo::PropertyPackageClassification::Derived => "derived",
        rf_thermo::PropertyPackageClassification::RemoteOnly => "remote-only",
    }
}

fn time_format_rfc3339(time: std::time::SystemTime) -> Result<String, time::error::Format> {
    let offset = time::OffsetDateTime::from(time);
    offset.format(&time::format_description::well_known::Rfc3339)
}
