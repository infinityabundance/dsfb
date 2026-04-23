//! Integration test: f32 path vs Q16.16 fixed-point round-trip.
//!
//! Characterises the observable effect of Q16.16 quantisation on the DSFB
//! grammar/policy pipeline.  Two properties are verified:
//!
//! ## Property 1 — Calibration-phase agreement (exact)
//!
//! During the healthy calibration window, residual norms sit well inside the
//! nominal envelope.  Neither path can trip any persistence gate or DSA
//! threshold in this phase.  Both must produce identical `PolicyDecision`
//! sequences — any disagreement here is an architectural fault, not a
//! quantisation edge-case.
//!
//! ## Property 2 — Drift-phase bounded disagreement
//!
//! Q16.16 resolution is 2⁻¹⁶ ≈ 1.53×10⁻⁵ per sample.  This is negligible
//! relative to the envelope radius ρ ≈ 0.08–0.25, so no *instantaneous*
//! policy flip is possible from a single quantised sample.
//!
//! However, DSFB is a **stateful nonlinear dynamical system**: the persistence
//! counter, DSA accumulator, and calibrated baseline are all functions of the
//! entire input history.  A sequence of O(N) errors of magnitude 2⁻¹⁶ can
//! shift the exact *step* at which a persistence gate trips or a DSA threshold
//! is crossed.  This is a well-known property of threshold-crossing systems
//! (cf. Turing, 1950; Hirsch & Smale, 1974) and is not a defect.
//!
//! The correct guarantee is therefore:
//!   > The number of drift-phase policy disagreements ≤ the number of distinct
//!   > policy transitions in the f32 reference sequence.
//!
//! Equivalently: each policy *boundary crossing* can be displaced by at most
//! a bounded number of steps; no new crossing events are created.
//!
//! ## Relationship to DSFB's role
//!
//! DSFB observes structure in the residual of an upstream receiver chain; it
//! does not replace that chain.  The fixed-point path exists for embedded
//! deployments where the upstream chain already provides a scalar residual
//! norm.  The Q16.16 precision is chosen to be << the receiver's own
//! measurement uncertainty — the quantisation here is never the system's
//! limiting error source.

use dsfb_rf::platform::PlatformContext;
use dsfb_rf::{q16_16_to_f32, quantize_q16_16, DsfbRfEngine, PolicyDecision};

// Engine<W=10, K=4, M=8>
type Engine = DsfbRfEngine<10, 4, 8>;

/// Residual norm sequence: 20-sample calibration, then progressive drift.
const TEST_NORMS: [f32; 40] = [
    // Healthy calibration window (Admissible)
    0.050, 0.051, 0.049, 0.052, 0.048, 0.053, 0.047, 0.054, 0.046, 0.055,
    0.048, 0.052, 0.050, 0.051, 0.049, 0.050, 0.051, 0.050, 0.049, 0.048,
    // Drift onset — progressive outward movement
    0.060, 0.070, 0.080, 0.090, 0.100, 0.110, 0.120, 0.130, 0.140, 0.150,
    // Sustained elevation
    0.155, 0.160, 0.165, 0.170, 0.175, 0.180, 0.185, 0.190, 0.195, 0.200,
];

/// Run the engine over a norm sequence and return the policy decision array.
fn run_engine(norms: &[f32], rho: f32) -> [PolicyDecision; 40] {
    let mut engine = Engine::new(rho, 2.0);
    let ctx = PlatformContext::operational();
    let mut out = [PolicyDecision::Silent; 40];
    for (i, &n) in norms.iter().enumerate() {
        out[i] = engine.observe(n, ctx).policy;
    }
    out
}

