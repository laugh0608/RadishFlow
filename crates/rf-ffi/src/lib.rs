mod engine;

use std::ffi::{CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

use engine::Engine;
use rf_types::{ErrorCode, RfError};
use serde::Serialize;

pub use engine::DEMO_PACKAGE_ID;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RfFfiStatus {
    Ok = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    InvalidEngineState = 3,
    Panic = 4,
    InvalidInput = 100,
    DuplicateId = 101,
    MissingEntity = 102,
    InvalidConnection = 103,
    Thermo = 104,
    Flash = 105,
    NotImplemented = 106,
}

impl RfFfiStatus {
    const fn from_error_code(code: ErrorCode) -> Self {
        match code {
            ErrorCode::InvalidInput => Self::InvalidInput,
            ErrorCode::DuplicateId => Self::DuplicateId,
            ErrorCode::MissingEntity => Self::MissingEntity,
            ErrorCode::InvalidConnection => Self::InvalidConnection,
            ErrorCode::Thermo => Self::Thermo,
            ErrorCode::Flash => Self::Flash,
            ErrorCode::NotImplemented => Self::NotImplemented,
        }
    }
}

/// # Safety
///
/// `out_engine` must be a valid writable pointer to receive the newly allocated engine handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_create(out_engine: *mut *mut Engine) -> RfFfiStatus {
    catch_unwind(AssertUnwindSafe(|| {
        if out_engine.is_null() {
            return RfFfiStatus::NullPointer;
        }

        let engine = Box::new(Engine::new());
        unsafe {
            *out_engine = Box::into_raw(engine);
        }
        RfFfiStatus::Ok
    }))
    .unwrap_or(RfFfiStatus::Panic)
}

/// # Safety
///
/// `engine` must either be null or a handle previously returned by `engine_create` that has not
/// already been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_destroy(engine: *mut Engine) {
    if engine.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(engine));
    }
}

/// # Safety
///
/// `engine` must be a valid engine handle and `out_message` must be a valid writable pointer.
/// The returned string must be released with `rf_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_last_error_message(
    engine: *const Engine,
    out_message: *mut *mut c_char,
) -> RfFfiStatus {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || out_message.is_null() {
            return RfFfiStatus::NullPointer;
        }

        let engine = unsafe { &*engine };
        match allocate_c_string(engine.last_error_message(), out_message) {
            Ok(()) => RfFfiStatus::Ok,
            Err(_) => RfFfiStatus::InvalidInput,
        }
    }))
    .unwrap_or(RfFfiStatus::Panic)
}

/// # Safety
///
/// `engine` must be a valid engine handle and `out_message` must be a valid writable pointer.
/// The returned string must be released with `rf_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_last_error_json(
    engine: *const Engine,
    out_message: *mut *mut c_char,
) -> RfFfiStatus {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || out_message.is_null() {
            return RfFfiStatus::NullPointer;
        }

        let engine = unsafe { &*engine };
        let json = match engine.last_error() {
            Some(error) => match serde_json::to_string_pretty(&FfiErrorJson::from_error(error)) {
                Ok(json) => json,
                Err(_) => return RfFfiStatus::InvalidInput,
            },
            None => "null".to_string(),
        };

        match allocate_c_string(&json, out_message) {
            Ok(()) => RfFfiStatus::Ok,
            Err(_) => RfFfiStatus::InvalidInput,
        }
    }))
    .unwrap_or(RfFfiStatus::Panic)
}

/// # Safety
///
/// `value` must either be null or a string pointer returned by this library that has not already
/// been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rf_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}

/// # Safety
///
/// `engine` must be a valid engine handle. When `json_len` is non-zero, `json_ptr` must point to
/// `json_len` bytes of valid readable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowsheet_load_json(
    engine: *mut Engine,
    json_ptr: *const u8,
    json_len: usize,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        let json = read_utf8_bytes(json_ptr, json_len)?;
        engine.load_flowsheet_json(&json)
    })
}

/// # Safety
///
/// `engine` must be a valid engine handle. When `package_id_len` is non-zero, `package_id_ptr`
/// must point to `package_id_len` bytes of valid readable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowsheet_solve(
    engine: *mut Engine,
    package_id_ptr: *const u8,
    package_id_len: usize,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        let package_id = read_utf8_bytes(package_id_ptr, package_id_len)?;
        engine.solve_flowsheet(&package_id)
    })
}

