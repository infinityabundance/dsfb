//! Property tests for the `observe()` public API.
//!
//! These tests verify structural invariants of the DSFB observer that hold
//! regardless of dataset or configuration:
//!
//!   1. Constant signal  → drift is zero everywhere, all grammar "Admissible"
//!   2. NaN / imputed    → never produce a "Violation" grammar or non-Silent decision
//!   3. Determinism      → identical inputs always produce byte-identical output

use dsfb_semiconductor::observe;

// ─────────────────────────────────────────────────────────────────────────────
// 1. Constant signal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn constant_signal_has_zero_drift_and_admissible_grammar() {
    // A perfectly flat residual stream has no drift by definition.
    // All episodes must be grammar == "Admissible" and drift == 0.0.
    let residuals: Vec<f64> = vec![1.0; 50];
    let episodes = observe(&residuals);

    assert!(!episodes.is_empty(), "observe() must return at least one episode");

    for e in &episodes {
        assert_eq!(
            e.grammar, "Admissible",
            "constant signal index {} has grammar {:?}, expected \"Admissible\"",
            e.index, e.grammar
        );
        assert_eq!(
            e.drift, 0.0,
            "constant signal index {} has drift {:?}, expected 0.0",
            e.index, e.drift
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. NaN / imputed samples never produce Violation or non-Silent decisions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nan_samples_never_produce_violation_or_escalate() {
    // NaN values represent imputed (missing) data.  DSFB must suppress
    // Violation grammar and Escalate / Review decisions for imputed samples,
    // because imputed values carry no physical signal.
    let residuals: Vec<f64> = vec![f64::NAN; 30];
    let episodes = observe(&residuals);

    for e in &episodes {
        assert_ne!(
            e.grammar, "Violation",
            "imputed sample index {} produced grammar \"Violation\", which is forbidden",
            e.index
        );
        assert_eq!(
            e.decision, "Silent",
            "imputed sample index {} produced decision {:?}, expected \"Silent\"",
            e.index, e.decision
        );
    }
}

#[test]
fn inf_samples_never_produce_violation_or_escalate() {
    // ±Inf must be treated the same as NaN (imputed / invalid).
    let residuals: Vec<f64> = vec![f64::INFINITY, f64::NEG_INFINITY, f64::NAN, f64::INFINITY];
    let episodes = observe(&residuals);

    for e in &episodes {
        assert_ne!(
            e.grammar, "Violation",
            "inf/nan sample index {} produced grammar \"Violation\", which is forbidden",
            e.index
        );
        assert_eq!(
            e.decision, "Silent",
            "inf/nan sample index {} produced decision {:?}, expected \"Silent\"",
            e.index, e.decision
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Determinism: two calls on the same input produce identical output
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn observe_is_deterministic() {
    let residuals: &[f64] = &[
        0.10, 0.25, 0.52, 0.87, 1.40,
        1.95, 2.60, 3.30, 4.10, 4.95,
        f64::NAN, 0.30, 0.55, 0.80, 1.10,
    ];

    let first  = observe(residuals);
    let second = observe(residuals);

    assert_eq!(
        first.len(), second.len(),
        "second call returned a different number of episodes"
    );

    for (i, (a, b)) in first.iter().zip(second.iter()).enumerate() {
        assert_eq!(
            a.index, b.index,
            "episode {i}: index mismatch ({} vs {})", a.index, b.index
        );
        // Use bit-exact comparison for floating-point fields so any
        // non-determinism (e.g. random initialisation) is caught.
        assert_eq!(
            a.residual_norm_sq.to_bits(), b.residual_norm_sq.to_bits(),
            "episode {i}: residual_norm_sq not bit-identical"
        );
        assert_eq!(
            a.drift.to_bits(), b.drift.to_bits(),
            "episode {i}: drift not bit-identical"
        );
        assert_eq!(
            a.grammar, b.grammar,
            "episode {i}: grammar mismatch"
        );
        assert_eq!(
            a.decision, b.decision,
            "episode {i}: decision mismatch"
        );
    }
}
