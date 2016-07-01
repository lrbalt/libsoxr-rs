# libsoxr-rs
This library is a thin wrapper for [libsoxr](https://sourceforge.net/projects/soxr/) which is a "High quality, one-dimensional sample-rate conversion library".

This wrapper library is licensed the same as libsoxr itself: GPLv2.

The documentation can be found [here](https://lrbalt.github.io/libsoxr-rs/libsoxr/).

Example:

```rust
// upscale factor 2, one channel with all the defaults
let s = Soxr::create(1.0, 2.0, 1, None, None, None).unwrap();

// source data, taken from 1-single-block.c of libsoxr examples.
let source: [f32; 48] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
                         1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0,
                         0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
                         -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];

// create room for 2*48 = 96 samples
let mut target: [f32; 96] = [0.0; 96];

// two runs. last run with None to inform resampler of end-of-input
let result = s.process(Some(&source), &mut target).and_then(|_| {
    s.process::<f32, f32>(None, &mut target[0..]).and_then(|_| {
        for s in target.iter() {
            print!("{:?}\t", s)
        }
        Ok(())
    })
});
```
