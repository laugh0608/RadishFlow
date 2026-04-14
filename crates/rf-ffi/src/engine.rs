use rf_flash::PlaceholderTpFlashSolver;
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveSnapshot, SolverServices};
use rf_store::{StoredProjectFile, parse_project_file_json};
use rf_thermo::{
    AntoineCoefficients, InMemoryPropertyPackageProvider, PlaceholderThermoProvider,
    PropertyPackageManifest, PropertyPackageProvider, PropertyPackageSource, ThermoComponent,
    ThermoSystem,
};
use rf_types::{ComponentId, RfError, RfResult};

pub const DEMO_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
const DEMO_PACKAGE_VERSION: &str = "2026.03.1";
const DEMO_REFERENCE_PRESSURE_PA: f64 = 100_000.0;

#[derive(Debug)]
pub struct Engine {
    package_provider: InMemoryPropertyPackageProvider,
    project: Option<StoredProjectFile>,
    latest_snapshot: Option<SolveSnapshot>,
    last_error: String,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            package_provider: build_demo_package_provider(),
            project: None,
            latest_snapshot: None,
            last_error: String::new(),
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

        serde_json::to_string_pretty(stream)
            .map_err(|error| RfError::invalid_input(format!("failed to serialize stream json: {error}")))
    }

    pub fn last_error(&self) -> &str {
        &self.last_error
    }

    pub fn replace_last_error(&mut self, message: impl Into<String>) {
        self.last_error = message.into();
    }

    pub fn clear_last_error(&mut self) {
        self.last_error.clear();
    }
}

fn build_demo_package_provider() -> InMemoryPropertyPackageProvider {
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

    InMemoryPropertyPackageProvider::new(vec![(
        PropertyPackageManifest::new(
            DEMO_PACKAGE_ID,
            DEMO_PACKAGE_VERSION,
            PropertyPackageSource::LocalBundled,
            vec![ComponentId::new("component-a"), ComponentId::new("component-b")],
        ),
        ThermoSystem::binary([first, second]),
    )])
}
