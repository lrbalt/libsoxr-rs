//! Encapsulated data types for samples
use libsoxr_sys as api;

/// Wrapper for `soxr_datatype_t`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Datatype {
    Float32I = api::SOXR_FLOAT32_I as isize,
    Float64I = api::SOXR_FLOAT64_I as isize,
    Int32I = api::SOXR_INT32_I as isize,
    Int16I = api::SOXR_INT16_I as isize,
    Float32S = api::SOXR_FLOAT32_S as isize,
    Float64S = api::SOXR_FLOAT64_S as isize,
    Int32S = api::SOXR_INT32_S as isize,
    Int16S = api::SOXR_INT16_S as isize,
}

impl Datatype {
    /// helper function to convert from `Datatype` to `soxr_datatype_t`
    pub fn to_soxr_datatype(self) -> api::soxr_datatype_t {
        match self {
            Datatype::Float32I => api::SOXR_FLOAT32_I,
            Datatype::Float64I => api::SOXR_FLOAT64_I,
            Datatype::Int32I => api::SOXR_INT32_I,
            Datatype::Int16I => api::SOXR_INT16_I,
            Datatype::Float32S => api::SOXR_FLOAT32_S,
            Datatype::Float64S => api::SOXR_FLOAT64_S,
            Datatype::Int32S => api::SOXR_INT32_S,
            Datatype::Int16S => api::SOXR_INT16_S,
        }
    }
}
