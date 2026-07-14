use inky_core::migrate;
use inky_core::pipeline::{Pipeline, PipelineOptions};
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

/// Options accepted by `inky_build`. All fields optional; unknown keys ignored.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct BuildOptions {
    inline_css: Option<bool>,
    framework_css: Option<bool>,
    components_dir: Option<String>,
    columns: Option<u32>,
    hybrid: bool,
    bulletproof_buttons: bool,
    plain_text: bool,
    data: Option<serde_json::Value>,
}

fn error_envelope(message: &str, warnings: &[String]) -> String {
    serde_json::json!({ "ok": false, "error": message, "warnings": warnings }).to_string()
}

/// Run the full build pipeline: layout/include/custom-component resolution,
/// template data merge, framework SCSS compilation and injection, component
/// transform, CSS inlining, and output cleanup — the same pipeline `inky build`
/// runs.
///
/// `base_path` (nullable) is the directory used to resolve layouts, includes,
/// custom components, and linked SCSS/CSS. `options_json` (nullable) is a JSON
/// object; see BuildOptions for keys and defaults.
///
/// Returns a JSON envelope:
///   `{"ok": true, "html": "...", "warnings": [...]}` (+ `"text"` when
///   `plain_text` was requested), or
///   `{"ok": false, "error": "...", "warnings": [...]}`.
/// Returns null only if `input` is null or an internal error occurs.
/// Caller must free the returned string with inky_free().
///
/// # Safety
/// Each pointer must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn inky_build(
    input: *const c_char,
    base_path: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let Some(html) = (unsafe { arg_str(input) }) else {
        return std::ptr::null_mut();
    };
    let base = unsafe { arg_str(base_path) };
    let opts_raw = unsafe { arg_str(options_json) };

    ffi_result(move || {
        let opts: BuildOptions = match opts_raw.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(s) => match serde_json::from_str(s) {
                Ok(o) => o,
                Err(e) => return error_envelope(&format!("Invalid options JSON: {}", e), &[]),
            },
            None => BuildOptions::default(),
        };

        let config = Config {
            column_count: opts.columns.unwrap_or(12),
            output_mode: if opts.hybrid {
                OutputMode::Hybrid
            } else {
                OutputMode::Table
            },
            bulletproof_buttons: opts.bulletproof_buttons,
            ..Config::default()
        };
        let pipeline_options = PipelineOptions {
            inline_css: opts.inline_css.unwrap_or(true),
            framework_css: opts.framework_css.unwrap_or(true),
            components_dir: opts
                .components_dir
                .unwrap_or_else(|| "components".to_string()),
        };

        let pipeline = Pipeline::new(config, pipeline_options);
        match pipeline.process(
            &html,
            base.as_deref().map(std::path::Path::new),
            opts.data.as_ref(),
        ) {
            Ok(processed) => {
                let mut envelope = serde_json::json!({
                    "ok": true,
                    "html": processed.html,
                    "warnings": processed.warnings,
                });
                if opts.plain_text {
                    let text = inky_core::plaintext::html_to_plain_text(
                        envelope["html"].as_str().unwrap_or_default(),
                    );
                    envelope["text"] = serde_json::Value::String(text);
                }
                envelope.to_string()
            }
            Err(e) => error_envelope(&e.to_string(), &e.warnings),
        }
    })
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

    fn call_build(html: &str, base: Option<&str>, opts: Option<&str>) -> serde_json::Value {
        let html_c = CString::new(html).unwrap();
        let base_c = base.map(|s| CString::new(s).unwrap());
        let opts_c = opts.map(|s| CString::new(s).unwrap());
        let ptr = unsafe {
            inky_build(
                html_c.as_ptr(),
                base_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
                opts_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            )
        };
        assert!(!ptr.is_null(), "inky_build returned null");
        let s = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { inky_free(ptr) };
        serde_json::from_str(&s).expect("envelope is not valid JSON")
    }

    #[test]
    fn build_success_envelope() {
        let env = call_build(
            r#"<button href="https://x.dev">Go</button>"#,
            None,
            Some(r#"{"framework_css": false, "inline_css": false}"#),
        );
        assert_eq!(env["ok"], true);
        assert!(env["html"].as_str().unwrap().contains(r#"class="button""#));
        assert!(env["warnings"].as_array().unwrap().is_empty());
        assert!(env.get("text").is_none());
    }

    #[test]
    fn build_with_data_and_plain_text() {
        let env = call_build(
            "<p>Hi {{ name }}</p>",
            None,
            Some(
                r#"{"framework_css": false, "inline_css": false, "plain_text": true, "data": {"name": "Joe"}}"#,
            ),
        );
        assert_eq!(env["ok"], true);
        assert!(env["html"].as_str().unwrap().contains("Hi Joe"));
        assert!(env["text"].as_str().unwrap().contains("Hi Joe"));
    }

    #[test]
    fn build_error_envelope_with_prefix() {
        let dir = std::env::temp_dir().join("inky-ffi-build-err");
        std::fs::create_dir_all(&dir).unwrap();
        let env = call_build(
            r#"<layout src="nope.html"><p>x</p></layout>"#,
            Some(dir.to_str().unwrap()),
            None,
        );
        assert_eq!(env["ok"], false);
        assert!(env["error"]
            .as_str()
            .unwrap()
            .starts_with("Failed to load layout 'nope.html'"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_error_envelope_carries_warnings() {
        let dir = std::env::temp_dir().join("inky-ffi-build-warn");
        std::fs::create_dir_all(&dir).unwrap();
        let env = call_build(
            r#"<link rel="stylesheet" href="nope.scss"><style type="text/scss">$broken: {</style><p>x</p>"#,
            Some(dir.to_str().unwrap()),
            None,
        );
        assert_eq!(env["ok"], false);
        assert!(env["error"]
            .as_str()
            .unwrap()
            .starts_with("SCSS compilation failed:"));
        let warnings = env["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].as_str().unwrap().contains("nope.scss"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_invalid_options_is_envelope_not_null() {
        let env = call_build("<p>x</p>", None, Some("{not json"));
        assert_eq!(env["ok"], false);
        assert!(env["error"]
            .as_str()
            .unwrap()
            .starts_with("Invalid options JSON:"));
    }

    #[test]
    fn build_null_input_returns_null() {
        let ptr = unsafe { inky_build(std::ptr::null(), std::ptr::null(), std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn build_hybrid_and_columns_options() {
        let env = call_build(
            "<row><column>A</column><column>B</column></row>",
            None,
            Some(r#"{"framework_css": false, "inline_css": false, "hybrid": true, "columns": 12}"#),
        );
        assert_eq!(env["ok"], true);
        assert!(env["html"].as_str().unwrap().contains("<!--[if mso]>"));
    }
}
