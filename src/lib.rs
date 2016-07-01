//! # libsoxr-rs
//!
//! This library is a thin wrapper for [libsoxr](https://sourceforge.net/projects/soxr/) which is
//! a "High quality, one-dimensional sample-rate conversion library".
//!
//! Use the `soxr` namespace to use soxr in your Rust program. For direct access to the libsoxr
//! functions, you can use the `api` namespace.
extern crate libc;
#[macro_use]
extern crate bitflags;

pub mod api;
pub mod soxr;
pub mod spec;
pub mod datatype;

mod error_handling;
mod wrapper_helpers;

pub use soxr::Soxr;
pub use datatype::Datatype;
pub use spec::{IOSpec, QualitySpec, RuntimeSpec};
