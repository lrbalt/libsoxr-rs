use approx::*;
use libsoxr::{
    spec::{QualityFlags, QualityRecipe},
    QualitySpec, Soxr,
};
use libsoxr_sys as soxr;

mod shared_input_fn;
use shared_input_fn::{TestState, test_input_fn};

#[test]
fn test_custom_spec() {
    // test for issue #2
    let spec = QualitySpec::new(&QualityRecipe::VeryHigh, QualityFlags::HI_PREC_CLOCK);
    let mut s = Soxr::create(1.0, 2.0, 1, None, Some(&spec), None).unwrap();

    // create state for input_fn
    let mut state = TestState {
        check: "libsoxr",
        command: 0,
        value: 2.3,
        source_buffer: [1.2; 100],
    };

    // convert to raw pointer to pass into libsoxr using set_input
    let state_data = Some(Box::into_raw(Box::new(&state)) as soxr::soxr_fn_state_t);
    assert!(s.set_input(Some(test_input_fn), state_data, 100).is_ok());

    // create buffer for resampled data
    let mut data = [1.1f32; 200];
    println!("first call");
    assert_eq!(200, s.output(&mut data[0..], 200));
    assert_abs_diff_ne!(data[0], 1.1);

    // tell test_input_fn to return end-of-input (0)
    state.command = 1;
    // other buffer for resampled data
    let mut buffer = [1.1f32; 200];
    println!("second");
    // flush all data from libsoxr until end-of-input
    while s.output(&mut buffer[0..], 200) > 0 {
        print!(".{}", buffer[0]);
        assert_abs_diff_ne!(buffer[0], 1.1);
    }
    println!();
}
