use super::*;
use radishflow_studio::{
    PropertyPackageCacheRoot, WorkspaceExecutionOutcome, WorkspaceRunCommand, WorkspaceRunOwner,
    WorkspaceRunPackageSelection, WorkspaceRunSubmission, WorkspaceSolveTrigger,
};
use rf_ffi::RfFfiStatus;
use rf_model::MaterialStreamState;
use rf_ui::{AppState, FlowResultRecord, FlowRunDisposition, RunStatus, latest_snapshot};
use std::ffi::{CStr, c_void};
use std::path::Path;
use std::ptr;
use std::time::{Duration, Instant};

const PACKAGE: &str = test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID;
const PROJECT: &str = "工程 example.rfproj.json";

struct NativeEngine(*mut c_void);

impl NativeEngine {
    fn new() -> Self {
        let mut engine = ptr::null_mut();
        assert_eq!(
            unsafe { rf_ffi::engine_create(&mut engine) },
            RfFfiStatus::Ok
        );
        Self(engine.cast())
    }

    fn register_package(&self, manifest: &Path, payload: &Path) -> RfFfiStatus {
        let manifest = manifest.to_str().unwrap();
        let payload = payload.to_str().unwrap();
        unsafe {
            rf_ffi::property_package_load_from_files(
                self.0.cast(),
                manifest.as_ptr(),
                manifest.len(),
                payload.as_ptr(),
                payload.len(),
            )
        }
    }

    fn load(&self, project: &[u8]) {
        assert_eq!(
            unsafe { rf_ffi::flowsheet_load_json(self.0.cast(), project.as_ptr(), project.len()) },
            RfFfiStatus::Ok,
        );
    }

    fn solve(&self, package: &str) -> RfFfiStatus {
        unsafe { rf_ffi::flowsheet_solve(self.0.cast(), package.as_ptr(), package.len()) }
    }

    fn snapshot(&self) -> Value {
        let mut output = ptr::null_mut();
        let status = unsafe { rf_ffi::flowsheet_get_snapshot_json(self.0.cast(), &mut output) };
        assert_eq!(status, RfFfiStatus::Ok);
        unsafe { read_owned_json(output) }
    }

    fn error(&self) -> Value {
        let mut output = ptr::null_mut();
        let status = unsafe { rf_ffi::engine_last_error_json(self.0.cast(), &mut output) };
        assert_eq!(status, RfFfiStatus::Ok);
        unsafe { read_owned_json(output) }
    }
}

impl Drop for NativeEngine {
    fn drop(&mut self) {
        unsafe { rf_ffi::engine_destroy(self.0.cast()) }
    }
}

// The ABI returns an owned NUL-terminated UTF-8 string on success.
unsafe fn read_owned_json(output: *mut std::ffi::c_char) -> Value {
    assert!(!output.is_null());
    let bytes = unsafe { CStr::from_ptr(output) }.to_bytes().to_vec();
    unsafe { rf_ffi::rf_string_free(output) };
    serde_json::from_slice(&bytes).unwrap()
}

