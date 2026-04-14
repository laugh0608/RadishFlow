mod engine;

use std::ffi::{CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

use engine::Engine;
use rf_types::{ErrorCode, RfError};

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

#[unsafe(no_mangle)]
pub extern "C" fn engine_create(out_engine: *mut *mut Engine) -> RfFfiStatus {
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

#[unsafe(no_mangle)]
pub extern "C" fn engine_destroy(engine: *mut Engine) {
    if engine.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(engine));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_last_error_message(
    engine: *const Engine,
    out_message: *mut *mut c_char,
) -> RfFfiStatus {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || out_message.is_null() {
            return RfFfiStatus::NullPointer;
        }

        let engine = unsafe { &*engine };
        match allocate_c_string(engine.last_error(), out_message) {
            Ok(()) => RfFfiStatus::Ok,
            Err(_) => RfFfiStatus::InvalidInput,
        }
    }))
    .unwrap_or(RfFfiStatus::Panic)
}

#[unsafe(no_mangle)]
pub extern "C" fn rf_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn flowsheet_load_json(
    engine: *mut Engine,
    json_ptr: *const u8,
    json_len: usize,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        let json = read_utf8_bytes(json_ptr, json_len)?;
        engine.load_flowsheet_json(&json)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn flowsheet_solve(
    engine: *mut Engine,
    package_id_ptr: *const u8,
    package_id_len: usize,
) -> RfFfiStatus {
    with_engine_mut(engine, |engine| {
        let package_id = read_utf8_bytes(package_id_ptr, package_id_len)?;
        engine.solve_flowsheet(&package_id)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_get_snapshot_json(
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
                engine.replace_last_error(error.message());
                map_error_to_status(&error)
            }
        }
    }))
    .unwrap_or_else(|_| {
        if !engine.is_null() {
            unsafe {
                (*engine).replace_last_error("ffi call panicked unexpectedly");
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
    std::str::from_utf8(bytes).map_err(|error| {
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

#[cfg(test)]
mod tests {
    use super::{
        DEMO_PACKAGE_ID, Engine, RfFfiStatus, engine_create, engine_destroy,
        engine_last_error_message, flowsheet_load_json, flowsheet_solve, rf_string_free,
        stream_get_snapshot_json,
    };
    use std::ffi::{CStr, c_char};
    use std::ptr;

    fn example_project_json() -> &'static str {
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
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
        assert!(value["phases"].as_array().is_some_and(|phases| !phases.is_empty()));
        assert_eq!(call_last_error(engine), "");

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
        assert!(call_last_error(engine).contains("must solve a flowsheet before exporting streams"));

        engine_destroy(engine);
    }
}
