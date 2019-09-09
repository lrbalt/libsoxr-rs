//! For specifying the runtime settings of the resampler
//! For specifying the data type of input and output
use crate::{api, datatype::Datatype};
use libc;

/// Wrapper for `soxr_runtime_spec_t`
pub struct RuntimeSpec {
    runtime_spec: api::soxr_runtime_spec_t,
}

impl RuntimeSpec {
    /// creates a new `RuntimeSpec` for `num_threads` threads
    pub fn new(num_threads: u32) -> RuntimeSpec {
        RuntimeSpec {
            runtime_spec: unsafe { api::soxr_runtime_spec(num_threads) },
        }
    }
    /// returns inner soxr struct
    pub fn soxr_spec(&self) -> &api::soxr_runtime_spec_t {
        &self.runtime_spec
    }
}

/// Wrapper for `soxr_io_spec_t`
pub struct IOSpec {
    io_spec: api::soxr_io_spec_t,
}

impl IOSpec {
    /// creates a new `IOSpec` using `soxr_io_spec`
    /// ```
    /// let spec = IOSpec::new(Datatype::Float32I, Datatype::Float32I);
    /// ```
    pub fn new(input_type: Datatype, output_type: Datatype) -> IOSpec {
        let itype = input_type.to_soxr_datatype();
        let otype = output_type.to_soxr_datatype();
        IOSpec {
            io_spec: unsafe { api::soxr_io_spec(itype, otype) },
        }
    }

    /// returns inner soxr struct
    pub fn soxr_spec(&self) -> &api::soxr_io_spec_t {
        &self.io_spec
    }
}

bitflags! {
    pub struct QualityFlags: libc::c_ulong {
        const ROLLOFF_SMALL = api::SOXR_ROLLOFF_SMALL as libc::c_ulong;
        const ROLLOFF_MEDIUM = api::SOXR_ROLLOFF_MEDIUM as libc::c_ulong;
        const ROLLOFF_NONE = api::SOXR_ROLLOFF_NONE as libc::c_ulong;
        const HI_PREC_CLOCK = api::SOXR_HI_PREC_CLOCK as libc::c_ulong;
        const VR = api::SOXR_VR as libc::c_ulong;
    }
}

pub enum QualityRecipe {
    Quick,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl QualityRecipe {
    /// convert to SOXR constant
    pub fn to_recipe(&self) -> u8 {
        match *self {
            QualityRecipe::Quick => api::SOXR_QQ,
            QualityRecipe::Low => api::SOXR_LQ,
            QualityRecipe::Medium => api::SOXR_MQ,
            QualityRecipe::High => api::SOXR_HQ,
            QualityRecipe::VeryHigh => api::SOXR_VHQ,
        }
    }
}

/// Wrapper for `soxr_quality_spec_t`
pub struct QualitySpec {
    quality_spec: api::soxr_quality_spec_t,
}

impl QualitySpec {
    pub fn new(quality: &QualityRecipe, flags: QualityFlags) -> QualitySpec {
        QualitySpec {
            quality_spec: unsafe {
                api::soxr_quality_spec(
                    libc::c_ulong::from(quality.to_recipe()),
                    flags.bits as libc::c_ulong,
                )
            },
        }
    }

    /// returns inner soxr struct
    pub fn soxr_spec(&self) -> &api::soxr_quality_spec_t {
        &self.quality_spec
    }
}

#[test]
fn test_create_io_spec() {
    let spec = IOSpec::new(Datatype::Float32I, Datatype::Int32I);
    assert_eq!(
        Datatype::Float32I.to_soxr_datatype() as isize,
        spec.io_spec.itype as isize
    );
    assert_eq!(
        Datatype::Int32I.to_soxr_datatype() as isize,
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
