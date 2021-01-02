//! Convenience functions for error handling
use std::borrow::Cow;
use std::fmt;

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

#[derive(Debug)]
pub struct Error(pub(crate) Option<Cow<'static, str>>, pub(crate) ErrorType);

impl Error {
    pub fn new(func: Option<Cow<'static, str>>, t: ErrorType) -> Error {
        Error(func, t)
    }
    pub fn invalid_str(func: &'static str) -> Error {
        Error(Some(func.into()), ErrorType::InvalidString)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "SOXR error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(ref s) => write!(f, "SOXR error: '{}' from function '{}'", s, self.1),
            None => write!(f, "SOXR error: '{}'", self.1),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