fn run_studio(f: &Fixture) -> AppState {
    let mut app = load_project_app_state(&f.root.join(PROJECT)).unwrap();
    let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
    let mut owner = WorkspaceRunOwner::new();
    let submission = owner
        .submit_cached(
            &mut app,
            PropertyPackageCacheRoot::persistent(f.root.join("cache")),
            &index,
            &WorkspaceRunCommand::new(
                WorkspaceSolveTrigger::Manual,
                WorkspaceRunPackageSelection::Explicit(PACKAGE.into()),
            ),
            SystemTime::now(),
        )
        .unwrap();
    let WorkspaceRunSubmission::Accepted(id) = submission else {
        panic!("expected an accepted worker, got {submission:?}");
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    let completion = loop {
        if let Some(completion) = owner.poll(&mut app, SystemTime::now()) {
            break completion;
        }
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::yield_now();
    };
    assert_eq!(completion.run_id, id);
    assert_eq!(completion.disposition, FlowRunDisposition::Applied);
    assert!(matches!(
        completion.outcome,
        WorkspaceExecutionOutcome::Succeeded | WorkspaceExecutionOutcome::Failed,
    ));
    assert!(owner.try_close().is_ok());
    app
}

fn package_paths(f: &Fixture) -> (PathBuf, PathBuf, String) {
    let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
    let record = &index.property_packages[0];
    let root = f.root.join("cache");
    let payload = record.payload_path_under(&root).unwrap();
    let fingerprint = rf_store::property_package_payload_integrity(
        &rf_store::read_property_package_payload(&payload).unwrap(),
    )
    .unwrap()
    .hash;
    (record.manifest_path_under(root), payload, fingerprint)
}

fn result_target(stream: &MaterialStreamState, field: Value) -> Value {
    json!({"object":{"stream":stream.id},"section":"results","field":field})
}

fn assert_read(row: &Value, target: &Value, value: Option<f64>, unit: &str) {
    assert_eq!(row["target"], *target);
    assert_si_value(row["value"].as_f64(), value);
    assert_eq!(row["unit"], unit);
    if value.is_some() {
        assert_eq!(row["source"]["kind"], "solve_result");
    }
}

fn assert_si_value(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => {
            // Parsing the two JSON projections can round by a few ULPs.
            let tolerance = 8.0 * f64::EPSILON * actual.abs().max(expected.abs()).max(1.0);
            assert!(
                (actual - expected).abs() <= tolerance,
                "{actual} != {expected}"
            );
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn studio_cli_and_c_abi_agree_on_legacy_flow_values() {
    let f = Fixture::new();
    let native = NativeEngine::new();
    let (manifest, payload, fingerprint) = package_paths(&f);
    assert_eq!(
        native.register_package(&manifest, &payload),
        RfFfiStatus::Ok
    );
    let path = f.root.join(PROJECT);
    let mut project = rf_store::read_project_file(&path).unwrap();

    for version in [1, 2] {
        project.schema_version = version;
        if version == 2 {
            project.presentation.display_units = rf_types::units::DisplayUnitSet::engineering();
        }
        rf_store::write_project_file(&path, &project).unwrap();
        let original = fs::read(&path).unwrap();
        native.load(&original);
        assert_eq!(native.solve(PACKAGE), RfFfiStatus::Ok);
        let native_result = native.snapshot();
        assert_eq!(native_result["status"], "converged");
        let streams: Vec<MaterialStreamState> =
            serde_json::from_value(native_result["streams"].clone()).unwrap();
        let app = run_studio(&f);
        let Some(FlowResultRecord::Acyclic(studio)) = latest_snapshot(&app.workspace) else {
            panic!("expected an accepted acyclic result");
        };
        assert_eq!(studio.status, RunStatus::Converged);
        let origin = studio.origin.as_ref().unwrap().execution().unwrap();
        assert_eq!(
            origin.package.as_ref().unwrap().payload_fingerprint,
            fingerprint
        );
        assert_eq!(studio.streams.len(), streams.len());
        assert_eq!(
            native_result["steps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|step| step["unitId"].as_str().unwrap())
                .collect::<Vec<_>>(),
            studio
                .steps
                .iter()
                .map(|step| step.unit_id.as_str())
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            native_result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostic| diagnostic["code"].as_str().unwrap())
                .collect::<Vec<_>>(),
            studio
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
        );
        let mut reads = Vec::new();
        for stream in &streams {
            let ui = studio
                .streams
                .iter()
                .find(|ui| ui.stream_id == stream.id)
                .unwrap();
            assert_si_value(Some(ui.temperature_k), Some(stream.temperature_k));
            assert_si_value(Some(ui.pressure_pa), Some(stream.pressure_pa));
            assert_si_value(
                Some(ui.total_molar_flow_mol_s),
                Some(stream.total_molar_flow_mol_s),
            );
            reads.extend([
                (
                    result_target(stream, json!("temperature")),
                    Some(stream.temperature_k),
                    "K",
                ),
                (
                    result_target(stream, json!("pressure")),
                    Some(stream.pressure_pa),
                    "Pa",
                ),
                (
                    result_target(stream, json!("molar_flow")),
                    Some(stream.total_molar_flow_mol_s),
                    "mol/s",
                ),
            ]);
            assert_eq!(
                ui.overall_mole_fractions.len(),
                stream.overall_mole_fractions.len()
            );
            for (component, x) in &stream.overall_mole_fractions {
                assert_si_value(
                    Some(
                        ui.overall_mole_fractions
                            .iter()
                            .find(|(id, _)| id == component.as_str())
                            .unwrap()
                            .1,
                    ),
                    Some(*x),
                );
                reads.push((
                    result_target(stream, json!({"mole_fraction":component})),
                    Some(*x),
                    "mol/mol",
                ));
            }
            assert_eq!(ui.phases.len(), stream.phases.len());
            for phase in &stream.phases {
                let label = phase.label.as_str();
                let ui_phase = ui.phases.iter().find(|ui| ui.label == label).unwrap();
                assert_si_value(Some(ui_phase.phase_fraction), Some(phase.phase_fraction));
                assert_si_value(
                    ui_phase.molar_enthalpy_j_per_mol,
                    phase.molar_enthalpy_j_per_mol,
                );
                reads.extend([
                    (
                        result_target(stream, json!({"phase_fraction":phase.label})),
                        Some(phase.phase_fraction),
                        "mol/mol",
                    ),
                    (
                        result_target(stream, json!({"phase_molar_enthalpy":phase.label})),
                        phase.molar_enthalpy_j_per_mol,
                        "J/mol",
                    ),
                ]);
                assert_eq!(ui_phase.composition.len(), phase.mole_fractions.len());
                for (component, x) in &phase.mole_fractions {
                    assert_si_value(
                        Some(
                            ui_phase
                                .composition
                                .iter()
                                .find(|(id, _)| id == component.as_str())
                                .unwrap()
                                .1,
                        ),
                        Some(*x),
                    );
                    reads.push((result_target(stream, json!({"phase_mole_fraction":{"phase":phase.label,"component":component}})), Some(*x), "mol/mol"));
                }
            }
        }

        let mut request = f.request();
        request["package_id"] = json!(PACKAGE);
        request["writes"] = json!([]);
        request["reads"] = json!(
            reads
                .iter()
                .map(|(target, _, _)| target)
                .collect::<Vec<_>>()
        );
        let (code, cli) = f.run(&request);
        assert_eq!(code, 0, "{cli}");
        assert_eq!(cli["status"], "ok");
        assert_eq!(cli["package_id"], PACKAGE);
        let rows = cli["variables"].as_array().unwrap();
        assert_eq!(rows.len(), reads.len());
        for (row, (target, value, unit)) in rows.iter().zip(&reads) {
            assert_read(row, target, *value, unit);
        }
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}

#[test]
fn studio_cli_and_c_abi_preserve_connection_failure_targets() {
    let f = Fixture::new();
    let native = NativeEngine::new();
    let (manifest, payload, _) = package_paths(&f);
    assert_eq!(
        native.register_package(&manifest, &payload),
        RfFfiStatus::Ok
    );

    for (project, expected_status, expected_code) in [
        (
            include_bytes!("../../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json")
                .as_slice(),
            RfFfiStatus::InvalidInput,
            "solver.topological_ordering.self_loop_cycle",
        ),
        (
            include_bytes!(
                "../../../../examples/flowsheets/failures/unbound-inlet-port.rfproj.json"
            )
            .as_slice(),
            RfFfiStatus::InvalidConnection,
            "solver.connection_validation.unbound_inlet_port",
        ),
    ] {
        fs::write(f.root.join(PROJECT), project).unwrap();
        native.load(project);
        assert_eq!(native.solve(PACKAGE), expected_status);
        let native_error = native.error();
        assert_eq!(native_error["diagnosticCode"], expected_code);

        let app = run_studio(&f);
        let studio = app
            .workspace
            .solve_session
            .latest_diagnostic
            .as_ref()
            .unwrap();
        assert_eq!(studio.primary_code.as_deref(), Some(expected_code));
        assert!(latest_snapshot(&app.workspace).is_none());
        let units = json!(studio.related_unit_ids);
        let streams = json!(studio.related_stream_ids);
        let ports = &studio.related_port_targets;
        assert!(!ports.is_empty());
        assert_eq!(native_error["relatedUnitIds"], units);
        assert_eq!(native_error["relatedStreamIds"], streams);
        assert_eq!(
            native_error["relatedPortTargets"],
            json!(
                ports
                    .iter()
                    .map(|p| json!({"unitId":p.unit_id,"portName":p.port_name}))
                    .collect::<Vec<_>>()
            )
        );

        let mut request = f.request();
        request["document_id"] = json!(app.workspace.document.metadata.document_id.as_str());
        request["package_id"] = json!(PACKAGE);
        request["writes"] = json!([]);
        let (code, cli) = f.run(&request);
        assert_eq!(code, 4, "{cli}");
        assert_eq!(cli["status"], "failed");
        let diagnostic = &cli["diagnostic"];
        assert_eq!(diagnostic["code"], expected_code);
        assert_eq!(diagnostic["unit_ids"], units);
        assert_eq!(diagnostic["stream_ids"], streams);
        assert_eq!(
            diagnostic["ports"],
            json!(
                ports
                    .iter()
                    .map(|p| json!({"unit_id":p.unit_id,"port":p.port_name}))
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(cli["variables"], json!([]));
    }
}

#[test]
fn studio_cli_and_c_abi_keep_missing_thermal_data_at_solve_time() {
    let f = Fixture::new();
    let (manifest_path, payload_path, _) = package_paths(&f);
    let mut payload = rf_store::read_property_package_payload(&payload_path).unwrap();
    for component in &mut payload.components {
        component.vapor_heat_capacity_j_per_mol_k = None;
    }
    let integrity = rf_store::property_package_payload_integrity(&payload).unwrap();
    rf_store::write_property_package_payload(payload_path.as_path(), &payload).unwrap();
    let mut manifest = rf_store::read_property_package_manifest(&manifest_path).unwrap();
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    rf_store::write_property_package_manifest(&manifest_path, &manifest).unwrap();
    let index_path = f.root.join("index.json");
    let mut index = rf_store::read_auth_cache_index(&index_path).unwrap();
    index.property_packages[0].hash = integrity.hash;
    index.property_packages[0].size_bytes = integrity.size_bytes;
    write_auth_cache_index(index_path, &index).unwrap();

    let native = NativeEngine::new();
    assert_eq!(
        native.register_package(&manifest_path, &payload_path),
        RfFfiStatus::Ok
    );
    native.load(&fs::read(f.root.join(PROJECT)).unwrap());
    assert_eq!(native.solve(PACKAGE), RfFfiStatus::Thermo);
    let native_error = native.error();
    assert_eq!(native_error["diagnosticCode"], "solver.step.execution");
    assert!(
        native_error["message"]
            .as_str()
            .unwrap()
            .contains("heat capacity")
    );

    let app = run_studio(&f);
    assert!(latest_snapshot(&app.workspace).is_none());
    let studio = app
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .unwrap();
    assert_eq!(
        studio.primary_code.as_deref(),
        native_error["diagnosticCode"].as_str()
    );
    assert_eq!(
        json!(studio.related_unit_ids),
        native_error["relatedUnitIds"]
    );
    assert_eq!(
        json!(studio.related_stream_ids),
        native_error["relatedStreamIds"]
    );

    let mut request = f.request();
    request["writes"] = json!([]);
    let (code, cli) = f.run(&request);
    assert_eq!(code, 4, "{cli}");
    assert_eq!(cli["error"]["code"], "solve_failed");
    assert_eq!(cli["diagnostic"]["code"], native_error["diagnosticCode"]);
    assert_eq!(
        cli["diagnostic"]["unit_ids"],
        native_error["relatedUnitIds"]
    );
    assert_eq!(
        cli["diagnostic"]["stream_ids"],
        native_error["relatedStreamIds"]
    );
    assert_eq!(cli["variables"], json!([]));
}

#[test]
fn package_selection_and_expiration_keep_host_failure_categories() {
    let f = Fixture::new();
    let (manifest_path, payload_path, _) = package_paths(&f);
    let native = NativeEngine::new();
    native.load(&fs::read(f.root.join(PROJECT)).unwrap());
    let missing = "missing-package";
    assert_eq!(native.solve(missing), RfFfiStatus::MissingEntity);
    assert_eq!(native.error()["code"], "missing_entity");
    let mut request = f.request();
    request["writes"] = json!([]);
    request["package_id"] = json!(missing);
    let (code, cli) = f.run(&request);
    assert_eq!(code, 3, "{cli}");
    assert_eq!(cli["error"]["code"], "cached_package_missing");

    let expires_at = Some(UNIX_EPOCH + Duration::from_secs(1));
    let index_path = f.root.join("index.json");
    let mut index = rf_store::read_auth_cache_index(&index_path).unwrap();
    index.property_packages[0].downloaded_at = UNIX_EPOCH;
    index.property_packages[0].expires_at = expires_at;
    write_auth_cache_index(index_path, &index).unwrap();
    let mut manifest = rf_store::read_property_package_manifest(&manifest_path).unwrap();
    manifest.expires_at = expires_at;
    rf_store::write_property_package_manifest(&manifest_path, &manifest).unwrap();
    assert_eq!(
        native.register_package(&manifest_path, &payload_path),
        RfFfiStatus::InvalidInput,
    );
    assert_eq!(native.error()["diagnosticCode"], "property_package.expired");

    let app = run_studio(&f);
    assert!(latest_snapshot(&app.workspace).is_none());
    let diagnostic = app
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .unwrap();
    assert_eq!(
        diagnostic.primary_code.as_deref(),
        Some("workspace.run.property_package_missing")
    );
    request["package_id"] = json!(PACKAGE);
    let (code, cli) = f.run(&request);
    assert_eq!(code, 4, "{cli}");
    assert_eq!(cli["error"]["code"], "solve_failed");
    assert_eq!(
        cli["diagnostic"]["code"],
        "workspace.run.property_package_missing"
    );
    assert_eq!(cli["variables"], json!([]));
}
