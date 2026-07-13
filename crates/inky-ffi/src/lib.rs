use inky_core::migrate;
use inky_core::validate::{self, Severity};
use inky_core::{Config, Inky, OutputMode};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Read a C string argument. Returns None if the pointer is null.
/// Invalid UTF-8 is replaced with U+FFFD rather than rejected.
///
/// # Safety
/// `ptr` must be null or a valid null-terminated C string.
unsafe fn arg_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Run `f`, catching panics so they cannot unwind across the FFI boundary
/// (which would abort the host process). Returns null on panic.
fn ffi_result<F: FnOnce() -> String>(f: F) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(s) => CString::new(s).unwrap_or_default().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Transform Inky HTML to email-safe HTML.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(|| Inky::new().transform(&html))
}

/// Transform with custom column count.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_with_columns(
    input: *const c_char,
    column_count: u32,
) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || {
        let config = Config {
            column_count,
            ..Default::default()
        };
        Inky::with_config(config).transform(&html)
    })
}

/// Transform Inky HTML and inline CSS from `<style>` blocks.
/// Returns the result HTML, or the original transform output if inlining fails.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_inline(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(
        move || match Inky::new().transform_and_inline(&html, None) {
            Ok(r) => r,
            Err(_) => Inky::new().transform(&html),
        },
    )
}

/// Transform Inky HTML with MiniJinja data merge, then inline CSS.
///
/// `data_json` must be a valid JSON C string with merge variables.
/// Missing keys render as empty strings (lenient mode).
/// Returns null if any argument is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` and `data_json` must each be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_with_data(
    input: *const c_char,
    data_json: *const c_char,
) -> *mut c_char {
    let (Some(html), Some(json_str)) = (unsafe { arg_str(input) }, unsafe { arg_str(data_json) })
    else {
        return std::ptr::null_mut();
    };
    ffi_result(move || {
        let data: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        let merged = inky_core::templating::render_template(&html, &data, false)
            .unwrap_or_else(|_| html.to_string());
        match Inky::new().transform_and_inline(&merged, None) {
            Ok(r) => r,
            Err(_) => Inky::new().transform(&merged),
        }
    })
}

/// Transform using hybrid output mode (div + MSO ghost tables).
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_hybrid(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || {
        let config = Config {
            output_mode: OutputMode::Hybrid,
            ..Default::default()
        };
        Inky::with_config(config).transform(&html)
    })
}

/// Migrate v1 Inky syntax to v2.
/// Returns the migrated HTML string.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_migrate(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || migrate::migrate(&html).html)
}

/// Migrate v1 syntax and return a JSON string with `html` and `changes` fields.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_migrate_with_details(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || {
        let result = migrate::migrate(&html);
        let changes: Vec<String> = result
            .changes
            .iter()
            .map(|c| format!("\"{}\"", escape_json(&c.description)))
            .collect();
        format!(
            r#"{{"html":"{}","changes":[{}]}}"#,
            escape_json(&result.html),
            changes.join(",")
        )
    })
}

/// Validate an Inky template and return diagnostics as a JSON array.
/// Each entry has `severity`, `rule`, and `message` fields.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_validate(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || {
        let config = Config::default();
        diagnostics_to_json(&validate::validate(&html, &config))
    })
}

/// Convert HTML to plain text for multipart email.
/// Returns null if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_to_plain_text(input: *const c_char) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    ffi_result(move || inky_core::plaintext::html_to_plain_text(&html))
}

/// Get the Inky version string.
/// Caller must free the returned string with inky_free().
#[no_mangle]
pub extern "C" fn inky_version() -> *mut c_char {
    CString::new(env!("CARGO_PKG_VERSION"))
        .unwrap_or_default()
        .into_raw()
}

/// Free a string returned by any inky_* function.
///
/// # Safety
/// `ptr` must be a pointer returned by one of the inky_* functions, or null.
#[no_mangle]
pub unsafe extern "C" fn inky_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

fn diagnostics_to_json(diagnostics: &[validate::Diagnostic]) -> String {
    let items: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let severity = match d.severity {
                Severity::Warning => "warning",
                Severity::Error => "error",
            };
            format!(
                r#"{{"severity":"{}","rule":"{}","message":"{}"}}"#,
                severity,
                escape_json(d.rule),
                escape_json(&d.message)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_input_returns_null() {
        let out = unsafe { inky_transform(std::ptr::null()) };
        assert!(out.is_null());
        let out = unsafe { inky_transform_with_data(std::ptr::null(), std::ptr::null()) };
        assert!(out.is_null());
    }

    #[test]
    fn panic_is_caught_and_returns_null() {
        let out = ffi_result(|| panic!("boom"));
        assert!(out.is_null());
    }

    #[test]
    fn transform_round_trip_and_free() {
        let input = CString::new(r#"<button href="https://x.com">Go</button>"#).unwrap();
        let out = unsafe { inky_transform(input.as_ptr()) };
        assert!(!out.is_null());
        let s = unsafe { CStr::from_ptr(out) }.to_str().unwrap();
        assert!(s.contains("https://x.com"));
        unsafe { inky_free(out) };
    }
}