/// # Safety
///
/// `engine` must be a valid engine handle. Non-empty manifest and payload path inputs must point
/// to readable byte ranges with the supplied lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn property_package_load_from_files(
    engine: *mut Engine,
    manifest_path_ptr: *const u8,
    manifest_path_len: usize,
    payload_path_ptr: *const u8,
    payload_path_len: usize,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        let manifest_path = read_utf8_bytes(manifest_path_ptr, manifest_path_len)?;
        let payload_path = read_utf8_bytes(payload_path_ptr, payload_path_len)?;

        if manifest_path.trim().is_empty() {
            return Err(RfError::invalid_input(
                "ffi property package load requires a non-empty manifest path",
            ));
        }

        if payload_path.trim().is_empty() {
            return Err(RfError::invalid_input(
                "ffi property package load requires a non-empty payload path",
            ));
        }

        engine.load_property_package_files(&manifest_path, &payload_path)?;
        Ok(())
    })
}

/// # Safety
///
/// `engine` must be a valid engine handle and `out_json` must be a valid writable pointer. The
/// returned string must be released with `rf_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn property_package_list_json(
    engine: *mut Engine,
    out_json: *mut *mut c_char,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        if out_json.is_null() {
            return Err(ffi_error(
                RfFfiStatus::NullPointer,
                "ffi property package list export requires a non-null output pointer",
            ));
        }

        let json = engine.property_package_list_json()?;
        allocate_c_string(&json, out_json).map_err(|error| {
            RfError::invalid_input(format!("failed to allocate ffi output string: {error}"))
        })?;
        Ok(())
    })
}

/// # Safety
///
/// `engine` must be a valid engine handle. When `stream_id_len` is non-zero, `stream_id_ptr` must
/// point to `stream_id_len` readable bytes. `out_json` must be a valid writable pointer, and the
/// returned string must be released with `rf_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stream_get_snapshot_json(
    engine: *mut Engine,
    stream_id_ptr: *const u8,
    stream_id_len: usize,
    out_json: *mut *mut c_char,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        if out_json.is_null() {
            return Err(ffi_error(
                RfFfiStatus::NullPointer,
                "ffi stream snapshot export requires a non-null output pointer",
            ));
        }

        let stream_id = read_utf8_bytes(stream_id_ptr, stream_id_len)?;
        let json = engine.stream_snapshot_json(&stream_id)?;
        allocate_c_string(&json, out_json).map_err(|error| {
            RfError::invalid_input(format!("failed to allocate ffi output string: {error}"))
        })?;
        Ok(())
    })
}

/// # Safety
///
/// `engine` must be a valid engine handle and `out_json` must be a valid writable pointer. The
/// returned string must be released with `rf_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowsheet_get_snapshot_json(
    engine: *mut Engine,
    out_json: *mut *mut c_char,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        if out_json.is_null() {
            return Err(ffi_error(
                RfFfiStatus::NullPointer,
                "ffi solve snapshot export requires a non-null output pointer",
            ));
        }

        let json = engine.flowsheet_snapshot_json()?;
        allocate_c_string(&json, out_json).map_err(|error| {
            RfError::invalid_input(format!("failed to allocate ffi output string: {error}"))
        })?;
        Ok(())
    })
}

fn with_engine_mut(
    engine: *mut Engine,
    action: impl FnOnce(&mut Engine) -> Result<(), RfError>,
) -> RfFfiStatus {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() {
            return RfFfiStatus::NullPointer;
        }

        let engine = unsafe { &mut *engine };
        match action(engine) {
            Ok(()) => {
                engine.clear_last_error();
                RfFfiStatus::Ok
            }
            Err(error) => {
                engine.replace_last_error(error.clone());
                map_error_to_status(&error)
            }
        }
    }))
    .unwrap_or_else(|_| {
        if !engine.is_null() {
            unsafe {
                (*engine).replace_last_error(ffi_error(
                    RfFfiStatus::Panic,
                    "ffi call panicked unexpectedly",
                ));
            }
        }
        RfFfiStatus::Panic
    })
}

