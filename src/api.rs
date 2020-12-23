//! This is a thin wrapper around the libsoxr C API.
//!
//! The documentation for this api namespace was copied from
//! [`soxr.h`](https://sourceforge.net/p/soxr/code/ci/master/tree/src/soxr.h) of libsoxr
//!
//! # API conventions
//!
//! Buffer lengths (and occupancies) are expressed as the number of contained
//! samples per channel.
//!
//! Parameter names for buffer lengths have the suffix `len'.
//!
//! A single-character `i' or 'o' is often used in names to give context as
//! input or output (e.g. ilen, olen).

#![allow(non_camel_case_types)]

use libc::{c_char, c_double, c_uint, c_ulong, c_void, size_t};
// {c_int, c_uint, c_long, c_longlong, size_t, ssize_t, c_void, c_char,
// c_uchar, c_ulong, c_double, FILE, c_ushort, c_short, pid_t, timeval, timespec, pollfd};

pub enum soxr {}
pub type soxr_t = *mut soxr;
pub type soxr_error_t = *const c_char;

/// If needed by the input function. As given to `soxr_set_input_fn`
pub type soxr_fn_state_t = *const c_void;
/// Either a `soxr_cbuf_t` or `soxr_cbufs_t`, depending on `itype` in `soxr_io_spec_t`.
pub type soxr_in_t = *const c_void;
/// Either a `soxr_buf_t` or `soxr_bufs_t`,depending on `otype` in `soxr_io_spec_t`.
pub type soxr_out_t = *mut c_void;

/// Function to supply data to be resampled.
///
/// * `input_fn_state` - As given to `soxr_set_input_fn` (below).
/// * `data` - Returned data; see below. N.B. ptr to ptr(s)
/// * `requested_len` - Samples per channel, >= returned `data_len`.
///
/// returns `data_len`
///
/// ```ignore
///  data_len  *data     Indicates    Meaning
///  ------- -------   ------------  -------------------------
///    !=0     !=0       Success     *data contains data to be
///                                  input to the resampler.
///     0    !=0 (or   End-of-input  No data is available nor
///          not set)                shall be available.
///     0       0        Failure     An error occurred whilst trying to
///                                  source data to be input to the resampler.  */
/// ```
/// and be registered with a previously created stream resampler using `soxr_set_input_fn`
pub type soxr_input_fn_t = extern "C" fn(*const c_void, *mut soxr_in_t, usize) -> usize;

/// Datatypes supported for I/O to/from the resampler:
///
/// * Internal; do not use (left out):
///   * `SOXR_FLOAT32`, `SOXR_FLOAT64`, `SOXR_INT32`, `SOXR_INT16`, `SOXR_SPLIT` = 4,
/// * Use for interleaved channels:
///   * `SOXR_FLOAT32_I` = `SOXR_FLOAT32`, `SOXR_FLOAT64_I`, `SOXR_INT32_I`, `SOXR_INT16_I`,
/// * Use for split channels:
///   * `SOXR_FLOAT32_S` = `SOXR_SPLIT`  , `SOXR_FLOAT64_S`, `SOXR_INT32_S`, `SOXR_INT16_S`
#[repr(C)]
pub enum soxr_datatype_t {
    SOXR_FLOAT32_I = 0 as isize,
    SOXR_FLOAT64_I = 1 as isize,
    SOXR_INT32_I = 2 as isize,
    SOXR_INT16_I = 3 as isize,
    SOXR_FLOAT32_S = 4 as isize,
    SOXR_FLOAT64_S = 5 as isize,
    SOXR_INT32_S = 6 as isize,
    SOXR_INT16_S = 7 as isize,
}

#[repr(C)]
pub struct soxr_io_spec_t {
    /// Input datatype. Typically SOXR_FLOAT32_I
    pub itype: soxr_datatype_t,
    /// Output datatype. typically SOXR_FLOAT32_I
    pub otype: soxr_datatype_t,
    /// Linear gain to apply during resampling. typically 1
    pub scale: c_double,
    /// Reserved for internal use. typically 0
    pub e: *mut c_void,
    /// Per the following #defines. typically 0
    pub flags: c_ulong,
}

/// Applicable only if otype is INT16.
pub const SOXR_TPDF: u8 = 0;
/// Disable the above.
pub const SOXR_NO_DITHER: u8 = 8;

