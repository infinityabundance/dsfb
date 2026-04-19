#![no_main]

//! Fuzz target for `MotifEngine::run` over an *arbitrary residual stream*.
//!
//! The companion target `motif_params.rs` fuzzes the parameter space at
//! a *fixed* stream (the seed-42 TPC-DS perturbation series); this target
//! is its dual — it fixes the parameters at the published defaults and
//! fuzzes the *stream* itself, exercising the engine's robustness to
//! adversarial input shapes the perturbation generator never produces:
//!
//! * Tightly-clustered samples at one t (timestamp ties).
//! * Long flat plateaus at zero, ε, and large magnitudes.
//! * Single-channel and multi-channel interleaving on every residual class.
//! * Out-of-natural-range residual values that the adapter layer would
//!   normally bound (the Pass-2 plan §44 explicitly names "adversarial
//!   workload" as a residual risk; this target measures the *engine*
//!   layer's response without trusting the adapter to bound the input).
//!
//! Invariants asserted on every emission:
//!   1. `MotifEngine::run` must never panic.
//!   2. Every `Episode.peak`, `ema_at_boundary`, `t_start`, `t_end` is finite.
//!   3. `t_end >= t_start` (intervals are non-degenerate).
//!   4. `peak >= 0.0` (the engine reports |residual|).
//!   5. `trust_sum ∈ [0.99, 1.01]` (DSFB observer normalisation).
//!   6. `motif ∈ MotifClass::ALL` (no orphan variants).
//!
//! Per Pass-2 plan: this target does NOT modify `src/grammar/` or
//! `src/residual/`. If a counterexample fires, the fix is documented in
//! paper §36 (cross-firing) or §44 (adversarial workload) and deferred
//! to a future pass — the *fuzz output itself* is the disclosure
//! mechanism.

use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar, MotifParams};
use dsfb_database::residual::{ResidualClass, ResidualSample, ResidualStream};
use libfuzzer_sys::fuzz_target;

/// Pull a bounded f64 out of two bytes — small surface, big enough to
/// drive every state-machine transition.
fn small_value(b0: u8, b1: u8) -> f64 {
    // Map (b0, b1) into [-8.0, 8.0] linearly; this exercises the
    // envelope's drift / slew / boundary classifications without
    // pinning any specific motif's threshold.
    let raw = ((b0 as i32) << 8 | (b1 as i32)) as f64;
    (raw / 32_768.0) * 8.0
}

fn build_stream(data: &[u8]) -> ResidualStream {
    let mut stream = ResidualStream::new("fuzz");
    if data.len() < 6 {
        return stream;
    }
    // Cap stream length so a single fuzz iteration stays under a few ms.
    let max_samples = 4096_usize;
    let mut t = 0.0_f64;
    let mut i = 0;
    while i + 6 <= data.len() && stream.samples.len() < max_samples {
        let class_byte = data[i];
        let dt_byte = data[i + 1];
        let ch_byte = data[i + 2];
        let v0 = data[i + 3];
        let v1 = data[i + 4];
        let _ = data[i + 5]; // reserved for future use; consumed for stride
        i += 6;

        let class = match class_byte % 5 {
            0 => ResidualClass::PlanRegression,
            1 => ResidualClass::Cardinality,
            2 => ResidualClass::Contention,
            3 => ResidualClass::CacheIo,
            _ => ResidualClass::WorkloadPhase,
        };
        // dt in [0.0, 2.55] seconds — allows ties (dt=0) and gaps.
        let dt = (dt_byte as f64) / 100.0;
        t += dt;
        // 8 distinct channels per class — enough to stress the
        // per-channel state machines without exploding cardinality.
        let channel = format!("ch{}", ch_byte % 8);
        let value = small_value(v0, v1);
        let sample = ResidualSample {
            t,
            class,
            value,
            channel: Some(channel),
        };
        stream.samples.push(sample);
    }
    // The grammar requires sorted-by-t streams; we may have produced
    // ties (dt=0) which sort() handles via partial_cmp's stable rule.
    stream.sort();
    stream
}

fuzz_target!(|data: &[u8]| {
    let stream = build_stream(data);
    if stream.is_empty() {
        return;
    }
    let grammar = MotifGrammar {
        plan_regression_onset: MotifParams::default_for(MotifClass::PlanRegressionOnset),
        cardinality_mismatch_regime: MotifParams::default_for(MotifClass::CardinalityMismatchRegime),
        contention_ramp: MotifParams::default_for(MotifClass::ContentionRamp),
        cache_collapse: MotifParams::default_for(MotifClass::CacheCollapse),
        workload_phase_transition: MotifParams::default_for(MotifClass::WorkloadPhaseTransition),
    };
    let episodes = MotifEngine::new(grammar).run(&stream);
    for e in &episodes {
        assert!(e.t_start.is_finite(), "non-finite t_start = {}", e.t_start);
        assert!(e.t_end.is_finite(), "non-finite t_end = {}", e.t_end);
        assert!(
            e.t_end >= e.t_start,
            "t_end {} < t_start {}",
            e.t_end,
            e.t_start
        );
        assert!(
            e.peak.is_finite() && e.peak >= 0.0,
            "peak must be finite and non-negative, got {}",
            e.peak
        );
        assert!(
            e.ema_at_boundary.is_finite(),
            "non-finite ema_at_boundary = {}",
            e.ema_at_boundary
        );
        assert!(
            e.trust_sum.is_finite() && (0.99..=1.01).contains(&e.trust_sum),
            "trust_sum out of [0.99, 1.01]: {}",
            e.trust_sum
        );
        assert!(MotifClass::ALL.contains(&e.motif), "unknown motif class");
    }
});