fn read_utf8_bytes(ptr: *const u8, len: usize) -> Result<String, RfError> {
    if len == 0 {
        return Ok(String::new());
    }

    if ptr.is_null() {
        return Err(ffi_error(
            RfFfiStatus::NullPointer,
            "ffi string input requires a non-null pointer when length is non-zero",
        ));
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes)
        .map_err(|error| {
            ffi_error(
                RfFfiStatus::InvalidUtf8,
                format!("ffi string input must be valid UTF-8: {error}"),
            )
        })
        .map(str::to_owned)
}

fn ffi_error(status: RfFfiStatus, message: impl Into<String>) -> RfError {
    let message = message.into();
    let diagnostic_code = match status {
        RfFfiStatus::NullPointer => "ffi.null_pointer",
        RfFfiStatus::InvalidUtf8 => "ffi.invalid_utf8",
        RfFfiStatus::InvalidEngineState => "ffi.invalid_engine_state",
        RfFfiStatus::Panic => "ffi.panic",
        _ => "ffi.invalid_input",
    };
    let base = match status {
        RfFfiStatus::MissingEntity => RfError::missing_entity("ffi value", "unknown"),
        RfFfiStatus::InvalidConnection => RfError::invalid_connection(message.clone()),
        RfFfiStatus::Thermo => RfError::thermo(message.clone()),
        RfFfiStatus::Flash => RfError::flash(message.clone()),
        RfFfiStatus::NotImplemented => RfError::not_implemented(message.clone()),
        _ => RfError::invalid_input(message),
    };
    base.with_diagnostic_code(diagnostic_code)
}

fn allocate_c_string(value: &str, out: *mut *mut c_char) -> Result<(), std::ffi::NulError> {
    let c_string = CString::new(value)?;
    unsafe {
        *out = c_string.into_raw();
    }
    Ok(())
}

