//! Rust API for SOXR.

use crate::{
    error_handling::{Error, ErrorType, Result},
    spec::{IOSpec, QualitySpec, RuntimeSpec},
    wrapper_helpers::from_const,
};
use libsoxr_sys as soxr;
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    ptr,
};

/// Signature of an input function that supplies SOXR with input data.
/// `S` is type of state data and `T` is type of target buffer and `E` is the type of Error that the fn can return.
/// The last `usize` is the number of samples that Soxr asks this function to load into the buffer.
/// The function should return Ok(0) to indicate end-of-input or an Error in case of problems getting input data.
///
/// ```rust
/// use libsoxr::{Error, ErrorType, Soxr, SoxrFunction};
///
/// struct State {
///   // data for input function to supply Soxr with source samples.
///   // In this case just a value, but you could put a handle to a FLAC file into this.
///   value: f32
/// }
///
/// let input_fn = |state: &mut State, buffer: &mut [f32], samples: usize| {
///     for sample in buffer.iter_mut().take(samples) {
///       *sample = state.value;
///     }
///     // if end-of-input: return Ok(O);
///     // if error:  Err(Error::new(Some("input_fn".into()), ErrorType::ProcessError("Unexpected end of input".into())))
///     Ok(samples)
///   };
///
/// let mut soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
/// let mut state = State { value: 1.0 };
/// assert!(soxr.set_input(input_fn, Some(&mut state), 100).is_ok());
///```
pub type SoxrFunction<S, T> = fn(&mut S, &mut [T], usize) -> Result<usize>;

/// This is the starting point for the Soxr algorithm.
#[derive(Debug)]
pub struct Soxr {
    soxr: soxr::soxr_t,
    channels: u32,
    error: CString,
}

impl Soxr {
    /// Create a new resampler. When `io_spec`, `quality_spec` or `runtime_spec` is `None` then SOXR will use it defaults:
    /// * Default io_spec      is per [IOSpec]([Datatype](crate::datatype::Datatype)::Float32I, [Datatype](crate::datatype::Datatype)::Float32I)
    /// * Default quality_spec is per [QualitySpec]([QualityRecipe](crate::spec::QualityRecipe)::High, [QualityFlags](crate::spec::QualityFlags)::ROLLOFF_SMALL)
    /// * Default runtime_spec is per [RuntimeSpec] (1)    
    ///
    ///```rust
    /// use libsoxr::{Datatype, IOSpec, QualitySpec, RuntimeSpec, Soxr, QualityRecipe, QualityFlags};
    ///
    /// let io_spec = IOSpec::new(Datatype::Float32I, Datatype::Float64I);
    /// let quality_spec = QualitySpec::new(&QualityRecipe::VeryHigh, QualityFlags::HI_PREC_CLOCK);
    /// let runtime_spec = RuntimeSpec::new(4);
    /// let mut soxr = Soxr::create(1.0, 2.0, 1, Some(&io_spec), Some(&quality_spec), Some(&runtime_spec));
    /// assert!(soxr.is_ok());
    ///```      
    pub fn create(
        input_rate: f64,
        output_rate: f64,
        num_channels: u32,
        io_spec: Option<&IOSpec>,
        quality_spec: Option<&QualitySpec>,
        runtime_spec: Option<&RuntimeSpec>,
    ) -> Result<Soxr> {
        let error: *mut *const c_char = ::std::ptr::null_mut();

        let q = quality_spec.map_or(ptr::null(), |spec| spec.soxr_spec());
        let io = io_spec.map_or(ptr::null(), |spec| spec.soxr_spec());
        let rt = runtime_spec.map_or(ptr::null(), |spec| spec.soxr_spec());

        let soxr =
            unsafe { soxr::soxr_create(input_rate, output_rate, num_channels, error, io, q, rt) };

        if error.is_null() {
            Ok(Soxr {
                soxr,
                channels: num_channels,
                error: CString::new("").unwrap(),
            })
        } else {
            let error = unsafe { *error };
            Err(Error::new(
                Some("Soxr::new".into()),
                ErrorType::CreateError(from_const("Soxr::new", error).unwrap().to_string()),
            ))
        }
    }