#[repr(C)]
#[derive(Debug)]
pub struct soxr_quality_spec {
    /// Conversion precision (in bits). Typically 20
    pub precision: c_double,
    /// 0=minimum, ... 50=linear, ... 100=maximum. Typically 50
    pub phase_response: c_double,
    /// 0dB pt. bandwidth to preserve; nyquist=1. Typically 0.913
    pub passband_end: c_double,
    /// Aliasing/imaging control; > passband_end. Typically 1
    pub stopband_begin: c_double,
    /// Reserved for internal use. Typically 0
    pub e: *mut c_void,
    /// Per the following #defines. Typically  0
    pub flags: c_ulong,
}
pub type soxr_quality_spec_t = soxr_quality_spec;

/// <= 0.01 dB
pub const SOXR_ROLLOFF_SMALL: u8 = 0;
/// <= 0.35 dB
pub const SOXR_ROLLOFF_MEDIUM: u8 = 1;
/// For Chebyshev bandwidth.
pub const SOXR_ROLLOFF_NONE: u8 = 2;
/// Increase `irrational' ratio accuracy.
pub const SOXR_HI_PREC_CLOCK: u8 = 8;
/// Use D.P. calcs even if precision <= 20.
pub const SOXR_DOUBLE_PRECISION: u8 = 16;
/// Variable-rate resampling.
pub const SOXR_VR: u8 = 32;

#[repr(C)]
pub struct soxr_runtime_spec_t {
    /// For DFT efficiency. [8,15]          Typically 11
    pub log2_min_dft_size: c_uint,
    /// For DFT efficiency. [8,20]          Typically 17
    pub log2_large_dft_size: c_uint,
    /// For SOXR_COEF_INTERP_AUTO (below).  Typically 400
    pub coef_size_kbytes: c_uint,
    /// If built so. 0 means `automatic'.    Typically 1
    pub num_threads: c_uint,
    /// Reserved for internal use.           Typically 0
    pub e: *mut c_void,
    /// Per the following #defines.          Typically 0
    pub flags: c_ulong,
}

/// Auto select coef. interpolation.
pub const SOXR_COEF_INTERP_AUTO: u8 = 0;
/// Man. select: less CPU, more memory.
pub const SOXR_COEF_INTERP_LOW: u8 = 2;
/// Man. select: more CPU, less memory.
pub const SOXR_COEF_INTERP_HIGH: u8 = 3;

// Quality Spec
/// 'Quick' cubic interpolation.
pub const SOXR_QQ: u8 = 0;
/// 'Low' 16-bit with larger rolloff.
pub const SOXR_LQ: u8 = 1;
/// 'Medium' 16-bit with medium rolloff.
pub const SOXR_MQ: u8 = 2;
/// 'High quality'.
pub const SOXR_HQ: u8 = SOXR_20_BITQ;
/// 'Very high quality'.
pub const SOXR_VHQ: u8 = SOXR_28_BITQ;
pub const SOXR_16_BITQ: u8 = 3;
pub const SOXR_20_BITQ: u8 = 4;
pub const SOXR_24_BITQ: u8 = 5;
pub const SOXR_28_BITQ: u8 = 6;
pub const SOXR_32_BITQ: u8 = 7;
pub const SOXR_LINEAR_PHASE: u8 = 0x00;
pub const SOXR_INTERMEDIATE_PHASE: u8 = 0x10;
pub const SOXR_MINIMUM_PHASE: u8 = 0x30;
pub const SOXR_STEEP_FILTER: u8 = 0x40;

