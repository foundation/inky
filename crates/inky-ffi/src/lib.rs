use inky_core::migrate;
use inky_core::validate::{self, Severity};
use inky_core::{Config, Inky};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Transform Inky HTML to email-safe HTML.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = Inky::new().transform(html);
    CString::new(result).unwrap_or_default().into_raw()
}

/// Transform with custom column count.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_with_columns(
    input: *const c_char,
    column_count: u32,
) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let config = Config {
        column_count,
        ..Default::default()
    };
    let result = Inky::with_config(config).transform(html);
    CString::new(result).unwrap_or_default().into_raw()
}

/// Transform Inky HTML and inline CSS from `<style>` blocks.
/// Returns the result HTML, or the original transform output if inlining fails.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_inline(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = match Inky::new().transform_and_inline(html, None) {
        Ok(r) => r,
        Err(_) => Inky::new().transform(html),
    };
    CString::new(result).unwrap_or_default().into_raw()
}

/// Transform Inky HTML with MiniJinja data merge, then inline CSS.
///
/// `data_json` must be a valid JSON C string with merge variables.
/// Missing keys render as empty strings (lenient mode).
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` and `data_json` must be valid, non-null, null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn inky_transform_with_data(
    input: *const c_char,
    data_json: *const c_char,
) -> *mut c_char {
    let html = unsafe { CStr::from_ptr(input) }.to_str().unwrap_or("");
    let json_str = unsafe { CStr::from_ptr(data_json) }
        .to_str()
        .unwrap_or("{}");
    let data: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();
    let merged = inky_core::templating::render_template(html, &data, false)
        .unwrap_or_else(|_| html.to_string());
    let result = match Inky::new().transform_and_inline(&merged, None) {
        Ok(r) => r,
        Err(_) => Inky::new().transform(&merged),
    };
    CString::new(result).unwrap_or_default().into_raw()
}

/// Migrate v1 Inky syntax to v2.
/// Returns the migrated HTML string.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_migrate(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = migrate::migrate(html).html;
    CString::new(result).unwrap_or_default().into_raw()
}

/// Migrate v1 syntax and return a JSON string with `html` and `changes` fields.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_migrate_with_details(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = migrate::migrate(html);
    let changes: Vec<String> = result
        .changes
        .iter()
        .map(|c| format!("\"{}\"", escape_json(&c.description)))
        .collect();
    let json = format!(
        r#"{{"html":"{}","changes":[{}]}}"#,
        escape_json(&result.html),
        changes.join(",")
    );
    CString::new(json).unwrap_or_default().into_raw()
}

/// Validate an Inky template and return diagnostics as a JSON array.
/// Each entry has `severity`, `rule`, and `message` fields.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// `input` must be a valid, non-null, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_validate(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let config = Config::default();
    let diagnostics = validate::validate(html, &config);
    let json = diagnostics_to_json(&diagnostics);
    CString::new(json).unwrap_or_default().into_raw()
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