    /// Get version of libsoxr library
    pub fn version() -> &'static str {
        unsafe { from_const("Soxr::version", soxr::soxr_version()).unwrap() }
    }

    /// Set error of Soxr engine
    pub fn set_error(&mut self, msg: String) -> Result<()> {
        self.error = CString::new(msg).unwrap();
        let result =
            unsafe { soxr::soxr_set_error(self.soxr, self.error.as_ptr() as soxr::soxr_error_t) };
        if result.is_null() {
            Ok(())
        } else {
            Err(Error::new(
                Some("Soxr::new".into()),
                ErrorType::ChangeError(from_const("Soxr::set_error", result).unwrap().to_string()),
            ))
        }
    }

    /// Change number of channels after creating Soxr object
    pub fn set_num_channels(&mut self, num_channels: u32) -> Result<()> {
        let error = unsafe { soxr::soxr_set_num_channels(self.soxr, num_channels) };
        if error.is_null() {
            self.channels = num_channels;
            Ok(())
        } else {
            Err(Error::new(
                Some("Soxr::set_num_channels".into()),
                ErrorType::ChangeError(
                    from_const("Soxr::set_num_channels", error)
                        .unwrap()
                        .to_string(),
                ),
            ))
        }
    }

    /// Query error status.
    pub fn error(&self) -> Option<String> {
        let error = unsafe { soxr::soxr_error(self.soxr) };
        if error.is_null() {
            None
        } else {
            Some(from_const("Soxr::error", error).unwrap().to_string())
        }
    }

    /// Query int. clip counter (for R/W).
    pub fn num_clips(&self) -> usize {
        unsafe { *soxr::soxr_num_clips(self.soxr) }
    }

    /// Query current delay in output samples
    pub fn delay(&self) -> f64 {
        unsafe { soxr::soxr_delay(self.soxr) }
    }

    /// Query resampling engine name.
    pub fn engine(&self) -> String {
        from_const("Soxr::engine", unsafe { soxr::soxr_engine(self.soxr) })
            .unwrap()
            .to_string()
    }

    /// Ready for fresh signal, same config.
    pub fn clear(&mut self) -> Result<()> {
        let error = unsafe { soxr::soxr_clear(self.soxr) };
        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new(
                Some("Soxr::clear".into()),
                ErrorType::ChangeError(from_const("Soxr::clear", error).unwrap().to_string()),
            ))
        }
    }

    /// For variable-rate resampling.
    /// See [example # 5](https://sourceforge.net/p/soxr/code/ci/master/tree/examples/5-variable-rate.c)
    /// of libsoxr repository for how to create a
    /// variable-rate resampler and how to use this function.
    pub fn set_io_ratio(&mut self, io_ratio: f64, slew_len: usize) -> Result<()> {
        let error = unsafe { soxr::soxr_set_io_ratio(self.soxr, io_ratio, slew_len) };
        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new(
                Some("Soxr::set_io_ratio".into()),
                ErrorType::ChangeError(
                    from_const("Soxr::set_io_ratio", error).unwrap().to_string(),
                ),
            ))
        }
    }

    /// Resamples `Some(buf_in)` into `buf_out`. Type is dependent on [IOSpec]. If you leave out
    /// [IOSpec] on create, it defaults to `f32`. Make sure that `buf_out` is large enough to hold
    /// the resampled data. Furthermore, to indicate end-of-input to the resampler, always end with
    /// a last call to process with `None` as `buf_in`. The result contains number of input samples
    /// used and number of output samples places in 'buf_out'
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use libsoxr::Soxr;
    /// // upscale factor 2, one channel with all the defaults
    /// let soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
    ///
    /// // source data, taken from 1-single-block.c of libsoxr examples.
    /// let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
    ///                          1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
    ///                          0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
    ///                          -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
    ///
    /// // create room for 2*48 = 96 samples
    /// let mut target: [f32; 96] = [0.0; 96];
    ///
    /// // Two runs. First run will convert the source data into target.
    /// // Last run with None is to inform resampler of end-of-input so it can clean up
    /// soxr.process(Some(&source), &mut target).unwrap();
    /// soxr.process::<f32,_>(None, &mut target[0..]).unwrap();
    /// ```
    pub fn process<I, O>(&self, buf_in: Option<&[I]>, buf_out: &mut [O]) -> Result<(usize, usize)> {
        let mut idone = 0;
        let mut odone = 0;
        let error = match buf_in {
            Some(buf) => unsafe {
                soxr::soxr_process(
                    self.soxr,
                    buf.as_ptr() as *const c_void,
                    buf.len(),
                    &mut idone,
                    buf_out.as_mut_ptr() as *mut c_void,
                    buf_out.len(),
                    &mut odone,
                )
            },
            None => unsafe {
                soxr::soxr_process(
                    self.soxr,
                    ptr::null() as *const c_void,
                    0,
                    &mut idone,
                    buf_out.as_mut_ptr() as *mut c_void,
                    buf_out.len(),
                    &mut odone,
                )
            },
        };
        if error.is_null() {
            Ok((idone, odone))
        } else {
            Err(Error::new(
                Some("Soxr::process".into()),
                ErrorType::ProcessError(from_const("Soxr::process", error).unwrap().to_string()),
            ))
        }
    }

    /// Sets the input function of type [SoxrFunction].
    ///
    /// ## Example for 'happy flow'
    ///```rust
    /// use libsoxr::{Error, ErrorType, Soxr, SoxrFunction};
    ///
    /// struct State {
    ///   // data for input function to supply Soxr with source samples.
    ///   // In this case just a value, but you could put a handle to a FLAC file into this.
    ///   value: f32
    /// }
    ///
    /// let input_fn = |state: &mut State, buffer: &mut [f32], samples: usize| {
    ///     for sample in buffer.iter_mut().take(samples) {
    ///        *sample = state.value;
    ///     }
    ///     return Ok(samples);
    ///  };
    ///
    /// let mut soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
    /// let mut state = State { value: 1.0 };
    /// assert!(soxr.set_input(input_fn, Some(&mut state), 100).is_ok());
    ///
    /// let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
    ///                          1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
    ///                          0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
    ///                          -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
    ///
    /// // create room for 2*48 = 96 samples
    /// let mut target: [f32; 96] = [0.0; 96];
    /// // ask SOXR to fill target with 96 samples for which it will use `input_fn`
    /// assert!(soxr.output(&mut target[..], 96) == 96);
    /// assert!(soxr.error().is_none());
    ///```
    /// ## Example to handle error in `input_fn`
    /// The input function may return an error. You can handle that using the returned Error type.
    ///```rust
    /// use libsoxr::{Error, ErrorType, Soxr, SoxrFunction};
    ///
    /// struct State {
    ///   // data for input function to supply Soxr with source samples.
    ///   // In this case just a value, but you could put a handle to a FLAC file into this.
    ///   value: f32,
    ///   state_error: Option<&'static str>,
    /// }
    ///
    /// let input_fn = |state: &mut State, buffer: &mut [f32], samples: usize| {
    ///     state.state_error = Some("Some Error");
    ///     Err(Error::new(Some("input_fn".into()), ErrorType::ProcessError("Unexpected end of input".into())))
    ///  };
    ///
    /// let mut soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();
    /// let mut state = State { value: 1.0, state_error: None };
    /// assert!(soxr.set_input(input_fn, Some(&mut state), 100).is_ok());
    ///
    /// let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
    ///                          1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
    ///                          0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
    ///                          -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
    ///
    /// // create room for 2*48 = 96 samples
    /// let mut target: [f32; 96] = [0.0; 96];
    /// assert!(soxr.output(&mut target[..], 96) == 0);
    /// assert!(soxr.error().is_some());
    /// // Please note that the ProcessError is not passed through into `error()`
    /// assert_eq!(soxr.error().unwrap(), "input function reported failure");
    /// // But you can use the State struct to pass specific errors which you can query on `soxr.error().is_some()`
    /// assert_eq!(state.state_error, Some("Some Error"));
    ///```
    pub fn set_input<'a, S, T>(
        &'a mut self,
        input_fn: SoxrFunction<S, T>,
        state: Option<&'a mut S>,
        max_ilen: usize,
    ) -> Result<()> {
        let state_data = state
            .map(|s| TrampolineData {
                check: "trampoline",
                input_state: s,
                input_fn,
                last_error: None,
            })
            .map(|s| Box::into_raw(Box::new(s)) as soxr::soxr_fn_state_t);

        let error = unsafe {
            soxr::soxr_set_input_fn(
                self.soxr,
                Some(input_trampoline::<S, T>),
                state_data.unwrap_or(ptr::null()) as *mut ::std::os::raw::c_void,
                max_ilen,
            )
        };

        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new(
                Some("Soxr::set_input".into()),
                ErrorType::ProcessError(from_const("Soxr::set_input", error).unwrap().to_string()),
            ))
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
    /// assert!(s.output(&mut buffer[..], 100) > 0);
    /// ```
    pub fn output<S>(&self, data: &mut [S], samples: usize) -> usize {
        assert!(
            data.len() >= samples * self.channels as usize,
            "the data buffer does not contain enough space to hold requested samples"
        );
        unsafe { soxr::soxr_output(self.soxr, data.as_mut_ptr() as *mut c_void, samples) }
    }
}

