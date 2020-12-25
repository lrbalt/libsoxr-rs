use approx::*;
use libsoxr::Soxr;

pub struct MyState {
    check: &'static str,
    command: u32,
    value: f32,
    channels: usize,
}

fn test_input_fn(state: &mut MyState, buf: &mut [f32], req_len: usize) -> usize {
    assert_eq!("libsoxr", state.check);

    print!(
        "setting {} values for {} with {} channels and with cmd {}\t",
        req_len, state.check, state.channels, state.command
    );

    // just for this test, so we can check the output
    let value_to_use = state.value;

    // be aware: req_len is number of samples per channel, so for stereo you want to return
    // req_len * 2 values in buf
    for value in buf.iter_mut().take(req_len) {
        *value = value_to_use;
    }

    // for testing: set command to non-zero to force end-of-input (eoi)
    if state.command == 0 {
        println!("returning {:?}", req_len);
        req_len
    } else {
        println!("eoi");
        0
    }
}

#[test]
fn test_data_fn() {
    println!("Creating Soxr");
    let mut s = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();

    // create state for input_fn
    let mut state = MyState {
        check: "libsoxr",
        command: 0,
        value: 2.3,
        channels: 2,
    };

    println!("Setting input function");
    assert!(s
        .set_input_safe::<MyState, f32>(test_input_fn, Some(&mut state), 100)
        .is_ok());

    // create buffer for resampled data
    let mut data = [1.1f32; 200];
    println!("first call");
    assert_eq!(100, s.output(&mut data[0..], 100));
    assert_abs_diff_ne!(data[0], 1.1);

    // tell test_input_fn to return end-of-input (0)
    state.command = 1;
    // other buffer for resampled data
    let mut buffer = [1.1f32; 200];
    println!("second");
    // flush all data from libsoxr until end-of-input
    while s.output(&mut buffer[0..], 100) > 0 {
        print!(".");
        assert_abs_diff_ne!(buffer[0], 1.1);
    }
    println!();
}
