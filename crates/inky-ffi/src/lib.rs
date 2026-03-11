use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use inky_core::{Inky, Config};

/// Transform Inky HTML to email-safe HTML.
/// Caller must free the returned string with inky_free().
#[no_mangle]
pub extern "C" fn inky_transform(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = Inky::new().transform(html);
    CString::new(result).unwrap_or_default().into_raw()
}

/// Transform with custom column count.
/// Caller must free the returned string with inky_free().
#[no_mangle]
pub extern "C" fn inky_transform_with_columns(
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

/// Free a string returned by inky_transform or inky_transform_with_columns.
#[no_mangle]
pub extern "C" fn inky_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}