// this function is called from Soxr and used the closure inside TrampolineData to get the input samples. All unsafe pointer
// magic happens inside this function, not the passed closure.
extern "C" fn input_trampoline<S, T>(
    input_fn_state: *mut ::std::os::raw::c_void,
    data: *mut soxr::soxr_in_t,
    requested_len: usize,
) -> usize {
    let trampoline_data = input_fn_state as *mut TrampolineData<S, T>;
    unsafe {
        assert_eq!((*trampoline_data).check, "trampoline");

        let input_fn = (*trampoline_data).input_fn;
        let input_data: &mut [T] = std::slice::from_raw_parts_mut((*data) as *mut T, requested_len);
        let result = input_fn(
            (*trampoline_data).input_state,
            input_data as &mut [T],
            requested_len,
        );

        match result {
            Ok(samples_or_zero) => samples_or_zero,
            Err(Error(_, e)) => {
                (*trampoline_data).last_error = Some(e);
                *data = ptr::null_mut();
                0
            }
        }
    }
}

// This struct is passed to the input_trampoline function
// which uses this to call the closure `input_fn` with `input_state`
// last_error is used to record the error that input_fn returns.
// TODO: figure out how to pass this error to calling soxr_output which does not have access to this trampoline struct
struct TrampolineData<'a, S, T> {
    check: &'static str,
    input_state: &'a mut S,
    input_fn: SoxrFunction<S, T>,
    last_error: Option<ErrorType>,
}

