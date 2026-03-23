//! Kani proof harnesses for bounded DSFB invariants.
//!
//! These harnesses deliberately cover small, explicit properties rather than implying whole-engine
//! formal verification.

#![cfg(kani)]

use dsfb_semiotics_engine::engine::types::{GrammarReasonCode, TrustScalar};

#[kani::proof]
fn proof_trust_scalar_in_unit_interval() {
    let value: f64 = kani::any();
    let trust = TrustScalar::new(value).value();
    assert!((0.0..=1.0).contains(&trust));
}

#[kani::proof]
fn proof_trust_scalar_not_nan() {
    let value: f64 = kani::any();
    let trust = TrustScalar::new(value).value();
    assert!(!trust.is_nan());
}

#[kani::proof]
fn proof_grammar_reason_enum_is_closed() {
    let codes = [
        GrammarReasonCode::Admissible,
        GrammarReasonCode::Boundary,
        GrammarReasonCode::RecurrentBoundaryGrazing,
        GrammarReasonCode::SustainedOutwardDrift,
        GrammarReasonCode::AbruptSlewViolation,
        GrammarReasonCode::EnvelopeViolation,
    ];
    for code in codes {
        match code {
            GrammarReasonCode::Admissible
            | GrammarReasonCode::Boundary
            | GrammarReasonCode::RecurrentBoundaryGrazing
            | GrammarReasonCode::SustainedOutwardDrift
            | GrammarReasonCode::AbruptSlewViolation
            | GrammarReasonCode::EnvelopeViolation => {}
        }
    }
}
/// Theorem 1 alignment: finite-time envelope exit under sustained outward drift.
///
/// The theorem states: if ‖r(t)‖ grows at rate ≥ α > 0 starting inside an envelope
/// of radius ρ, then exit occurs by t* ≤ t₀ + ρ/α.
///
/// This harness verifies the core arithmetic invariant: the exit-time upper bound
/// t* = ρ/α is finite and positive whenever the preconditions hold. It does not
/// verify the full engine pipeline — it verifies that the bound computation itself
/// is well-formed and cannot overflow or produce non-finite results under any
/// finite positive inputs.
///
/// Scope: this is a bounded arithmetic invariant, not a whole-engine proof.
/// The theorem's behavioral claim (that the engine detects exit) is validated
/// by integration tests; this harness verifies that the bound is computable.
#[kani::proof]
fn proof_theorem1_exit_bound_is_finite_and_positive() {
    // α: sustained outward drift rate (strictly positive by theorem precondition)
    let alpha: f64 = kani::any();
    kani::assume(alpha > 0.0);
    kani::assume(alpha.is_finite());
    kani::assume(alpha <= 1.0e6); // exclude astronomically large rates

    // ρ: envelope radius (strictly positive by EnvelopeSpec::validate precondition)
    let rho: f64 = kani::any();
    kani::assume(rho > 0.0);
    kani::assume(rho.is_finite());
    kani::assume(rho <= 1.0e6);

    // Theorem 1 exit-time upper bound: t* ≤ ρ / α
    let exit_bound = rho / alpha;

    // The bound must be finite and strictly positive
    assert!(exit_bound.is_finite());
    assert!(exit_bound > 0.0);

    // A trajectory starting inside the envelope (‖r₀‖ ≤ ρ) and growing at rate α
    // must cross the boundary before the bound.
    // Verified: initial norm inside envelope + drift * bound ≥ rho.
    let initial_norm: f64 = kani::any();
    kani::assume(initial_norm >= 0.0);
    kani::assume(initial_norm <= rho); // starts inside envelope
    kani::assume(initial_norm.is_finite());

    let norm_at_bound = initial_norm + alpha * exit_bound;
    assert!(norm_at_bound >= rho);
}

/// Companion invariant: inward drift cannot produce envelope exit.
///
/// Theorem 2 (Envelope Invariance): if drift is strictly inward (d/dt ‖r‖ ≤ -β < 0),
/// the trajectory cannot exit a fixed envelope of radius ρ from inside.
///
/// This harness verifies the arithmetic claim: norm decreasing at rate β from
/// any point inside the envelope never exceeds ρ within the interval.
///
/// Scope: arithmetic invariant only, same caveats as above.
#[kani::proof]
fn proof_theorem2_inward_drift_preserves_admissibility() {
    let rho: f64 = kani::any();
    kani::assume(rho > 0.0);
    kani::assume(rho.is_finite());
    kani::assume(rho <= 1.0e6);

    let initial_norm: f64 = kani::any();
    kani::assume(initial_norm >= 0.0);
    kani::assume(initial_norm <= rho);
    kani::assume(initial_norm.is_finite());

    // Inward drift rate (positive scalar, drift is -beta)
    let beta: f64 = kani::any();
    kani::assume(beta > 0.0);
    kani::assume(beta.is_finite());

    // For any t >= 0, norm(t) = initial_norm - beta * t <= initial_norm <= rho
    let t: f64 = kani::any();
    kani::assume(t >= 0.0);
    kani::assume(t.is_finite());
    kani::assume(t <= 1.0e6);

    let norm_at_t = (initial_norm - beta * t).max(0.0); // norm is non-negative
    assert!(norm_at_t <= rho);
}