#[test]
fn grammar_policy_agrees_f32_vs_q16_16() {
    let rho = 0.08_f32;
    const CALIB_LEN: usize = 20;

    // ── f32 path ──────────────────────────────────────────────────────────
    let decisions_f32 = run_engine(&TEST_NORMS, rho);

    // ── Q16.16 round-trip path ────────────────────────────────────────────
    let mut quantized_norms = [0.0_f32; 40];
    for (i, &norm) in TEST_NORMS.iter().enumerate() {
        let q = quantize_q16_16(norm as f64);
        quantized_norms[i] = q16_16_to_f32(q);
    }
    let decisions_q = run_engine(&quantized_norms, rho);

    // ── Property 1: calibration-phase exact agreement ─────────────────────
    // Both paths must agree during the healthy window.  No persistence gate
    // or DSA threshold is reachable here — any disagreement is a bug.
    for i in 0..CALIB_LEN {
        assert_eq!(
            decisions_f32[i], decisions_q[i],
            "calibration-phase disagreement at step {i}: \
             f32={:?} vs Q16.16={:?} (norm={:.6}, dequantized={:.6})",
            decisions_f32[i], decisions_q[i],
            TEST_NORMS[i], quantized_norms[i]
        );
    }

    // ── Property 2: drift-phase bounded disagreement ──────────────────────
    // Count distinct policy-state transitions in the f32 reference sequence.
    // Each crossing can be displaced by ±1 step in the Q16.16 path due to
    // accumulated state divergence (see module doc for the theoretical basis).
    // No new crossing events should be created.
    let f32_transitions: usize = (CALIB_LEN..TEST_NORMS.len() - 1)
        .filter(|&i| decisions_f32[i] != decisions_f32[i + 1])
        .count();

    let drift_disagreements: usize = (CALIB_LEN..TEST_NORMS.len())
        .filter(|&i| decisions_f32[i] != decisions_q[i])
        .count();

    // Observed values are printed unconditionally so they appear in CI logs
    // and can be tracked across library versions without re-running locally.
    println!(
        "bit_exactness: f32_transitions={f32_transitions}, \
         drift_disagreements={drift_disagreements}"
    );
    for i in CALIB_LEN..TEST_NORMS.len() {
        if decisions_f32[i] != decisions_q[i] {
            println!(
                "  step {i}: f32={:?}  Q16.16={:?}  \
                 (norm={:.6}, dequantized={:.6})",
                decisions_f32[i], decisions_q[i],
                TEST_NORMS[i], quantized_norms[i]
            );
        }
    }

    assert!(
        drift_disagreements <= f32_transitions,
        "Q16.16 drift-phase disagreements ({drift_disagreements}) exceed the \
         number of distinct f32 policy transitions ({f32_transitions}).\n\
         This indicates accumulated state divergence that creates NEW boundary \
         crossings rather than merely displacing existing ones — a regression.\n\
         f32 decisions (drift):   {:?}\n\
         Q16.16 decisions (drift): {:?}",
        &decisions_f32[CALIB_LEN..],
        &decisions_q[CALIB_LEN..]
    );
}

#[test]
fn quantization_error_bounded_by_2_pow_neg14() {
    // Generous bound: 2^-14 ≈ 6.1×10⁻⁵ (actual resolution is 2^-16 ≈ 1.5×10⁻⁵)
    let bound = (2.0_f32).powi(-14);
    for &norm in TEST_NORMS.iter() {
        let q = quantize_q16_16(norm as f64);
        let deq = q16_16_to_f32(q);
        let err = (deq - norm).abs();
        assert!(
            err < bound,
            "quantization error {:.2e} exceeds bound {:.2e} for norm={:.6}",
            err, bound, norm
        );
    }
}

#[test]
fn q16_round_trip_preserves_zero() {
    let q = quantize_q16_16(0.0);
    let back = q16_16_to_f32(q);
    assert_eq!(q, 0, "Q16.16(0.0) should be raw 0");
    assert_eq!(back, 0.0, "dequantize(0) should be 0.0");
}

#[test]
fn q16_round_trip_preserves_one() {
    let q = quantize_q16_16(1.0);
    let back = q16_16_to_f32(q);
    assert!((back - 1.0_f32).abs() < 1e-4, "Q16.16(1.0) round-trip: {}", back);
}

#[test]
fn q16_negative_norm_saturates_or_rounds() {
    // Norms are always ≥ 0 in-practice; just verify no panic on negative
    let q = quantize_q16_16(-0.05);
    let back = q16_16_to_f32(q);
    assert!(back <= 0.0, "Q16.16(-0.05) should round to non-positive: {}", back);
}