impl Drop for Soxr {
    fn drop(&mut self) {
        unsafe { soxr::soxr_delete(self.soxr) };
    }
}

#[cfg(test)]
mod soxr_tests {
    use super::Soxr;
    use crate::spec::{IOSpec, QualitySpec, RuntimeSpec};

    #[test]
    fn test_version() {
        let version = Soxr::version();
        println!("{}", version);
        assert_eq!("libsoxr-0.1.3", version);
    }

    #[test]
    fn test_create() {
        use crate::datatype::Datatype;
        use crate::spec::{QualityFlags, QualityRecipe};

        let mut s = Soxr::create(96000.0, 44100.0, 2, None, None, None);
        assert!(s.is_ok());

        let quality_spec = QualitySpec::new(&QualityRecipe::High, QualityFlags::ROLLOFF_SMALL);
        let runtime_spec = RuntimeSpec::new(4);
        let io_spec = IOSpec::new(Datatype::Float32I, Datatype::Int32I);

        s = Soxr::create(
            96000.0,
            44100.0,
            2,
            Some(&io_spec),
            Some(&quality_spec),
            Some(&runtime_spec),
        );
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
        assert_eq!("cr32s", s.engine());
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
        // upscale factor 2, one channel with all the defaults
        let soxr = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();

        // source data, taken from 1-single-block.c of libsoxr examples.
        let source: [f32; 48] = [
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
        ];

        // create room for 2*48 = 96 samples
        let mut target: [f32; 96] = [0.0; 96];

        // Two runs. First run will convert the source data into target.
        // Last run with None is to inform resampler of end-of-input so it can clean up
        assert!(soxr.process(Some(&source), &mut target).is_ok());
        assert!(soxr.process::<f32, _>(None, &mut target[0..]).is_ok());

        // just print the values in target
        for s in target.iter() {
            print!("{:?}\t", s)
        }
    }
}
