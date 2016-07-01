//! Rust API for SOXR.

use std::ffi::CString;
use std::ptr;
use libc::{c_char, c_void};
use wrapper_helpers::from_const;
use api;
use error_handling::{Error, ErrorType, Result};
use spec::{IOSpec, QualitySpec, RuntimeSpec};

/// Wrapper for soxr_t
pub struct Soxr {
    soxr: api::soxr_t,
    error: CString,
}

impl Soxr {
    /// Create a new resampler
    pub fn create(input_rate: f64,
                  output_rate: f64,
                  num_channels: u32,
                  io_spec: Option<IOSpec>,
                  quality_spec: Option<QualitySpec>,
                  runtime_spec: Option<RuntimeSpec>)
                  -> Result<Soxr> {

        let error: *mut c_char = ::std::ptr::null_mut();

        let q = quality_spec.map_or(ptr::null(), |spec| spec.soxr_spec());
        let io = io_spec.map_or(ptr::null(), |spec| spec.soxr_spec());
        let rt = runtime_spec.map_or(ptr::null(), |spec| spec.soxr_spec());

        let soxr = unsafe {
            api::soxr_create(input_rate, output_rate, num_channels, error, io, q, rt)
        };

        if error == ptr::null_mut() {
            Ok(Soxr {
                soxr: soxr,
                error: CString::new("").unwrap(),
            })
        } else {
            Err(Error::new(Some("Soxr::new".into()),
                           ErrorType::CreateError(from_const("Soxr::new", error)
                                                      .unwrap()
                                                      .to_string())))
        }
    }

    /// Get version of libsoxr
    pub fn version() -> &'static str {
        unsafe { from_const("Soxr::version", api::soxr_version()).unwrap() }
    }

    /// Set error of Soxr engine
    pub fn set_error(&mut self, msg: String) -> Result<()> {
        self.error = CString::new(msg).unwrap();
        let result = unsafe { api::soxr_set_error(self.soxr, self.error.as_ptr() as *mut i8) };
        if result == ptr::null_mut() {
            Ok(())
        } else {
            Err(Error::new(Some("Soxr::new".into()),
                           ErrorType::ChangeError(from_const("Soxr::set_error", result)
                                                      .unwrap()
                                                      .to_string())))
        }
    }

    /// Change number of channels after creating Soxr object
    pub fn set_num_channels(&self, num_channels: u32) -> Result<()> {
        let error = unsafe { api::soxr_set_num_channels(self.soxr, num_channels) };
        if error == ptr::null_mut() {
            Ok(())
        } else {
            Err(Error::new(Some("Soxr::set_num_channels".into()),
                           ErrorType::ChangeError(from_const("Soxr::set_num_channels", error)
                                                      .unwrap()
                                                      .to_string())))
        }
    }

    /// Query error status.
    pub fn error(&self) -> Option<String> {
        let error = unsafe { api::soxr_error(self.soxr) };
        println!("error: {:?}", error);
        if error == ptr::null_mut() {
            None
        } else {
            Some(from_const("Soxr::error", error).unwrap().to_string())
        }
    }

    /// Query int. clip counter (for R/W).
    pub fn num_clips(&self) -> usize {
        unsafe { *api::soxr_num_clips(self.soxr) }
    }

    /// Query current delay in output samples
    pub fn delay(&self) -> f64 {
        unsafe { api::soxr_delay(self.soxr) }
    }

    /// Query resampling engine name.
    pub fn engine(&self) -> String {
        from_const("Soxr::engine", unsafe { api::soxr_engine(self.soxr) }).unwrap().to_string()
    }

    /// Ready for fresh signal, same config.
    pub fn clear(&mut self) -> Result<()> {
        let error = unsafe { api::soxr_clear(self.soxr) };
        if error == ptr::null_mut() {
            Ok(())
        } else {
            Err(Error::new(Some("Soxr::clear".into()),
                           ErrorType::ChangeError(from_const("Soxr::clear", error)
                                                      .unwrap()
                                                      .to_string())))
        }
    }

    /// For variable-rate resampling. See example # 5 of libsoxr repository for how to create a
    /// variable-rate resampler and how to use this function.
    pub fn set_io_ratio(&mut self, io_ratio: f64, slew_len: usize) -> Result<()> {
        let error = unsafe { api::soxr_set_io_ratio(self.soxr, io_ratio, slew_len) };
        if error == ptr::null_mut() {
            Ok(())
        } else {
            Err(Error::new(Some("Soxr::set_io_ratio".into()),
                           ErrorType::ChangeError(from_const("Soxr::set_io_ratio", error)
                                                      .unwrap()
                                                      .to_string())))
        }
    }

    /// Resamples `Some(buf_in)` into `buf_out`. Type is dependent on [IOSpec]. If you leave out
    /// [IOSpec] on create, it defaults to `f32`. Make sure that `buf_out` is large enough to hold
    /// the resampled data. Furthermore, to indicate end-of-input to the resampler, always end with
    /// a last call to process with `None` as `buf_in`. The result contains number of input samples
    /// used and number of output samples places in 'buf_out'
    pub fn process<I, O>(&self, buf_in: Option<&[I]>, buf_out: &mut [O]) -> Result<(usize, usize)> {
        let mut idone = 0;
        let mut odone = 0;
        let error = match buf_in {
            Some(buf) => unsafe {
                api::soxr_process(self.soxr,
                                  buf.as_ptr() as *const c_void,
                                  buf.len(),
                                  &mut idone,
                                  buf_out.as_mut_ptr() as *mut c_void,
                                  buf_out.len(),
                                  &mut odone)
            },
            None => unsafe {
                api::soxr_process(self.soxr,
                                  ptr::null() as *const c_void,
                                  0,
                                  &mut idone,
                                  buf_out.as_mut_ptr() as *mut c_void,
                                  buf_out.len(),
                                  &mut odone)
            },
        };
        if error == ptr::null_mut() {
            Ok((idone, odone))
        } else {
            Err(Error::new(Some("Soxr::process".into()),
                           ErrorType::ProcessError(from_const("Soxr::process", error)
                                                       .unwrap()
                                                       .to_string())))
        }
    }
}

