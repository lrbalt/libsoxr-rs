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
//! [dependencies]
//! libsoxr = "0.2"
//! ```
//!
//! # Example
//!
//! ```rust
//! # use libsoxr::Soxr;
//! // upscale factor 2, one channel with all the defaults
//! let soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
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
//! // Two runs. First run will convert the source data into target.
//! // Last run with None is to inform resampler of end-of-input so it can clean up
//! soxr.process(Some(&source), &mut target).unwrap();
//! soxr.process::<f32,_>(None, &mut target[0..]).unwrap();
//!
//! // just print the values in target
//! for s in target.iter() {
//!   print!("{:?}\t", s)
//! }
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
    error_handling::{Error, ErrorType, Result},
    soxr::{Soxr, SoxrFunction},
    spec::{IOSpec, QualityFlags, QualityRecipe, QualitySpec, RuntimeSpec},
};
