//! Convenience functions for error handling
use std::borrow::Cow;
use std::fmt;

/// A Soxr error
#[derive(Debug)]
pub enum ErrorType {
    InvalidString,
    CreateError(String),
    ChangeError(String),
    ProcessError(String),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorType::InvalidString => write!(f, "Invalid string"),
            ErrorType::CreateError(ref s) => write!(f, "Could not create soxr struct: {}", s),
            ErrorType::ChangeError(ref s) => write!(f, "Could not change soxr struct: {}", s),
            ErrorType::ProcessError(ref s) => write!(f, "Could not process data: {}", s),
        }
    }
}

/// A Soxr error returned from a particular function
#[derive(Debug)]
pub struct Error {
    pub function: Option<Cow<'static, str>>,
    pub error_type: ErrorType
}

impl Error {
    pub fn new(func: Option<Cow<'static, str>>, t: ErrorType) -> Error {
        Error { function: func, error_type: t }
    }

    pub fn invalid_str(func: &'static str) -> Error {
        Error::new(Some(func.into()), ErrorType::InvalidString)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "SOXR error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.function {
            Some(ref s) => write!(f, "SOXR error: '{}' from function '{}'", s, self.error_type),
            None => write!(f, "SOXR error: '{}'", self.error_type),
        }
    }
}

/// Convenience type alias for `std::result::Result<T, crate::Error>`
pub type Result<T> = ::std::result::Result<T, Error>;