fn map_error_to_status(error: &RfError) -> RfFfiStatus {
    match error.context().diagnostic_code() {
        Some("ffi.null_pointer") => RfFfiStatus::NullPointer,
        Some("ffi.invalid_utf8") => RfFfiStatus::InvalidUtf8,
        Some(code) if code.starts_with("ffi.engine_state.") => RfFfiStatus::InvalidEngineState,
        Some("ffi.invalid_engine_state") => RfFfiStatus::InvalidEngineState,
        Some("ffi.panic") => RfFfiStatus::Panic,
        _ => RfFfiStatus::from_error_code(error.code()),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiErrorJson {
    ffi_status: &'static str,
    code: &'static str,
    message: String,
    diagnostic_code: Option<String>,
    related_unit_ids: Vec<String>,
    related_stream_ids: Vec<String>,
    related_port_targets: Vec<FfiPortTargetJson>,
}

impl FfiErrorJson {
    fn from_error(error: &RfError) -> Self {
        let ffi_status = map_error_to_status(error);
        Self {
            ffi_status: ffi_status.as_str(),
            code: error.code().as_str(),
            message: error.message().to_string(),
            diagnostic_code: error.context().diagnostic_code().map(str::to_string),
            related_unit_ids: error
                .context()
                .related_unit_ids()
                .iter()
                .map(|unit_id| unit_id.as_str().to_string())
                .collect(),
            related_stream_ids: error
                .context()
                .related_stream_ids()
                .iter()
                .map(|stream_id| stream_id.as_str().to_string())
                .collect(),
            related_port_targets: error
                .context()
                .related_port_targets()
                .iter()
                .map(FfiPortTargetJson::from_target)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FfiPortTargetJson {
    unit_id: String,
    port_name: String,
}

impl FfiPortTargetJson {
    fn from_target(target: &rf_types::DiagnosticPortTarget) -> Self {
        Self {
            unit_id: target.unit_id.as_str().to_string(),
            port_name: target.port_name.clone(),
        }
    }
}

impl RfFfiStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NullPointer => "null_pointer",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::InvalidEngineState => "invalid_engine_state",
            Self::Panic => "panic",
            Self::InvalidInput => "invalid_input",
            Self::DuplicateId => "duplicate_id",
            Self::MissingEntity => "missing_entity",
            Self::InvalidConnection => "invalid_connection",
            Self::Thermo => "thermo",
            Self::Flash => "flash",
            Self::NotImplemented => "not_implemented",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEMO_PACKAGE_ID, Engine, RfFfiStatus, engine_create as raw_engine_create,
        engine_destroy as raw_engine_destroy, engine_last_error_json as raw_engine_last_error_json,
        engine_last_error_message as raw_engine_last_error_message,
        flowsheet_get_snapshot_json as raw_flowsheet_get_snapshot_json,
        flowsheet_load_json as raw_flowsheet_load_json, flowsheet_solve as raw_flowsheet_solve,
        property_package_list_json as raw_property_package_list_json,
        property_package_load_from_files as raw_property_package_load_from_files,
        rf_string_free as raw_rf_string_free,
        stream_get_snapshot_json as raw_stream_get_snapshot_json,
    };
    use std::ffi::{CStr, c_char};
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::{Component, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
    use rf_store::{
        StoredAntoineCoefficients, StoredDocumentMetadata, StoredProjectFile,
        StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageSource,
        StoredThermoComponent, write_property_package_manifest, write_property_package_payload,
    };
    use rf_types::{ComponentId, PortDirection, PortKind};

    fn example_project_json() -> &'static str {
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-rf-ffi-{name}-{unique}"))
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn engine_create(out_engine: *mut *mut Engine) -> RfFfiStatus {
        unsafe { raw_engine_create(out_engine) }
    }

    fn engine_destroy(engine: *mut Engine) {
        unsafe { raw_engine_destroy(engine) }
    }

    fn engine_last_error_message(
        engine: *const Engine,
        out_message: *mut *mut c_char,
    ) -> RfFfiStatus {
        unsafe { raw_engine_last_error_message(engine, out_message) }
    }

    fn engine_last_error_json(engine: *const Engine, out_message: *mut *mut c_char) -> RfFfiStatus {
        unsafe { raw_engine_last_error_json(engine, out_message) }
    }

    fn rf_string_free(value: *mut c_char) {
        unsafe { raw_rf_string_free(value) }
    }

    fn flowsheet_load_json(
        engine: *mut Engine,
        json_ptr: *const u8,
        json_len: usize,
    ) -> RfFfiStatus {
        unsafe { raw_flowsheet_load_json(engine, json_ptr, json_len) }
    }

    fn flowsheet_solve(
        engine: *mut Engine,
        package_id_ptr: *const u8,
        package_id_len: usize,
    ) -> RfFfiStatus {
        unsafe { raw_flowsheet_solve(engine, package_id_ptr, package_id_len) }
    }

    fn property_package_load_from_files(
        engine: *mut Engine,
        manifest_path_ptr: *const u8,
        manifest_path_len: usize,
        payload_path_ptr: *const u8,
        payload_path_len: usize,
    ) -> RfFfiStatus {
        unsafe {
            raw_property_package_load_from_files(
                engine,
                manifest_path_ptr,
                manifest_path_len,
                payload_path_ptr,
                payload_path_len,
            )
        }
    }

    fn property_package_list_json(engine: *mut Engine, out_json: *mut *mut c_char) -> RfFfiStatus {
        unsafe { raw_property_package_list_json(engine, out_json) }
    }

    fn stream_get_snapshot_json(
        engine: *mut Engine,
        stream_id_ptr: *const u8,
        stream_id_len: usize,
        out_json: *mut *mut c_char,
    ) -> RfFfiStatus {
        unsafe { raw_stream_get_snapshot_json(engine, stream_id_ptr, stream_id_len, out_json) }
    }

    fn flowsheet_get_snapshot_json(engine: *mut Engine, out_json: *mut *mut c_char) -> RfFfiStatus {
        unsafe { raw_flowsheet_get_snapshot_json(engine, out_json) }
    }

    fn sample_runtime_project_json() -> String {
        let mut flowsheet = Flowsheet::new("ffi-package-load");
        flowsheet
            .insert_component(Component::new("component-a", "Component A"))
            .expect("expected component");
        flowsheet
            .insert_component(Component::new("component-b", "Component B"))
            .expect("expected component");
        flowsheet
            .insert_stream(MaterialStreamState::from_tpzf(
                "stream-feed",
                "Feed",
                300.0,
                120_000.0,
                5.0,
                vec![
                    (ComponentId::new("component-a"), 0.35),
                    (ComponentId::new("component-b"), 0.65),
                ]
                .into_iter()
                .collect(),
            ))
            .expect("expected stream");
        flowsheet
            .insert_stream(MaterialStreamState::from_tpzf(
                "stream-heated",
                "Heated Outlet",
                345.0,
                95_000.0,
                0.0,
                vec![
                    (ComponentId::new("component-a"), 0.5),
                    (ComponentId::new("component-b"), 0.5),
                ]
                .into_iter()
                .collect(),
            ))
            .expect("expected stream");
        flowsheet
            .insert_stream(MaterialStreamState::from_tpzf(
                "stream-liquid",
                "Liquid Outlet",
                345.0,
                95_000.0,
                0.0,
                vec![
                    (ComponentId::new("component-a"), 0.5),
                    (ComponentId::new("component-b"), 0.5),
                ]
                .into_iter()
                .collect(),
            ))
            .expect("expected stream");
        flowsheet
            .insert_stream(MaterialStreamState::from_tpzf(
                "stream-vapor",
                "Vapor Outlet",
                345.0,
                95_000.0,
                0.0,
                vec![
                    (ComponentId::new("component-a"), 0.5),
                    (ComponentId::new("component-b"), 0.5),
                ]
                .into_iter()
                .collect(),
            ))
            .expect("expected stream");
        flowsheet
            .insert_unit(UnitNode::new(
                "feed-1",
                "Feed",
                "feed",
                vec![UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                )],
            ))
            .expect("expected unit");
        flowsheet
            .insert_unit(UnitNode::new(
                "heater-1",
                "Heater",
                "heater",
                vec![
                    UnitPort::new(
                        "inlet",
                        PortDirection::Inlet,
                        PortKind::Material,
                        Some("stream-feed".into()),
                    ),
                    UnitPort::new(
                        "outlet",
                        PortDirection::Outlet,
                        PortKind::Material,
                        Some("stream-heated".into()),
                    ),
                ],
            ))
            .expect("expected unit");
        flowsheet
            .insert_unit(UnitNode::new(
                "flash-1",
                "Flash Drum",
                "flash_drum",
                vec![
                    UnitPort::new(
                        "inlet",
                        PortDirection::Inlet,
                        PortKind::Material,
                        Some("stream-heated".into()),
                    ),
                    UnitPort::new(
                        "liquid",
                        PortDirection::Outlet,
                        PortKind::Material,
                        Some("stream-liquid".into()),
                    ),
                    UnitPort::new(
                        "vapor",
                        PortDirection::Outlet,
                        PortKind::Material,
                        Some("stream-vapor".into()),
                    ),
                ],
            ))
            .expect("expected unit");

        let project = StoredProjectFile::new(
            flowsheet,
            StoredDocumentMetadata::new("ffi-project", "FFI Package Load Demo", timestamp(10)),
        );
        serde_json::to_string_pretty(&project).expect("expected project json")
    }

    fn write_runtime_package_files(root: &Path, package_id: &str) -> (PathBuf, PathBuf) {
        let manifest_path = root.join("manifest.json");
        let payload_path = root.join("payload.rfpkg");
        let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(StoredAntoineCoefficients::new(
            ((2.0_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));
        let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(StoredAntoineCoefficients::new(
            ((0.5_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let manifest = StoredPropertyPackageManifest::new(
            package_id,
            "2026.04.14",
            StoredPropertyPackageSource::LocalBundled,
            vec![
                ComponentId::new("component-a"),
                ComponentId::new("component-b"),
            ],
        );
        let payload =
            StoredPropertyPackagePayload::new(package_id, "2026.04.14", vec![first, second]);

        write_property_package_manifest(&manifest_path, &manifest)
            .expect("expected manifest write");
        write_property_package_payload(&payload_path, &payload).expect("expected payload write");

        (manifest_path, payload_path)
    }

    fn call_last_error(engine: *mut Engine) -> String {
        let mut output = ptr::null_mut::<c_char>();
        let status = engine_last_error_message(engine.cast_const(), &mut output);
        assert_eq!(status, RfFfiStatus::Ok);
        let text = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);
        text
    }

    fn call_last_error_json(engine: *mut Engine) -> serde_json::Value {
        let mut output = ptr::null_mut::<c_char>();
        let status = engine_last_error_json(engine.cast_const(), &mut output);
        assert_eq!(status, RfFfiStatus::Ok);
        let text = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);
        serde_json::from_str(&text).expect("expected json")
    }

    #[test]
    fn ffi_engine_loads_solves_and_exports_stream_json() {
        let mut engine = ptr::null_mut::<Engine>();
        let create_status = engine_create(&mut engine);
        assert_eq!(create_status, RfFfiStatus::Ok);
        assert!(!engine.is_null());

        let project_json = example_project_json().as_bytes();
        let load_status = flowsheet_load_json(engine, project_json.as_ptr(), project_json.len());
        assert_eq!(load_status, RfFfiStatus::Ok);

        let package_id = DEMO_PACKAGE_ID.as_bytes();
        let solve_status = flowsheet_solve(engine, package_id.as_ptr(), package_id.len());
        assert_eq!(solve_status, RfFfiStatus::Ok);

        let stream_id = b"stream-vapor";
        let mut output = ptr::null_mut::<c_char>();
        let export_status =
            stream_get_snapshot_json(engine, stream_id.as_ptr(), stream_id.len(), &mut output);
        assert_eq!(export_status, RfFfiStatus::Ok);

        let json = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);

        let value: serde_json::Value = serde_json::from_str(&json).expect("expected json");
        assert_eq!(value["id"], "stream-vapor");
        assert_eq!(value["name"], "Vapor Outlet");
        assert!(
            value["phases"]
                .as_array()
                .is_some_and(|phases| !phases.is_empty())
        );
        assert_eq!(call_last_error(engine), "");

        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_lists_registered_property_packages_as_json() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let root = unique_temp_path("package-list");
        std::fs::create_dir_all(&root).expect("expected temp dir");
        let (manifest_path, payload_path) =
            write_runtime_package_files(&root, "runtime-binary-package");
        let manifest = manifest_path.to_string_lossy().to_string();
        let payload = payload_path.to_string_lossy().to_string();
        assert_eq!(
            property_package_load_from_files(
                engine,
                manifest.as_bytes().as_ptr(),
                manifest.len(),
                payload.as_bytes().as_ptr(),
                payload.len(),
            ),
            RfFfiStatus::Ok
        );

        let mut output = ptr::null_mut::<c_char>();
        let status = property_package_list_json(engine, &mut output);
        assert_eq!(status, RfFfiStatus::Ok);
        let text = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);
        let value: serde_json::Value = serde_json::from_str(&text).expect("expected json");

        let packages = value.as_array().expect("expected manifest array");
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0]["packageId"], "binary-hydrocarbon-lite-v1");
        assert_eq!(packages[0]["source"], "local-bundled");
        assert_eq!(packages[1]["packageId"], "runtime-binary-package");
        assert_eq!(packages[1]["version"], "2026.04.14");

        std::fs::remove_dir_all(&root).ok();
        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_exports_full_solve_snapshot_json() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let project_json = example_project_json().as_bytes();
        assert_eq!(
            flowsheet_load_json(engine, project_json.as_ptr(), project_json.len()),
            RfFfiStatus::Ok
        );

        let package_id = DEMO_PACKAGE_ID.as_bytes();
        assert_eq!(
            flowsheet_solve(engine, package_id.as_ptr(), package_id.len()),
            RfFfiStatus::Ok
        );

        let mut output = ptr::null_mut::<c_char>();
        let status = flowsheet_get_snapshot_json(engine, &mut output);
        assert_eq!(status, RfFfiStatus::Ok);

        let text = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);
        let value: serde_json::Value = serde_json::from_str(&text).expect("expected json");

        assert_eq!(value["status"], "converged");
        assert_eq!(value["summary"]["diagnosticCount"], 4);
        assert_eq!(value["steps"].as_array().map(Vec::len), Some(3));
        assert_eq!(value["streams"].as_array().map(Vec::len), Some(4));
        assert_eq!(value["diagnostics"][0]["code"], "solver.execution_order");

        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_loads_runtime_property_package_files_and_solves() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let root = unique_temp_path("package-files");
        std::fs::create_dir_all(&root).expect("expected temp dir");
        let (manifest_path, payload_path) =
            write_runtime_package_files(&root, "runtime-binary-package");

        let manifest = manifest_path.to_string_lossy().to_string();
        let payload = payload_path.to_string_lossy().to_string();
        assert_eq!(
            property_package_load_from_files(
                engine,
                manifest.as_bytes().as_ptr(),
                manifest.len(),
                payload.as_bytes().as_ptr(),
                payload.len(),
            ),
            RfFfiStatus::Ok
        );

        let project_json = sample_runtime_project_json();
        assert_eq!(
            flowsheet_load_json(engine, project_json.as_bytes().as_ptr(), project_json.len()),
            RfFfiStatus::Ok
        );

        let package_id = b"runtime-binary-package";
        assert_eq!(
            flowsheet_solve(engine, package_id.as_ptr(), package_id.len()),
            RfFfiStatus::Ok
        );

        let mut output = ptr::null_mut::<c_char>();
        assert_eq!(
            flowsheet_get_snapshot_json(engine, &mut output),
            RfFfiStatus::Ok
        );
        let text = unsafe { CStr::from_ptr(output) }
            .to_str()
            .expect("expected utf-8")
            .to_string();
        rf_string_free(output);
        let value: serde_json::Value = serde_json::from_str(&text).expect("expected json");
        assert_eq!(value["status"], "converged");
        assert_eq!(value["streams"].as_array().map(Vec::len), Some(4));

        std::fs::remove_dir_all(&root).ok();
        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_reports_missing_manifest_during_property_package_load() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let missing_manifest = "D:/Code/RadishFlow/does-not-exist-manifest.json".to_string();
        let missing_payload = "D:/Code/RadishFlow/does-not-exist-payload.rfpkg".to_string();

        let status = property_package_load_from_files(
            engine,
            missing_manifest.as_bytes().as_ptr(),
            missing_manifest.len(),
            missing_payload.as_bytes().as_ptr(),
            missing_payload.len(),
        );
        assert_eq!(status, RfFfiStatus::MissingEntity);
        let json = call_last_error_json(engine);
        assert_eq!(json["ffiStatus"], "missing_entity");
        assert_eq!(json["code"], "missing_entity");

        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_reports_missing_package_through_last_error() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let project_json = example_project_json().as_bytes();
        assert_eq!(
            flowsheet_load_json(engine, project_json.as_ptr(), project_json.len()),
            RfFfiStatus::Ok
        );

        let package_id = b"missing-package";
        let status = flowsheet_solve(engine, package_id.as_ptr(), package_id.len());
        assert_eq!(status, RfFfiStatus::MissingEntity);
        assert!(call_last_error(engine).contains("missing property package `missing-package`"));
        let json = call_last_error_json(engine);
        assert_eq!(json["ffiStatus"], "missing_entity");
        assert_eq!(json["code"], "missing_entity");
        assert_eq!(
            json["message"],
            "missing property package `missing-package`"
        );
        assert!(json["diagnosticCode"].is_null());

        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_requires_solve_before_exporting_stream_json() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let project_json = example_project_json().as_bytes();
        assert_eq!(
            flowsheet_load_json(engine, project_json.as_ptr(), project_json.len()),
            RfFfiStatus::Ok
        );

        let stream_id = b"stream-vapor";
        let mut output = ptr::null_mut::<c_char>();
        let status =
            stream_get_snapshot_json(engine, stream_id.as_ptr(), stream_id.len(), &mut output);
        assert_eq!(status, RfFfiStatus::InvalidEngineState);
        assert!(output.is_null());
        assert!(
            call_last_error(engine).contains("must solve a flowsheet before exporting streams")
        );
        let json = call_last_error_json(engine);
        assert_eq!(json["ffiStatus"], "invalid_engine_state");
        assert_eq!(json["code"], "invalid_input");
        assert_eq!(
            json["diagnosticCode"],
            "ffi.engine_state.snapshot_not_available"
        );

        engine_destroy(engine);
    }

    #[test]
    fn ffi_engine_requires_solve_before_exporting_full_snapshot_json() {
        let mut engine = ptr::null_mut::<Engine>();
        assert_eq!(engine_create(&mut engine), RfFfiStatus::Ok);

        let project_json = example_project_json().as_bytes();
        assert_eq!(
            flowsheet_load_json(engine, project_json.as_ptr(), project_json.len()),
            RfFfiStatus::Ok
        );

        let mut output = ptr::null_mut::<c_char>();
        let status = flowsheet_get_snapshot_json(engine, &mut output);
        assert_eq!(status, RfFfiStatus::InvalidEngineState);
        assert!(output.is_null());
        let json = call_last_error_json(engine);
        assert_eq!(json["ffiStatus"], "invalid_engine_state");
        assert_eq!(
            json["diagnosticCode"],
            "ffi.engine_state.snapshot_not_available"
        );

        engine_destroy(engine);
    }
}
