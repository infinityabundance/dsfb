#![no_main]

//! Fuzz target for `MotifParams`.
//!
//! The motif grammar exposes five tunable knobs per class (ρ, σ₀,
//! drift_threshold, slew_threshold, min_dwell_seconds). A reviewer's
//! obvious question is *"does a pathological parameter choice crash the
//! engine or produce NaN-polluted episodes?"* This target takes a fuzzed
//! byte sequence, derives one `MotifParams` instance from it (identical
//! parameters for all five classes — keeps the search space small
//! enough to hit coverage), and runs the engine over a fixed 60 s slice
//! of the seed-42 TPC-DS perturbation stream.
//!
//! Invariants:
//!   1. `MotifEngine::run` must never panic.
//!   2. Every emitted `Episode.ema_at_boundary` must be finite.
//!   3. Every emitted `Episode.peak` must be finite and non-negative.
//!   4. `trust_sum` is the DSFB observer's aggregate weight at the
//!      boundary; the observer normalises to 1.0, so the audit invariant
//!      is `trust_sum ∈ [0.99, 1.01]`.
//!   5. `t_end >= t_start`.
//!
//! Non-finite or non-positive parameter values are mapped to the
//! crate's published defaults — the fuzzer is meant to exercise *legal*
//! parameter ranges, not to test that we gracefully reject garbage (the
//! config loader's job, not the engine's).

use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar, MotifParams};
use dsfb_database::perturbation::tpcds_with_perturbations;
use dsfb_database::residual::ResidualStream;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

static STREAM: OnceLock<ResidualStream> = OnceLock::new();

/// Consume 8 bytes from `data` at offset `i`, interpret as a little-
/// endian f64, coerce to `fallback` if non-finite or out-of-range.
fn bounded_f64(data: &[u8], i: usize, lo: f64, hi: f64, fallback: f64) -> f64 {
    let start = i.checked_mul(8).unwrap_or(usize::MAX);
    let end = start.checked_add(8).unwrap_or(usize::MAX);
    if end > data.len() {
        return fallback;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[start..end]);
    let v = f64::from_le_bytes(buf);
    if !v.is_finite() || v < lo || v > hi {
        return fallback;
    }
    v
}

fn params_from_bytes(data: &[u8]) -> MotifParams {
    MotifParams {
        // ρ must be in [0,1) — an EMA smoothing factor.
        rho: bounded_f64(data, 0, 0.0, 0.9999, 0.9),
        // σ₀ > 0 — a trust-softness scale.
        sigma0: bounded_f64(data, 1, 1e-6, 10.0, 0.05),
        // Thresholds strictly positive; capped at 100 to keep the
        // episode mass non-zero on a realistic stream.
        drift_threshold: bounded_f64(data, 2, 1e-6, 100.0, 0.1),
        slew_threshold: bounded_f64(data, 3, 1e-6, 100.0, 0.3),
        // Dwell capped at 600 s — longer than the stream's worth of any
        // single episode, exercising the "always-rejected" branch too.
        min_dwell_seconds: bounded_f64(data, 4, 0.0, 600.0, 5.0),
    }
}

fuzz_target!(|data: &[u8]| {
    let stream = STREAM.get_or_init(|| tpcds_with_perturbations(42).0);

    let p = params_from_bytes(data);
    let grammar = MotifGrammar {
        plan_regression_onset: p.clone(),
        cardinality_mismatch_regime: p.clone(),
        contention_ramp: p.clone(),
        cache_collapse: p.clone(),
        workload_phase_transition: p,
    };
    let engine = MotifEngine::new(grammar);
    let episodes = engine.run(stream);
    for e in &episodes {
        assert!(
            e.ema_at_boundary.is_finite(),
            "non-finite ema_at_boundary = {}",
            e.ema_at_boundary
        );
        assert!(
            e.peak.is_finite() && e.peak >= 0.0,
            "peak must be finite and non-negative, got {}",
            e.peak
        );
        assert!(
            e.trust_sum.is_finite() && (0.99..=1.01).contains(&e.trust_sum),
            "trust_sum out of [0.99, 1.01]: {}",
            e.trust_sum
        );
        assert!(
            e.t_end >= e.t_start,
            "t_end {} < t_start {}",
            e.t_end,
            e.t_start
        );
        assert!(MotifClass::ALL.contains(&e.motif), "unknown motif class");
    }
});
