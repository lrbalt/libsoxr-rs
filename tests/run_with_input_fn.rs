use approx::*;
use libsoxr::{QualityFlags, QualityRecipe, QualitySpec, Result, Soxr};

pub struct MyState {
    check: &'static str,
    command: u32,
    value: f32,
    samples_created: usize,
    channels: usize,
}

fn test_input_fn(state: &mut MyState, buf: &mut [f32], req_samples: usize) -> Result<usize> {
    assert_eq!("libsoxr", state.check);

    print!(
        "@@@@ Setting {} samples for {} channels values for {} and with cmd {}: ",
        req_samples, state.channels, state.check, state.command
    );

    // just for this test, so we can check the output
    let value_to_use = state.value;

    // be aware: req_samples is number of samples per channel, so for stereo you want to return
    // req_samples * 2 values in buf
    for value in buf.iter_mut().take(req_samples * state.channels) {
        *value = value_to_use;
    }
    state.samples_created += req_samples;

    // for testing: set command to non-zero to force end-of-input (eoi)
    if state.command == 0 {
        println!("returning {:?}", req_samples);
        Ok(req_samples)
    } else {
        println!("eoi");
        Ok(0)
    }
}

#[test]
fn test_data_fn() {
    println!("Creating Soxr");
    let mut soxr = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();

    // create state for input_fn
    let mut state = MyState {
        check: "libsoxr",
        command: 0,
        value: 2.3,
        channels: 2,
        samples_created: 0,
    };

    println!("Setting input function");
    assert!(soxr.set_input(test_input_fn, Some(&mut state), 75).is_ok());

    // create buffer for resampled data
    let mut data = [1.1f32; 300];
    println!("First call");
    assert_eq!(150, soxr.output(&mut data, 150));
    assert_abs_diff_ne!(data[0], 1.1);

    // tell test_input_fn to return end-of-input (0)
    state.command = 1;
    // other buffer for resampled data
    let mut buffer = [1.1f32; 200];
    println!("Second");
    // flush all data from libsoxr until end-of-input
    while soxr.output(&mut buffer, 100) > 0 {
        print!(".");
        assert_abs_diff_ne!(buffer[0], 1.1);
    }
    println!();
}

#[test]
fn test_with_custom_specs() {
    println!("Creating Soxr");
    let spec = QualitySpec::new(&QualityRecipe::VeryHigh, QualityFlags::HI_PREC_CLOCK);
    let mut soxr = Soxr::create(100.0, 200.0, 2, None, Some(&spec), None).unwrap();

    // create state for input_fn
    let mut state = MyState {
        check: "libsoxr",
        command: 0,
        samples_created: 0,
        value: 2.3,
        channels: 2,
    };

    println!("Setting input function");
    assert!(soxr.set_input(test_input_fn, Some(&mut state), 500).is_ok());

    // create buffer for resampled data
    let mut data = [1.1f32; 2000];
    println!("First call");
    assert_eq!(1000, soxr.output(&mut data, 1000));
    println!("First call done");
    assert_eq!(1000, state.samples_created);
    assert_abs_diff_ne!(data[0], 1.1);

    // tell test_input_fn to return end-of-input (0)
    state.command = 1;
    // other buffer for resampled data
    let mut buffer = [1.1f32; 200];
    println!("Second");
    // flush all data from libsoxr until end-of-input
    while soxr.output(&mut buffer, 100) > 0 {
        print!(".");
        assert_abs_diff_ne!(buffer[0], 1.1);
    }
    println!();
}

#[test]
fn test_with_two_soxrs() {
    println!("Creating Soxr 1");
    let mut soxr1 = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();
    soxr1.clear().unwrap();

    // create state for input_fn
    let mut state = MyState {
        check: "libsoxr",
        command: 0,
        value: 2.3,
        channels: 2,
        samples_created: 0,
    };

    println!("Setting input function 1");
    assert!(soxr1
        .set_input(test_input_fn, Some(&mut state), 2500)
        .is_ok());

    println!("Creating Soxr 2");
    let mut soxr2 = Soxr::create(1.0, 2.0, 2, None, None, None).unwrap();
    soxr2.clear().unwrap();

    println!("Setting input function 2");
    assert!(soxr2
        .set_input(test_input_fn, Some(&mut state), 2500)
        .is_ok());

    // create buffer for resampled data; 5000 * 2 channels
    let mut data1 = [1.1f32; 10000];
    let mut data2 = [1.1f32; 10000];
    println!("First call");
    assert_eq!(5000, soxr1.output(&mut data1, 5000));
    assert_eq!(5000, soxr2.output(&mut data2, 5000));
    assert_abs_diff_ne!(data1[0], 1.1);
    assert_abs_diff_ne!(data2[0], 1.1);

    // other buffer for resampled data
    let mut buffer1 = [1.1f32; 5000];
    let mut buffer2 = [1.1f32; 5000];
    println!("Second");
    // flush all data from libsoxr until end-of-input
    loop {
        let x = soxr1.output(&mut buffer1, 2500);
        let y = soxr2.output(&mut buffer2, 2500);
        print!(".");
        assert_abs_diff_ne!(buffer1[0], 1.1);
        assert_abs_diff_ne!(buffer2[0], 1.1);
        if x == 0 || y == 0 {
            break;
        }
        // tell test_input_fn to return end-of-input (0)
        state.command = 1;
    }
    println!();
}
