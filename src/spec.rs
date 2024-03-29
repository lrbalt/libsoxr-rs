//! For specifying the runtime settings of the resampler
//! For specifying the data type of input and output
use crate::datatype::Datatype;
use libsoxr_sys as soxr;

/// Runtime parameters for resampler. Can be used to control number of threads the resampler uses. Wrapper for `soxr_runtime_spec_t`
pub struct RuntimeSpec {
    runtime_spec: soxr::soxr_runtime_spec_t,
}

impl RuntimeSpec {
    /// creates a new `RuntimeSpec` for `num_threads` threads
    /// ```
    /// use libsoxr::{Soxr, RuntimeSpec};
    ///
    /// let spec = RuntimeSpec::new(4);
    /// let soxr = Soxr::create(96000.0, 44100.0, 2, None, None, Some(&spec));
    /// assert!(soxr.is_ok());    
    /// ```
    pub fn new(num_threads: u32) -> RuntimeSpec {
        RuntimeSpec {
            runtime_spec: unsafe { soxr::soxr_runtime_spec(num_threads) },
        }
    }
    /// returns inner soxr struct
    pub(crate) fn soxr_spec(&self) -> &soxr::soxr_runtime_spec_t {
        &self.runtime_spec
    }
}

/// IOSpec can be used to set the datatype of the input buffer and output buffer. Wrapper for `soxr_io_spec_t`
#[derive(Debug, Clone)]
pub struct IOSpec {
    io_spec: soxr::soxr_io_spec_t,
    input_type: Datatype,
    output_type: Datatype,
}

impl IOSpec {
    /// creates a new `IOSpec` using `soxr_io_spec`
    /// ```
    /// use libsoxr::{Soxr, Datatype, IOSpec};
    ///
    /// let spec = IOSpec::new(Datatype::Float32I, Datatype::Float32I);
    /// let soxr = Soxr::create(96000.0, 44100.0, 2, Some(&spec), None, None);
    /// assert!(soxr.is_ok());    
    /// ```
    pub fn new(input_type: Datatype, output_type: Datatype) -> IOSpec {
        let itype = input_type.to_soxr_datatype();
        let otype = output_type.to_soxr_datatype();
        IOSpec {
            io_spec: unsafe { soxr::soxr_io_spec(itype, otype) },
            input_type,
            output_type,
        }
    }

    /// returns inner soxr struct
    pub(crate) fn soxr_spec(&self) -> &soxr::soxr_io_spec_t {
        &self.io_spec
    }

    pub fn input_type(&self) -> Datatype {
        self.input_type
    }

    pub fn output_type(&self) -> Datatype {
        self.output_type
    }
}

bitflags! {
    /// Quality flags
    pub struct QualityFlags: std::os::raw::c_ulong {
        /// <= 0.01 dB
        const ROLLOFF_SMALL = soxr::SOXR_ROLLOFF_SMALL as std::os::raw::c_ulong;
        /// <= 0.35 dB
        const ROLLOFF_MEDIUM = soxr::SOXR_ROLLOFF_MEDIUM as std::os::raw::c_ulong;
        /// For Chebyshev bandwidth.
        const ROLLOFF_NONE = soxr::SOXR_ROLLOFF_NONE as std::os::raw::c_ulong;
        /// Increase `irrational' ratio accuracy
        const HI_PREC_CLOCK = soxr::SOXR_HI_PREC_CLOCK as std::os::raw::c_ulong;
        ///  Use D.P. calcs even if precision <= 20
        const DOUBLE_PRECISION = soxr::SOXR_DOUBLE_PRECISION as std::os::raw::c_ulong;
        /// Variable-rate resampling
        const VR = soxr::SOXR_VR as std::os::raw::c_ulong;
    }
}

pub enum QualityRecipe {
    /// 'Quick' cubic interpolation
    Quick,
    /// 'Low' 16-bit with larger rolloff
    Low,
    /// 'Medium' 16-bit with medium rolloff
    Medium,
    /// 'High quality'
    High,
    /// 'Very high quality'
    VeryHigh,
}

impl QualityRecipe {
    /// convert to SOXR constant
    pub(crate) fn to_recipe(&self) -> u32 {
        match self {
            QualityRecipe::Quick => soxr::SOXR_QQ,
            QualityRecipe::Low => soxr::SOXR_LQ,
            QualityRecipe::Medium => soxr::SOXR_MQ,
            QualityRecipe::High => soxr::SOXR_HQ,
            QualityRecipe::VeryHigh => soxr::SOXR_VHQ,
        }
    }
}

/// QualitySpec controls the quality of the resampling. Wrapper for `soxr_quality_spec_t`
#[derive(Debug)]
pub struct QualitySpec {
    quality_spec: soxr::soxr_quality_spec_t,
}

impl QualitySpec {
    /// Create a new spec from supplied recipe and flags
    ///
    ///```rust
    /// use libsoxr::{QualityFlags, QualityRecipe, QualitySpec, Soxr};
    ///
    /// let spec = QualitySpec::new(&QualityRecipe::High, QualityFlags::ROLLOFF_SMALL);
    /// let soxr = Soxr::create(96000.0, 44100.0, 2, None, Some(&spec), None);
    /// assert!(soxr.is_ok());
    ///```
    pub fn new(quality: &QualityRecipe, flags: QualityFlags) -> QualitySpec {
        QualitySpec {
            quality_spec: unsafe {
                soxr::soxr_quality_spec(
                    std::os::raw::c_ulong::from(quality.to_recipe()),
                    flags.bits() as std::os::raw::c_ulong,
                )
            },
        }
    }

    /// returns inner soxr struct
    pub(crate) fn soxr_spec(&self) -> &soxr::soxr_quality_spec_t {
        &self.quality_spec
    }
}

#[test]
fn test_create_io_spec() {
    let spec = IOSpec::new(Datatype::Float32I, Datatype::Int32I);
    assert_eq!(Datatype::Float32I, spec.input_type());
    assert_eq!(Datatype::Int32I, spec.output_type());
    assert_eq!(
        Datatype::Float32I.to_soxr_datatype() as isize,
        spec.io_spec.itype as isize
    );
    assert_eq!(
        Datatype::Int32I.to_soxr_datatype() as isize,
        spec.io_spec.otype as isize
    );

    let spec = IOSpec::new(Datatype::Float32S, Datatype::Int32S);
    assert_eq!(Datatype::Float32S, spec.input_type());
    assert_eq!(Datatype::Int32S, spec.output_type());
    assert_eq!(
        Datatype::Float32S.to_soxr_datatype() as isize,
        spec.io_spec.itype as isize
    );
    assert_eq!(
        Datatype::Int32S.to_soxr_datatype() as isize,
        spec.io_spec.otype as isize
    );
}

#[test]
fn test_create_runtime_spec() {
    let spec = RuntimeSpec::new(16);
    assert_eq!(16, spec.runtime_spec.num_threads);
}

#[test]
fn test_create_quality_spec() {
    let spec = QualitySpec::new(
        &QualityRecipe::High,
        QualityFlags::ROLLOFF_SMALL | QualityFlags::ROLLOFF_MEDIUM,
    );
    let result = QualityFlags::from_bits_truncate(spec.soxr_spec().flags);
    assert!(result.contains(QualityFlags::ROLLOFF_SMALL | QualityFlags::ROLLOFF_MEDIUM));
}
