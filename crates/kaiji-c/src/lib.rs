use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Normalize a NUL-terminated UTF-8 string.
/// Returns a heap-allocated NUL-terminated string that must be freed with kaiji_free_string().
/// Returns NULL on error.
#[unsafe(no_mangle)]
pub extern "C" fn kaiji_normalize(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let s = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match ::kaiji::normalize_default(s) {
        Ok(result) => match CString::new(result.as_ref()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns 1 if a and b match after normalization, 0 if not, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn kaiji_matches(a: *const c_char, b: *const c_char) -> i32 {
    if a.is_null() || b.is_null() {
        return -1;
    }
    let a_str = match unsafe { CStr::from_ptr(a) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let b_str = match unsafe { CStr::from_ptr(b) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match ::kaiji::matches_default(a_str, b_str) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

/// Returns Jaro-Winkler similarity in [0.0, 1.0], or -1.0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn kaiji_similarity(a: *const c_char, b: *const c_char) -> f32 {
    if a.is_null() || b.is_null() {
        return -1.0;
    }
    let a_str = match unsafe { CStr::from_ptr(a) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1.0,
    };
    let b_str = match unsafe { CStr::from_ptr(b) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1.0,
    };
    let cfg = ::kaiji::NormalizerConfig::default();
    ::kaiji::similarity_score(a_str, b_str, &cfg).unwrap_or(-1.0)
}

/// Free a string previously returned by kaiji_normalize().
#[unsafe(no_mangle)]
pub extern "C" fn kaiji_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}
