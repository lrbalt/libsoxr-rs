//! # libsoxr-rs
//!
//! This library is a thin wrapper for [libsoxr](https://sourceforge.net/projects/soxr/) which is
//! a "High quality, one-dimensional sample-rate conversion library".
//!
//! For direct access to the libsoxr functions, you can use the [libsoxr-sys](https://github.com/lrbalt/libsoxr-sys) crate.
//!
//! This wrapper library is licensed the same as libsoxr itself: LGPLv2.
//!
//! The documentation can be found [here](https://lrbalt.github.io/libsoxr-rs/libsoxr/).
//!
//! # Install
//!
//! add the following to your Cargo.toml:
//! ```toml
//! [dependencies.libsoxr]
//! git = "https://github.com/lrbalt/libsoxr-rs"
//! ```
//!
//! and add the crate:
//!
//! ```ignore
//! extern crate libsoxr;
//! use libsoxr::Soxr;
//! ```
//!
//! # Example
//!
//! ```rust
//! # use libsoxr::Soxr;
//! // upscale factor 2, one channel with all the defaults
//! let s = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
//!
//! // source data, taken from 1-single-block.c of libsoxr examples.
//! let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
//!                          1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
//!                          0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
//!                          -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
//!
//! // create room for 2*48 = 96 samples
//! let mut target: [f32; 96] = [0.0; 96];
//!
//! // two runs. last run with None to inform resampler of end-of-input
//! let result = s.process(Some(&source), &mut target).and_then(|_| {
//!     s.process::<f32, f32>(None, &mut target[0..]).and_then(|_| {
//!         for s in target.iter() {
//!             print!("{:?}\t", s)
//!         }
//!         Ok(())
//!     })
//! });
//! ```
#[macro_use]
extern crate bitflags;

pub mod datatype;
pub mod soxr;
pub mod spec;

mod error_handling;
mod wrapper_helpers;

pub use crate::{
    datatype::Datatype,
    soxr::Soxr,
    spec::{IOSpec, QualitySpec, RuntimeSpec},
    error_handling::{ Error, Result }
};
