#![no_main]

//! Fuzz target for every CSV-consuming adapter.
//!
//! The two existing fuzz targets only cover the Postgres CSV adapter and
//! the SQLShare-text adapter. This one extends coverage to the four
//! remaining `DatasetAdapter` implementations that take a CSV on disk —
//! CEB, JOB, Snowset, and SQLShare — so a fuzz run exercises every
//! trust-boundary parser in the crate.
//!
//! Dispatch: the first byte selects the adapter, the remainder is the
//! file contents. The fuzzer converges on coverage for whichever adapter
//! has the richest parse surface on that input without us having to
//! maintain four separate corpora.
//!
//! Invariants (per adapter):
//!   1. `load()` must never panic — every malformed row is either
//!      dropped by `filter_map(Result::ok)` or bubbled up as `Err`.
//!   2. On the `Ok` path the returned `ResidualStream` must carry a
//!      time-sorted sequence of finite-valued samples. The motif engine
//!      downstream assumes this; a malformed input that slipped through
//!      to produce NaN times or unsorted samples would be a reportable
//!      bug.

use dsfb_database::adapters::{
    ceb::Ceb, job::Job, snowset::Snowset, sqlshare::SqlShare, DatasetAdapter,
};
use libfuzzer_sys::fuzz_target;
use std::io::Write;

fn assert_stream_invariants(stream: &dsfb_database::residual::ResidualStream) {
    let mut prev_t = f64::NEG_INFINITY;
    for s in &stream.samples {
        assert!(s.t.is_finite(), "adapter emitted non-finite t = {}", s.t);
        assert!(
            s.value.is_finite(),
            "adapter emitted non-finite value = {}",
            s.value
        );
        assert!(
            s.t >= prev_t,
            "adapter emitted out-of-order sample: {} then {}",
            prev_t,
            s.t
        );
        prev_t = s.t;
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let selector = data[0] % 4;
    let body = &data[1..];

    let Ok(mut tmp) = tempfile::NamedTempFile::new() else {
        return;
    };
    if tmp.write_all(body).is_err() {
        return;
    }
    if tmp.flush().is_err() {
        return;
    }
    let path = tmp.path();

    let result = match selector {
        0 => Ceb.load(path),
        1 => Job.load(path),
        2 => Snowset.load(path),
        _ => SqlShare.load(path),
    };
    if let Ok(stream) = result {
        assert_stream_invariants(&stream);
    }
});
