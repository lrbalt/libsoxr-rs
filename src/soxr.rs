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
    io_spec: Option<IOSpec>,
    error: CString,
    last_trampoline_data: Option<*mut ::std::os::raw::c_void>,
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
                io_spec: io_spec.cloned(),
                error: CString::new("").unwrap(),
                last_trampoline_data: None,
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
    /// used and number of output samples placed in 'buf_out'
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
        let mut idone_in_samples = 0;
        let mut odone_in_samples = 0;

        let mut split_buf_in: Vec<*const c_void> = Vec::with_capacity(self.channels as usize);
        let mut split_buf_out: Vec<*mut c_void> = Vec::with_capacity(self.channels as usize);

        let error = match buf_in {
            Some(buf_in) => unsafe {
                let samples_in_buf_in = buf_in.len() / self.channels as usize;
                let samples_in_buf_out = buf_out.len() / self.channels as usize;

                let buf_in_ptr = self.get_buf_in_ptr(buf_in, &mut split_buf_in);
                let buf_out_ptr = self.get_buf_out_ptr(buf_out, &mut split_buf_out);

                soxr::soxr_process(
                    self.soxr,
                    buf_in_ptr,
                    samples_in_buf_in,
                    &mut idone_in_samples,
                    buf_out_ptr,
                    samples_in_buf_out,
                    &mut odone_in_samples,
                )
            },
            None => unsafe {
                let buf_out_ptr = self.get_buf_out_ptr(buf_out, &mut split_buf_out);

                soxr::soxr_process(
                    self.soxr,
                    ptr::null() as *const c_void,
                    0,
                    &mut idone_in_samples,
                    buf_out_ptr,
                    buf_out.len() / self.channels as usize,
                    &mut odone_in_samples,
                )
            },
        };
        if error.is_null() {
            Ok((idone_in_samples, odone_in_samples))
        } else {
            Err(Error::new(
                Some("Soxr::process".into()),
                ErrorType::ProcessError(from_const("Soxr::process", error).unwrap().to_string()),
            ))
        }
    }

    fn get_buf_in_ptr<I>(&self, buf_in: &[I], split_buf: &mut Vec<*const c_void>) -> *const c_void {
        let Some(io_spec) = self.io_spec.as_ref() else {
            // assume interleaved
            return buf_in.as_ptr() as *const c_void;
        };
        
        if io_spec.input_type().is_interleaved() {
            return buf_in.as_ptr() as *const c_void;
        }
        
        let samples_in_buf = buf_in.len() / self.channels as usize;
        for channel in 0..self.channels as usize {
            split_buf.push(buf_in[channel * samples_in_buf..].as_ptr() as *const c_void);
        }
        split_buf.as_ptr() as *const c_void        
    }

    fn get_buf_out_ptr<O>(&self, buf_out: &[O], split_buf: &mut Vec<*mut c_void>) -> *mut c_void {
        let Some(io_spec) = self.io_spec.as_ref() else {
            // assume interleaved
            return buf_out.as_ptr() as *mut c_void;
        };
        
        if io_spec.input_type().is_interleaved() {
            return buf_out.as_ptr() as *mut c_void;
        }
        
        let samples_in_buf = buf_out.len() / self.channels as usize;
        for channel in 0..self.channels as usize {
            split_buf.push(buf_out[channel * samples_in_buf..].as_ptr() as *mut c_void);
        }
        split_buf.as_ptr() as *mut c_void        
    }

    /// Sets the input function of type [SoxrFunction].
    ///
    /// Please note that SoxrFunction gets a buffer as parameter which the function should fill.
    /// This is different from native `libsoxr` where you need to return the used input buffer from the input function.
    ///
    /// The input buffer is allocated for you using a Vec<T> with `initial_capacity` set to `max_samples * channels`
    /// that you supplied.
    ///
    /// Please note that the state you pass into `set_input` may not be moved in memory. Thus it is wise to keep the
    /// state in a Box (heap) and not on the stack. TODO: check if `Pin`is an option here.
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
    /// let mut state = Box::new(State { value: 1.0 });
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
    /// let mut state = Box::new(State { value: 1.0, state_error: None });
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
        max_samples: usize,
    ) -> Result<()> {
        self.drop_last_trampoline();

        self.last_trampoline_data = state
            .map(|s| TrampolineData {
                check: "trampoline",
                input_state: s,
                input_fn,
                last_error: None,
                input_buffer_size: max_samples * self.channels as usize,
                input_buffer: Vec::<T>::with_capacity(max_samples * self.channels as usize),
            })
            .map(|s| Box::into_raw(Box::new(s)) as soxr::soxr_fn_state_t_mut);

        let error = unsafe {
            soxr::soxr_set_input_fn(
                self.soxr,
                Some(input_trampoline::<S, T>),
                self.last_trampoline_data.unwrap_or(ptr::null_mut()) as *mut ::std::os::raw::c_void,
                max_samples,
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

    fn drop_last_trampoline(&mut self) {
        if let Some(last_trampoline_data) = self.last_trampoline_data {
            // unsafe assumption: we assume here that dropping TrampolineData<S,T> always equals TrampolineData<i32,f32>
            // this is assumed because TrampolineData only contains references to S or T, but not instances of S or T
            // This assumption is unit tested below
            let data: Box<TrampolineData<i32, f32>> =
                unsafe { Box::from_raw(last_trampoline_data as *mut TrampolineData<i32, f32>) };
            drop(data);

            self.last_trampoline_data = None;
        }
    }
}

// this function is called from Soxr and uses the closure inside TrampolineData
// to get the input samples. All unsafe pointer magic happens inside this
// function, not inside the passed closure.
extern "C" fn input_trampoline<S, T>(
    input_fn_state: *mut ::std::os::raw::c_void,
    data: *mut soxr::soxr_in_t,
    requested_number_of_samples: usize,
) -> usize {
    unsafe {
        let trampoline_data = &mut *(input_fn_state as *mut TrampolineData<S, T>);
        assert_eq!(trampoline_data.check, "trampoline");

        let input_data: &mut [T] = std::slice::from_raw_parts_mut(
            trampoline_data.input_buffer.as_mut_slice().as_mut_ptr(),
            trampoline_data.input_buffer_size,
        );

        let result = (trampoline_data.input_fn)(
            trampoline_data.input_state,
            input_data,
            requested_number_of_samples,
        );

        match result {
            Ok(samples_or_zero) => {
                *data = trampoline_data.input_buffer.as_slice().as_ptr() as soxr::soxr_in_t;
                samples_or_zero
            }
            Err(Error(_, e)) => {
                trampoline_data.last_error = Some(e);
                *data = ptr::null_mut();
                0
            }
        }
    }
}

// This struct is passed to the input_trampoline function
// which uses it to call the closure `input_fn` with `input_state`
// last_error is used to record the error that input_fn returns.
// TODO: figure out how to pass this error to calling soxr_output which does not have access to this trampoline struct
struct TrampolineData<'a, S, T> {
    check: &'static str,
    input_state: &'a mut S,
    input_fn: SoxrFunction<S, T>,
    last_error: Option<ErrorType>,
    input_buffer_size: usize,
    input_buffer: Vec<T>,
}

impl Drop for Soxr {
    fn drop(&mut self) {
        // clean up memory used for trampoline data
        self.drop_last_trampoline();

        // let soxr clean up itself
        unsafe { soxr::soxr_delete(self.soxr) };
    }
}

#[cfg(test)]
mod soxr_tests {
    use approx::assert_abs_diff_eq;

    use super::{Soxr, TrampolineData};
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
        assert!(s.error().is_none());
    }

    #[test]
    fn test_engine() {
        let s = Soxr::create(96000.0, 44100.0, 2, None, None, None).unwrap();
        // cr32 on Linux, but cr32s on MacOS
        assert!(s.engine() == "cr32s" || s.engine() == "cr32");
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
    fn test_process_mono() {
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

    #[test]
    fn test_process_stereo() {
        // upscale factor 2, one channel with all the defaults
        let soxr = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();

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

    #[test]
    fn test_process_stereo_2() {
        // Example from https://github.com/lrbalt/libsoxr-rs/issues/4

        let soxr = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();
        let mut in_buf: [f32; 2000] = [1.0; 2000];
        for n in 1000..2000 {
            in_buf[n] = -1.0
        }

        let mut out_buf: [f32; 4000] = [999.0; 4000];
        let (i1, o1) = soxr
            .process(Some(&in_buf[0..1000]), &mut out_buf[0..2000])
            .unwrap();
        println!("i = {}, o = {}", i1, o1);
        let (i2, o2) = soxr.process::<f32, _>(None, &mut out_buf[0..2000]).unwrap();
        println!("i = {}, o = {}", i2, o2);

        assert_eq!(500, i1); // 1000 for 2 channels --> 500 samples
        assert_eq!(1000, o2); // 500 samples 1Hz to 2Hz means more than 500 samples

        let mut has_negative = false;
        for (i, v) in out_buf.iter().enumerate() {
            if *v < 0.0 {
                has_negative = true;
                println!("{}: {}", i, v);
                break;
            }
        }

        println!("\n{:?}", out_buf);

        assert!(
            !has_negative,
            "Output contains negative samples. in_buf was read out of bounds!"
        );

        // Output sentinel overwritten. out_buf was written out of bounds!
        assert_abs_diff_eq!(out_buf[2000], 999.0);
    }

    #[test]
    fn test_drop_assumption() {
        // Drop assumes the following two concrete types of TrampolineData have same size
        // so dropping TrampolineData<S,T> without knowing S or T is same as dropping
        // TrampolineData<i32, f32> (or any type for that matter)
        struct MyData {
            _v1: f32,
            _v2: Vec<f32>,
        }

        let s1 = std::mem::size_of::<TrampolineData<i32, f32>>();
        let s2 = std::mem::size_of::<TrampolineData<MyData, f64>>();

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_unsafe_drop() {
        struct MyState {
            check: &'static str,
        }

        fn test_input_fn(
            state: &mut MyState,
            buf: &mut [f32],
            req_samples: usize,
        ) -> crate::Result<usize> {
            if state.check != "test" {
                println!("check failed");
            }
            for value in buf.iter_mut().take(req_samples * 2) {
                *value = 1.1;
            }
            Ok(req_samples)
        }

        let mut i = 0;
        while i < 100 {
            {
                let mut soxr = Soxr::create(1.0, 1.0, 2, None, None, None).unwrap();
                let mut state = MyState { check: "test" };
                let mut buffer = [0.0f32; 5000];
                soxr.set_input(test_input_fn, Some(&mut state), 500)
                    .unwrap();
                let mut j = 0;
                while j < 1_000 {
                    let out = soxr.output(&mut buffer, 2500);
                    assert_eq!(2500, out);
                    j += 1;
                }
                for value in buffer.iter() {
                    assert_abs_diff_eq!(1.1f32, *value);
                }
            } // drop
            i += 1;
        }
    }

    #[test]
    fn test_split_channels() {
        use crate::Datatype::{Float32S, Float64S};

        let io_spec = IOSpec::new(Float32S, Float64S);

        // upscale factor 2, one channel with all the defaults
        let soxr = Soxr::create(1.0, 2.0, 2, Some(&io_spec), None, None).unwrap();

        // source data, taken from 1-single-block.c of libsoxr examples.
        let source: [f32; 96] = [
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
        ];

        // create room for 2*96 = 192 samples
        let mut target: [f64; 192] = [0.0; 192];

        // Two runs. First run will convert the source data into target.
        // Last run with None is to inform resampler of end-of-input so it can clean up
        soxr.process(Some(&source), &mut target).unwrap();
        soxr.process::<f32, _>(None, &mut target).unwrap();

        // just print the values in target
        println!("{:?}", target);
        println!("{:?}", target.len());
    }

    #[test]
    fn test_interleaved_channels() {
        use crate::Datatype::{Float32I, Float64I};

        let io_spec = IOSpec::new(Float32I, Float64I);

        // upscale factor 2, one channel with all the defaults
        let soxr = Soxr::create(1.0, 2.0, 2, Some(&io_spec), None, None).unwrap();

        // source data, taken from 1-single-block.c of libsoxr examples.
        let source: [f32; 96] = [
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
        ];

        // create room for 2*96 = 192 samples
        let mut target: [f64; 192] = [0.0; 192];

        // Two runs. First run will convert the source data into target.
        // Last run with None is to inform resampler of end-of-input so it can clean up
        soxr.process(Some(&source), &mut target).unwrap();
        soxr.process::<f32, _>(None, &mut target).unwrap();

        // just print the values in target
        println!("{:?}", target);
        println!("{:?}", target.len());
    }
}
