//! Encapsulated data types for samples
use libsoxr_sys as soxr;

/// Datatypes supported for I/O to/from the resampler. 
/// Use the I data types for Interleaved channels and use the S data types for split channels. Wrapper for `soxr_datatype_t`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Datatype {
    Float32I = soxr::SOXR_FLOAT32_I as isize,
    Float64I = soxr::SOXR_FLOAT64_I as isize,
    Int32I = soxr::SOXR_INT32_I as isize,
    Int16I = soxr::SOXR_INT16_I as isize,
    Float32S = soxr::SOXR_FLOAT32_S as isize,
    Float64S = soxr::SOXR_FLOAT64_S as isize,
    Int32S = soxr::SOXR_INT32_S as isize,
    Int16S = soxr::SOXR_INT16_S as isize,
}

impl Datatype {
    /// helper function to convert from `Datatype` to `soxr_datatype_t`
    pub fn to_soxr_datatype(self) -> soxr::soxr_datatype_t {
        match self {
            Datatype::Float32I => soxr::SOXR_FLOAT32_I,
            Datatype::Float64I => soxr::SOXR_FLOAT64_I,
            Datatype::Int32I => soxr::SOXR_INT32_I,
            Datatype::Int16I => soxr::SOXR_INT16_I,
            Datatype::Float32S => soxr::SOXR_FLOAT32_S,
            Datatype::Float64S => soxr::SOXR_FLOAT64_S,
            Datatype::Int32S => soxr::SOXR_INT32_S,
            Datatype::Int16S => soxr::SOXR_INT16_S,
        }
    }
}
