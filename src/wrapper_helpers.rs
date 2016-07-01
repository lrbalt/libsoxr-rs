//! Convenience helper functions to handle conversion between C-types and Rust-types

use std::ptr;
use std::ffi::CStr;
use libc::{c_void, c_char, free};
use error_handling::{Error, Result};

pub fn from_const<'a>(func: &'static str, s: *const c_char) -> Result<&'a str> {
    if s == ptr::null() {
        return Err(Error::invalid_str(func));
    };
    let cstr = unsafe { CStr::from_ptr(s) };
    ::std::str::from_utf8(cstr.to_bytes()).map_err(|_| Error::invalid_str(func))
}

pub fn _from_alloc(func: &'static str, s: *const c_char) -> Result<String> {
    if s == ptr::null_mut() {
        return Err(Error::invalid_str(func));
    };
    let cstr = unsafe { CStr::from_ptr(s) };
    let rust_string = try!(::std::str::from_utf8(cstr.to_bytes()).map_err(|_| {
                          unsafe {
                              free(s as *mut c_void);
                          }
                          Error::invalid_str(func)
                      }))
                          .to_string();
    unsafe {
        free(s as *mut c_void);
    }
    Ok(rust_string)
}
