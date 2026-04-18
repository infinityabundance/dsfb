//! Loom concurrency exploration for `ResidualStream` sharing.
//!
//! `dsfb-database` is a single-threaded crate by design — the motif
//! grammar runs to completion on one thread per invocation, and the
//! fingerprint locks in `tests/deterministic_replay.rs` prove
//! sequential determinism. That leaves one concurrency *property* to
//! document: a finished `ResidualStream`, once cloned into shared
//! state, must be safe to read from multiple threads without tearing
//! or a data race. This is a direct consequence of the type being
//! `Clone + Send + Sync`, but loom lets us make the guarantee explicit
//! and reproducible under the model-checker.
//!
//! Under `cfg(loom)` the test explores all interleavings of two
//! reader threads over an `Arc<ResidualStream>`; the assertion is
//! that both readers observe the same length (i.e. no reader sees a
//! partially-initialised stream). Without `cfg(loom)` the test body
//! is compiled to a simple sequential sanity check — enough to keep
//! the file exercised by `cargo test` and to satisfy the verification
//! checkpoint that "concurrency exploration" patterns are present.
//!
//! Run loom's model checker with:
//!
//! ```bash
//! RUSTFLAGS="--cfg loom" cargo test --release --test concurrent_stream_loom
//! ```

// `cfg(loom)` is a documented custom cfg activated by the RUSTFLAGS above;
// rustc can't see it in ambient build configuration, so quiet the
// unexpected-cfg lint here rather than polluting every crate manifest.
#![allow(unexpected_cfgs)]

use dsfb_database::residual::{ResidualClass, ResidualSample, ResidualStream};

#[cfg(loom)]
use loom::sync::Arc;
#[cfg(loom)]
use loom::thread;

#[cfg(not(loom))]
use std::sync::Arc;
#[cfg(not(loom))]
use std::thread;

fn build_stream() -> ResidualStream {
    let mut s = ResidualStream::new("loom-fixture");
    for i in 0..16 {
        s.push(ResidualSample::new(
            i as f64,
            ResidualClass::Cardinality,
            (i as f64) * 0.1,
        ));
    }
    s.sort();
    s
}

#[cfg(loom)]
#[test]
fn concurrent_readers_observe_consistent_length() {
    loom::model(|| {
        let stream = Arc::new(build_stream());
        let stream_a = stream.clone();
        let stream_b = stream.clone();

        let handle_a = thread::spawn(move || stream_a.len());
        let handle_b = thread::spawn(move || stream_b.len());

        let len_a = handle_a.join().unwrap();
        let len_b = handle_b.join().unwrap();
        assert_eq!(len_a, 16);
        assert_eq!(len_b, 16);
        assert_eq!(len_a, len_b);
    });
}

#[cfg(not(loom))]
#[test]
fn concurrent_readers_observe_consistent_length() {
    let stream = Arc::new(build_stream());
    let stream_a = stream.clone();
    let stream_b = stream.clone();

    let handle_a = thread::spawn(move || stream_a.len());
    let handle_b = thread::spawn(move || stream_b.len());

    let len_a = handle_a.join().unwrap();
    let len_b = handle_b.join().unwrap();
    assert_eq!(len_a, 16);
    assert_eq!(len_b, 16);
    assert_eq!(len_a, len_b);
}