extern "C" {
    /// Create a stream resampler
    ///
    /// * `input_rate` - Input sample-rate.
    /// * `output_rate` - Output sample-rate.
    /// * `num_channels` - Number of channels to be used.
    ///
    /// All following arguments are optional (may be set to NULL)
    ///
    /// * `error` - To report any error during creation.
    /// * `io_spec` - To specify non-default I/O formats.
    /// * `quality_spec` - To specify non-default resampling quality.
    /// * `runtime_spec` - To specify non-default runtime resources.
    ///
    /// Default `io_spec`      is per `soxr_io_spec(SOXR_FLOAT32_I, SOXR_FLOAT32_I)`.
    /// Default `quality_spec` is per `soxr_quality_spec(SOXR_HQ, 0)`.
    /// Default `runtime_spec` is per `soxr_runtime_spec(1)`.
    pub fn soxr_create(
        input_rate: c_double,
        output_rate: c_double,
        num_channels: c_uint,
        error: soxr_error_t,
        io_spec: *const soxr_io_spec_t,
        quality_spec: *const soxr_quality_spec_t,
        runtime_spec: *const soxr_runtime_spec_t,
    ) -> soxr_t;

    /// If not using an app-supplied input function, after creating a stream resampler, repeatedly
    /// call `soxr_process`
    ///
    /// * resampler - As returned by soxr_create.
    /// * Input (to be resampled):
    ///   * buf_in - Input buffer(s); may be NULL (see below).
    ///   * ilen - Input buf. length (samples per channel).
    ///   * idone - To return actual # samples used (<= ilen).
    /// * Output (resampled):
    ///   * buf_out - Output buffer(s).
    ///   * olen - Output buf. length (samples per channel).
    ///   * odone - To return actual # samples out (<= olen).
    ///
    /// Note that no special meaning is associated with ilen or olen equal to zero.
    /// End-of-input (i.e. no data is available nor shall be available) may be indicated by
    /// setting `in' to NULL.
    pub fn soxr_process(
        resampler: soxr_t,
        buf_in: soxr_in_t,
        ilen: size_t,
        idone: *mut size_t,
        buf_out: soxr_out_t,
        olen: size_t,
        odone: *mut size_t,
    ) -> soxr_error_t;

    /// Set (or reset) an input function.
    ///
    /// * resampler - As returned by soxr_create.
    /// * func - Function to supply data to be resampled.
    /// * input_fn_state - If needed by the input function.
    /// * max_ilen - Maximum value for input fn. requested_len.
    pub fn soxr_set_input_fn(
        resampler: soxr_t,
        func: soxr_input_fn_t,
        input_fn_state: soxr_fn_state_t,
        max_ilen: usize,
    ) -> soxr_error_t;

    /// Resample and output a block of data. If using an app-supplied input function, it must
    /// look and behave like `soxr_input_fn_t` and be registered with a previously created stream
    /// resampler using `soxr_set_input_fn` then repeatedly call `soxr_output`.
    ///
    /// * resampler - As returned by soxr_create.
    /// * data - App-supplied buffer(s) for resampled data.
    /// * olen - Amount of data to output; >= return value
    /// returns number of samples in buffer
    pub fn soxr_output(resampler: soxr_t, data: soxr_out_t, olen: usize) -> usize;

    /// Query library version: "libsoxr-x.y.z"
    pub fn soxr_version() -> *const c_char;

    /// Query error status.
    pub fn soxr_error(soxr: soxr_t) -> soxr_error_t;

    /// Query int. clip counter (for R/W).
    pub fn soxr_num_clips(soxr: soxr_t) -> *const size_t;

    /// Query current delay in output samples
    pub fn soxr_delay(soxr: soxr_t) -> c_double;

    /// Query resampling engine name.
    pub fn soxr_engine(soxr: soxr_t) -> *const c_char;

    /// Ready for fresh signal, same config.
    pub fn soxr_clear(soxr: soxr_t) -> soxr_error_t;

    /// Free resources.
    pub fn soxr_delete(soxr: soxr_t);

    /// For variable-rate resampling. See example # 5 for how to create a variable-rate resampler
    /// and how to use this function.
    pub fn soxr_set_io_ratio(soxr: soxr_t, io_ratio: c_double, slew_len: size_t) -> soxr_error_t;

    /// These functions allow setting of the most commonly-used structure parameters, with other
    /// parameters being given default values.  The default values may then be overridden,
    /// directly in the structure, if needed.
    ///
    /// * `recipe` - Per the #defines immediately below.
    /// * `flags` - As `soxr_quality_spec_t.flags`.
    pub fn soxr_quality_spec(recipe: c_ulong, flags: c_ulong) -> soxr_quality_spec_t;

    /// These functions allow setting of the most commonly-used structure parameters, with other
    /// parameters being given default values.  The default values may then be overridden,
    /// directly in the structure, if needed.
    pub fn soxr_io_spec(itype: soxr_datatype_t, otype: soxr_datatype_t) -> soxr_io_spec_t;

    /// These functions allow setting of the most commonly-used structure parameters, with other
    /// parameters being given default values.  The default values may then be overridden,
    /// directly in the structure, if needed.
    pub fn soxr_runtime_spec(num_threads: c_uint) -> soxr_runtime_spec_t;

    /// Set error of already created `soxr_t`
    pub fn soxr_set_error(soxr: soxr_t, error: soxr_error_t) -> soxr_error_t;

    /// Set number of channels of already created `soxr_t`
    pub fn soxr_set_num_channels(soxr: soxr_t, num_channels: c_uint) -> soxr_error_t;
}
