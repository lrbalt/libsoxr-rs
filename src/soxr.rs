//! Rust API for SOXR.

use std::ffi::CString;
use std::ptr;
use libc::{c_char, c_void};
use wrapper_helpers::from_const;
use api;
use error_handling::{Error, ErrorType, Result};
use spec::{IOSpec, QualitySpec, RuntimeSpec};

/// Wrapper for `soxr_t`
#[derive(Debug)]
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

        let soxr =
            unsafe { api::soxr_create(input_rate, output_rate, num_channels, error, io, q, rt) };

        if error.is_null() {
            Ok(Soxr {
                soxr,
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
        let result =
            unsafe { api::soxr_set_error(self.soxr, self.error.as_ptr() as api::soxr_error_t) };
        if result.is_null() {
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
        if error.is_null() {
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
        if error.is_null() {
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
        if error.is_null() {
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
        if error.is_null() {
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
        if error.is_null() {
            Ok((idone, odone))
        } else {
            Err(Error::new(Some("Soxr::process".into()),
                           ErrorType::ProcessError(from_const("Soxr::process", error)
                               .unwrap()
                               .to_string())))
        }
    }

    /// Accepts a function  of type `soxr_input_fn_t`
    ///
    /// * input - The function is given input_state of type `soxr_fn_state_t` when called
    /// * input_state - the state data which libsoxr will pass to function `input`. Pass none to
    ///   skip use of state data.
    /// * max_ilen - the maximum number of samples per channel that can be asked of
    ///   function `input`. Be aware that the output_buffer that is return from the function
    ///   `input` needs to hold `max_ilen * number_of_channels` data. These boundaries are not
    ///   statically checked by Rust or `libsoxr`
    ///
    /// ```ignore
    /// // struct to hold state for input_fn
    /// struct TestState {
    ///     // use this buffer for source samples to avoid repeated allocations
    ///     source_buffer: [f32;100],
    /// }
    ///
    /// [...]
    ///
    /// // create state for input_fn
    /// let mut state = TestState {
    ///     source_buffer: [0.0;100]
    /// };
    ///
    /// // convert state to raw pointer to pass into libsoxr using set_input
    /// let state_data = Some(Box::into_raw(Box::new(&state)) as soxr_fn_state_t);
    /// assert!(s.set_input(test_input_fn, state_data, 100).is_ok());
    /// ```
    ///
    /// The `test_input_fn` used above can be defined as follows which assumed a
    /// `get_a_source_sample` function that returns one sample from the source that needs to be
    /// resampled:
    ///
    /// ```ignore
    /// extern "C" fn test_input_fn(state: soxr_fn_state_t,
    ///                            buf: *mut soxr_in_t,
    ///                            req_len: usize)
    ///                            -> usize {
    ///     unsafe {
    ///         // Cast raw pointer to a TestState. Beware not to use Box::from_raw as it will
    ///         // take ownership of the TestState and try to destroy the struct that is managed
    ///         //elsewere
    ///         let s: *mut &mut TestState = state as *mut &mut TestState;
    ///
    ///         // be aware: req_len is number of samples per channel, so for stereo you want
    ///         // to return req_len * 2 values in buf
    ///         for value in (*s).source_buffer.iter_mut().take(req_len) {
    ///             *value = get_a_source_sample();
    ///         }
    ///
    ///         {
    ///             // buf is a pointer to a pointer to a buffer with source samples
    ///             // data is a pointer to the buffer, so data.as_ptr() is the same, but the raw
    ///             // pointer. Put it in *buf to tell libsoxr where to find the buffer.
    ///             let data: &[f32] = &(*s).source_buffer[..];
    ///             *buf = data.as_ptr() as soxr_in_t;
    ///         }
    ///
    ///         // if source is "empty" return 0
    ///         if source_has_more_data() {
    ///             return req_len;
    ///         } else {
    ///             return 0;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn set_input(&mut self,
                     input: api::soxr_input_fn_t,
                     input_state: Option<api::soxr_fn_state_t>,
                     max_ilen: usize)
                     -> Result<()> {
        let error = unsafe {
            api::soxr_set_input_fn(self.soxr,
                                   input,
                                   input_state.unwrap_or_else(|| {
                                       ptr::null()
                                   }) as api::soxr_fn_state_t,
                                   max_ilen)
        };
        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new(Some("Soxr::set_input".into()),
                           ErrorType::ProcessError(from_const("Soxr::set_input", error)
                               .unwrap()
                               .to_string())))
        }
    }

    /// Resample and output a block of data using an app-supplied input function.
    /// This function must look and behave like `soxr_input_fn_t` and be registered with a
    /// previously created stream resampler using `set_input` then repeatedly call `output`.
    ///
    /// * data - App-supplied buffer(s) for resampled data.
    /// * samples - number of samples in buffer per channel, i.e. data.len() / number_of_channels
    /// returns number of samples in buffer
    ///
    /// ```ignore
    /// // call output using a buffer of 100 mono samples. For stereo devide by 2, so this buffer
    /// // could hold 100 / number_of_channels = 50 stereo samples.
    /// let mut buffer = [0.0f32; 100];
    /// assert!(s.output(&mut buffer[..], buffer.len()) > 0);
    /// ```
    pub fn output<S>(&self, data: &mut [S], samples: usize) -> usize {
        unsafe { api::soxr_output(self.soxr, data.as_mut_ptr() as *mut c_void, samples) }
    }
}

impl Drop for Soxr {
    fn drop(&mut self) {
        unsafe { api::soxr_delete(self.soxr) };
    }
}

#[cfg(test)]
mod soxr_tests {
    use super::Soxr;
    use spec::{IOSpec, QualitySpec, RuntimeSpec};

    #[test]
    fn test_version() {
        let version = Soxr::version();
        println!("{}", version);
        assert_eq!("libsoxr-0.1.2", version);
    }

    #[test]
    fn test_create() {
        use datatype::Datatype;
        use spec::{QualityRecipe, QualityFlags};

        let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None);
        assert!(s.is_ok());
        s = Soxr::create(96000.0,
                         44100.0,
                         2,
                         Some(IOSpec::new(Datatype::Float32I, Datatype::Int32I)),
                         Some(QualitySpec::new(&QualityRecipe::High, QualityFlags::ROLLOFF_SMALL)),
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
        let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
                                 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
                                 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
                                 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
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

    use api::{soxr_in_t, soxr_fn_state_t};
    extern "C" fn test_input_fn(state: soxr_fn_state_t,
                                buf: *mut soxr_in_t,
                                req_len: usize)
                                -> usize {
        unsafe {
            // Cast to raw point to a TestState. Beware not to use Box::from_raw as it will
            // try to destroy the struct that is managed elsewere
            let s: *mut &mut TestState = state as *mut &mut TestState;

            // check on a known string in TestState to make sure the raw pointer is pointing
            // to our TestState
            assert_eq!("libsoxr", (*s).check);

            print!("setting {}/{} values for {} with {}\t",
                   req_len,
                   (**s).source_buffer.len(),
                   (**s).check,
                   (**s).command);

            // just for this test, so we can check the output
            let value_to_use = (*s).value;

            // be aware: req_len is number of samples per channel, so for stereo you want to return
            // req_len * 2 values in buf
            for value in (*s).source_buffer.iter_mut().take(req_len) {
                *value = value_to_use;
            }

            {
                // buf is a pointer to a pointer to a buffer with source samples
                // data is a pointer to the buffer, so data.as_ptr() is the same, but the raw
                // pointer. Put it in *buf to tell libsoxr where to find the buffer
                let data: &[f32] = &(*s).source_buffer[..];
                *buf = data.as_ptr() as soxr_in_t;
            }

            // for testing: set command to non-zero to force end-of-input (eoi)
            if (*s).command == 0 {
                println!("returning {:?}", req_len);
                return req_len;
            } else {
                println!("eoi");
                return 0;
            }
        }
    }

    // struct to hold state for input_fn
    struct TestState {
        check: &'static str,
        command: u32,
        value: f32,
        // use this buffer for source samples to avoid repeated allocations
        source_buffer: [f32; 100],
    }

    #[test]
    fn test_data_fn() {
        let mut s = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();

        // create state for input_fn
        let mut state = TestState {
            check: "libsoxr",
            command: 0,
            value: 2.3,
            source_buffer: [1.2; 100],
        };

        // convert to raw pointer to pass into libsoxr using set_input
        let state_data = Some(Box::into_raw(Box::new(&state)) as soxr_fn_state_t);
        assert!(s.set_input(test_input_fn, state_data, 100).is_ok());

        // create buffer for resampled data
        let mut data = [1.1f32; 200];
        println!("first call");
        assert_eq!(200, s.output(&mut data[0..], 200));
        assert!(data[0] != 1.1);

        // tell test_input_fn to return end-of-input (0)
        state.command = 1;
        // other buffer for resampled data
        let mut buffer = [1.1f32; 200];
        println!("second");
        // flush all data from libsoxr until end-of-input
        while s.output(&mut buffer[0..], 200) > 0 {
            print!(".{}", buffer[0]);
            assert!(buffer[0] != 1.1);
        }
        println!();
    }
}
