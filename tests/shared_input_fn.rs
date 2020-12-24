use libsoxr_sys as soxr;

pub extern "C" fn test_input_fn(
    state: soxr::soxr_fn_state_t_mut,
    buf: *mut soxr::soxr_in_t,
    req_len: usize,
) -> usize {
    unsafe {
        // Cast to raw point to a TestState. Beware not to use Box::from_raw as it will
        // try to destroy the struct that is managed elsewere
        let s: *mut &mut TestState = state as *mut &mut TestState;

        // check on a known string in TestState to make sure the raw pointer is pointing
        // to our TestState
        assert_eq!("libsoxr", (*s).check);

        print!(
            "setting {}/{} values for {} with {}\t",
            req_len,
            (**s).source_buffer.len(),
            (**s).check,
            (**s).command
        );

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
            *buf = data.as_ptr() as soxr::soxr_in_t;
        }

        // for testing: set command to non-zero to force end-of-input (eoi)
        if (*s).command == 0 {
            println!("returning {:?}", req_len);
            req_len
        } else {
            println!("eoi");
            0
        }
    }
}
pub struct TestState {
    pub check: &'static str,
    pub command: u32,
    pub value: f32,
    // use this buffer for source samples to avoid repeated allocations
    pub source_buffer: [f32; 100],
}
