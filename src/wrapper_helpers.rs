//! Convenience helper functions to handle conversion between C-types and Rust-types

use crate::error_handling::{Error, Result};
use libc::free;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

pub fn from_const<'a>(func: &'static str, s: *const c_char) -> Result<&'a str> {
    if s.is_null() {
        return Err(Error::invalid_str(func));
    };
    let cstr = unsafe { CStr::from_ptr(s) };
    ::std::str::from_utf8(cstr.to_bytes()).map_err(|_| Error::invalid_str(func))
}

pub fn _from_alloc(func: &'static str, s: *const c_char) -> Result<String> {
    if s.is_null() {
        return Err(Error::invalid_str(func));
    };
    let cstr = unsafe { CStr::from_ptr(s) };
    let rust_string = std::str::from_utf8(cstr.to_bytes())
        .map_err(|_| {
            unsafe {
                free(s as *mut c_void);
            }
            Error::invalid_str(func)
        })?
        .to_string();
    unsafe {
        free(s as *mut c_void);
    }
    Ok(rust_string)
}