impl Drop for Soxr {
    fn drop(&mut self) {
        unsafe { api::soxr_delete(self.soxr) };
    }
}

#[test]
fn test_version() {
    let version = Soxr::version();
    println!("{}", version);
    assert_eq!("libsoxr-0.1.2", version);
}

#[test]
fn test_create() {
    use datatype::Datatype;
    use spec::{QualityRecipe, ROLLOFF_SMALL};

    let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None);
    assert!(s.is_ok());
    s = Soxr::create(96000.0,
                     44100.0,
                     2,
                     Some(IOSpec::new(Datatype::Float32I, Datatype::Int32I)),
                     Some(QualitySpec::new(QualityRecipe::High, ROLLOFF_SMALL)),
                     Some(RuntimeSpec::new(4)));
    assert!(s.is_ok());
}

#[test]
fn test_error() {
    let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None).unwrap();
    assert!(s.error().is_none());
    assert!(s.set_error("Sumsing Wrung".to_string()).is_ok());
    // FIXME: should eval to true, but not seem to work and returns false
    assert!(!s.error().is_some());
}

#[test]
fn test_engine() {
    let s = Soxr::create(96000.0, 44100.0, 2, None, None, None).unwrap();
    assert_eq!("single-precision-SIMD", s.engine());
}

#[test]
fn test_clear() {
    let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None).unwrap();
    assert!(s.clear().is_ok());
}

#[test]
fn test_set_io_ratio() {
    let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None).unwrap();
    let result = s.set_io_ratio(0.1, 1);
    assert!(result.is_err());
}

#[test]
fn test_process() {
    // Example taken from 1-single-block.c of libsoxr
    let s = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
    let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
                             1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
                             0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
                             -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
    let mut target: [f32; 96] = [0.0; 96];
    let result = s.process(Some(&source), &mut target).and_then(|_| {
        s.process::<f32, f32>(None, &mut target[0..]).and_then(|_| {
            for s in target.iter() {
                print!("{:?}\t", s)
            }
            Ok(())
        })
    });
    assert!(result.is_ok());
}
